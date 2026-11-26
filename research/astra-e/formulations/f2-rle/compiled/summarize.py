from pathlib import Path
import csv, statistics, math, collections
d=Path(__file__).resolve().parent
g=list(csv.DictReader((d/"native.csv").open()))
print("large density admissions",len(g),"bytes",sum(int(x["rows"]) for x in g))
print("outcomes",dict(collections.Counter(x["fallback_reason"] for x in g)))
print("navigation capacity frozen",g[0]["nav_capacity"],"maximum",max(int(x["nav_capacity"]) for x in g))
rows=list(csv.DictReader((d/"phases.csv").open()))
assert len(rows)==360
for field in ["wall_ns","cpu_ns","constructor_ns","navigation_ns","owner_ns","unrle_ns","crc_ns"]:
    sums={a:[sum(int(r[field]) for r in rows if r["arm"]==a and int(r["round"])==i) for i in range(1,21)] for a in ["sparse","opend"]}
    if field in ("wall_ns","cpu_ns"):
        v=[math.log(a/b) for a,b in zip(sums["sparse"],sums["opend"])]
        m=statistics.mean(v);rad=2.093024*statistics.stdev(v)/math.sqrt(len(v))
        print(field, "paired",*[round(math.exp(x),4) for x in (m,m-rad,m+rad)])
    print(field,*(f"{a} {statistics.median(s)/1e6:.5f} ms" for a,s in sums.items()))
