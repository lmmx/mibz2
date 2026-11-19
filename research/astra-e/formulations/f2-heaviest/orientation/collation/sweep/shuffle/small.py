"""Exhaust arbitrary original columns and occurrence moves on monochromatic sections."""
from collections import Counter
from itertools import product
import json
from census import Oracle,labels
from interface import shuffle,geometry,reconstruct,incoming_by_lf,emit


def main():
    counters=Counter()
    witnesses={}
    omitted={}
    for n in range(1,8):
        for z in product(range(2),repeat=n):
            last=bytes(z);oracle=Oracle(last)
            lf=[0]*n
            for i,x in enumerate(oracle.forward):lf[x]=i
            expected=[]
            for p in range(n):
                row=p;text=bytearray()
                for _ in range(n):
                    text.append(oracle.first[row]);row=oracle.forward[row]
                expected.append(bytes(text))
            for a in range(n):
                for b in range(a+1,n+1):
                    if oracle.first[a]!=oracle.first[b-1]:continue
                    sec=a,b
                    words,counts,src,inc=labels(oracle,sec)
                    m=b-a
                    old=[None]*m
                    for j in range(m):
                        row=oracle.forward[a+j]
                        while not a<=row<b:row=oracle.forward[row]
                        old[j]=row-a
                    for mask in product(range(2),repeat=m):
                        g=geometry(oracle.forward,sec,mask)
                        if g is None:
                            counters["collisions"]+=1
                            continue
                        physical,t=g
                        sigma=shuffle(mask,t)
                        new,_=incoming_by_lf(physical,last,lf)
                        sol=reconstruct(words,counts,mask,t,new,oracle.first[a:a+1])
                        assert old==sol
                        for mark in range(n):
                            assert emit(oracle,sec,mark,src,words,sol)==expected[mark]
                        counters["injective_occurrence_assignments"]+=1
                        counters["marks"]+=n
                        counters["nontrivial_shuffles"]+=sigma!=list(range(m))
                        typewise=all(len({mask[i] for i in range(m) if src[i]==k})==1 for k in range(len(words)))
                        uniform=all(len({mask[j] for j in range(m) if inc[j]==k})==1 for k in range(len(words)))
                        transformed=[new[sigma[old[i]]] for i in range(m)]
                        one_name=len(set(zip(src,transformed)))==len(words)
                        distinct=one_name and len(set(transformed))==len(words)
                        counters["original_restrictions_eligible"]+=typewise and uniform and distinct
                        for tag,flag in [("context_dependent",not one_name),("colliding_names",one_name and not distinct)]:
                            if flag and tag not in witnesses:
                                witnesses[tag]={"L":last.hex(),"U":sec,"S":mask,"T":t,
                                    "old_words":[w.hex() for w in words],"old_counts":counts,
                                    "physical_in":[w.hex() for w in new],"tau":sol}
                        # Seek a genuine observed transcript ambiguity, rather than
                        # treating all abstract binary shuffles as BWT-realizable.
                        if typewise and uniform and distinct:
                            for missing in ["physical mask","source mask","both masks"]:
                                if missing in witnesses:continue
                                key=(missing,n,sec,tuple(words),tuple(counts),tuple(new),
                                     tuple(mask) if missing=="physical mask" else (),
                                     tuple(t) if missing=="source mask" else ())
                                example={"L":last.hex(),"S":list(mask),"T":t,"tau":sol}
                                if key in omitted and omitted[key]["tau"]!=sol:
                                    witnesses[missing]={"U":sec,"words":[w.hex() for w in words],
                                        "counts":counts,"physical_in":[w.hex() for w in new],
                                        "examples":[omitted[key],example]}
                                else: omitted[key]=example
    print(json.dumps({"counts":counters,"first_witnesses":witnesses},indent=2))


if __name__=="__main__":
    main()
