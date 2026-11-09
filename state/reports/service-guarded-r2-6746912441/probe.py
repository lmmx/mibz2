import json,sys
sys.path.insert(0,'r45/repo/bench')
import test_service_guarded as t
results=[]
for status in ['0\n','17\n','124\n','137\n','','zero\n']:
    v=t.judge(t.files(**{'preflight.exit':status}))
    results.append({'kind':'preflight','value':repr(status),'valid':v['valid'],'failure':v['failure']})
    assert v['valid'] == (status=='0\n')
for status in ['success','exit 0','exit 17','signal 9','signal 15','unknown',None,True]:
    rows=t.rows()
    if status is None: rows[0].pop('termination')
    else: rows[0]['termination']=status
    v=t.judge(r=rows)
    results.append({'kind':'termination','value':status,'valid':v['valid'],'failure':v['failure']})
    assert v['valid'] == (status=='success')
f=t.files();f['diagnostics/exits.txt']='env.json: exit 137\nsample-after.jsonl: exit 17\n'
v=t.judge(f);assert v['valid'];results.append({'kind':'optional diagnostic failures','valid':v['valid']})
print(json.dumps(results,indent=2))
