#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
d="$root/research/astra-e/formulations/f2-scheduler/sorted"
tmp=${1:?scratch dir}
deps=${2:?absolute release deps}
mkdir -p "$tmp"
sed 's@../../f2c/second.rs@second.rs@' "$d/candidate.rs" > "$tmp/candidate.rs"
sed 's@../f2c/second.rs@second.rs@' "$d/../candidate.rs" > "$tmp/two.rs"
for p in candidate two; do cat "$d/../../f2c/price_impl.rs" >> "$tmp/$p.rs"; done
cp "$d/../../f2c/second.rs" "$tmp/"
sed -e 's/\["refill","batch","lazy"\]/["sorted","refill","lazy"]/' "$d/../price_main.rs" > "$tmp/price_main.rs"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$tmp/price_main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$tmp/price"
cpu=$(python3 -c 'import os;print(min(os.sched_getaffinity(0)))')
echo "fixed affinity CPU $cpu" >&2
taskset -c "$cpu" "$tmp/price" "$root/corpora/wikidata.bz2"
