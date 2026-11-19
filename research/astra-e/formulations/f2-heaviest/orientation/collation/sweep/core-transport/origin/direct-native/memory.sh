#!/usr/bin/env bash
set -euo pipefail
out=${1:?scratch containing price binary}
data=${2:?original capped admissions}
hashes=${3:?original marked output hashes}
cpu=${4:-2}
for arm in 0 1 2 3 4 5; do
    nice -n 12 taskset -c "$cpu" "$out/price" rss "$data" "$arm" "$hashes" > "$out/memory-$arm.csv"
done
python3 - "$out" <<'PY'
from pathlib import Path
import sys
for i in range(6):
    rows=(Path(sys.argv[1])/f"memory-{i}.csv").read_text().splitlines()
    assert len(rows)==229
    print('\n'.join(rows[i>0:]))
PY
