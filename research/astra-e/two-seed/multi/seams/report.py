"""Verify position balance and aggregate paired log ratios over complete laps."""
from collections import defaultdict, Counter
from pathlib import Path
import csv
import json
import math
import statistics

d=Path(__file__).resolve().parent
tot=defaultdict(int)
balance=Counter()
blocks=set()
with (d/'paid.csv').open() as f:
    for r in csv.DictReader(f):
        k=int(r['round']);b=int(r['block']);a=r['arm'];v=int(r['cpu_ns'])
        blocks.add(b)
        balance[b,int(r['position']),a]+=1
        for g in ('all','dense' if b in (41,56,165,270,278) else 'ordinary',str(b)):
            tot[g,k,a]+=v
assert len(blocks)==14 and len(balance)==14*36 and set(balance.values())=={6}
def ci(logs):
    t=2.030107928
    m=statistics.mean(logs);h=t*statistics.stdev(logs)/math.sqrt(len(logs))
    return [round(math.exp(v),6) for v in (m,m-h,m+h)]
report={}
for g in ['all','ordinary','dense']+[str(b) for b in sorted(blocks)]:
    results={}
    for a,b in [('stock twin','stock'),('joined twin','joined'),('sliced twin','sliced')]:
        results[f'{a}/{b}']=ci([math.log(tot[g,k,a]/tot[g,k,b]) for k in range(1,37)])
    for a,b in [('sliced','stock'),('joined','stock'),('sliced','joined')]:
        zs=[]
        for k in range(1,37):
            logmean=lambda arm: (math.log(tot[g,k,arm])+math.log(tot[g,k,arm+' twin']))/2
            zs.append(logmean(a)-logmean(b))
        results[f'paired {a}/{b}']=ci(zs)
    report[g]=results
print(json.dumps(report,indent=2))
