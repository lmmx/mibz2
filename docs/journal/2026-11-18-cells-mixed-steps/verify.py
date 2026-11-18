#!/usr/bin/env python3
"""Structural transducer for the two ending orders of lazy Cells. No entropy or output decoder."""
import argparse
import hashlib
import itertools
import json
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

from census import census, irec

E, T, B = irec.EQUAL, irec.TOP_WINS, irec.BOTTOM_WINS
MARK = 1 << 31
SMALL = 16


class Recipes:
    """Literal handles are canonical. Each other call constructs a fresh arena word or rule."""
    def __init__(self, costs):
        self.costs = costs
        self.length = [1] * 256 + [MARK + 1] * 256
        self.payload = [bytes([x]) for x in range(256)] * 2

    def join(self, a, b):
        length = self.length[a] + self.length[b]
        self.costs["joins"] += 1
        if length <= SMALL:
            payload = self.payload[a] + self.payload[b]
            self.costs["copy16"] += 2
            self.costs["arena_bytes"] += length
        else:
            payload = (a, b)
            self.costs["rules"] += 1
        word = len(self.length)
        self.length.append(length)
        self.payload.append(payload)
        return word

    def expand(self, root):
        """Audit only. Preserve the mark, including its position inside a long rule."""
        stack, data, marks = [root], bytearray(), []
        while stack:
            x = stack.pop()
            if 256 <= x < 512:
                marks.append(len(data))
            parts = self.payload[x]
            if isinstance(parts, bytes):
                data.extend(parts)
            else:
                a, b = parts
                stack.extend((b, a))
        assert len(marks) == 1
        return bytes(data), marks[0]


@dataclass(slots=True)
class Front:
    prev: int
    width: int
    word: int
    other: int
    slot: int
    letter: int  # audit name, never used to choose a cut


@dataclass(slots=True)
class Top:
    prev: int = 0
    width: int = 0
    word: int = 0
    other: int = 0
    letter: int = -1


@dataclass(slots=True)
class Bottom:
    prev: int = 0
    other: int = 0


