"""Compare explicit all-subset optimum with pruning for arbitrary vote sequences."""
from collections import Counter
from itertools import product, groupby
from pathlib import Path
import json
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check import subpartition, subsets, positions


def reduced(labels, cuts, penalty):
    n = len(labels)
    if n - max(Counter(labels).values()) <= penalty:
        return set()
    candidates = set(cuts)
    out = set()
    start = 0
    for _, run in groupby(labels):
        end = start + sum(1 for _ in run)
        interior = sorted(z for z in candidates if start < z < end)
        if interior:
            out.update((interior[0], interior[-1]))
        if end in candidates:
            out.add(end)
        start = end
    return out


def check():
    counts = Counter()
    for alpha, maximum in ((2,7),(3,5)):
        for n in range(1, maximum+1):
            for labels in product(range(alpha), repeat=n):
                scores = []
                for selected in range(1 << (n-1)):
                    bounds = [0, *positions(selected,n), n]
                    scores.append(sum(max(Counter(labels[l:r]).values()) for l,r in zip(bounds,bounds[1:])))
                for allowed in range(1 << (n-1)):
                    for charge in (0,1,2,4):
                        cuts = set(positions(allowed,n))
                        keep = reduced(labels,cuts,charge)
                        want = max(scores[s]-charge*s.bit_count() for s in subsets(allowed))
                        val, selected = subpartition(0,n,labels,keep,1,charge)
                        assert val == want and selected <= keep <= cuts
                        counts['checked'] += 1
                        counts['smaller_candidate_sets'] += keep != cuts
    labels = 'a'*5+'b'*8+'a'*5
    both = {6,12}
    target, take = subpartition(0,len(labels),labels,both,1,1)
    assert take == both
    single = [subpartition(0,len(labels),labels,{s},1,1)[0] for s in sorted(both)]
    assert max(single) < target
    return dict(counts=dict(counts),two_extremes_needed=dict(labels=labels,charge=1,cuts=sorted(both),
                objective=target,best_one_seam=max(single)))


if __name__=='__main__':
    print(json.dumps(check(),indent=2))
