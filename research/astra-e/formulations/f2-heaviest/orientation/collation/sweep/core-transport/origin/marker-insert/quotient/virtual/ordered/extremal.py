"""Verify the sharp universal monotone-cursor bound and realize it by a fixed token alphabet."""
from bisect import bisect_left
from collections import Counter
from functools import lru_cache
from itertools import groupby,permutations
from math import isqrt
import json
from cursor_order import Graph,Quotient,full

def triangular_index(n):return (isqrt(8*n+1)-1)//2

def cover(s):
    if not s:return []
    budget=triangular_index(len(s))
    tops=[]; pos=[]; prev=[]; piles=[]
    for j,v in enumerate(s):
        i=bisect_left(tops,v)
        prev.append(-1 if i==0 else pos[i-1])
        if i==len(tops):
            tops.append(v);pos.append(j);piles.append([])
        else:tops[i]=v;pos[i]=j
        piles[i].append(v)
    if len(tops)<=budget:return piles
    p=pos[-1];lane=[]
    while p>=0:
        lane.append(s[p]);p=prev[p]
    lane.reverse()
    chosen=set(lane)
    return [lane]+cover([v for v in s if v not in chosen])

def verify(s,lanes):
    assert sorted(v for lane in lanes for v in lane)==sorted(s)
    index={x:i for i,x in enumerate(s)}
    for lane in lanes:
        assert all(index[x]<index[y] for x,y in zip(lane,lane[1:]))
        assert sorted(lane) in (lane,lane[::-1])
    assert len(lanes)<=triangular_index(len(s))

def minimum(s):
    masks=[]
    for mask in range(1,1<<len(s)):
        v=[x for i,x in enumerate(s) if mask>>i&1]
        if sorted(v) in (v,v[::-1]):masks.append(mask)
    @lru_cache(None)
    def cost(left):
        if not left:return 0
        first=left&-left
        return 1+min(cost(left^m) for m in masks if m&first and m&left==m)
    return cost((1<<len(s))-1)

def realize(pi):
    # Codes carry only the order of the next c contexts. They do not use d or c.
    width=max(1,(len(pi)-1).bit_length())
    payload=b''.join(bytes([1])+bytes(2+((x>>j)&1) for j in range(width-1,-1,-1)) for x in pi[::-1])
    w=bytes(len(payload)+1)+payload
    rotations=sorted(range(len(w)),key=lambda i:w[i:]+w[:i])
    last=[w[i-1] for i in rotations]
    f=Counter(last);order=sorted(f)
    raw=[(c,sum(1 for _ in g)) for c,g in groupby(last)]
    q=Quotient(raw,{c:bytes([c]) for c in order},order,f,{0},0)
    g=Graph(full(q),q.bases,[f[c] for c in order],0)
    assert g.parent==list(range(-1,len(g.parent)-1)) and len(g.parent)==len(payload)+1
    c=q.ids[1]
    actual=[x-q.bases[c] for k,x in zip(g.word,g.pos) if k==c]
    assert actual==list(pi)
    return len(w),len(raw)

exhaustive=[]
for n in range(1,8):
    best=0;seen=0
    for s in permutations(range(n)):
        verify(s,cover(s));best=max(best,minimum(s));seen+=1
    assert best==triangular_index(n)
    exhaustive.append(dict(n=n,permutations=seen,maximum_optimum=best))
families=[]
for k in range(1,15):
    start=0;pi=[]
    for width in range(k,0,-1):
        pi.extend(range(start+width-1,start-1,-1));start+=width
    n=len(pi);lanes=cover(pi);verify(pi,lanes)
    assert len(lanes)==k==triangular_index(n)
    rows,runs=realize(pi)
    families.append(dict(k=k,queries=n,rows=rows,runs=runs,lanes=len(lanes)))
print(json.dumps(dict(exhaustive=exhaustive,realized=families),indent=2))
