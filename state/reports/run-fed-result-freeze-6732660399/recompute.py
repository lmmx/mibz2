"""Recompute descriptive paired contrasts; no decoding or timing. Run in this directory."""
import json, math, statistics, pathlib
p = pathlib.Path(__file__).parent
rs = [json.loads(x) for x in (p/'rfl-runs.jsonl').read_text().splitlines()]
assert len(rs) == 210
idx = {(r['threads'],r['arm'],r['round']):r for r in rs}
assert len(idx) == 210
assert all(r['exit']==0 and r['err']=='' for r in rs)
assert len({(r['hash'],r['out']) for r in rs}) == 1
def arm(s):
 return 'mibz2-default33' if s=='ref' else 'mibz2-pipe3-runfed-'+s
results = {}
for t in (9,3):
 for a,b in [('dir','ref'),('ctl','ref'),('reg','ctl'),('dir','ctl'),('dir','reg'),('ls','reg'),('ls','dir')]:
  if t==3 and b=='ref': continue  # forced three-team default is not a fair one-team control
  cell = {}
  for metric in ('wall','cpu'):
   logs = [math.log(idx[t,arm(a),i][metric]/idx[t,arm(b),i][metric]) for i in range(15)]
   mean = statistics.mean(logs)
   half = 2.1447866879169273 * statistics.stdev(logs) / math.sqrt(15)
   cell[metric] = [math.exp(mean),math.exp(mean-half),math.exp(mean+half)]
  results[f'{a}/{b}@{t}'] = cell
print(json.dumps(results,indent=2))
