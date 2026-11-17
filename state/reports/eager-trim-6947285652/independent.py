import json,math,statistics,itertools
from pathlib import Path
root=Path(__file__).parent
rs=[json.loads(s) for s in (root/'data/run-20261117T114640Z/judged.jsonl').read_text().splitlines()]
arms={6:['mibz2','mibz2-oce-5-20','mibz2-occ-5-20'],4:['mibz2','mibz2-oce-3-12'],10:['mibz2','mibz2-oce-7-28','mibz2-occ-7-28']}
key=lambda r:(r['threads'],r['arm'],r['corpus'],r['round'])
by={key(r):r for r in rs}
assert len(rs)==len(by)==128
assert set(by)=={(t,a,c,i) for t,aa in arms.items() for a in aa for c in ['wikidata-x4','text'] for i in range(8)}
for r in rs:
 assert r['termination']=='success' and not r['err']
 assert r['cpus']==','.join(map(str,range(r['threads'])))
 assert all(math.isfinite(r[k]) and r[k]>0 for k in ['wall','cpu','rss_kb'])
 if r['threads']<10: assert r['held_spinner']['cpu']==19 and r['held_spinner']['overlap'] is False
 else: assert r.get('held_spinner') is None
for c in ['wikidata-x4','text']:
 assert len({(r['out'],r.get('sha256') or r['hash']) for r in rs if r['corpus']==c})==1
result=[]
for t,aa in arms.items():
 for a in aa[1:]:
  for c in ['wikidata-x4','text']:
   ref=[by[t,'mibz2',c,i] for i in range(8)];cand=[by[t,a,c,i] for i in range(8)]
   logs=[math.log(x['wall']/y['wall']) for x,y in zip(cand,ref)];mu=statistics.mean(logs);h=2.365*statistics.stdev(logs)/math.sqrt(8)
   result.append(dict(threads=t,arm=a,corpus=c,ratio=math.exp(mu),lo=math.exp(mu-h),hi=math.exp(mu+h),rss=statistics.median(r['rss_kb'] for r in cand)/statistics.median(r['rss_kb'] for r in ref)))
(root/'independent.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
