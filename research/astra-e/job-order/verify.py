"""Brute force all 720 static orders per instance, including every non-delay dispatch."""
import itertools
import random
from model import Jobs,bounds,fifo_independent,simulate

rng=random.Random(318232)
n=6;records=0;hit=0;improves=0;worsens=0;largest=(0,None)
for trial in range(32):
    a=Jobs([rng.randrange(1,4) for _ in range(n)],[rng.randrange(1,12) for _ in range(n)],
           [rng.randrange(1,4) for _ in range(n)],list(range(n)))
    w=2;cap=3+trial%3
    floor=bounds(a,w,cap);fifo=simulate(a,w,cap,'fifo')[0]
    assert fifo==fifo_independent(a,w,cap)
    best=fifo
    for perm in itertools.permutations(range(n)):
        ranks={v:i for i,v in enumerate(perm)}
        t=simulate(a,w,cap,lambda q:min(q,key=ranks.get))[0]
        assert t>=floor
        improves+=t<fifo;worsens+=t>fifo;best=min(best,t);records+=1
    hit+=floor==best
    if fifo-best>largest[0]: largest=(fifo-best,(a,cap,floor,fifo,best))
print('static_priority_schedules',records,'samples',32,'exact_bound',hit,'improve',improves,'regress',worsens)
print('largest_FIFO_gap',largest)
