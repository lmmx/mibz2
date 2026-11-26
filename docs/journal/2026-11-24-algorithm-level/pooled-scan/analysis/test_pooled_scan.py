#!/usr/bin/env python3
"""Checks of pooled_scan.py on synthetic runs with known effects: a pooled scan that captures
most of the room behind C (CONTINUE), each of K1 to K4 alone (KILL), the VOIDs, and the
identities: a pooled trace failing only the block-indexed ones is complete, one failing a
timeline is INCOMPLETE, and an unpooled trace failing a block-indexed one is INCOMPLETE.

    python3 docs/journal/2026-11-24-algorithm-level/pooled-scan/analysis/test_pooled_scan.py
"""

import copy
import io
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import pooled_scan as ps  # noqa: E402

BLOCKS, TB, FCPU, NB = 12, 3, 4, 1724
I9 = "Model name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
ACT = ["A2", "A3", "job_wait", "pool_find", "pool_copy", "pool_wait", "resid"]
IDS = ["threads", "timelines partitioned", "flow: x", "linear order: x", "forced flow: x",
       "conservation: x", "admission: x", "Little's law: x", "utilisation: x"]  # fmt: skip


def plan():
    cells = [{"section": "one", "k": 1, "arm": a} for a in ("none", "P")]
    cells += [{"section": "kill", "k": k, "arm": a} for k in (6, 7, 8, 9, 10) for a in ps.ARMS]
    return {"cells": cells, "trace_blocks": TB, "finder_cpu": FCPU, "judged": [8, 9]}


# the walls at 8 and 9 (ms): S binds behind C; P captures most of SC's room behind C
BASE = {"none": 194.0, "P": 190.0, "C": 183.0, "PC": 140.0, "SC": 125.0, "S": 188.0}


