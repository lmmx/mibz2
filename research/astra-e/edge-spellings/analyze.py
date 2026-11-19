"""Replay capacities, exact savings and per-lap native CPU intervals."""
import csv
import json
import math
import statistics
from collections import Counter,defaultdict
from pathlib import Path

here=Path(__file__).resolve().parent
def rows(name):
    with (here/name).open() as f:
        for r in csv.DictReader(f):
            yield {k:int(v) for k,v in r.items()}

known={r['block']:r for r in rows('handles.csv')}
shape=defaultdict(dict)
for r in rows('checked.csv'):shape[r['arm']][r['block']]=r
slots={r['block']:r for r in rows('slots.csv') if r['slots']==64}
assert len(known)==len(slots)==233
for b,d in known.items():
    a,c,f=(shape[i][b] for i in [0,2,3])
    assert a['edge']==c['edge']==f['edge']==d['edge']
    assert a['spelling']==d['spelling'] and a['offset']==d['offsets']
    assert c['spelling']==a['spelling']-slots[b]['reuse_bytes'] and c['offset']==a['offset']-4*slots[b]['reuse_words']
    assert f['spelling']==a['spelling']-d['bytes_spelling'] and f['offset']==a['offset']-4*d['bytes_bands']
def total(rows):
    return sum(r['spelling']+r['offset']+r['edge'] for r in rows)
result={'all_actual_geometry':{}}
for a,vals in shape.items():
    r=list(vals.values())
    result['all_actual_geometry'][a]={'total_walk_bytes':total(r),
                                      'sum_spelling_bytes':sum(x['spelling'] for x in r),
                                      'sum_offsets':sum(x['offset'] for x in r),
                                      'max_walk_vector_capacities_one_reused_worker':max(x['walk_vec_capacity'] for x in r),
                                      'min_walk_bytes':min(total([q]) for q in r)}
totals=defaultdict(int)
order=Counter()
dense=set();ordinary=set()
for r in rows('paid.csv'):
    group='dense' if r['dense'] else 'ordinary'
    (dense if r['dense'] else ordinary).add(r['block'])
    for key in ['all',group]:
        totals[key,r['round'],r['arm']]+=r['cpu_ns']
    order[r['block'],r['position'],r['arm']]+=1
assert len(dense)==5 and set(order.values())=={6} and len(order)==16*(len(dense)+len(ordinary)) and len(totals)==3*4*24
result['dense_blocks']=sorted(dense);result['ordinary_blocks']=sorted(ordinary)
result['paid_cpu_over_edge']={}
for group in ['all','dense','ordinary']:
    result['paid_cpu_over_edge'][group]={}
    for a in [1,2,3]:
        logs=[math.log(totals[group,i,a]/totals[group,i,0]) for i in range(1,25)]
        mid=statistics.mean(logs);half=2.06865761*statistics.stdev(logs)/math.sqrt(24)
        result['paid_cpu_over_edge'][group][a]=[math.exp(z) for z in [mid,mid-half,mid+half]]
result['paid_cpu_over_same_code_twin']={}
for group in ['all','dense','ordinary']:
    result['paid_cpu_over_same_code_twin'][group]={}
    for a in [2,3]:
        logs=[math.log(totals[group,i,a]/totals[group,i,1]) for i in range(1,25)]
        mid=statistics.mean(logs);half=2.06865761*statistics.stdev(logs)/math.sqrt(24)
        result['paid_cpu_over_same_code_twin'][group][a]=[math.exp(z) for z in [mid,mid-half,mid+half]]
print(json.dumps(result,indent=2))
