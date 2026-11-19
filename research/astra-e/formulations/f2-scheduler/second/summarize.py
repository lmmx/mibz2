"""Price the second layer and pair complete verified inversions on the same nine blocks."""
import csv
from collections import defaultdict
from pathlib import Path
from statistics import median
import sys
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[4]/"bench/moonshot"))
from moonshot import paired_ratio
def read(name):
    with (here/name).open() as f:return list(csv.DictReader(f))
profile=read("profile.csv")
blocks=sorted({r["stream"] for r in profile})
print("isolated serial second-level CPU", {k:round(sum(median(int(r[k]) for r in profile if r["stream"]==b) for b in blocks)/1e6,4) for k in profile[0] if k.endswith("_ns")})
graph=[r for r in read("waves.csv") if r["lanes"]=="8"]
print("eight-lane graph",{k:sum(int(r[k]) for r in graph) for k in ("nodes","lookups","lower_waves","batch_waves","refill_waves","dict_bytes","selectors")})
r=read("phase.csv")
assert len(r)==9*4*32
for arm in ("refill","batch","scalar","lazy"):
    print(arm,{k:round(sum(median(int(x[k]) for x in r if x["stream"]==b and x["arm"]==arm) for b in blocks)/1e6,4) for k in ("residual_output_ns","total_ns")})
for arm,ref in (("refill","scalar"),("batch","scalar"),("refill","batch"),("refill","lazy"),("batch","lazy")):
    for key in (["total_ns"] if ref=="lazy" else ["residual_output_ns","total_ns"]):
        samples=lambda a:[sum(int(x[key]) for x in r if x["arm"]==a and int(x["round"])==j) for j in range(1,33)]
        z=paired_ratio(samples(arm),samples(ref))
        print(f"{arm}/{ref} {key}: {z['geomean']:.4f} [{z['lo']:.4f},{z['hi']:.4f}]")
replay=read("replay.csv")
assert len(replay)==9*3*96
for a,ref in (("refill","scalar"),("batch","scalar"),("refill","batch")):
    for k in ("discover_ns","complete_second_ns"):
        total=lambda arm:[sum(int(x[k]) for x in replay if x["arm"]==arm and int(x["round"])==j) for j in range(1,97)]
        z=paired_ratio(total(a),total(ref))
        print(f"pinned replay {a}/{ref} {k}: {z['geomean']:.4f} [{z['lo']:.4f},{z['hi']:.4f}]")
