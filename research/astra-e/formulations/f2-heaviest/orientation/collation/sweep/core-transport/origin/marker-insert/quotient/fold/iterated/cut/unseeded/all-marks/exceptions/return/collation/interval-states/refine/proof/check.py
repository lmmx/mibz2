"""Exhaustively compare simultaneous target refinement and a sparse optimal segmenter."""
from bisect import bisect_right
from collections import Counter, defaultdict
from itertools import product
from pathlib import Path
import json
import sys

SYS = Path(__file__).resolve().parents[5]
sys.path.insert(0, str(SYS))
from clock import index


def positions(mask, n):
    return [z for z in range(1, n) if mask & (1 << (z-1))]


def subsets(mask):
    t = mask
    while True:
        yield t
        if t == 0:
            return
        t = (t-1) & mask


def evaluate(column, lf, source, target):
    counts = defaultdict(Counter)
    for i, a in enumerate(column):
        counts[source[i]][a, target[lf[i]]] += 1
    return sum(max(row.values()) for row in counts.values())


def subpartition(lo, hi, labels, candidates, scale, charge):
    """Return scaled reward less a charge per internal seam, plus a maximizing set."""
    prefix = {}
    last = {}
    floor, restart = -charge, lo
    best, start = -charge, lo
    previous = lo
    endpoints = sorted({i for i in range(lo+1, hi) if labels[i] != labels[i-1]}
                       | set(candidates) | {hi})
    for endpoint in endpoints:
        label = labels[previous]
        value, origin = last.get(label, (floor, restart))
        if value < floor:
            value, origin = floor, restart
        value += scale * (endpoint-previous)
        last[label] = value, origin
        if value > best:
            best, start = value, origin
        if endpoint == hi or endpoint in candidates:
            prefix[endpoint] = start
            if best-charge > floor:
                floor, restart = best-charge, endpoint
        previous = endpoint
    x = prefix[hi]
    cuts = []
    while x > lo:
        cuts.append(x)
        x = prefix[x]
    return best+charge, set(cuts)


def score_dp(n, old, allowed, labels, scale, charge):
    boundaries = [0, *positions(old, n), n]
    cuts = 0
    total = 0
    for lo, hi in zip(boundaries, boundaries[1:]):
        local = {z for z in positions(allowed, n) if lo < z < hi}
        val, chosen = subpartition(lo, hi, labels, local, scale, charge)
        total += val
        for z in chosen:
            cuts |= 1 << (z-1)
    return total, old|cuts


