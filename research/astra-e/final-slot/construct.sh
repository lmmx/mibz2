#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
deps=${CARGO_TARGET_DIR:-bench/moonshot/target}/release/deps
bin=$(mktemp)
trap 'rm -f -- "$bin"' EXIT
rustc --edition 2021 -C opt-level=2 research/astra-e/final-slot/construct.rs \
    --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$bin"
"$bin" "${1:?supply the canonical Wikidata corpus}"
