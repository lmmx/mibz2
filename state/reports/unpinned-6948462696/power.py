import json, math, statistics, sys
from pathlib import Path
from scipy.stats import t, nct
from scipy.optimize import brentq
root=Path(__file__).parent
source=Path(sys.argv[1]) if len(sys.argv)>1 else root.parent/'eager-review/data/run-20261117T114640Z/judged.jsonl'
rs=[json.loads(s) for s in source.read_text().splitlines()]
out=[]
for threads,arm in [(10,'mibz2-occ-7-28'),(6,'mibz2-oce-5-20'),(4,'mibz2-oce-3-12'),(6,'mibz2-occ-5-20'),(8,'mibz2-occ-6-24')]:
 if threads==8:
  # Only rounded published interval available for this cell; not a raw replay.
  sigma=math.log(.933/.918)/2*math.sqrt(8)/2.365
 else:
  by={(r['arm'],r['round']):r for r in rs if r['threads']==threads and r['corpus']=='wikidata-x4'}
  sigma=statistics.stdev(math.log(by[arm,i]['wall']/by['mibz2',i]['wall']) for i in range(8))
 se=sigma/math.sqrt(8)
 def power(gain,alpha): return nct.cdf(t.ppf(alpha,7),7,math.log(1-gain)/se)
 out.append(dict(threads=threads,arm=arm,sigma=sigma,halfwidth95_log=t.ppf(.975,7)*se,observed_gain_to_cross_001=1-math.exp(t.ppf(.01,7)*se),power_at_2pct_alpha001=power(.02,.01),power_at_2pct_alpha005=power(.02,.05),gain_for_80pct_alpha001=brentq(lambda g:power(g,.01)-.8,.00001,.3)))
(root/'power.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
