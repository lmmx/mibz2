#!/usr/bin/env python3
"""Checks of ideal_matrix.py (the full matrix's analysis) on synthetic runs with known effects. At 10
workers: the scan has ROOM (0.80); emission none alone but 0.85 behind the scan (MASKED, S and E
INTERACT); construction ROOM (0.90); M, R and D nothing. At 6 nothing matters (NO ROOM,
ADDITIVE). The fraction's effects are recovered exactly; and each VOID and INCOMPLETE.

    python3 docs/journal/2026-11-24-algorithm-level/ideal-parts/results/analysis/test_ideal_matrix.py
"""

import importlib.util
import io
import math
import os
import random
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import ideal_matrix as im  # noqa: E402

# the planner, bench/ideal_parts.py (its name is slice A's analysis's beside this file)
_spec = importlib.util.spec_from_file_location(
    "ideal_parts_planner",
    os.path.join(HERE, "..", "..", "..", "..", "..", "..", "bench", "ideal_parts.py"),
)
planner = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(planner)

BLOCKS, TB, SCPU = 12, 2, 4
I9 = "Model name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
NA = len(im.ACT)


def plan():
    cells = [{"section": "pinned", "k": k, "arm": a, "pinned": 1} for k in (6, 10) for a in im.ARMS]
    cells += [{"section": "link", "k": 8, "arm": a} for a in ("none", im.ALL)]
    cells += [{"section": "pinned", "k": 8, "arm": a, "pinned": 1} for a in im.ARMS]
    return {
        "cells": cells,
        "trace_blocks": TB,
        "scanner_cpu": SCPU,
        "link": {"k": 8, "arms": ["none", im.ALL]},
    }


def ratio(k, arm):
    """the walls' model, a ratio to all real: at 10 the scan 0.80, emission 0.85 behind the scan
    alone, construction 0.90; at 6 and 8 nothing"""
    if k != 10:
        return 1.0
    r = 1.0
    if "S" in arm:
        r *= 0.80
        if "E" in arm:
            r *= 0.85
    if "C" in arm:
        r *= 0.90
    return r


WORK_NS = {"A2": 5e6, "A3": 6e7, "A7": 1.4e8, "A6a": 2e7, "A6b": 6e7, "A6c": 2e7, "A5": 9e7,
           "A9": 7e7, "copy_out": 4e7}  # fmt: skip


def threads(arm, scanner_cpus):
    def th(role, acts, cpus=None):
        states = [[0] * NA for _ in range(3)]
        for a, ns in acts.items():
            states[1][im.ACT.index(a)] = int(ns)
        return {"role": role, "states": states, "visits": [0] * (NA - 1), "cpus": cpus or {}}

    def own(part):
        role, acts = im.WORK[part]
        return {} if part in arm else {a: WORK_NS[a] for a in acts}

    hand = {f"hand_{p}": 3e6 for p in "ECMR" if p in arm}
    scan = {**own("S"), **({"hand_S": 2e6} if "S" in arm else {})}
    work = {**own("E"), **own("C"), **own("M"), **own("R"), **hand}
    return [th("Scanner", scan, scanner_cpus), th("Worker", work), th("Reader", own("D"))]


def records(fail=None, seed=7):
    rnd = random.Random(seed)
    p = plan()
    out = []
    for b in range(BLOCKS):
        base = 0.2 * math.exp(rnd.gauss(0, 0.01))
        for ci, c in enumerate(p["cells"]):
            for on in (0, 1) if b < TB else (0,):
                d = "D" in c["arm"]
                x = {"kind": "decode", "k": c["k"], "on": on, "on_arm": on, "cell": ci, "block": b,
                     "section": c["section"], "arm": c["arm"],
                     "wall_s": base * ratio(c["k"], c["arm"]) * math.exp(rnd.gauss(0, 0.004)),
                     "bytes": 0 if d else im.OUT_BYTES, "majflt": 0, "peak_kb": 800_000, "identities_ok": True,
                     "replays": {"D_discarded_bytes": im.OUT_BYTES if d else 0},
                     "check_pass": {"identical": True}, "pmu": {"cycles": [4.9e9], "ref-cycles": [3.7e9]},
                     "oracle_store": {"symbol_bytes": 149 << 20, "code_bytes": 10 << 20, "C": {"bytes": 114 << 20},
                                      "M": {"bytes": 32 << 20}, "R": {"bytes": 200 << 20}, "total_bytes": 505 << 20}}  # fmt: skip
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
    code, rows = im.analyse(p, recs, lscpu, BLOCKS, out=buf)
    return code, buf.getvalue(), rows


