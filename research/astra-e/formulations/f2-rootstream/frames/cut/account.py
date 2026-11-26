"""Resource ledger for the remaining construction and a zero-cost merge screen."""
import csv
from collections import defaultdict
from itertools import groupby
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent
FRAMES=HERE.parent
FORM=FRAMES.parents[1]
sys.path.insert(0,str(HERE.parents[5]/"bench/moonshot"))
from moonshot import paired_ratio

def rows(path):
    with path.open() as f: return list(csv.DictReader(f))

def report(name,values):
    print(name,*(f"{values[x]:.5f}" for x in ("geomean","lo","hi")))

a=[r for r in rows(FORM/"f2b/geometry.csv") if r["took"]=="1"]
b=[r for r in rows(FORM/"f2c/native.csv") if r["took"]=="1"]
assert len(a)==len(b)==228
print("first geometry (frozen, pooled)")
for field in ("run_count","nodes","edges","selectors","words","dict_bytes","root_runs","token_rows","lookups"):
    print(field,a[0][field],sum(int(r[field]) for r in a))
print("second geometry (frozen, pooled)")
for field in ("lookups","copy_spans","final_visits","token_rows","dict_bytes","physical_nodes","merged_pieces"):
    print(field,b[0][field],sum(int(r[field]) for r in b))

phase=rows(FRAMES/"phase.csv")
idx={(int(r["round"]),int(r["stream"]),r["arm"]):r for r in phase}
blocks=sorted({b for _,b,_ in idx})
fields=("prep_mark_ns","discover_ns","merge_ns","residual_output_ns")
print("phase (frozen, systematic observations) seconds then fraction of full inverse")
for group in ([blocks[0]],blocks):
    sums={f:[sum(int(idx[j,b,"frames"][f]) for b in group) for j in range(1,41)]
          for f in (*fields,"total_ns")}
    lazy=[sum(int(idx[j,b,"lazy"]["total_ns"]) for b in group) for j in range(1,41)]
    for f in fields:
        print(group[0],len(group),f,round(sum(sums[f])/40/1e9,7),round(sum(sums[f])/sum(sums["total_ns"]),5))
    remaining=[x-y for x,y in zip(sums["total_ns"],sums["merge_ns"])]
    report("zero-cost merge / Lazy",paired_ratio(remaining,lazy))
    report("observed frames / Lazy",paired_ratio(sums["total_ns"],lazy))
    print("mean residual additional saving needed vs Lazy for .80",round(1-sum(lazy)*.8/sum(remaining),5))

print("supported oracle fraction of frozen/real merge work saved at <=10% of full extra storage")
for block,subset in groupby(rows(HERE/"frontier.csv"),key=lambda r:r["block"]):
    points=[(int(r["extra_pieces"]),int(r["work"])) for r in subset]
    m,w=points[-1]
    am,aw=[p for p in points if p[0]<=m//10][-1]
    print(block,am,aw,round(1-aw/points[0][1],5))
