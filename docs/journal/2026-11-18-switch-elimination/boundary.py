"""Two independent constructors from ordered open/close intervals, with a dense LF oracle.

The interval index computes one record for each actual matching pair. No loop in
it ranges over heights, over slabs, or over the interiors of the tree paths.
"""
import argparse
import heapq
from bisect import bisect_right
from collections import Counter
from itertools import accumulate, groupby
from pathlib import Path


class OverlapIndex:
    """Newest live opening fragment meeting a query interval of elementary heights."""

    def __init__(self, heights):
        self.size = 1 << (max(1, heights) - 1).bit_length()
        self.heaps = [[] for _ in range(2 * self.size)]
        self.maximum = [-1] * (2 * self.size)
        self.fragments = []
        self.alive = []

    def _update(self, node, a, b, lo, hi, key, insert):
        if hi <= a or b <= lo:
            return
        heap = self.heaps[node]
        if lo <= a and b <= hi:
            if insert:
                heapq.heappush(heap, (-self.fragments[key][2], -key))
            while heap and not self.alive[-heap[0][1]]:
                heapq.heappop(heap)
        else:
            mid = (a + b) // 2
            self._update(2 * node, a, mid, lo, hi, key, insert)
            self._update(2 * node + 1, mid, b, lo, hi, key, insert)
        best = -1 if not heap else -heap[0][1]
        if node < self.size:
            best = self.newer(best, self.maximum[2 * node], self.maximum[2 * node + 1])
        self.maximum[node] = best

    def newer(self, *keys):
        return max(keys, key=lambda k: (-1, -1) if k < 0 else (self.fragments[k][2], k))

    def put(self, lo, hi, stamp, port):
        if lo == hi:
            return
        key = len(self.fragments)
        self.fragments.append((lo, hi, stamp, port))
        self.alive.append(True)
        self._update(1, 0, self.size, lo, hi, key, True)

    def take(self, key):
        lo, hi, stamp, port = self.fragments[key]
        self.alive[key] = False
        self._update(1, 0, self.size, lo, hi, key, False)
        return lo, hi, stamp, port

    def latest(self, lo, hi):
        def query(node, a, b):
            if hi <= a or b <= lo:
                return -1
            if lo <= a and b <= hi:
                return self.maximum[node]
            mid = (a + b) // 2
            heap = self.heaps[node]
            return self.newer(-heap[0][1] if heap else -1,
                              query(2 * node, a, mid), query(2 * node + 1, mid, b))
        return query(1, 0, self.size)


def interval_match(ports):
    """Port order is fixed. IDs 2i+1 open, IDs 2i close. Input is one tree."""
    coordinates = sorted({z for _, lo, hi in ports for z in (lo, hi)})
    at = {z: i for i, z in enumerate(coordinates)}
    index = OverlapIndex(len(coordinates) - 1)
    output = []
    for stamp, (port, lo, hi) in enumerate(ports):
        lo, hi = at[lo], at[hi]
        if port % 2:
            index.put(lo, hi, stamp, port)
            continue
        work = [(lo, hi)]
        while work:
            a, b = work.pop()
            key = index.latest(a, b)
            assert key >= 0
            l, r, t, opener = index.take(key)
            x, y = max(a, l), min(b, r)
            output.append((opener, port, coordinates[x], coordinates[y]))
            index.put(l, x, t, opener)
            index.put(y, r, t, opener)
            if a < x:
                work.append((a, x))
            if y < b:
                work.append((y, b))
    assert index.maximum[1] == -1
    return output


def slab_match(ports):
    changes = {}
    for i, (port, lo, hi) in enumerate(ports):
        for z in (lo, hi):
            changes.setdefault(z, []).append((i, port))
    active = set()
    segments = {}
    tests = 0
    keys = sorted(changes)
    for z, end in zip(keys, keys[1:]):
        active.symmetric_difference_update(changes[z])
        stack = []
        for _, port in sorted(active):
            if port % 2:
                stack.append(port)
            else:
                pair = (stack.pop(), port)
                tests += 1
                if pair in segments:
                    assert segments[pair][1] == z
                    segments[pair][1] = end
                else:
                    segments[pair] = [z, end]
        assert not stack
    return [(*pair, *region) for pair, region in segments.items()], tests


def initial(column):
    runs = [(c, sum(1 for _ in it)) for c, it in groupby(column)]
    starts = [0, *accumulate(w for _, w in runs)]
    totals = Counter(column)
    slots = [0, *accumulate(totals[c] for c in range(256))]
    arcs = []
    for u, (c, w) in enumerate(runs):
        y = slots[c]
        slots[c] += w
        v = bisect_right(starts, y) - 1
        k = 0
        while k < w:
            h = min(w - k, starts[v+1] - y - k)
            arcs.append((u, v, k, y + k - starts[v], h))
            k += h
            v += 1
    return starts, arcs