def census():
    count = Counter()
    examples = {}
    penalty = [(1,0),(2,1),(1,1),(2,3),(1,3)]
    for n in range(1, 8):
        for sigma in (2, 3) if n <= 4 else (2,):
            for col in product(range(sigma), repeat=n):
                F, psi, lf = index(col)
                full = (1 << (n-1))-1
                fseams = sum(1 << (z-1) for z in range(1,n) if F[z-1] != F[z])
                x, lap = lf[0], 1
                while x != 0:
                    x, lap = lf[x], lap+1
                primitive = lap == n
                info = []
                for old in range(full+1):
                    cuts = positions(old,n)
                    st = [bisect_right(cuts, i) for i in range(n)]
                    labels = [(a,st[lf[i]]) for i,a in enumerate(col)]
                    nearest = [(col[i],st[i]) for i in psi]
                    safe = sum(1 << (z-1) for z in range(1,n)
                               if nearest[z-1] != nearest[z])
                    # Equivalently each old source cut maps under C[a]+Occ_a.
                    edge = set()
                    for a in set(col):
                        c = F.index(a)
                        for i in (0,*cuts,n):
                            at = c + col[:i].count(a)
                            if 0 < at < n:
                                edge.add(at)
                    assert set(positions(safe, n)) == edge
                    protected = full
                    groups = defaultdict(lambda: defaultdict(list))
                    for i, t in enumerate(labels):
                        groups[st[i]][t].append(lf[i])
                    for row in groups.values():
                        winner = min(row, key=lambda t: (-len(row[t]),t))
                        hits = sorted(row[winner])
                        assert hits == list(range(hits[0],hits[-1]+1))
                        for z in range(hits[0]+1,hits[-1]+1):
                            protected &= ~(1 << (z-1))
                    assert safe & ~protected == 0
                    info.append((st,labels,safe,evaluate(col,lf,st,st),protected))
                for old,(st,labels,safe,before,protected) in enumerate(info):
                    count['old_partitions'] += 1
                    allowed = safe & ~old
                    best = [-n*n-1]*len(penalty)
                    for new in subsets(full ^ old):
                        joint = old|new
                        jt,_,new_safe,joint_score,_ = info[joint]
                        supports = defaultdict(set)
                        for i, a in enumerate(col):
                            supports[jt[i],a,st[lf[i]]].add(jt[lf[i]])
                        unchanged = all(len(v) == 1 for v in supports.values())
                        assert unchanged == (new & ~new_safe == 0)
                        count['nested_old_new'] += 1
                        if unchanged:
                            count['target_histograms_neutral'] += 1
                            assert joint_score == evaluate(col, lf, jt, st)
                            assert joint_score >= before
                        if new & ~protected == 0:
                            count['inherited_winner_subsets'] += 1
                            assert joint_score >= before
                            if joint_score < evaluate(col,lf,jt,st):
                                count['winner_safe_lacks_DP_equality'] += 1
                                if primitive and 'winner_safe_only' not in examples:
                                    examples['winner_safe_only'] = dict(L=list(col),
                                        old=positions(old,n), added=positions(new,n), S=before,
                                        refined_S=joint_score,source_only_S=evaluate(col,lf,jt,st))
                        if new & ~safe == 0:
                            assert unchanged
                            count['old_safe_subsets'] += 1
                            count['joint_safe_2_or_more'] += new.bit_count() >= 2
                            for h,(scale,charge) in enumerate(penalty):
                                val = scale*joint_score-charge*new.bit_count()
                                best[h] = max(best[h], val)
                        if new.bit_count() == 1 and joint_score < before:
                            count['single_seam_loss'] += 1
                            if primitive:
                                tags = ['primitive']
                                if old == fseams:
                                    tags.append('primitive_from_F')
                                for tag in tags:
                                    if tag not in examples:
                                        examples[tag] = dict(L=list(col),LF=lf,S=before,
                                            added=positions(new,n),old=positions(old,n),
                                            refined_S=joint_score,old_safe=positions(safe,n),
                                            source_only_S=evaluate(col,lf,jt,st),F=F)
                    for i,(scale,charge) in enumerate(penalty):
                        val,selected = score_dp(n,old,allowed,labels,scale,charge)
                        added = selected ^ old
                        assert added & ~allowed == 0
                        assert val == best[i] == scale*info[selected][3]-charge*added.bit_count()
                        count['exact_dp_comparisons'] += 1
                count['L_columns'] += 1
    loss_family = []
    for g in (1,2,3,5,16,31,128):
        col = (1,)*g + (0,)*(4*g+1)
        n=len(col)
        F,psi,lf=index(col)
        assert lf==[(i-g)%n for i in range(n)]
        old=[int(i>=4*g+1) for i in range(n)]
        joint=[int(i>=2*g)+int(i>=4*g+1) for i in range(n)]
        vals=[evaluate(col,lf,old,old),evaluate(col,lf,joint,old),evaluate(col,lf,joint,joint)]
        assert vals==[4*g+1,4*g+1,3*g+1]
        loss_family.append(dict(g=g,n=n,old=vals[0],source_only=vals[1],joint=vals[2]))
    return dict(counts=dict(count),counterexamples=examples,primitive_loss_family=loss_family)


if __name__ == '__main__':
    print(json.dumps(census(), indent=2))