class Machine:
    def __init__(self, width, byte, bottom, marked, joint):
        self.width, self.byte, self.order, self.marked = width, byte, bottom, marked
        self.r = self.kt = self.kb = len(width)
        self.tag = [0] * self.r
        self.tops, self.bots, self.free = [None], [None], []
        self.joint = joint
        self.cost = Counter()
        self.recipes = Recipes(self.cost)
        self.step = 0

    def take(self):
        self.cost["take"] += 1
        if self.free:
            return self.free.pop()
        i = len(self.tops)
        self.tops.append(Top())
        self.bots.append(Bottom())
        return i, i

    def give(self, t, b):
        self.cost["give"] += 1
        self.free.append((t, b))

    def literal(self, x, side):
        pending = -1 - x  # the shipped ENTRY tag, shared across orders
        return Front(0, self.width[x], self.byte[x] + 256 * (x == self.marked),
                     pending if side == "top" else 0, pending if side == "bottom" else 0, x)

    def settle_top(self, a):
        if a.other < 0:
            x = -1 - a.other
            a.slot, a.other = self.take()
            self.bots[a.other].prev = 0
            self.tag[x] = a.other

    def settle_bottom(self, b, init_link=True):
        if b.slot < 0:
            x = -1 - b.slot
            b.other, b.slot = self.take()
            if init_link:
                self.tops[b.other].prev = 0
                self.cost["bottom_zero_init"] += 1
            self.tag[x] = b.other

    def read_top(self, link):
        self.cost["top_read"] += 1
        if not link:
            assert self.kt > 0
            self.kt -= 1
            x = self.kt
            self.cost["top_header"] += 1
            link = self.tag[x]
            if not link:
                return self.literal(x, "top"), irec.FRESH_A
        t = self.tops[link]
        self.cost["top_slot"] += 1
        return Front(t.prev, t.width, t.word, t.other, link, t.letter), irec.SLOT_A

    def bottom_source(self, link):
        """Both paths perform this mandatory dispatch once. A slot payload can still change."""
        self.cost["bottom_read"] += 1
        if link:
            return link, None
        assert self.kb > 0
        self.kb -= 1
        x = self.order[self.kb]
        self.cost["bottom_header"] += 1
        link = self.tag[x]
        return (link, None) if link else (0, x)

    def bottom_payload(self, source):
        link, x = source
        if x is not None:
            return self.literal(x, "bottom"), irec.FRESH_B
        b = self.bots[link]
        t = self.tops[b.other]
        self.cost["bottom_slot"] += 1
        return Front(b.prev, t.width, t.word, b.other, link, t.letter), irec.SLOT_B

    def cmp(self, a, b):
        self.cost["width_decisions"] += 1
        return E if a == b else T if a > b else B

    def cell(self, t, b, width, word, letter):
        p = self.tops[t]
        p.width, p.word, p.other, p.letter = width, word, b, letter

    def ordinary(self, a, b, kind, word, x):
        """The three slot updates in cells_lazy_induce.rs. Reads of successor slots follow."""
        t, k = b.other, a.other
        if kind == T:
            self.cell(t, k, b.width, word, x)
            self.bots[b.slot].prev = self.bots[k].prev
            self.bots[k].prev, self.bots[k].other = b.slot, t
            a.other = b.slot
        elif kind == B:
            self.bots[k].other = t
            self.tops[a.slot].prev = self.tops[t].prev
            self.cost["bottom_relink_read"] += 1
            self.tops[t].prev = a.slot
            self.cell(t, k, a.width, word, x)
            b.other = a.slot
        else:
            self.cell(t, k, a.width, word, x)
            self.bots[k].other = t
            self.give(a.slot, b.slot)

    def pair(self, a, b, d, s, kind, xword, yword, x):
        """Direct final fields. Never install the continuation of A after its first cut."""
        t, k, g, u = b.other, a.other, b.slot, d.other
        h = self.bots[k].prev
        self.cell(t, k, b.width, xword, x)
        self.cell(u, g, s, yword, x + 1)
        self.bots[k].prev, self.bots[k].other = g, t
        self.bots[g].prev, self.bots[g].other = h, u
        if kind == E:
            self.give(a.slot, d.slot)
        else:
            self.tops[a.slot].prev = 0
            self.tops[u].prev = a.slot
            d.other = a.slot
            d.width -= s

    def run(self, audit=None):
        # Cursor/pool checks are the successful-record path. There is no admission
        # model here. A bounded caller must debit both logical cells (see journal).
        na = nb = 0
        while True:
            a, af = self.read_top(na)
            b, bf = self.bottom_payload(self.bottom_source(nb))
            fresh = af | bf
            if a.other == b.slot:
                assert a.letter == b.letter
                if audit:
                    audit.close(self.step, a.letter)
                self.cost["closures"] += 1
                if self.recipes.length[a.word] & MARK:
                    return a.letter, self.recipes.expand(a.word)
                if a.other >= 0:
                    self.give(a.slot, b.slot)
                na, nb = a.prev, b.prev
                continue
            self.settle_top(a)
            self.settle_bottom(b)
            known = None
            while True:
                kind = self.cmp(a.width, b.width) if known is None else known
                known = None
                x = self.r + self.step
                if audit:
                    audit.cell(self.step, a, b, kind, fresh, self.recipes)
                first = self.recipes.join(a.word, b.word)
                self.step += 1
                # Move ONLY the successor-source read. Use the raw untagged entry
                # directly. For a slot, wait for ordinary T's writes before reading it.
                source = self.bottom_source(b.prev) if self.joint and kind == T else None
                if source is not None and not source[0]:
                    d, df = self.bottom_payload(source)
                    s = a.width - b.width
                    second = self.cmp(s, d.width)
                    self.cost["fresh"] += 1
                    if second != T:
                        self.cost["eligible"] += 1
                        self.cost["fresh_TB"] += second == B
                        self.settle_bottom(d, init_link=second == E)
                        self.cost["equal_tests"] += 1
                        same_operand = b.word == d.word
                        if same_operand:
                            self.cost["short_tests"] += 1
                        share = same_operand and self.recipes.length[first] <= SMALL
                        if share:
                            self.cost["short_TE" if second == E else "short_TB"] += 1
                            self.cost["bytes_saved"] += self.recipes.length[first]
                            last = first
                        else:
                            last = self.recipes.join(a.word, d.word)
                        if audit:
                            audit.cell(self.step, a, d, second, df, self.recipes, width=s)
                        self.pair(a, b, d, s, second, first, last, x)
                        self.step += 1
                        if second == E:
                            na, nb = a.prev, d.prev
                            break
                        a, fresh = self.read_top(a.prev)
                        self.settle_top(a)
                        b = d
                        continue
                    self.ordinary(a, b, T, first, x)
                    self.settle_bottom(d)
                    a.width, b, fresh, known = s, d, df, T
                    continue
                self.ordinary(a, b, kind, first, x)
                if kind == E:
                    na, nb = a.prev, b.prev
                    break
                if kind == T:
                    a.width -= b.width
                    if source is None:
                        source = self.bottom_source(b.prev)
                    b, fresh = self.bottom_payload(source)
                    self.settle_bottom(b)
                else:
                    b.width -= a.width
                    a, fresh = self.read_top(a.prev)
                    self.settle_top(a)


