#!/usr/bin/env python3
"""Count front relations and first arrivals in recorded Cells cuts, without driving the induction."""

import argparse
import csv
import sys
from collections import Counter
from itertools import accumulate
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "bench"))
import irec

SIDE = 64


def roots(widths, symbols):
    """Yield exclusive endpoints in top order using a stable byte-row prefix sum."""
    counts = [0] * 256
    for c, width in zip(symbols, widths):
        counts[c] += width
    ends = list(accumulate(counts, initial=0))
    q = 0
    for c, width in zip(symbols, widths):
        q += width
        ends[c] += width
        yield q, ends[c]


def screen(rec):
    """Read the recorded fronts and check the first-arrival predictions against the tape."""
    a = rec.a
    widths, symbols = a["l_lam"], a["l_byte"]
    endpoints = list(roots(widths, symbols))
    assert list(accumulate(widths[i] for i in a["l_bottom"])) == [
        endpoints[i][1] for i in a["l_bottom"]
    ]
    shift = [b - t for t, b in endpoints]
    arrival = [max(p) for p in endpoints]
    hits = set()
    count = Counter()
    count["raw_top"] = sum(x < 0 for x in shift)
    count["raw_bottom"] = sum(x > 0 for x in shift)
    count["root_zero"] = sum(x == 0 for x in shift)
    cap_records = 0
    last_c, combined = None, 0
    for c, w in zip(symbols, widths):
        if c != last_c:
            cap_records += (combined + 126) // 127
            combined = 0
        last_c, combined = c, combined + w
    count["records"] = cap_records + (combined + 126) // 127
    f, ci = rec.n, 0
    prev = -1
    total_keys = set(arrival)
    for i in range(rec.steps + 1):
        while ci < rec.closures and a["c_step"][ci] == i:
            k = a["c_letter"][ci]
            assert f == arrival[k] and shift[k] == 0
            ci += 1
            if k == rec.final:
                break
            width = widths[k] if k < rec.r else min(a["s_ra"][k - rec.r], a["s_rb"][k - rec.r])
            f -= width
            prev = -1
        if i == rec.steps:
            break
        x, y = a["s_a"][i], a["s_b"][i]
        rx, ry = a["s_ra"][i], a["s_rb"][i]
        kind, fresh = a["s_type"][i], a["s_fresh"][i]
        assert shift[x] < 0 < shift[y]
        for k in (x, y):
            if k not in hits:
                assert arrival[k] == f
                hits.add(k)
        child_shift = shift[x] + shift[y]
        when = f - min(-shift[x], shift[y])
        assert 0 < when < f
        shift.append(child_shift)
        arrival.append(when)
        count["negative"] += child_shift < 0
        count["positive"] += child_shift > 0
        count["diagonal"] += child_shift == 0
        count["far_posts"] += (f - 1) // SIDE != (when - 1) // SIDE
        count["is_equal"] += kind == irec.EQUAL
        count["top_win"] += kind == irec.TOP_WINS
        count["bottom_win"] += kind == irec.BOTTOM_WINS
        count["unit_pair"] += rx == ry == 1
        count["unit_min"] += min(rx, ry) == 1
        count["just_born"] += i > 0 and rec.r + i - 1 in (x, y)
        count["same_strict"] += kind == prev and kind != irec.EQUAL
        count["both_raw_equal"] += fresh == 3 and kind == irec.EQUAL
        if prev in (irec.TOP_WINS, irec.BOTTOM_WINS):
            winner = x if prev == irec.TOP_WINS else y
            count["carried_front"] += 1
            assert winner == (a["s_a"] if prev == irec.TOP_WINS else a["s_b"])[i - 1]
            count["carried_short_load"] += (
                a["w_kind"][rec.r + i] == a["w_kind"][rec.r + i - 1] == irec.ARENA
            )
        total_keys.add(when)
        f -= min(rx, ry)
        prev = kind
    for key in (
        "negative",
        "positive",
        "diagonal",
        "far_posts",
        "is_equal",
        "top_win",
        "bottom_win",
        "unit_pair",
        "unit_min",
        "just_born",
        "same_strict",
        "both_raw_equal",
        "carried_front",
        "carried_short_load",
    ):
        count[key] += 0
    return dict(
        corpus=rec.corpus,
        block=rec.block,
        n=rec.n,
        letters=rec.r,
        cuts=rec.steps,
        closures=rec.closures,
        period=rec.period,
        coordinates=len(total_keys),
        **dict(sorted(count.items())),
    )


def main():
    """Write one exact structural census row per supplied tape."""
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("records", nargs="+")
    args = p.parse_args()
    out = None
    for base in args.records:
        for f in irec.paths(base):
            row = screen(irec.load(f))
            if out is None:
                out = csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
                out.writeheader()
            out.writerow(row)


if __name__ == "__main__":
    main()
