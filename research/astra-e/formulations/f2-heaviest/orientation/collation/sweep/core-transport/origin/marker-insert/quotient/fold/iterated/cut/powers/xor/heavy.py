"""A forced heaviest XOR-matching witness and an independent finite check."""
from collections import Counter
from itertools import product
import json
from chooser import select,signed


def witness(d):
    size=len(d)
    if size==1:
        return 0,d[0],0
    t=max(range(1,size),key=d.__getitem__)
    pivot=1<<(t.bit_length()-1)
    def change(x):
        if x&pivot:
            x^=t^pivot
        if bool(x&pivot)!=bool(x&1):
            x^=pivot|1
        return x
    assert change(t)==1
    transformed=[0]*size
    for x,n in enumerate(d):
        transformed[change(x)]=n
    mask=1
    bound=transformed[0]-transformed[1]
    width=2
    while width<size:
        value=sum(signed(mask,x)*transformed[x] for x in range(width,2*width))
        if value>0:
            mask|=width
        bound-=abs(value)
        width*=2
    old=sum(1<<i for i in range(size.bit_length()-1) if signed(mask,change(1<<i))<0)
    assert signed(old,t)<0 and sum(signed(old,x)*w for x,w in enumerate(d))==bound<=d[0]-d[t]
    return old,bound,d[t]


def main():
    count=Counter()
    for n in range(1,8):
        for k in range(1,min(n,4)+1):
            q=1<<(k-1).bit_length()
            for col in product(range(k),repeat=n):
                if len(set(col))!=k:continue
                d=[0]*q
                g=[{} for _ in range(k)]
                for a,b in zip(col,sorted(col)):
                    d[a^b]+=1
                    g[a][b]=g[a].get(b,0)+1
                m,score,bonus=witness(d)
                _,_,greedy=select(g,0)
                _,_,best=select(g,0,True)
                inside={i for i in range(k) if signed(m,i)<0}
                actual=sum(w for a,edges in enumerate(g) if a not in inside
                           for b,w in edges.items() if b in inside)
                assert 4*actual==n-score and actual<=best and 4*best>=n-d[0]+bonus
                count.update(histograms=1,strictly_beats_unforced=int(actual>greedy),
                             strictly_loses_to_unforced=int(actual<greedy),reaches_exact_optimum=int(actual==best))
    print(json.dumps(count,indent=2))


if __name__=='__main__':main()
