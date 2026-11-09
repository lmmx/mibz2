"""Independent untimed collector/identity/reporter boundary matrix; source 28af8f7."""
import json, math, sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent/'bench'))
import test_c29_checkpoints as t
out=[]
def check(name, ident, expected, recs=None):
    code,text=t.report(t.runs(1.0) if recs is None else recs,ident)
    assert code==expected,(name,code,text)
    out.append(dict(case=name,exit=code,quota=ident.get('cpu_quota'),output=text))
for state in ('unlimited','finite','unknown',None,'bad'):
    for value in (None,False,0,2,4,8,'4',float('nan'),float('inf')):
        env=dict(t.ENV,cpu_quota_state=state,cpu_quota=value)
        expected=0 if state=='unlimited' and value is None or state=='finite' and type(value) in (int,float) and math.isfinite(value) and value>=4 else 3
        check(f'{state}/{value}',t.identity(json.dumps(env)),expected)
for field in ('cpu_quota_state','both'):
    env=dict(t.ENV)
    del env['cpu_quota_state']
    if field=='both':del env['cpu_quota']
    check('absent_'+field,t.identity(json.dumps(env)),3)
for status in (101,-1,False,None,'0'):
    check('status_'+repr(status),t.identity(run_status=status),3)
for text in ('','{','null','[]','4'):
    check('broken_env_'+text,t.identity(text),3)
r=t.runs();r[0]['exit']=1
check('invalid_precedence',t.identity(run_status=101),2,r)
# Explicit positive state without legacy redundant value is accepted; not unknown detection.
env=dict(t.ENV);del env['cpu_quota']
check('explicit_unlimited_missing_legacy_number',t.identity(json.dumps(env)),0)
Path(__file__).with_name('results.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'checks':len(out),'exit_counts':{str(c):sum(x['exit']==c for x in out) for c in (0,2,3)}}))
