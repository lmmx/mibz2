"""Check all measured support counts, including rejection depth and forest fragmentation."""
from pathlib import Path
import csv,json
p=Path(__file__).resolve().parent
seed=list(csv.DictReader((p/'seed-blocks.csv').open()))
forest=list(csv.DictReader((p/'forest-blocks.csv').open()))
baseline={(v['block'],v['k']):v for v in csv.DictReader((p.parent/'blocks.csv').open())}
out={}
for a,b in zip(seed,forest,strict=True):
 assert a['block']==b['block'] and a['k']==b['k']
 s={k:int(v)for k,v in a.items()};f={k:int(v)for k,v in b.items()}
 o=baseline[a['block'],a['k']]
 assert f['events']==s['exception_rows']==int(o['rows'])
 assert f['classes']==int(o['alphabet']) and f['output_runs']==int(o['runs']) and f['literal_bytes']==int(o['distinct_return_spelling_bytes'])
 assert f['intersections']<=f['internal_nodes']+s['original_runs']-1 and f['covered_rows']==s['n']
 assert f['unary']+f['branch']==f['internal_nodes'] and f['internal_nodes']-f['seeds']+f['classes']==f['seed_queries']
 assert s['tested']+s['skipped']==s['seams'] and s['psi_calls']==2*s['compared_pairs']
 rejected=[s['reject_at_'+str(i)]for i in range(1,17)]
 assert sum(rejected)+s['mixed_contexts']==s['tested']
 assert sum((j+1)*x for j,x in enumerate(rejected))+s['k']*s['mixed_contexts']==s['compared_pairs']
 t=out.setdefault(a['k'],dict(seed={},forest={}))
 for data,part in [(s,'seed'),(f,'forest')]:
  for key,value in data.items():
   if key not in ('n','block','k','peak_live_merge_records'):
    t[part][key]=t[part].get(key,0)+value
 t['forest']['maximum_block_live_merge_records']=max(t['forest'].get('maximum_block_live_merge_records',0),f['peak_live_merge_records'])
print(json.dumps(out,indent=2))
