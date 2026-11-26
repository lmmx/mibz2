#!/usr/bin/env bash
# Reconstruct the standalone scorer candidate and its property tests in DIR.
set -euo pipefail
dir=$(cd "$(dirname "$0")" && pwd)
out=${1:?use an output directory}
mkdir -p "$out"
cp "$dir/../packed.rs" "$out/candidate.rs"
cp "$dir/../checks.rs" "$out/checks.rs"
patch --silent "$out/candidate.rs" < "$dir/medium.patch"
