"""First complete compiler pass as a share of same-pass opend and target time."""
import csv
import math
import statistics
import sys
from collections import defaultdict

rows = list(csv.DictReader(open(sys.argv[1])))
assert len(rows) == 297
print("clock,comparison,paired_geomean,log_t95_low,log_t95_high,median_first_ms,median_remaining_budget_ms,mean_compiled_token_rows")
for clock,full in [("first_ns","elapsed_ns"),("first_cpu_ns","thread_cpu_ns")]:
    first = defaultdict(int)
    all_times, counts = defaultdict(int),defaultdict(int)
    for x in rows:
        i, a = int(x["round"]), x["arm"]
        all_times[a,i] += int(x[full])
        if a == "phrases":
            first[i] += int(x[clock])
            counts[i] += int(x["token_rows"])
    for name, denom, factor in [("first/opend","opend16",1),("first/80pct_lazy_stage","lazy",.8),("first/full_hierarchy","phrases",1)]:
        totals = [all_times[denom,i] * factor for i in range(9)]
        logs = [math.log(first[i]/totals[i]) for i in range(9)]
        m = statistics.mean(logs)
        err = statistics.stdev(logs)*2.306004/3
        room = [totals[i]-first[i] for i in range(9)]
        print(f"{clock},{name},{math.exp(m):.5f},{math.exp(m-err):.5f},{math.exp(m+err):.5f},"
              f"{statistics.median(first.values())/1e6:.5f},{statistics.median(room)/1e6:.5f},{statistics.mean(counts.values()):.0f}")
