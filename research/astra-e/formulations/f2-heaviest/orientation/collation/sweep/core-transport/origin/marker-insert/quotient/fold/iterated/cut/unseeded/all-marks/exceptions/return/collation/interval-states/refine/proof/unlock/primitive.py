"""Exact positive unit-charge optima for a one-row perturbation of the cyclic cover."""
from bisect import bisect_right
from collections import Counter
from itertools import accumulate
import json

from batches import graph_width, subsets


def example(k, delta, scale=None, exhaustive=False):
    g = k if scale is None else scale
    n = 3*k*g+delta
    bound = n-3*g
    last = '1'*g+'0'*g+'1'*(2*g)+'0'*(n-4*g)
    psi = sorted(range(n), key=last.__getitem__)
    lf = [0]*n
    for i,y in enumerate(psi):
        lf[y] = i
    lap = [0]
    while psi[lap[-1]] != 0:
        lap.append(psi[lap[-1]])
    from math import gcd
    assert (len(lap)==n) == (gcd(3*g, g-delta)==1)
    if len(lap)!=n:
        return {'k':k,'scale':g,'delta':delta,'primitive':False}
    special = {g,bound+g}
    stops = special|{0,bound}
    paths = {}
    for endpoint in stops:
        x = lf[endpoint]
        backwards = []
        while x not in stops:
            backwards.append(x)
            x = lf[x]
        paths[endpoint] = backwards
    assert not paths[0] and not paths[g]
    support = {t:set(range(psi[t-1]+1,psi[t]+1)) for t in range(1,n) if t!=bound}
    assert {t for t,s in support.items() if len(s)!=1} == special
    ones = list(accumulate((c=='1' for c in last), initial=0))
    def mass(selected):
        b = [0,*sorted(selected|{bound}),n]
        return sum(max(ones[r]-ones[l],r-l-ones[r]+ones[l]) for l,r in zip(b,b[1:]))
    base = mass(set())
    budget = n-base
    def prefixes(chain):
        return [set(chain[:i]) for i in range(min(budget,len(chain))+1)]
    rprefix = prefixes(paths[bound])
    dprefix = prefixes(paths[bound+g])
    best, single = 0, 0
    winners = []
    for branches in subsets(special):
        for i,rooted in enumerate(rprefix):
            for j,other in enumerate(dprefix if bound+g in branches else dprefix[:1]):
                if i+j+len(branches)>budget:
                    break
                selected = branches|rooted|other
                if any(not support[t] & (selected|{bound}) for t in branches):
                    continue
                gain = mass(selected)-base-len(selected)
                if gain>best:
                    best,winners = gain,[]
                if gain==best:
                    winners.append(selected)
                if gain>single:
                    reached = rooted|{bound}
                    while True:
                        before = len(reached)
                        for t in branches:
                            if support[t] & (reached|{t}):
                                reached.add(t)
                                if t==bound+g:
                                    reached |= other
                        if len(reached)==before:
                            break
                    if selected<=reached:
                        single = gain
    width = min(graph_width({bound},T,support) for T in winners)
    if exhaustive:
        # All subsets, independent direct source/target pair histograms and actual stability.
        all_best, single_best, min_width = -n,-n,n
        for selected in subsets(support):
            b = sorted(selected|{bound})
            targets = [bisect_right(b,x) for x in range(n)]
            hist = [Counter() for _ in range(len(b)+1)]
            groups = {}
            for t,c in enumerate(last):
                parent,destination = targets[t],targets[lf[t]]
                if (parent,c) in groups and groups[parent,c]!=destination:
                    break
                groups[parent,c] = destination
                hist[parent][c,destination] += 1
            else:
                gain = sum(max(c.values()) for c in hist)-base-len(selected)
                w = graph_width({bound},selected,support)
                if w<=1:
                    single_best=max(single_best,gain)
                if gain>all_best:
                    all_best,min_width = gain,w
                elif gain==all_best:
                    min_width=min(min_width,w)
        assert (best,single,width) == (all_best,single_best,min_width)
    return {'k':k,'scale':g,'delta':delta,'primitive':True,'n':n,'joint_gain':best,
            'singleton_gain':single,'least_winner_batch':width,'winner_count':len(winners),
            'first_winner':sorted(winners[0])}


if __name__ == '__main__':
    gates = [example(k,d,g,True) for k,g in ((3,1),(4,1),(5,1),(3,2)) for d in (-1,1)]
    census = [example(k,d) for k in range(3,18) for d in (-1,1)]
    census.extend(example(k,-1) for k in (24,36,42))
    print(json.dumps({'arbitrary_subset_oracle':gates,'exact_charged_optima':census},indent=2))
