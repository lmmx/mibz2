#!/usr/bin/env python3
"""Independent ordinary-interval oracle on every marked small column."""

import itertools
import json
from collections import Counter

from sweep import MARK, Words, sweep


def letters(column, p):
    """Partition an arbitrary column into runs with a singleton at the primary row."""
    width, symbol, marked = [], [], None
    for i, c in enumerate(column):
        if i in (p, p + 1) or not symbol or c != symbol[-1]:
            if i == p:
                marked = len(width)
            symbol.append(c)
            width.append(0)
        width[-1] += 1
    return width, symbol, marked


class Orders:
    """Independent two-order oracle with explicit widths, spellings and marked offsets."""

    def __init__(self, column, primary, witness):
        width, symbol, marked = letters(column, primary)
        self.top = list(range(len(width)))
        self.bot = sorted(self.top, key=symbol.__getitem__)
        self.width = list(width)
        self.words = Words()
        self.pack = [Words.literal(c, i == marked) for i, c in enumerate(symbol)]
        self.spell = [bytes([c]) for c in symbol]
        self.mark = [0 if i == marked else None for i in self.top]
        self.q = len(column)
        self.fresh = (True, True)
        self.witness = witness
        self.label = dict(column="".join(map(str, column)), primary=primary)
        self.last_post = {}
        self.events = self.unmarked = 0

    def cell(self, i, q, a, b, cw, t, u):
        """Check the proposed cut, then perform it geometrically in both orders."""
        x, y = self.top[-1], self.bot[-1]
        assert x != y and (q, a.name, b.name, t, u) == (self.q, x, y, *self.fresh)
        wa, wb = self.width[x], self.width[y]
        assert (a.word, b.word) == (self.pack[x], self.pack[y])
        assert self.words.join(self.pack[x], self.pack[y]) == cw
        sa, sb = a.distance, b.distance
        # Direct endpoints of each current interval, independent of accumulated shifts.
        bt = sum(self.width[z] for z in self.bot[: self.bot.index(x) + 1])
        tb = sum(self.width[z] for z in self.top[: self.top.index(y) + 1])
        assert (sa, sb) == (q - bt, q - tb)
        sign = (sb > sa) - (sb < sa)
        key = q - min(sa, sb)
        info = dict(**self.label, cut=i, frontier=q, widths=[wa, wb], gaps=[sa, sb], post=key)
        if sign:
            if wa == wb == 1:
                self.witness.setdefault("unit_equal_" + ("top" if sign < 0 else "bottom"), info)
            previous = self.last_post.get(sign)
            if previous and previous > key:
                self.witness.setdefault("decreasing_posts", dict(**info, previous=previous))
            if previous and previous < key:
                self.witness.setdefault("increasing_posts", dict(**info, previous=previous))
            self.last_post[sign] = key
        if sa == sb and wa != wb:
            self.witness.setdefault("zero_born_at_strict", info)
        w = min(wa, wb)
        new = len(self.width)
        self.spell.append(self.spell[x] + self.spell[y])
        self.mark.append(
            self.mark[x]
            if self.mark[x] is not None
            else (len(self.spell[x]) + self.mark[y] if self.mark[y] is not None else None)
        )
        self.pack.append(cw)
        self.top.insert(self.top.index(y) + 1, new)
        self.bot.insert(self.bot.index(x) + 1, new)
        self.width.append(w)
        for z in (x, y):
            self.width[z] -= w
            if not self.width[z]:
                self.top.remove(z)
                self.bot.remove(z)
        self.fresh = (wa <= wb, wb <= wa)
        self.q -= w
        self.events += 1

    def close(self, i, q, item):
        """Remove the common final band and classify its marked stopping behaviour."""
        assert (
            self.top[-1] == self.bot[-1] == item.name and self.q == q and self.fresh == (True, True)
        )
        z = self.top.pop()
        self.bot.pop()
        assert item.word == self.pack[z] and item.distance == 0
        self.q -= self.width[z]
        if not item.word & MARK:
            self.unmarked += 1
            self.witness.setdefault("unmarked_closure", dict(**self.label, frontier=q, cut=i))
        elif self.q:
            self.witness.setdefault(
                "early_mark", dict(**self.label, frontier=q, rows_left=self.q, cut=i)
            )


def run(column, primary, witness, budget=None):
    """Compare the sweep with the geometric oracle and verify the marked period by LF."""
    width, symbol, mark = letters(column, primary)
    a = Orders(column, primary, witness)
    root, words, box, cuts, closed = sweep(width, symbol, mark, a, budget)
    assert (words.tail, words.rule, words.pairs) == (a.words.tail, a.words.rule, a.words.pairs)
    if root is None:
        assert cuts == budget and a.top[-1] != a.bot[-1]
        return None
    marked = a.mark[root.name]
    assert marked is not None
    text = a.spell[root.name]
    start = marked + 1
    expected = text[start:] + text[:start]
    inverse = sorted(range(len(column)), key=column.__getitem__)
    row = primary
    for i, c in enumerate(expected):
        row = inverse[row]
        assert column[row] == c and ((row == primary) == (i + 1 == len(text)))
    return len(text), cuts, a.unmarked


def main():
    """Enumerate all declared small inputs and record the first witnesses."""
    count, witness = Counter(), {}
    for alphabet, limit in ((2, 9), (3, 6)):
        for n in range(1, limit + 1):
            for column in itertools.product(range(alphabet), repeat=n):
                for primary in range(n):
                    period, cuts, dropped = run(column, primary, witness)
                    count["cases"] += 1
                    count["full"] += period == n
                    count["early_period"] += period < n
                    count["unmarked_closures"] += dropped
                    if n <= 5:
                        for budget in range(cuts + 1):
                            answer = run(column, primary, {}, budget)
                            assert (answer is None) == (budget < cuts)
                            count["budget_cases"] += 1
    # Exercise the transition between nonadjacent groups of 64 rows as well.
    for col, p in (
        ((1,) * 33 + (0,) * 98, 2),
        ((0,) * 80 + (1,) * 75, 3),
        ((1,) * 65 + (0,) * 67, 64),
    ):
        run(col, p, witness)
    print(json.dumps(dict(totals=count, witnesses=witness), sort_keys=True, indent=2))


if __name__ == "__main__":
    main()
