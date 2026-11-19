import csv,json
from pathlib import Path
p=Path(__file__).resolve().parent
r=list(csv.DictReader((p/'levels.csv').open()));assert len(r)==418
result={}
for q in (2,4,8,16):
 sel=[{k:int(v)for k,v in a.items()}for a in r if int(a['q'])==q]
 for a in sel:
  assert a['rauzy_residue_pieces']-min(a['old_rows'],q)==a['path_sinks']
  assert a['rauzy_equal']+a['rauzy_self']==min(a['old_rows'],q)-1+a['empty_paths']
  assert a['canonical_bands']==1+a['finite_paths_with_residue'] and a['reverse_visited']<=max(a['old_rows']-q,0)
 t={}
 for label,which in [('first',[a for a in sel if a['level']==0]),('tower',sel)]:
  t[label]={k:sum(a[k]for a in which)for k in ('old_rows','old_bands','retained','path_sinks','canonical_bands','reverse_visited','reverse_lookups',
            'packet_splits','representative_steps','new_joins','rauzy_strict','rauzy_equal','rauzy_self','sort_image_comparisons','sort_cut_comparisons')}
  t[label]['rauzy_joins']=t[label]['rauzy_strict']+t[label]['rauzy_equal']
 result[q]=t
print(json.dumps(result,indent=2))
