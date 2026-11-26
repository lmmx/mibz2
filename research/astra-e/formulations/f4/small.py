"""Compare every boundary arrow, plus the expanded result at every possible mark."""
import itertools
import json
from phrases import compress, expanded_marked, inverse, spelling, transform

counts = {"columns": 0, "marks": 0, "swallowed": 0, "short_nondividing": 0, "arrows": 0}
for alphabet, limit in [(2, 7), (3, 6), (4, 4)]:
    for n in range(1, limit + 1):
        for col in itertools.product(range(alphabet), repeat=n):
            psi = sorted(range(n), key=col.__getitem__)
            lf = [None] * n
            for j, i in enumerate(psi):
                lf[i] = j
            F = [col[psi[i]] for i in range(n)]
            for asc in [False, True]:
                S = [i for i in range(n) if (col[i] >= F[i] if asc else col[i] <= F[i])]
                ranks = {row: i for i, row in enumerate(S)}
                counts["columns"] += 1
                for p in range(n):
                    result = transform(compress(col), p, asc)
                    token_runs, rp, skip, parents, dictionary, _ = result
                    expected = inverse(col, p)
                    assert expanded_marked(result, n) == expected
                    counts["marks"] += 1
                    counts["swallowed"] += int(skip > 0)
                    o = psi[p]
                    period = 1
                    while o != p:
                        o = psi[o]
                        period += 1
                    counts["short_nondividing"] += n % period != 0
                    if p != 0:
                        continue
                    T = [c for c, w in token_runs for _ in range(w)]
                    order = sorted(range(len(T)), key=T.__getitem__)
                    for dst, src in enumerate(order):
                        row = S[src]
                        word = []
                        while True:
                            word.append(col[row])
                            row = lf[row]
                            if row in ranks:
                                break
                        assert dst == ranks[row] and word[::-1] == spelling(dictionary[T[src]], parents)
                        counts["arrows"] += 1
print(json.dumps(counts, sort_keys=True))
