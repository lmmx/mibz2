#!/usr/bin/env bash
# Prepared recipe only. Execution requires the Coordinator's separate PROTOCOL9 steps.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
if [ "$#" -ne 0 ] && [ "$#" -ne 2 ]; then
  echo "usage: run.sh [CPU CORPUS_DIRECTORY]" >&2
  exit 2
fi
mkdir -p target/cells-live-path
DIR="target/cells-live-path/$(date -u +%Y%m%dT%H%M%SZ)"
mkdir "$DIR"
exec > >(tee -a "$DIR/console.log") 2>&1
finish() {
  local result=$?
  printf '%s\n' "$result" > "$DIR/exit"
  bash bench/bundle.sh "$DIR"
  exit "$result"
}
trap finish EXIT
if [ "$#" -eq 0 ]; then
  # Preparation only: no measured decode, extra warmup or changed driver schedule.
  python3 bench/cells-gap/live/workstation.py "$DIR"
  CPU=$(cat "$DIR/cpu")
  CORPUS=$(realpath "$DIR/inputs")
else
  CPU=$1
  CORPUS=$(realpath "$2")
fi
# All preparation/build/driver failures retain one submit-compatible evidence bundle.
if [ ! -d bench/cells-gap/live/.source ]; then
  python3 bench/cells-gap/live/prepare.py
fi
python3 bench/cells-gap/live/build.py "$DIR/bin"
python3 bench/cells-gap/live/driver.py run "$DIR/bin" "$CORPUS" "$DIR/attempt" --cpu "$CPU"
