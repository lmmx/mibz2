import json, math
import numpy as np
from scipy.stats import t, norm
from s5 import s5

rng=np.random.default_rng(6760245172)
B,L=24,30
cut=0.10439042628459579
tq=float(t.ppf(.95,23))
N=100000
out={'seed':6760245172,'N':N,'tq':tq}
# Independent normal batches: a nonflat fixed conditional mean can increase t size.
g=np.full(B,-1/B);g[0]=(B-1)/B
for ratio in [1,10,43,100,1000000]:
    z=rng.normal(size=(N,B));z[:,1:]/=math.sqrt(ratio)
    def reject(y): return y.mean(1)+tq*y.std(1,ddof=1)/math.sqrt(B)<=0
    out['hetero_ratio_'+str(ratio)]={'flat_size':float(reject(z).mean()),'nonflat_size':float(reject(z+g).mean())}
out['degenerate_limit_size']=float(norm.cdf(-tq/(tq+1))-norm.cdf(-tq/(tq-1)))
# S5 false alarms under independent Gaussian noise plus a fixed within-batch path.
shape=np.arange(L)-(L-1)/2
shape=shape/np.sqrt(np.mean(shape**2))
counts={str(a):0 for a in [0,.25,.5,1,2]}
hetero_count=0;common_reject=0;common_pass_reject=0
max_invariance=0.0
for start in range(0,N,1000):
    z=rng.normal(size=(1000,B,L))
    v=s5(z.reshape(1000,-1),L)
    for a in [0,.25,.5,1,2]:
        counts[str(a)]+=int((s5((z+a*shape).reshape(1000,-1),L)>cut).sum())
    zh=z.copy();zh[:,0,:]*=math.sqrt(43)
    hetero_count+=int((s5(zh.reshape(1000,-1),L)>cut).sum())
    common=rng.normal(size=(1000,1,1))
    y=z+common
    vv=s5(y.reshape(1000,-1),L)
    max_invariance=max(max_invariance,float(np.max(abs(v-vv))))
    means=y.mean(2)
    rej=means.mean(1)+tq*means.std(1,ddof=1)/math.sqrt(B)<=0
    common_reject+=int(rej.sum());common_pass_reject+=int((rej&(vv<=cut)).sum())
out['S5_fixed_ramp_false_alarm']={a:n/N for a,n in counts.items()}
out['S5_independent_gaussian_one_batch_variance43_false_alarm']=hetero_count/N
out['common_shift']={'raw_size':common_reject/N,'size_after_S5':common_pass_reject/N,'max_S5_difference':max_invariance}
out['scope']='Synthetic mathematical countermodels, not pilot replay or decoder data; trend veto not simulated.'
print(json.dumps(out,indent=2))
