from collections import Counter,defaultdict
from pathlib import Path
from math import exp,log,sqrt
from statistics import mean,stdev,median
import csv,json
here=Path(__file__).resolve().parent
obs=defaultdict(Counter);order=Counter()
with (here/'allocation.csv').open() as f:
    for r in csv.DictReader(f):
        a,i,p=(int(r[k]) for k in ['arm','round','position'])
        order[r['case'],a,p]+=1
        obs[a,i].update({k:int(r[k]) for k in ['patch_ns','before_capacity','after_capacity','pointer_moved','compound_bytes']})
assert set(order.values())=={6} and len(order)==5*4*4 and len(obs)==24*4
out={}
for key,a in [('cold',1),('warm',3)]:
    z=[log(obs[a,i]['patch_ns']/obs[a-1,i]['patch_ns']) for i in range(1,25)]
    m=mean(z);h=2.06865761*stdev(z)/sqrt(len(z))
    out[key]={'reserved_over_incremental':[exp(q) for q in (m,m-h,m+h)],
              'median_us_per_lap':[median(obs[k,i]['patch_ns'] for i in range(1,25))/1000 for k in [a-1,a]],
              'capacity_B_per_lap':[median(obs[k,i]['after_capacity'] for i in range(1,25)) for k in [a-1,a]],
              'relocations_across_120_calls':[sum(obs[k,i]['pointer_moved'] for i in range(1,25)) for k in [a-1,a]]}
print(json.dumps(out,indent=2))
