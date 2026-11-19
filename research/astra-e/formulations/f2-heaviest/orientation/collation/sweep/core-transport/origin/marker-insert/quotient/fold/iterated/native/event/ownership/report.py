"""Check paired rotations and derive full-stage and free-emitter ceilings."""
from collections import Counter,defaultdict
from math import exp,log,sqrt
from statistics import mean,stdev
from pathlib import Path
import csv,sys
d=Path(sys.argv[1]);read=lambda f:list(csv.DictReader((d/f).open()))
for a in range(4):
 r=read(f'rss-{a}.csv');assert len(r)==228
 print(r[0]['arm'],'max capacity',max(int(x['total_capacity']) for x in r),'max RSS KiB',max(int(x['hwm']) for x in r))
for f in ['ten.csv','rss-3.csv']:
 r=read(f)
 for k in ['n','period','leaf_copies','leaf_bytes','copied_bytes','hits','partial_hits','partial_misses','visits','published','owner_slots','owner_bytes','stack']:
  v=[int(x[k]) for x in r];print(f,k,'sum',sum(v),'min',min(v),'max',max(v))
def ratios(totals,pairs):
 for a,b in pairs:
  per=[log(totals[i,a]/totals[i,b]) for i in range(32)]
  cs=[mean(per[i:i+4]) for i in range(0,32,4)];c=mean(cs);r=2.364624251*stdev(cs)/sqrt(8)
  print(f'{a}/{b}={exp(c):.4f} [{exp(c-r):.4f},{exp(c+r):.4f}]')
for scope in ['phase','resident']:
 r=read(scope+'.csv');assert len(r)==1280
 bal=Counter();totals=defaultdict(int);parts=defaultdict(Counter)
 for v in r:
  c,rep,st,p=map(int,(v['cluster'],v['rep'],v['stream'],v['position']));assert c==rep//4
  a=v['arm'];bal[c,st,p,a]+=1;m=int(v['total_ns'] if scope=='phase' else v['emit_ns']);totals[rep,a]+=m
  for k in ['total_ns','first_ns','index_ns','contract_ns','emit_ns']:
   if k in v:parts[a][k]+=int(v[k])
  if scope=='phase':
   if a=='dense':totals[rep,'free_emit']+=m-int(v['emit_ns'])
   if a=='owner':totals[rep,'owner_emit_only']+=int(v['emit_ns'])
   if a=='named':totals[rep,'named_second']+=int(v['emit_ns'])
 assert len(bal)==1280 and all(x==1 for x in bal.values())
 for a,v in parts.items():print(scope,a,' '.join(f'{k}={v[k]/320/1e3:.2f}us' for k in v))
 pairs=[('owner','dense'),('owner','named'),('owner','indexed'),('free_emit','dense'),('free_emit','named'),('owner_emit_only','named_second')] if scope=='phase' else [('owner_a','plain_a'),('owner_b','plain_b'),('plain_b','plain_a'),('owner_b','owner_a')]
 ratios(totals,pairs)
