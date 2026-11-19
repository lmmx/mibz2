"""A spelling-only lower bound for arbitrary independent one-byte cut movements."""
from collections import Counter
from itertools import product
from pathlib import Path
import csv
import sys

from optimize import Column, Oracle, heavy_set, here, objects, suffixes


def bound(words):
    c=words[0][:1]
    assert all(w.startswith(c) for w in words)
    tails=[w[1:] for w in words]
    q0=suffixes(words)
    q1=suffixes([v+c for v in tails])
    if not any(tails):
        return (1, q1, 0, 0)
    vertices={v[i:] for v in tails for i in range(1,len(v))}
    # Each edge (u,u+c) must be covered. Right-append c has unique successor
    # and predecessor; every connected component is a path. c must be in the cover.
    roots=sorted(u for u in vertices if not u.endswith(c) or u[:-1] not in vertices)
    pinned=1
    edges=0
    for x in roots:
        state={0:0,1:1} if x!=c else {1:0}
        while x in vertices:
            edges+=1
            x+=c
            # c cannot be the right end of an edge from a nonempty word.
            state={0:state[1],1:1+min(state.values())}
        pinned+=min(state.values())
    assert edges==len(vertices)
    lower=min(q0,1+pinned)
    return lower,q1,len(vertices),sum(s.endswith(c) for s in vertices)


def small():
    stat=Counter(); witness=None
    for n in range(1,8):
        for col in product(range(2),repeat=n):
            obj=Oracle(bytes(col))
            for a in range(n):
                for b in range(a+1,n+1):
                    if obj.first[a]!=obj.first[b-1]:
                        continue
                    words,src,ret=objects(obj,(a,b));c=words[0][:1]
                    lower,full,_,collisions=bound(words)
                    best=full
                    for bits in product((0,1),repeat=b-a):
                        altered=[words[t][bits[j]:]+c*bits[ret[j]] for j,t in enumerate(src)]
                        if b'' in altered:
                            continue
                        q=suffixes(altered)
                        assert q>=lower
                        if not collisions:
                            assert q>=full
                        best=min(best,q)
                        stat['positive_cut_assignments']+=1
                    stat['sections']+=1
                    stat['bound_exact']+=lower==best
                    stat['uniform_exact']+=full==best
                    if best<full and witness is None:
                        witness=dict(column=bytes(col).hex(),section=[a,b],uniform=full,optimum=best,lower=lower)
    import json
    print(json.dumps(dict(counts=stat,mixed_advantage=witness),indent=2))


def real(tapes):
    sys.path.insert(0,str(here.parents[7]/'docs/journal/2026-11-18-astra-c-output-first'))
    from inputs import read_input,paths
    wr=csv.writer(sys.stdout,lineterminator='\n')
    wr.writerow(('ordinal','section','types','mass','lower_any_occurrence','full_uniform','proper_tail_suffixes','ending_in_cut_byte'))
    for tape in paths(tapes):
        runs,_=read_input(tape)
        col=Column(runs);oracle=Oracle(b''.join(bytes([c])*n for c,n in runs))
        quote=col.interval(b'"');deep=heavy_set(col,quote,1)[0][0]
        for name,s in (('quote',quote),('frequent',deep)):
            words,_,_=objects(oracle,s)
            wr.writerow((int(tape.name.split('.')[0]),name,len(words),s[1]-s[0],*bound(words)))
            sys.stdout.flush()


if __name__=='__main__':
    real(Path(sys.argv[1])) if len(sys.argv)>1 else small()
