"""Independent exact fixed-cylinder, positive-first-hit and marked-section checks."""
from array import array
from bisect import bisect_left
from collections import Counter
from hashlib import sha256
from itertools import product
from pathlib import Path
import sys


P = bytes.fromhex("223a22")
C = bytes.fromhex("222c2270726f7065727479")
A = bytes.fromhex("223a2276616c7565")
QUOTE = b'"'
COMMA = b','


class Column:
    def __init__(self, last):
        self.n = len(last)
        self.pos = [array("I") for _ in range(256)]
        for row, symbol in enumerate(last):
            self.pos[symbol].append(row)
        self.psi = array("I")
        self.base = []
        for rows in self.pos:
            self.base.append(len(self.psi))
            self.psi.extend(rows)
        self.first = bytes(last[row] for row in self.psi)

    def interval(self, word):
        low, high = 0, self.n
        for symbol in reversed(word):
            rows, base = self.pos[symbol], self.base[symbol]
            low, high = base + bisect_left(rows, low), base + bisect_left(rows, high)
        return range(low, high)

    def first_hit(self, word, target):
        """Prepend via stable occurrence numbers, allowing a start in target."""
        current = target
        for depth, symbol in enumerate(reversed(word)):
            if depth:
                current = current - target
            current = {self.base[symbol] + ordinal
                       for ordinal, row in enumerate(self.pos[symbol]) if row in current}
        return current

    def walk(self, row, length):
        emitted = bytearray()
        for _ in range(length):
            emitted.append(self.first[row])
            row = self.psi[row]
        return bytes(emitted), row


def explicit_first_hit(col, word, target):
    starts = set()
    for start in range(col.n):
        pos = start
        for i, symbol in enumerate(word):
            if col.first[pos] != symbol or (i > 0 and pos in target):
                break
            pos = col.psi[pos]
        else:
            if pos in target:
                starts.add(start)
    return starts


def marked_check(col, section, stats):
    # Read one edge per section row. Test both partial edge emission and all marks
    # outside covered cycles against an independent, byte-at-a-time forward walk.
    edges = {}
    for row in section:
        data = bytearray([col.first[row]])
        nxt = col.psi[row]
        while nxt not in section:
            data.append(col.first[nxt])
            nxt = col.psi[nxt]
        edges[row] = bytes(data), nxt
    for start in range(col.n):
        row = start
        got = bytearray()
        while row not in section:
            got.append(col.first[row])
            row = col.psi[row]
            if row == start:
                period = len(got)
                stats["uncovered_marks"] += 1
                got *= (col.n + period - 1) // period
                break
        else:
            stats["covered_marks"] += 1
            while len(got) < col.n:
                data, row = edges[row]
                got.extend(data)
        assert bytes(got[:col.n]) == col.walk(start, col.n)[0]


def fixed_check(col, landing, payload, stats):
    target = set(col.interval(landing))
    cut = set(col.interval(payload + landing))
    starts = explicit_first_hit(col, payload, target)
    assert starts == col.first_hit(payload, target)
    if any((payload + landing).startswith(landing, j) for j in range(1, len(payload))):
        stats["early_word_cases"] += 1
        stats["early_with_literal"] += bool(cut)
        assert not starts
    else:
        assert starts == cut
    return starts


def tiny():
    stats = Counter()
    words = [bytes(w) for n in range(1, 4) for w in product(range(2), repeat=n)]
    for n in range(1, 8):
        for last in product(range(2), repeat=n):
            col = Column(last)
            for landing in words:
                observed = {r for r in range(n) if col.walk(r, len(landing))[0] == landing}
                assert observed == set(col.interval(landing))
                for payload in words:
                    fixed_check(col, landing, payload, stats)
                    stats["pattern_pairs"] += 1
                marked_check(col, set(col.interval(landing)), stats)
            stats["columns"] += 1
    col = Column(bytes([1, 0, 0]))  # marked orbit spells periodic 001
    assert list(col.interval(bytes([1, 0, 0]))) == [2]
    assert not explicit_first_hit(col, bytes([1, 0]), set(col.interval(bytes([0]))))
    # One offset-zero occurrence and an overlap across the payload/landing seam.
    col = Column(b"aaa")
    assert fixed_check(col, b"aaa", b"a", Counter()) == set(range(3))
    assert not fixed_check(col, b"aaa", b"aa", Counter())
    print("tiny", dict(sorted(stats.items())))


def cyclic_bwt(text):
    rotations = sorted(range(len(text)), key=lambda i: text[i:] + text[:i])
    return bytes(text[i-1] for i in rotations)


def shifted(col, rows, steps):
    return {col.walk(row, steps)[1] for row in rows}


def shape(rows):
    return len(rows), sum(row - 1 not in rows for row in rows)


def geometry(col):
    table = {}
    for name, pattern in (("quote", QUOTE), ("quote-colon-quote", P),
                          ("bare-comma", COMMA), ("J1", COMMA + QUOTE),
                          ("old-comma-token", QUOTE + COMMA + QUOTE),
                          ("bare-C", C), ("J2", C + P), ("third", A + C + P)):
        span = col.interval(pattern)
        table[name] = span
        print(name, span.start, span.stop, len(span))
    for name, depth, dest in (("old-comma-token", 1, "J1"), ("third", len(A), "J2")):
        image = shifted(col, table[name], depth)
        assert image <= set(table[dest])
        print(name, "shift", depth, "image_mass_components", shape(image),
              "missing", len(table[dest]) - len(image))
    return table


def named():
    # The rank set has two copies of the same nonconstant orbit plus a fixed point.
    raw = A + C + P + b"|" + C + P + b"|" + C + b'!,"'
    doubled = b"".join(bytes([x, x]) for x in cyclic_bwt(raw))
    col = Column(b"\0" + doubled)
    table = geometry(col)
    stats = Counter()
    for name, landing, payload in (("J1", QUOTE, COMMA), ("J2", P, C)):
        cut = fixed_check(col, landing, payload, stats)
        assert cut == set(table[name])
        marked_check(col, cut, stats)
    assert len(table["third"]) == 2 < len(table["J2"]) == 4 < len(table["bare-C"]) == 6
    assert 0 < stats["uncovered_marks"] < stats["covered_marks"]
    for raw, missing in ((b"x", (QUOTE, P)), (P, (C + P, COMMA + QUOTE))):
        other = Column(cyclic_bwt(raw))
        for pattern in missing:
            assert not other.interval(pattern)
            marked_check(other, set(), stats)
    print("named", dict(sorted(stats.items())))


def frozen(path):
    repo = next(p for p in Path(__file__).resolve().parents if (p / "Cargo.toml").exists())
    sys.path.insert(0, str(repo / "docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import read_input
    runs, primary = read_input(path)
    last = b"".join(bytes([symbol]) * width for symbol, width in runs)
    col = Column(last)
    print("frozen dense L", sha256(last).hexdigest(), "n", col.n, "p", primary)
    table = geometry(col)
    for name, landing, payload in (("J1", QUOTE, COMMA), ("J2", P, C)):
        target = set(col.interval(landing))
        assert col.first_hit(payload, target) == set(table[name])
        assert explicit_first_hit(col, payload, target) == set(table[name])


if __name__ == "__main__":
    tiny()
    named()
    if len(sys.argv) == 2:
        frozen(Path(sys.argv[1]))
