"""Independent stable-rank and every-successor tests on arbitrary marked columns."""
from collections import Counter
from itertools import accumulate,combinations,groupby,product
import json,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent.parent))
from clock import Clock,index,seq
from rope import Rope
from holes import pair_holes
from transport import acquire

def check_holes(limit):
    stats=Counter()
    for n in range(1,limit+1):
        for k in range(n+1):
            choices=[set(x) for x in combinations(range(n),k)]
            for U in choices:
                for V in choices:
                    got,_=pair_holes(n,U,V)
                    f=dict(zip((x for x in range(n)if x not in U),(x for x in range(n)if x not in V)))
                    for x in V:
                        y=x;t=0
                        while y not in U:y=f[y];t+=1
                        assert got[x]==(y,t);stats["nontrivial_paths"]+=t>1
                    stats["maps"]+=1
    return stats

def rle(data):return [(v,sum(1 for _ in h)) for v,h in groupby(data)]

def check_columns():
    result=Counter();maxima=Counter()
    for alphabet,nmax in [(2,7),(3,4)]:
        for n in range(1,nmax+1):
            for L in product(range(alphabet),repeat=n):
                runs=rle(L);result['columns']+=1
                for k in sorted({0,1,2,3,n+1}):
                    c=Clock(L,k);cnt=Counter(c.state);start=[0,*accumulate(cnt[x] for x in range(len(c.words)))]
                    ex={x for x in range(n) if L[x]!=c.modal[c.state[x]]}
                    for mark in range(n):
                        res,stats=acquire(runs,start,c.modal,mark)
                        E=ex|{mark};assert set(res)==E
                        for x in E:
                            y=c.lf[x];steps=1
                            while y not in E:y=c.lf[y];steps+=1
                            assert res[x]==(y,steps)
                        y=mark;ell=0;clock=[]
                        while True:
                            if y in ex:clock.append((ell,c.state[c.lf[y]] if k else L[y]))
                            y,h=res[y];ell+=h
                            if y==mark:break
                        period=c.emit((c.state[mark],clock),ell)
                        for horizon in (0,n,n+1,2*n+3):
                            got=(period*((horizon+ell-1)//ell))[:horizon]
                            assert got==seq(c.F,c.psi,mark,horizon);result['emitted_horizons']+=1
                        result['marked_tables']+=1;result['event_returns']+=len(res)
                        result['all_default_marks']+=not ex;result['foreign_cycle_marks']+=ell<n
                        result['anchored_real_paths']+=stats['real_via_anchors']
                        result['nontrivial_fifo_cases']+=stats['maximum_fifo']>1
                        for name in ('cycle_anchors','maximum_fifo','regular_rows_bulk'):
                            maxima[name]=max(maxima[name],stats[name])
    return dict(totals=result,maxima=maxima)

def huge():
    N=2**48+12
    U={N-2,N-1};V={0,1};p,bulk=pair_holes(N,U,V)
    assert p=={0:(N-2,N//2-1),1:(N-1,N//2-1)}
    z,_=pair_holes(N,V,U);assert z=={u:(v,h)for v,(u,h)in p.items()}
    # Binary variable-gap lift from CLOCK-REDUCTION, represented in runs only.
    # Physical exception rows E are first a of each variable gap. Their first LF
    # image is the original b source; this exposes the exact old LF partners.
    G=2**42+1;k=3;L=(1,0,0,1,1,0,1);m=len(L);count0=L.count(0)
    n=G*m+m+count0;B=n-m;p=2;mark=B+p
    runs=[(1,count0),*rle(L),(0,G*m)]
    merged=[]
    for x,w in runs:
        if merged and merged[-1][0]==x:merged[-1]=(x,merged[-1][1]+w)
        else:merged.append((x,w))
    start=[0]+[n-k*m+j*m for j in range(k+1)];assert start[-1]==n
    got,stats=acquire(merged,start,[0]*(k+1),mark)
    F,psi,lf=index(L);count1=m-count0;old_to_event={}
    for u,ch in enumerate(L):
        # Terminating b in the long-only highest layer, or in the full next layer.
        old_to_event[u]=L[:u].count(0) if ch==0 else count0+u
    for x in range(m):
        e=old_to_event[x];v,age=got[e];expect=old_to_event[lf[x]]
        if lf[x]==p:
            assert v==mark and age==1
            v,h=got[v];age+=h
        assert v==expect and age==G+2-L[lf[x]]
    assert stats['virtual_rows']>G and stats['rope_nodes_created']<200
    return dict(monotone_line=N,positive_and_negative_regular_gap=bulk,lifted_rows=n,source_rows=m,stats=stats)
if __name__=='__main__':
    print(json.dumps(dict(partial_maps=check_holes(8),marked=check_columns(),large=huge()),indent=2))
