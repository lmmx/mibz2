"""Replay structural eligibility, geometry, contextual decode costs, and the registered panel."""
import csv
import json
import math
import statistics
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]

def rows(path):
    with path.open() as f:
        yield from csv.DictReader(f)

blocks = defaultdict(dict)
for x in rows(HERE/'census.csv'):
    r = {k: int(v) for k, v in x.items()}
    blocks[r['block']][r['limit']] = r
assert all(set(by_limit) == set(range(17)) for by_limit in blocks.values())
routes = {int(r['ordinal']): r for r in rows(ROOT/'docs/journal/2026-11-18-astra-d-memory/evidence/corpus-routes.csv')}
samples = defaultdict(dict)
for row in rows(HERE.parent/'front-fusion/cap-batch/phase.csv'):
    if row['arm'] == '0':
        samples[int(row['stream'])][int(row['round'])] = int(row['cpu_ns'])
cost = {b: statistics.median([v for i,v in by_round.items() if i > 0] or [by_round[0]])
        for b,by_round in samples.items()}
assert set(routes) == set(cost) == set(range(431))

def total(ids):
    ids = list(ids)
    return {'blocks': len(ids), 'coded': sum(int(routes[b]['rows']) for b in ids),
            'output': sum(int(routes[b]['output']) for b in ids),
            'historical_decode_block_thread_ns': sum(cost[b] for b in ids),
            'fraction_of_historical_decode_block_cpu': sum(cost[b] for b in ids)/sum(cost.values())}

def live(r):
    assert r['marked_visits'] == r['m'] and r['marked_spell']==r['spell'] and r['marked_offset']==r['offset']
    return r['edge']+r['marked_spell']+r['marked_offset']

def recurring(r,shift):
    return ((r['edge']+63)//64)*64+r['recurrent_spell_'+shift]+r['recurrent_offset_'+shift]

result = {'whole':total(routes),'admitted':total(blocks),'footprint_minimum_bytes': {},
          'zero_width':{}, 'budget': {}}
measures = {'all':live,'recurrent_aligned':lambda r:recurring(r,'aligned'),
            'recurrent_shifted':lambda r:recurring(r,'shifted')}
for key,fn in measures.items():
    values = [(fn(r),b,t) for b,bs in blocks.items() for t,r in bs.items() if t]
    v,b,t=min(values)
    result['footprint_minimum_bytes'][key]={'block':b,'limit':t,'bytes':v}
for cap in (192*1024,256*1024):
    obs = {}
    for name,fn in {**measures, 'edge':lambda r:r['edge'], 'certified_edge':lambda r:r['edge_bound']}.items():
        crossing={b:max((t for t,r in bs.items() if t and fn(r)<=cap),default=0) for b,bs in blocks.items()}
        active={b:t for b,t in crossing.items() if t}
        changed={b:t for b,t in active.items() if t<16}
        record={'largest_positive_width':dict(sorted(Counter(active.values()).items())),
                'positive':total(active),'changed_positive':total(changed)}
        if name=='certified_edge' and changed:
            record['changed_geometry_relative_to_width16']={
                k: sum(blocks[b][t][k] for b,t in changed.items())/sum(blocks[b][16][k] for b in changed)
                for k in ['m','h','edge','spell','offset','initial_nodes','steps','rules','bands']}
            record['narrow_endpoint_guard_failures_at_selected_width']=sum(blocks[b][t]['h']>(65535-3)//3 for b,t in changed.items())
        obs[name]=record
    result['budget'][cap]=obs
for k in ['m','h','edge','spell','offset','initial_nodes','steps','rules','bands']:
    result['zero_width'][k]=sum(by_limit[0][k] for by_limit in blocks.values())
result['zero_width']['exact_single_word_spells_entire_block']=sum(bs[0]['words']==1 and bs[0]['spell']==bs[0]['n'] for bs in blocks.values())
result['nonzero_edge_certificate_overcharges']=sum(r['edge']<r['edge_bound'] for bs in blocks.values() for t,r in bs.items() if t)
print(json.dumps(result,indent=2))
