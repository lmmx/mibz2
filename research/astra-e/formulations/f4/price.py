"""Paired per-pass sums over the 11 fixed Wikidata blocks, nine balanced passes."""
import csv
import math
import statistics
import sys
from collections import defaultdict
rows = list(csv.DictReader(open(sys.argv[1])))
assert len(rows) == 9*11*3
print("clock,numerator,denominator,passes,geomean,log_t95_low,log_t95_high,median_numerator_ms,median_denominator_ms")
for field in ["elapsed_ns","thread_cpu_ns"]:
    sums = defaultdict(int)
    for row in rows:
        sums[row["arm"],int(row["round"])] += int(row[field])
    for a,b in [("phrases","opend16"),("phrases","lazy"),("opend16","lazy")]:
        x = [sums[a,i] for i in range(9)]
        y = [sums[b,i] for i in range(9)]
        logs = [math.log(v/u) for v,u in zip(x,y)]
        m = statistics.mean(logs)
        err = statistics.stdev(logs)*2.306004/3
        print(f"{field},{a},{b},9,{math.exp(m):.5f},{math.exp(m-err):.5f},{math.exp(m+err):.5f},"
              f"{statistics.median(x)/1e6:.5f},{statistics.median(y)/1e6:.5f}")
