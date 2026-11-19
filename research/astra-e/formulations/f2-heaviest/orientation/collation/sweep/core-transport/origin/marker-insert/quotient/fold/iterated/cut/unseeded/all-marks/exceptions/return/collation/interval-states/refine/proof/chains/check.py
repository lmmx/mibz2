"""Compare suffix/necklace choices to all physical partitions and all pair histograms."""
from collections import Counter
from itertools import combinations,groupby,product
from pathlib import Path
import json
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[6]))
from clock import index
from geometry import Geometry,Choices


def cuts(mask,n):
    return tuple(z for z in range(1,n) if mask>>(z-1)&1)


def powerset(z):
    return (s for k in range(len(z)+1) for s in combinations(z,k))


def necklace_oracle(psi,F):
    n=len(F)
    words=[]
    for x in range(n):
        word=[]
        for _ in range(2*n):
            word.append(F[x]);x=psi[x]
        words.append(tuple(word))
    assert words==sorted(words)
    return {i for i in range(1,n) if words[i]!=words[i-1]}


def run():
    count=Counter()
    max_necklaces=max_cycle=0
    for alphabet,maximum in ((2,7),(3,4)):
        for n in range(1,maximum+1):
            for L in product(range(alphabet),repeat=n):
                runs=[(a,len(list(rows))) for a,rows in groupby(L)]
                geo=Geometry(runs)
                F,psi,lf=index(L)
                assert [geo.psi(i) for i in range(n)]==psi
                assert [geo.lf(i) for i in range(n)]==lf
                base=geo.paths(set())
                group=geo.fibers()
                assert set(x for p in base for x in p)==necklace_oracle(psi,F)
                assert len(base)==len(runs)-1 and len(group)<=len(runs)
                max_necklaces=max(max_necklaces,len(group))
                cycle=sum(g-1 for _,g in group)
                max_cycle=max(max_cycle,cycle)
                # Recover every path length and endpoint independently by an original LF first return.
                D=set(geo.src[:-1]);back={p[0]:p[::-1] for p in base}
                first_returns=0
                for old_source in D:
                    x=lf[old_source];p=[x]
                    while x not in D:
                        x=lf[x];p.append(x)
                    if p[0]>0:
                        assert p==back[x]
                    else:
                        assert p==[0]
                    first_returns+=len(p)
                assert first_returns==sum(len(orbit) for orbit,_ in group)
                count['columns']+=1
                count['occurrence_cycles']+=cycle
                if n>1 and cycle:count['columns_with_pure_mandatory_cycles']+=1
                for old in powerset(tuple(range(1,n))):
                    oldset=set(old)
                    choices=Choices(geo,old,group)
                    q0=set(choices.enumerate(False))
                    qc=set(choices.enumerate(True))
                    assert qc<=q0
                    raw=set()
                    for c in powerset(tuple(x for x in range(1,n) if x not in oldset)):
                        q=tuple(sorted(old+tuple(c)))
                        allowed=set(q)
                        # Rowwise check: no old label in any new source can land in two new targets.
                        state=lambda x,st:sum(t<=x for t in st)
                        images={}
                        ok=True
                        for x,a in enumerate(L):
                            key=(state(x,q),a,state(lf[x],old))
                            v=state(lf[x],q)
                            if images.setdefault(key,v)!=v:
                                ok=False;break
                        if ok: raw.add(q)
                    assert q0==raw
                    score={q:geo.scores(q,q) for q in q0}
                    assert all(score[q]==geo.scores(q,old) for q in q0)
                    for mul,charge in ((1,0),(2,1),(1,1),(1,3)):
                        value=lambda q:mul*score[q]-charge*(len(q)-len(old))
                        assert max(value(q) for q in q0)==max(value(q) for q in qc)
                        count['canonical_optima']+=1
                    count['old_partitions']+=1
                    count['all_neutral']+=len(q0)
                    count['canonical_candidates']+=len(qc)
                    count['strictly_reduced_partitions']+=qc!=q0
    # Width-independent handling of a thick cover, including internal inherited cuts.
    w=10**12+17
    seed=[1,0,1,1,0,0,0,0,0]
    runs=[(v,len(list(xs))*w) for v,xs in groupby(seed)]
    geo=Geometry(runs);necklaces=geo.fibers()
    old=geo.fseams|{2*w+w//3}
    c=Choices(geo,old,necklaces)
    count_big=0;best=-1
    for q in c.enumerate(True):
        best=max(best,geo.scores(q,q)-(len(q)-len(old)))
        count_big+=1
    return dict(counts=dict(count),max_distinct_necklaces=max_necklaces,max_pure_cycles=max_cycle,
                thick_cover=dict(n=geo.n,run_count=len(runs),necklaces=len(necklaces),multiplicity=w,
                  stored_path_rows=sum(map(len,c.paths)),offset_slabs=len(c.slabs),old=sorted(old),
                  tested_canonical=count_big,maximum_unit_charge_objective=best))


if __name__=='__main__':print(json.dumps(run(),indent=2))
