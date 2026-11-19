#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
study="$root/research/astra-e/formulations/f2d"
dest=${1:?scratch dir}
deps=${2:?absolute release/deps directory}
mkdir -p "$dest"
cp "$study/output-owned/candidate.rs" "$dest/candidate.rs"
sed -i '/^#\[path = "checks.rs"\]$/,+1d' "$dest/candidate.rs"
# Preserve the final cfg(test) attribute for this added test body.
cat "$study/mixed/probe_test.rs" >> "$dest/candidate.rs"
sed -e '/pub(super) struct Second {/a\    pub(super) proj:(u32,u32),' -e '/        self.discover(d, first_words);/i\        self.proj=(row,begin);' "$study/output-owned/second.rs" > "$dest/second.rs"
cat "$study/mixed/probe_impl.rs" >> "$dest/second.rs"
cp "$study/output-owned/property.rs" "$dest/"
rustc --edition=2021 -O --test "$dest/property.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/probe"
F2_CORPUS="$root/corpora/wikidata.bz2" F2_CSV="$dest/thresholds.csv" "$dest/probe" candidate::hotspot --nocapture
