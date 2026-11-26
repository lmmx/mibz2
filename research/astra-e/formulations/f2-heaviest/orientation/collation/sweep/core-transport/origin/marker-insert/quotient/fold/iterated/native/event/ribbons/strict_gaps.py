"""All source-stable token columns in the declared finite family; every eligible schedule."""
from collections import Counter
from itertools import groupby,product
import json

def ribbons(image,rows):
    bands=[]
    for x in rows:
        y=image[x]
        if bands:
            s,d,w=bands[-1]
            if s+w==x and d+w==y:
                bands[-1]=(s,d,w+1)
                continue
        bands.append((x,y,1))
    return bands

def verify(column,counts):
    m=len(column)
    owners=sorted(column)
    p=[0]*m
    for k in set(owners):
        src=(i for i,c in enumerate(owners) if c==k)
        for j,z in enumerate(column):
            if z==k:p[next(src)]=j
    def alive(p,k):
        return [x for x in range(m) if owners[x]==k and p[x]>=0]
    active=sorted(set(owners))
    start=tuple(p);seen={start};todo=[start]
    origin=sum(len(ribbons(start,alive(start,k))) for k in active)
    while todo:
        p=todo.pop();counts['states']+=1
        rows={k:alive(p,k) for k in active}
        live=[x for x,y in enumerate(p) if y>=0]
        assert sorted(p[x] for x in live)==live
        old={k:ribbons(p,v) for k,v in rows.items()}
        total=sum(map(len,old.values()))
        assert total<=min(len(live),origin+m-len(live))
        for k,xs in rows.items():
            for x,y in zip(xs,xs[1:]):
                assert p[y]-p[x]>=y-x
            target={owners[p[x]] for x in xs}
            if len(target)!=1 or k in target:continue
            j=next(iter(target));q=list(p)
            for x in xs:q[x]=p[p[x]];q[p[x]]=-1
            generated=[]
            for u,v,w in old[k]:
                for t,y,b in old[j]:
                    lo=max(t,v);hi=min(t+b,v+w)
                    if lo<hi:generated.append((u+lo-v,y+lo-t,hi-lo))
            assert generated==ribbons(q,xs)
            counts['events']+=1;counts['intersections']+=len(generated)
            counts['new_strict_gaps']+=len(generated)-len(old[k])
            nq=tuple(q)
            if nq not in seen:seen.add(nq);todo.append(nq)

if __name__=='__main__':
    c=Counter()
    for alphabet,max_n in [(2,10),(3,7),(4,6)]:
        for n in range(1,max_n+1):
            for col in product(range(alphabet),repeat=n):verify(col,c);c['columns']+=1
    print(json.dumps(dict(c),indent=2))
