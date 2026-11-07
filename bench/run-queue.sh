#!/usr/bin/env bash
# Run the workstation queue: every entry in bench/workstation-queue.txt (as on origin's master)
# whose ready/<recipe> tag is set, once each, in order, through bench/await-submit.sh. Then
# suspend, unless --no-suspend.
#
#   just workstation-queue [--no-suspend] [--hours N]
#
# --hours N waits up to N hours for each entry's ready tag (default 0: run what is ready
# now, and skip the rest). An entry that already ran here, or is on HOLD, is skipped.
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

# This run's queue and waiter, both from origin's master, fixed for the whole run.
queue=$(git show origin/master:bench/workstation-queue.txt) || { say "no queue on master"; exit 5; }
waiter=$(mktemp /tmp/await-submit.XXXXXX.sh)
git show origin/master:bench/await-submit.sh > "$waiter"

entries=$(grep -v '^[[:space:]]*#' <<<"$queue" | awk 'NF>=2 {print $1, $2}')
if [[ -z "$entries" ]]; then
  say "the queue is empty: nothing to run"
else
  say "queue:"; sed 's/^/  /' <<<"$entries"
fi

summary=(); worst=0
while read -r recipe branch; do
  [[ -z "$recipe" ]] && continue
  say "== $recipe ($branch)"
  AWAIT_SUBMIT_COPY=1 bash "$waiter" "$recipe" "$branch" --no-suspend --hours "$hours"
  rc=$?
  case $rc in
    0) what="ran: passed" ;; 1) what="ran: failed" ;; 3) what="on hold: not run" ;;
    4) what="not ready: not run" ;; 5) what="refused: not run" ;; *) what="exit $rc" ;;
  esac
  summary+=("$recipe: $what")
  (( rc == 1 )) && worst=1
done <<<"$entries"

git checkout --quiet master 2>/dev/null && git merge --quiet --ff-only origin/master 2>/dev/null
say "summary:"; for s in "${summary[@]}"; do say "  $s"; done
if ((suspend)); then
  say "suspending in 30 s (Ctrl-C to cancel)"; sleep 30
  systemctl suspend || say "systemctl suspend failed"
fi
exit $worst
