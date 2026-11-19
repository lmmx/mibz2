from collections import Counter
from itertools import groupby,product
from bisect import bisect_right
import json

def rle(s):return [(k,len(list(g))) for k,g in groupby(s)]
def test(col,T):
    K=max(col)+1; freq=Counter(col);base=[0]
    for k in range(K):base.append(base[-1]+freq[k])
    owner=lambda y:bisect_right(base,y)-1
    P=sorted(range(len(col)),key=col.__getitem__)
    dead={P[x] for x in range(len(col)) if owner(x) not in T and owner(P[x]) in T}
    keep=[x for x in range(len(col)) if x not in dead];embed={x:i for i,x in enumerate(keep)}
    def key(k,j):return (k,None if k in T else j)
    ids={};truth=[None]*len(keep);words=[]
    for x in keep:
        k=owner(x);y=P[x];j=owner(y);c=key(k,j)
        if c not in ids:
            ids[c]=len(ids);words.append((k,j) if k not in T and j in T else (k,))
        dest=P[y] if y in dead else y
        truth[embed[dest]]=ids[c]

    # Metadata is independent of this implementation's read schedule.
    pairs=[{} for _ in range(K)]
    for pos,k in enumerate(col):
        j=owner(pos);pairs[k][j]=pairs[k].get(j,0)+1
    first=[];written=[]
    for k in range(K):
        first.append(len(written))
        if k in T:
            deleted=sum(pairs[a].get(k,0) for a in range(K) if a not in T)
            if deleted<freq[k]:written.append((k,))
        else:
            for j in pairs[k]:written.append((k,j) if j in T else (k,))
    assert words==written
    ords=[{j:i for i,j in enumerate(pairs[k])} for k in range(K)]
    cursor=base[:-1].copy();out=[];selected=Counter()
    for pos,k in enumerate(col):
        j=owner(pos)
        if k not in T:
            if j in T:continue
            out.append(first[k]+ords[k][j])
        else:
            u=col[cursor[k]];cursor[k]+=1
            out.append(first[u]+ords[u][k] if u not in T else first[k])
            selected[k]+=1
    assert all(cursor[k]==base[k+1] for k in T)
    assert out==truth
    # Test the distance lost before a surviving marked row even if it is not a singleton.
    for p in range(len(col)):
        u=col[p];k=owner(p)
        if p in dead:
            src=base[u]+sum(c==u for c in col[:p])
            actual=embed[src];offset=1
            assert actual==base[u]-sum(x<base[u] for x in dead)+src-base[u]
        else:src=p;actual=embed[p];offset=0;assert actual==p-sum(x<p for x in dead)
        if k not in T and freq[k]==1:
            assert offset==0 and ids[key(k,owner(P[p]))]==first[k]
    return len(keep),len(dead),sum(freq[t]==sum(owner(x)==t for x in dead) for t in T)

if __name__=='__main__':
    counts=Counter()
    for K,limit in [(2,9),(3,7),(4,6)]:
        for n in range(K,limit+1):
            for col in product(range(K),repeat=n):
                if len(set(col))!=K:continue
                for mask in range(1<<K):
                    T={k for k in range(K) if mask>>k&1}
                    q,d,e=test(col,T);counts.update(cuts=1,source_rows=n,kept=q,deleted=d,empty_targets=e,marks=n)
    print(json.dumps(counts,indent=2))
