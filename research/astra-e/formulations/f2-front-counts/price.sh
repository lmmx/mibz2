#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
study="$root/research/astra-e/formulations/f2-front-counts"
dest=${1:?scratch directory}
deps=${2:?absolute release/deps directory}
mkdir -p "$dest"
sed 's@../f2c/second.rs@second.rs@' "$study/candidate.rs" > "$dest/candidate.rs"
sed 's/self.prepare(r.records(),n as u32)/self.prepare_counted(r.records(),n as u32,r.byte_counts())/' "$study/../f2c/price_impl.rs" >> "$dest/candidate.rs"
sed 's@../f2c/second.rs@second.rs@' "$study/../f2-scheduler/candidate.rs" > "$dest/two.rs"
cat "$study/../f2c/price_impl.rs" >> "$dest/two.rs"
cp "$study/../f2c/second.rs" "$study/price_main.rs" "$dest/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price_main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/price"
"$dest/price" "$root/corpora/wikidata.bz2"
