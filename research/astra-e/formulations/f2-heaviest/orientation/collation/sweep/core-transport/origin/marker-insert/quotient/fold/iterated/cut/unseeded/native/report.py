"""Compare sums across ten tapes within each paired round, then use eight-round clusters."""
import csv
import json
from math import exp,log,sqrt
from pathlib import Path
from statistics import mean,median,stdev
import sys
folder=Path(sys.argv[1])
times={}
with (folder/'pairs.csv').open() as f:
    for r in csv.DictReader(f):
        key=(int(r['round']),r['tag'],r['arm'])
        assert key not in times
        times[key]=(int(r['cpu_ns']),int(r['wall_ns']))
arms=('singleton','split-ordinal','unseeded','dense')
tags=sorted({key[1] for key in times})
assert len(tags)==10 and len(times)==40*10*4
report={}
def paired(v):
    clusters=[mean(v[i:i+8]) for i in range(0,40,8)]
    m=mean(clusters);half=2.7764451051977987*stdev(clusters)/sqrt(5)
    return [round(exp(x),6) for x in [m,m-half,m+half]]
for a,b in [('split-ordinal','singleton'),('unseeded','split-ordinal'),('unseeded','singleton'),
            ('unseeded','dense'),('singleton','dense')]:
    entry={}
    for j,name in enumerate(('cpu','wall')):
        data=[log(sum(times[r,t,a][j] for t in tags)/sum(times[r,t,b][j] for t in tags)) for r in range(40)]
        entry[name]=paired(data)
    entry['per_tape_cpu']={t:paired([log(times[r,t,a][0]/times[r,t,b][0]) for r in range(40)]) for t in tags}
    report[a+'/'+b]=entry
rss=list(csv.DictReader((folder/'rss.csv').open()))
assert len(rss)==40
for arm in arms:
    v=[int(r['peak_kib']) for r in rss if r['arm']==arm]
    report[arm]=dict(peak_kib=max(v),median_peak_kib=median(v))
print(json.dumps(report,indent=2))
