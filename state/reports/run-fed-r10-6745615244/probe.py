import copy, json
import run_fed_leader_ws as w
from test_run_fed_leader_ws import host_env, snapshot, LSCPU_I9
base=(host_env(), LSCPU_I9, [{'state':snapshot(4900,t=i)} for i in range(3)])
out={}
for name in ['positive','missing_quota','float_cpu_counts','integer_turbo','invented_cpu_ids']:
    e,l,s=copy.deepcopy(base)
    if name=='missing_quota': e.pop('cpu_quota')
    if name=='float_cpu_counts': e.update(cpus=20.0,cores=10.0)
    if name=='integer_turbo':
        e['machine']['turbo']=1
        for x in s:x['state']['turbo']=1
    if name=='invented_cpu_ids':
        for x in s:x['state']['cpu_khz']={f'not-a-cpu-{i}':4900000 for i in range(20)}
    p=w.preflight_problems(e,l,s)
    out[name]={'verdict':'REFUSED' if p else 'PASS','problems':p}
print(json.dumps(out,indent=2))
