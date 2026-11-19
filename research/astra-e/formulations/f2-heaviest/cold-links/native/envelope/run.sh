#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
d="$root/research/astra-e/formulations/f2-heaviest/cold-links/native"
out=${1:?scratch directory}
deps=${2:?absolute release deps folder}
cpu=${3:-4}
mkdir -p "$out"
python3 "$d/generate.py" "$out" > "$out/sources.txt"
cat "$d/envelope/full.inc.rs" >> "$out/compact.rs"
cp "$d/envelope/main.rs" "$out/main.rs"
rustc --edition=2021 -A unexpected_cfgs -C opt-level=3 -C codegen-units=1 "$out/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/price"
nice -n 8 taskset -c "$cpu" "$out/price" "$root/corpora/wikidata.bz2"
