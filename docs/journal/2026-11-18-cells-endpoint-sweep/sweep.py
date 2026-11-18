#!/usr/bin/env python3
"""Width-free structural sweep. Output is a cut/closure log, never decoded block bytes."""

import argparse
import csv
import sys
from dataclasses import dataclass
from itertools import groupby
from pathlib import Path

from census import SIDE, irec, roots

MARK = 1 << 31
LOW = 0xFFFFFFFF
NONE = -1
# Pinned arena-relative addresses. The same helper for both cuts uses fixed, disjoint
# base addresses so exact W16 offsets, marks and lengths can be compared.
ARENA = 4096
RULES = 1 << 28


class Words:
    """Model W16 allocation offsets and ordered rule pairs using disjoint region bases."""

    def __init__(self):
        self.tail = ARENA + 16 + 256 + 16
        self.rule = RULES
        self.pairs = []

    @staticmethod
    def literal(c, marked):
        """Construct the pinned one-byte descriptor, including its optional mark."""
        return ((ARENA + 16 + c) << 32) | 1 | (MARK if marked else 0)

    def join(self, left, right):
        """Advance the short cursor or store the ordered pair and advance the rule cursor."""
        size = (left & LOW) + (right & LOW)
        if size <= 16:
            at = self.tail
            self.tail += size
        else:
            at = self.rule
            self.rule += 16
            self.pairs.append((left, right))
        return (at << 32) | size


@dataclass(slots=True)
class Packet:
    """One first arrival with a displacement magnitude, word and observer-only name."""

    distance: int
    word: int
    name: int  # audit only


