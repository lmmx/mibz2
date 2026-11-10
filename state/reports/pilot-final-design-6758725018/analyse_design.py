"""Offline design arithmetic; no candidate mean, ratio, or qualification bound is printed.

Inputs: menu.json regenerated with pinned tg_ni_pilot.py from run 42748681243.
Power integral: pinned service_tg_power.py (saved as power.py).
No decoder execution. Gaussian draws below are statistical sensitivity simulations.
"""
import json, math, statistics
from pathlib import Path
import numpy as np
from scipy.stats import t
from scipy import stats, integrate
from power import power

HERE = Path(__file__).resolve().parent
v = json.loads((HERE / 'menu.json').read_text())
assert v['valid'] and len(v['cells']) == 42
sigmas = {k: math.sqrt(max(b['lrv'] for b in c['batches'])) for k,c in v['cells'].items()}
rows = []
for R,m in [(600,30),(960,48),(1200,60),(1440,60),(2400,60),(2400,120),(2880,120)]:
    for inflation in [1,2]:
        p = {k: power(R,R//m,s*math.sqrt(inflation),1.005) for k,s in sigmas.items()}
        miss = sum(1-a for a in p.values())
        rows.append(dict(R=R,m=m,B=R//m,variance_multiplier=inflation,
                         min_power=min(p.values()),limiting=min(p,key=p.get),
                         all_cell_lower=max(0,1-miss),independent_product=math.prod(p.values()),
                         with_trend_veto_lower=max(0,1-miss-.01)))

B=24
cutoff=float(t.ppf(1-.01/(2*42),B-2))
# Exact batch-level Gaussian AR(1) covariance scenarios: does not fit CI or claim calibration.
rng=np.random.default_rng(6758725018)
N=200000
models=[]
for rho in [0,.1,.2,.4]:
    cov=rho**np.abs(np.arange(B)[:,None]-np.arange(B)[None,:])
    L=np.linalg.cholesky(cov)
    passes=0; passes_with_veto=0
    tt=np.arange(B)-(B-1)/2
    sxx=float(tt@tt)
    for _ in range(N//10000):
        y=rng.standard_normal((10000,B))@L.T
        ym=y.mean(axis=1);se=y.std(axis=1,ddof=1)/math.sqrt(B)
        passed=ym+t.ppf(.95,B-1)*se<=0 # centred at NI null boundary
        slope=(y@tt)/sxx
        resid=y-ym[:,None]-slope[:,None]*tt
        tse=np.sqrt((resid**2).sum(axis=1)/(B-2)/sxx)
        trend=np.abs(slope/tse)>cutoff
        passes+=int(passed.sum());passes_with_veto+=int((passed&~trend).sum())
    models.append(dict(batch_rho=rho,draws=N,null_pass=passes/N,
                       null_pass_with_trend_veto=passes_with_veto/N))

budgets={k:dict(measured_minutes=s['measurement_s']/60,
               R2880_measure_minutes=s['measurement_s']/60*4.8,
               R2880_plus_15_setup=s['measurement_s']/60*4.8+15,
               R2880_1p4_pace_plus_15=s['measurement_s']/60*4.8*1.4+15)
         for k,s in v['shards'].items()}
# The 20-second settle sleeps precede the child. But waited_s itself is emitted
# after the child: elapsed since before settle minus one settle interval, rounded.
# Its sum is NOT pure waiting. Preserve every row; report this diagnostic literally.
waits={}
for f in sorted((HERE/'artifacts').glob('tg-ni-*-a1/*/runs.jsonl')):
    rs=[json.loads(l) for l in f.read_text().splitlines() if l.strip()]
    ws=[r['waited_s'] for r in rs]
    waits[f.parent.name]=dict(rows=len(rs),reported_waited_s_sum=sum(ws),
                              reported_waited_s_max=max(ws),rows_field_over_10=sum(w>10 for w in ws))

# Algebraic cancellation of constant and linearly varying order coefficients.
s=np.array([1,-1,-1,1]);assert s.sum()==0 and s@np.arange(4)==0

out=dict(planning_sigmas=sigmas,power=rows,trend_cutoff=cutoff,
         gaussian_batch_sensitivity=models,shard_budget=budgets,waits=waits,
         total_measure_minutes=sum(s['measurement_s'] for s in v['shards'].values())/60,
         power_assumption='iid normal batch means, pilot max LRV scaled as stated; not guaranteed',
         trend_veto_assumption='constant mean iid Gaussian batch means, Bonferroni family veto <=.01')
sig=sigmas['wikidata/1/wall'];lam=math.log(1.005)*math.sqrt(2880)/sig
flag=integrate.quad(lambda z:stats.norm.cdf(lam-t.ppf(.975,23)*math.sqrt(z/23))*stats.chi2.pdf(z,23),0,math.inf)[0]
out['harm_rule_example']=dict(cell='wikidata/1/wall',planning_sigma=sig,
    assumed_true_ratio=1.005,R=2880,B=24,noncentrality=lam,slower_flag_probability=flag,
    assumption='iid Gaussian batches and same hypothetical true ratio; not a pilot mean or recurrence claim across CPU models')
(HERE/'design-results.json').write_text(json.dumps(out,indent=2,sort_keys=True)+'\n')
print(json.dumps({k:out[k] for k in ['power','trend_cutoff','gaussian_batch_sensitivity','shard_budget','waits']},indent=2))
