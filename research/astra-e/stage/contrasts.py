"""Paired log-t intervals for the 11 distinct measured passes in laps.csv."""
import csv
import math
import statistics
import sys
from collections import defaultdict

times = defaultdict(dict)
for row in csv.DictReader(open(sys.argv[1])):
    times[row["arm"], int(row["round"])][int(row["ordinal"])] = int(row["elapsed_ns"])
arms = sorted({a for a, _ in times})
assert sorted({p for _, p in times}) == list(range(11))
comparisons = [(a, "opend16") for a in arms if a != "opend16"]
comparisons.extend([("opend16", "lazy"), ("two_step", "page_trace")])
print("numerator,denominator,geomean,log_t95_low,log_t95_high,sum_of_block_medians_ratio")
for a, b in comparisons:
    logs = [math.log(sum(times[a, i].values()) / sum(times[b, i].values())) for i in range(11)]
    mean = statistics.mean(logs)
    radius = 2.228139 * statistics.stdev(logs) / math.sqrt(11)
    def block_medians(name):
        return sum(statistics.median(times[name, i][key] for i in range(11)) for key in times[name, 0])
    print(f"{a},{b},{math.exp(mean):.5f},{math.exp(mean-radius):.5f},{math.exp(mean+radius):.5f},"
          f"{block_medians(a) / block_medians(b):.5f}")
