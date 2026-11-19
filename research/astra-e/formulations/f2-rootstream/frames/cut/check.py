"""Check framed hybrid objective by interpreting each legal small cut."""
from collections import Counter
from fractions import Fraction
from itertools import product
import json
from refine import Framed,Model,local_counts
from data import statistics
from tree import encode,from_root,materialize,routes

def play(tapes,full,m,cut):
    if 0 in cut:
        return encode(full[0]),0
    stream=[encode(a) if v in cut else t for v,(a,t) in enumerate(zip(full,tapes))]
    index=[0]*m.n; used=[0]*m.n; events=0; result=[]
    def take(v,n):
        nonlocal events
        while n:
            events+=1
            child,width=stream[v][index[v]]
            k=min(n,width-used[v])
            n-=k;used[v]+=k
            if used[v]==width:
                index[v]+=1;used[v]=0
            if v in cut or v==child or not m.children[child]:
                result.append((child,k))
            else:
                take(child,k)
    take(0,len(full[0]))
    return result,events

def test(labels,parent,stats):
    paths=routes(parent);tapes=from_root(labels,paths)
    if any(not t for t in tapes): return
    full=materialize(tapes)
    keys=["node","parent","depth","S","R","G","incoming_pieces","terminal_width"]
    m=Framed(Model([dict(zip(keys,r)) for r in statistics(labels,parent)]),*local_counts(labels,paths))
    options=[]
    stats["tapes"]+=1
    for bits in product([False,True],repeat=m.n):
        cut={v for v,x in enumerate(bits) if x}
        if any(v and not m.children[v] for v in cut) or any(sum(v in cut for v in p)>1 for p in paths):
            continue
        result,events=play(tapes,full,m,cut)
        assert result==encode(labels)
        built=sum(m.sub[v] for v in cut)
        mem=built-(m.r[0] if 0 in cut else 0)
        expected=sum(m.g[v] for v,p in enumerate(paths) if not any(u in cut for u in p))
        expected+=sum(m.lookup[v] for v in cut if v!=0)
        assert events==expected
        options.append((mem,events+built))
        stats["decoded_cuts"]+=1
    for x in (Fraction(0),Fraction(1,4),Fraction(1),Fraction(2),Fraction(8)):
        assert m.optimize(x)==min(options,key=lambda z:(z[1]+x*z[0],z[0]))
        stats["optima"]+=1

stats=Counter()
for n in range(1,5):
    for tail in product(*(range(i) for i in range(1,n))):
        for m in range(1,5):
            for labels in product(range(n),repeat=m):
                test(labels,(-1,)+tail,stats)
print(json.dumps(stats,indent=2))
