import json,sys,pathlib
P=pathlib.Path(__file__).resolve().parent
sys.path.insert(0,str(P/'bench'))
import test_service_lookahead as t
def short(rows,traces,files):
 for tr in traces.values():
  tr['settings']['rounds']=1;tr['children_expected']=3;tr['runs']=[r for r in tr['runs'] if r['round']==0];tr['results'][0]['timeline']['complete_rounds']=1
def corrupt(rows,traces,files):
 tr=traces[t.A3W8];tr['runs'][0]['line']=tr['runs'][0]['line'].replace('hash=0123456789abcdef','hash=BAD').replace('err=-','err=failed')
def impossible_share(rows,traces,files):
 x=next(r for r in traces[t.A3W8]['runs'] if r['mode']=='timeline');x['line']=x['line'].replace('t.svc.team.idle_prep_share=0.15','t.svc.team.idle_prep_share=99')
def missing_share(rows,traces,files):
 x=next(r for r in traces[t.A3W8]['runs'] if r['mode']=='timeline');x['line']=x['line'].replace('t.svc.team.idle_prep_share=0.15','t.svc.team.idle_prep_share=NaN')
out={}
for n,f in [('control',None),('three_children_instead_of_eighteen',short),('raw_hash_and_error_disagree_with_parity',corrupt),('one_complete_round_share_99',impossible_share),('one_complete_round_share_NaN',missing_share),('one_complete_round_six_requested',t.with_timeline(t.A3W8,complete_rounds=1))]:
 d=t.run(f);out[n]={k:d[k] for k in ['verdict','exit','mechanism']};out[n]['failed_checks']=t.failed(d)
(P/'probe.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
