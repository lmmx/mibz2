#!/usr/bin/env python3
"""Checks of bench/clock_mechanism.py: its plan (the fastest core pinned, the rest masked, the spin
sized from the calibration), and its report on synthetic decodes: an arm that never runs at the
low clock is a CONTROL, one that runs at it more is WORSE, and each VOID."""

import argparse
import io
import json
import math
import os
import random
import sys
import tempfile
from contextlib import redirect_stdout

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import clock_mechanism as cm  # noqa: E402

I9 = "Model name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
OUT = 4 * 209_700_196
N = 48


def topo(d):
    p = os.path.join(d, "lscpu-e.txt")
    with open(p, "w") as f:
        f.write("CPU NODE SOCKET CORE L1d:L1i:L2:L3 ONLINE MAXMHZ MINMHZ MHZ\n")
        for c in range(20):
            f.write(f"{c} 0 0 {c % 10} {c % 10}:{c % 10}:{c % 10}:0 yes 5300 800 4900\n")
    cores = os.path.join(d, "cores.json")
    json.dump({str(c): {"iterations": 200_000_000, "repeats": [[0, 0, 20_000_000 - (5_000_000 if c == 4 else 0)]]}
               for c in range(10)}, open(cores, "w"))  # fmt: skip
    return p, cores


def records(shares, fail=None, seed=5):
    rnd = random.Random(seed)
    recs = []
    for b in range(N):
        for arm, share in shares.items():
            g = 4.09 if rnd.random() < share else 4.91
            recs.append({"kind": "decode", "arm": arm, "block": b, "k": 10, "bytes": OUT, "majflt": 0,
                         "wall_s": 0.2 * 4.91 / g, "pmu": {"cycles": [g / 3.7 * 1e9], "ref-cycles": [1e9]}})  # fmt: skip
    if fail:
        fail(recs)
    return recs


def report(plan, recs, lscpu=I9):
    d = tempfile.mkdtemp()
    pp, rp, lp = (os.path.join(d, n) for n in ("plan.json", "runs.jsonl", "lscpu.txt"))
    json.dump(plan, open(pp, "w"))
    open(rp, "w").write("".join(json.dumps(r) + "\n" for r in recs))
    open(lp, "w").write(lscpu)
    a = argparse.Namespace(plan=pp, runs=rp, lscpu=lp, out=d, blocks=N, out_bytes=OUT)
    buf = io.StringIO()
    with redirect_stdout(buf):
        code = cm.report(a)
    return code, buf.getvalue(), json.load(open(os.path.join(d, "clock.json")))


def main():
    res = []

    def check(name, ok, why=""):
        res.append(ok)
        print(f"{'ok  ' if ok else 'FAIL'} {name}{'' if ok else ': ' + str(why)[:600]}")

    d = tempfile.mkdtemp()
    lse, cores = topo(d)
    a = argparse.Namespace(lscpu_e=lse, cores=cores, out=os.path.join(d, "plan.json"))
    with redirect_stdout(io.StringIO()):
        p = cm.plan(a)
    check("the plan: the fastest core's first CPU, its core masked off, a 100 ms spin",
          p["scanner_cpu"] == 4 and 4 not in p["mask"] and 14 not in p["mask"] and len(p["mask"]) == 18
          and p["spin_iterations"] == int(0.1e9 * 200_000_000 / 15_000_000) and p["cut_ghz"] == 4.50,
          p)  # fmt: skip
    code, text, out = report(
        p, records({"S0": 0.3, "S0.2": 0.3, "S1": 0.3, "S3": 0.3, "SPIN": 0.0})
    )
    check(
        "an arm never at the low clock: CONTROL",
        code == 0 and out["SPIN"]["label"] == "CONTROL",
        text,
    )
    check("the reference is labelled so", out["S1"]["label"] == "(ref)", out["S1"])
    code, text, out = report(
        p, records({"S0": 0.2, "S0.2": 0.8, "S1": 0.2, "S3": 0.2, "SPIN": 0.2})
    )
    check("an arm at the low clock more: WORSE", out["S0.2"]["label"] == "WORSE", text)
    code, text, out = report(
        p, records({"S0": 0.25, "S0.2": 0.25, "S1": 0.25, "S3": 0.25, "SPIN": 0.25})
    )
    check(
        "like arms: not CONTROL, not WORSE",
        all(out[a]["label"] in ("NO-EFFECT", "UNRESOLVED") for a in ("S0", "S0.2", "S3", "SPIN")),
        out,
    )
    code, text, _ = report(
        p,
        records({"S0": 0.3, "S0.2": 0.3, "S1": 0.3, "S3": 0.3, "SPIN": 0.0}),
        lscpu="Model name: Xeon\n",
    )
    check("VOID: not the i9", code == 1 and "not the i9" in text)
    code, text, _ = report(
        p,
        records(
            {"S0": 0.3, "S0.2": 0.3, "S1": 0.3, "S3": 0.3, "SPIN": 0.0}, fail=lambda r: r.pop(7)
        ),
    )
    check("VOID: a decode missing", code == 1 and "0 decodes" in text, text[:300])
    code, text, _ = report(
        p,
        records(
            {"S0": 0.3, "S0.2": 0.3, "S1": 0.3, "S3": 0.3, "SPIN": 0.0},
            fail=lambda r: [x.update(pmu={}) for x in r[:40]],
        ),
    )
    check(
        "VOID: the clock unread in over 10% of decodes",
        code == 1 and "without the clock" in text,
        text[:300],
    )
    lo, hi = cm.newcombe(10, 48, 10, 48)
    check(
        "Newcombe's interval of a nil difference is symmetric about 0",
        abs(lo + hi) < 1e-9 and lo < 0 < hi,
        (lo, hi),
    )
    failed = sum(1 for x in res if not x)
    print(f"\n{len(res) - failed} of {len(res)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
