#!/usr/bin/env bash
set -euo pipefail
d=research/astra-e/formulations/f2-pages/payload/slots/width
out=${1:?fresh scratch directory}
mkdir "$out"
python3 "$d/capacity.py" "$out"
if [[ -n ${PAGE_DEPS:-} ]]; then deps="$PAGE_DEPS"; else
  cargo build --locked --release --features moonshot --lib
  deps="${CARGO_TARGET_DIR:-target}/release/deps"
fi
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$out/capacity.rs" \
  --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/capacity"
"$out/capacity" corpora/wikidata.bz2 geometry > "$out/capacity.csv"
