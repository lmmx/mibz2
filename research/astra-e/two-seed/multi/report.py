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
assert len(blocks)==14 and len(balance)==14*64 and set(balance.values())=={5}
def ci(logs):
    t=2.022690920
    m=statistics.mean(logs);h=t*statistics.stdev(logs)/math.sqrt(len(logs))
    return [round(math.exp(v),6) for v in (m,m-h,m+h)]
report={}
for g in ['all','ordinary','dense']+[str(b) for b in sorted(blocks)]:
    results={}
    for a,b in [('stock twin','stock'),('two twin','two'),('three twin','three'),('four twin','four')]:
        results[f'{a}/{b}']=ci([math.log(tot[g,k,a]/tot[g,k,b]) for k in range(1,41)])
    for a,b in [('two','stock'),('three','stock'),('four','stock'),('three','two'),('four','two'),('four','three')]:
        zs=[]
        for k in range(1,41):
            logmean=lambda arm: (math.log(tot[g,k,arm])+math.log(tot[g,k,arm+' twin']))/2
            zs.append(logmean(a)-logmean(b))
        results[f'paired {a}/{b}']=ci(zs)
    report[g]=results
print(json.dumps(report,indent=2))
