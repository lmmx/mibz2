"""Reproduce the complete two-pass comparison using matched per-round class weights."""
import csv
from collections import Counter,defaultdict
from math import exp,log,sqrt
from pathlib import Path
from statistics import mean,stdev
import sys
d=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else Path(__file__).resolve().parent
with (d/"panel.csv").open() as f: rows=list(csv.DictReader(f))
paid=[r for r in rows if r['round']!='0']
panel=Counter(int(r['stream'])for r in paid if r['arm']=='0')
assert Counter(panel.values())=={3:228,24:5}
print('228 ordinary real blocks measured three times each; all five additional dense blocks 24 times')
for group in ["ordinary","extra","weighted"]:
    totals=defaultdict(float)
    counts=Counter((r['round'],r['class']) for r in paid if r['arm']=='0')
    for r in paid:
        c=r['class'];rnd=r['round'];a=int(r['arm'])
        if group!='weighted' and group!=c:continue
        j=int(rnd)-1;p=int(r['position'])
        assert a==(j%4+(p if j//4%2==0 else 4-p))%4
        w=(228 if c=='ordinary'else 5)/counts[rnd,c] if group=='weighted' else 1
        totals[int(rnd),a]+=w*int(r['cpu_ns'])
    for a,b in [(1,0),(2,0),(3,0),(2,3)]:
        values=[log(totals[j,a]/totals[j,b])for j in range(1,25)]
        m=mean(values);h=2.068658*stdev(values)/sqrt(24)
        print(group,f"{a}/{b}",*[round(exp(x),5) for x in (m,m-h,m+h)])
