"""20 rotated same-process repetitions. Each repetition includes all nine fixed blocks."""
import csv
import itertools
import sys
import math
from pathlib import Path
from statistics import mean, median, stdev

here = Path(__file__).resolve().parent
rows = list(csv.DictReader((here / (sys.argv[1] if len(sys.argv) > 1 else "phase-raw.csv")).open()))
assert len(rows) == 9 * 20 * 2
for field in ("wall_ns", "cpu_ns"):
    totals = {(r, a): sum(int(x[field]) for x in rows if int(x["round"]) == r and x["arm"] == a)
              for r, a in itertools.product(range(1, 21), ("direct", "opend"))}
    ratios = [math.log(totals[r, "direct"] / totals[r, "opend"]) for r in range(1, 21)]
    mid = mean(ratios)
    radar = 2.093024 * stdev(ratios) / math.sqrt(len(ratios))  # df=19
    print(f"{field} direct/opend {math.exp(mid):.5f} [{math.exp(mid-radar):.5f}, {math.exp(mid+radar):.5f}]")
for arm in ("direct", "opend"):
    print(arm, "sum of per-input median milliseconds")
    for k in ("construction_ns", "book_ns", "walk_ns", "unrle_ns", "crc_ns", "cpu_ns"):
        stage = sum(median(int(r[k]) for r in rows if r["arm"] == arm and r["stream"] == stream)
                    for stream in set(r["stream"] for r in rows))
        print(f"{k} {stage/1e6:.4f}")
geo = list(csv.DictReader((here / "geometry.csv").open()))
assert len(geo) == 228 and sum(int(r["rows"]) for r in geo) == 205197794
print("streamed bytes validated", sum(int(r["decoded_bytes"]) for r in geo))
for k in ("action_count", "action_bytes", "cache_bytes", "cache_capacity", "action_capacity"):
    print(f"{k} frozen {geo[0][k]} max {max(int(r[k]) for r in geo)}")
