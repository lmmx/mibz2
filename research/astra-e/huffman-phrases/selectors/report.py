"""Replay the five-arm small-block price and independent selector-use census."""
from collections import Counter
import csv
from pathlib import Path
import sys
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[3]/"bench/moonshot"))
from moonshot import paired_ratio
arms=["stock","twin","scatter","pairtiny","singletiny"]
data=list(csv.DictReader((here/"times.csv").open()))
assert len(data)==200 and len({r["metadata_fold"]for r in data})==1
table={}
for k,r in enumerate(data):
    lap,pos=divmod(k,5)
    expect=arms[(lap+(pos if (lap//5)%2==0 else 4-pos))%5]
    assert r["arm"]==expect and (r["blocks"],r["symbols"])==("431","24979299")
    assert int(r["round"])==lap and int(r["position"])==pos
    table[lap,expect]=r
assert set(Counter((r["position"],r["arm"])for r in data).values())=={8}
out=csv.writer(sys.stdout,lineterminator="\n")
out.writerow(["metric","candidate","reference","paired_rounds","geo","lo95","hi95"])
for metric in ("thread_ns","wall_ns"):
    for a,b in [("twin","stock"),("scatter","stock"),("pairtiny","stock"),("singletiny","stock"),("pairtiny","twin"),("singletiny","twin"),("pairtiny","scatter"),("singletiny","scatter"),("singletiny","pairtiny")]:
        c=[int(table[i,a][metric])for i in range(40)]
        ref=[int(table[i,b][metric])for i in range(40)]
        r=paired_ratio(c,ref)
        out.writerow([metric,a,b,40,*[f"{r[k]:.8f}"for k in ("geomean","lo","hi")]])
usage=[{k:int(v)for k,v in row.items()}for row in csv.DictReader((here/"usage.csv").open())]
assert len(usage)==1796 and sum(r["emitted_symbols"]for r in usage)==24979299
assert all(r["listed_uses"]>=r["consumed_uses"] for r in usage)
small=[r for r in usage if r["declared_selectors"]<=64]
print("small blocks/tables/symbols",len({r["block"]for r in small}),len(small),sum(r["emitted_symbols"]for r in small),file=sys.stderr)
print("zero listed / zero used",sum(not r["listed_uses"]for r in usage),sum(not r["consumed_uses"]for r in usage),file=sys.stderr)
