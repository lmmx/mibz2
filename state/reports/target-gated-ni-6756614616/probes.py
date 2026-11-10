"""Independent research probes; uses the pinned judge and synthetic repository fixtures."""
import json, math, pathlib, tempfile
import numpy as np
from scipy import stats
import service_tg as tg
import test_service_tg as fix

out={}
def judge(root,man):
    v=tg.ni_judge(root,man,fix.HEAD,reg=fix.REG,check_git=False,rounds=fix.R,batches=fix.B)
    return {k:v[k] for k in ('valid','passes','problems')}

with tempfile.TemporaryDirectory() as root:
    man=fix.build(root)
    man['jobs']=[{'name':'shard (runs)', 'run_attempt':1,'conclusion':'cancelled','status':'completed'}]
    out['cancelled_job_with_complete_artifact']=judge(root,man)
with tempfile.TemporaryDirectory() as root:
    man=fix.build(root,runs=[{'attempt':1,'drop':('measurement-started.txt',)}, {'attempt':2}])
    out['prior_rows_but_missing_start_marker']=judge(root,man)
with tempfile.TemporaryDirectory() as root:
    man=fix.build(root,runs=[{'attempt':2}])
    man['jobs']=[{'name':'shard (runs)','run_attempt':1,'conclusion':'cancelled','status':'completed',
                  'steps':[{'name':'Run (the measurement)','status':'completed','conclusion':'cancelled'}]}]
    out['prior_measured_job_without_artifact']=judge(root,man)
with tempfile.TemporaryDirectory() as root:
    man=fix.build(root,text=[{'over':{'workflow_sha':'0'*40,'run_attempt':99}}])
    out['wrong_workflow_sha_and_attempt']=judge(root,man)
out['same_dispatch_can_confirm_itself']=tg.confirm({'valid':True,'flags':['runs/2']},{'valid':True,'flags':['runs/2']})

# Stationary Gaussian AR(1) at the null boundary. This is simulation, not CI data.
# Equally sized batch means have an exact multivariate normal law, so sample that law.
R,B,N=600,20,100000
m=R//B
inds=np.arange(R)
out['ar1_boundary_simulation']=[]
for rho in (0.0,0.5,0.9,0.99):
    C=rho**np.abs(inds[:,None]-inds[None,:])
    G=C.reshape(B,m,B,m).mean(axis=(1,3))
    varmean=G.sum()/B**2
    expected_se2=(np.trace(G)-B*varmean)/(B-1)/B
    rng=np.random.default_rng(6756614616)
    z=rng.multivariate_normal(np.zeros(B),G,size=N)
    crit=stats.t.ppf(.95,B-1)
    passed=z.mean(axis=1)+crit*z.std(axis=1,ddof=1)/math.sqrt(B)<=0
    out['ar1_boundary_simulation'].append({'rho':rho,'N':N,'nominal_size':.05,
        'empirical_size':float(passed.mean()),'mc_se':float(np.sqrt(passed.mean()*(1-passed.mean())/N)),
        'expected_batch_se_squared_over_true_mean_variance':float(expected_se2/varmean),
        'long_run_variance_inflation':(1+rho)/(1-rho)})

# The power formula's sign/critical value independently checked, using a robust normal/chi-square integral.
from scipy.integrate import quad
def power(R,sigma,mu,B=20):
    df=B-1;crit=stats.t.ppf(.95,df);delta=(math.log(1.02)-mu)*math.sqrt(R)/sigma
    return quad(lambda x:stats.norm.cdf(delta-crit*math.sqrt(x/df))*stats.chi2.pdf(x,df),0,np.inf,epsabs=1e-10)[0]
import service_tg_power as pw
runs,sds=pw.history(str(pathlib.Path(__file__).with_name('links.jsonl')))
out['power_history']={'n':len(runs),'first':runs[0]['date'],'last':runs[-1]['date'],'cells':len(sds)}
out['power_recalculation']=[]
for R in (600,900,960):
    for ratio in (1.0,1.005):
        ps=[power(R,pw.quantile(sds[k],.9),math.log(ratio)) for k in sds]
        out['power_recalculation'].append({'R':R,'B':20,'m':R//20,'ratio':ratio,'sd':'p90',
           'weakest':min(ps),'sum_misses':sum(1-p for p in ps),'lower':max(0,1-sum(1-p for p in ps)),'product':math.prod(ps)})
out['batch_duration_seconds_at_pilot_pace']=[30*1.17,30*2.22]
path=pathlib.Path(__file__).with_name('probe-results.json')
path.write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
