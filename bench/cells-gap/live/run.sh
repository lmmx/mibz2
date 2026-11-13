#!/usr/bin/env bash
# Prepared recipe only. Execution requires the Coordinator's separate PROTOCOL9 steps.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
if [ "$#" -ne 2 ]; then
  echo "usage: run.sh CPU CORPUS_DIRECTORY" >&2
  exit 2
fi
CPU=$1
CORPUS=$(realpath "$2")
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
# Input/source/build failures are preserved; no retry or automatic fetch/corpus replay.
if [ ! -d bench/cells-gap/live/.source ]; then
  python3 bench/cells-gap/live/prepare.py
fi
python3 bench/cells-gap/live/build.py "$DIR/bin"
python3 bench/cells-gap/live/driver.py run "$DIR/bin" "$CORPUS" "$DIR/attempt" --cpu "$CPU"
