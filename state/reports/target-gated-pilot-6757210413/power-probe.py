import json
import math
from pathlib import Path
import service_tg_power as p
runs, sds = p.history(Path(__file__).with_name('links.jsonl'))
out = {'historical_runs': len(runs), 'cells': len(sds), 'scenarios': []}
for R, ratio in [(600,1.0),(600,1.005),(960,1.005)]:
    values = [p.power(R,20,p.quantile(sds[k],0.9),ratio) for k in sorted(sds)]
    out['scenarios'].append({'R': R, 'B':20, 'scenario':'p90', 'true_ratio':ratio,
                             'all_cell_lower':max(0,1-sum(1-x for x in values)),
                             'independent_product':math.prod(values)})
try:
    p.power(900,20,0.05,1.005)
except ValueError:
    out['odd_batches_rejected'] = True
print(json.dumps(out,indent=2))
