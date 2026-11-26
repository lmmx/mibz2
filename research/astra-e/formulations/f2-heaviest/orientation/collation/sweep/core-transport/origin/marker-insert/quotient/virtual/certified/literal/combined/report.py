from collections import Counter,defaultdict
from math import exp,log,sqrt
from pathlib import Path
from statistics import mean,stdev
import csv
import sys
d=Path(sys.argv[1]);rows=list(csv.DictReader((d/'phase.csv').open()))
assert len(rows)==5120
t=defaultdict(int);pos=Counter();cell=set()
for r in rows:
 i=int(r['round']);c=int(r['cluster']);a=r['arm'];s=int(r['stream'])
 key=(i,s,a);assert key not in cell and i//8==c;cell.add(key)
 pos[c,s,int(r['slot']),a]+=1
 for name in ('whole_ns','post_first_ns','index_ns','discovery_ns','second_merge_ns','second_output_ns'):
  t[i,a,name]+=int(r[name])
assert len(pos)==5120 and set(pos.values())=={1}
with (d/'clusters.csv').open('w') as f:
 w=csv.writer(f,lineterminator='\n');w.writerow(('h','contrast','metric','cluster','log_ratio'))
 for h in ('h4','h12'):
  for label,pos,neg in [('both/explicit',['both'],['explicit']),('both/child',['both'],['child']),('both/direct',['both'],['direct']),('child/explicit',['child'],['explicit']),('direct/explicit',['direct'],['explicit']),('interaction',['both','explicit'],['child','direct'])]:
   for k in ('whole_ns','post_first_ns','index_ns','discovery_ns','second_merge_ns','second_output_ns'):
    logs=[sum(log(t[i,h+'_'+a,k]) for a in pos)-sum(log(t[i,h+'_'+a,k]) for a in neg) for i in range(64)]
    groups=[mean(logs[i:i+8]) for i in range(0,64,8)]
    for c,v in enumerate(groups):w.writerow((h,label,k,c,v))
    center=mean(groups);half=2.3646242515927844*stdev(groups)/sqrt(8)
    print(h,label,k,*(f'{exp(x):.5f}' for x in (center,center-half,center+half)))
