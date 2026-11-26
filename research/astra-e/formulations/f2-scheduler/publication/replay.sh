#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
d="$root/research/astra-e/formulations/f2-scheduler/publication"
tmp=${1:?scratch}
deps=${2:?absolute release deps}
python3 "$d/prepare.py" "$tmp"
cp "$d/../../f2c/second.rs" "$d/price_main.rs" "$tmp/"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$tmp/price_main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$tmp/price"
cpu=$(python3 -c 'import os;print(min(os.sched_getaffinity(0)))')
echo "fixed CPU $cpu" >&2
taskset -c "$cpu" "$tmp/price" "$root/corpora/wikidata.bz2"
