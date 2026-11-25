#!/usr/bin/env python3
"""Checks of ideal_parts.py (the slice-A analysis) on synthetic runs with known effects: at 10
workers the scan has ROOM, emission none alone but room behind the scan (MASKED), and the two
INTERACT; at 6 neither part matters (NO ROOM, ADDITIVE); and each VOID and INCOMPLETE.

    python3 docs/journal/2026-11-24-algorithm-level/ideal-parts/results/analysis/test_ideal_parts.py
"""

import io
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ideal_parts as ip  # noqa: E402

BLOCKS, TB, SCPU = 12, 2, 4
I9 = "Model name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
NA = len(ip.ACT)


def plan():
    cells = [{"section": "pinned", "k": k, "arm": a, "pinned": 1} for k in (6, 10) for a in ip.ARMS]
    cells += [{"section": "link", "k": 8, "arm": a} for a in ("none", "SE")]
    cells += [{"section": "pinned", "k": 8, "arm": a, "pinned": 1} for a in ip.ARMS]
    return {
        "cells": cells,
        "trace_blocks": TB,
        "scanner_cpu": SCPU,
        "link": {"k": 8, "arms": ["none", "SE"]},
    }


# the walls' model (ratios to all real): at 10 the scan 0.80; emission 1.00 alone, 0.85 behind
# the scan; at 6 and 8 nothing
MODEL = {10: {"none": 1.0, "S": 0.80, "E": 1.0, "SE": 0.80 * 0.85}}


def threads(arm, scanner_cpus):
    def th(role, acts, cpus=None):
        states = [[0] * NA for _ in range(3)]
        for a, ns in acts.items():
            states[1][ip.ACT.index(a)] = ns
        return {"role": role, "states": states, "visits": [0] * NA, "cpus": cpus or {}}

    scan = {} if "S" in arm else {"A2": 5_000_000, "A3": 60_000_000}
    emit = {} if "E" in arm else {"A7": 140_000_000}
    return [th("Scanner", scan, scanner_cpus), th("Worker", emit), th("Reader", {})]


def records(fail=None, seed=7):
    rnd = random.Random(seed)
    p = plan()
    out = []
    for b in range(BLOCKS):
        base = 0.2 * math.exp(rnd.gauss(0, 0.01))
        for ci, c in enumerate(p["cells"]):
            for on in (0, 1) if b < TB else (0,):
                r = MODEL.get(c["k"], {}).get(c["arm"], 1.0)
                x = {"kind": "decode", "k": c["k"], "on": on, "on_arm": on, "cell": ci, "block": b,
                     "section": c["section"], "arm": c["arm"], "wall_s": base * r * math.exp(rnd.gauss(0, 0.005)),
                     "bytes": ip.OUT_BYTES, "majflt": 0, "peak_kb": 300_000, "identities_ok": True,
                     "check_pass": {"identical": True}, "pmu": {"cycles": [4.9e9], "ref-cycles": [3.7e9]},
                     "oracle_store": {"symbol_bytes": 40 << 20, "code_bytes": 10 << 20},
                     "replayed": 431 if "S" in c["arm"] else 0, "supplied_blocks": 233 if "E" in c["arm"] else 0,
                     "deliver_ns": 1_000_000, "consume_ns": 2_000_000}  # fmt: skip
                if on:
                    x["threads"] = threads(
                        c["arm"], {str(SCPU): 1} if c.get("pinned") else {"3": 1}
                    )
                out.append(x)
    if fail:
        fail(out)
    return p, out


def run(p, recs, lscpu=I9):
    buf = io.StringIO()
    code, rows = ip.analyse(p, recs, lscpu, BLOCKS, out=buf)
    return code, buf.getvalue(), rows


def label(rows, k, measure):
    return next((r["label"] for r in rows if r["k"] == k and r["measure"] == measure), None)


def main():
    res = []

    def check(name, ok, why=""):
        res.append(ok)
        print(f"{'ok  ' if ok else 'FAIL'} {name}{'' if ok else ': ' + str(why)[:700]}")

    code, text, rows = run(*records())
    check(
        "a whole run: exit 0, no VOID",
        code == 0 and not any(ln.startswith("VOID") for ln in text.splitlines()),
        text[:600],
    )
    check(
        "k 10: the scan has ROOM (0.80)",
        label(rows, 10, "room S") == "ROOM",
        [r for r in rows if r["k"] == 10],
    )
    check("k 10: emission MASKED (no room alone, 0.85 behind the scan)", label(rows, 10, "room E") == "MASKED",
          [r for r in rows if r["k"] == 10])  # fmt: skip
    check(
        "k 10: the parts INTERACT", label(rows, 10, "interaction S.E") == "INTERACTS", text[-1500:]
    )
    check("k 6: neither part has room", label(rows, 6, "room S") == "NO ROOM" and label(rows, 6, "room E") == "NO ROOM",
          [r for r in rows if r["k"] == 6])  # fmt: skip
    check(
        "k 6: ADDITIVE",
        label(rows, 6, "interaction S.E") == "ADDITIVE",
        [r for r in rows if r["k"] == 6],
    )
    fl = next(r for r in rows if r["k"] == 10 and r["measure"] == "floor")
    check("k 10: the floor is 0.80 x 0.85", abs(fl["value"] - 0.68) < 0.01, fl)
    check(
        "the removed parts read 0% in their ideal arms",
        "S: 0.00%" in text and "E: 0.00%" in text,
        text[:900],
    )

    def bad_check(r):
        r[5]["check_pass"] = {"identical": False}

    code, text, _ = run(*records(fail=bad_check))
    check("VOID: a check pass not byte-identical", code == 1 and "not byte-identical" in text)

    def scan_left(r):
        for x in r:
            if x.get("on") == 1 and x["arm"] == "S" and x["k"] == 10:
                x["threads"][0]["states"][1][ip.ACT.index("A3")] = 10_000_000

    code, text, _ = run(*records(fail=scan_left))
    check(
        "VOID: the scan's work over 5% in its ideal arm",
        code == 1 and "S's work" in text,
        text[:600],
    )

    def pin_off(r):
        for x in r:
            if x.get("on") == 1 and x["k"] == 10:
                x["threads"][0]["cpus"] = {"9": 1}

    code, text, _ = run(*records(fail=pin_off))
    check("VOID: the pin broken in over 10% of pinned traces", code == 1 and "pin broken" in text)
    code, text, _ = run(*records(fail=lambda r: r.pop(3)))
    check("VOID: a decode missing", code == 1 and "0 decodes" in text)
    code, text, _ = run(*records(), lscpu="Model name: Xeon\n")
    check("VOID: not the i9", code == 1 and "not the i9" in text)

    def ident(r):
        next(x for x in r if x.get("on") == 1)["identities_ok"] = False

    code, text, _ = run(*records(fail=ident))
    check("INCOMPLETE: an identity failed (no VOID)", code == 0 and "INCOMPLETE" in text)
    code, text, _ = run(*records())
    check(
        "the link reported", "the unpinned link at 8: all ideal over all real" in text, text[-800:]
    )
    failed = sum(1 for x in res if not x)
    print(f"\n{len(res) - failed} of {len(res)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
