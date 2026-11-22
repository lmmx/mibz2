#!/usr/bin/env bash
# Run the workstation queue: every entry in bench/workstation-queue.txt (as on origin's default
# branch) whose ready/<recipe> tag is set, once each, in order, through bench/await-submit.sh.
# Then suspend, unless --no-suspend. One runner can serve several repositories' queues at once.
#
#   just workstation-queue [--repo PATH]... [--no-suspend] [--hours N] [--wait H]
#
# Which repositories: each --repo PATH (repeatable), else the checkouts listed one absolute path
# a line in ${XDG_CONFIG_HOME:-~/.config}/workstation/repos (blank lines and # comments are
# ignored), else the current checkout only. Each repository keeps its own queue file, its own
# ready/<recipe> and hold/<recipe> tags on its own origin, its own evidence branch and channel:
# an entry runs in its own checkout through that repository's own bench/await-submit.sh (read
# from its origin's default branch) and its own `just submit`. Nothing is shared but the machine.
#
# An entry is a line "<recipe> <branch> [args=A,B] <what it measures>": the optional args= column
# is passed to the waiter as --args A,B (as wikidata-pq's queue uses it).
#
# --hours N waits up to N hours for each entry's ready tag (default 0: run what is ready
# now, and skip the rest). An entry that already ran here, or is on HOLD, is skipped.
#
# Sharing the machine: the repositories take turns, in the order listed. In its turn a
# repository runs at most one entry (it skips past entries that are not ready, on hold or
# already run), then the next repository has its turn, until every repository's list for the
# pass is done. So a busy repository can't starve another.
#
# Work added while the queue runs is run too. After each pass, every repository's queue and
# ready tags are read again from its origin. Another pass runs every entry whose ready commit
# has not yet run on this machine, such as a rerun queued after an earlier bundle was read.
# The queue suspends only after a pass where no repository has anything new (at most 10
# passes in all).
#
# --wait H keeps the queue open for new work: once nothing is left to run, it checks every
# origin every 5 minutes and runs any entry that becomes ready, until H hours pass with nothing
# new (H may be fractional, such as 1.5). Each run restarts that clock. An empty queue waits
# too, so it can be started before an entry is admitted. An entry refused at one commit (a
# dirty checkout, say) is not retried at that commit. Without --wait, nothing waits.
#
# One repository's trouble never stops the others: a checkout that is missing, dirty or whose
# origin can't be read is logged and left out (for the run, or for that pass), and a failed
# recipe is logged and the turns go on. The exit status is 1 if any run failed, 5 if no
# repository could be served at all (or the lock is held), else 0.
#
# One run at a time on the machine: the runner holds an flock on
# ${XDG_STATE_HOME:-~/.local/state}/workstation/queue.lock for the whole run, and refuses to
# start while anything else holds it. bench/await-submit.sh and `just submit` take the same
# lock (or inherit this runner's), so a second runner, or a stray `just submit`, can't overlap
# a queue run. Any other runner on the machine shares the machine safely only if it takes the
# same lock.
#
# State, per the XDG Base Directory spec (nothing is written to the home directory itself),
# stays per repository, under ${XDG_STATE_HOME:-~/.local/state}/<repo>/workstation/, where
# <repo> is the name of the checkout's origin repository: each entry's log in logs/, and
# bench/await-submit.sh's once-per-commit markers in ran/. The run's own log is
# logs/workstation-queue-<stamp>.log there when one repository is served, and under
# ${XDG_STATE_HOME:-~/.local/state}/workstation/logs/ when a list of repositories is.
#
# Nothing here names a repository: the file is the same in every repository that uses it.
set -uo pipefail

if [[ "${RUN_QUEUE_COPY:-}" != 1 ]]; then
  tmp=$(mktemp /tmp/run-queue.XXXXXX.sh); cp "$0" "$tmp"
  RUN_QUEUE_COPY=1 exec bash "$tmp" "$@"
fi

suspend=1; hours=0; wait=0; repo_args=()
poll=${RUN_QUEUE_POLL:-300}  # seconds between checks under --wait
while (($#)); do
  case $1 in
    --no-suspend) suspend=0 ;;
    --hours) hours=$2; shift ;;
    --wait) wait=$2; shift ;;
    --repo) repo_args+=("$2"); shift ;;
    *) echo "unknown option $1"; exit 2 ;;
  esac
  shift
