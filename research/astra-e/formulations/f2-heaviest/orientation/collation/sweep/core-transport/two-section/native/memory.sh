#!/usr/bin/env bash
set -euo pipefail
d=$(dirname "$0")
out=${1:?scratch containing price binary}
data=${2:?input admissions binary}
hashes=${3:?independent expected output hashes}
cpu=${4:-2}
sections="$d/../../../../clipped/native/admissions.csv"
for arm in -1 8 24 32 36 40 68 72; do
    nice -n 12 taskset -c "$cpu" "$out/price" rss "$data" "$arm" "$sections" "$hashes" > "$out/rss-$arm.csv"
done
python3 - "$out" <<'PY'
from pathlib import Path
import csv,sys
d=Path(sys.argv[1])
writer=None
for arm in [-1,8,24,32,36,40,68,72]:
    with (d/f'rss-{arm}.csv').open() as f:
        reader=csv.DictReader(f)
        if writer is None:
            writer=csv.DictWriter(sys.stdout,fieldnames=reader.fieldnames,lineterminator='\n')
            writer.writeheader()
        writer.writerows(reader)
PY
