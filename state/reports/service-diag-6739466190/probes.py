"""Untimed adversarial probes of exact #293 Python analyser, no decoder execution."""
import copy,json,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent/'bench'))
import svc_diag as d
from test_svc_diag import Sim,history,NineUnits,wait_of

out={}
# Independent lock's event takes sequence before hub Prepared, timestamp afterwards.
s=NineUnits().sim()
idx=next(i for i,e in enumerate(s.events) if e['kind']=='TeamWait')
e=copy.deepcopy(s.events[0]);e.update(kind='ReadWait',thread='reader',place=s.w['taken'],ns=8,hub=None)
s.events.insert(idx+1,e)
for i,e in enumerate(s.events):e['seq']=i
r=s.analyse();w=wait_of(r,0)
out['timestamp_inversion']={'wait_endpoint_length':w['end']-w['start'],'attributed_length':sum(w['classes'].values()),'defects':r['defects']}
assert out['timestamp_inversion']['wait_endpoint_length']==9
assert out['timestamp_inversion']['attributed_length']==16

# Three in flight; ready2/4, 3 still finishing. Taking2 frees credit, scanner not yet resumed.
s=Sim(T=2,K=2,L=2,W=3)
history(s,0,0);history(s,1,1)
for p in (2,3,4):s.issue(p)
assert s.decide(0)==('prepare',2);s.prepared_(0,2);s.team_take(0);s.team_walked(0,2)
assert s.decide(0)==('prepare',3);s.prepared_(0,3)
assert s.decide(0)==('finish',2)
s.team_take(1);s.team_walked(1,3)
assert s.decide(1)==('prepare',4);s.prepared_(1,4)
assert s.decide(1)==('finish',3)
s.team_take(0);s.team_walked(0,4)
s.finished(0,2);assert s.decide(0)==('finish',4);s.finished(0,4)
s.slot_wait();s.team_wait(0);s.take()
assert s.w['issued']-s.w['taken']==2<s.W
s.tick(5);s.emit('mibz2-scan','SlotGot',None,1,win=True);s.end()
r=s.analyse();w=wait_of(r,0)
out['released_credit']={'outstanding':s.w['issued']-s.w['taken'],'slots':s.W,'wrong_window_credit_ns':w['classes']['window_credit'],'defects':r['defects']}
assert w['classes']['window_credit']==5

# Finish occupancy does not establish a finish-over-eligible-prep decision.
s=Sim(T=2,K=2,L=2,W=8);history(s,0,0);history(s,1,1)
for p in (2,3):
 s.issue(p);assert s.decide(0)==('prepare',p);s.prepared_(0,p);s.team_take(p-2);s.team_walked(p-2,p)
assert s.decide(0)==('finish',2);assert s.decide(1)==('finish',3)
s.issue(4);s.team_wait(0);s.tick(5);s.end()
r=s.analyse();w=wait_of(r,0)
out['no_finish_priority_choice']={'finish_over_prep':r['finish_over_prep'],'finish_first_ns':w['classes']['finish_first'],'defects':r['defects']}
assert r['finish_over_prep']==0 and w['classes']['finish_first']==5

# Reader admission: incomplete logs and mixed configurations accepted with no defects.
s=Sim();s.tick(1);s.end();r=s.analyse()
out['empty_incomplete']={'blocks':r['blocks'],'defects':r['defects']}
r2=copy.deepcopy(r);r2['plan']['window']=12
out['mixed_plan']={'plans':[r['plan'],r2['plan']],'reported_plan':d.summarise([r,r2])['plan'],'defects':d.summarise([r,r2])['defects']}
print(json.dumps(out,indent=2))
