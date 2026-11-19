from collections import Counter
from itertools import product
import json
from math import gcd
from numeric import components,index,reverse,walk,width_quotient

def bwt(word):
    rotations=sorted((word[i:]+word[:i],i)for i in range(len(word)))
    return tuple(word[i-1] for _,i in rotations),next(j for j,(_,i)in enumerate(rotations)if i==0)

def main():
    total=Counter();family=Counter();witnesses={}
    for sigma,max_n in ((2,12),(3,7)):
        for n in range(1,max_n+1):
            for L in product(range(sigma),repeat=n):
                f,psi=index(L);g,Q=width_quotient(L);d=n//g;fq,pq=index(Q)
                c=components(f,psi);counts=Counter(c)
                assert all(len(w)==len(set(w[i:]+w[:i]for i in range(len(w)))) for w in c)
                assert g==gcd(*counts.values()) and (len(set(c))==1)==(len(components(fq,pq))==1)
                row,tapes,s=reverse(L);total.update(s);total["columns"]+=1;total["marks"]+=n
                if len(c)>1:total["multi_cycle_columns"]+=1
                if len(counts)>1:total["heterogeneous_columns"]+=1
                if g>1:total["repeated_lane_columns"]+=1
                for r in range(n):
                    q,ell=divmod(r,g);assert psi[r]==g*pq[q]+ell
                    want=walk(f,psi,r,n);assert want==walk(fq,pq,q,n)
                    owner,off=row[r];ring=tapes[owner];out=tuple(ring[(off+t)%len(ring)]for t in range(n));assert out==want
                    if g>1 and walk(fq,pq,q,d)*g!=want and 'unchecked_repeat' not in witnesses:
                        witnesses['unchecked_repeat']=dict(last=L,mark=r,g=g,quotient=Q,wrong=walk(fq,pq,q,d)*g,correct=want)
                orig,primary=bwt(L);g2,squeezed=width_quotient(orig)
                period=next(h for h in range(1,n+1) if n%h==0 and L==L[:h]*(n//h))
                assert g2==n//period
                ff,pp=index(squeezed);assert len(components(ff,pp))==1 and walk(ff,pp,primary//g2,n)==L
                family["valid_input_words"]+=1
    for k in range(2,90):
        n=2*k+1;L=(1,)*k+(0,)*(k+1)
        F,p=index(L);assert all(y==(x+k)%n for x,y in enumerate(p))
        pos=[-2*x%n for x in range(n)];assert all((pos[y]-pos[x])%n==1 for x,y in enumerate(p))
        assert n==1+sum((b-a)%n!=1 for a,b in zip(pos,pos[1:]));family["two_run_sharp_cases"]+=1
    # Mixed labels under one common translation; byte words may differ by lane.
    for n in range(2,22):
        for cut in range(1,n):
            for split in range(1,n):
                F=tuple(0 if i<split else (1 if i<max(cut,split) else 2) for i in range(n))
                if F[n-cut-1]==F[n-cut]:continue
                L=F[n-cut:]+F[:n-cut];f,p=index(L)
                assert all(y==(x+cut)%n for x,y in enumerate(p));h=gcd(cut,n);d=n//h
                inv=pow(cut//h,-1,d)
                for x,y in enumerate(p):assert x%h==y%h and ((y//h-x//h)*inv)%d==1%d
                family["modular_rotation_columns"]+=1
                if h>1 and len(set(components(f,p)))>1 and 'arithmetic_heterogeneous' not in witnesses:
                    witnesses['arithmetic_heterogeneous']=dict(last=L,shift=cut,cycles=components(f,p))
    print(json.dumps({"exhaustive":total,"families":family,"witnesses":witnesses},indent=2))
if __name__=="__main__":main()
