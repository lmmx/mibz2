"""Untimed count-coupling check; place at exact reviewed checkout root."""
import json, sys
from pathlib import Path
root=Path(__file__).resolve().parent
sys.path.insert(0,str(root/'bench'))
import test_run_fed_stage as t
r=t.base(); results=[]
for case in json.loads((root/'bench/fixtures/run_fed_stage_counterexamples.json').read_text())['cases']:
    p=t.problems(t.apply(t.base(),case)); assert p
    results.append({'case':case['name'],'problems':p})
viol=[]
for f in r['files']:
    tc=f['slots']//8-f['d_vector']; s=f['d_scalar']
    lo=(s+6)//7; hi=min(s,f['cells'])
    if not lo<=tc<=hi:
        viol.append({'name':f['name'],'B_slots':f['slots'],'D_vector':f['d_vector'],
            'D_scalar':s,'implied_tail_cells':tc,'permitted':[lo,hi]})
out={'source':'370a868f46e4a7326557859043e5e44960e7b592',
     'fixture_problems':t.problems(r),'fixture_violations':viol,'counterexamples':results,
     'synthetic_text':t.v.synthetic_expected(str(root))[0]}
(root/'recheck.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(viol,indent=2))
