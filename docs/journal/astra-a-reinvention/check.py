"""Exhaust arbitrary columns. Independently certify every published word and retirement."""
import itertools
import json
from collections import Counter

from shortcut import capped, reconstruct as split_walk
from whole import reconstruct as whole_walk


def check(column, p, mode, witnesses, totals):
    psi = sorted(range(len(column)), key=column.__getitem__)
    n = len(column)
    # F byte and translation distinguish the original Psi atoms.
    atoms = [(column[t], t-i) for i,t in enumerate(psi)]
    rows = [p]
    for _ in range(n):
        rows.append(psi[rows[-1]])
    expected = bytes(column[r] for r in rows[1:])
    period = rows[1:].index(p) + 1

    def evaluate(start, length):
        letters = bytearray()
        end = start
        for _ in range(length):
            end = psi[end]
            letters.append(column[end])
        return bytes(letters), end

    def verify(node):
        l, h, d, m, _ = node
        word = evaluate(l, m)[0]
        for start in range(l, h):
            assert evaluate(start, m) == (word, start+d)
        return word

    def audit(pos, row, node, take):
        assert rows[pos] == row
        assert verify(node)[:take] == expected[pos:pos+take]

    def publish(node, pos, install):
        l, h, d, m, _ = node
        assert verify(node) == expected[pos:pos+m]
        if install:
            totals["certificates_checked"] += 1
            return
        totals["retirements_checked"] += 1
        next_atoms = atoms[l+d:h+d]
        assert len(set(next_atoms)) > 1
        if len({v[0] for v in next_atoms}) == 1:
            witnesses.setdefault("same_byte_different_exit",
                dict(column=column.hex(), primary=p, source=[l,h], shift=d, length=m,
                     next_atoms=next_atoms))

    run = whole_walk if mode == "whole" else split_walk
    out, st = run(capped(column), n, p, audit=audit, publish=publish)
    assert out == expected
    totals["cases"] += 1
    totals["short_cycle"] += period < n
    totals["nondividing_cycle"] += n % period != 0
    if period == n and mode == "whole":
        assert st["unit_visits"] == st["roots"]-st["nodes"]
    for key in ("clipped", "splits", "aliases", "unit_offers", "copies", "whole", "installs", "retired"):
        if st[key]:
            totals[key] += 1
            witnesses.setdefault(key, dict(column=column.hex(), primary=p, period=period,
                                           output=out.hex(), count=st[key]))


if __name__ == "__main__":
    results = {}
    for mode in ("split", "whole"):
        totals, witnesses = Counter(), {}
        for alphabet, maxlen in [(2, 9), (3, 6)]:
            for length in range(1, maxlen+1):
                for word in itertools.product(range(alphabet), repeat=length):
                    for primary in range(length):
                        check(bytes(word), primary, mode, witnesses, totals)
        # Capped boundaries on either side of an exact multiple; arbitrary rejected-stream
        # columns also exercise the n-step convention independently of period divisibility.
        cap_checks = Counter()
        for length in (126, 127, 128, 253, 254, 255):
            word = b'\x01'+b'\x00'*length+b'\x01'
            for p in (0,1,length//2,length,length+1):
                check(word, p, mode, {}, cap_checks)
        results[mode] = dict(totals=totals, capped_cases=cap_checks, witnesses=witnesses)
    print(json.dumps(results, indent=2))
