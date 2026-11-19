"""Check every refinement by actual stable LF target intervals and modal pair histograms."""
from collections import Counter
from fractions import Fraction
from itertools import combinations
import json
from bisect import bisect_right

def all_sets(universe):
    for q in range(len(universe)+1):
        for s in combinations(universe,q):yield frozenset(s)

def example(text,lam):
    L=list(map(int,text));n=len(L);first=sorted(L)
    psi=sorted(range(n),key=L.__getitem__);lf=[psi.index(i) for i in range(n)]
    orbit=[0];x=lf[0]
    while x!=0:orbit.append(x);x=lf[x]
    assert len(orbit)==n
    P=frozenset(t for t in range(1,n) if first[t]!=first[t-1])
    universe=sorted(set(range(1,n))-P)
    supports={t:list(range(psi[t-1]+1,psi[t]+1)) for t in universe}
    def stats(T):
        Q=sorted(P|T);which=[bisect_right(Q,x) for x in range(n)]
        counts=[Counter() for _ in range(len(Q)+1)]
        old=[{} for _ in counts]
        for x,c in enumerate(L):
            j=which[x];target=which[lf[x]];counts[j][c,target]+=1
            old[j].setdefault(c,[]).append(lf[x])
        # Direct old-letter image straddling a proposed T seam is forbidden.
        neutral=all(not(min(v)<t<=max(v)) for d in old for v in d.values() for t in T)
        # The graph test is an independently phrased equivalent.
        assert neutral==all(set(supports[t])&(P|T) for t in T)
        return neutral,sum(max(c.values()) for c in counts)
    strict={frozenset()};weak={frozenset()};initial=stats(frozenset())[1]
    by={k:{} for k in ('neutral','prior_safe','one_at_a_time','no_initial_seed')}
    optimum={k:(Fraction(0),[]) for k in by}
    def record(kind,T,m):
        h=by[kind];h[len(T)]=max(h.get(len(T),-n),m)
        gain=m-initial-lam*len(T);prev,sets=optimum[kind]
        if gain>prev:optimum[kind]=(gain,[sorted(T)])
        elif gain==prev:sets.append(sorted(T))
    for T in all_sets(universe):
        ok,mass=stats(T)
        if not ok:continue
        if any(T-{t} in strict and set(supports[t])&(P|T-{t}) for t in T):strict.add(T)
        if any(T-{t} in weak for t in T):weak.add(T)
        record('neutral',T,mass)
        if T in strict:record('prior_safe',T,mass)
        if T in weak:record('one_at_a_time',T,mass)
        if all(not set(supports[t])&(P|{t}) for t in T):record('no_initial_seed',T,mass)
    assert optimum['neutral'][0]>optimum['one_at_a_time'][0]
    assert optimum['neutral'][0]==optimum['no_initial_seed'][0]
    return dict(last=text,lambda_exact=str(lam),initial_cuts=sorted(P),primary_zero_orbit=orbit,
                base_mass=initial,support_intervals=supports,mass_frontiers=by,
                best={k:dict(gain=str(v),choices=ts) for k,(v,ts) in optimum.items()})
if __name__=='__main__':
    print(json.dumps([example('111000001101000',Fraction(1)),example('2120010000',Fraction(1,2))],
                     indent=2))
