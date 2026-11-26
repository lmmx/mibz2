#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
study="$root/research/astra-e/formulations/f2-word"
dest=${1:?scratch directory}
deps=${2:?absolute release/deps directory}
mkdir -p "$dest"
cat "$study/candidate.rs" "$study/price_impl.rs" > "$dest/candidate.rs"
cat "$study/../f2c/candidate.rs" "$study/../f2c/price_impl.rs" > "$dest/two.rs"
cp "$study/../f2c/second.rs" "$study/price_main.rs" "$dest/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price_main.rs" \
  --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/price"
"$dest/price" "$root/corpora/wikidata.bz2"
