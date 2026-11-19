"""Order ambiguity persists in a nontrivial cut with fixed Q-ordered band descriptors."""
from collections import Counter
from itertools import combinations
from math import comb


def cut(f, incoming, inside):
    m=len(f)
    target=sorted(range(m),key=incoming.__getitem__)
    pred=[0]*m
    for x,y in enumerate(target):
        pred[y]=x
    keep=[not (f[x] in inside and f[pred[x]] not in inside) for x in range(m)]
    sources=[]
    ids={}
    last=None
    for i in sorted(set(f)):
        previous=None
        for x in (j for j,k in enumerate(f) if k==i):
            j=f[target[x]]
            if not keep[x]:
                continue
            group=(i,j if i not in inside and j in inside else None)
            if group!=previous:
                sources.append(group)
                previous=group
            ids[x]=len(sources)-1
    out=[]
    for y in range(m):
        if keep[y]:
            z=pred[y]
            if not keep[z]:
                z=pred[z]
            out.append(ids[z])
    return sources,tuple(out)


f=(0,1,2,2)
left=(1,2,0,2)
right=(1,2,2,0)
assert Counter(zip(f,left))==Counter(zip(f,right))
a=cut(f,left,{1})
b=cut(f,right,{1})
assert a[0]==b[0] and a[1]==(1,0,2) and b[1]==(1,2,0)
print('Singleton primary 0, F=0122, inside={1}: same counts; ordered outputs 102 vs 120.')

# F = D A^s B^s J^(2s) C. D and C form a separate 2-cycle.
# Incoming at A and B is J. The destination bucket J interleaves A and B.
for s in range(1,7):
    oldf=(0,)+(1,)*s+(2,)*s+(3,)*(2*s)+(4,)
    seen=set()
    masses=None
    for arows in combinations(range(2*s),s):
        chosen=set(arows)
        old=(4,)+(3,)*(2*s)+tuple(1 if i in chosen else 2 for i in range(2*s))+(0,)
        e=Counter(zip(oldf,old))
        if masses is not None:assert masses==e
        masses=e
        _,result=cut(oldf,old,{4})
        seen.add(result)
    assert len(seen)==comb(2*s,s)
    print(f's={s} identical pair-mass and single-band-per-type schedules, distinct ordered outputs={len(seen)}')
