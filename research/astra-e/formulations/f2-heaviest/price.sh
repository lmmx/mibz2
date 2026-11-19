#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
d="$root/research/astra-e/formulations/f2-heaviest"
dest=${1:?scratch directory}
deps=${2:?release dependencies directory containing libmibz2.rlib}
mkdir -p "$dest"
cat "$d/candidate.rs" "$d/price_impl.rs" > "$dest/candidate.rs"
cp "$d/price_main.rs" "$dest"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price_main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/price"
"$dest/price" "$root/corpora/wikidata.bz2"
