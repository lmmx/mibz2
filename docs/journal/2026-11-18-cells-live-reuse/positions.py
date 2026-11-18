#!/usr/bin/env python3
"""Audit destination and copy schedules against externally supplied, already decoded cycle bytes."""
import argparse
import array
import csv
import itertools
import sys
from collections import Counter
from pathlib import Path

from census import irec

MARK = 1 << 31
LENGTH = MARK - 1
BASE = 1 << 30  # model address at which long-rule records begin
NONE = -1


def ref(w):
    return w >> 32


def size(w):
    return w & LENGTH


def grammar(byte, marked, sa, sb, kinds, final, small=16):
    # Transcribe the existing immutable arena and rule array from the trace. The
    # schedule below receives just these, the root and the block length.
    arena = bytearray(range(256))
    words = [(c << 32) | 1 | (MARK if x == marked else 0) for x, c in enumerate(byte)]
    rules, cut = [], []
    for a, b, kind in zip(sa, sb, kinds):
        left, right = words[a], words[b]
        length = (left & 0xffffffff) + (right & 0xffffffff)
        if length <= small:
            at = len(arena)
            arena.extend(arena[ref(left):ref(left) + size(left)])
            arena.extend(arena[ref(right):ref(right) + size(right)])
        else:
            at = BASE + len(rules)
            rules.append((left, right))
            cut.append(kind)
        words.append(at << 32 | length)
    assert len(arena) < BASE
    return arena, rules, words[final], cut


def origin(rules, root):
    mark, w = 0, root
    while ref(w) >= BASE:
        a, b = rules[ref(w) - BASE]
        if a & MARK:
            w = a
        else:
            mark += size(a)
            w = b
    assert w & MARK and size(w) == 1
    return (-(mark + 1)) % size(root)


class Audit:
    """An oracle and a byte-coverage bitmap. No candidate uses them to choose positions."""
    def __init__(self, expected):
        self.expected = expected
        self.filled = bytearray(len(expected))
        self.counts = Counter()
        self.earliest_later_copy = None

    def occupy(self, d, length):
        assert 0 <= d and 0 < length and d + length <= len(self.expected)
        assert self.filled.find(b'\1', d, d + length) == -1, "overlapping destinations"
        self.filled[d:d + length] = b'\1' * length

    def literal(self, arena, w, d):
        length = size(w)
        self.occupy(d, length)
        assert self.expected[d:d + length] == arena[ref(w):ref(w) + length]
        if w & MARK:
            assert d == len(self.expected) - 1 and length == 1
        self.counts["literal_edges"] += 1
        self.counts["literal_bytes"] += length

    def copy(self, w, d, source):
        length = size(w)
        self.occupy(d, length)
        assert 0 <= source and source + length <= len(self.expected)
        assert source + length <= d or d + length <= source
        assert self.filled.find(b'\0', source, source + length) == -1, "unfinished copy source"
        assert self.expected[d:d + length] == self.expected[source:source + length]
        self.counts["copies"] += 1
        self.counts["copy_bytes"] += length
        if source > d:
            self.counts["from_later"] += 1
            if self.earliest_later_copy is None:
                self.earliest_later_copy = (d, source, length)

    def finish(self):
        assert self.filled.find(b'\0') == -1


def general(arena, rules, root, qroot, audit, least):
    """Baseline min and overwrite-on-reach. Include marked rules in cyclic coordinates."""
    p = size(root)
    if ref(root) < BASE:
        audit.literal(arena, root, 0)
        return
    pos = [NONE] * len(rules)
    pos[ref(root) - BASE] = qroot
    for j in reversed(range(len(rules))):
        q = pos[j]
        if q == NONE:
            continue
        a, b = rules[j]
        for w, d in ((a, q), (b, (q + size(a)) % p)):
            k = ref(w) - BASE
            if k >= 0:
                if not least or pos[k] == NONE or d < pos[k]:
                    pos[k] = d
    for j, (a, b) in enumerate(rules):
        q = pos[j]
        if q == NONE:
            continue
        for w, d in ((a, q), (b, (q + size(a)) % p)):
            k = ref(w) - BASE
            if k < 0:
                audit.literal(arena, w, d)
            elif pos[k] != d:
                audit.copy(w, d, pos[k])


