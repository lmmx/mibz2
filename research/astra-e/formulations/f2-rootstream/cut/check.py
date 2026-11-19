"""Check every cut of small selector trees by decoding with the chosen cut."""
from bisect import bisect_right
from collections import Counter
from fractions import Fraction
from itertools import accumulate, product
import json
from data import statistics
from frontier import Model
from fringe import choose
from policy import policy
from tree import encode,from_root,materialize,routes

def execute(tapes, projected, stops):
    # Interpret selected-subtree roots as pre-expanded terminal streams.
    # Source ids equal node ids, so use distinct tags for emitted terminals.
    choices,ends = [],[]
    for v,t in enumerate(tapes):
        pieces = [(True,id,w) for id,w in encode(projected[v])] if v in stops else [
            (id==v,id,w) for id,w in t]
        choices.append(pieces)
        ends.append(list(accumulate(p[2] for p in pieces)))
    count = Counter()
    if 0 in stops:
        return encode(projected[0]),count
    used = [0]*len(ends)
    out = []
    while used[0]<ends[0][-1]:
        width = len(projected[0])
        path = []
        v = 0
        while True:
            path.append(v)
            k = bisect_right(ends[v],used[v])
            width = min(width,ends[v][k]-used[v])
            is_term,child,_ = choices[v][k]
            count["cut" if v in stops else "route"] += 1
            if is_term:
                out.append((child,width))
                break
            v = child
        for u in path:
            used[u] += width
    return out,count

def check(labels,parent,stats):
    paths = routes(parent)
    tapes = from_root(labels,paths)
    if any(not t for t in tapes): return
    full = materialize(tapes)
    keys = ["node","parent","depth","S","R","G","incoming_pieces","terminal_width"]
    model = Model([dict(zip(keys,row)) for row in statistics(labels,parent)])
    plans = []
    stats["tree_tapes"] += 1
    for mask in product([False,True],repeat=model.n):
        stops = {u for u,z in enumerate(mask) if z}
        if any(sum(i in stops for i in path)>1 for path in paths):
            continue
        a,b,c,m,_ = model.count(stops)
        expected_writes = sum(len(encode(full[v])) for v,path in enumerate(paths)
                              if any(i in stops for i in path))
        assert c == expected_writes
        output,count = execute(tapes,full,stops)
        assert output == encode(labels)
        assert (a,b) == (count["route"],count["cut"])
        plans.append((m,a+b+c))
        stats["literal_mixed_decodes"] += 1
    for penalty in (Fraction(0),Fraction(1,4),Fraction(1),Fraction(2),Fraction(8)):
        point = min(plans,key=lambda a:(a[1]+penalty*a[0],a[0]))
        assert model.optimize(penalty) == point
        stops,cert = policy(model,penalty)
        a,b,c,m,_ = model.count(stops)
        assert sum(model.g)-sum((a,b,c))-penalty*m >= cert
        stops,cert = choose(model,penalty)
        a,b,c,m,_ = model.count(stops)
        assert sum(model.g)-sum((a,b,c)) >= cert
        stats["optima_and_certificates"] += 1

stats = Counter()
for n in range(1,5):
    for tail in product(*(range(i) for i in range(1,n))):
        for k in range(1,5):
            for labels in product(range(n),repeat=k):
                check(labels,(-1,)+tail,stats)
print(json.dumps(stats,indent=2))
