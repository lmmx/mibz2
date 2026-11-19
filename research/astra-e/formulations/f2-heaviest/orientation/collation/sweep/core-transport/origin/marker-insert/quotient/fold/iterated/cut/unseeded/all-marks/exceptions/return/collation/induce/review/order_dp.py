"""Independent subset costs and exhaustive complete-order checks of fixed-section induction."""
from collections import Counter
from functools import lru_cache
from itertools import permutations,product
from pathlib import Path
import json
import random
import sys
from finite import chunks,stable_lf

def starting_atoms(last,lf,cid):
    result=[];k=-1
    for i,b in enumerate(last):
        if i==0 or b!=last[i-1] or cid[i]!=cid[i-1] or cid[lf[i]]!=cid[lf[i-1]]:k+=1
        result.append(k)
    return result

def section(lf,atoms,order):
    locate={x:i for i,x in enumerate(order)}
    targets=[];segment=[];last_id=0;prev=None
    for x in order:
        word=[]
        while True:
            word.append(atoms[x]);x=lf[x]
            if x in locate:break
        if word!=prev:last_id+=1;prev=word
        if segment and last_id==segment[-1]:assert locate[x]==targets[-1]+1
        segment.append(last_id);targets.append(locate[x])
    return targets,segment

def tail(t,ids,ends):
    counts=Counter();nodes=max(ids)+1
    while len(t)>ends[-1][0]:
        start,identity=next((s,k) for s,k in ends if s<len(t))
        a,b=ids[-1],ids[t.index(len(t)-1)];width=min(ids.count(a),ids.count(b))
        stop=len(t)-width
        assert stop>=start
        counts[identity]+=1;used=False;nt=[];ni=[]
        for i in range(stop):
            j=t[i]
            if j<stop:
                nt.append(j);ni.append(ids[i])
            else:
                assert ids[i]==b and ids[j]==a
                nt.append(t[j]);ni.append(nodes);used=True
        if used:nodes+=1
        t,ids=nt,ni
    return counts

def audit(last,kept):
    n=len(last);comps=chunks(x for x in range(n) if x not in kept)
    if not 2<=len(comps)<=6:return None
    k=len(comps);allset=(1<<k)-1;V=sorted(kept);lf=stable_lf(last)
    # All initial V AND complement contiguous components have separate endpoints.
    cid=[None]*n
    for i,rows in enumerate([*chunks(V),*comps]):
        for q in rows:cid[q]=i
    atoms=starting_atoms(last,lf,cid)

    @lru_cache(None)
    def edge(mask,out):
        sequence=V+[x for i,r in enumerate(comps) if mask>>i&1 and i!=out for x in r]+comps[out]
        t,s=section(lf,atoms,sequence)
        return tail(t,s,[(len(sequence)-len(comps[out]),out)])[out]

    @lru_cache(None)
    def optimum(mask):
        return min((edge(mask,x)+optimum(mask^(1<<x)) for x in range(k) if mask>>x&1),default=0)

    actual=[]
    for perm in permutations(range(k)):
        sequence=V.copy();ends=[]
        for x in perm:
            ends.append((len(sequence),x));sequence.extend(comps[x])
        t,s=section(lf,atoms,sequence)
        costs=tail(t,s,list(reversed(ends)))
        mask=allset
        for x in reversed(perm):
            assert costs[x]==edge(mask,x)
            mask^=1<<x
        actual.append(sum(costs.values()))
    assert min(actual)==optimum(allset)
    return dict(k=k,orders=len(actual),edges=edge.cache_info().currsize,stable=actual[0],best=min(actual),worst=max(actual))

total=Counter();best_example=None
def record(l,v):
    global best_example
    z=audit(l,v)
    if z is None:return
    total["sections"]+=1;total["complete_orders"]+=z["orders"];total["distinct_subset_edges"]+=z["edges"]
    total["physical_component_removals"]+=z["orders"]*z["k"]
    total["order_matters"]+=int(z["best"]!=z["worst"])
    total["stable_suboptimal"]+=int(z["best"]<z["stable"])
    if z["best"]<z["stable"] and (best_example is None or z["stable"]-z["best"]>best_example["stable"]-best_example["best"]):
        best_example=dict(last=l,V=sorted(v),**z)
for n in range(2,7):
    for l in product(range(2),repeat=n):
        for mask in range(1,(1<<n)-1):
            record(l,{i for i in range(n) if mask>>i&1})
rng=random.Random(718982)
for _ in range(400):
    n=rng.randint(8,21);l=tuple(rng.randrange(3) for _ in range(n))
    v={x for x in range(n) if rng.random()<.42};v.add(rng.randrange(n));record(l,v)
p=Path(sys.argv[1]);p.write_text(json.dumps(dict(counts=total,witness=best_example),indent=2)+"\n")
