#!/usr/bin/env bash
# Rebuild the narrow-index arm with its original immediate or medium-literal grammar.
set -euo pipefail
dir=$(cd "$(dirname "$0")" && pwd)
out=${1:?use an output directory}
case "${2:-packed}" in
  packed)
    mkdir -p "$out"
    cp "$dir/../packed.rs" "$out/candidate.rs"
    cp "$dir/../checks.rs" "$out/checks.rs"
    ;;
  hybrid) "$dir/../hybrid/prepare.sh" "$out" ;;
  *) exit 2 ;;
esac
patch --silent --no-backup-if-mismatch "$out/candidate.rs" < "$dir/indices.patch"
