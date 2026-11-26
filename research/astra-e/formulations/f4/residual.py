"""Price the measured residual from each per-call difference on the verified output."""
import csv
import statistics
import sys
from collections import defaultdict
rows = list(csv.DictReader(open(sys.argv[1])))
assert len(rows) == 297
print("clock,passes,median_residual_ns_per_token,min,max,median_allowed_ns_per_token")
for full, part in [("elapsed_ns","first_ns"),("thread_cpu_ns","first_cpu_ns")]:
    groups = defaultdict(lambda:defaultdict(int))
    for x in rows:
        g=groups[int(x["round"])]
        if x["arm"] == "phrases":
            g["residual"] += int(x[full]) - int(x[part])
            g["first"] += int(x[part])
            g["m"] += int(x["token_rows"])
        elif x["arm"] == "opend16":
            g["opend"] += int(x[full])
    price=[g["residual"]/g["m"] for g in groups.values()]
    room=[(g["opend"]-g["first"])/g["m"] for g in groups.values()]
    print(f"{full},{len(price)},{statistics.median(price):.5f},{min(price):.5f},{max(price):.5f},{statistics.median(room):.5f}")
