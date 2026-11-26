"""Stable table switches preserving a true primitive cyclic BWT and its two event slots."""
import csv
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[4]))
from clock import index


def lfsr(k):
    q = 1 << k
    for mask in range(1, q, 2):
        def step(x):
            return (x >> 1) | (((x & mask).bit_count() % 2) << (k-1))
        row = step(1)
        orbit = []
        while row != 1:
            orbit.append(row)
            row = step(row)
        if len(orbit) == q-2:
            return mask, [step(x) for x in range(q)], orbit + [1]
    raise ValueError(k)


def original(column, k, p, event_post, primary_post):
    n = len(column)
    first, psi, lf = index(column)
    x = p
    rotation = []
    for _ in range(n):
        rotation.append(first[x])
        x = psi[x]
        if x == p and len(rotation) < n:
            raise ValueError('multiple cycles')
    assert x == p
    text = bytes(rotation)
    rows = sorted(range(n), key=lambda t: text[t:] + text[:t])
    assert rows[p] == 0 and bytes(text[t-1] for t in rows) == bytes(column)
    end = text+text
    contexts = []
    for row in rows:
        c = 0
        for b in end[row:row+k]:
            c = 2*c+b
        contexts.append(c)
    assert contexts == [0] + list(range(1 << k))
    assert column[:2] == [1, 0] and lf[0] == event_post and lf[p] == primary_post
    assert set(column[:2]) == {0, 1}
    # Tie chooses 0 in the repeated context. All remaining contexts are singleton.
    elapsed = 0
    x = 0
    while x != p:
        x = lf[x]
        elapsed += 1
    return elapsed


def physical(step):
    # Begin with the raw nonzero orbit. Its source 1 originally preceded T(1).
    # Splicing replaces that arc by 1 -> f -> e -> T(1).
    k = (len(step)-1).bit_length()
    return [1, 0, 0] + [step[x] >> (k-1) for x in range(2, len(step))]


def check(k):
    mask, step, z = lfsr(k)
    N, q = len(z), len(step)
    position = {x: j for j, x in enumerate(z)}
    halfway = N//2
    primary = z[halfway]
    borders = [N*t//4 for t in range(5)]

    def chords(first, second):
        start, end = borders[first:first+2]
        lo, hi = borders[second:second+2]
        return [x for x in z[start:end] if x^1 in position
                and lo <= position[x^1] < hi and {x, x^1}.isdisjoint((1, primary))]

    left, right = chords(0, 2), chords(1, 3)
    matches = list(zip(left, right))
    checked = 0
    if k <= 10:
        col = physical(step)
        p = primary+1
        _, _, lf = index(col)
        before = original(col, k, p, lf[0], lf[p])
        assert before == 1+halfway
    for x, y in matches:
        change = position[y^1]-position[x^1]-position[y]+position[x]
        assert change != 0
        if k <= 10:
            new = step.copy()
            new[x], new[x^1] = new[x^1], new[x]
            new[y], new[y^1] = new[y^1], new[y]
            other = physical(new)
            changed = {i for i, (a,b) in enumerate(zip(col,other)) if a != b}
            assert changed == {u+1 for u in (x, x^1, y, y^1)}
            assert original(other, k, p, lf[0], lf[p]) == before+change
            checked += 1
    return dict(k=k, K=q, n=q+1, primitive_feedback_mask=hex(mask),
                distinct_q1_to_q3=len(left), distinct_q2_to_q4=len(right),
                disjoint_switch_witnesses=len(matches), literal_switched_BWTs=checked)


if __name__ == '__main__':
    rows = [check(k) for k in range(3, 17)]
    out = csv.DictWriter(sys.stdout, rows[0], lineterminator='\n')
    out.writeheader()
    out.writerows(rows)
