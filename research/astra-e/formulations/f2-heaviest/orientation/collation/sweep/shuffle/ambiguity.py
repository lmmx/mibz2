"""Search genuine cyclic last columns for indistinguishable unlabelled transcripts after dropping masks."""
from collections import Counter,defaultdict
from itertools import product,accumulate
import json
from census import labels,Oracle,follower_classes
from interface import geometry,shuffle


def main():
    witnesses={}
    tests=Counter()
    for n in range(1,11):
        seen=defaultdict(dict)
        for digits in product(range(2),repeat=n):
            last=bytes(digits);o=Oracle(last)
            for a in range(n):
                for b in range(a+1,n+1):
                    if o.first[a]!=o.first[b-1]:continue
                    words,counts,src,inc=labels(o,(a,b))
                    cls,after,_=follower_classes(words,src,inc)
                    starts=list(accumulate([0]+counts[:-1]));tau=[0]*len(src)
                    for j,tid in enumerate(inc):tau[starts[tid]]=j;starts[tid]+=1
                    for choice in product(range(2),repeat=max(cls)+1):
                        bits=[choice[i] for i in cls]
                        transformed=[w[bits[i]:]+w[:choice[after[i]]] for i,w in enumerate(words)]
                        if not all(transformed) or len(set(transformed))!=len(words):continue
                        sm=tuple(bits[i] for i in src)
                        positions,tm=geometry(o.forward,(a,b),sm)
                        sigma=shuffle(sm,tm);physical=[None]*len(sm)
                        for j,x in enumerate(sigma):physical[x]=transformed[inc[j]]
                        physical=tuple(physical)
                        tests[n]+=1
                        for keep in ["S","T","neither"]:
                            if keep in witnesses:continue
                            key=(a,b,tuple(words),tuple(counts),physical,sm if keep=="S" else tuple(tm) if keep=="T" else ())
                            prior=seen[keep].get(key)
                            if prior and prior[0]!=tau:
                                def example(t,col,s,tmask):
                                    return {"L":list(col),"tau":t,"S":list(s),"T":list(tmask)}
                                witnesses[keep]=dict(n=n,U=[a,b],ordered_words=[list(w) for w in words],
                                                      counts=counts,physical_in=[list(w) for w in physical],
                                                      examples=[example(*prior),example(tau,last,sm,tm)])
                            if prior is None:seen[keep][key]=(tau,last,sm,tm)
        if len(witnesses)==3:break
    print(json.dumps({"eligible_by_n":tests,"witnesses_by_retained_mask":witnesses},indent=2))


if __name__=="__main__":
    main()
