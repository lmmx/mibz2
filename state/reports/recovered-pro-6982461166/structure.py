from bisect import bisect_left
from collections import Counter
from itertools import groupby, product, accumulate
from pathlib import Path
from functools import lru_cache


def endpoint(n, strips):
    # The two orders have the same labelled widths. IDs identify a
    # complementary translation or one of the empty boundary connectors.
    width = [0] + [w for x, y, w in strips]
    prefix = [[], []]
    suffix = [[], []]
    for side in (0,1):
        prior = 0
        for x,w,i in sorted((r[side],r[2],i+1) for i,r in enumerate(strips)) + [(n,0,0)]:
            if prior < x:
                k = len(width)
                width.append(x-prior)
                suffix[side].append(k)
                prefix[1-side].append(k)
            if i:
                suffix[side].append(i)
            prior=x+w

    before=[[0]*len(width) for _ in range(2)]
    after=[[0]*len(width) for _ in range(2)]
    for side in (0,1):
        order=[0]+prefix[side]+suffix[side]+[0]
        for a,b in zip(order,order[1:]):
            after[side][a]=b
            before[side][b]=a

    def cut(i,s):
        left,right=before[s][i],after[s][i]
        after[s][left], before[s][right]=right,left

    def insert(i, at, s):
        nxt=after[s][at]
        after[s][at],before[s][i],after[s][i],before[s][nxt]=i,at,nxt,i

    remaining, steps = n,0
    while remaining:
        a,b=before[0][0],before[1][0]
        taken=min(width[a],width[b])
        assert 0<taken<=remaining
        remaining-=taken
        steps+=1
        if a==b:
            cut(a,0)
            cut(a,1)
        elif width[a]<=width[b]:
            cut(a,0)
            insert(a,b,0)
            width[b]-=taken
            if width[b]==0:
                cut(b,0)
                cut(b,1)
        else:
            cut(b,1)
            insert(b,a,1)
            width[a]-=taken
    return len(width)-1,steps


def strip_table(column, mark, limit):
    counts=Counter(column)
    next_=list(accumulate([0]+[counts[i] for i in range(256)]))
    x=0
    strips=[]
    def add(lo,hi,c,y):
        if hi>lo:
            strips.append((lo,y,hi-lo))
    for c, g in groupby(column):
        w=sum(1 for _ in g)
        y=next_[c]
        if w>limit:
            if x<=mark<x+w:
                add(x,mark,c,y)
                add(mark+1,x+w,c,y+mark+1-x)
            else:
                add(x,x+w,c,y)
        next_[c]+=w
        x+=w
    return strips


def isolated(n, strips):
    incident=bytearray(n)
    for x,y,w in strips:
        incident[x:x+w]=bytes([1])*w
        incident[y:y+w]=bytes([1])*w
    dead=[i for i, flag in enumerate(incident) if not flag]
    revised=[(x-bisect_left(dead,x),y-bisect_left(dead,y),w) for x,y,w in strips]
    return len(dead),revised


def greedy(seed, target):
    x=seed+target
    p=len(seed)
    count=0
    while p<len(x):
        k=max(l for l in range(1,len(x)-p+1) if x[p:p+l] in x[:p])
        count+=1
        p+=k
    return count


def optimum(seed, target):
    x=seed+target
    @lru_cache(None)
    def count(p):
        if p==len(x):
            return 0
        return min(1+count(p+l) for l in range(1,len(x)-p+1) if x[p:p+l] in x[:p])
    return count(len(seed))


def check_greedy():
    cases=0
    for m in range(2,6):
        for t in range(8):
            for d in product("ab",repeat=m):
                d="".join(d)
                if len(set(d))<2:
                    continue
                for w in product("ab",repeat=t):
                    w="".join(w)
                    assert greedy(d,w)==optimum(d,w)
                    cases+=1
    return cases


if __name__=="__main__":
    col=Path("frozen.L").read_bytes()
    strips=strip_table(col,264045,16)
    n=len(col)
    dead, revised=isolated(n,strips)
    a,old=endpoint(n,strips)
    b,new=endpoint(n-dead,revised)
    assert (a,old)==(7786,11215)
    print("frozen",len(strips),dead,a,old,b,new,(old-new)/old)
    print("independent greedy vs optimum checks",check_greedy())
