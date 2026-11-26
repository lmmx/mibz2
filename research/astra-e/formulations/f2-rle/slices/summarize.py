from pathlib import Path
from collections import Counter
import csv,statistics,math,sys
d=Path(__file__).resolve().parent;rep=sys.argv[1] if len(sys.argv)>1 else ""
g=list(csv.DictReader((d/("capacity"+rep+".csv")).open()))
r=list(csv.DictReader((d/("phases"+rep+".csv")).open()))
assert len(g)==912 and len(r)==1152
patch=[x for x in g if x["arm"]=="patch"]
print("outcomes",dict(Counter(x["reason"] for x in patch)))
done=[x for x in patch if x["reason"]=="0"]
for k in ("events","patches","patched_spelling_bytes"):
    print(k,"admissions sum",sum(int(x[k]) for x in done),"frozen",done[0][k],"high",max(int(x[k]) for x in done))
ars=("opend","ahead","combo","patch")
for a in ars:
    print(a,"peak retained bytes",max(int(x["retained_bytes"]) for x in g if x["arm"]==a))
for f in ("wall_ns","cpu_ns"):
    sums={a:[sum(int(z[f]) for z in r if z["arm"]==a and int(z["round"])==i) for i in range(1,33)] for a in ars}
    for a,b in [("ahead","opend"),("combo","opend"),("combo","ahead"),("patch","opend"),("patch","ahead"),("patch","combo")]:
        log=[math.log(x/y) for x,y in zip(sums[a],sums[b])];m=statistics.mean(log);h=2.039513*statistics.stdev(log)/math.sqrt(32)
        print(f,a+"/"+b,*[round(math.exp(z),5) for z in (m,m-h,m+h)])