def get(rows, k, measure, key="label"):
    return next((r[key] for r in rows if r["k"] == k and r["measure"] == measure), None)


def main():
    res = []

    def check(name, ok, why=""):
        res.append(ok)
        print(f"{'ok  ' if ok else 'FAIL'} {name}{'' if ok else ': ' + str(why)[:900]}")

    check("the design: 32 fraction arms (even), 12 added (singles, five-part), 44 in all",
          len(im.FRACTION) == 32 and all(len(a.replace("none", "")) % 2 == 0 for a in im.FRACTION)
          and len(im.AUGMENT) == 12 and len(set(im.ARMS)) == 44, im.ARMS)  # fmt: skip
    check(
        "the planner's arms are the analysis's",
        list(planner.ARMS) == im.ARMS and planner.LINK == (8, ("none", im.ALL)),
        (planner.ARMS, planner.LINK),
    )
    cols = {e: [im.sign(e, a) for a in im.FRACTION] for e in im.MAIN + im.PAIRS + im.TRIPLES}
    orth = all(
        sum(x * y for x, y in zip(cols[e], cols[f])) == 0 for e in cols for f in cols if e < f
    )
    check(
        "the 31 effect columns are orthogonal on the fraction (resolution VI)",
        orth and len(cols) == 31,
    )

    code, text, rows = run(*records())
    check(
        "a whole run: exit 0, no VOID",
        code == 0 and not any(ln.startswith("VOID") for ln in text.splitlines()),
        text[:900],
    )
    check(
        "k 10: the scan has ROOM (0.80)",
        get(rows, 10, "room S") == "ROOM",
        [r for r in rows if r["k"] == 10][:8],
    )
    check("k 10: construction has ROOM (0.90)", abs(get(rows, 10, "room C", "value") - 0.90) < 0.005
          and get(rows, 10, "room C") == "ROOM")  # fmt: skip
    check("k 10: emission MASKED behind the scan", get(rows, 10, "room E") == "MASKED" and "MASKED (S)" in text,
          [r for r in rows if r["k"] == 10 and "E" in r["measure"]][:6])  # fmt: skip
    check("k 10: M, R and D NO ROOM", all(get(rows, 10, f"room {p}") == "NO ROOM" for p in "MRD"))
    check("k 10: S and E INTERACT; S and C ADDITIVE",
          get(rows, 10, "interaction S.E") == "INTERACTS" and get(rows, 10, "interaction S.C") == "ADDITIVE")  # fmt: skip
    qse = get(rows, 10, "interaction S.E", "value")
    check("q_SE is log(0.85)/4, the 2^2's coefficient", abs(qse - math.log(0.85) / 4) < 0.003, qse)
    qs = get(rows, 10, "q S", "value")
    check(
        "q_S is log(0.80)/2 + log(0.85)/4",
        abs(qs - (math.log(0.8) / 2 + math.log(0.85) / 4)) < 0.003,
        qs,
    )
    check("q_C is log(0.90)/2, q_M nil", abs(get(rows, 10, "q C", "value") - math.log(0.9) / 2) < 0.003
          and abs(get(rows, 10, "q M", "value")) < 0.003)  # fmt: skip
    check("k 6: nothing has room, every pair ADDITIVE",
          all(get(rows, 6, f"room {p}") == "NO ROOM" for p in im.FACTORS)
          and all(get(rows, 6, f"interaction {e[0]}.{e[1]}") == "ADDITIVE" for e in im.PAIRS))  # fmt: skip
    fl = get(rows, 10, "floor", "value")
    check("k 10: the floor is 0.80 x 0.85 x 0.90", abs(fl - 0.8 * 0.85 * 0.9) < 0.01, fl)
    fs = get(rows, 10, "floor share C", "value")
    check(
        "k 10: C's share of the floor (all but C over all ideal) is 1/0.90",
        abs(fs - 1 / 0.9) < 0.01,
        fs,
    )
    check(
        "k 10: S's room with E ideal is 0.80 x 0.85",
        abs(get(rows, 10, "room S with E ideal", "value") - 0.68) < 0.01,
    )
    check(
        "the removed parts read 0% in their ideal arms",
        all(f"{p}: 0.00%" in text for p in im.FACTORS),
        text[:1200],
    )
    net = get(rows, 10, "room C net of handover", "value")
    check("C's net of handover is below its measured room (3 ms over 10 workers)",
          net is not None and net < get(rows, 10, "room C", "value"), net)  # fmt: skip

    def bad_check(r):
        r[5]["check_pass"] = {"identical": False}

    code, text, _ = run(*records(fail=bad_check))
    check("VOID: a check pass not byte-identical", code == 1 and "not byte-identical" in text)

    def c_left(r):
        for x in r:
            if x.get("on") == 1 and "C" in x["arm"] and x["k"] == 10:
                x["threads"][1]["states"][1][im.ACT.index("A6b")] = 60_000_000

    code, text, _ = run(*records(fail=c_left))
    check(
        "VOID: construction's own work over 5% in its ideal arms",
        code == 1 and "C's work" in text,
        text[:700],
    )

    def hand_only(r):
        for x in r:
            if x.get("on") == 1 and "M" in x["arm"]:
                x["threads"][1]["states"][1][im.ACT.index("hand_M")] = 500_000_000

    code, text, _ = run(*records(fail=hand_only))
    check("a large handover is no VOID (its own entry, apart from A5)", code == 0, text[:600])

    def d_lost(r):
        next(x for x in r if "D" in x["arm"] and x["on"] == 0)["replays"]["D_discarded_bytes"] = 0

    code, text, _ = run(*records(fail=d_lost))
    check("VOID: a D arm that delivered nothing", code == 1 and "0 bytes delivered" in text)

    def pin_off(r):
        for x in r:
            if x.get("on") == 1 and x["k"] == 10:
                x["threads"][0]["cpus"] = {"9": 1}

    code, text, _ = run(*records(fail=pin_off))
    check("VOID: the pin broken in over 10% of pinned traces", code == 1 and "pin broken" in text)
    code, text, _ = run(*records(fail=lambda r: r.pop(3)))
    check("VOID: a decode missing", code == 1 and "0 decodes" in text)

    def failed(r):
        r[7] = {"kind": "failed", "section": r[7]["section"], "arm": r[7]["arm"], "k": r[7]["k"],
                "block": r[7]["block"], "cell": r[7]["cell"], "on_arm": 0}  # fmt: skip

    code, text, _ = run(*records(fail=failed))
    check(
        "VOID: a decode failed (as when an ideal part was not wholly replayed)",
        code == 1 and "a decode failed" in text,
    )
    code, text, _ = run(*records(), lscpu="Model name: Xeon\n")
    check("VOID: not the i9", code == 1 and "not the i9" in text)

    def ident(r):
        next(x for x in r if x.get("on") == 1)["identities_ok"] = False

    code, text, _ = run(*records(fail=ident))
    check("INCOMPLETE: an identity failed (no VOID)", code == 0 and "INCOMPLETE" in text)
    code, text, _ = run(*records())
    check(
        "the link reported",
        re.search(r"unpinned link at 8: all ideal over all real \d\.\d{4} \[", text) is not None,
        text[-900:],
    )
    failed_n = sum(1 for x in res if not x)
    print(f"\n{len(res) - failed_n} of {len(res)} checks pass")
    return 1 if failed_n else 0


if __name__ == "__main__":
    sys.exit(main())
