#!/usr/bin/env bash
# First unpack the pinned .irec.xz archives into the directory passed here.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
d=research/astra-e/formulations/f4
mkdir -p "$d/target"
cargo build --release --locked --features moonshot --lib
deps="${CARGO_TARGET_DIR:-target}/release/deps"
rustc -O --edition=2021 -L "dependency=$deps" --extern "mibz2=$deps/libmibz2.rlib" \
  "$d/check.rs" -o "$d/target/check"
python3 "$d/deep.py" "$d/target/deep.column"
"$d/target/check" 0:"$d/target/deep.column" 1001:"$d/target/deep.column" \
  11:"$d/target/deep.column" 1668:"$d/target/deep.column" 1041:"$d/target/deep.column"
# The established extractor creates the same ten real columns and three seeded
# synthetic columns as the direct native check receipt.
python3 research/astra-e/dense/check.py "${1:-$d/target/wikidata}" "$d/target/columns" "$d/target/check"
