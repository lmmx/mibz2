#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
study="$root/research/astra-e/formulations"
dest=${1:?scratch directory}
deps=${2:?absolute release/deps directory}
mkdir -p "$dest"
cat "$study/f2c/candidate.rs" "$study/f2c/price_impl.rs" > "$dest/old.rs"
sed 's|../../f2c/second.rs|second.rs|' "$study/f2d/splice/candidate.rs" > "$dest/new.rs"
cat "$study/f2c/price_impl.rs" >> "$dest/new.rs"
cp "$study/f2c/second.rs" "$study/f2d/splice/"{merge,price}.rs "$dest/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/price"
"$dest/price" "$root/corpora/wikidata.bz2"
