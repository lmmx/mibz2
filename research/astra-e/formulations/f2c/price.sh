#!/usr/bin/env bash
# Uses an existing moonshot release build containing the research-enabled mibz2 rlib.
set -euo pipefail
root=$(git rev-parse --show-toplevel)
study="$root/research/astra-e/formulations/f2c"
dest=${1:?scratch directory}; deps=${2:?release/deps directory}
mkdir -p "$dest"
cat "$study/candidate.rs" "$study/price_impl.rs" > "$dest/candidate.rs"
cp "$study/second.rs" "$study/price_main.rs" "$dest/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price_main.rs" \
  --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/price"
"$dest/price" "$root/corpora/wikidata.bz2"
