#!/usr/bin/env python3
"""Checks of bench/scan_profile.py, bench-scan-profile's report, on synthetic records: each
classification (SCANNER-BOUND, QUEUE-BOUND, INPUT-BOUND, MIXED), DECODE-DOMINATED and the largest
component named, PERTURBED, the least-squares fit recovering a known cost a symbol and a switch,
and each VOID"""

import io
import json
import os
import random
import sys
import tempfile
from contextlib import redirect_stdout

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import scan_profile as spr  # noqa: E402

I9 = "Model name:                           Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
OUT = spr.WIKIDATA_X4_OUT
FIELDS = [
    "index", "bits", "syms", "selectors", "switches", "tables", "slot_wait_ns", "read_ns",
    "input_wait_ns", "header_ns", "bitmap_ns", "selectors_ns", "tables_ns", "prep_ns",
    "decode_ns", "send_ns",
]  # fmt: skip
N = 40


def records(work=0.9, slot=0.02, inp=0.01, decode_share=0.8, overhead=1.0, edit=None, rounds=12,
            ns_sym=2.0, ns_switch=50.0, counts=(1, 6, 10), shape=None):  # fmt: skip
    """A run's records: at each count the scanner's work, slot waits and input waits as these
    shares of a 0.2 s wall; the work's Huffman decode share `decode_share`, its decode ns a
    symbol `ns_sym` and a switch `ns_switch`; timed walls `overhead` times the untimed. `shape`,
    if given, maps a count k to (wall, the scanner's work, a worker's busy time), in seconds"""
    rng = random.Random(3)
    lines = [json.dumps({"kind": "meta", "block_fields": FIELDS, "input_bytes": 47_000_000})]
    for r in range(rounds):
        for w in counts:
            # the blocks first (the decode's cost by its symbols and switches), then the wall
            # that makes the scanner's work `work` of it
            spec = [(100_000 + rng.randrange(50_000), 300 + rng.randrange(300)) for _ in range(N)]
            decs = [ns_sym * sy + ns_switch * sw for sy, sw in spec]
            works = [d / decode_share for d in decs]
            base = sum(works) / work / 1e9
            busy = 0.7 * base
            if shape:
                base, work_s, busy = shape(w)
                scale = work_s * 1e9 / sum(works)
                works = [x * scale for x in works]
                decs = [x * scale for x in decs]
            for timed in (True, False):
                wall = base * (1 + rng.gauss(0, 0.002)) * (overhead if timed else 1.0)
                x = {"kind": "decode", "round": r, "workers": w, "timed": timed, "order": 0,
                     "wall_s": wall, "bytes": OUT}  # fmt: skip
                if timed:
                    wns = base * 1e9
                    blocks = []
                    for i, ((syms, sw), dec, wb) in enumerate(zip(spec, decs, works)):
                        rest = wb - dec
                        inw = inp * wns / N
                        blocks.append([i, 400_000, syms, 1000, sw, 5, int(slot * wns / N),
                                       int(wb - rest * 0.1 + inw), int(inw),
                                       int(rest * 0.05), int(rest * 0.05), int(rest * 0.2),
                                       int(rest * 0.4), int(rest * 0.2), int(dec),
                                       int(rest * 0.1)])  # fmt: skip
                    x["n"] = len(blocks)
                    x["indices_ok"] = True
                    x["sums"] = [sum(col) for col in zip(*blocks)]
                    if r < 2:
                        x["blocks"] = blocks
                    x["workers_rec"] = [[N // w, int(wns - busy * 1e9), int(wns)] for _ in range(w)]
                x = edit(x) if edit else x
                if x is not None:
                    lines.append(json.dumps(x))
    return "\n".join(lines) + "\n"


def judge(text, lscpu=I9, extra=("--workers", "1,6,10", "--rounds", "12")):
    d = tempfile.mkdtemp()
    p, q = os.path.join(d, "runs.jsonl"), os.path.join(d, "lscpu.txt")
    with open(p, "w") as f:
        f.write(text)
    with open(q, "w") as f:
        f.write(lscpu)
    buf = io.StringIO()
    with redirect_stdout(buf):
        code = spr.main(["scan_profile.py", p, "--lscpu", q, *extra])
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
    res.append(
        check(
            "work 0.90, slot 0.02: SCANNER-BOUND, decode-dominated",
            records(),
            0,
            [
                "CLASSIFICATION at 10 workers: SCANNER-BOUND",
                "largest component: decode",
                "DECODE-DOMINATED",
            ],
            absent=["PERTURBED"],
        )  # fmt: skip
    )
    res.append(
        check(
            "slot waits 0.3: QUEUE-BOUND",
            records(work=0.6, slot=0.3),
            0,
            ["CLASSIFICATION at 10 workers: QUEUE-BOUND"],
        )  # fmt: skip
    )
    res.append(
        check(
            "input waits 0.2: INPUT-BOUND",
            records(work=0.7, slot=0.02, inp=0.2),
            0,
            ["CLASSIFICATION at 10 workers: INPUT-BOUND"],
        )  # fmt: skip
    )
    res.append(
        check(
            "work 0.75, slot 0.08: MIXED",
            records(work=0.75, slot=0.08),
            0,
            ["CLASSIFICATION at 10 workers: MIXED"],
        )  # fmt: skip
    )
    res.append(
        check(
            "decode 0.20 of the work: not decode-dominated, tables the largest",
            records(decode_share=0.2),
            0,
            ["SCANNER-BOUND", "largest component: tables"],
            absent=["DECODE-DOMINATED"],
        )  # fmt: skip
    )
    res.append(
        check("timed decodes 5% slower: PERTURBED", records(overhead=1.05), 0, ["PERTURBED"])
    )
    res.append(
        check(
            "the fit recovers 2.00 ns a symbol and 50 ns a switch (decode share 1.0)",
            records(decode_share=1.0),
            0,
            ["2.00 ns a symbol", "50.0 ns a switch"],
        )  # fmt: skip
    )
    # the grid: a serial stage of 0.05 s feeding k workers of 0.40 s of work in all (a / k each),
    # 0.01 s of fixed cost; rho = (0.40 / k) / 0.05 falls through 1 at k = 8, and the model's
    # shift point is a / b = 8
    grid = (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16, 20)
    shape = lambda k: (0.01 + max(0.05, 0.40 / k), 0.05, 0.40 / k)  # noqa: E731
    text = records(counts=grid, shape=shape, rounds=4)
    got, out = judge(text, extra=("--workers", ",".join(map(str, grid)), "--rounds", "4"))
    import re

    kstar = re.search(r"k\* = ([0-9.]+)", out)
    ok = (
        got == 0
        and "crossover (rho falls through 1, interpolated in k): k = 8.00" in out
        and kstar is not None
        and abs(float(kstar.group(1)) - 8) <= 0.1
        and "## The grid" in out
    )
    print(
        f"{'ok  ' if ok else 'FAIL'} the grid: the crossover at k = 8, the model's k* within 0.1 of 8"
    )
    if not ok:
        print(out)
    res.append(ok)
    text = records(counts=(1, 2, 4), shape=lambda k: (0.5 / k, 0.01, 0.5 / k), rounds=4)
    res.append(
        check(
            "the scanner never binds in the grid: no crossover",
            text,
            0,
            ["crossover (rho falls through 1, interpolated in k): none in the grid"],
            extra=("--workers", "1,2,4", "--judged", "4", "--rounds", "4"),
        )  # fmt: skip
    )
    for name, text, phrase, kw in [
        ("not the i9", records(), "not the i9-10900K", {"lscpu": "Model name: AMD EPYC\n"}),
        ("a decode missing", records(edit=lambda x: None if (x["round"], x["workers"], x["timed"]) == (4, 6, False) else x),
         "round 4 workers 6 untimed: missing", {}),
        ("output short", records(edit=lambda x: {**x, "bytes": 5} if x["round"] == 1 else x), "bytes, not", {}),
        ("a block missing (the harness's check)", records(edit=lambda x: {**x, "indices_ok": False} if x.get("sums") and x["round"] == 5 else x),
         "not one each of", {}),
        ("a block record missing (round 0)", records(edit=lambda x: {**x, "blocks": x["blocks"][1:]} if x.get("blocks") and x["round"] == 0 else x),
         "block records not", {}),
        ("a worker record missing", records(edit=lambda x: {**x, "workers_rec": x["workers_rec"][1:]} if x.get("workers_rec") and x["workers"] == 6 else x),
         "worker records, not 6", {}),
        ("a NaN wall", records(edit=lambda x: {**x, "wall_s": float("nan")} if x["round"] == 3 else x), "wall", {}),
        ("no meta", "\n".join(records().splitlines()[1:]) + "\n", "0 meta lines", {}),
    ]:  # fmt: skip
        res.append(
            check(f"{name}: VOID", text, 1, ["VOID", phrase], absent=["CLASSIFICATION"], **kw)
        )
    failed = res.count(False)
    print(f"\n{len(res) - failed} of {len(res)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
