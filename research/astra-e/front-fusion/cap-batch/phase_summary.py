import csv
from collections import Counter,defaultdict
from math import exp,log,sqrt
from pathlib import Path
from statistics import mean,median,stdev
here=Path(__file__).resolve().parent
rows=list(csv.DictReader((here/"phase.csv").open()))
scope=[z for z in rows if int(z['round'])>0]
coverage=Counter(int(z['stream']) for z in scope if z['arm']=='0')
assert Counter(coverage.values())=={3:228,24:5}
assert all(int(z["phase_ns"])>0 for z in scope)
print("coverage: all 228 ordinary blocks three times; all five additional dense blocks 24 times")
for metric in ["phase_ns","cpu_ns"]:
    for group in ["ordinary","extra","weighted"]:
        sums=defaultdict(float);samples=Counter((z["round"],z["class"])for z in scope if z["arm"]=="0")
        for z in scope:
            c=z["class"];r=z["round"];arm=int(z["arm"])
            if group!="weighted" and group!=c:continue
            w=(228 if c=="ordinary"else 5)/samples[r,c] if group=="weighted"else 1
            sums[int(r),arm]+=w*int(z[metric])
        for a,b in [(1,0),(2,0),(3,0),(3,2)]:
            logs=[log(sums[r,a]/sums[r,b])for r in range(1,25)];m=mean(logs);h=2.068658*stdev(logs)/sqrt(24)
            print(metric,group,f"{a}/{b}",*[round(exp(x),5)for x in (m,m-h,m+h)])
for arm in range(4):
    sample=[z for z in rows if z['arm']==str(arm) and z['round']=='0']
    print("single full-corpus phase / parsed-symbols-through-CRC",arm,round(sum(int(z['phase_ns'])for z in sample)/sum(int(z['cpu_ns'])for z in sample),5))
