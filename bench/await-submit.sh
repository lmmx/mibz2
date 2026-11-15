#!/usr/bin/env bash
# Wait for the coordinator to mark a frozen recipe ready, run it once with `just submit`,
# then suspend the machine.
#
#   just submit-when-ready <recipe> <branch> [--no-suspend] [--hours N]
#   (or: bash bench/await-submit.sh <recipe> <branch> ...)
#
# The signal is a tag on origin. The coordinator pushes it only once the recipe is frozen,
# reviewed and green:
#   ready/<recipe>  points at the exact commit to run. The script requires that commit to
#                   be on <branch>, checks it out (detached), then runs
#                   `just submit <recipe>` once. So the branch may move on meanwhile.
#   hold/<recipe>   stop without running anything.
# It polls every 5 minutes, for at most --hours (default 8; 0 looks once and doesn't wait). Whatever the outcome (run passed,
# run failed, hold, timeout or a refusal), it suspends afterwards, unless --no-suspend.
# Nothing is written to the home directory itself. The waiter's state goes where the XDG
# Base Directory spec puts state, under ${XDG_STATE_HOME:-~/.local/state}/mibz2/workstation/:
#   logs/await-submit-<recipe>-<stamp>.log   everything this waiter prints;
#   ran/<recipe>-<sha>                       the once-per-commit markers.
# (Until 15 November 2026 these were ~/await-submit-*.log and ~/.cache/await-submit/; they have
# been moved, and markers in the old place are no longer read.)
#
# Guarantees:
# - a run happens at most once per recipe and commit on this machine: a marker directory
#   under ran/ is made atomically just before the run, so a restarted or second watcher
#   refuses;
# - HOLD is checked once more just before the run;
# - an annotated ready tag is resolved to its commit;
# - a failed check of origin is logged, and retried at the next poll;
# - the exit status is 0 only when the run ran and passed: 1 when it failed, 3 on HOLD,
#   4 on timeout, 5 on a refusal.
# Notices: when a run starts, and when one fails, a LOG is posted on #145 (the run's
# success is posted there by `just submit` with its bundle), so the coordinator sees the
# workstation pick work up without waiting for the bundle. A notice is best effort: one that
# can't be posted (gh signed out, no network) is logged and the run goes on.
# AWAIT_SUBMIT_NOTICES=0 turns them off.
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

# State, per the XDG Base Directory spec (a relative XDG_STATE_HOME is ignored, as it says)
state_home=${XDG_STATE_HOME:-}
[[ $state_home == /* ]] || state_home=$HOME/.local/state
state=$state_home/mibz2/workstation
mkdir -p "$state/logs" "$state/ran" || { echo "could not make $state"; exit 2; }
log=$state/logs/await-submit-${recipe}-$(date -u +%Y%m%dT%H%M%SZ).log
exec > >(tee -a "$log") 2>&1
say() { echo "[$(date -u +%H:%M:%S)] $*"; }

# Post a LOG on #145 (best effort, at most 60 s: a notice never stops or fails the run)
notice() {
  [[ ${AWAIT_SUBMIT_NOTICES:-1} == 1 ]] || return 0
  local gh=${GH:-gh} repo=${EVIDENCE_REPO:-lmmx/mibz2} body url
  body=$(printf 'FROM: louis via workstation\nTO: coordinator\nRE: %s\nTYPE: LOG\n\n%s\n' \
    "$recipe at ${ready:-?}" "$1")
  if url=$(timeout 60 "$gh" api "repos/$repo/issues/145/comments" -f "body=$body" --jq .html_url 2>/dev/null); then
    say "notice posted: $url"
  else
    say "notice not posted (gh failed or timed out); the run goes on"
  fi
  return 0
}

finish() {
  say "outcome: $1"
  if ((suspend)); then
    say "suspending in 30 s (Ctrl-C to cancel)"
    sleep 30
    systemctl suspend || say "systemctl suspend failed"
  fi
  exit "${2:-0}"
}

# Print origin's ready/hold refs, with annotated tags resolved to their commits.
# Fails, and says so, when origin can't be read.
query() {
  local out
  if ! out=$(git ls-remote --tags origin "refs/tags/ready/$recipe" "refs/tags/ready/$recipe^{}" \
               "refs/tags/hold/$recipe" 2>&1); then
    say "could not read origin's tags (retrying at the next poll): $out"
    return 1
  fi
  printf '%s\n' "$out"
}
ready_commit() {  # the peeled commit if the tag is annotated, else the tag's own target
  local peeled plain
  peeled=$(awk -v r="refs/tags/ready/$recipe^{}" '$2==r {print $1}' <<<"$1")
  plain=$(awk -v r="refs/tags/ready/$recipe" '$2==r {print $1}' <<<"$1")
  echo "${peeled:-$plain}"
}

cd "$(git rev-parse --show-toplevel)" || { say "not in a git checkout"; exit 2; }
say "waiting for ready/$recipe or hold/$recipe on origin (every 5 min, up to ${hours} h); log $log"

deadline=$(( $(date +%s) + hours * 3600 ))
ready=""
while :; do
  if refs=$(query); then
    if grep -q "refs/tags/hold/$recipe\$" <<<"$refs"; then
      finish "HOLD: the coordinator put the run on hold; nothing was run" 3
    fi
    ready=$(ready_commit "$refs")
  fi
  [[ -n "$ready" ]] && break
  (( $(date +%s) + 300 <= deadline )) || break
  sleep 300
done
[[ -z "$ready" ]] && finish "TIMEOUT: no ready/$recipe after ${hours} h; nothing was run" 4
say "ready/$recipe -> $ready"

if [[ -n "$(git status --porcelain)" ]]; then
  git status --short
  finish "REFUSED: the checkout has uncommitted changes; nothing was run" 5
fi
git fetch origin "$branch" || finish "REFUSED: could not fetch $branch" 5
git merge-base --is-ancestor "$ready" "origin/$branch" 2>/dev/null \
  || finish "REFUSED: the ready commit $ready is not on $branch; nothing was run" 5
git checkout --quiet --detach "$ready" || finish "REFUSED: could not check out $ready" 5
head=$(git rev-parse HEAD)
[[ "$head" == "$ready" ]] || finish "REFUSED: HEAD is $head, not the ready commit $ready; nothing was run" 5

# HOLD once more, just before the run.
if refs=$(query) && grep -q "refs/tags/hold/$recipe\$" <<<"$refs"; then
  finish "HOLD: the coordinator put the run on hold before it started; nothing was run" 3
fi
# At most once per recipe and commit on this machine (mkdir is atomic).
mark=$state/ran/$recipe-$ready
if ! mkdir "$mark" 2>/dev/null; then
  finish "REFUSED: $recipe at $ready already ran on this machine ($mark); nothing was run" 5
fi

say "running: just submit $recipe"
started=$(date -u +%Y-%m-%dT%H:%M:%SZ)
notice "**Started:** \`just submit $recipe\` at \`$ready\` ($branch), $started, on $(lscpu 2>/dev/null | sed -n 's/^Model name: *//p' | head -1). Its bundle is posted here when it ends; a failure is posted too."
if just submit "$recipe"; then
  finish "RUN DONE: just submit $recipe exited 0" 0
else
  rc=$?
  notice "**Failed:** \`just submit $recipe\` at \`$ready\` exited $rc, started $started, ended $(date -u +%Y-%m-%dT%H:%M:%SZ). The waiter's log on the workstation: \`$log\`. A bundle, if the recipe made one, is posted separately."
  finish "RUN FAILED: just submit $recipe exited $rc" 1
fi
