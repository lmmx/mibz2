#!/usr/bin/env bash
# Reuses the immutable F2c phase timer for the candidate and both comparator arms.
set -euo pipefail
root=$(git rev-parse --show-toplevel)
study="$root/research/astra-e/formulations/f2d"
dest=${1:?scratch directory}
deps=${2:?absolute release/deps directory}
mkdir -p "$dest"/{f2c,f2d}
cp "$study"/../f2c/{candidate.rs,second.rs} "$dest/f2c"
cat "$study"/../f2c/price_impl.rs >> "$dest/f2c/candidate.rs"
cp "$study/candidate.rs" "$dest/f2d"
sed 's/^impl Candidate {/impl<const CHAIN:bool> Solver<CHAIN> {/' "$study"/../f2c/price_impl.rs >> "$dest/f2d/candidate.rs"
cp "$study/native.rs" "$dest/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/native.rs" \
  --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/native"
"$dest/native" "$root/corpora/wikidata.bz2" "$dest/geometry.csv" > "$dest/phase-cpu.csv"
