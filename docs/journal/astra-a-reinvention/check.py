"""Exhaustive falsifier on arbitrary last columns. Independently verify each claimed interval."""
import itertools
import json
from collections import Counter

from shortcut import capped, reconstruct


def check(column, p, witnesses, totals):
    # Stable argsort of L is Psi. This oracle has no run shortcuts or learned dictionary.
    psi = sorted(range(len(column)), key=column.__getitem__)
    n = len(column)
    rows = [p]
    for _ in range(n):
        rows.append(psi[rows[-1]])
    expected = bytes(column[r] for r in rows[1:])
    period = rows[1:].index(p) + 1
    def audit(pos, row, node, take):
        l, h, d, m, _ = node
        assert rows[pos] == row
        word = None
        for start in range(l, h):
            end, letters = start, bytearray()
            for _ in range(m):
                end = psi[end]
                letters.append(column[end])
            assert end == start+d
            if word is None:
                word = letters
            assert word == letters
        assert word[:take] == expected[pos:pos+take]
    out, st = reconstruct(capped(column), n, p, audit=audit)
    assert out == expected
    totals["cases"] += 1
    totals["short_cycle"] += period < n
    totals["nondividing_cycle"] += n % period != 0
    for key in ("clipped", "splits", "aliases", "unit_offers", "copies", "whole"):
        if st[key]:
            totals[key] += 1
            witnesses.setdefault(key, dict(column=column.hex(), primary=p, period=period,
                                           output=out.hex(), count=st[key]))


if __name__ == "__main__":
    totals, witnesses = Counter(), {}
    for alphabet, maxlen in [(2, 9), (3, 6)]:
        for length in range(1, maxlen+1):
            for word in itertools.product(range(alphabet), repeat=length):
                for primary in range(length):
                    check(bytes(word), primary, witnesses, totals)
    print(json.dumps({"totals": dict(totals), "witnesses": witnesses}, indent=2))
