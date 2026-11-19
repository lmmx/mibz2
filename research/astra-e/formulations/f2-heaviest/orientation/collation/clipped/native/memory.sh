#!/usr/bin/env bash
set -euo pipefail
d=$(dirname "$0")
out=${1:?absolute scratch directory containing memory binary}
data=${2:?original admissions binary}
cpu=${3:-3}
nice -n 10 taskset -c "$cpu" "$out/memory" truth "$data" "$d/admissions.csv" > "$out/expected.csv"
for arm in original quote frequent fifth half raw_quote; do
    nice -n 10 taskset -c "$cpu" "$out/memory" "$arm" "$data" "$d/admissions.csv" "$out/expected.csv" > "$out/rss-$arm.csv"
done