done
wait_secs=$(awk -v h="$wait" 'BEGIN { if (h !~ /^[0-9]*\.?[0-9]+$/) exit 1; printf "%d", h * 3600 }') \
  || { echo "--wait takes a number of hours, such as 2 or 1.5"; exit 2; }

# State and config, per the XDG Base Directory spec (a relative path is ignored, as it says)
state_home=${XDG_STATE_HOME:-}
[[ $state_home == /* ]] || state_home=$HOME/.local/state
config_home=${XDG_CONFIG_HOME:-}
[[ $config_home == /* ]] || config_home=$HOME/.config
config=$config_home/workstation/repos

# The machine-wide lock, held (fd 9) until this runner exits; its children inherit it.
lock=$state_home/workstation/queue.lock
mkdir -p "${lock%/*}" || { echo "could not make ${lock%/*}"; exit 2; }
exec 9>>"$lock" || { echo "could not open $lock"; exit 2; }
if ! flock -n 9; then
  echo "REFUSED: another workstation run holds the machine lock $lock" \
       "($(cat "$lock" 2>/dev/null || echo "holder unknown")); nothing was run." \
       "Wait for it to end, or stop it first."
  exit 5
fi
echo "pid $$, run-queue.sh, since $(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$lock"
export WORKSTATION_LOCK_FD=9

# The repositories: --repo, else the config file, else the current checkout.
multi=1
if ((${#repo_args[@]})); then
  repos=("${repo_args[@]}")
elif [[ -f $config ]]; then
  mapfile -t repos < <(sed -e 's/[[:space:]]*#.*//' -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' "$config" | grep .)
else
  multi=0
  top=$(git rev-parse --show-toplevel 2>/dev/null) || { echo "not in a git checkout (and no --repo, nor $config)"; exit 2; }
  repos=("$top")
fi

# The name of a checkout's origin repository (the last part of its URL, less .git), and its
# owner/name (the last two parts; a scp-like git@host:owner/name URL counts its colon as a /)
origin_name() { local u; u=$(git -C "$1" remote get-url origin 2>/dev/null) || return 1
  u=${u%/}; u=${u%.git}; u=${u##*/}; [[ -n $u ]] && echo "$u"; }

if ((multi)); then
  log_dir=$state_home/workstation/logs
else
  name=$(origin_name "${repos[0]}") || { echo "${repos[0]} has no origin remote"; exit 2; }
  log_dir=$state_home/$name/workstation/logs
fi
mkdir -p "$log_dir" || { echo "could not make $log_dir"; exit 2; }
log=$log_dir/workstation-queue-$(date -u +%Y%m%dT%H%M%SZ).log
exec > >(tee -a "$log" 9>&-) 2>&1
say() { echo "[$(date -u +%H:%M:%S)] $*"; }
indent() { awk '{printf "  %s %s", $1, $2; if ($3 != "-") printf " args=%s", $3; if ($4 != "") printf " %s", $4; print ""}'; }

# Per repository i: its checkout, name, state directory, origin's default branch, waiter
# (a copy of its bench/await-submit.sh from that branch), queue entries, and this pass's to-do.
R_DIR=(); R_NAME=(); R_STATE=(); R_BRANCH=(); R_WAITER=(); R_ENTRIES=(); R_TODO=()
summary=(); worst=0; ran=0; served=0; declare -A tried=() seen=()
label() { ((multi)) && printf '[%s] ' "${R_NAME[$1]}"; return 0; }
refuse_repo() { say "REFUSED: $1"; summary+=("$1"); }

for path in "${repos[@]}"; do
  if [[ $path != /* ]]; then refuse_repo "$path: not an absolute path; left out"; continue; fi
  if ! dir=$(git -C "$path" rev-parse --show-toplevel 2>/dev/null); then
    refuse_repo "$path: not a git checkout; left out"; continue
  fi
  if ! name=$(origin_name "$dir"); then refuse_repo "$dir: no origin remote; left out"; continue; fi
  if [[ -n ${seen[$name]:-} ]]; then
    refuse_repo "$dir: $name is already served from ${seen[$name]}; left out"; continue
  fi
  if [[ -n "$(git -C "$dir" status --porcelain)" ]]; then
    git -C "$dir" status --short
    if ((multi)); then
      refuse_repo "$name: the checkout $dir has uncommitted changes; none of its entries were run"
    else
      say "REFUSED: the checkout has uncommitted changes; nothing was run"
    fi
    continue
  fi
  # origin's default branch: as the clone recorded it, else as origin says, else master
  branch=$(git -C "$dir" symbolic-ref --quiet --short refs/remotes/origin/HEAD 2>/dev/null)
  branch=${branch#origin/}
  [[ -n $branch ]] || branch=$(git -C "$dir" ls-remote --symref origin HEAD 2>/dev/null \
    | awk '$1 == "ref:" {sub("refs/heads/", "", $2); print $2; exit}')
  [[ -n $branch ]] || branch=master
  i=${#R_DIR[@]}
  seen[$name]=$dir
  R_DIR[i]=$dir; R_NAME[i]=$name; R_STATE[i]=$state_home/$name/workstation; R_BRANCH[i]=$branch
  R_WAITER[i]=$(mktemp /tmp/await-submit.XXXXXX.sh); R_ENTRIES[i]=""; R_TODO[i]=""
done
((multi)) && say "serving ${#R_DIR[@]} of ${#repos[@]} repositories (machine lock $lock)"

# Read repository i's queue and waiter afresh from its origin's default branch.
read_queue() {
  local i=$1 queue
  cd "${R_DIR[i]}" || { say "$(label "$i")could not enter ${R_DIR[i]}"; return 1; }
  git fetch --quiet origin "${R_BRANCH[i]}" || { say "$(label "$i")could not fetch ${R_BRANCH[i]}"; return 1; }
  queue=$(git show "origin/${R_BRANCH[i]}:bench/workstation-queue.txt") \
    || { say "$(label "$i")no queue on ${R_BRANCH[i]}"; return 1; }
  git show "origin/${R_BRANCH[i]}:bench/await-submit.sh" > "${R_WAITER[i]}" \
    || { say "$(label "$i")no bench/await-submit.sh on ${R_BRANCH[i]}"; return 1; }
  # grep exits 1 on a queue of comments only; that is an empty queue, not a failure
  # each entry as "recipe branch args", args being - when the line has no args= column
  R_ENTRIES[i]=$( { grep -v '^[[:space:]]*#' <<<"$queue" || true; } \
    | awk 'NF>=2 {a = ($3 ~ /^args=./) ? substr($3, 6) : "-"; print $1, $2, a}')
}
# The commit ready/<recipe> names, or nothing when it is unset or hold/<recipe> is set
# (run in the repository's checkout).
ready_of() {
  local refs
  refs=$(git ls-remote --tags origin "refs/tags/ready/$1" "refs/tags/ready/$1^{}" \
           "refs/tags/hold/$1" 2>/dev/null) || return 0
  grep -q "refs/tags/hold/$1\$" <<<"$refs" && return 0
  awk -v p="refs/tags/ready/$1^{}" -v r="refs/tags/ready/$1" \
      '$2==p {pk=$1} $2==r {pl=$1} END {print (pk ? pk : pl)}' <<<"$refs"
}
# Repository i's entries whose ready tag is set, with no hold, at a commit not yet run on this
# machine (no marker in its ran/, which the waiter makes just before a run), and not already
# tried at that commit by this queue run.
new_entries() {
  local i=$1 recipe branch args ready
  cd "${R_DIR[i]}" || return 0
  while read -r recipe branch args; do
    [[ -z "$recipe" ]] && continue
    ready=$(ready_of "$recipe")
    [[ -n "$ready" && ! -e ${R_STATE[i]}/ran/$recipe-$ready ]] || continue
    [[ -n "${tried[$i $recipe-$ready]:-}" ]] && continue
    echo "$recipe $branch $args $ready"
  done <<<"${R_ENTRIES[i]}"
}
# Run one "recipe branch args [ready]" line of repository i through its waiter; 0 if it ran.
run_one() {
  local i=$1 recipe branch args ready rc what with=()
  read -r recipe branch args ready <<<"$2"
  [[ -n $args && $args != - ]] && with=(--args "$args")
  [[ -z "$recipe" ]] && return 1
  cd "${R_DIR[i]}" || { summary+=("$(label "$i")$recipe: refused: could not enter ${R_DIR[i]}"); return 1; }
  [[ -z "$ready" ]] && ready=$(ready_of "$recipe")
  [[ -n "$ready" ]] && tried[$i $recipe-$ready]=1
  say "== $(label "$i")$recipe ($branch)"
  # stdin from /dev/null, so that nothing the run starts can read this loop's entries
  AWAIT_SUBMIT_COPY=1 bash "${R_WAITER[i]}" "$recipe" "$branch" --no-suspend --hours "$hours" "${with[@]}" </dev/null
  rc=$?
  case $rc in
    0) what="ran: passed" ;; 1) what="ran: failed" ;; 3) what="on hold: not run" ;;
    4) what="not ready: not run" ;; 5) what="refused: not run" ;; *) what="exit $rc" ;;
  esac
  summary+=("$(label "$i")$recipe: $what")
  (( rc == 1 )) && worst=1
  (( rc == 0 || rc == 1 )) || return 1
  ran=1
}
# One pass over every repository's R_TODO: in turn, each runs entries until one runs (or its
# list is done), then the next has its turn; round after round until every list is empty.
run_pass() {
  local i line left=1
  while ((left)); do
    left=0
    for i in "${!R_DIR[@]}"; do
      while [[ -n ${R_TODO[i]} ]]; do
        line=${R_TODO[i]%%$'\n'*}
        if [[ ${R_TODO[i]} == *$'\n'* ]]; then R_TODO[i]=${R_TODO[i]#*$'\n'}; else R_TODO[i]=""; fi
        run_one "$i" "$line" && break
      done
      [[ -n ${R_TODO[i]} ]] && left=1
    done
  done
}
# Read every repository afresh; set each R_TODO to its new entries. 0 if any has something new.
gather_new() {
  local i any=1
  for i in "${!R_DIR[@]}"; do
    R_TODO[i]=""
    read_queue "$i" || { say "$(label "$i")left out of this pass"; continue; }
    R_TODO[i]=$(new_entries "$i")
    if [[ -n ${R_TODO[i]} ]]; then
      any=0; say "$1 $(label "$i")new on origin:"; indent <<<"${R_TODO[i]}"
    fi
  done
  return $any
}

# The first pass: every entry of every repository's queue, through its waiter.
for i in "${!R_DIR[@]}"; do
  if ! read_queue "$i"; then
    ((multi)) || exit 5
    refuse_repo "${R_NAME[i]}: its queue could not be read; none of its entries were run"
    continue
  fi
  served=1
  if [[ -z "${R_ENTRIES[i]}" ]]; then
    say "$(label "$i")the queue is empty: nothing to run"
  else
    say "$(label "$i")queue:"; indent <<<"${R_ENTRIES[i]}"
  fi
  R_TODO[i]=${R_ENTRIES[i]}
done
if ! ((served)); then
  say "no repository could be served: nothing was run"
  for s in "${summary[@]}"; do say "  $s"; done
  exit 5
fi
run_pass

# Later passes: whatever was queued or re-tagged meanwhile, until a pass finds nothing new.
pass=1
while (( ran && pass < 10 )); do
  ran=0; pass=$((pass + 1))
  gather_new "pass $pass:" || { say "pass $pass: nothing new on origin"; break; }
  run_pass
done

# --wait: keep checking every origin until H hours pass with nothing new to run.
if (( wait_secs > 0 )); then
  deadline=$(( $(date +%s) + wait_secs ))
  say "waiting up to $wait h for new work (checking origin every $(( poll / 60 )) min)"
  while (( $(date +%s) + poll <= deadline )); do
    sleep "$poll"
    gather_new "" || continue
    run_pass
    deadline=$(( $(date +%s) + wait_secs ))
    say "waiting up to $wait h more for new work"
  done
  say "nothing new for $wait h: done waiting"
fi

for i in "${!R_DIR[@]}"; do
  b=${R_BRANCH[i]}
  git -C "${R_DIR[i]}" checkout --quiet "$b" 2>/dev/null \
    && git -C "${R_DIR[i]}" merge --quiet --ff-only "origin/$b" 2>/dev/null
  rm -f "${R_WAITER[i]}"
done
say "summary:"; for s in "${summary[@]}"; do say "  $s"; done
if ((suspend)); then
  say "suspending in 30 s (Ctrl-C to cancel)"; sleep 30
  systemctl suspend || say "systemctl suspend failed"
fi
exit $worst
