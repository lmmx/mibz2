"""Four-factor comparisons with paired per-round sums over nine inputs."""
import csv
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[4]/"bench/moonshot"))
from moonshot import paired_ratio

with (HERE/"phase.csv").open() as f:
    phase={(int(r["round"]),r["stream"],r["arm"]):r for r in csv.DictReader(f)}
blocks=sorted({b for _,b,_ in phase})
assert len(blocks)==9 and len(phase)==40*9*5
for field in ("merge_ns","total_ns"):
    arms=["leaf","maximal","both","refill"]+(["lazy"] if field=="total_ns" else [])
    samples={a:[sum(int(phase[r,b,a][field]) for b in blocks) for r in range(1,41)] for a in arms}
    pairs=[("leaf","refill"),("maximal","refill"),("both","refill"),("both","leaf"),("both","maximal")]
    if field=="total_ns":
        pairs.extend([("both","lazy"),("leaf","lazy"),("maximal","lazy")])
    for a,b in pairs:
        res=paired_ratio(samples[a],samples[b])
        print(field,a,"/",b,*(f"{res[k]:.5f}" for k in ("geomean","lo","hi")))

def accepted(path):
    with path.open() as f:
        return {int(r["stream"]):r for r in csv.DictReader(f) if r["took"]=="1"}
control=accepted(HERE.parents[1]/"f2b/geometry.csv")
current=accepted(HERE/"geometry.csv")
assert len(current)==228 and current.keys()==control.keys()
for i,c in current.items():
    p=control[i]
    for field in ("nodes","words","selectors","root_runs","token_rows","dict_bytes"):
        assert c[field]==p[field]
    assert int(c["merged"])==int(c["merge_steps"])<int(p["merged"])
baseline=sum(int(c["merged"]) for c in control.values())
reduced=sum(int(c["merged"]) for c in current.values())
print("materialized local pairs",baseline,reduced,"fraction removed",1-reduced/baseline)
sys.path.insert(0,str(HERE.parent/"cut"))
from frontier import load
with (HERE.parent/"audit/real.csv").open() as f:
    examples={int(r["block"]):r for r in csv.DictReader(f)}
by_mark={(int(c["rows"]),int(c["primary"])):c for c in current.values()}
for b,m in load(HERE.parent/"cut/nodes.csv.xz").items():
    p=examples[b]
    k=int(p["rows"]),int(p["primary"])
    if k in by_mark:
        assert int(by_mark[k]["merged"])==sum(m.r)-sum(not m.children[v] for v in range(1,m.n))
