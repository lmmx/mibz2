"""Range sweeps, fractional boundaries, arbitrary true columns and huge positive-width instances."""
import itertools,json,sys
from bisect import bisect_right
from collections import Counter
from itertools import accumulate
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[2]))
from rangemap import create,span
from sweep import route_holes
from holes import pair_holes
from clock import Clock

def grouped(S,n,salt):
    out=[]
    for x in sorted(S):
        if out and out[-1][0]+out[-1][2]==x and (x*17+salt)%4:
            p,y,w,h=out[-1];out[-1]=(p,y,w+1,h)
        else:out.append((x,(n+1)*(n-x)+x,1,(3*x+salt)%7))
    return out

def holes():
    r=Counter()
    for n in range(1,9):
        for k in range(n+1):
            sets=[set(x) for x in itertools.combinations(range(n),k)]
            for i,U in enumerate(sets):
                for j,V in enumerate(sets):
                    u=grouped(U,n,i+j);v=grouped(V,n,3*i+j)
                    q,st=route_holes(n,u,v,7)
                    old,_=pair_holes(n,U,V)
                    ut={p+a:(z+a,h) for p,z,w,h in u for a in range(w)}
                    vt={p+a:(z+a,h) for p,z,w,h in v for a in range(w)}
                    want={vt[b][0]:(ut[x][0],ut[x][1]+7*h+vt[b][1]) for b,(x,h) in old.items()}
                    got={s+a:(t+a,h) for s,t,w,h in q for a in range(w)}
                    assert got==want and sum(w for s,t,w,h in q)==len(want)
                    r['partial_maps']+=1;r['sign_splits']+=st['sign_splits'];r['emitted_intervals']+=len(q)
    return r

def query(canon,union,x):
    lookup=[s for s,t,w,h in canon];v=union.rank(x);j=bisect_right(lookup,v)-1
    s,t,w,h=canon[j];assert s<=v<s+w
    return union.select(t+v-s),h

def columns():
    r=Counter()
    for alphabet,nmax in ((2,7),(3,4)):
        for n in range(1,nmax+1):
            for L in itertools.product(range(alphabet),repeat=n):
                runs=[]
                for i,c in enumerate(L):
                    if runs and runs[-1][0]==c:runs[-1]=(c,runs[-1][1]+1)
                    else:runs.append((c,1))
                for k in sorted({0,1,2,3,n+1}):
                    c=Clock(L,k);counts=Counter(c.state);start=[0,*accumulate(counts[x] for x in range(len(c.words)))]
                    physical={x for x in range(n)if L[x]!=c.modal[c.state[x]]}
                    for mark in range(n):
                        pieces,union,st=create(runs,start,c.modal,mark)
                        E=physical|{mark}
                        for x in E:
                            y=c.lf[x];h=1
                            while y not in E:y=c.lf[y];h+=1
                            assert query(pieces,union,x)==(y,h)
                        r['marked_models']+=1;r['physical_partners']+=len(E);r['return_pieces']+=len(pieces)
                        r['pure_default_cases']+=not physical;r['disconnected_cases']+=c.lap(mark)[0]<n
    return r

def huge():
    # More than a trillion holes on either side with unequal terminal alignment.
    w=10**12+7;m=4096;rem=w//3;n=(m+1)*w+rem;U=[(n-w,81,w,23)];V=[(0,13,w,17)]
    q,bill=route_holes(n,U,V,11)
    want=[(13,81+w-rem,rem,(m+1)*11+40),(13+rem,81,w-rem,m*11+40)]
    assert sorted(q)==want
    q2,_=route_holes(n,[(0,81,w,23)],[(n-w,13,w,17)],11)
    assert sum(span for _,_,span,_ in q2)==w and len(q2)==2
    for s,t,span,age in q2:
        for z in (0,span-1):
            x=n-w+s-13+z;h=x//w
            assert x%w==t-81+z and age==40+11*h
    # A thick stable cover gives physical minority mass proportional to N, but P is unchanged.
    L=(2,1,0,0,2,1,0,1,0);c=Clock(L,1);g=2**40+7
    cnt=Counter(c.state);start=[0,*accumulate(g*cnt[x] for x in range(len(c.words)))]
    runs=[]
    for b in L:
        if runs and runs[-1][0]==b:runs[-1]=(b,runs[-1][1]+g)
        else:runs.append((b,g))
    ex={x for x in range(len(L))if L[x]!=c.modal[c.state[x]]}
    primary=next(x for x in range(len(L))if x not in ex);lane=g//5;mark=g*primary+lane
    pieces,union,cost=create(runs,start,c.modal,mark);assert cost['events']==g*len(ex)+1
    offsets={0,g-1,lane-1,lane,lane+1}
    for s,t,w,h in pieces:offsets.update((union.select(s)%g,union.select(s+w-1)%g))
    verified=0
    for base in ex|{primary}:
        for z in offsets:
            if base not in ex and z!=lane:continue
            y=c.lf[base];h=1
            while y not in ex and not(y==primary and z==lane):y=c.lf[y];h+=1
            assert query(pieces,union,g*base+z)==(g*y+z,h);verified+=1
    assert cost['birth_intervals']<20 and cost['rope_nodes_created']<150
    return dict(annotated_holes=w,anchor_line=n,published=sorted(q),sweep=bill,
                lifted_rows=g*len(L),stable_lane_probes=verified,ranges=cost)
if __name__=='__main__':print(json.dumps(dict(holes=holes(),columns=columns(),huge=huge()),indent=2))