def owners(arena, rules, root, qroot, audit, cut_truth=None):
    """Full-period specialization. Reverse reads and forward commands share the rule array."""
    p = size(root)
    if ref(root) < BASE:
        audit.literal(arena, root, 0)
        return
    nr = len(rules)
    assert ref(root) == BASE + nr - 1
    work = rules.copy()  # the harness retains an untouched baseline
    pos = [NONE] * nr  # poison for assertions, only the root needs initialization
    pos[nr - 1] = qroot
    tail = nr
    next_a = next_b = NONE
    for j in reversed(range(nr)):
        assert j < tail
        a, b = work[j]
        q = pos[j]
        assert q != NONE, "missing owner in the marked component"
        right = q + size(a)
        if right >= p:
            right -= p
            audit.counts["wraps"] += 1
        assert (right < q) == bool(a & MARK)
        old_tail = tail
        for side, (w, later, d) in enumerate(((a, next_a, q), (b, next_b, right))):
            k = ref(w) - BASE
            if k < 0:
                audit.literal(arena, w, d)
                continue
            survives = w == later
            if cut_truth is not None:  # independent check, never supplies the decision
                assert survives == (cut_truth[j] == (irec.TOP_WINS, irec.BOTTOM_WINS)[side])
            if survives:
                tail -= 1
                assert j <= tail
                work[tail] = w, d
            else:
                assert pos[k] == NONE
                pos[k] = d
        assert old_tail - tail <= 1
        next_a, next_b = a, b
    assert NONE not in pos
    for i in range(tail, nr):
        w, d = work[i]
        audit.copy(w, d, pos[ref(w) - BASE])
    assert audit.counts["copies"] == nr - tail


def check(arena, rules, root, cut, n, expected, check_overwrite=False):
    assert size(root) == len(expected)
    q = origin(rules, root)
    audit = Audit(expected)
    if n == size(root):
        owners(arena, rules, root, q, audit, cut)
    else:
        general(arena, rules, root, q, audit, least=False)
    audit.finish()
    baseline = Audit(expected)
    general(arena, rules, root, q, baseline, least=True)
    baseline.finish()
    assert baseline.counts["from_later"] == 0
    matches = [audit]
    if check_overwrite and n == size(root):
        alternative = Audit(expected)
        general(arena, rules, root, q, alternative, least=False)
        alternative.finish()
        matches.append(alternative)
    for schedule in matches:
        for key in ("copies", "copy_bytes", "literal_edges", "literal_bytes"):
            assert schedule.counts[key] == baseline.counts[key]
    return audit


def check_lf(rec, expected):
    column = b"".join(bytes([c]) * w for c, w in zip(rec.a["l_byte"], rec.a["l_lam"]))
    count = [0] * 256
    for c in column:
        count[c] += 1
    next_row = [0, *itertools.accumulate(count[:-1])]
    inverse = array.array("I", [0]) * len(column)
    for i, c in enumerate(column):
        inverse[next_row[c]] = i
        next_row[c] += 1
    row = rec.p
    for i, c in enumerate(expected):
        row = inverse[row]
        assert column[row] == c and ((row == rec.p) == (i + 1 == len(expected)))


def recorded(rec, lf=False, overwrite=False):
    a = rec.a
    ar, rules, root, cut = grammar(a["l_byte"], rec.row_letter, a["s_a"], a["s_b"], a["s_type"], rec.final)
    # Independent marked location and cyclic bytes from irec/1, rather than the candidate positions.
    word, x, shift = rec.word(rec.final), rec.final, 1
    while x >= rec.r:
        l, r = rec.children(x)
        if a["w_kind"][l] & irec.MARKED:
            x = l
        else:
            shift += a["w_len"][l]
            x = r
    assert x == rec.row_letter
    shift %= len(word)
    expected = word[shift:] + word[:shift]
    if lf:
        check_lf(rec, expected)
    audit = check(ar, rules, root, cut, rec.n, expected, check_overwrite=overwrite)
    keys = ("copies", "copy_bytes", "literal_edges", "literal_bytes", "wraps", "from_later")
    return dict(corpus=rec.corpus, block=rec.block, full=int(rec.n == rec.period),
                **{k: audit.counts[k] for k in keys}, example=audit.earliest_later_copy)


