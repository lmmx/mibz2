#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
study="$root/research/astra-e/formulations"
dest=${1:?scratch directory}
deps=${2:?absolute release/deps directory}
mkdir -p "$dest"/{old,new,batched,combined}
cat "$study/f2c/candidate.rs" "$study/f2c/price_impl.rs" > "$dest/old/candidate.rs"
sed 's|../f2c/second.rs|second.rs|' "$study/f2d/candidate.rs" > "$dest/new/candidate.rs"
sed 's/^impl Candidate {/impl<const CHAIN:bool> Solver<CHAIN> {/' "$study/f2c/price_impl.rs" >> "$dest/new/candidate.rs"
sed 's|../f2c/second.rs|second.rs|' "$study/f2-frontier/candidate.rs" > "$dest/batched/candidate.rs"
cat "$study/f2c/price_impl.rs" >> "$dest/batched/candidate.rs"
sed 's|../../f2c/second.rs|second.rs|' "$study/f2d/frontier/candidate.rs" > "$dest/combined/candidate.rs"
cat "$study/f2c/price_impl.rs" >> "$dest/combined/candidate.rs"
cp "$study/f2c/second.rs" "$dest/old/second.rs"
cp "$study/f2c/second.rs" "$dest/batched/second.rs"
cp "$study/f2c/second.rs" "$dest/new/second.rs"
cp "$study/f2c/second.rs" "$dest/combined/second.rs"
cp "$study/f2d/frontier/price.rs" "$dest/price.rs"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/price"
"$dest/price" "$root/corpora/wikidata.bz2"
