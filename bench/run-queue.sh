#!/usr/bin/env bash
# Run the workstation queue: every entry in bench/workstation-queue.txt (as on origin's master)
# whose ready/<recipe> tag is set, once each, in order, through bench/await-submit.sh. Then
# suspend, unless --no-suspend.
#
#   just workstation-queue [--no-suspend] [--hours N]
#
# --hours N waits up to N hours for each entry's ready tag (default 0: run what is ready
# now, and skip the rest). An entry that already ran here, or is on HOLD, is skipped.
#
# Work added while the queue runs is run too. After each pass, the queue and the ready tags
# are read again from origin. Another pass runs every entry whose ready commit has not yet
# run on this machine, such as a rerun the coordinator queued after reading an earlier
# bundle. The queue suspends only after a pass that finds nothing new (at most 10 passes).
# The run's log is ~/workstation-queue-<stamp>.log; each entry also keeps its own log.
set -uo pipefail

if [[ "${RUN_QUEUE_COPY:-}" != 1 ]]; then
  tmp=$(mktemp /tmp/run-queue.XXXXXX.sh); cp "$0" "$tmp"
  RUN_QUEUE_COPY=1 exec bash "$tmp" "$@"
fi

suspend=1; hours=0
while (($#)); do
  case $1 in
    --no-suspend) suspend=0 ;;
    --hours) hours=$2; shift ;;
    *) echo "unknown option $1"; exit 2 ;;
  esac
  shift
done

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
# (no marker under ~/.cache/await-submit, which the waiter makes just before a run).
new_entries() {
  local recipe branch refs ready
  while read -r recipe branch; do
    [[ -z "$recipe" ]] && continue
    refs=$(git ls-remote --tags origin "refs/tags/ready/$recipe" "refs/tags/ready/$recipe^{}" \
             "refs/tags/hold/$recipe" 2>/dev/null) || continue
    grep -q "refs/tags/hold/$recipe\$" <<<"$refs" && continue
    ready=$(awk -v p="refs/tags/ready/$recipe^{}" -v r="refs/tags/ready/$recipe" \
              '$2==p {pk=$1} $2==r {pl=$1} END {print (pk ? pk : pl)}' <<<"$refs")
    [[ -n "$ready" && ! -e ~/.cache/await-submit/$recipe-$ready ]] && echo "$recipe $branch"
  done <<<"$entries"
}
run_entries() {  # run each "recipe branch" line of $1 through the waiter
  local recipe branch rc what
  while read -r recipe branch; do
    [[ -z "$recipe" ]] && continue
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

summary=(); worst=0
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

git checkout --quiet master 2>/dev/null && git merge --quiet --ff-only origin/master 2>/dev/null
say "summary:"; for s in "${summary[@]}"; do say "  $s"; done
if ((suspend)); then
  say "suspending in 30 s (Ctrl-C to cancel)"; sleep 30
  systemctl suspend || say "systemctl suspend failed"
fi
exit $worst
