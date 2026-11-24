#!/usr/bin/env python3
"""Checks of bench/four_arm.py, bench-four-arm's verdict, on synthetic records: each verdict of
the rule (INTERACTION, ADDITIVE, NO POTENTIAL, MARGINAL), the interaction margin and the D/X
interval each binding, the harness check labelling a B that does not reproduce journal §30
UNVALIDATED, and each VOID"""

import io
import math
import os
import random
import sys
import tempfile
from contextlib import redirect_stdout

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import four_arm as fa  # noqa: E402

I9 = "Model name:                           Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
OUT = fa.WIKIDATA_X4_OUT
BLOCKS = 1724
CRC = "2fd63e6f"
BASE = {1: 1.30, 6: 0.30, 10: 0.215}


def tsv(
    ratios=None, noise=0.004, edit=None, meta=None, checks=None, seed=1, rounds=20, arm_noise=None
):
    """The harness's records: `ratios` maps (workers, arm) to the arm's true wall over A's (B 0.99,
    C 0.97 and D 0.96 by default); each round's walls carry lognormal noise (`noise`, or
    `arm_noise`'s for an arm it names, on that arm alone); `edit` edits a run
    row (a dict) or drops it (None); `meta` edits the meta row; `checks` edits a check row"""
    rng = random.Random(seed)
    ratios = ratios or {}
    rows = ["\t".join(fa.COLS)]
    m = {
        "kind": "meta",
        "round": "",
        "workers": "10",
        "arm": "record",
        "order": "",
        "wall_s": "0.3",
        "bytes": str(OUT),
        "crc": CRC,
        "peak_kb": "300000",
        "replayed": "",
        "open_blocks": "932",
        "deliver_ns": "",
        "consume_ns": "",
        "mismatches": "0",
        "detail": f"blocks={BLOCKS} distinct=431 symbol_bytes=50086174 conflicts=0 missing=0 "
        "code_keys=233 codes=26567581 code_bytes=106270324",
    }
    m = meta(m) if meta else m
    if m is not None:
        rows.append("\t".join(m[c] for c in fa.COLS))
    for r in range(rounds):
        for w in (1, 6, 10):
            common = rng.gauss(0, noise)
            for o, a in enumerate(fa.ARMS):
                default = {"A": 1.0, "B": 0.99, "C": 0.97, "D": 0.96}[a]
                own = (arm_noise or {}).get(a, noise)
                wall = BASE[w] * ratios.get((w, a), default) * math.exp(common + rng.gauss(0, own))
                row = {
                    "kind": "run",
                    "round": str(r),
                    "workers": str(w),
                    "arm": a,
                    "order": str(o),
                    "wall_s": f"{wall:.6f}",
                    "bytes": str(OUT),
                    "crc": "",
                    "peak_kb": str(300000 + (50000 if a in "CD" else 0)),
                    "replayed": str(BLOCKS if a in "CD" else 0),
                    "open_blocks": "932",
                    "deliver_ns": "1",
                    "consume_ns": "1",
                    "mismatches": "",
                    "detail": "",
                }
                row = edit(row) if edit else row
                if row is not None:
                    rows.append("\t".join(row[c] for c in fa.COLS))
        c = {
            "kind": "check",
            "round": str(r),
            "workers": "10",
            "arm": "check",
            "order": "",
            "wall_s": "0.3",
            "bytes": str(OUT),
            "crc": CRC,
            "peak_kb": "",
            "replayed": str(BLOCKS),
            "open_blocks": "932",
            "deliver_ns": "",
            "consume_ns": "",
            "mismatches": "0",
            "detail": "",
        }
        c = checks(c) if checks else c
        if c is not None:
            rows.append("\t".join(c[k] for k in fa.COLS))
    return "\n".join(rows) + "\n"


def judge(text, lscpu=I9, extra=()):
    d = tempfile.mkdtemp()
    p, q = os.path.join(d, "runs.tsv"), os.path.join(d, "lscpu.txt")
    with open(p, "w") as f:
        f.write(text)
    with open(q, "w") as f:
        f.write(lscpu)
    buf = io.StringIO()
    with redirect_stdout(buf):
        code = fa.main(["four_arm.py", p, "--lscpu", q, *extra])
    return code, buf.getvalue()


def check(name, text, code, phrases, absent=(), **kw):
    got, out = judge(text, **kw)
    ok = got == code and all(p in out for p in phrases) and not any(p in out for p in absent)
    print(f"{'ok  ' if ok else 'FAIL'} {name}: exit {got}")
    if not ok:
        print(out)
    return ok