class Audit:
    """Assertions only. No result of these methods is read by either machine."""
    def __init__(self, rec):
        self.rec, self.closed = rec, 0

    def close(self, step, letter):
        a = self.rec.a
        assert (a["c_step"][self.closed], a["c_letter"][self.closed]) == (step, letter)
        self.closed += 1

    def cell(self, step, a, b, kind, fresh, recipes, width=None):
        log = self.rec.a
        assert (a.letter, b.letter, a.width if width is None else width, b.width, kind, fresh) == (
            log["s_a"][step], log["s_b"][step], log["s_ra"][step], log["s_rb"][step],
            log["s_type"][step], log["s_fresh"][step])
        for f in (a, b):
            length = recipes.length[f.word]
            assert length & ~MARK == log["w_len"][f.letter]
            assert bool(length & MARK) == bool(log["w_kind"][f.letter] & irec.MARKED)


def compare(inputs, rec=None):
    plain, joint = (Machine(*inputs, joint=flag) for flag in (False, True))
    audit = Audit(rec) if rec is not None else None
    expected = plain.run()
    result = joint.run(audit)
    assert result == expected
    p, j = plain.cost, joint.cost
    saved = j["short_TE"] + j["short_TB"]
    assert plain.step == joint.step == j["width_decisions"] == p["width_decisions"]
    for key in ("top_read", "bottom_read", "top_header", "bottom_header", "top_slot",
                "bottom_slot", "take", "give", "closures", "rules"):
        assert p[key] == j[key], key
    assert len(plain.tops) == len(joint.tops)
    assert (plain.kt, plain.kb, plain.tag, plain.free) == (joint.kt, joint.kb, joint.tag, joint.free)
    assert j["equal_tests"] == j["eligible"]
    assert saved <= j["short_tests"] <= j["equal_tests"]
    assert p["joins"] - j["joins"] == saved
    assert p["copy16"] - j["copy16"] == 2 * saved
    assert p["arena_bytes"] - j["arena_bytes"] == j["bytes_saved"]
    for key in ("bottom_zero_init", "bottom_relink_read"):
        assert p[key] - j[key] == j["fresh_TB"], key
    if rec is not None:
        word, mark = result[1]
        assert audit.closed == rec.closures and joint.step == rec.steps and result[0] == rec.final
        assert word == rec.word(rec.final) and len(word) == rec.period
        # Locate the mark independently down the recorded, unshared grammar.
        x, q = rec.final, 0
        while x >= rec.r:
            a, b = rec.children(x)
            if rec.a["w_kind"][a] & irec.MARKED:
                x = a
            else:
                q += rec.a["w_len"][a]
                x = b
        assert x == rec.row_letter and q == mark
        counts = census(rec)
        assert j["short_tests"] == j["eligible"] - counts["different"]
        for key in ("fresh", "eligible", "fresh_TB", "short_TE", "short_TB", "bytes_saved"):
            assert j[key] == counts[key], key
    return result[1], j


def letters(column, p):
    width, byte, mark = [], [], None
    for i, c in enumerate(column):
        if i == p or i == p + 1 or not byte or c != byte[-1]:
            if i == p:
                mark = len(byte)
            byte.append(c)
            width.append(0)
        width[-1] += 1
    return width, byte, sorted(range(len(byte)), key=byte.__getitem__), mark


def lf_output(column, p):
    # Inverse of stable sort, walked from the marked row in forward output order.
    inverse_lf = sorted(range(len(column)), key=column.__getitem__)
    data, row = bytearray(), p
    while True:
        row = inverse_lf[row]
        data.append(column[row])
        if row == p:
            return bytes(data)


def exhaustive():
    stats = Counter()
    for size, bound in ((2, 9), (3, 6)):
        for n in range(1, bound + 1):
            for column in itertools.product(range(size), repeat=n):
                for p in range(n):
                    (word, mark), counts = compare(letters(column, p))
                    assert word[mark + 1:] + word[:mark + 1] == lf_output(column, p)
                    stats["marked_columns"] += 1
                    stats["TE"] += counts["eligible"] - counts["fresh_TB"]
                    stats["TB"] += counts["fresh_TB"]
                    stats["shared"] += counts["short_TE"] + counts["short_TB"]
    return stats


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("records", nargs="*")
    parser.add_argument("--small", action="store_true")
    args = parser.parse_args()
    if args.small:
        print(json.dumps(dict(exhaustive()), sort_keys=True))
    for group in args.records:
        for path in irec.paths(group):
            rec = irec.load(path)
            a = rec.a
            (word, mark), cost = compare((a["l_lam"], a["l_byte"], a["l_bottom"], rec.row_letter), rec)
            evidence = {key: cost[key] for key in ("fresh", "eligible", "short_TE", "short_TB",
                                                    "bytes_saved", "width_decisions")}
            print(json.dumps(dict(corpus=rec.corpus, block=rec.block, period=len(word), mark=mark,
                                  word_sha256=hashlib.sha256(word).hexdigest(), **evidence)), flush=True)


if __name__ == "__main__":
    main()
