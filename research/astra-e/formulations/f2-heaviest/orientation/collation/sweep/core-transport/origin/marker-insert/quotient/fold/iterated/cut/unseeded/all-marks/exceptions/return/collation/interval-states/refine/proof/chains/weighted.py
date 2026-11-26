"""Normalize arbitrary lane choices on genuinely periodic, possibly heterogeneous columns."""
from collections import Counter
from itertools import groupby
from pathlib import Path
import random
import sys
import json
sys.path.insert(0,str(Path(__file__).resolve().parents[6]))
from clock import index
from geometry import Choices,Geometry


def neutral(geo,old,chosen):
    for z in chosen-old:
        s=geo.support(z)
        if s and not any(t in chosen for t in range(s[0],s[1]+1)):
            return False
    return True


def check():
    rng=random.Random(3071)
    result=Counter()
    for _ in range(1800):
        length=rng.randrange(2,14)
        col=[rng.randrange(2+rng.randrange(3)) for _ in range(length)]
        _,_,lf=index(col)
        width=[0]*length
        for x in range(length):
            weight=rng.randrange(1,9)
            while width[x]==0:
                width[x]=weight
                x=lf[x]
        L=[a for a,g in zip(col,width) for _ in range(g)]
        geo=Geometry([(a,len(list(xs))) for a,xs in groupby(L)])
        old={z for z in range(1,geo.n) if rng.random()<0.04}
        chosen=old|{z for z in range(1,geo.n) if rng.random()<0.08}
        for _ in range(geo.n):
            missing=[(l,r) for z in chosen-old if (s:=geo.support(z))
                     for l,r in (s,) if not any(t in chosen for t in range(l,r+1))]
            if not missing:
                break
            for l,r in missing:
                chosen.add(rng.randrange(l,r+1))
        assert neutral(geo,old,chosen)
        c=Choices(geo,old,geo.fibers())
        saved=initial=len(chosen)
        base=score=geo.scores(chosen,chosen)
        for group,a,b in c.slabs:
            current=[j for j in range(a,b+1) if group[0]+j in chosen]
            assert all((x+j in chosen)==(j in current) for x in group for j in range(a,b+1))
            if not current:
                continue
            stripped=chosen-{x+j for x in group for j in current}
            candidates=((a,),(b,)) if len(current)==1 else ((a,b),)
            tests=[stripped|{x+j for x in group for j in idx} for idx in candidates]
            assert all(neutral(geo,old,q) and old<=q for q in tests)
            vals=[geo.scores(q,q) for q in tests]
            v=max(vals)
            chosen=tests[vals.index(v)]
            assert v>=score and len(chosen)<=saved and v==geo.scores(chosen,old)
            result['strict_step_score_gains']+=v>score
            result['strict_step_cut_reductions']+=len(chosen)<saved
            result['positive_slabs']+=1
            score,saved=v,len(chosen)
        result['positive_total_gains']+=score>base
        result['positive_total_cut_savings']+=len(chosen)<initial
        result['heterogeneous']+=len(geo.fibers())>1
        result['models']+=1
    return result


if __name__=='__main__':print(json.dumps(check(),indent=2))
