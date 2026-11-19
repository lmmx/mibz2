#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
study="$root/research/astra-e/formulations/f2-dual"
dest=${1:?scratch directory}
deps=${2:?absolute moonshot release/deps directory}
mkdir -p "$dest"
cp "$study/price.rs" "$dest/price.rs"
cat "$study/candidate.rs" "$study/plain.rs" > "$dest/dual.rs"
cat "$study/../f2-word/candidate.rs" "$study/../f2-word/price_impl.rs" > "$dest/trie.rs"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price.rs" \
    --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/price"
"$dest/price" "$root/corpora/wikidata.bz2"
