import json
from svc_target_fixtures import record, edit
from svc_target_check import check
r,x=record(True)
cases={
 'positive':r,
 'infinite_quota':edit(r,lambda r:r['machine'].update(quota={'kind':'cpus','cpus':float('inf')})),
 'missing_team_shape':edit(r,lambda r:r['at'][0].update(plan={'kind':'teams'})),
 'float_budget':edit(r,lambda r:r['at'][0].update(threads=9.0)),
 'float_service_plan':edit(r,lambda r:r['service_plan'].update(teams=2.0)),
 'negative_pooled':edit(r,lambda r:r['at'][1]['built']['counts'].update(pooled=-1)),
 'missing_plan_negative_control':edit(r,lambda r:r['at'][0].pop('plan')),
 'unknown_quota_negative_control':edit(r,lambda r:r['machine'].update(quota={'kind':'unknown'})),
}
out={k:check(v,x) for k,v in cases.items()}
# Valid JSON may overflow Python's float during parsing too; no Infinity extension needed.
text=json.dumps(cases['infinite_quota']).replace('Infinity','1e309')
out['json_1e309_quota']=check(json.loads(text),x)
n,nx=record(False)
n['at'][0]['decoded']=False
n['why'].append('decode mismatch')
out['non_target_decode_failure']=check(n,nx)
print(json.dumps(out,indent=2))
