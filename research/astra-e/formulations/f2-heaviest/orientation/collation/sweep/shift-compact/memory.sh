#!/usr/bin/env bash
set -euo pipefail
d=$(dirname "$0");out=${1:?scratch with price binary};data=${2:?binary original admissions}
tags="$d/../../clipped/native/admissions.csv";hash=${3:?independent stable-sort stream FNVs};cpu=${4:-2}
for arm in -2 -1 0 1 2 3; do
    nice -n 10 taskset -c "$cpu" "$out/price" rss "$data" "$arm" "$tags" "$hash" > "$out/rss-$arm.csv"
done
python3 -B "$d/memory.py" "$out"
