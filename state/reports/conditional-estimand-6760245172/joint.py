import json,math
import numpy as np
from scipy.stats import t
from s5 import s5
rng=np.random.default_rng(6760245173)
B,L,N=24,30,100000
tq=t.ppf(.95,23);tv=t.ppf(1-.01/(2*42),22);cut=.10439042628459579
idx=np.arange(B)-(B-1)/2;ss=idx@idx
g=np.full(B,-1/B);g[0]=(B-1)/B
def decide(c):
    y=c.mean(2);mu=y.mean(1)
    ni=mu+tq*y.std(1,ddof=1)/math.sqrt(B)<=0
    slope=y@idx/ss
    res=y-mu[:,None]-slope[:,None]*idx
    trend=np.abs(slope/np.sqrt((res**2).sum(1)/(22*ss)))>tv
    sv=s5(c.reshape(len(c),-1),L)>cut
    return np.array([ni.sum(),trend.sum(),sv.sum(),(ni&~trend&~sv).sum(),(~trend&~sv).sum()])
counts={k:np.zeros(5,dtype=np.int64) for k in ['iid','hetero43_flat','hetero43_nonflat','common_shift']}
for start in range(0,N,1000):
    z=rng.normal(size=(1000,B,L))
    counts['iid']+=decide(z)
    zh=z.copy();zh[:,0,:]*=math.sqrt(43)
    counts['hetero43_flat']+=decide(zh)
    counts['hetero43_nonflat']+=decide(zh+g[None,:,None]*math.sqrt(43/L))
    counts['common_shift']+=decide(z+rng.normal(size=(1000,1,1)))
out={'seed':6760245173,'N':N,'trend_cutoff':float(tv),'scope':'Synthetic per-cell joint rule; not 42-cell global adoption, pilot replay, or decoder data','models':{}}
for k,v in counts.items():
    out['models'][k]=dict(zip(['raw_size','trend_invalid','S5_invalid','joint_false_accept','both_diagnostics_pass'],(v/N).tolist()))
    out['models'][k]['false_accept_given_diagnostics_pass']=float(v[3]/v[4])
print(json.dumps(out,indent=2))
