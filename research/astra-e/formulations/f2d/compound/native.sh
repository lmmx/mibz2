#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
study="$root/research/astra-e/formulations"
dest=${1:?scratch directory}
deps=${2:?absolute release/deps directory}
mkdir -p "$dest"/{f2c,f2d,word}
cat "$study/f2-word/candidate.rs" "$study/f2-word/price_impl.rs" > "$dest/word/candidate.rs"
cat "$study/f2d/compound/candidate.rs" "$study/f2-word/price_impl.rs" > "$dest/word/learn.rs"
cat "$study/f2c/candidate.rs" "$study/f2c/price_impl.rs" > "$dest/f2c/candidate.rs"
cp "$study/f2c/second.rs" "$dest/f2c"
cp "$study/f2d/candidate.rs" "$dest/f2d"
sed 's/^impl Candidate {/impl<const CHAIN:bool> Solver<CHAIN> {/' "$study/f2c/price_impl.rs" >> "$dest/f2d/candidate.rs"
cp "$study/f2d/compound/native.rs" "$dest/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/native.rs" \
  --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/native"
"$dest/native" "$root/corpora/wikidata.bz2" "$dest/geometry.csv" > "$dest/phase-cpu.csv"
