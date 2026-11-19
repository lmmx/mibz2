from pathlib import Path
import math,statistics,csv
p=Path(__file__).resolve().parent
rows=list(csv.DictReader((p/"phases.csv").open()))
assert len(rows)==9*3*21
for field in ["wall_ns","cpu_ns"]:
    sums={arm:[sum(int(r[field]) for r in rows if r["arm"]==arm and int(r["round"])==i) for i in range(1,22)] for arm in ["laps","compiled","opend"]}
    for a,b in [("laps","compiled"),("laps","opend"),("compiled","opend")]:
        ratios=[math.log(i/j) for i,j in zip(sums[a],sums[b])]
        m=statistics.mean(ratios);h=2.085963*statistics.stdev(ratios)/math.sqrt(len(ratios))
        print(field,a+"/"+b,*[round(math.exp(x),5) for x in (m,m-h,m+h)])
