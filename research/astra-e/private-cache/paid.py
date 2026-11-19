"""Summarize by complete paired lap. 24 rotations are the sampling units."""
import csv
from collections import Counter, defaultdict
from math import exp, log, sqrt
from pathlib import Path
from statistics import mean, stdev
import sys
path=Path(__file__).resolve().parent
totals=defaultdict(int)
placed=Counter()
limits=defaultdict(dict)
with (path/'paid.csv').open() as f:
    for x in csv.DictReader(f):
        r={k:int(v) for k,v in x.items()}
        i,b,p,a,t=r['round'],r['block'],r['position'],r['arm'],r['limit']
        placed[b,p,a]+=1
        limits[b][a]=t
        totals[i,a]+=r['cpu_ns']
assert len(limits)==12 and len(placed)==12*8*8 and set(placed.values())=={3} and len(totals)==24*8
names=['unchanged','same_code','equal_charge','cross256','above','zero','below','cross192']
out=csv.writer(sys.stdout,lineterminator='\n')
out.writerow(['arm','versus','CPU_ratio','lo95','hi95','laps'])
for a in range(1,8):
    for ref in ([0,2] if a>=3 else [0]):
        z=[log(totals[i,a]/totals[i,ref]) for i in range(1,25)]
        g=mean(z); radius=2.06865761*stdev(z)/sqrt(24)
        out.writerow([names[a],names[ref],*(f'{exp(x):.6f}' for x in [g,g-radius,g+radius]),24])
with (path/'panel-selection.csv').open('w') as f:
    w=csv.writer(f,lineterminator='\n')
    w.writerow(['block',*names])
    for b in sorted(limits):w.writerow([b,*(limits[b][i] for i in range(8))])
