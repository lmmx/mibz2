import copy,json,math,sys,tempfile,subprocess
from pathlib import Path
from scipy.stats import binom,t
P=Path(__file__).resolve().parent
sys.path.insert(0,str(P/'bench'))
import service_tg as tg
import test_service_tg as fx
import walk_stats_check as wc
out={}
f=fx.Replicated();f.setUp()
try:
 ds=f.programme()
 assert tg.replicated(ds,fx.HEAD,rounds=fx.R)['valid']
 root,man=f.dispatch(0)
 shard_jobs=[j for j in man['jobs'] if j['name'].startswith('shard (')]
 shard_jobs[0]['completed_at']='2026-11-10T02:00:00Z'
 known=tg.ni_judge(root,man,fx.HEAD,reg=fx.REG,check_git=False,rounds=fx.R)
 ds[0]=known
 known_program=tg.replicated(ds,fx.HEAD,rounds=fx.R)
 assert not known_program['valid']
 del shard_jobs[0]['completed_at']
 missing=tg.ni_judge(root,man,fx.HEAD,reg=fx.REG,check_git=False,rounds=fx.R)
 ds[0]=missing
 missing_program=tg.replicated(ds,fx.HEAD,rounds=fx.R)
 assert missing['valid'] and missing_program['valid']
 out['missing_one_shard_completion']={'with_timestamp_valid':known_program['valid'],'known_problems':known_program['problems'],'after_deletion_dispatch_valid':missing['valid'],'after_deletion_program_valid':missing_program['valid'],'after_deletion_completed':missing['completed']}
 root,lastman=f.dispatch(11)
 for j in lastman['jobs']:j.pop('completed_at',None)
 ds[-1]=tg.ni_judge(root,lastman,fx.HEAD,reg=fx.REG,check_git=False,rounds=fx.R)
 v=tg.replicated(ds,fx.HEAD,rounds=fx.R)
 assert v['valid'] and ds[-1]['completed'] is None
 out['last_completion_missing']={'dispatch_valid':ds[-1]['valid'],'completed':None,'programme_valid':v['valid']}
 v=tg.confirm({'valid':True,'head':fx.HEAD,'runs':[],'flags':['runs/4']},{'valid':True,'head':fx.HEAD,'runs':[],'flags':['runs/4']})
 assert v['valid']
 out['empty_confirmation_runs']=v
finally:f.tearDown()
out['two_point_mean_boundary']={}
for k in [12,24,56,64,128,256]:
 # X=B-a+19*a*Bernoulli(1/19); offset a cancels from the studentized bound.
 accepted=[]
 for m in range(k+1):
  delta=-1+19*m/k
  se=19*math.sqrt(m*(k-m)/(k*k*(k-1)))
  if delta+t.ppf(.95,k-1)*se<=0:accepted.append(m)
 out['two_point_mean_boundary'][k]={'accepted_tail_counts':accepted,'size':sum(binom.pmf(m,k,1/19) for m in accepted),'no_tail':(18/19)**k}
out['sign_test_alternative']={}
for k in [12,24]:
 threshold=next(s for s in range(k+1) if binom.sf(s-1,k,.5)<=.05)
 out['sign_test_alternative'][k]={'success_threshold':threshold,'boundary_size':binom.sf(threshold-1,k,.5),'power_at_p90':binom.sf(threshold-1,k,.9),'42_cell_union_power_lower_at_p90':max(0,1-42*binom.cdf(threshold-1,k,.9))}

def ledger(hb):
 rows=[]
 for tid,b in [(0,1),(1,hb[0]),(2,hb[1])]:
  rows.append(dict(kind='call',team=0,block=0,thread=tid,role='leader' if tid==0 else 'helper',walker='traverse',takes=int(b>0),exhausts=1,bytes=b,tail=0,fragments=0,copied=0))
 for tid in (1,2):rows.append(dict(kind='share',team=0,block=0,thread=tid))
 rows.append(dict(kind='summary',team=0,block=0,rows=1+sum(hb),starts=1+sum(b>0 for b in hb),seg_bytes=1+sum(hb),entered=7,joined=True,built=6))
 rows.append(dict(kind='end',records=len(rows)))
 return rows
out['combined_checker']={}
with tempfile.TemporaryDirectory() as td:
 q=Path(td);(q/'classes').write_text('T\n')
 for name,hb in [('balanced',[1,1]),('unbalanced',[1,99]),('zero_work',[0,0])]:
  lines=[json.dumps(x) for x in ledger(hb)]
  by,bad=wc.validate(lines,{0:'T'},2);pred=wc.predicates(by,'traverse',2)
  assert not bad and all(pred[k][0] for k in 'acd')
  (q/'stats').write_text('\n'.join(lines)+'\n')
  r=subprocess.run([sys.executable,str(P/'bench/walk_stats_check.py'),str(q/'stats'),'--walker','traverse','--helpers','2','--classes',str(q/'classes')],capture_output=True,text=True)
  assert r.returncode==(0 if name=='balanced' else 1) and not r.stderr
  out['combined_checker'][name]={'exit':r.returncode,'stdout':r.stdout,'predicates':pred}
print(json.dumps(out,indent=2))
