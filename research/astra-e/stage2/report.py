"""Paired pass sums and illustrative 95% log-t intervals across 21 rotated passes."""
import csv
import math
import statistics
import sys
from collections import defaultdict

rows = list(csv.DictReader(open(sys.argv[1])))
arms = sorted({r["arm"] for r in rows})
ordinals = [1, 19, 61, 108, 150, 194, 240, 282, 329, 371, 407]
assert len(arms) == 7
assert len(rows) == 7 * 21 * 11
print("clock,arm,reference,passes,median_sample_ms,geomean,log_t95_low,log_t95_high,block_medians_ratio")
for field in ("elapsed_ns", "thread_cpu_ns"):
    data = defaultdict(dict)
    for r in rows:
        group = data[r["arm"], int(r["round"])]
        block = int(r["ordinal"])
        assert block not in group
        group[block] = int(r[field])
    assert all(sorted(blocks) == ordinals for blocks in data.values())
    for base in ["opend16", "eager", "lazy"]:
        for a in arms:
            laps = [sum(data[a, i].values()) for i in range(21)]
            logs = [math.log(t / sum(data[base, i].values())) for i, t in enumerate(laps)]
            m = statistics.mean(logs)
            err = 2.085963 * statistics.stdev(logs) / math.sqrt(21)
            def median_sum(arm):
                return sum(statistics.median(data[arm, i][j] for i in range(21)) for j in ordinals)
            print(f"{field},{a},{base},21,{statistics.median(laps)/1e6:.4f},{math.exp(m):.5f},"
                  f"{math.exp(m-err):.5f},{math.exp(m+err):.5f},{median_sum(a)/median_sum(base):.5f}")
