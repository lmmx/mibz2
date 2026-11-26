#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
d="$root/research/astra-e/formulations/f2-dual"
dest=${1:?scratch directory}
deps=${2:?absolute release dependencies directory}
python3 "$d/defer.py" "$dest"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$dest/price.rs" \
  --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$dest/price"
"$dest/price" "$root/corpora/wikidata.bz2"
