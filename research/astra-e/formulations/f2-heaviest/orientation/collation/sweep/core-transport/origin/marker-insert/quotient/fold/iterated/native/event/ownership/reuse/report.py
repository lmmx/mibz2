import csv, math, statistics as st,sys
from collections import Counter,defaultdict
from pathlib import Path
d=Path(sys.argv[1]);read=lambda f:list(csv.DictReader((d/f).open()))
for a in range(4):
 r=read(f'rss-{a}.csv');print(r[0]['arm'],'max capacity',max(int(v['total_capacity']) for v in r),'RSS KiB',max(int(v['hwm']) for v in r),'extra cache',max(int(v['owner_bytes']) for v in r),'borrowed capacity',max(int(v['borrowed_bytes']) for v in r))
p=read('phase.csv');assert len(p)==1280
bal=Counter();vals=defaultdict(int);parts=defaultdict(Counter)
for v in p:
 r=int(v['rep']);c=int(v['cluster']);assert c==r//4;k=v['arm']
 bal[c,int(v['stream']),int(v['position']),k]+=1;vals[r,k]+=int(v['total_ns'])
 for fld in ['total_ns','first_ns','index_ns','contract_ns','emit_ns']:parts[k][fld]+=int(v[fld])
assert len(bal)==1280 and all(x==1 for x in bal.values())
for k,v in parts.items():print(k,' '.join(f'{n}={z/320/1000:.2f}us' for n,z in v.items()))
for a,b in [('reuse','dense'),('side_a','dense'),('reuse','side_a'),('reuse','side_b'),('side_b','side_a')]:
 z=[math.log(vals[r,a]/vals[r,b]) for r in range(32)]
 c=[st.mean(z[i:i+4]) for i in range(0,32,4)];v=st.mean(c);h=2.364624251*st.stdev(c)/math.sqrt(8)
 print(f'{a}/{b}={math.exp(v):.4f} [{math.exp(v-h):.4f},{math.exp(v+h):.4f}]')
