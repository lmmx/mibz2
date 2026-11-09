import sys,json,copy,re,ast,hashlib
from pathlib import Path
root=Path(__file__).resolve().parent
sys.path.insert(0,str(root/'bench'))
import run_fed_stage as v
import test_run_fed_stage as tests
base=tests.base()
out={'positive_problems':tests.problems(base),'negative_cases':[],'census':[]}
cases=v.load(str(root/'bench/fixtures/run_fed_stage_counterexamples.json'))['cases']
for i,case in enumerate(cases,1):
 rec=copy.deepcopy(base);tests.apply(rec,case);p=tests.problems(rec)
 assert p and any(case['expect'] in x for x in p),(i,p)
 out['negative_cases'].append({'case':i,'name':case['name'],'problems':p})
for f in base['files']:
 t=f['slots']//8-f['d_vector'];s=f['d_scalar'];c=f['cells']
 out['census'].append({'name':f['name'],'tail_cells':t,'lower':(s+6)//7,'upper':min(s,c),'relations':v.count_relations(f)})
 assert (s+6)//7<=t<=min(s,c)
# Parse only the Rust literal census table, compare each number and digest to JSON.
rust=(root/'examples/run_fed_stage.rs').read_text()
block=rust.split('d_vector, d_scalar, d_streams, bytes, output, sha) = [',1)[1].split('][i];',1)[0]
table=ast.literal_eval('['+block+']')
keys=['blocks_runs','blocks_fallback','rows','rows_runs','records','cells','slots','d_vector','d_scalar','d_streams','bytes','output','sha256']
assert len(table)==7
for row,f in zip(table,base['files']): assert all(f[k]==x for k,x in zip(keys,row))
out['rust_literal_counts_match_json']=True
out['current_synthetic']=v.synthetic_expected(str(root))
import coord
channels=json.loads((root/'channels.json').read_text())
for cs in channels.values():
 for c in cs:c.setdefault('created_at',None)
coord.comments=lambda issue:channels[str(issue)]
out['routing']=coord.route('coordinator','REVIEW')
out['audit']=coord.audit(0)
out['audit_unresolved']=[x for x in out['audit'] if not x['resolved']]
assert not out['audit_unresolved']
(root/'probe.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({k:out[k] for k in ['positive_problems','rust_literal_counts_match_json','census','current_synthetic','audit_unresolved']},indent=2))
