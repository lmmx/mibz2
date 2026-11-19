from pathlib import Path
from math import exp,log,sqrt
from statistics import mean,stdev
from collections import Counter,defaultdict
import csv,sys
d=Path(sys.argv[1]); read=lambda p:list(csv.DictReader((d/p).open()))
names=['named','repeated','indexed','dense','flat','tree']
rr=[read(f'rss-{a}.csv') for a in range(6)]
for a in range(6):
 r=rr[a];assert len(r)==228 and all(z['arm']==names[a] for z in r)
 assert [z['hash'] for z in r]==[z['hash'] for z in rr[3]]
 print(names[a],'peak aggregate vectors (tree uses payload lower bound):',max(int(z['total_capacity']) for z in r),
       'peak dedicated RSS KiB:',max(int(z['hwm']) for z in r))
for a in [4,5]:
 r=rr[a]
 assert all(z['terminal']=='true' and z['cold']=='0' for z in r)
 for z,y in zip(r,rr[3]):
  for f in ['stream','old','live','events','edits','moved','period','nodes','queue_pushes','pops','peak_queue','leaf_copies','stack']:
   assert z[f]==y[f],(a,f,z,y)
 for z in r:
  q=lambda name:int(z[name])
  assert q('intersections')<=q('edits') and q('source_visited')<=q('intersections')
  assert q('final_ribbons')==q('initial_ribbons')+q('left_splits')+q('right_splits')-q('source_visited')-q('coalesced')
  assert 2*q('high')<=q('old')+q('initial_ribbons')
 for f in ['old','initial_ribbons','high','incl_tmp','source_visited','intersections','left_splits','right_splits','generated','coalesced','flat_suffix_moved','table_memory','aux_capacity','first_runs','eligible_tests','owner_queries','fixed_queries']:
  v=[int(z[f]) for z in r]
  print(names[a],f,'sum',sum(v),'max',max(v))
for z,y in zip(*rr[4:]):
 for f in ['initial_ribbons','high','source_visited','intersections','left_splits','right_splits','generated','coalesced']:
  assert z[f]==y[f]
p=read('phase.csv');assert len(p)==2160
bal=Counter();totals=defaultdict(int);parts=defaultdict(Counter)
for z in p:
 c,r,st,pos=map(int,(z['cluster'],z['rep'],z['stream'],z['position']))
 assert c==r//6;bal[c,st,pos,z['arm']]+=1
 totals[r,z['arm']]+=int(z['total_ns'])
 for fld in ['total_ns','first_ns','index_ns','contract_ns','emit_ns']:parts[z['arm']][fld]+=int(z[fld])
assert len(bal)==2160 and all(v==1 for v in bal.values())
for arm,v in parts.items():print(arm,' '.join(f'{k}={v[k]/360/1e3:.2f}us' for k in v))
for a,b in [('flat','dense'),('tree','dense'),('tree','flat'),('flat','indexed'),('tree','indexed'),('flat','named'),('tree','named'),('dense','repeated')]:
 per=[log(totals[i,a]/totals[i,b]) for i in range(36)]
 cs=[mean(per[i:i+6]) for i in range(0,36,6)];z=mean(cs);h=2.570581836*stdev(cs)/sqrt(6)
 print(f'{a}/{b}={exp(z):.4f} [{exp(z-h):.4f},{exp(z+h):.4f}]')
