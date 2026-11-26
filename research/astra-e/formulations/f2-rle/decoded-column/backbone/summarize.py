from collections import Counter
from pathlib import Path
from statistics import mean,stdev
from math import log,exp,sqrt
import csv
d=Path(__file__).resolve().parent
g=list(csv.DictReader((d/"capacity.csv").open()));r=list(csv.DictReader((d/"phases.csv").open()))
assert len(g)==912 and len(r)==1152
u=[z for z in g if z["arm"]=="update"]
print("outcomes",Counter(z["reason"] for z in u))
for c in ("events","ranks","explicit","holes"):
    direct=[int(z[c]) for z in u if z["reason"]=="0"];print(c,"sum",sum(direct),"frozen",direct[0],"largest",max(direct))
arms=("opend","ahead","combo","update")
for a in arms:print("retained high water",a,max(int(z["retained_bytes"])for z in g if z["arm"]==a))
for key in ("wall_ns","cpu_ns"):
    sums={a:[sum(int(z[key]) for z in r if z["arm"]==a and int(z["round"])==i) for i in range(1,33)]for a in arms}
    for a in arms[:-1]:
        ls=[log(x/y)for x,y in zip(sums["update"],sums[a])];m=mean(ls);h=2.039513*stdev(ls)/sqrt(32)
        print(key,"update/"+a,*[round(exp(z),4) for z in (m,m-h,m+h)])
