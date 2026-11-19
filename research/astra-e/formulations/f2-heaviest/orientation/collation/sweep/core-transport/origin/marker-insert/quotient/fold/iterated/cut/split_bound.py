"""Exhaust all one-cut contiguous alias repairs of the protected long self-chain."""
from collections import Counter
from itertools import combinations, product
import json


def main():
    counts=Counter()
    for m in range(2,13):
        for t in range(2,m+1):
            # Last row is the protected b singleton.
            for internal in combinations(range(1,m-1),t-2):
                cuts=(0,*internal,m-1,m)
                best=0
                for colors in product((0,1),repeat=t-1):
                    ring=(*colors,0)
                    rows=[color for color,l,r in zip(ring,cuts[:-1],cuts[1:]) for _ in range(r-l)]
                    entering=sum(not rows[i] and rows[(i+1)%m] for i in range(m))
                    assert entering==sum(not ring[i] and ring[(i+1)%t] for i in range(t))
                    assert 2*entering<=t
                    best=max(best,entering)
                    counts['colorings']+=1
                assert best==t//2
                counts['partitions']+=1
    print(json.dumps(counts,indent=2))


if __name__=='__main__':
    main()
