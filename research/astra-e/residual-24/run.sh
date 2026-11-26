#!/usr/bin/env bash
set -euo pipefail
d=research/astra-e/residual-24
out=${1:?fresh scratch directory}
mkdir "$out"
out=$(cd "$out" && pwd)
python3 "$d/price.py" "$out"
deps=${PAGE_DEPS:-${CARGO_TARGET_DIR:-target}/release/deps}
echo "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915  corpora/wikidata.bz2" | sha256sum --check
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$out/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/price"
{ rustc -vV; echo 'source 80d8409359214b4e5abfd4430213921e82e9cd30'; sha256sum "$out"/{main.rs,old.rs,new.rs,price} "$deps/libmibz2.rlib"; } > "$out/manifest.txt"
"$out/price" small > "$out/check.txt"
"$out/price" corpora/wikidata.bz2 "$out" geometry
nice -n 10 taskset -c 4 "$out/price" corpora/wikidata.bz2 "$out" price > "$out/first.csv"
nice -n 10 taskset -c 2 "$out/price" corpora/wikidata.bz2 "$out" price > "$out/repeat.csv"
