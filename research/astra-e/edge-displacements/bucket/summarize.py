"""Join the exact queue accounting with unmodified width-16 dictionary sizes."""
from collections import Counter, defaultdict
import csv
from pathlib import Path
import statistics

here=Path(__file__).resolve().parent
research=here.parents[1]

def records(path):
    return list(csv.DictReader(path.open()))

base={int(r['block']):r for r in records(research/'private-cache/census.csv') if r['limit']=='16'}
routes={int(r['ordinal']):int(r['output']) for r in records(research.parents[1]/'docs/journal/2026-11-18-astra-d-memory/evidence/corpus-routes.csv')}
phases=defaultdict(list)
for r in records(research/'front-fusion/cap-batch/phase.csv'):
    if r['arm']=='0' and (r['round']!='0' or int(r['stream']) not in base):
        phases[int(r['stream'])].append(int(r['cpu_ns']))
cost={i:statistics.median(v) for i,v in phases.items()}
assert len(cost)==431
tables=defaultdict(Counter)
for r in records(here/'bucket.csv'):
    b=int(r['block']);ref=base[b]
    group=tables[r['mode'],int(r['group'])]
    for k in ('m','band_masks','band_prefix','band_descriptors','group_heads','group_unary','delta_bytes','boundary_groups','boundary_entries'):
        group[k]+=int(r[k])
    payload=sum(int(r[k]) for k in ('band_masks','band_prefix','band_descriptors','group_heads','group_unary','delta_bytes'))
    dic=int(ref['spell'])+int(ref['offset'])
    group.update(blocks=1,edges=payload,full=payload+dic,dictionary=dic,old=int(ref['edge'])+dic)
    for cap in (192,256):
        if payload<=1024*cap: group[f'edgefit_{cap}']+=1
        if payload+dic<=1024*cap:
            group[f'fullfit_{cap}']+=1
            group[f'outputfit_{cap}']+=routes[b]
            group[f'cpufit_{cap}']+=cost[b]
print("layout,g,edges,full,band_masks,band_prefix,band_descriptors,group_heads,group_unary,delta_bytes,boundary_groups,boundary_entries,full_ratio,edge_bytes_per_row,edgefit_192,edgefit_256,fullfit_192,fullfit_256,outputfit_192,outputfit_256,cpu_fraction_192,cpu_fraction_256")
for (kind,g),d in sorted(tables.items()):
    assert d['blocks']==len(base)==233
    print(kind,g,*(d[k] for k in ['edges','full','band_masks','band_prefix','band_descriptors','group_heads','group_unary','delta_bytes','boundary_groups','boundary_entries']),*(round(d['full']/d['old'],5),round(d['edges']/d['m'],4)),
          *(d[f'edgefit_{c}'] for c in (192,256)),*(d[f'fullfit_{c}'] for c in (192,256)),*(d[f'outputfit_{c}'] for c in (192,256)),*(round(d[f'cpufit_{c}']/sum(cost.values()),5) for c in (192,256)),sep=',')