def dyadic_counts(adj, arcs, chords, new):
    """Independent cost of eagerly building the two directed products of tree climbs."""
    n = len(adj)
    parent = [None] * n
    depth = [0] * n
    for root in range(n):
        if parent[root] is not None:
            continue
        parent[root] = root
        pending = [root]
        for u in pending:
            for v, _, _ in adj[u]:
                if parent[v] is None:
                    parent[v] = u
                    depth[v] = depth[u] + 1
                    pending.append(v)
    up = [parent]
    while 1 << len(up) <= max(depth):
        prev = up[-1]
        up.append([prev[prev[i]] for i in range(n)])
    seen = set()

    def query(u, exponent, reverse):
        key = u, exponent, reverse
        if exponent and key not in seen:
            seen.add(key)
            query(u, exponent-1, reverse)
            query(up[exponent-1][u], exponent-1, reverse)

    joins = recipes = 0
    for u, v, _, _, _ in new:
        entry, end = arcs[chords[u]][1], arcs[chords[v]][0]
        if entry == end:
            continue
        recipes += 1
        a, b = entry, end
        while depth[a] > depth[b]:
            a = parent[a]
        while depth[b] > depth[a]:
            b = parent[b]
        while a != b:
            a, b = parent[a], parent[b]
        for site, reverse in [(entry, True), (end, False)]:
            k = depth[site] - depth[a]
            for j in reversed(range(k.bit_length())):
                if k & (1 << j):
                    query(site, j, reverse)
                    joins += 1
                    site = up[j][site]
    return len(seen), len(seen) + joins, recipes


def section(arcs, num, mark, products=False):
    parent, sizes = list(range(num)), [1] * num

    def root(u):
        while parent[u] != u:
            parent[u] = parent[parent[u]]
            u = parent[u]
        return u

    adj = [[] for _ in range(num)]
    chords = []
    for i in sorted(range(len(arcs)), key=lambda j: -arcs[j][4]):
        u, v, x, y, w = arcs[i]
        a, b = root(u), root(v)
        marked = u == mark[0] and x <= mark[1] < x + w
        if marked:
            marked_arc = i
        if a == b or marked:
            chords.append(i)
        else:
            if sizes[a] < sizes[b]:
                a, b = b, a
            parent[b] = a
            sizes[a] += sizes[b]
            adj[u].append((v, x-y, False))
            adj[v].append((u, y-x, True))
    chords.sort()
    if len(chords) == len(arcs):
        return None
    sites = [[] for _ in adj]
    for j, i in enumerate(chords):
        u, v, x, y, w = arcs[i]
        sites[u].append((2*j, x, w))
        sites[v].append((2*j+1, y, w))
    # Emitting the signed stubs in an in-order traversal is enough. The two
    # matchers below receive neither the tree nor any interior edge rectangle.
    seen = set()
    components = []
    for start in range(num):
        if start in seen:
            continue
        ports = []
        stack = [(start, -1, 0, False)]
        while stack:
            u, par, z, visit = stack.pop()
            if visit:
                for port, x, w in sorted(sites[u], key=lambda a: (a[0] % 2 == 0, a[0])):
                    ports.append((port, z+x, z+x+w))
                continue
            seen.add(u)
            for incoming in [False, True]:
                if incoming:
                    stack.append((u, par, z, True))
                stack.extend((v, u, z+d, False) for v, d, flag in adj[u]
                             if v != par and flag == incoming)
        if ports:
            components.append(ports)
    offsets = {p: lo for ports in components for p, lo, _ in ports}
    matches = []
    tests = 0
    for ports in components:
        result = interval_match(ports)
        slabs, j = slab_match(ports)
        assert sorted(result) == sorted(slabs)
        matches.extend(result)
        tests += j
    new = sorted(((a//2, b//2, l-offsets[a], l-offsets[b], h-l)
                  for a, b, l, h in matches), key=lambda e: (e[0], e[2]))
    new_mark = (chords.index(marked_arc), mark[1] - arcs[marked_arc][2])
    extra = dyadic_counts(adj, arcs, chords, new) if products else ()
    return chords, new, new_mark, tests, extra


def run(column, p, products=False):
    starts, arcs = initial(column)
    u = bisect_right(starts, p) - 1
    mark = (u, p - starts[u])
    n = len(column)
    row_map = starts[:-1]
    # Dense independent oracle only. Never passed to either open solver.
    total = Counter(column)
    slots = [0, *accumulate(total[c] for c in range(256))]
    lf = []
    for c in column:
        lf.append(slots[c])
        slots[c] += 1
    suffix = ",dyadic_fragments,eager_joins,path_recipes" if products else ""
    print("layer,sites,arcs,rows,chords,retained,slab_pairs,interval_reports" + suffix)
    level = 0
    while result := section(arcs, len(row_map), mark, products):
        chords, new, mark, tests, extra = result
        bases = [row_map[arcs[i][0]] + arcs[i][2] for i in chords]
        keep = bytearray(n)
        for base, i in zip(bases, chords):
            w = arcs[i][4]
            keep[base:base+w] = bytes([1]) * w
        for u, v, x, y, w in new:
            for k in range(w):
                row = lf[bases[u] + x + k]
                while not keep[row]:
                    row = lf[row]
                assert row == bases[v] + y + k
        print(level, len(row_map), len(arcs), sum(e[4] for e in arcs), len(chords),
              sum(keep), tests, len(new), *extra, sep=",", flush=True)
        row_map, arcs = bases, new
        level += 1


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("column", type=Path)
    parser.add_argument("primary", type=int)
    parser.add_argument("--products", action="store_true")
    args = parser.parse_args()
    run(args.column.read_bytes(), args.primary, args.products)
