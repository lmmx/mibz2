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

old = accepted(HERE.parent/"geometry.csv")
leaf = accepted(HERE/"geometry.csv")
assert len(leaf)==228 and old.keys()==leaf.keys()
for k in old:
    assert all(old[k][f]==leaf[k][f] for f in ("nodes","words","selectors","root_runs","token_rows","merged"))
    assert int(old[k]["merge_steps"]) >= int(leaf[k]["merge_steps"])
with (HERE.parent/"audit/real.csv").open() as f:
    rows = {int(x["block"]):x for x in csv.DictReader(f)}
for b,m in load(HERE.parent/"cut/nodes.csv.xz").items():
    k = int(rows[b]["rows"]),int(rows[b]["primary"])
    if k in leaf:
        expected = sum(m.g[v] for v in range(m.n) if m.children[v])
        assert expected == int(leaf[k]["merge_steps"])
before = sum(int(r["merge_steps"]) for r in old.values())
after = sum(int(r["merge_steps"]) for r in leaf.values())
print("cursor visits",before,after,"fraction removed",1-after/before)
with (HERE/"phase.csv").open() as f:
    phase = {(int(r["round"]),r["stream"],r["arm"]):r for r in csv.DictReader(f)}
blocks = sorted({b for _,b,_ in phase})
assert len(blocks)==9 and len(phase)==32*9*4
for field, controls in (("merge_ns",("stream","refill")),("total_ns",("stream","refill","lazy"))):
    for baseline in controls:
        sums = {a:[sum(int(phase[r,b,a][field]) for b in blocks) for r in range(1,33)] for a in ("leaf",baseline)}
        res = paired_ratio(sums["leaf"],sums[baseline])
        print(field,"leaf /",baseline,*(f"{res[k]:.5f}" for k in ("geomean","lo","hi")))
