import json,math,statistics,pathlib
from scipy.stats import t
p=pathlib.Path(__file__).parent
rows=[json.loads(l) for l in (p/'data/run-20261117T102836Z/judged.jsonl').read_text().splitlines()]
idx={(r['corpus'],r['threads'],r['arm'],r['round']):r for r in rows}
assert len(rows)==len(idx)==176
out={}
for n,arms in [(8,['6-24','5-20']),(6,['6-24','5-20','4-16']),(4,['4-16','3-12','2-8'])]:
 for a in arms:
  for c in ['wikidata-x4','text']:
   ar=[idx[c,n,'mibz2-occ-'+a,i] for i in range(8)];br=[idx[c,n,'mibz2',i] for i in range(8)]
   logs=[math.log(x['wall']/y['wall']) for x,y in zip(ar,br)]
   mu=statistics.mean(logs);se=statistics.stdev(logs)/math.sqrt(8);h=t.ppf(.975,7)*se
   out[f'{n}:{a}:{c}']=dict(gm=math.exp(mu),lo=math.exp(mu-h),hi=math.exp(mu+h),p=float(t.cdf(mu/se,7)),rss=statistics.median(x['rss_kb'] for x in ar)/statistics.median(x['rss_kb'] for x in br))
ps=sorted((v['p'],k) for k,v in out.items() if k.endswith('wikidata-x4'));reject=[]
for i,(pv,k) in enumerate(ps):
 if pv>.05/(8-i):break
 reject.append(k)
assert reject==['8:6-24:wikidata-x4','6:5-20:wikidata-x4'],reject
result={'cells':out,'holm_reject':reject}
(p/'independent.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
