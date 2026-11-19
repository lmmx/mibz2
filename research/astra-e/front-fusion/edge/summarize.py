from collections import Counter,defaultdict
from statistics import mean,stdev
from math import log,exp,sqrt
from pathlib import Path
import csv,sys
d=Path(__file__).resolve().parent
r=list(csv.DictReader(Path(sys.argv[1]).open()));warm={int(z["stream"]):z for z in r if z["arm"]=="0" and z["round"]=="0"}
def kind(i):
    z=warm[i];n=int(z["n"]);caps=int(z["caps"])
    if not caps:return "column"
    if n<65536:return "short"
    return "edge-only" if caps*20>=3*n else "ordinary"
total=Counter(kind(i)for i in warm)
sel=Counter(kind(i)for i in {int(z["stream"]) for z in r if int(z["round"])>0})
print("corpus",dict(total),"panel",dict(sel))
timed=[z for z in r if int(z["round"])>0]
R=max(int(z["round"])for z in timed)
edge_ids = sorted(i for i in warm if kind(i) == "edge-only")
for group in [*total, "panel", "weighted", *edge_ids]:
    for a,b in [(1,0),(2,0),(2,1)]:
        sums=defaultdict(float)
        for z in timed:
            g=kind(int(z["stream"]));arm=int(z["arm"])
            if arm not in (a,b) or group not in (g, "weighted", "panel", int(z["stream"])):continue
            w=total[g]/sel[g] if group=="weighted"else 1
            sums[(int(z["round"]),arm)]+=w*int(z["cpu_ns"])
        logs=[log(sums[(i,a)]/sums[(i,b)])for i in range(1,R+1)]
        m=mean(logs);h=2.068658*stdev(logs)/sqrt(R)
        print(group,f"{a}/{b}",*[round(exp(x),5) for x in (m,m-h,m+h)])
