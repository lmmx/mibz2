"""Charge affine discontinuities in first return to arbitrary surviving word ranks."""
from collections import Counter
from itertools import product,groupby
import json


def audit(column,live):
    n=len(column)
    order=sorted(range(n),key=column.__getitem__)
    owner=[column[i] for i in order]
    seams={x for x in range(n-1) if order[x]+1!=order[x+1] or owner[x]!=owner[x+1]}
    r=len(seams)+1
    assert r==sum(1 for _ in groupby(column))
    border={x for x in range(n-1) if (x in live)!=(x+1 in live)}
    components=sum(x-1 not in live for x in live)
    targets={}
    for x in live:
        y=order[x]
        while y not in live:y=order[y]
        targets[x]=y
    actual=components
    witnesses={}
    for x in sorted(live):
        if x+1 not in live:continue
        damaged=owner[x]!=owner[x+1] or targets[x]+1!=targets[x+1]
        if not damaged:continue
        actual+=1
        p=x
        while True:
            if p in seams:
                witness=('old',p)
                break
            p=order[p]
            if p in border:
                witness=('section',p)
                break
            assert p not in live
        assert witness not in witnesses
        witnesses[witness]=x
    bound=components+r-1+len(border)
    assert actual<=min(len(live),bound)
    return r,components,len(border),actual,bound


def main():
    stats=Counter()
    attained={}
    for n in range(1,9):
        for col in product((0,1),repeat=n):
            for mask in range(1,1<<n):
                live={x for x in range(n) if mask>>x&1}
                r,c,b,t,bound=audit(col,live)
                stats.update(cases=1,live=len(live),ribbons=t,charged=t-c)
                if b and t==bound and t<len(live):
                    if str(c) not in attained:
                        attained[str(c)]=dict(column=col,live=sorted(live),initial=r,
                                              components=c,border=b,ribbons=t)
    print(json.dumps(dict(counts=stats,attained=attained),indent=2))


if __name__=='__main__':main()
