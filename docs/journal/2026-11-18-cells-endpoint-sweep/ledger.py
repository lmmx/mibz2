#!/usr/bin/env python3
"""Rebuild the disclosed scalar and field-operation screen from the three measured ledgers."""

import csv
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent


def rows(name):
    """Read the committed measurements, converting decimal count fields to integers."""
    with (HERE / name).open() as f:
        for row in csv.DictReader(f):
            yield {k: (int(v) if v.isdecimal() else v) for k, v in row.items()}


def prices(trace, sweep, address):
    """Charge one block according to the scalar schedules in the accompanying ledger."""
    r, c, m = trace["letters"], trace["cuts"], trace["records"]
    equal, top, bottom = trace["is_equal"], trace["top_win"], trace["bottom_win"]
    closed, n = trace["closures"], trace["n"]
    assert closed == 1 and trace["period"] == n and not trace["root_zero"]
    fresh = equal + closed
    h_top, h_bot = fresh + bottom, fresh + top
    seen, pages = sweep["reads"], sweep["pages"]
    remote, near = sweep["drained"], sweep["posted_here"]
    assert seen == r + c == h_top + h_bot - closed and remote + near == seen
    mode = address["model"]
    # Budget/word parts are common. Explicit field accesses are added separately below.
    common = 9 * c + 3 * (c - sweep["rules"])
    if mode in ("eager", "lazy"):
        scalar = c + 2 * (top + bottom) + fresh + closed + 2 * top
        if mode == "eager":
            scalar += h_bot + 2 * (h_top + h_bot) + h_bot
            setup = 11 * r + 256
        else:
            raw_bottom = trace["raw_bottom"]
            scalar += h_bot - raw_bottom + 2 * (h_top + h_bot)
            scalar += 6 * r + r + r + 6 * r + 2 * raw_bottom + r
            scalar += 2 * (fresh - closed) + r + 4 * address["pool"]
            setup = 5 * r + 512
    else:
        # Pull arrivals (descending nonempty offsets), detect a diagonal and inspect its mark.
        scalar = 7 * (c + closed) + seen + h_top + closed + 4 * pages
        scalar += 6 * c  # min by compare/select, frontier subtraction, signed sum, abs, sign
        setup = 9 * r + m + 256 + 2 * pages
        if mode == "bucket":
            setup += 8 * r
            scalar += 5 * c + c + 3 * (remote - r) + 3 * near + c
            scalar += 9 * remote + pages
        else:
            setup += 7 * r
            # Partition q into page and bit, select/set mask, form interleaved address;
            # the local-vs-remote test is needed for every newborn.
            scalar += 7 * c + c + seen
    geometry = address["core_r"] + address["core_w"] + address["hint_r"]
    word_fields = address["word_r"] + address["word_w"]
    core = common + scalar + geometry + word_fields
    setup += address["setup_r"] + address["setup_w"]
    return dict(
        block=trace["block"],
        model=mode,
        eager=int(64 * m >= n and 10 * m < n and c <= 4 * n // 25),
        lazy=int(64 * m >= n and 20 * m < 3 * n and c <= 8 * n // 25),
        setup=setup,
        controls=scalar,
        geometry=geometry,
        recipe=common + word_fields,
        core=core,
        setup_core=setup + core,
        transfers32=address["l1_fills"] + address["l1_dirty"],
        transfers256=address["l2_fills"] + address["l2_dirty"],
    )


def main():
    """Join the measured tables by block and emit all four schedules."""
    census = {r["block"]: r for r in rows("census.csv")}
    runs = {r["block"]: r for r in rows("schedules.csv") if r["corpus"] == "wikidata.bz2"}
    writer = None
    for address in rows("traffic.csv"):
        k = address["block"]
        row = prices(census[k], runs[k], address)
        if writer is None:
            writer = csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
            writer.writeheader()
        writer.writerow(row)


if __name__ == "__main__":
    main()
