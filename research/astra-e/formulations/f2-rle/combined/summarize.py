from collections import Counter
import csv,math,statistics,sys
from pathlib import Path
d=Path(__file__).resolve().parent
tag=sys.argv[1] if len(sys.argv)>1 else ""
g=list(csv.DictReader((d/("capacity"+tag+".csv")).open()))
arms=["opend","ahead","bundle","combo"]
assert len(g)==228*4
for a in arms:
    stream=[x for x in g if x["arm"]==a]
    print(a,"kernel max",max(int(x["kernel_bytes"]) for x in stream),"coded max",max(int(x["coded_capacity"]) for x in stream),
          "output max",max(int(x["output_capacity"]) for x in stream),"total max",max(int(x["retained_bytes"]) for x in stream),
          "frozen",stream[0]["retained_bytes"])
r=list(csv.DictReader((d/("phases"+tag+".csv")).open()))
assert len(r)==9*32*4
for a in arms:
    for pos in range(4):assert sum(x["arm"]==a and int(x["position"])==pos for x in r)==9*8
for k in ("wall_ns","cpu_ns"):
    t={a:[sum(int(x[k]) for x in r if x["arm"]==a and int(x["round"])==i) for i in range(1,33)] for a in arms}
    for a,b in [("ahead","opend"),("bundle","opend"),("combo","opend"),("combo","ahead"),("combo","bundle")]:
        log=[math.log(x/y) for x,y in zip(t[a],t[b])];m=statistics.mean(log);rad=2.039513*statistics.stdev(log)/math.sqrt(32)
        print(k,a+"/"+b,*[round(math.exp(v),5) for v in [m,m-rad,m+rad]])
