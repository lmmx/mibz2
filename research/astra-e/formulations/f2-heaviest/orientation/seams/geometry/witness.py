"""Look for a small real cyclic source where permitting different seams for equal words helps."""
from collections import Counter
from itertools import groupby, product
from random import Random
import json

from optimize import Column, events, optimize
from packets import traces


def examine(tokens):
    text=b"".join(tokens)
    order=sorted(range(len(text)),key=lambda i:text[i:]+text[:i])
    last=bytes(text[i-1] for i in order)
    col=Column([(c,sum(1 for _ in g)) for c,g in groupby(last)])
    sect=[col.interval(b"\0")]
    words,_,inc,_=events(col,sect)
    first,_,_,_,_=optimize(words,inc)
    splits=traces(col,sect)
    second,seams,_,_,_=optimize([w for w,trace in splits],[[(r,) for r in trace] for w,trace in splits])
    if second>=first:
        return None
    return dict(text=text.hex(),last=last.hex(),primary=order.index(0),word_optimum=first,
                trace_optimum=second,trace_seams=[(w.hex(),trace,seam) for (w,trace),seam in zip(splits,seams)])


rng=Random(87031)
pool=[b"\0"+bytes(w) for n in (1,2,3,4) for w in product((1,2,3),repeat=n)]
cases=0
for _ in range(4500):
    family=rng.sample(pool,rng.randrange(4,15))
    a=family+rng.choices(family,k=rng.randrange(5,34))
    rng.shuffle(a)
    result=examine(a)
    cases+=1
    if result:
        # Minimize the number of occurrences by deleting one token whenever strict gain survives.
        at=0
        while at<len(a) and len(a)>2:
            shorter=a[:at]+a[at+1:]
            win=examine(shorter)
            if win:
                a,result=shorter,win
                at=0
            else:
                at+=1
        break
print(json.dumps(dict(trials=cases,witness=result),indent=2))
