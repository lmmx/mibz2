"""Exact sharpness, graph certificates, and the realized first original-byte cuts."""
import argparse
import bz2
from collections import Counter, defaultdict
import csv
from itertools import groupby, product
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "powers"))
import real
from cut import choose, geometry, overlay, seed


def matching_cut(edges, anchor):
    weights = Counter()
    for a, targets in enumerate(edges):
        for b, mass in targets.items():
            if a != b:
                weights[min(a,b),max(a,b)] += mass
    adj = [dict() for _ in edges]
    for (a,b), weight in weights.items():
        adj[a][b] = adj[b][a] = weight
    mate = {}
    matched = 0
    for (a,b), weight in sorted(weights.items(), key=lambda p: (-p[1],p[0])):
        if a not in mate and b not in mate:
            mate[a], mate[b] = b, a
            matched += weight
    colors = {}
    for a in range(len(edges)):
        if a in colors:
            continue
        b = mate.get(a)
        group = [(a,False)] + ([(b,True)] if b is not None else [])
        gain = 0
        for k,c in group:
            for j,w in adj[k].items():
                if j in colors:
                    gain += w if colors[j]==c else -w
        for k,c in group:
            colors[k] = c ^ (gain>0)
    target = {k for k in colors if colors[k] != colors[anchor]}
    q = ingress(edges,target)
    assert 4*q >= sum(weights.values()) + matched
    assert len(colors)==len(edges) and anchor not in target
    return target, dict(undirected_pairs=len(weights),matched_pairs=len(mate)//2,matched=matched)


def ingress(edges, target):
    enter = sum(v for i,row in enumerate(edges) if i not in target for j,v in row.items() if j in target)
    leave = sum(v for i,row in enumerate(edges) if i in target for j,v in row.items() if j not in target)
    assert enter==leave
    return enter


def graph(words):
    keys = sorted(set(words))
    assert keys==list(range(len(keys)))
    # Stable equal-rotation order and original primary are explicit.
    starts = sorted(range(len(words)),key=lambda i: words[i:]+words[:i])
    primary = starts.index(0)
    last = [words[(i-1)%len(words)] for i in starts]
    runs = [(k,sum(1 for _ in g)) for k,g in groupby(last)]
    bounds = geometry(runs,len(keys))[1]
    edges = overlay(runs,bounds)[1]
    src_order = sorted(range(len(last)),key=last.__getitem__)
    p = primary
    recovered = []
    for _ in last:
        p = src_order[p]
        recovered.append(last[p])
    assert recovered == list(words)
    from_word = Counter(zip(words,words[1:]+words[:1]))
    assert from_word == {(i,j):v for i,row in enumerate(edges) for j,v in row.items()}
    return edges


def euler(k):
    remaining = [[j for j in range(k) if j!=i] for i in range(k)]
    stack = [0]
    tour = []
    while stack:
        links = remaining[stack[-1]]
        if links:
            stack.append(links.pop())
        else:
            tour.append(stack.pop())
    word = tuple(reversed(tour))[:-1]
    assert len(word)==k*(k-1) and len(set(zip(word,word[1:]+word[:1])))==len(word)
    return word


def finite():
    out = []
    for k in range(2,13):
        for anchored in (False,True):
            w = euler(k)
            if anchored:
                w = (k,)+w
            e = graph(w)
            assert bz2.decompress(bz2.compress(bytes(w)))==bytes(w)
            anchor = k if anchored else 0
            best = max(ingress(e,{i for i in range(len(e)) if mask>>i & 1})
                       for mask in range(1 << len(e)) if not mask>>anchor & 1)
            q,r = matching_cut(e,anchor)
            ceiling = k*k//4 + int(anchored)
            assert best <= ceiling
            if not anchored:
                assert best == ceiling == ingress(e,q) and 4*best == len(w)+r['matched']
            out.append(dict(k=k,singleton=anchored,rows=len(w),best=best,ceiling=ceiling,realized=ingress(e,q),**r))
    return out


def check_tiny():
    checked = 0
    for n in range(1,8):
        for raw in product(range(3),repeat=n):
            ids = {v:k for k,v in enumerate(sorted(set(raw)))}
            last = [ids[v] for v in raw]
            runs = [(k,sum(1 for _ in g)) for k,g in groupby(last)]
            e = overlay(runs,geometry(runs,len(ids))[1])[1]
            for anchor in range(len(ids)):
                matching_cut(e,anchor)
                checked += 1
    return checked


def corpus(tapes):
    out = []
    for path in sorted(tapes.glob("*.irec.xz")):
        meta,_,mark,raw,words = real.source(path)
        raw,words,anchor = seed(raw,words,mark)
        bounds=geometry(raw,len(words))[1]
        slices,edges=overlay(raw,bounds)
        marked = sum(row.get(k,0) for k,row in enumerate(edges))
        new,extra = matching_cut(edges,anchor)
        old = choose(edges,anchor)
        q,control = ingress(edges,new),ingress(edges,old)
        out.append(dict(index=meta['index'],rows=bounds[-1],types=len(words),runs=len(raw),
                        overlays=len(slices),self_mass=marked,
                        bound_old_numerator=bounds[-1]-marked,
                        bound_new_numerator=bounds[-1]-marked+extra['matched'],
                        ingress_old=control,ingress_matching=q,**extra))
    assert len(out)==10
    return out


if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("mode",choices=["finite","small","real"])
    p.add_argument("tapes",type=Path,nargs="?")
    args=p.parse_args()
    if args.mode=="small":
        print(check_tiny(),"small anchored columns")
    else:
        rows=finite() if args.mode=="finite" else corpus(args.tapes)
        writer=csv.DictWriter(sys.stdout,fieldnames=rows[0],lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
