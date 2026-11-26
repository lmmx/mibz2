"""Pair each pass over the same admitted blocks in the five-arm comparison."""
import csv
import statistics
import sys
from pathlib import Path

here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[4]/"bench/moonshot"))
from moonshot import paired_ratio

records=list(csv.DictReader((here/"phase.csv").open()))
blocks={r["stream"] for r in records}
assert len(blocks)==9 and len(records)==9*5*30
for arm in ("direct-refill","direct-batch","nested-refill","nested-batch","lazy"):
    print(arm,end="")
    for key in ("prep_mark_ns","discover_ns","merge_ns","residual_output_ns","total_ns","wall_ns"):
        summed=sum(statistics.median(int(r[key]) for r in records if r["stream"]==block and r["arm"]==arm) for block in blocks)
        print(f" {key}={summed/1e6:.4f} ms",end="")
    print()
for a,b in (("direct-refill","direct-batch"),("nested-refill","nested-batch"),("direct-refill","nested-refill"),("direct-refill","lazy"),("nested-refill","lazy")):
    for key in ("total_ns","wall_ns"):
        vals=lambda arm:[sum(int(r[key]) for r in records if r["arm"]==arm and int(r["round"])==i) for i in range(1,31)]
        v=paired_ratio(vals(a),vals(b))
        print(f"{a}/{b} {key} {v['geomean']:.4f} [{v['lo']:.4f},{v['hi']:.4f}]")
