"""Transform a compressed source column and recover all old return ranks and clock gaps."""
from collections import Counter
from itertools import product
import json,math
from clock import Clock,index,seq

def transform(L,s,G):
    return bytes(int(x==j) for j in range(s) for x in L if x<=j)+bytes(G*len(L))

def cyclic_bwt(w):
    order=sorted(range(len(w)),key=lambda i:w[i:]+w[:i])
    return bytes(w[i-1] for i in order),order.index(0)

def verify(L,s,k,G):
    old=bytes(L);m=len(old);T=transform(old,s,G);n=len(T)
    F,psi,lf=index(old);c=Clock(T,k)
    assert all(a==0 for a in c.modal)
    B=n-m;expected_post=c.words.index(bytes([1])+bytes(k-1)) if k else None
    returns=events=0
    for x in range(m):
        z=B+x;steps=G+s-old[x]  # g(old[x])+1
        for t in range(steps):
            assert (T[z]==1)==(t==steps-1)
            z=c.lf[z]
        assert z==B+lf[x];returns+=1
        # Full marked backward lap. Each event identifies a preceding source byte.
        length,packed=c.lap(B+x);last=-1;recovered=[]
        for t,post in packed[1]:
            assert post==(expected_post if k else 1)
            recovered.append(G+s-(t-last));last=t
        assert last==length-1 and recovered
        y=x
        for a in recovered:assert old[y]==a;y=lf[y]
        assert y==x
        orig=bytes(reversed(recovered))
        assert (orig*((m+len(orig)-1)//len(orig)))[:m]==seq(F,psi,x,m)
        events+=len(packed[1])
    return returns,events

def run():
    totals=Counter()
    for s,limit in ((2,7),(3,4)):
        for n in range(1,limit+1):
            for L in product(range(s),repeat=n):
                for k in (0,1,3):
                    a,e=verify(L,s,k,k+2);totals['lifted_tables']+=1;totals['exact_b_returns']+=a;totals['marked_clock_events']+=e
                # Independent true-BWT construction on the represented marked word.
                w=bytes(L);raw,p=cyclic_bwt(w);k=2;G=4
                encoded=b''.join(bytes([1])+bytes(G+s-1-c) for c in w)
                col,z=cyclic_bwt(encoded)
                assert col==transform(raw,s,G)
                # Equal-rotation tie lanes can use another rank. Both spell the same marked bytes.
                F,psi,_=index(col);b=len(col)-len(w)+p
                assert seq(F,psi,b,len(encoded))==encoded==seq(F,psi,z,len(encoded))
                totals['independent_encoded_bwts']+=1
    examples=[]
    for m,zeros in [(5,2),(7,3),(11,5),(13,6)]:
        s=2;k=2;G=16;ones=m-zeros;schedules={};common=None
        for w in product(range(2),repeat=m):
            if w.count(0)!=zeros:continue
            w=bytes(w);canonical=min(w[i:]+w[:i] for i in range(m))
            if canonical!=w:continue
            L,_=cyclic_bwt(w);t=transform(L,s,G);c=Clock(t,k);mark=len(t)-m
            state,ev=c.encode_from_end(mark,len(t))
            spec=Counter(seq(c.F,c.psi,r,k+1)for r in range(len(t)))
            geometry=(len(t),mark,state,tuple(c.words),tuple(c.modal),tuple(c.default),tuple(x for _,x in ev),tuple(sorted(spec.items())))
            if common is None:common=geometry
            assert geometry==common
            times=tuple(x for x,_ in ev);assert times not in schedules;schedules[times]=w
        count=math.comb(m,zeros)//m;assert len(schedules)==count
        examples.append(dict(m=m,zeros=zeros,k=k,G=G,expanded_rows=common[0],same_table_and_spectrum=True,different_clocks=count))
    print(json.dumps(dict(totals=totals,entropy_witnesses=examples),indent=2))
if __name__=='__main__':run()
