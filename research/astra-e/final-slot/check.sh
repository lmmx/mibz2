#!/usr/bin/env bash
# After building the scorer in release mode with the current candidate.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
deps=${CARGO_TARGET_DIR:-bench/moonshot/target}/release/deps
libbz2=("$deps"/libbzip2-*.rlib)
test "${#libbz2[@]}" -eq 1
bin=$(mktemp)
trap 'rm -f -- "$bin"' EXIT
CARGO_MANIFEST_DIR=$PWD rustc --edition 2021 -C opt-level=2 research/astra-e/final-slot/check.rs \
    --extern mibz2="$deps/libmibz2.rlib" --extern bzip2="${libbz2[0]}" \
    -L "dependency=$deps" -o "$bin"
"$bin" "${1:?supply the frozen bzip2 block}"
