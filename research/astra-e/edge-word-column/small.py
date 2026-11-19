"""Exhaust every binary L, nonempty retained rank set, and retained mark through n=7."""
from collections import Counter
from itertools import product


def ordinary(weights, successor):
    """Minimal contiguous stable refinement and independent inverse from its column."""
    source = []
    base = []
    decoded = []
    for x, word in enumerate(weights):
        if x == 0 or word != weights[x-1] or successor[x] < successor[x-1]:
            base.append(x)
            decoded.append(word)
        source.append(len(base)-1)
    incoming = [None] * len(source)
    for x, y in enumerate(successor):
        incoming[y] = source[x]
    counts = [0] * len(base)
    forward = [None] * len(source)
    for y, symbol in enumerate(incoming):
        x = base[symbol] + counts[symbol]
        counts[symbol] += 1
        forward[x] = y
    assert forward == successor
    return [decoded[k] for k in source], forward


def emitted(object_, mark, offset, n):
    words, dest = object_
    row = mark
    output = b""
    while len(output) < n:
        output += words[row][offset:]
        offset = 0
        row = dest[row]
    return output[:n]


def run(n, L, keep, counts, witnesses):
    psi = sorted(range(n), key=L.__getitem__)
    f = bytes(L[j] for j in psi)
    raw = ([bytes([c]) for c in f], psi)
    s = [i for i in range(n) if keep & (1 << i)]
    loc = {x: i for i, x in enumerate(s)}
    w = []
    t = []
    for x in s:
        end = psi[x]
        spell = bytes([f[x]])
        while end not in loc:
            spell += bytes([f[end]])
            end = psi[end]
        w.append(spell)
        t.append(loc[end])
    # Construct conjugate source order via stable LF. Rank equality persists for tied byte itineraries.
    m = len(s)
    q = sorted(range(m), key=lambda i: L[s[i]])
    sig = [q.index(i) for i in range(m)]
    p = [sig[t[x]] for x in range(m)]
    k_rows = [psi.index(s[i]) for i in q]
    assert sorted(k_rows) == k_rows
    y_rows = sorted(set(s) | set(k_rows))
    j = {x: i for i, x in enumerate(y_rows)}
    wx = [None] * len(y_rows)
    dest = [None] * len(y_rows)
    for x, orig in enumerate(s):
        pivot = k_rows[p[x]]
        if pivot != orig:
            assert len(w[x]) > 1 and orig not in k_rows
            wx[j[orig]] = w[x][:-1]
            dest[j[orig]] = j[pivot]
        else:
            assert len(w[x]) == 1
        # The positive union quotient identifies literal X with K at the same original rank.
        wx[j[pivot]] = w[x][-1:]
        dest[j[pivot]] = j[s[t[x]]]
    for i, orig in enumerate(y_rows):
        end = psi[orig]
        spell = bytes([f[orig]])
        while end not in j:
            spell += bytes([f[end]])
            end = psi[end]
        assert (wx[i], dest[i]) == (spell, j[end])
    # Equal weights have strictly increasing destinations even across physically separated pieces.
    for words, ds in [(w, t), (wx, dest)]:
        positions = {}
        for i, key in enumerate(words):
            assert ds[i] > positions.get(key, -1)
            positions[key] = ds[i]
    physical = ordinary(w, t)
    union = ordinary(wx, dest)
    c = [L[s[i]] for i in range(m)]
    shift_dest = [sig[t[x]] for x in q]
    shift = ordinary([bytes([c[x]]) + w[x][:-1] for x in q], shift_dest)
    conj = ordinary([w[x] for x in q], shift_dest)
    alt = ordinary([v[:-1] for v in w] + [bytes([c[x]]) for x in q],
                   [m+v for v in p] + q)
    if any(w[:i].count(key) and key != w[i-1] for i, key in enumerate(w)):
        witnesses.setdefault("nonconvex", (L.hex(), s, w, t))
    cw = [w[x] for x in q]
    if any(cw[i] == cw[k] and shift_dest[i] > shift_dest[k]
           for i in range(m) for k in range(i+1,m)):
        witnesses.setdefault("conjugate_descent", (L.hex(), s, cw, shift_dest))
    if any(s[i] != k_rows[p[i]] for i in range(m)):
        counts["proper_union"] += 1
    for x, orig in enumerate(s):
        truth = emitted(raw, orig, 0, n)
        assert truth == emitted(physical,x,0,n) == emitted(alt,x,0,n)
        assert truth == emitted(shift,sig[x],1,n) == emitted(conj,sig[x],0,n)
        assert truth == emitted(union,j[orig],0,n)
        period = 1
        z = psi[orig]
        while z != orig:
            z = psi[z]
            period += 1
        counts["short_marked_cycles"] += period < n
        counts["short_nondivisor"] += n % period != 0
        counts["marked_cases"] += 1
    counts["geometries"] += 1


if __name__ == "__main__":
    counts = Counter()
    witnesses = {}
    for n in range(1, 8):
        for bits in product(range(2), repeat=n):
            L = bytes(bits)
            for keep in range(1, 1 << n):
                run(n, L, keep, counts, witnesses)
    for k,v in sorted(counts.items()):
        print(k, v)
    for name, witness in witnesses.items():
        L, ranks, words, dest = witness
        print(name, "L", L, "S", ranks, "weights", [w.hex() for w in words], "successor", dest)
