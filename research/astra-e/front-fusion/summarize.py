from pathlib import Path
from collections import Counter
from statistics import mean,stdev
from math import log,exp,sqrt
import csv,sys
d=Path(__file__).resolve().parent
f=Path(sys.argv[1])
data=list(csv.DictReader(f.open()))
warm={int(z["stream"]):z for z in data if z["arm"]=="1" and z["round"]=="0"}
def kind(z):
    if z["fused"]=="true":return "fused"
    if int(z["caps"])==0:return "column"
    return "large-replay" if int(z["n"])>=65536 else "short-stock"
groups=Counter(kind(z)for z in warm.values())
rows=Counter()
for z in warm.values(): rows[kind(z)]+=int(z["n"])
print("all block counts",dict(groups),"raw rows",dict(rows))
timed=[z for z in data if int(z["round"])>0]
if timed:
    selected=Counter(kind(warm[i]) for i in {int(z["stream"])for z in timed})
    print("timed counts",dict(selected))
    rounds=max(int(z["round"])for z in timed)
    print("paired thread CPU ratios across",rounds,"rounds; geometric means and 95% log-t intervals")
    crit={18:2.109816,24:2.068658,30:2.04523,36:2.030108}[rounds]
    for group in (*groups,"panel","weighted-corpus"):
        observations=[]
        for z in timed:
            g=kind(warm[int(z["stream"])])
            if group not in (g,"panel","weighted-corpus"):continue
            weight=groups[g]/selected[g] if group=="weighted-corpus" else 1
            observations.append((int(z["round"]),int(z["arm"]),int(z["cpu_ns"])*weight))
        for candidate in [1,2]:
            logs=[]
            for i in range(1,rounds+1):
                a=sum(t for r,k,t in observations if r==i and k==candidate)
                b=sum(t for r,k,t in observations if r==i and k==0)
                logs.append(log(a/b))
            m=mean(logs);h=crit*stdev(logs)/sqrt(rounds)
            print(group,candidate,*[round(exp(x),5)for x in(m,m-h,m+h)])
