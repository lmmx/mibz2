import sys,json,copy
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent/'bench'))
import test_team_screen as t
import team_screen as ts
import walk_stats_check as ws
out={}
for case in ['baseline','artifact_wrong_head','rejection_null','rejection_negative_signal_exit','rejection_timeout_exit']:
 m,d,a=t.manifest(),t.delta(),t.artifacts();r=t.good_records(m,d,a)
 if case=='artifact_wrong_head':a['head']['sha']='9'*40
 if case.startswith('rejection_'):
  for w in r:
   if w['kind']=='reject':w['exit_status']={'rejection_null':None,'rejection_negative_signal_exit':-9,'rejection_timeout_exit':124}[case]
 out[case]=ts.outputs_check(m,d,a,r)
 assert out[case]==[],(case,out[case])
rows=[]
def call(b,h):
 return dict(kind='call',team=0,block=b,thread=h,role='leader' if h==0 else 'helper',walker='traverse',takes=1,exhausts=1,bytes=1,tail=0,fragments=0,copied=0)
rows.append(call(0,0));rows.append(dict(kind='summary',team=0,block=0,rows=1,starts=1,seg_bytes=1,entered=1,joined=True,built=0))
for h in [1,2]:rows.extend([call(1,h),dict(kind='share',team=0,block=1,thread=h)])
rows.append(dict(kind='summary',team=0,block=1,rows=2,starts=2,seg_bytes=2,entered=6,joined=False,built=6))
rows.append(dict(kind='end',records=len(rows)))
by,bad=ws.validate([json.dumps(x) for x in rows],{0:'T',1:'T'},2)
pred=ws.predicates(by,'traverse',2)
assert not bad and all(v[0] for v in pred.values())
out['missing_both_helpers_in_one_T']={'invalid_reasons':bad,'predicates':pred}
print(json.dumps(out,indent=2))
