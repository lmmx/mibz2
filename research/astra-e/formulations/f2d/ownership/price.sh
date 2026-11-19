#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
study="$root/research/astra-e/formulations"
dest=${1:?scratch directory}
deps=${2:?absolute release/deps directory}
mkdir -p "$dest"/{old,new}
cat "$study/f2c/candidate.rs" "$study/f2c/price_impl.rs" > "$dest/old/candidate.rs"
cat "$study/f2d/ownership/candidate.rs" "$study/f2d/ownership/price_impl.rs" > "$dest/new/candidate.rs"
cp "$study/f2c/second.rs" "$dest/old"
cp "$study/f2d/ownership/second.rs" "$dest/new"
sed -e 's|"old.rs"|"old/candidate.rs"|' -e 's|"new.rs"|"new/candidate.rs"|' "$study/f2d/ownership/price.rs" > "$dest/price.rs"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/price"
"$dest/price" "$root/corpora/wikidata.bz2"
