"""Audit stored joins and report absolute residual acquisition budget on matched ten-block laps."""
import csv
import json
import math
from pathlib import Path
import statistics
here=Path(__file__).resolve().parent
summary=json.loads((here/'summary.json').read_text())
labels=summary['labels'];raw=here/'raw'
rows=list(csv.DictReader((raw/'times.csv').open()));assert len(rows)==1440
blocks=sorted({int(q['block'])for q in rows})
seen=set();totals={v:[0]*24 for v in labels.values()}
for q in rows:
    b,r,arm=int(q['block']),int(q['round']),labels[q['arm']]
    key=(r,b,arm);assert key not in seen;seen.add(key)
    totals[arm][r-1]+=int(q['thread_ns'])
for a,t in totals.items():assert statistics.median(t)/1e6==summary['median_ms'][a]
timings={}
for a in ('prepared_bank','build_bank','jump_rules'):
    delta=[(x-y)/1e6 for x,y in zip(totals['scalar'],totals[a],strict=True)]
    g=[statistics.mean(delta[i:i+6])for i in range(0,24,6)]
    m=statistics.mean(g);h=3.182446305284263*statistics.stdev(g)/2
    timings[a]=dict(paired_budget_ms=m,lo=m-h,hi=m+h,median_paired_ms=statistics.median(delta),groups=g)
for n,arm in enumerate(list(labels.values())[:3]):
    phases=list(csv.DictReader((raw/f'phases-{n}.csv').open()));assert len(phases)==250
    ordered=[q for q in rows if labels[q['arm']]==arm]
    for p,q in zip(phases[10:],ordered,strict=True):
        assert p['block']==q['block'] and int(p['setup_ns'])==0
        assert sum(int(p[k+'_ns'])for k in ('scatter','order','prepare','emit'))<=int(q['thread_ns'])
    timings[arm].update(max_worker_scratch=max(int(z['scratch_bytes'])for z in phases),
                        max_input_bytes=max(int(z['side_bytes'])for z in phases))
rss=list(csv.DictReader((raw/'rss.csv').open()));assert len({(v['round'],v['block'],v['arm'])for v in rss})==180
for arm,v in summary['memory'].items():
    assert max(int(x['after_kib'])for x in rss if x['arm']==arm)==v['max_hwm_kib']
print(json.dumps(timings,indent=2))
