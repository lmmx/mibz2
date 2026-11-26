#!/usr/bin/env bash
set -euo pipefail
d=research/astra-e/residual-24
out=${1:?fresh directory}
mkdir "$out"
out=$(cd "$out" && pwd)
python3 "$d/census.py" "$out"
deps=${PAGE_DEPS:-${CARGO_TARGET_DIR:-target}/release/deps}
echo "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915  corpora/wikidata.bz2" | sha256sum --check
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$out/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/census"
"$out/census" corpora/wikidata.bz2 > "$out/census.csv"
