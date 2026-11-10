#!/usr/bin/env bash
# bench-par-scan's finalisation (docs/journal/2026-11-10-parallel-scan.md, section 10): the
# outcome written, then the attempt's directory bundled under a deadline. A bundle exists as
# DIR.tar.gz only if the bundling step succeeded, so `just submit` (which takes the one new
# target/*/*.tar.gz) has nothing to submit after a failed finalisation.
#   bash bench/par_scan_finish.sh DIR OUTCOME SECS BUNDLER [ARG...]
# - OUTCOME goes to DIR/outcome.txt, then BUNDLER runs under bench/bounded.py with a deadline
#   of SECS seconds (its group stopped and checked as there).
# - If it fails or passes its deadline, no VALID or DRY outcome survives: outcome.txt is
#   rewritten as INVALID, naming the failure and the outcome it replaces;
#   FINALISATION-FAILED.txt records the same; and any DIR.tar.gz (or its .tmp) is renamed
#   DIR.tar.gz.failed, which neither `just submit` nor bench/submit.sh accepts. The directory
#   is kept, as the failure's record. Exit 2.
# - Otherwise exit 0, outcome.txt as given.
set -uo pipefail
dir=${1:?usage: par_scan_finish.sh DIR OUTCOME SECS BUNDLER [ARG...]}
outcome=${2:?usage: par_scan_finish.sh DIR OUTCOME SECS BUNDLER [ARG...]}
secs=${3:?usage: par_scan_finish.sh DIR OUTCOME SECS BUNDLER [ARG...]}
shift 3
[ "$#" -ge 1 ] || { echo "par_scan_finish: no bundler given"; exit 2; }
here=$(dirname "$0")
echo "$outcome" > "$dir/outcome.txt"
st=0
python3 "$here/bounded.py" "$secs" 30 "$@" || st=$?
[ "$st" -eq 0 ] && exit 0
case "$st" in
  124) why="the bundle passed its deadline of $secs s" ;;
  125) why="the bundle's process group could not be stopped" ;;
  *) why="the bundle failed (exit $st)" ;;
esac
failed="INVALID: finalisation failed: $why; the outcome before it was: $outcome"
echo "$failed" > "$dir/outcome.txt"
echo "$failed" > "$dir/FINALISATION-FAILED.txt"
for f in "$dir.tar.gz" "$dir.tar.gz.tmp"; do
  [ -e "$f" ] && mv -f "$f" "$dir.tar.gz.failed"
done
echo "par_scan_finish: $failed"
echo "par_scan_finish: no bundle to submit; the attempt's directory is kept: $dir"
exit 2
