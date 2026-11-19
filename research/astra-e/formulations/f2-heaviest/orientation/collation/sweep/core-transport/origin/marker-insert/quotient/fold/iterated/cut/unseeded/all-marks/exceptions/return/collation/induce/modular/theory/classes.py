"""Exact variance gate on affine bands and a two-ordered-list interval composition."""
from collections import Counter
from itertools import product
from math import gcd
from pathlib import Path
import json
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[5]))
from clock import index
from trajectory import runs


def destinations(arcs,q):
    deltas=[[0,0,0] for _ in range(q+1)]
    for s,t,w,_ in arcs:
        d=(t-s)%q
        a=s%q
        y=a+min(q,w)
        ranges=((a,y),) if y<=q else ((a,q),(0,y-q))
        for lo,hi in ranges:
            for j,v in enumerate((1,d,d*d)):
                deltas[lo][j]+=v
                deltas[hi][j]-=v
    cnt=sd=sq=0;out=[]
    for r in range(q):
        c,z,zz=deltas[r];cnt+=c;sd+=z;sq+=zz
        out.append((r+sd//cnt)%q if cnt and cnt*sq==sd*sd else None)
    return out


def phase(arcs,q,r):
    out=[]
    for s,t,w,word in arcs:
        a=(r-s)%q
        if a<w:
            out.append(((s+a)//q,(t+a)//q,1+(w-1-a)//q,word))
    return out


def compose(left,right):
    first=sorted(range(len(left)),key=lambda i:left[i][1])
    second=sorted(range(len(right)),key=lambda i:right[i][0])
    i=j=at=0;out=[];src=[[] for _ in left];dst=[[] for _ in right]
    while i<len(first):
        a,b=first[i],second[j]
        x,y,w,v=left[a];l,t,h,u=right[b]
        end=min(y+w,l+h)
        assert y<=at<end and l<=at
        idx=len(out)
        src[a].append(idx);dst[b].append(idx)
        out.append((x+at-y,t+at-l,end-at,v+u))
        i+=end==y+w;j+=end==l+h;at=end
    assert j==len(second) and len(out)<=len(left)+len(right)-1
    # Each stable bucket's pieces are already in that arc's source and target order.
    top=[z for a in sorted(range(len(left)),key=lambda a:left[a][0]) for z in src[a]]
    bot=[z for b in sorted(range(len(right)),key=lambda b:right[b][1]) for z in dst[b]]
    assert top==sorted(range(len(out)),key=lambda i:out[i][0])
    assert bot==sorted(range(len(out)),key=lambda i:out[i][1])
    return [out[z] for z in top]


def check():
    cells=Counter()
    for alphabet,maximum in ((2,7),(3,4)):
        for n in range(1,maximum+1):
            for L in product(range(alphabet),repeat=n):
                F,psi,lf=index(L)
                arcs=[(s,t,w,bytes([v])) for s,t,w,v in runs(L)]
                for q in (2,3,4,8):
                    f=destinations(arcs,q)
                    expected=[]
                    for r in range(q):
                        edges={lf[x]%q for x in range(r,n,q)}
                        expected.append(next(iter(edges)) if len(edges)==1 else None)
                    assert f==expected
                    for start in range(min(q,n)):
                        r=start;orbit=[]
                        while r is not None and r not in orbit:
                            orbit.append(r);r=f[r]
                        cells['queried_groups']+=1
                        if r!=start:
                            continue
                        blocks=[phase(arcs,q,r) for r in orbit]
                        A=sum(map(len,blocks))
                        while len(blocks)>1:
                            blocks=[compose(blocks[i],blocks[i+1]) if i+1<len(blocks) else blocks[i]
                                    for i in range(0,len(blocks),2)]
                        answer=blocks[0]
                        assert len(answer)<=1+A-len(orbit)
                        for s,t,w,word in answer:
                            for off in range(w):
                                x=(s+off)*q+start
                                data=[]
                                while True:
                                    data.append(L[x]);x=lf[x]
                                    if x%q==start:break
                                assert x//q==t+off and bytes(data)==word and len(data)==len(orbit)
                                cells['physical_partner_words']+=1
                        cells['deterministic_cycles']+=1
                        cells['cycle_len_greater_than_one']+=len(orbit)>1
                        cells['balanced_three_or_more']+=len(orbit)>=3
                for g in (1,2,3,5,6):
                    for m in (2,3,4,7):
                        H=gcd(g,m)
                        for p in range(m):
                            for lane in range(g):
                                actual=[x for x in range(n) if (g*x+lane)%m==p]
                                eligible=(p-lane)%H==0
                                if eligible:
                                    reduced=m//H
                                    cls=((p-lane)//H*pow(g//H,-1,reduced))%reduced
                                    assert actual==list(range(cls,n,reduced))
                                else:
                                    assert not actual
                                cells['lift_lane_congruences']+=1
    return dict(cells)


if __name__=='__main__':print(json.dumps(check(),indent=2))
