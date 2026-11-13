"""Untimed finite mathematical checks; no decoder or corpus execution."""
from itertools import product
import json

def lf(word):
    seen = {}
    counts = {c: word.count(c) for c in set(word)}
    starts = {c: sum(v for a, v in counts.items() if a < c) for c in counts}
    out = []
    for c in word:
        out.append(starts[c] + seen.get(c, 0))
        seen[c] = seen.get(c, 0) + 1
    return out

columns = sections = 0
for n in range(1, 7):
    for word in product('ab', repeat=n):
        t = lf(word)
        inv = [t.index(i) for i in range(n)]
        starts = [i for i in range(n) if i == 0 or word[i] != word[i-1]]
        image_starts = {t[i] for i in starts}
        d = [t[(inv[(i-1) % n]+1) % n] for i in range(n)]
        assert all(d[i] == i for i in range(n) if i not in image_starts)
        columns += 1
        for mask in range(1, 1 << n):
            k = {i for i in range(n) if mask >> i & 1}
            levels = []
            for x in k:
                y = x
                while True:
                    levels.append(y)
                    y = t[y]
                    if y in k:
                        break
            covered = set()
            for x in k:
                y = x
                while y not in covered:
                    covered.add(y)
                    y = t[y]
            assert len(levels) == len(set(levels)) == len(covered)
            assert set(levels) == covered
            sections += 1

assert lf('ab') == [0, 1]  # K={0} misses the orbit of origptr=1.
assert sorted('ab') == sorted('ba') and 'ab' != 'ba'
rotations = {w: lf(w) for w in ('baaaa', 'bbbba')}
for t in rotations.values():
    n = len(t)
    inv = [t.index(i) for i in range(n)]
    assert [t[(inv[(i-1)%n]+1)%n] for i in range(n)] == list(range(n))
assert rotations['baaaa'] != rotations['bbbba']
print(json.dumps({'binary_columns_n1_through_n6': columns,
                  'nonempty_sections': sections, 'checks': 'PASS',
                  'missed_orbit': {'L': 'ab', 'LF': [0,1], 'K': [0], 'origptr': 1},
                  'identical_commutator_distinct_LF': rotations,
                  'scope': 'finite mathematical witnesses; no performance evidence'}, indent=2))
