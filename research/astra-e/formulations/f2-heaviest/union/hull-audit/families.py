"""Construct byte columns with sharp best/worst occupied-run costs over multiple colours."""

import json
from check import discover, geometry, ranges


def column(cycles):
    # Compare infinite cyclic byte sequences. A prefix of length p+q is
    # sufficient to distinguish two different periodic strings of periods p,q.
    width = 2 * max(map(len, cycles))
    rotations = []
    for i, w in enumerate(cycles):
        for k in range(len(w)):
            v = w[k:] + w[:k]
            rotations.append(((v * ((width + len(w) - 1) // len(w)))[:width], i, k, w[k - 1]))
    ordered = sorted(rotations)
    return tuple(item[-1] for item in ordered), [(item[1], item[2]) for item in ordered]


def binary_id(j, width):
    return bytes(1 + ((j >> k) & 1) for k in reversed(range(width)))


def evaluate(cycles, chosen):
    l, original = column(cycles)
    first, psi, _, _, _ = geometry(l)
    # Every generated cycle is primitive. Equal words retain copy order; the rotation
    # oracle recovers precisely its intended successor, not just a spelling.
    for x, (i, k) in enumerate(original):
        assert original[psi[x]] == (i, (k + 1) % len(cycles[i]))
    selected = [x for x, (i, k) in enumerate(original) if k == 0 and cycles[i] in chosen]
    roots = ranges(selected)
    stats, _, _ = discover(l, b"\0", chosen, roots)
    return dict(components=len(roots), column_length=len(l), **stats)


def spine(h, depth, obstacles, copies=1):
    bits = (2 * h - 2).bit_length()
    cycles = []
    chosen = set()
    for j in range(2 * h - 1):
        head = b"\0" + binary_id(j, bits)
        if j % 2 == 0 or not obstacles:
            w = head + b"\3" * depth
            cycles.append(w)
            if j % 2 == 0:
                chosen.add(w)
                cycles.extend([w] * (copies - 1))
        else:
            cycles.extend(head + b"\4" + b"\3" * k for k in range(depth))
    if copies > 1:
        assert min(cycles.count(w) for w in chosen) > max(cycles.count(w) for w in set(cycles) - chosen)
    row = evaluate(cycles, chosen)
    assert row["components"] == h
    assert row["M"] == h * (depth + bits + 1)
    if obstacles:
        # All of the h-1 interior blockers are lost at each unary step.
        assert row["unary_hidden"] >= depth * (h - 1)
        assert row["unary_hidden"] == row["root_holes"] == depth * (h - 1)
    else:
        assert row["M"] - row["J"] >= depth * (h - 1)
    return dict(depth=depth, selected_copies=copies, **row)


def gap(count):
    bits = (count - 1).bit_length()
    cycles = [b"\0" + binary_id(j, bits) + bytes([3 + j % 2]) for j in range(count)]
    l, order = column(cycles)
    roots = [x for x, (idx, k) in enumerate(order) if k == 0 and idx in (0, count - 1)]
    assert len(roots) == 2 and l[roots[0]] == l[roots[1]]
    hull = l[roots[0] : roots[-1] + 1]
    whole = 1 + sum(a != b for a, b in zip(hull, hull[1:]))
    assert whole == count
    return dict(parent_types=count, byte_rows=len(l), hull_root_runs=whole, occupied_root_runs=2)


if __name__ == "__main__":
    print(
        json.dumps(
            dict(
                shared=[spine(h, d, False) for h in [2, 4, 8] for d in [4, 16, 64]],
                separated=[spine(h, d, True) for h in [2, 4, 8] for d in [4, 16, 64]],
                strict_frequency_equal_rotations=[
                    spine(h, 16, obstacles, 2) for h in [2, 4, 8] for obstacles in [False, True]
                ],
                naive_hull=[gap(n) for n in [3, 5, 17, 65, 257]],
            ),
            indent=2,
            sort_keys=True,
        )
    )
