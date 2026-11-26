import csv
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[6]/"bench/moonshot"))
from moonshot import paired_ratio
with (HERE/"phase.csv").open() as f:
    rows=list(csv.DictReader(f))
idx={(int(r["round"]),r["stream"],r["arm"]):int(r["cpu_ns"]) for r in rows}
blocks=sorted({b for _,b,_ in idx})
assert len(rows)==len(idx)==40*9*4
sums={a:[sum(idx[j,b,a] for b in blocks) for j in range(1,41)] for a in ("complete","first","cut1","cut8")}
for a,t in sums.items():print(a,"mean nine-block ms",round(sum(t)/40e6,4))
for a,b in (("cut1","complete"),("cut8","complete"),("cut1","first"),("cut8","first"),("first","complete")):
    result=paired_ratio(sums[a],sums[b])
    print(a,"/",b,*(f"{result[k]:.5f}" for k in ("geomean","lo","hi")))
