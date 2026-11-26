"""Summarize each arm's repeated complete-block CPU phases."""
import csv
from collections import defaultdict
from pathlib import Path
from statistics import median
import sys

src = Path(sys.argv[1])
with src.open() as f:
    data = list(csv.DictReader(f))
assert len(data) == 20 * 11 * 5
assert len({(r["round"], r["ordinal"], r["mode"]) for r in data}) == len(data)
phases = ["classify_and_queue_ns", "induction_ns", "project_edges_ns", "emit_and_call_ns"]
writer = csv.writer(sys.stdout, lineterminator="\n")
writer.writerow(["mode", "phase", "sum_of_11_block_medians_ns", "median_complete_lap_ns"])
for mode in range(5):
    for phase in phases:
        blocks = defaultdict(list)
        laps = defaultdict(int)
        for r in data:
            if int(r["mode"]) == mode:
                blocks[r["ordinal"]].append(int(r[phase]))
                laps[r["round"]] += int(r[phase])
        assert len(blocks) == 11 and len(laps) == 20
        writer.writerow([mode, phase, int(sum(median(xs) for xs in blocks.values())), int(median(laps.values()))])