def tiny(column, p, small):
    """Independent scalar induction in two explicit orders, for arbitrary small columns."""
    byte, width, marked = [], [], None
    for i, c in enumerate(column):
        if i in (p, p + 1) or not byte or c != byte[-1]:
            if i == p:
                marked = len(byte)
            byte.append(c)
            width.append(0)
        width[-1] += 1
    top = list(range(len(byte)))
    bottom = sorted(top, key=byte.__getitem__)
    holds_mark = [x == marked for x in top]
    aa, bb, types = [], [], []
    while True:
        a, b = top[-1], bottom[-1]
        if a == b:
            if holds_mark[a]:
                break
            top.pop()
            bottom.pop()
            continue
        t = irec.EQUAL if width[a] == width[b] else irec.TOP_WINS if width[a] > width[b] else irec.BOTTOM_WINS
        x = len(width)
        w = min(width[a], width[b])
        width.append(w)
        holds_mark.append(holds_mark[a] or holds_mark[b])
        aa.append(a)
        bb.append(b)
        types.append(t)
        top.insert(top.index(b) + 1, x)
        bottom.insert(bottom.index(a) + 1, x)
        for c in (a, b):
            width[c] -= w
            if not width[c]:
                top.remove(c)
                bottom.remove(c)
    inverse = sorted(range(len(column)), key=column.__getitem__)
    row, expected = p, bytearray()
    while True:
        row = inverse[row]
        expected.append(column[row])
        if row == p:
            break
    return grammar(byte, marked, aa, bb, types, a, small), bytes(expected)


def exhaustive():
    counts, witness = Counter(), {}
    for alphabet, bound in ((2, 9), (3, 6)):
        for n in range(1, bound + 1):
            for col in itertools.product(range(alphabet), repeat=n):
                for p in range(n):
                    for limit in (16, 1):
                        g, expected = tiny(col, p, limit)
                        audit = check(*g, n, expected, check_overwrite=True)
                        counts[limit, "total"] += 1
                        counts[limit, "full"] += len(expected) == n
                        counts[limit, "copies"] += audit.counts["copies"]
                        if audit.earliest_later_copy and "later" not in witness:
                            witness["later"] = (col, p, limit, audit.earliest_later_copy)
                        if len(expected) != n and "missing" not in witness and ref(g[2]) >= BASE:
                            try:
                                owners(*g[:3], origin(g[1], g[2]), Audit(expected), g[3])
                            except AssertionError:
                                witness["missing"] = (col, p, limit)
    # Two literal-run columns exercise the actual 16-byte cutover.
    g, expected = tiny((1, 1) + (0,) * 33, 2, 16)
    assert len(expected) == 35
    assert check(*g, 35, expected).earliest_later_copy == (0, 17, 17)
    g, expected = tiny((1, 1) + (0,) * 34, 3, 16)
    assert len(expected) == 18 and ref(g[2]) == BASE + 1
    assert ref(g[1][1][1]) == ref(g[1][2][1]) == BASE
    assert g[3][1] == irec.BOTTOM_WINS and g[3][2] == irec.EQUAL
    check(*g, 36, expected)
    return counts, witness


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("records", nargs="*")
    parser.add_argument("--small", action="store_true")
    parser.add_argument("--lf", action="store_true", help="also compare to direct stable-sort inversion")
    parser.add_argument("--overwrite", action="store_true", help="also test universal overwrite on whole orbits")
    args = parser.parse_args()
    if args.small:
        print(exhaustive())
    writer = None
    for path in (p for arg in args.records for p in irec.paths(arg)):
        row = recorded(irec.load(path), args.lf, args.overwrite)
        if writer is None:
            writer = csv.DictWriter(sys.stdout, fieldnames=row.keys(), lineterminator="\n")
            writer.writeheader()
        writer.writerow(row)


if __name__ == "__main__":
    main()
