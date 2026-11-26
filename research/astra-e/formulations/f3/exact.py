"""Exact all-mark tests of an ordered binary alphabet split, including arbitrary columns."""
from itertools import product
import json


def successor(l):
    # Original stable order breaks ties. The list is Psi on F ranks.
    return sorted(range(len(l)), key=l.__getitem__)


def word(labels, step, row, n):
    result = []
    for _ in range(n):
        result.append(labels[row])
        row = step[row]
    return result


def bwt(t):
    rotations = sorted(range(len(t)), key=lambda i: t[i:] + t[:i])
    return [t[i - 1] for i in rotations], rotations.index(0)


def decode(l, row):
    return word(sorted(l), successor(l), row, len(l))


def witnesses():
    result = {}
    for name, t in [('child', [0, 1, 2]), ('skeleton', [1, 0, 2])]:
        l, p = bwt(t)
        candidate = [x for x in l if x < 2] if name == 'child' else [int(x >= 2) for x in l]
        want = [x for x in t if x < 2] if name == 'child' else [int(x >= 2) for x in t]
        possible = [decode(candidate, q) for q in range(len(candidate))]
        assert want not in possible and decode(l, p) == t
        result[name] = dict(t=t, l=l, p=p, projected=want, true_bwt=bwt(want),
                            candidate=candidate, candidate_outputs=possible)
    return result


def main():
    columns = marks = cycle_columns = class_preserved = 0
    multi_witness = None
    for n in range(1, 7):
        for l in product(range(3), repeat=n):
            colours = [int(x >= 2) for x in l]
            labels = sorted(colours)
            full, skeleton = successor(l), successor(colours)
            m = labels.count(0)
            child0 = [x for x in l if x < 2]
            child1 = [x for x in l if x >= 2]
            local = successor(child0) + [m + x for x in successor(child1)]
            assert full == [skeleton[j] for j in local]
            assert sum(a != b for a, b in zip(full, skeleton)) == sum(i != j for i, j in enumerate(local))
            colours_full = [word(labels, full, p, n) for p in range(n)]
            colours_skeleton = [word(labels, skeleton, p, n) for p in range(n)]
            compatible = all(colours_skeleton[p] == colours_skeleton[local[p]] for p in range(n))
            assert compatible == (colours_skeleton == colours_full)
            orbit = {0}
            q = full[0]
            while q not in orbit:
                orbit.add(q)
                q = full[q]
            if len(orbit) < n:
                cycle_columns += 1
                for p in range(n):
                    if multi_witness is None and colours_full[p] not in colours_skeleton:
                        multi_witness = dict(l=l, p=p, marked=decode(l, p), wanted=colours_full[p],
                                             plain_skeleton=colours, possible=colours_skeleton)
            hist = [l.count(a) for a in range(3)]
            prefix = [l[:m].count(a) for a in range(3)]
            mistakes = 0
            for group in ([0, 1], [2]):
                end_zero = sum(prefix[c] for c in group)
                start = common = 0
                for c in group:
                    common += max(0, min(end_zero, start + prefix[c]) - start)
                    start += hist[c]
                mistakes += 2 * (end_zero - common)
            assert mistakes == sum(labels[full[p]] != labels[skeleton[p]] for p in range(n))
            columns += 1
            marks += n
            class_preserved += compatible
    print(json.dumps(dict(columns=columns, marks=marks, multi_cycle_columns=cycle_columns,
                          all_mark_colour_codings_preserved=class_preserved, witnesses=witnesses(),
                          multiple_cycle_witness=multi_witness), indent=2))


if __name__ == '__main__':
    main()
