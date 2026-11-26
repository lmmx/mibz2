from pathlib import Path
import math,csv,collections,statistics
d=Path(__file__).resolve().parent
rows=list(csv.DictReader((d/"phases.csv").open()))
assert len(rows)==567
for field in ("wall_ns","cpu_ns"):
    all_times={a:[sum(int(x[field]) for x in rows if x["arm"]==a and int(x["round"])==i) for i in range(1,22)] for a in ("bundle","opend","laps")}
    for a,b in [("bundle","opend"),("bundle","laps"),("laps","opend")]:
        t=[math.log(x/y) for x,y in zip(all_times[a],all_times[b])];m=statistics.mean(t);h=2.085963*statistics.stdev(t)/math.sqrt(21)
        print(field,a+"/"+b,*[round(math.exp(v),5) for v in (m,m-h,m+h)])
for dirname in (d,d.parent/"laps"):
    data=list(csv.DictReader((dirname/"native.csv").open()))
    print(dirname.name,"outcomes",dict(collections.Counter(x["fallback_reason"] for x in data)),"steps",
          sum(int(x["rewinds"]) for x in data),"cap bytes",max(int(x["nav_capacity"]) for x in data))
    for idx in ("1","134","397"):
        r=next(row for row in data if row["stream"]==idx);print(idx,r["rewinds"],r["owners"],r["fallback_reason"])
