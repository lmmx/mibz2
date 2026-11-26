from collections import Counter
import csv
from pathlib import Path
import sys
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[3]/"bench/moonshot"))
from moonshot import paired_ratio
arms=["stock","twin","stockskip","singletiny","skipboth"]
rows=list(csv.DictReader((here/"times.csv").open()))
assert len(rows)==400 and len({r["metadata_fold"]for r in rows})==1
table={}
for j,r in enumerate(rows):
    lap,pos=divmod(j,5)
    arm=arms[(lap+(pos if lap//5%2==0 else 4-pos))%5]
    assert (int(r["round"]),int(r["position"]),r["arm"])==(lap,pos,arm)
    assert (int(r["symbols"]),int(r["blocks"]))==(24979299,431)
    table[lap,arm]=r
assert set(Counter((r["arm"],r["position"])for r in rows).values())=={16}
out=csv.writer(sys.stdout,lineterminator="\n")
out.writerow(["metric","candidate","reference","paired_rounds","geo","lo95","hi95"])
for metric in ("thread_ns","wall_ns"):
    for a,b in [("twin","stock"),("stockskip","stock"),("stockskip","twin"),("singletiny","stock"),("singletiny","twin"),("skipboth","singletiny"),("skipboth","stock"),("skipboth","twin"),("skipboth","stockskip")]:
        x=[int(table[i,a][metric])for i in range(80)]
        y=[int(table[i,b][metric])for i in range(80)]
        r=paired_ratio(x,y)
        out.writerow([metric,a,b,80,*[f"{r[k]:.8f}"for k in ("geomean","lo","hi")]])