def main():
    res = []
    at10 = lambda **r: {(10, a): v for a, v in r.items()}  # noqa: E731
    res.append(
        check(
            "D 0.80, B 0.99, C 0.90 at 10: INTERACTION",
            tsv(at10(B=0.99, C=0.90, D=0.80)),
            0,
            ["DECISION: INTERACTION", "X = C", "reproduced"],
            absent=["UNVALIDATED"],
        )
    )
    res.append(
        check(
            "D 0.87, C 0.88 at 10: ADDITIVE (D within 0.03 of the better single arm)",
            tsv(at10(B=0.99, C=0.88, D=0.87)),
            0,
            ["DECISION: ADDITIVE"],
        )
    )
    # the D/X interval binding: D 0.84 against C 0.875 by its mean, C's own noise wide enough
    # that D/X's interval reaches 1 while D/A's stays narrow
    res.append(
        check(
            "D below C by 0.035 but D/X's interval not below 1: ADDITIVE",
            tsv(at10(B=0.99, C=0.875, D=0.84), arm_noise={"C": 0.2}, seed=4),
            0,
            ["DECISION: ADDITIVE"],
        )
    )
    res.append(
        check(
            "D 0.99 at 10: NO POTENTIAL",
            tsv(at10(B=0.99, C=0.99, D=0.99)),
            0,
            ["DECISION: NO POTENTIAL"],
        )
    )
    res.append(
        check(
            "D 0.92 at 10: MARGINAL",
            tsv(at10(B=0.99, C=0.95, D=0.92)),
            0,
            ["DECISION: MARGINAL"],
        )
    )
    res.append(
        check(
            "D 0.89 with noise wide enough that its upper bound passes 0.90: MARGINAL",
            tsv(at10(B=0.99, C=0.95, D=0.89), noise=0.05, seed=2),
            0,
            ["DECISION: MARGINAL"],
        )
    )
    res.append(
        check(
            "B 0.90 at 10, not §30's: the verdict UNVALIDATED",
            tsv(at10(B=0.90, C=0.95, D=0.85)),
            0,
            ["NOT REPRODUCED", "UNVALIDATED"],
        )
    )
    res.append(
        check(
            "the 1- and 6-worker counts decide nothing",
            tsv({(1, "D"): 0.70, (6, "D"): 0.70, (10, "D"): 0.99}),
            0,
            ["DECISION: NO POTENTIAL", "decides nothing at this count"],
        )
    )
    res.append(
        check(
            "peaks not measured (0): reported so, the verdict stands",
            tsv(edit=lambda r: {**r, "peak_kb": "0"}),
            0,
            ["not measured", "DECISION:"],
        )
    )
    for name, kw, phrase in [
        ("not the i9", {"lscpu": "Model name: AMD EPYC\n"}, "not the i9-10900K"),
        (
            "a decode missing",
            {
                "text": tsv(
                    edit=lambda r: (
                        None if (r["round"], r["arm"], r["workers"]) == ("3", "C", "6") else r
                    )
                )
            },
            "round 3 workers 6 arm C: missing",
        ),
        (
            "a decode's output short",
            {"text": tsv(edit=lambda r: {**r, "bytes": "5"} if r["round"] == "7" else r)},
            "bytes, not",
        ),
        (
            "an arm C decode with a block not replayed",
            {
                "text": tsv(
                    edit=lambda r: {**r, "replayed": str(BLOCKS - 1)} if r["arm"] == "C" else r
                )
            },
            "blocks replayed, not",
        ),
        (
            "an arm A decode with a block replayed",
            {"text": tsv(edit=lambda r: {**r, "replayed": "1"} if r["arm"] == "A" else r)},
            "blocks replayed, not 0",
        ),
        (
            "a NaN wall",
            {"text": tsv(edit=lambda r: {**r, "wall_s": "nan"} if r["round"] == "2" else r)},
            "not finite",
        ),
        (
            "a check with a mismatch",
            {"text": tsv(checks=lambda c: {**c, "mismatches": "1"} if c["round"] == "5" else c)},
            "mismatches",
        ),
        (
            "a check with another CRC",
            {"text": tsv(checks=lambda c: {**c, "crc": "deadbeef"} if c["round"] == "5" else c)},
            "not the reference's",
        ),
        (
            "a check missing",
            {"text": tsv(checks=lambda c: None if c["round"] == "9" else c)},
            "round 9: no check",
        ),
        (
            "the store with a conflict",
            {
                "text": tsv(
                    meta=lambda m: {
                        **m,
                        "detail": m["detail"].replace("conflicts=0", "conflicts=2"),
                    }
                )
            },
            "conflicts 2",
        ),
        ("no meta line", {"text": tsv(meta=lambda m: None)}, "0 meta lines"),
        ("other columns", {"text": "kind\tround\n"}, "not the harness's columns"),
        (
            "a decode twice",
            {"text": tsv() + "\t".join(tsv().splitlines()[2].split("\t")) + "\n"},
            "twice",
        ),
    ]:
        text = kw.pop("text", tsv())
        res.append(check(f"{name}: VOID", text, 1, ["VOID", phrase], absent=["DECISION"], **kw))
    failed = res.count(False)
    print(f"\n{len(res) - failed} of {len(res)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
