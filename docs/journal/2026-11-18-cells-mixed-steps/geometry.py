#!/usr/bin/env python3
"""Literal two-order proofs by finite checks and a local obstruction measured on trace files."""
import argparse
import itertools
import json
from collections import Counter

from census import irec
from verify import letters, lf_output


def replace(order, by):
    return [y for x in order for y in by.get(x, [x])]


def scalar(top, bot, widths, x):
    a, b = top[-1], bot[-1]
    w = min(widths[a], widths[b])
    widths = {**widths, a: widths[a] - w, b: widths[b] - w, x: w}
    top = [v for v in replace(top, {b: [b, x]}) if widths[v]]
    bot = [v for v in replace(bot, {a: [a, x]}) if widths[v]]
    return top, bot, {k: v for k, v in widths.items() if v}


def formula(top, bot, widths, x):
    a, b, d = top[-1], bot[-1], bot[-2]
    y = x + 1
    s, t = widths[a] - widths[b], widths[d]
    assert a != d and 0 < s <= t
    top = replace(top, {a: [], b: [x], d: [d, y] if s < t else [y]})
    bot = replace(bot, {b: [], a: [y, x], d: [d] if s < t else []})
    widths = {**widths, a: 0, b: 0, d: t - s, x: widths[b], y: s}
    return top, bot, {k: v for k, v in widths.items() if v}


def permutations():
    count = 0
    for n in range(3, 7):
        top = list(range(n))
        for bot in itertools.permutations(top):
            a, b, d = top[-1], bot[-1], bot[-2]
            if a in (b, d):
                continue
            for wb, wd in itertools.product(range(1, 4), repeat=2):
                for s in range(1, wd + 1):
                    widths = dict.fromkeys(top, 1)
                    widths.update({a: wb + s, b: wb, d: wd})
                    assert formula(top, bot, widths, n) == scalar(*scalar(top, bot, widths, n), n + 1)
                    count += 1
    return count


def witnesses():
    found = {}
    settings = [(n, alphabet) for n in range(2, 9) for alphabet in (2, 3)]
    for n, alphabet in settings:
        for col in itertools.product(range(alphabet), repeat=n):
            if len(lf_output(col, 0)) != n:
                continue
            for p in range(n):
                width, byte, bot, mark = letters(col, p)
                r = len(width)
                top, widths, seen = list(range(r)), dict(enumerate(width)), set()
                for i in range(n):
                    a, b, x = top[-1], bot[-1], r + i
                    seen.update((a, b))
                    if a == b:
                        break  # one cycle, so its marked letter closes
                    if widths[a] > widths[b] and len(bot) >= 2:
                        d = bot[-2]
                        s = widths[a] - widths[b]
                        info = dict(L="".join(map(str, col)), p=p, step=i, top=top, bottom=bot,
                                    A=a, B=b, D=d, widths=[widths[k] for k in (a, b, d)])
                        if d == a:
                            found.setdefault("alias", info)
                        elif s <= widths[d] and d < r and d not in seen and b < r:
                            if byte[d] == byte[b] and mark in (b, d):
                                found.setdefault("mark", info)
                            if byte[d] == byte[b] and mark not in (a, b, d) and a < r:
                                if top.index(d) + 1 != top.index(b):
                                    found.setdefault("distinct_sites", info)
                            if byte[d] != byte[b] and mark not in (b, d):
                                found.setdefault("different_bytes", info)
                    top, bot, widths = scalar(top, bot, widths, x)
                if len(found) == 4:
                    return found
    return found


def screen(rec):
    log = rec.a
    orders = irec.Orders(rec)
    count, first = Counter(), {}
    for i in range(rec.steps):
        orders._close()
        if (log["s_type"][i] == irec.TOP_WINS and i + 1 < rec.steps
                and log["s_fresh"][i + 1] & irec.FRESH_B and log["s_type"][i + 1] != irec.TOP_WINS):
            a, b = orders.ttail, orders.btail
            d = orders.bp[b]
            assert d == log["s_b"][i + 1] and d < rec.r
            short = log["w_kind"][rec.r + i] == irec.ARENA
            same = (b < rec.r and log["l_byte"][b] == log["l_byte"][d]
                    and b != rec.row_letter and d != rec.row_letter)
            if same and short:
                # Bottom order after the joint update is Y,X. Can these two
                # translations be a single band? Only if top also has Y,X.
                align = orders.tn[d] == b
                assert d + 1 < b, (i, b, d)  # a distinct initial top site between them
                count["can_glue" if align else "cannot_glue"] += 1
                name = ("TE" if log["s_type"][i + 1] == irec.EQUAL else "TB") + ("_glue" if align else "_apart")
                first.setdefault(name, [i, a, b, d, orders.lam[a], orders.lam[b], orders.lam[d],
                                        log["l_byte"][d], log["w_len"][rec.r + i]])
        orders.step()
    orders.finish()
    return dict(corpus=rec.corpus, block=rec.block, **count, first=first)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("records", nargs="*")
    parser.add_argument("--small", action="store_true")
    args = parser.parse_args()
    if args.small:
        print(json.dumps(dict(permutations=permutations(), witnesses=witnesses())))
    for group in args.records:
        for path in irec.paths(group):
            print(json.dumps(screen(irec.load(path))), flush=True)


if __name__ == "__main__":
    main()
