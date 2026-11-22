"""Summarize optional per-worker decode_block clocks (microseconds)."""
import csv
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "2026-11-22-open-construction"))
from summarize import ci, summary

raw, output = map(Path, sys.argv[1:])
pairs = {}
for line in raw.read_text().splitlines():
    rnd, conf, workers, arm, rows = json.loads(line)
    assert conf == f"scorpus{workers}"
    assert set(rows) == {f"blk{i:03}" for i in range(431)}
    key = (workers, arm, rnd)
    assert key not in pairs
    pairs[key] = rows

with output.open("w") as f:
    out = csv.writer(f, lineterminator="\n")
    out.writerow(["block", "workers", "master_us", "burst_us", "ratio", "lower95",
                  "upper95", "aa_ratio", "aa_lower95", "aa_upper95"])
    for i in range(431):
        for w in [1, 4]:
            times = [[pairs[w, arm, r][f"blk{i:03}"] for r in range(12)] for arm in ["a", "b"]]
            master, burst = zip(*times[1])
            out.writerow([i, w, summary(master), summary(burst), *ci(times[1]), *ci(times[0])])
