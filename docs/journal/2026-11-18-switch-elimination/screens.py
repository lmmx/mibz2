"""Early gates: exact weighted paths, deeper presentations, and small counterexamples."""
import itertools
import sys
from bisect import bisect_right
from pathlib import Path

from boundary import initial, interval_match, section, slab_match


def matching(starts, arcs):
    # In image order the earlier arcs sharing a source or target form a suffix.
    first_source, first_target = {}, {}
    best = [0]
    for k, (u, v, x, y, w) in enumerate(sorted(arcs, key=lambda a: starts[a[1]] + a[3])):
        left = min(first_source.setdefault(u, k), first_target.setdefault(v, k))
        best.append(max(best[-1], w + best[left]))
    return best[-1]


def refine(starts, arcs):
    # Pull back the current cell boundaries once. A new vertex is an old
    # transition. Its image can meet several new vertices.
    fine = [starts[u] + x for u, _, x, _, _ in arcs] + [starts[-1]]
    new = []
    for i, (_, v, _, y, w) in enumerate(arcs):
        a = starts[v] + y
        j = bisect_right(fine, a) - 1
        k = 0
        while k < w:
            s = min(w - k, fine[j + 1] - a - k)
            new.append((i, j, k, a+k-fine[j], s))
            k += s
            j += 1
    return fine, new


def tree_weight(starts, arcs, p):
    parents = list(range(len(starts) - 1))
    size = [1] * len(parents)

    def find(v):
        while parents[v] != v:
            parents[v] = parents[parents[v]]
            v = parents[v]
        return v

    q = m = 0
    for u, v, x, _, w in sorted(arcs, key=lambda a: -a[4]):
        a, b = find(u), find(v)
        if a == b or starts[u]+x <= p < starts[u]+x+w:
            q += 1
            m += w
        else:
            if size[a] < size[b]:
                a, b = b, a
            parents[b] = a
            size[a] += size[b]
    return q, m


def witness():
    seen = {}
    for n in range(3, 14):
        for l in itertools.product(range(2), repeat=n):
            starts, arcs = initial(l)
            r = len(starts) - 1
            f = [0] * n
            for u, v, x, y, w in arcs:
                a, b = starts[u]+x, starts[v]+y
                f[a:a+w] = range(b, b+w)
            cycle = {0}
            at = f[0]
            while at not in cycle:
                cycle.add(at)
                at = f[at]
            if len(cycle) != n:
                continue
            profile = tuple(sorted((l[x], y-x) for x, y in zip(starts, starts[1:])))
            key = n, profile
            value = len(arcs)-r
            if key in seen and seen[key][1] != value:
                a, d = seen[key]
                print("primitive, same per-letter runs and mark 0",
                      "".join(map(str, a)), d, "".join(map(str, l)), value)
                return
            seen[key] = l, value


def worked_example():
    l = bytes(map(int, "11010110100"))
    starts, arcs = initial(l)
    chords, new, _, _, _ = section(arcs, len(starts)-1, (0, 0))
    assert [(starts[arcs[i][0]] + arcs[i][2], arcs[i][4]) for i in chords] == [(0, 2), (8, 1), (10, 1)]
    assert new == [(0, 1, 0, 0, 1), (0, 0, 1, 0, 1), (1, 2, 0, 0, 1), (2, 0, 0, 1, 1)]
    print("eleven-row first returns at original rows: 0 8 10 1 0")


def nested(k):
    # On [0,k), every site is a self-loop, split once at its own i.
    # At height k the internal tree is the directed chain of all k sites,
    # closed by one chord. All sites have the same active range [0,k+1).
    ports = []
    chord = 0
    closing_chord = 2*k - 1
    for site in range(k):
        terms = []
        for lo, hi in [(0, site), (site, k)]:
            if lo < hi:
                terms.extend([(2*chord+1, lo, hi), (2*chord, lo, hi)])
                chord += 1
        if site == 0:
            terms.append((2*closing_chord+1, k, k+1))
        if site == k-1:
            terms.append((2*closing_chord, k, k+1))
        ports.extend(sorted(terms, key=lambda item: item[0] % 2 == 0))
    result, comparisons = slab_match(ports)
    assert sorted(result) == sorted(interval_match(ports)) and len(result) == 2*k
    assert comparisons == k*k + 1
    return k, comparisons, len(result)


if __name__ == "__main__":
    column = Path(sys.argv[1]).read_bytes()
    mark = int(sys.argv[2])
    starts, arcs = initial(column)
    value = matching(starts, arcs)
    print("best simultaneous one-in/one-out mass", value, "outside", len(column)-value)
    print("context_length,vertices,arcs,chords,retained")
    for k in range(1, 8):
        print(k, len(starts)-1, len(arcs), *tree_weight(starts, arcs, mark), sep=",")
        starts, arcs = refine(starts, arcs)
    worked_example()
    witness()
    print("chain vertices, slab matches, interval reports", nested(64))
