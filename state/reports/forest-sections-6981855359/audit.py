from collections import defaultdict, deque
from dataclasses import dataclass
from itertools import groupby, product
from random import Random


@dataclass(frozen=True)
class Arc:
    u: int
    v: int
    x: int
    y: int
    w: int
    word: bytes


def lf(column):
    images = [0] * len(column)
    for rank, row in enumerate(sorted(range(len(column)), key=lambda i: (column[i], i))):
        images[row] = rank
    return images


def network(widths, mapping, label):
    sites = [(v, k) for v, w in enumerate(widths) for k in range(w)]
    arcs = []
    for row, (u, x) in enumerate(sites):
        v, y = sites[mapping[row]]
        if arcs:
            a = arcs[-1]
            if (a.u, a.v, a.x+a.w, a.y+a.w, a.word) == (u, v, x, y, label(row)):
                arcs[-1] = Arc(a.u, a.v, a.x, a.y, a.w+1, a.word)
                continue
        arcs.append(Arc(u, v, x, y, 1, label(row)))
    return arcs


def marked_output(mapping, mark, labels, limit):
    inverse = {y: x for x, y in mapping.items()}
    words, here = [], mark
    while True:
        here = inverse[here]
        words.append(labels[here])
        if here == mark:
            break
    period = b"".join(words)
    return (period * ((limit + len(period)-1)//len(period)))[:limit]


def expand(arcs):
    forward, edges = {}, {}
    for i, a in enumerate(arcs):
        for k in range(a.w):
            u, v = (a.u, a.x+k), (a.v, a.y+k)
            assert u not in forward
            forward[u], edges[u] = v, (i,k)
    assert set(forward) == set(forward.values())
    return forward, edges


def coordinates(tree, arcs):
    comp, pos, length = {}, {}, {}
    for root in range(len(tree)-1, -1, -1):
        if root in comp:
            continue
        comp[root], pos[root], length[root] = root, 0, 0
        queue = deque([root])
        while queue:
            u = queue.popleft()
            for v, i, sign in tree[u]:
                if v in comp:
                    continue
                a = arcs[i]
                comp[v] = root
                pos[v] = pos[u] + sign*(a.x-a.y)
                length[v] = length[u] + sign*len(a.word)
                queue.append(v)
    return comp, pos, length


def first_return(arcs, vertices, mark):
    forward, edges = expand(arcs)
    mark_id = edges[mark][0]
    # Independent explicit relabelling of connected classes, no union-find.
    cls = list(range(vertices))
    forest, chords = [], []
    for i in sorted(range(len(arcs)), key=lambda k: (-arcs[k].w, k)):
        a = arcs[i]
        left, right = cls[a.u], cls[a.v]
        if i == mark_id or left == right:
            chords.append(i)
            continue
        forest.append(i)
        cls = [left if v == right else v for v in cls]
    chords.sort()
    keep = {point: (chords.index(i), k) for point, (i, k) in edges.items() if i in chords}
    new_mark = keep[mark]
    assert sum(arcs[i].w for i in chords) == len(keep)
    tree = [[] for _ in range(vertices)]
    for i in forest:
        a = arcs[i]
        tree[a.u].append((a.v, i, 1))
        tree[a.v].append((a.u, i, -1))
    comp, pos, time = coordinates(tree, arcs)
    for i in forest:
        a = arcs[i]
        assert pos[a.u]+a.x == pos[a.v]+a.y
        assert time[a.v]-time[a.u] == len(a.word)
    ports = [[] for _ in tree]
    for j, i in enumerate(chords):
        a = arcs[i]
        ports[a.u].append((False, j, pos[a.u]+a.x, a.w, a.u))
        ports[a.v].append((True, j, pos[a.v]+a.y, a.w, a.v))

    def order(u, parent):
        for v, _, sign in reversed(tree[u]):
            if v != parent and sign < 0:
                yield from order(v, u)
        yield from sorted(ports[u], reverse=True)
        for v, _, sign in reversed(tree[u]):
            if v != parent and sign > 0:
                yield from order(v, u)

    def left_side(start, omit):
        todo, done = [start], {start}
        while todo:
            for v, i, _ in tree[todo.pop()]:
                if i != omit and v not in done:
                    done.add(v)
                    todo.append(v)
        return done

    domains = {i: left_side(arcs[i].u, i) for i in forest}
    via_ports = {}
    for root in sorted(set(comp.values())):
        po = list(order(root, -1))
        heights = {z for _, _, lo, w, _ in po for z in range(lo, lo+w)}
        for z in heights:
            fibre = [p for p in po if p[2] <= z < p[2]+p[3]]
            for i, side in domains.items():
                a = arcs[i]
                if comp[a.u] == root:
                    parity = sum(site in side for _, _, _, _, site in fibre) % 2
                    assert parity == (pos[a.u]+a.x <= z < pos[a.u]+a.x+a.w)
            stack = []
            for p in fibre:
                if p[0]:
                    stack.append(p)
                else:
                    opening = stack.pop()
                    via_ports[(opening[1], z-opening[2])] = ((p[1], z-p[2]), opening[4], p[4])
            assert not stack
    assert len(via_ports) == len(keep)
    dense = {}
    for point, source in keep.items():
        i, _ = edges[point]
        here, labels = forward[point], [arcs[i].word]
        while here not in keep:
            j, _ = edges[here]
            labels.append(arcs[j].word)
            here = forward[here]
            assert len(labels) <= len(forward)
        dest, start_site, end_site = via_ports[source]
        assert dest == keep[here]
        word = b"".join(reversed(labels))
        assert len(word) == len(arcs[i].word)+time[end_site]-time[start_site]
        dense[source] = (dest, word)
    residual = []
    for (u,x), ((v,y), word) in sorted(dense.items()):
        if residual:
            a = residual[-1]
            if (a.u,a.v,a.x+a.w,a.y+a.w,a.word) == (u,v,x,y,word):
                residual[-1] = Arc(a.u,a.v,a.x,a.y,a.w+1,a.word)
                continue
        residual.append(Arc(u,v,x,y,1,word))
    # A named pair's offsets span one nonempty band.
    assert len({(a.u,a.v) for a in residual}) == len(residual)
    q, c = len(chords), len(set(comp.values()))
    assert q == len(arcs)-vertices+c
    assert len(residual) <= 2*q-c
    assert sum(a.w*len(a.word) for a in residual) == sum(a.w*len(a.word) for a in arcs)
    return residual, q, c, new_mark, len(forest), len(keep)


def audit(arcs, vertices, mark, levels=50):
    forward, edges = expand(arcs)
    n = len(forward)
    expected = marked_output(forward, mark, {p: arcs[i].word for p,(i,_) in edges.items()}, n)
    first = None
    for _ in range(levels):
        residual, q, c, new_mark, kept, retained = first_return(arcs, vertices, mark)
        if first is None:
            first = (len(arcs), vertices, q, c, len(residual), kept, retained, residual)
        target, edges = expand(residual)
        produced = marked_output(target, new_mark, {p: residual[i].word for p,(i,_) in edges.items()}, n)
        assert produced == expected
        if kept == 0 or retained == 1:
            break
        arcs, vertices, mark = residual, q, new_mark
    return first


def test():
    count = 0
    witness = None
    for alphabet, max_size in [(2, 8), (3, 5)]:
        for n in range(1, max_size+1):
            for col in product(range(alphabet), repeat=n):
                f = lf(col)
                widths = [len(list(g)) for _,g in groupby(col)]
                arcs = network(widths, f, lambda i: bytes([col[i]]))
                points = list(expand(arcs)[0])
                cycle = {0}
                at = f[0]
                while at not in cycle:
                    cycle.add(at)
                    at = f[at]
                for p in points:
                    a = audit(arcs, len(widths), p)
                    e, v, q, c, b, k, *_ = a
                    if witness is None and n==len(cycle) and k>0 and e>v and b==2*q-c:
                        witness = ("".join(str(i) for i in col), p, a)
                    count += 1
    print("marked LF columns", count)
    print("primitive nontrivial tight example", witness)
    rng = Random(569698)
    for _ in range(1000):
        n = rng.randrange(1,35)
        widths = [len(list(g)) for _,g in groupby(sorted(rng.randrange(9) for _ in range(n)))]
        mapping = list(range(n))
        rng.shuffle(mapping)
        arcs = network(widths, mapping, lambda row: bytes([(row//3)%4]))
        audit(arcs, len(widths), rng.choice(list(expand(arcs)[0])))
    print("arbitrary labelled permutation networks",1000)


# Recheck the tight family without the prototype or the recorded CSVs.
def check_unit_descent():
    for width in (3, 8, 64):
        column = bytes([1] + [0] * width)
        arcs = network([1, width], lf(column), lambda i: column[i:i+1])
        vertices, mark = 2, (0, 0)
        for expected_mass in range(width, 0, -1):
            arcs, vertices, comp, mark, kept, actual_mass = first_return(arcs, vertices, mark)
            assert (comp, kept, actual_mass) == (1, 1, expected_mass)
    print("primitive unit descent verified at widths 3, 8 and 64")


if __name__ == "__main__":
    test()
    check_unit_descent()
