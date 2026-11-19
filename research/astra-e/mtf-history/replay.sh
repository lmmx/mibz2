#!/bin/bash
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
out=$(realpath -m "${1:?fresh output directory}")
lib=$(realpath "${2:?release libmibz2.rlib built with moonshot}")
input=$(realpath "${3:?Wikidata input}")
mkdir -p "$out"
python3 "$d/ablate.py" "$out"
cp "$d/ablate.rs" "$d/census.rs" "$out/"
for f in check placements ablate census; do
    rustc -A unused --edition=2021 -C opt-level=3 -C codegen-units=1 --extern "mibz2=$lib" -L "dependency=$(dirname "$lib")" "$out/$f.rs" -o "$out/$f"
done
"$out/census" "$input" "$out/blocks.csv" "$out/tail.csv"
"$out/check" "$input" check 2>"$out/check.txt"
(echo 'depth,round,position,arm,ns,records'
 for stack in 0 3 9; do "$out/placements" "$input" "$stack"; done) >"$out/stages.csv"
"$out/ablate" "$input" >"$out/ablation.csv" 2>"$out/ablation-check.txt"
