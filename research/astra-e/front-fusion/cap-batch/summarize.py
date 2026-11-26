import csv
from collections import Counter, defaultdict
from math import exp, log, sqrt
from pathlib import Path
from statistics import mean,stdev

base=Path(__file__).resolve().parent
with (base/"panel.csv").open() as f:
    rows=list(csv.DictReader(f))
sizes=Counter(r["class"] for r in rows if r["round"]=="0" and r["arm"]=="0")
selected=Counter(r["class"] for r in rows if r["round"]=="1" and r["arm"]=="0")
print("population",dict(sizes),"selected",dict(selected))
assert sizes==Counter(ordinary=228,extra=5,short=150,column=48)
assert selected==Counter(ordinary=9,extra=5,short=150,column=48)
for kind in [*sizes,"weighted"]:
    costs=defaultdict(float)
    for row in rows:
        rnd=int(row["round"])
        if not rnd or kind not in [row["class"],"weighted"]:continue
        arm=int(row["arm"])
        r=rnd-1;pos=int(row["position"])
        assert arm==(r%4+(pos if r//4%2==0 else 4-pos))%4
        w=sizes[row["class"]]/selected[row["class"]] if kind=="weighted" else 1
        costs[rnd,arm]+=w*int(row["cpu_ns"])
    for a,b in [(1,0),(2,0),(3,0),(3,2)]:
        values=[log(costs[i,a]/costs[i,b]) for i in range(1,25)]
        m=mean(values);half=2.068658*stdev(values)/sqrt(24)
        print(kind,f"{a}/{b}",*[round(exp(x),5)for x in (m,m-half,m+half)])
