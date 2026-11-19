"""Paired CPU stage panel, independent per-section structural cross check."""
from collections import defaultdict
import csv
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0,str(ROOT/"bench/moonshot"))
from moonshot import paired_ratio
sys.path.insert(0,str(HERE.parent/"cut"))
from frontier import load

def accepted(path):
    with path.open() as f:
        return {(int(r["rows"]),int(r["primary"])):r for r in csv.DictReader(f) if r["took"]=="1"}

with (HERE/"phase.csv").open() as f:
    phase = {(int(r["round"]),r["stream"],r["arm"]):r for r in csv.DictReader(f)}
blocks = sorted({b for _,b,_ in phase})
assert len(blocks)==9 and len(phase)==40*9*4
for field, controls in (("merge_ns",("leaf","refill")),("total_ns",("leaf","refill","lazy"))):
    for baseline in controls:
        sums = {a:[sum(int(phase[r,b,a][field]) for b in blocks) for r in range(1,41)] for a in ("frames",baseline)}
        res = paired_ratio(sums["frames"],sums[baseline])
        print(field,"frames /",baseline,*(f"{res[k]:.5f}" for k in ("geomean","lo","hi")))

full=accepted(HERE.parent/"geometry.csv")
leaf=accepted(HERE.parent/"leaf/geometry.csv")
frames=accepted(HERE/"geometry.csv")
assert len(frames)==228 and full.keys()==leaf.keys()==frames.keys()
for k,r in frames.items():
    assert all(r[f]==full[k][f] for f in ("words","nodes","selectors","root_runs","merged","token_rows"))
    assert int(r["merge_steps"])<=int(leaf[k]["merge_steps"])
with (HERE.parent/"audit/real.csv").open() as f:
    shape={int(r["block"]):(int(r["rows"]),int(r["primary"])) for r in csv.DictReader(f)}
with (HERE/"intervals.csv").open() as f:
    for r in csv.DictReader(f):
        k=shape[int(r["block"])]
        if k in frames:
            assert int(r["overlaps"])==int(frames[k]["merge_steps"])
costs=[sum(int(r["merge_steps"]) for r in arm.values()) for arm in (full,leaf,frames)]
print("original visits, absorbed visits, frame overlaps",*costs)
print("suspensions",sum(int(r["token_visits"]) for r in frames.values()))
