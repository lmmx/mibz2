#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
d="$root/research/astra-e/formulations/f2-heaviest/cold-links/native"
out=${1:?scratch directory}
deps=${2:?absolute release deps directory containing libmibz2.rlib}
cpu_a=${3:-4}
cpu_b=${4:-3}
mkdir -p "$out"
out=$(cd "$out" && pwd)
python3 "$d/generate.py" "$out" > "$out/sources.txt"
rustc --edition=2021 -A unexpected_cfgs -C opt-level=3 -C codegen-units=1 "$out/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/price"
corpus="$root/corpora/wikidata.bz2"
"$out/price" small > "$out/small.txt"
"$out/price" geometry "$corpus" > "$out/geometry.csv" 2> "$out/geometry.log"
nice -n 8 taskset -c "$cpu_a" "$out/price" phase "$corpus" > "$out/phase.csv" 2> "$out/phase.log"
nice -n 8 taskset -c "$cpu_b" "$out/price" phase "$corpus" > "$out/repeat.csv" 2> "$out/repeat.log"
printf '%s\n' 'stream,rows,arm,scratch_capacity_bytes,process_resident_kib,process_peak_kib' > "$out/rss.csv"
for arm in refill known compact cut1x8 cut8x8 cut8x1; do
    nice -n 8 taskset -c "$cpu_a" "$out/price" rss "$corpus" "$arm" >> "$out/rss.csv"
done
PYTHONDONTWRITEBYTECODE=1 python3 "$d/summarize.py" "$out" > "$out/summary.txt"
