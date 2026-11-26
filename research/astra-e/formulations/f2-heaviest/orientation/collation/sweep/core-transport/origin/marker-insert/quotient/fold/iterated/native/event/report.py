from pathlib import Path
from collections import Counter, defaultdict
from math import exp, log, sqrt
from statistics import mean, stdev
import csv,sys
d=Path(sys.argv[1]);read=lambda p:list(csv.DictReader((d/p).open()))
real=read('rss-3.csv');assert len(real)==228
for k in ['n','old','events','edits','queue_pushes','pops','nodes','moved','leaf_copies','live','stack']:
    v=[int(r[k]) for r in real];print(f'{k} sum={sum(v)} min={min(v)} max={max(v)}')
for a in range(4):
    rr=read(f'rss-{a}.csv'); assert [r['hash'] for r in rr]==[r['hash'] for r in real]
    print(rr[0]['arm'],'max retained',max(int(r['total_capacity']) for r in rr),'aux bytes',max(int(r['aux_capacity']) for r in rr),'VmHWM KiB',max(int(r['hwm']) for r in rr))
r=read('phase.csv');assert len(r)==1280;bal=Counter();totals=defaultdict(int);parts=defaultdict(Counter)
for a in r:
    c,n,st,p=map(int,(a['cluster'],a['rep'],a['stream'],a['position']));assert c==n//4
    bal[c,st,p,a['arm']]+=1;totals[n,a['arm']]+=int(a['total_ns'])
    for f in ['total_ns','first_ns','index_ns','contract_ns','emit_ns']:parts[a['arm']][f]+=int(a[f])
assert len(bal)==1280 and all(v==1 for v in bal.values())
for a,v in parts.items():
    print(a,' '.join(f'{k}={v[k]/320/1e3:.2f}us' for k in v))
for a,b in [('event','repeated'),('event','named'),('event','indexed'),('indexed','named'),('repeated','named')]:
    per=[log(totals[i,a]/totals[i,b]) for i in range(32)]
    cs=[mean(per[i:i+4]) for i in range(0,32,4)];z=mean(cs);h=2.364624251*stdev(cs)/sqrt(8)
    print(f'{a}/{b}={exp(z):.4f} [{exp(z-h):.4f},{exp(z+h):.4f}]')
