"""Check that atom itineraries fix the physical final partition across conjugations."""
from collections import Counter
from itertools import product
import json
from pathlib import Path
import random
import sys
from finite import induce,layout

policies=("stable","reflected","short_first","long_first","reverse_spans")
join_check=dict(layouts=0,joins=0,distinct_totals=Counter());counts=Counter();witness={};deltas={p:Counter() for p in policies}
def check(last,kept):
    reference=None;cost=0
    for policy in policies:
        order,sign=layout(last,kept,policy)
        borders,steps=induce(last,kept,order,sign,check_cylinders=True,join_check=join_check)
        if policy=="stable":
            reference=borders;cost=steps
        assert borders==reference
        counts["layouts"]+=1;counts["kept_adjacencies"]+=len(borders)
        label="fewer" if steps<cost else "more" if steps>cost else "equal"
        deltas[policy][label]+=1
        if steps!=cost and (policy,label) not in witness:
            witness[policy,label]=dict(last=last,kept=sorted(kept),baseline=cost,changed=steps)
    counts["sections"]+=1

for alphabet,top in [(2,6),(3,4)]:
    for n in range(1,top+1):
        for last in product(range(alphabet),repeat=n):
            for mask in range(1,1<<n):
                check(last,{i for i in range(n) if mask>>i&1})
rng=random.Random(710549)
for _ in range(6000):
    n=rng.randint(7,21);alphabet=rng.choice([2,3,4]);last=tuple(rng.randrange(alphabet) for j in range(n))
    freq=rng.choice([.1,.25,.5,.75]);v={j for j in range(n) if rng.random()<freq};v.add(rng.randrange(n))
    check(last,v)
p=Path(sys.argv[1]);p.write_text(json.dumps(dict(counts=counts,joins=join_check,step_directions=deltas,
                       witnesses={a+" "+b:v for (a,b),v in witness.items()}),indent=2)+"\n")
