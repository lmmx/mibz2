import json,math,statistics
from pathlib import Path
d=Path(__file__).resolve().parent
rows=[json.loads(s) for s in (d/'charge.jsonl').read_text().splitlines()]
assert len(rows)==20 and len({(r['label'],r['round'])for r in rows})==20
assert [(r['round'],r['label']) for r in rows]==[(k,a)for k in range(10)for a in (['base','audit'] if k%2==0 else ['audit','base'])]
assert all(r['count']==(5 if r['label']=='audit'else 0) and r['bytes']==209700196 for r in rows)
z=[r for r in rows if r['label']=='audit'];base=[r for r in rows if r['label']=='base']
assert {(r['coded'],r['rows'])for r in z}=={(4499868,1083380)}
def geom(v):
 x=[math.log(k)for k in v];m=statistics.mean(x);h=2.26215716*statistics.stdev(x)/math.sqrt(len(x));return [math.exp(x) for x in [m,m-h,m+h]]
emit=geom([r['emit_ns']/r['process_ns'] for r in z])
out={'full_worker_cpu_overhead_ratio':geom([r['process_ns']/s['process_ns']for r,s in zip(z,base)]),
     'dense_emit_full_process_fraction':emit,
     'dense_complete_inverse_full_process_fraction':geom([(r['construct_ns']+r['emit_ns'])/r['process_ns']for r in z]),
     'conditional_previous_17pct_emitter_saving':[v*(1-.822339) for v in emit],
     'mean_ms':{k:statistics.mean(r[k] for r in z)/1e6 for k in ['construct_ns','emit_ns','process_ns']},
     'coded_fraction':4499868/209697662,'retained_rank_fraction':1083380/26567581}
print(json.dumps(out,indent=2))
