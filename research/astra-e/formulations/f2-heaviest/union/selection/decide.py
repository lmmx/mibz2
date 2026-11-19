"""Apply the frozen decision rule to all fixed candidates."""
from collections import defaultdict
import csv
from pathlib import Path
import sys

p=Path(sys.argv[1]); raw=defaultdict(dict)
with (p/"fixed.csv").open() as f:
    for r in csv.DictReader(f):
        if r["role"]=="heldout":
            raw[r["marker"]][r["stream"]]=tuple(int(r[k]) for k in ("lookup_steps","residual_rows","spelling_bytes"))
ref="3a22";limits=(95,95,100)
base=raw[ref]
def totals(rows):
    return tuple(map(sum,zip(*rows.values())))
bst=totals(base)
def passes(x,y,pct):
    return bool(x[1]) and all(100*a<=b*r for a,b,r in zip(x,y,pct))
def strict(x,y):
    return passes(x,y,(100,100,100)) and x!=y
rows=[]
for marker,data in raw.items():
    assert set(base)==set(data) and len(base)==8
    paired=[(data[s],base[s]) for s in base]
    ratio=[a/b for a,b in zip(totals(data),bst)]
    cap=[max(a[j]/b[j] for a,b in paired) for j in range(3)]
    passpoint=all(passes(a,b,limits) for a,b in paired)
    zeropoint=all(passes(a,b,(100,100,100)) for a,b in paired) and any(strict(a,b) for a,b in paired)
    passpool=passes(totals(data),bst,limits)
    rows.append(dict(marker=marker,Q_ratio=ratio[0],rows_ratio=ratio[1],bytes_ratio=ratio[2],
                     worst_Q=cap[0],worst_rows=cap[1],worst_bytes=cap[2],
                     heldout_passes=sum(passes(a,b,limits) for a,b in paired),
                     all_margin=passpoint,all_zero=zeropoint,pooled_margin=passpool,
                     pooled_zero=strict(totals(data),bst)))
with (p/"decision.csv").open("w") as f:
    w=csv.DictWriter(f,fieldnames=list(rows[0]),lineterminator="\n");w.writeheader()
    w.writerows({k:(round(v,6) if isinstance(v,float) else v) for k,v in r.items()} for r in rows)
print("Held-out streams",*sorted(map(int,base)),"baseline paid Q, rows, bytes",*bst)
for field in ("all_margin","all_zero","pooled_margin","pooled_zero"):
    winners=[r["marker"] for r in rows if r[field]]
    print(field,len(winners),",".join(winners))
other=[r for r in rows if r["marker"]!=ref]
print("maximum individual heldout passes for another candidate",max(r["heldout_passes"] for r in other))
for k in ("Q_ratio","rows_ratio","bytes_ratio"):
    r=min(other,key=lambda r:r[k])
    print("minimum pooled",k,r["marker"],*(round(r[j],5) for j in ("Q_ratio","rows_ratio","bytes_ratio")))
print("pooled pairwise Pareto front including reference")
for r in rows:
    keys=("Q_ratio","rows_ratio","bytes_ratio")
    if not any(other is not r and all(other[k]<=r[k] for k in keys) and any(other[k]<r[k] for k in keys) for other in rows):
        print(r["marker"],*(round(r[k],5) for k in keys))
