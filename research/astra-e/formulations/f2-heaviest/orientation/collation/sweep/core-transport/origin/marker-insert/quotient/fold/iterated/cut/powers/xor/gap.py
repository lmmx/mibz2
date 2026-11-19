"""Contiguous unselected-neighbor aliases and an independent dense partner check."""
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from cut import append_run,forward


def split(slices,edges,words,target,protected):
    alias={}
    weights=[]
    selected=set()
    for k,g in enumerate(edges):
        last=None
        for j in g:
            group=0 if k in target else (1+j if j in target else -1)
            if group!=last:
                key=len(weights)
                weights.append(words[k])
                if k not in target and j in target:
                    selected.add(key)
                last=group
            alias[k,j]=key
    raw=[]
    for k,j,n in slices:
        append_run(raw,alias[k,j],n)
    return raw,weights,selected,alias[protected,next(iter(edges[protected]))]


def check(root,bounds,words,refined,rb,rwords,f,mark,t):
    old=forward(root,bounds[:-1])
    mid=forward(refined,rb[:-1])
    final=forward(f.root,f.bases)
    prior=None
    rid=-1
    for (y,k),(z,u) in zip(old,mid,strict=True):
        j=old[y][1]
        label=(k,0 if k in t else (1+j if j in t else -1))
        if label!=prior:
            rid+=1
            prior=label
        assert y==z and u==rid and rwords[u] is words[k]
        assert (u in f.chosen)==(k not in t and j in t)
    assert rid+1==len(rwords)
    dead={y for y,k in mid if k in f.chosen}
    keep={x:i for i,x in enumerate(x for x in range(len(old)) if x not in dead)}
    for x,i in keep.items():
        y,k=mid[x]
        z,u=final[i]
        assert u==f.ids[k]
        if y in dead:
            target,j=mid[y]
            assert f.weights[u].left is rwords[k] and f.weights[u].right is rwords[j]
        else:
            target=y
            assert f.weights[u] is rwords[k]
        assert z==keep[target]
    return len(old)+len(final),keep[mark]
