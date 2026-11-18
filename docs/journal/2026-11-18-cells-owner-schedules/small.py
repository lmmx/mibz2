#!/usr/bin/env python3
"""Enumerate every ownership tree on small full-period Cells columns."""
import itertools
from collections import Counter

from first_visit import walk
from schedules import Grammar, locality_limits, measure, optimal, study5


def family(g, expected):
    bounds = locality_limits(g)
    walk(g, expected)
    choices = (edges or [-1] for edges in g.incoming)
    scores = []
    for owner in itertools.product(*choices):
        # For prescribed owners, measure also computes depth by intersecting actual
        # command source intervals with actual destination holes.
        result = measure(g, "given", expected, owner)
        assert result["same_page"] <= bounds["page_bound"]
        assert result["same_line"] <= bounds["line_bound"]
        scores.append((result["rounds"], result["span_bytes"], result["distance"]))
    assert optimal(g, False)[1] == min(x[0] for x in scores)
    assert optimal(g, True)[1] == min(x[1] for x in scores)
    return scores


def main():
    for limit in (1, 16):
        counts = Counter()
        for n in range(1, 10):
            for column in itertools.product((0, 1), repeat=n):
                for primary in range(n):
                    (arena, rules, root, _), expected = study5.tiny(column, primary, limit)
                    counts["columns_with_primary"] += 1
                    if len(expected) != n:
                        continue
                    counts["full"] += 1
                    scores = family(Grammar(arena, rules, root), expected)
                    counts["trees"] += len(scores)
                    counts["nontrivial"] += len(scores) > 1
        print(f"cutoff={limit}", dict(counts))
    for n in (4, 5, 10, 17, 35, 71):
        (arena, rules, root, _), expected = study5.tiny((1,) * (n - 1) + (0,), n - 2, 16)
        g = Grammar(arena, rules, root)
        row, _ = walk(g, expected)
        assert len(expected) == n and len(rules) == n - 1
        assert row["copies"] == 0 and row["stack_peak"] == row["pushes"] == n - 2
        assert row["fields"] == 5 * n - 7 > 4 * (n - 1)
    print("marked left comb: peak n-2 and negative field balance at all six native-cutoff sizes")
    # Real cutoff, two-run primitive columns, primary indexed from zero.
    for n, ones, primary in ((52, 3, 3), (71, 4, 3), (88, 5, 4)):
        column = (1,) * ones + (0,) * (n - ones)
        (arena, rules, root, _), expected = study5.tiny(column, primary, 16)
        assert len(expected) == n
        g = Grammar(arena, rules, root)
        scores = family(g, expected)
        last = measure(g, "last", expected)
        median = measure(g, "median", expected)
        best = tuple(map(min, zip(*scores)))
        print(f"n={n} ones={ones} primary={primary} trees={len(scores)} "
              f"copies={last['copies']} bytes={last['copy_bytes']} "
              f"last={(last['rounds'], last['span_bytes'], last['distance'])} "
              f"coordinatewise_minima={best} median_distance={median['distance']}")
        if n == 52:
            assert (last["distance"], median["distance"]) == (51, 34)
        if n == 71:
            assert (last["rounds"], best[0], last["span_bytes"], best[1]) == (2, 1, 35, 18)
        if n == 88:
            assert (median["distance"], best[2]) == (88, 70)


if __name__ == "__main__":
    main()
