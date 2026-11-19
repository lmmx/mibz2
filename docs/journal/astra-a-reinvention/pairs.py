"""Exact structural cost of one ordered substitution followed by the ordinary binary geometry."""
import csv
import heapq
import itertools
import sys
from pathlib import Path

import numpy as np

from early import lf_data


def project(column, p, cut):
    first, lf, _ = lf_data(column)
    low = column <= cut
    survive = ~(low & (first > cut))
    prev = column[lf]
    # Lexicographic names, including unpaired "a" before all "(a,b)".
    token = np.where(~low & (prev <= cut), prev.astype(int)*257+column+1, column.astype(int)*257)
    if survive[p]:
        anchor, offset = p, 0
    else:
        anchor, offset = lf[p], 1
    assert survive[anchor]
    start = np.r_[True, column[1:] != column[:-1]]
    start[1:] |= ~low[1:] & (token[1:] != token[:-1])
    boundary = int(low.sum())
    if 0 < boundary < len(start) and low[boundary]:
        start[boundary] = True
    return token[survive], int(survive[:anchor].sum()), offset, int(start.sum())


def cuts(column, p):
    # Generic full-cycle instance of the verified decreasing first-arrival theorem.
    # The byte alphabet does not occur in the geometry.
    _, names, counts = np.unique(column, return_inverse=True, return_counts=True)
    boundaries = np.unique(np.r_[0, np.flatnonzero(names[1:] != names[:-1])+1, p, p+1, len(names)])
    cursor = list(map(int, counts.cumsum()-counts))
    heap = []
    for top, end in zip(boundaries[:-1], boundaries[1:]):
        c = names[top]
        cursor[c] += int(end-top)
        bottom = cursor[c]
        heap.append((-max(int(end),bottom), bottom-int(end)))
    heapq.heapify(heap)
    a = b = 0
    result = 0
    while heap:
        frontier = -heap[0][0]
        while heap and -heap[0][0] == frontier:
            _, d = heapq.heappop(heap)
            if d == 0:
                assert not heap and frontier == 1
                return result
            if d < 0:
                a = -d
            else:
                b = d
        assert min(a,b) > 0
        heapq.heappush(heap, (min(a,b)-frontier,b-a))
        result += 1
    raise AssertionError("missing closure")


def check_small():
    checks = swallowed = 0
    for n in range(2, 8):
        for word in itertools.product(range(3), repeat=n):
            rots = [word[i:]+word[:i] for i in range(n)]
            if len(set(rots)) != n:
                continue
            order = sorted(range(n), key=rots.__getitem__)
            L = np.array([word[i-1] for i in order], np.uint8)
            for t in (0,1):
                keep = [i for i in range(n) if not (word[i-1] <= t < word[i])]
                keys = [(word[i]*257 + (word[(i+1)%n]+1 if word[i]<=t<word[(i+1)%n] else 0))
                        for i in keep]
                korder = sorted(range(len(keys)), key=lambda i: keys[i:]+keys[:i])
                actual, anchor, skip, _ = project(L, order.index(0), t)
                assert list(actual) == [keys[i-1] for i in korder]
                expected_start = 0 if 0 in keep else n-1
                assert (anchor, skip) == (korder.index(keep.index(expected_start)), int(0 not in keep))
                checks += 1
                swallowed += skip
    return checks, swallowed


if __name__ == "__main__":
    if sys.argv[1] == "small":
        print(check_small())
        sys.exit()
    cols, summary = Path(sys.argv[1]), Path(sys.argv[2])
    out = csv.writer(sys.stdout, lineterminator="\n")
    out.writerow(("block","original_cuts","pair_pieces","pair_runs","pair_cuts","anchor","offset"))
    for r in csv.DictReader(summary.open()):
        block, p, t = int(r["block"]), int(r["primary"]), int(r["pair_cut"])
        column = np.fromfile(cols / f"{block:04}.L", dtype=np.uint8)
        reduced, new_p, offset, pieces = project(column, p, t)
        assert 1+np.count_nonzero(reduced[1:]!=reduced[:-1]) == int(r["pair_runs"])
        out.writerow((block,cuts(column,p),pieces,r["pair_runs"],cuts(reduced,new_p),new_p,offset))