class Blocks:
    """One 16-byte reusable packet per original and one bucket head per machine word of rows."""

    def __init__(self, n, r):
        self.head = [NONE] * ((n + SIDE - 1) // SIDE)
        self.packet = [None] * r
        self.route = [0] * r
        self.page = len(self.head)
        self.mask = [0, 0]
        self.local = [NONE] * (2 * SIDE)
        self.remote = self.near = self.drained = self.pages = self.reads = 0
        self.peak_bucket = 0

    def activate(self, code, idx):
        """Install one valid payload index in the current group."""
        side, b = divmod(code, SIDE)
        bit = 1 << b
        assert not self.mask[side] & bit
        self.mask[side] |= bit
        self.local[code] = idx

    def post(self, q, side, idx, item):
        """File a first arrival in its local mask or future group chain."""
        assert q > 0
        page, b = divmod(q - 1, SIDE)
        code = side * SIDE + b
        self.packet[idx] = item
        if page == self.page:
            self.near += 1
            self.activate(code, idx)
        else:
            assert page < self.page
            self.remote += 1
            self.route[idx] = ((self.head[page] + 1) << 7) | code
            self.head[page] = idx

    def next(self):
        """Return the highest remaining frontier and its available front indices."""
        while not (self.mask[0] | self.mask[1]):
            self.page -= 1
            assert self.page >= 0, "no marked closure"
            self.pages += 1
            i = self.head[self.page]
            drained = 0
            while i != NONE:
                route = self.route[i]
                self.activate(route & 127, i)
                i = (route >> 7) - 1
                drained += 1
            self.drained += drained
            self.peak_bucket = max(self.peak_bucket, drained)
        b = (self.mask[0] | self.mask[1]).bit_length() - 1
        bit = 1 << b
        top = bottom = NONE
        if self.mask[0] & bit:
            self.mask[0] ^= bit
            top = self.local[b]
            self.reads += 1
        if self.mask[1] & bit:
            self.mask[1] ^= bit
            bottom = self.local[SIDE + b]
            self.reads += 1
        return SIDE * self.page + b + 1, top, bottom


def sweep(widths, symbols, marked, audit, budget=None):
    """Induce from initial widths, symbols and a primary singleton using only first arrivals."""
    n, r = sum(widths), len(widths)
    boxes, words = Blocks(n, r), Words()
    for i, (top, bottom) in enumerate(roots(widths, symbols)):
        displacement = bottom - top
        word = words.literal(symbols[i], i == marked)
        boxes.post(max(top, bottom), int(displacement > 0), i, Packet(abs(displacement), word, i))
    left = right = None
    cuts = closures = 0
    while True:
        q, t, b = boxes.next()
        if t != NONE:
            left = boxes.packet[t]
            if left.distance == 0:
                assert b == NONE
                closures += 1
                audit.close(cuts, q, left)
                if left.word & MARK:
                    return left, words, boxes, cuts, closures
                left = right = None
                continue
        if b != NONE:
            right = boxes.packet[b]
        assert left is not None and right is not None
        if budget is not None and cuts == budget:
            return None, words, boxes, cuts, closures
        name = r + cuts
        cw = words.join(left.word, right.word)
        audit.cell(cuts, q, left, right, cw, t != NONE, b != NONE)
        a, d = left.distance, right.distance
        new = Packet(abs(d - a), cw, name)
        boxes.post(q - min(a, d), int(d > a), t if t != NONE else b, new)
        cuts += 1


class TraceAudit:
    """All trace arrays are confined to the observer. They cannot choose a transition."""

    def __init__(self, rec):
        self.rec, self.ci, self.q = rec, 0, rec.n
        self.words = Words()
        self.table = [Words.literal(c, i == rec.row_letter) for i, c in enumerate(rec.a["l_byte"])]
        self.fresh = (True, True)

    def cell(self, i, q, left, right, cw, t, b):
        """Compare a reported join and its incoming sides with the historical transition."""
        v = self.rec.a
        x, y = v["s_a"][i], v["s_b"][i]
        a, d = v["s_ra"][i], v["s_rb"][i]
        assert (q, left.name, right.name, t, b) == (self.q, x, y, *self.fresh)
        assert (left.word, right.word) == (self.table[x], self.table[y])
        made = self.words.join(self.table[x], self.table[y])
        assert made == cw and (cw & (MARK - 1)) == v["w_len"][self.rec.r + i]
        representation = irec.ARENA if (cw & LOW) <= 16 else irec.RULE
        assert v["w_kind"][self.rec.r + i] == representation | (irec.MARKED if cw & MARK else 0)
        assert v["s_type"][i] == (
            irec.EQUAL if a == d else irec.TOP_WINS if a > d else irec.BOTTOM_WINS
        )
        self.table.append(made)
        self.q -= min(a, d)
        self.fresh = (a <= d, d <= a)

    def close(self, i, q, item):
        """Check the identity, coordinate and descriptor of the next recorded closure."""
        v = self.rec.a
        assert self.fresh == (True, True)
        assert (q, i, item.name) == (self.q, v["c_step"][self.ci], v["c_letter"][self.ci])
        assert item.word == self.table[item.name]
        self.ci += 1
        k = item.name
        if k < self.rec.r:
            self.q -= v["l_lam"][k]
        else:
            self.q -= min(v["s_ra"][k - self.rec.r], v["s_rb"][k - self.rec.r])
        self.fresh = (True, True)


def check_marked_cycle(rec):
    """Independent LF check of the marked grammar already matched by the cut observer."""
    node, rotation = rec.final, 1
    while node >= rec.r:
        left, right = rec.children(node)
        if rec.a["w_kind"][left] & irec.MARKED:
            node = left
        else:
            rotation += rec.a["w_len"][left]
            node = right
    assert node == rec.row_letter
    word = rec.word(rec.final)
    assert len(word) == rec.period
    rotation %= len(word)
    column = b"".join(bytes([c]) * w for c, w in zip(rec.a["l_byte"], rec.a["l_lam"]))
    inverse = sorted(range(rec.n), key=column.__getitem__)
    row = rec.p
    for i, c in enumerate(word[rotation:] + word[:rotation]):
        row = inverse[row]
        assert column[row] == c and ((row == rec.p) == (i + 1 == rec.period))


def column_letters(column, primary):
    """Supply roots from an independently extracted last column and its primary row."""
    widths, symbols = [], []
    mark = None
    start = 0
    for symbol, run in groupby(column):
        end = start + sum(1 for _ in run)
        if start <= primary < end:
            if start < primary:
                widths.append(primary - start)
                symbols.append(symbol)
            mark = len(widths)
            widths.append(1)
            symbols.append(symbol)
            start = primary + 1
        if start < end:
            widths.append(end - start)
            symbols.append(symbol)
        start = end
    assert mark is not None
    return widths, symbols, mark


def check(rec, lf=False, column=None):
    """Compare the entire emitted history with a real tape and report scheduling counts."""
    v = rec.a
    audit = TraceAudit(rec)
    if column is None:
        widths, symbols, mark = v["l_lam"], v["l_byte"], rec.row_letter
    else:
        widths, symbols, mark = column_letters(column, rec.p)
        assert (widths, symbols, mark) == (list(v["l_lam"]), list(v["l_byte"]), rec.row_letter)
    root, words, box, cuts, closed = sweep(widths, symbols, mark, audit)
    assert (root.name, root.word & (MARK - 1), cuts, closed) == (
        rec.final,
        rec.period,
        rec.steps,
        rec.closures,
    )
    assert (words.tail, words.rule, words.pairs) == (
        audit.words.tail,
        audit.words.rule,
        audit.words.pairs,
    )
    if lf:
        check_marked_cycle(rec)
    return dict(
        corpus=rec.corpus,
        block=rec.block,
        n=rec.n,
        r=rec.r,
        cuts=cuts,
        closures=closed,
        period=rec.period,
        reads=box.reads,
        posted_far=box.remote,
        posted_here=box.near,
        drained=box.drained,
        pages=box.pages,
        peak_bucket=box.peak_bucket,
        rules=len(words.pairs),
        short_bytes=words.tail - ARENA - 288,
    )


def main():
    """Run the structural observer on the requested tapes."""
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument(
        "--lf", action="store_true", help="also check the marked spelling against stable LF"
    )
    p.add_argument(
        "--column", type=Path, help="supply an independent raw last column for a single tape"
    )
    p.add_argument("records", nargs="+")
    args = p.parse_args()
    if args.column and (len(args.records) != 1 or Path(args.records[0]).is_dir()):
        p.error("--column takes one trace file")
    column = args.column.read_bytes() if args.column else None
    out = None
    for base in args.records:
        for path in irec.paths(base):
            row = check(irec.load(path), args.lf, column)
            if out is None:
                out = csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
                out.writeheader()
            out.writerow(row)


if __name__ == "__main__":
    main()