def records(model=None, rss=None, fail_ids=None, pooled=None):
    model = model or {}
    rng = random.Random(7)
    p = plan()
    out = []
    for ci, c in enumerate(p["cells"]):
        k, a = c["k"], c["arm"]
        for b in range(BLOCKS):
            for on in (0, 1) if b < TB else (0,):
                base = 1450.0 if k == 1 else model.get((k, a), BASE[a])
                w = base * math.exp(rng.gauss(0, 0.004)) / 1e3
                pl = a in ps.POOLED and k >= 2
                st = {"dispatched": NB if pl else 0, "accepted": NB if pl else 0, "rejected": 0,
                      "fallbacks": 0, "degraded": 0, "copied": 47_446_638 if pl else 0}  # fmt: skip
                if pooled and (k, a) == pooled[0]:
                    st.update(pooled[1])
                x = {"kind": "decode", "cell": ci, "block": b, "on_arm": on, "on": on, "k": k,
                     "arm": a, "section": c["section"], "wall_s": w, "bytes": ps.OUT_BYTES,
                     "check_pass": {"identical": True}, "pooled": st, "user_s": w * k, "sys_s": 0.01,
                     "peak_kb": (rss or {}).get(a, 700_000), "blocks": NB}  # fmt: skip
                if on:
                    bad = (fail_ids or {}).get((k, a), [])
                    x["identities"] = [{"name": n, "ok": n not in bad} for n in IDS]
                    x["activities"] = ACT
                    life = int(w * 1e9)
                    st_ = lambda vals: [vals + [0], [0] * len(ACT), [0] * len(ACT)]  # noqa: E731
                    if pl:
                        scanner = st_([0, 0, 0, 12_000_000, 6_500_000, life // 2])
                        wk = st_([9_000_000 // k * 8, 70_000_000 // k * 8, life // 30, 0, 0, 0])
                    else:
                        scanner = st_([26_000_000, 240_000_000, 0, 0, 0, 0])
                        wk = st_([0, 0, life // 5, 0, 0, 0])
                    x["threads"] = [{"role": "Scanner", "life_ns": life, "states": scanner}] + [
                        {"role": "Worker", "life_ns": life, "states": wk} for _ in range(k)
                    ]
                    x["queue_syms_bytes"] = 0 if pl else 300 << 20
                out.append(x)
    return p, out


FINDER = {"kind": "finder", "repeats": 11, "ns": [5_200_000] * 11, "median_ns": 5_200_000,
          "interval_ns": [5_100_000, 5_400_000], "interval_ranks": [2, 10],
          "interval_coverage": 0.9883, "cpu": FCPU, "blocks": NB, "ends": 48, "avx2": True,
          "gb_s": 9.1}  # fmt: skip
EXACT = "the exactness phase: PASSED\n"


def run(p, recs, finder=FINDER, exact=EXACT, lscpu=I9):
    buf = io.StringIO()
    code, verdict, rows = ps.analyse(p, recs, finder, exact, lscpu, BLOCKS, out=buf)
    return code, verdict, buf.getvalue()


def check(name, cond, text=""):
    if not cond:
        print(text)
        raise SystemExit(f"FAIL: {name}")
    print(f"ok: {name}")


def main():
    p, recs = records()
    code, v, t = run(p, recs)
    check("a pooled scan with the room behind C: CONTINUE", (code, v) == (0, "CONTINUE"), t)
    check(
        "its traces complete (the block-indexed identities exempt in P)", "INCOMPLETE" not in t, t
    )
    check("the copy reported beside its estimate", "pool_copy" in t and "estimated about 5" in t, t)

    # K1: PC barely under C, so short of sqrt(SC/C) (about 0.826 of C)
    p, recs = records({(8, "PC"): 175.0})
    code, v, t = run(p, recs)
    check("K1 at 8: KILL", v == "KILL" and "K1 at 8" in t and "K1 at 9" not in t, t)
    # K1 decided at a count not judged: not a kill
    p, recs = records({(10, "PC"): 182.0})
    check("K1 at 10 alone: CONTINUE", run(p, recs)[1] == "CONTINUE")

    # K2: P 5% slower than none at 9
    p, recs = records({(9, "P"): 204.0})
    code, v, t = run(p, recs)
    check("K2 at 9: KILL", v == "KILL" and "K2 at 9" in t, t)
    p, recs = records({(9, "P"): 198.0})
    check("P 2% slower at 9 (under K2's 3%): CONTINUE", run(p, recs)[1] == "CONTINUE")

    # K3: P's peak RSS 15% over none's
    p, recs = records(rss={"P": 805_000, "none": 700_000})
    code, v, t = run(p, recs)
    check("K3: KILL", v == "KILL" and "K3 at 6" in t, t)

    # K4: the finder at 11 ms
    p, recs = records()
    f = dict(FINDER, median_ns=11_000_000)
    check("K4: KILL", run(p, recs, finder=f)[1] == "KILL")

    # VOIDs
    p, recs = records()
    check("not the i9: VOID", run(p, recs, lscpu="Model name: AMD EPYC\n")[1] == "VOID")
    check("the exactness phase failed: VOID", run(p, recs, exact="FAILED\n")[1] == "VOID")
    check("K4's pass missing: VOID", run(p, recs, finder=None)[1] == "VOID")
    check("K4's pass off its CPU: VOID", run(p, recs, finder=dict(FINDER, cpu=0))[1] == "VOID")
    check(
        "K4's pass missing a block: VOID",
        run(p, recs, finder=dict(FINDER, blocks=NB - 1))[1] == "VOID",
    )
    r2 = copy.deepcopy(recs)
    del r2[5]
    check("a decode missing: VOID", run(p, r2)[1] == "VOID")
    r2 = copy.deepcopy(recs)
    r2[3]["check_pass"]["identical"] = False
    check("a check pass not identical: VOID", run(p, r2)[1] == "VOID")
    p, recs = records(pooled=((8, "P"), {"fallbacks": 3}))
    check("a P decode at 8 not wholly pooled: VOID", run(p, recs)[1] == "VOID")
    p, recs = records(pooled=((1, "P"), {"dispatched": 5}))
    check("P at 1 off the shipped path: VOID", run(p, recs)[1] == "VOID")

    # identities
    p, recs = records(fail_ids={(8, "P"): ["flow: x", "forced flow: x", "utilisation: x"]})
    t = run(p, recs)[2]
    check("P failing block-indexed identities only: complete", "INCOMPLETE" not in t, t)
    p, recs = records(fail_ids={(8, "P"): ["timelines partitioned"]})
    code, v, t = run(p, recs)
    check(
        "P failing a timeline: INCOMPLETE, the verdict stands",
        "INCOMPLETE" in t and v == "CONTINUE",
        t,
    )
    p, recs = records(fail_ids={(8, "C"): ["flow: x"]})
    check("C failing flow: INCOMPLETE", "INCOMPLETE" in run(p, recs)[2])
    p, recs = records(fail_ids={(1, "P"): ["flow: x"]})
    check("P at 1 (the shipped path) failing flow: INCOMPLETE", "INCOMPLETE" in run(p, recs)[2])
    print("all checks passed")


if __name__ == "__main__":
    main()
