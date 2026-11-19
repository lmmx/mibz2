"""Exact one-splice certificate for an arbitrary permutation and a proposed modular shift."""
from itertools import permutations
from math import gcd
from numeric import index
import json

def certify(p,a):
    n=len(p);h=gcd(a,n);d=n//h;step=pow(a//h,-1,d) if d>1 else 0
    b=list(range(h));dest=[None]*h
    for r,z in enumerate(p):
        old=(r+a)%n
        if old==z:continue
        t=r%h
        if dest[t] is not None:return None
        b[t]=old;dest[t]=z
    perm=[]
    for t,z in enumerate(dest):
        if z is None:perm.append(t)
        else:
            u=z%h;assert z==b[u];perm.append(u)
    assert sorted(perm)==list(range(h))
    tag=[None]*h;order=[0]*h;weight=[]
    for start in range(h):
        if tag[start] is not None:continue
        z=start;count=0
        while tag[z] is None:tag[z]=len(weight);order[z]=count;count+=1;z=perm[z]
        assert z==start;weight.append(d*count)
    def at(r):
        t=r%h
        return tag[t],d*order[t]+(step*(r//h-b[t]//h))%d
    return at,weight

def brute_case(p,a):
    # Independently read the actual cycles; each participating lane must occupy
    # one cyclic contiguous arc and every interior step must be the fixed shift.
    n=len(p);h=gcd(a,n);seen=set()
    for r in range(n):
        if r in seen:continue
        cycle=[];x=r
        while x not in seen:seen.add(x);cycle.append(x);x=p[x]
        assert x==r
        for t in {x%h for x in cycle}:
            seq=[x%h==t for x in cycle]
            if sum(seq)!=n//h or sum(not seq[j-1] and yes for j,yes in enumerate(seq))>1:return False
            if all(seq):
                if any(p[x]!=(x+a)%n for x in cycle):return False
            elif any(p[x]!=(x+a)%n for x in cycle if x%h==t and p[x]%h==t):return False
    return True

def verify():
    cases=accepted=0
    for n in range(2,8):
        for p in permutations(range(n)):
            for a in range(1,n):
                cert=certify(p,a);assert (cert is not None)==brute_case(p,a);cases+=1
                if cert:
                    at,w=cert;accepted+=1
                    assert len({at(x) for x in range(n)})==sum(w)==n
                    for x,y in enumerate(p):
                        k,t=at(x);assert at(y)==(k,(t+1)%w[k])
    families=sharp=0
    for h in range(2,35):
        for d in range(2,37):
            n=h*d;L=(1,0)+(1,)*(h-1)+(0,)*(n-h-1);_,p=index(L)
            at,w=certify(p,h);assert w==[n];origin=at(0)[1]
            o=[(at(r)[1]-origin)%n for r in range(n)]
            for r in range(n):
                q,t=divmod(r,h);want=(t-1)*d+q+1 if t else ((h-1)*d+q if q else 0)
                assert o[r]==want and o[p[r]]==(o[r]+1)%n
            families+=1
            if h==2:
                delta=[(y-x)%n for x,y in zip(o,o[1:])]
                assert not any(x==y for x,y in zip(delta,delta[1:]));sharp+=1
    print(json.dumps(dict(arbitrary_permutation_shift_cases=cases,admitted=accepted,four_run_families=families,unrestricted_slope_worst_cases=sharp),indent=2))
if __name__=="__main__":verify()
