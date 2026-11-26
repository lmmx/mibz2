"""Join the published route census and the older single-thread phase observations."""
import csv
from collections import defaultdict
from pathlib import Path

base = Path(__file__).resolve().parents[3] / "docs/journal"
routes = {}
groups = defaultdict(lambda: [0, 0, 0, 0, 0.0])
for x in csv.DictReader((base / "2026-11-18-astra-d-memory/evidence/corpus-routes.csv").open()):
    n = int(x["rows"])
    if x["run_front"] == "0":
        name = "column"
    elif int(x["records"]) * 20 >= n * 3:
        name = "dense runs"
    else:
        name = "open eligible"
    routes[x["ordinal"]] = name
    g = groups[name]
    g[0] += 1
    g[1] += n
    g[2] += int(x["output"])
for x in csv.DictReader((base / "2026-11-18-open-return/evidence/phases-wikidata.csv").open()):
    if x["arm"] == "lazy":
        g = groups[routes[x["ordinal"]]]
        g[3] += 1
        g[4] += float(x["joint_median_s"])
print("route,blocks,rows,output_bytes,historical_inverse_samples,seconds_sum_of_block_medians")
for name in sorted(groups):
    blocks, rows, output, samples, seconds = groups[name]
    print(f"{name},{blocks},{rows},{output},{samples},{seconds:.9f}")
