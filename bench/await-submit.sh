#!/usr/bin/env bash
# Wait for the coordinator to mark a frozen recipe ready, run it once with `just submit`,
# then suspend the machine.
#
#   bash bench/await-submit.sh <recipe> <branch> [--no-suspend] [--hours N]
#
# The signal is a tag on origin. The coordinator pushes it only once the recipe is frozen,
# reviewed and green:
#   ready/<recipe>  points at the exact commit to run. The script checks out <branch>,
#                   requires its head to be exactly that commit, then runs
#                   `just submit <recipe>` once.
#   hold/<recipe>   stop without running anything.
# It polls every 5 minutes, for at most --hours (default 8). Whatever the outcome (run passed,
# run failed, hold, timeout or a refusal), it suspends afterwards, unless --no-suspend.
# Everything is logged to ~/await-submit-<recipe>-<stamp>.log.
set -uo pipefail

# Run from a copy, so that checking out another branch can't change this script mid-run.
if [[ "${AWAIT_SUBMIT_COPY:-}" != 1 ]]; then
  tmp=$(mktemp /tmp/await-submit.XXXXXX.sh)
  cp "$0" "$tmp"
  AWAIT_SUBMIT_COPY=1 exec bash "$tmp" "$@"
fi

recipe=${1:?usage: await-submit.sh <recipe> <branch> [--no-suspend] [--hours N]}
branch=${2:?usage: await-submit.sh <recipe> <branch> [--no-suspend] [--hours N]}
shift 2
suspend=1
hours=8
while (($#)); do
  case $1 in
    --no-suspend) suspend=0 ;;
    --hours) hours=$2; shift ;;
    *) echo "unknown option $1"; exit 2 ;;
  esac
  shift
done

log=~/await-submit-${recipe}-$(date -u +%Y%m%dT%H%M%SZ).log
exec > >(tee -a "$log") 2>&1
say() { echo "[$(date -u +%H:%M:%S)] $*"; }

finish() {
  say "outcome: $1"
  if ((suspend)); then
    say "suspending in 30 s (Ctrl-C to cancel)"
    sleep 30
    systemctl suspend || say "systemctl suspend failed"
  fi
  exit 0
}

cd "$(git rev-parse --show-toplevel)" || { say "not in a git checkout"; exit 2; }
say "waiting for ready/$recipe or hold/$recipe on origin (every 5 min, up to ${hours} h); log $log"

deadline=$(( $(date +%s) + hours * 3600 ))
ready=""
while (( $(date +%s) < deadline )); do
  refs=$(git ls-remote --tags origin "refs/tags/ready/$recipe" "refs/tags/hold/$recipe" 2>/dev/null)
  if grep -q "refs/tags/hold/$recipe" <<<"$refs"; then
    finish "HOLD: the coordinator put the run on hold; nothing was run"
  fi
  ready=$(awk -v r="refs/tags/ready/$recipe" '$2==r {print $1}' <<<"$refs")
  [[ -n "$ready" ]] && break
  sleep 300
done
[[ -z "$ready" ]] && finish "TIMEOUT: no ready/$recipe after ${hours} h; nothing was run"
say "ready/$recipe -> $ready"

if [[ -n "$(git status --porcelain)" ]]; then
  git status --short
  finish "REFUSED: the checkout has uncommitted changes; nothing was run"
fi
git fetch origin "$branch" || finish "REFUSED: could not fetch $branch"
git checkout "$branch" && git merge --ff-only "origin/$branch" || finish "REFUSED: could not check out $branch"
head=$(git rev-parse HEAD)
if [[ "$head" != "$ready" ]]; then
  finish "REFUSED: $branch is at $head, not the ready commit $ready; nothing was run"
fi

say "running: just submit $recipe"
if just submit "$recipe"; then
  finish "RUN DONE: just submit $recipe exited 0"
else
  finish "RUN FAILED: just submit $recipe exited $?"
fi
