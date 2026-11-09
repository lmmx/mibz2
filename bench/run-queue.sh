#!/usr/bin/env bash
# Run the workstation queue: every entry in bench/workstation-queue.txt (as on origin's master)
# whose ready/<recipe> tag is set, once each, in order, through bench/await-submit.sh. Then
# suspend, unless --no-suspend.
#
#   just workstation-queue [--no-suspend] [--hours N] [--wait H]
#
# --hours N waits up to N hours for each entry's ready tag (default 0: run what is ready
# now, and skip the rest). An entry that already ran here, or is on HOLD, is skipped.
#
# Work added while the queue runs is run too. After each pass, the queue and the ready tags
# are read again from origin. Another pass runs every entry whose ready commit has not yet
# run on this machine, such as a rerun the coordinator queued after reading an earlier
# bundle. The queue suspends only after a pass that finds nothing new (at most 10 passes).
#
# --wait H keeps the queue open for new work: once nothing is left to run, it checks origin
# every 5 minutes and runs any entry that becomes ready, until H hours pass with nothing new
# (H may be fractional, such as 1.5). Each run restarts that clock. An empty queue waits too,
# so it can be started before an entry is admitted. An entry refused at one commit (a dirty
# checkout, say) is not retried at that commit. Without --wait, nothing waits.
# The run's log is ~/workstation-queue-<stamp>.log; each entry also keeps its own log.
set -uo pipefail

if [[ "${RUN_QUEUE_COPY:-}" != 1 ]]; then
  tmp=$(mktemp /tmp/run-queue.XXXXXX.sh); cp "$0" "$tmp"
  RUN_QUEUE_COPY=1 exec bash "$tmp" "$@"
fi

suspend=1; hours=0; wait=0
poll=${RUN_QUEUE_POLL:-300}  # seconds between checks under --wait
while (($#)); do
  case $1 in
    --no-suspend) suspend=0 ;;
    --hours) hours=$2; shift ;;
    --wait) wait=$2; shift ;;
    *) echo "unknown option $1"; exit 2 ;;
  esac
  shift
done
wait_secs=$(awk -v h="$wait" 'BEGIN { if (h !~ /^[0-9]*\.?[0-9]+$/) exit 1; printf "%d", h * 3600 }') \
  || { echo "--wait takes a number of hours, such as 2 or 1.5"; exit 2; }

log=~/workstation-queue-$(date -u +%Y%m%dT%H%M%SZ).log
exec > >(tee -a "$log") 2>&1
say() { echo "[$(date -u +%H:%M:%S)] $*"; }

cd "$(git rev-parse --show-toplevel)" || { say "not in a git checkout"; exit 2; }
[[ -z "$(git status --porcelain)" ]] || { git status --short; say "REFUSED: the checkout has uncommitted changes; nothing was run"; exit 5; }
git fetch --quiet origin master || { say "could not fetch master"; exit 5; }

# Each pass reads the queue and the waiter afresh from origin's master.
read_queue() {
  git fetch --quiet origin master || { say "could not fetch master"; return 1; }
  queue=$(git show origin/master:bench/workstation-queue.txt) || { say "no queue on master"; return 1; }
  git show origin/master:bench/await-submit.sh > "$waiter" || return 1
  entries=$(grep -v '^[[:space:]]*#' <<<"$queue" | awk 'NF>=2 {print $1, $2}')
}
# The entries whose ready tag is set, with no hold, at a commit not yet run on this machine
# (no marker under ~/.cache/await-submit, which the waiter makes just before a run), and not
# already tried at that commit by this queue run.
# The commit ready/<recipe> names, or nothing when it is unset or hold/<recipe> is set.
ready_of() {
  local refs
  refs=$(git ls-remote --tags origin "refs/tags/ready/$1" "refs/tags/ready/$1^{}" \
           "refs/tags/hold/$1" 2>/dev/null) || return 0
  grep -q "refs/tags/hold/$1\$" <<<"$refs" && return 0
  awk -v p="refs/tags/ready/$1^{}" -v r="refs/tags/ready/$1" \
      '$2==p {pk=$1} $2==r {pl=$1} END {print (pk ? pk : pl)}' <<<"$refs"
}
new_entries() {
  local recipe branch ready
  while read -r recipe branch; do
    [[ -z "$recipe" ]] && continue
    ready=$(ready_of "$recipe")
    [[ -n "$ready" && ! -e ~/.cache/await-submit/$recipe-$ready ]] || continue
    [[ -n "${tried[$recipe-$ready]:-}" ]] && continue
    echo "$recipe $branch $ready"
  done <<<"$entries"
}
run_entries() {  # run each "recipe branch" line of $1 through the waiter
  local recipe branch ready rc what
  while read -r recipe branch ready; do
    [[ -z "$recipe" ]] && continue
    [[ -z "$ready" ]] && ready=$(ready_of "$recipe")
    [[ -n "$ready" ]] && tried[$recipe-$ready]=1
    say "== $recipe ($branch)"
    # stdin from /dev/null, so that nothing the run starts can read this loop's entries
    AWAIT_SUBMIT_COPY=1 bash "$waiter" "$recipe" "$branch" --no-suspend --hours "$hours" </dev/null
    rc=$?
    case $rc in
      0) what="ran: passed" ;; 1) what="ran: failed" ;; 3) what="on hold: not run" ;;
      4) what="not ready: not run" ;; 5) what="refused: not run" ;; *) what="exit $rc" ;;
    esac
    summary+=("$recipe: $what")
    (( rc == 1 )) && worst=1
    (( rc == 0 || rc == 1 )) && ran=1
  done <<<"$1"
}

waiter=$(mktemp /tmp/await-submit.XXXXXX.sh)
read_queue || exit 5
if [[ -z "$entries" ]]; then
  say "the queue is empty: nothing to run"
else
  say "queue:"; sed 's/^/  /' <<<"$entries"
fi

summary=(); worst=0; declare -A tried=()
ran=0
run_entries "$entries"
# Later passes: whatever was queued or re-tagged meanwhile, until a pass finds nothing new.
pass=1
while (( ran && pass < 10 )); do
  ran=0; pass=$((pass + 1))
  read_queue || break
  more=$(new_entries)
  [[ -z "$more" ]] && { say "pass $pass: nothing new on origin"; break; }
  say "pass $pass: new on origin:"; sed 's/^/  /' <<<"$more"
  run_entries "$more"
done

# --wait: keep checking origin until H hours pass with nothing new to run.
if (( wait_secs > 0 )); then
  deadline=$(( $(date +%s) + wait_secs ))
  say "waiting up to $wait h for new work (checking origin every $(( poll / 60 )) min)"
  while (( $(date +%s) + poll <= deadline )); do
    sleep "$poll"
    read_queue || continue
    more=$(new_entries)
    [[ -z "$more" ]] && continue
    say "new on origin:"; sed 's/^/  /' <<<"$more"
    run_entries "$more"
    deadline=$(( $(date +%s) + wait_secs ))
    say "waiting up to $wait h more for new work"
  done
  say "nothing new for $wait h: done waiting"
fi

git checkout --quiet master 2>/dev/null && git merge --quiet --ff-only origin/master 2>/dev/null
say "summary:"; for s in "${summary[@]}"; do say "  $s"; done
if ((suspend)); then
  say "suspending in 30 s (Ctrl-C to cancel)"; sleep 30
  systemctl suspend || say "systemctl suspend failed"
fi
exit $worst
