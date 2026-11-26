"""Paired ratios of per-pass sums across the fixed sample, excluding verification time."""
import csv
import statistics
import sys
from collections import defaultdict

rows = list(csv.DictReader(open(sys.argv[1])))
groups = defaultdict(dict)
for row in rows:
    block = int(row["ordinal"])
    key = int(row["round"]), row["arm"]
    assert block not in groups[key]
    groups[key][block] = int(row["elapsed_ns"])

arms = sorted({arm for _, arm in groups})
rounds = sorted({r for r, _ in groups})
for g in groups.values():
    assert sorted(g) == [1, 19, 61, 108, 150, 194, 240, 282, 329, 371, 407]
print("arm,rounds,median_sample_ms,geomean_over_lazy,geomean_over_opend16,min_over_opend16,max_over_opend16")
for arm in arms:
    times = [sum(groups[r, arm].values()) for r in rounds]
    lazy = [t / sum(groups[r, "lazy"].values()) for r, t in zip(rounds, times)]
    opend = [t / sum(groups[r, "opend16"].values()) for r, t in zip(rounds, times)]
    print(f"{arm},{len(rounds)},{statistics.median(times) / 1e6:.4f},{statistics.geometric_mean(lazy):.5f},"
          f"{statistics.geometric_mean(opend):.5f},{min(opend):.5f},{max(opend):.5f}")
