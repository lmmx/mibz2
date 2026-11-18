from itertools import groupby, product
from bisect import bisect_right
import random


def lf(column):
    inverse = sorted(range(len(column)), key=lambda i: (column[i], i))
    return [j for _, j in sorted((i, j) for j, i in enumerate(inverse))]


def intervals(points):
    return [tuple(group) for _, group in groupby(points, key=lambda x, j=iter(range(len(points))): x-next(j))]


def audit(L, S):
    n, m = len(L), len(S)
    f = lf(L)
    E = sorted(f[i] for i in S)
    s_idx, e_idx = ({x: i for i, x in enumerate(a)} for a in (S, E))
    v = tuple(L[i] for i in S)
    fv = lf(v)
    assert [e_idx[f[i]] for i in S] == fv

    C = [i for i in range(n) if i not in s_idx]
    strips = []
    for _, group in groupby(range(n), key=lambda i: (i in s_idx, L[i])):
        band = tuple(group)
        if band[0] not in s_idx:
            strips.append(band)
    sid = {x: j for j, strip in enumerate(strips) for x in strip}
    sw = intervals(S)
    ew = intervals(E)
    exit_id = {x:j for j, s in enumerate(sw) for x in s}
    e_id = {x:j for j, e in enumerate(ew) for x in e}
    entrance_bytes = [v[j] for j in sorted(range(m), key=lambda j: fv[j])]
    expected = []
    for e in E:
        x, path = e, []
        while x not in s_idx:
            path.append(sid[x])
            x = f[x]
            assert len(path) <= len(C)
        expected.append((s_idx[x], tuple(path), exit_id[x], e_id[e]))
    g = [expected[k][0] for k in fv]
    assert sorted(g) == list(range(m))
    sym = [L[piece[0]] for piece in strips]
    by_dst = {dst: (j, (v[j],) + tuple(sym[t] for t in expected[k][1]))
              for j, k in enumerate(fv) for dst in (g[j],)}
    f_inv = {y:x for x,y in enumerate(f)}
    for p in S:
        ref, q = [], p
        while True:
            q = f_inv[q]
            ref.append(L[q])
            if q == p:
                break
        actual, j = [], s_idx[p]
        while True:
            j, word = by_dst[j]
            actual.extend(reversed(word))
            if j == s_idx[p]:
                break
        assert actual == ref

    # Count gaps by independently following them backward to an admissible origin.
    gap_back = {f[x]: x for piece in strips for x in piece[1:]}
    source_gaps = {x for e in ew for x in e[1:]}
    boundaries = {z[0] for z in strips + sw} - {0}
    charged = set()
    for b in boundaries:
        x = b
        while x in gap_back:
            x = gap_back[x]
        if x in source_gaps:
            charged.add(x)
    broken_plain, broken_cuts = set(), set()
    for i in range(1, m):
        x, y = expected[i-1], expected[i]
        if y[0] != x[0]+1 or x[1:] != y[1:]:
            broken_plain.add(E[i])
        if entrance_bytes[i] != entrance_bytes[i-1]:
            broken_cuts.add(E[i])
    assert broken_plain == (set(E) - source_gaps - {E[0]}) | charged
    B = 1 + len(broken_plain | broken_cuts)
    H, sigma = len(strips), len(set(L))
    e = 1 + len((set(E)-source_gaps-{E[0]}) | broken_cuts)
    assert B <= e + H + len(sw) - 1 <= 3*H + sigma
    # A label for each initial strip and connector preserves its itinerary.
    widths, words, domain, image = {}, {}, {}, {}
    def add(d, a, w, label=()):
        i = len(widths)
        widths[i], words[i] = w, label
        domain[d], image[a] = i, i
    for group in ew:
        add(e_idx[group[0]], m+group[0], len(group))
    for k, group in enumerate(strips):
        add(m+group[0], m+f[group[0]], len(group), (k,))
    for group in sw:
        add(m+group[0], s_idx[group[0]], len(group))
    top = [domain[k] for k in sorted(domain)]
    bot = [image[k] for k in sorted(image)]
    total = n
    while total:
        x, y = top[-1], bot[-1]
        a, b = widths[x], widths[y]
        w = min(a,b)
        assert w <= total
        total -= w
        if x == y:
            top.pop()
            bot.pop()
        elif a <= b:
            top.pop()
            top.insert(top.index(y)+1, x)
            words[x] += words[y]
            if a == b:
                top.remove(y)
                bot.pop()
            else:
                widths[y] -= w
        else:
            bot.pop()
            bot.insert(bot.index(x)+1, y)
            words[y] = words[x] + words[y]
            widths[x] -= w
        assert len(top) == len(bot) and sum(widths[i] for i in top) == m+total
    j, bottom_starts = 0, {}
    for i in bot:
        bottom_starts[i] = j
        j += widths[i]
    induced = [(bottom_starts[i]+r, tuple(reversed(words[i]))) for i in top for r in range(widths[i])]
    assert induced == [x[:2] for x in expected]
    k, induced_bounds = 0, set()
    for i in top[:-1]:
        k += widths[i]
        induced_bounds.add(E[k])
    assert induced_bounds == broken_plain, (L, S, induced_bounds, broken_plain)

    cut = sum(x in s_idx and f[x] not in s_idx for x in range(n))
    assert cut == len(set(E)-set(S)) == len(set(S)-set(E)) == sum(x not in s_idx and f[x] in s_idx for x in range(n))
    return B, e, H, len(sw), sigma


def threshold_sets(L):
    runs = [tuple(g) for _, g in groupby(range(len(L)), key=L.__getitem__)]
    bounds = sorted(set(map(len, runs)))
    for t in bounds[:-1]:
        S = {x for g in runs if len(g)<=t for x in g}
        for p in range(len(L)):
            if p in S and p != min(S):
                continue
            yield sorted(S | {p}), t, p


def test():
    total = 0
    for alphabet, max_n in [(2, 7), (3, 5)]:
        for n in range(1, max_n+1):
            for L in product(range(alphabet), repeat=n):
                for mask in range(1, 1<<n):
                    S = [x for x in range(n) if mask>>x&1]
                    audit(L, S)
                    total += 1
    print(f"{total} arbitrary-column and nonempty retained-set checks, all their marks")
    rng = random.Random(559565)
    witness = {}
    for _ in range(30000):
        L = tuple(rng.randrange(2 + rng.randrange(2)) for _ in range(rng.randrange(7,28)))
        for S,t,p in threshold_sets(L):
            B,e,H,s,sigma = audit(L,S)
            key = (H, sigma)
            if H and B == 3*H+sigma and key not in witness:
                row = ("".join(map(str,L)),S,t,p,B,e,H,s,sigma)
                witness[key] = row
                print("tight", row)
        if (1,2) in witness and any(h>1 for h,_ in witness):
            break


if __name__ == "__main__":
    test()
