#!/usr/bin/env bash
# Frozen i9-10900K probe: 8-worker knee on the pinned text panel vs wikidata (text-knee-i9).
# Produces one self-contained run directory under target/text-knee/run-<UTC> and packs it
# into one tarball via bench/bundle.sh (compatible with `just submit`).
set -euo pipefail

export RUSTUP_TOOLCHAIN=1.99.1

parent="$PWD/target/text-knee"
mkdir -p "$parent"
out="$parent/run-$(date -u +%Y%m%dT%H%M%SZ)"
mkdir "$out"

exec 3>&1 4>&2
exec > >(tee -a "$out/console.log") 2>&1
logger=$!

finish() {
  rc=$?
  if [[ ! -s "$out/outcome.txt" ]]; then
    echo "INCOMPLETE (exit $rc)" > "$out/outcome.txt"
  fi
  exec 1>&3 2>&4
  wait "$logger" || rc=1
  bash bench/bundle.sh "$out" || rc=1
  exit "$rc"
}
trap finish EXIT

python3 bench/text_knee.py "$out"
