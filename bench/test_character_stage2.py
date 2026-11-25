#!/usr/bin/env python3
"""Checks of bench/character_stage2.py's report (stage 2a) on synthetic records with known effects:
a coordinate that controls (CONTROLS), one that does not (NEGLIGIBLE), a missed dose labelled
DOSE-MISSED, an additive surface's roundness near 1, the gates' contrast CHANGED, and each VOID.
"""

import argparse
import io
import json
import math
import os
import random
import re
import sys
import tempfile
from contextlib import redirect_stdout

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import character_stage2 as s2  # noqa: E402

I9 = "Model name:                           Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
OUT = 4 * 209_700_196
BLOCKS = 6
NA = len(s2.ACT)


def decode(k, wall, svc=None, blocks=1724):
    """a decode record: its wall, and per role the exclusive time and visits of the activities in
    `svc` ({(role, act): ns a visit})"""
    threads = []
    for role in ("Scanner", "Worker", "Reader"):
        states = [[0] * NA, [0] * NA, [0] * NA]
        visits = [0] * (NA - 1)
        for (r, act), ns in (svc or {}).items():
            if r == role:
                i = s2.ACT.index(act)
                states[1][i] = int(ns * blocks)
                visits[i] = blocks
        threads.append(
            {"role": role, "states": states, "visits": visits, "life_ns": int(wall * 1e9)}
        )
    return {"kind": "decode", "k": k, "wall_s": wall, "bytes": OUT, "majflt": 0, "blocks": blocks,
            "identities_ok": True, "threads": threads, "admission": {"waits": [3, 1, 0]}}  # fmt: skip


def scenario(miss=False, fail=None):
    """k* 8: A3 controls (its +20% slows the wall by 20%), A9 does not; the surface additive"""
    rnd = random.Random(1)
    base_svc = {("Scanner", "A3"): 100_000.0, ("Worker", "A9"): 50_000.0}
    cells = [
        {"section": "screen", "k": 8, "arm": "base", "treat": []},
        {"section": "screen", "k": 8, "arm": "A3", "treat": ["pace A3 1"], "delta": 0.2},
        {"section": "screen", "k": 8, "arm": "A9", "treat": ["pace A9 1"], "delta": 0.2},
    ]
    for sd in (0, 10, 20):
        for wd in (0, 10, 20):
            cells.append({"section": "surface", "k": 8, "arm": f"s{sd:02d}w{wd:02d}", "treat": []})
    cells += [
        {"section": "gates", "k": 8, "arm": "W4k_cap12", "treat": []},
        {"section": "gates", "k": 8, "arm": "W8k_cap12", "treat": []},
    ]
    plan = {"kstar": 8, "step": 16, "anchors": [8], "cycles_per_iter": 0.5,
            "coords": {"8": {"kept": ["A3", "A9"]}}, "cells": cells}  # fmt: skip
    recs = []
    for b in range(BLOCKS):
        noise = lambda: math.exp(rnd.gauss(0, 0.004))  # noqa: E731
        w0 = 0.30 * noise()
        for ci, c in enumerate(cells):
            svc = dict(base_svc)
            wall = w0 * noise()
            if c["section"] == "screen" and c["arm"] == "A3":
                svc[("Scanner", "A3")] *= 1.05 if miss else 1.2
                wall = w0 * 1.2 * noise()
            if c["section"] == "screen" and c["arm"] == "A9":
                svc[("Worker", "A9")] *= 1.2
            if c["section"] == "surface":
                sd, wd = int(c["arm"][1:3]), int(c["arm"][4:6])
                wall = w0 * (1 + 0.5 * sd / 100) * (1 + 0.3 * wd / 100) * noise()
            if c["section"] == "gates" and c["arm"] == "W8k_cap12":
                wall = w0 * 0.9 * noise()
            # (a timed decode, the ledgers off, and its trace, the ledgers on)
            for on in (0, 1):
                x = decode(8, wall * (1.02 if on else 1.0), svc)
                x.update(
                    {
                        "on": on,
                        "on_arm": on,
                        "cell": ci,
                        "block": b,
                        **{kk: v for kk, v in c.items() if kk != "treat"},
                    }
                )
                recs.append(x)
    if fail:
        fail(recs)
    return plan, recs


def scenario2b(homogeneous=True, trace_blocks=2, drop=None):
    """stage 2b at 8: a wall homogeneous of degree 1 in the scanner's and the others' demands, a
    smooth max T = (s^4 + o^4)^(1/4) with s = 1 and o = 0.8, so the scanner's elasticity to a
    slow-down and to a speed-up differ by a known amount. Not homogeneous: every arm's time
    beyond the base grows by half its dose only"""
    rnd = random.Random(2)
    cells = [{"section": "screen", "k": 8, "arm": "base", "treat": [], "pinned": 1}]
    for lam in (10, 20):
        for arm in ("scan", "others", "all"):
            cells.append(
                {"section": "speed", "k": 8, "arm": f"{arm}{lam}", "treat": [], "delta": lam / 100}
            )
    plan = {"kstar": 8, "step": 16, "anchors": [8], "cycles_per_iter": 0.5, "trace_blocks": trace_blocks,
            "coords": {"8": {"kept": []}}, "cells": cells}  # fmt: skip
    T = lambda s, o: (s**4 + o**4) ** 0.25  # noqa: E731
    recs = []
    for b in range(BLOCKS):
        w0 = 0.2 * math.exp(rnd.gauss(0, 0.003))
        for ci, c in enumerate(cells):
            L = 1 + c.get("delta", 0)
            s_, o_ = {"scan": (L, 0.8), "others": (1, 0.8 * L), "all": (L, 0.8 * L)}.get(
                c["arm"][:-2] if c["arm"] != "base" else "", (1, 0.8)
            )
            r = T(s_, o_) / T(1, 0.8)
            if not homogeneous and c["arm"].startswith("all"):
                r = 1 + (L - 1) / 2
            for on in (0, 1) if b < trace_blocks else (0,):
                x = decode(8, w0 * r * math.exp(rnd.gauss(0, 0.002)))
                x.update({"on": on, "on_arm": on, "cell": ci, "block": b,
                          **{kk: v for kk, v in c.items() if kk != "treat"}})  # fmt: skip
                recs.append(x)
    if drop:
        drop(recs)
    return plan, recs


def run_report(plan, recs, lscpu=I9, phases=None, oracle=None):
    d = tempfile.mkdtemp()
    pp, rp, lp = (os.path.join(d, n) for n in ("plan.json", "runs.jsonl", "lscpu.txt"))
    ph = None
    if phases is not None:
        ph = os.path.join(d, "phases.tsv")
        open(ph, "w").write("phase\tstatus\tstarted\tended\tlast line\n" + phases)
    json.dump(plan, open(pp, "w"))
    with open(rp, "w") as f:
        for x in recs:
            f.write(json.dumps(x) + "\n")
    open(lp, "w").write(lscpu)
    a = argparse.Namespace(
        plan=pp, runs=rp, lscpu=lp, out=d, blocks=BLOCKS, seed=1, out_bytes=OUT, oracle=oracle,
        phases=ph,
    )  # fmt: skip
    buf = io.StringIO()
    with redirect_stdout(buf):
        code = s2.report(a)
    return code, buf.getvalue()


def check_plan2b(check):
    """plan2b on a 10-core, 20-CPU topology and a stage 1 stand-in: the scanner pinned to the
    fastest core's first CPU, every other thread masked off that core; 8 and 10 alone; traces in
    the first 2 blocks; the speed arms' treatments nested (all = scan + others)"""
    d = tempfile.mkdtemp()
    lse = os.path.join(d, "lscpu-e.txt")
    with open(lse, "w") as f:
        f.write("CPU NODE SOCKET CORE L1d:L1i:L2:L3 ONLINE MAXMHZ MINMHZ MHZ\n")
        for c in range(20):
            f.write(f"{c} 0 0 {c % 10} {c % 10}:{c % 10}:{c % 10}:0 yes 5300 800 4900\n")
    cores = os.path.join(d, "cores.json")
    json.dump(
        {str(c): {"repeats": [[0, 0, 2000 - (500 if c == 3 else 0)]]} for c in range(10)},
        open(cores, "w"),
    )
    a = argparse.Namespace(stage1="S1", calibration="CAL", lscpu="L", lscpu_e=lse, cores=cores,
                           out=os.path.join(d, "plan.json"))  # fmt: skip
    svc = {c: 1000.0 for c in s2.ACT}
    fake = {"step": 16, "cycles_per_iter": 0.5, "calibration": {},
            "coords": {k: {"kept": ["A3", "A7"], "svc_ns": svc, "ghz": 4.9} for k in (6, 8, 10, 16)}}  # fmt: skip
    real_plan, real_remove = s2.plan, os.remove
    s2.plan = lambda a2: (open(a2.out, "w").close(), fake)[1]
    try:
        with redirect_stdout(io.StringIO()):
            p = s2.plan2b(a)
    finally:
        s2.plan = real_plan
    cells = p["cells"]
    pinned = [c for c in cells if c.get("pinned")]
    tr = {(c["k"], c["arm"]): c["treat"] for c in cells if c["section"] == "speed"}
    nest = all(
        sorted(tr[(k, f"all{n}")]) == sorted(set(tr[(k, f"scan{n}")]) | set(tr[(k, f"others{n}")]))
        for k in (8, 10) for n in (10, 20)
    )  # fmt: skip
    ok = (
        p["scanner_cpu"] == 3
        and p["anchors"] == [8, 10]
        and p["trace_blocks"] == 2
        and all(
            f"scanner_cpu 3" in c["treat"] and 3 not in c["mask"] and 13 not in c["mask"]
            for c in pinned
        )
        and {c["section"] for c in cells if not c.get("pinned")} == {"link", "memory", "placement"}
        and {c["k"] for c in cells if c["section"] in ("screen", "speed")} == {8, 10}
        and nest
    )
    check("2b's plan: the fastest core pinned and isolated, 8 and 10, the speed arms nested", ok,
          (p["scanner_cpu"], p["anchors"], nest, [(c["section"], c["k"], c["arm"]) for c in cells][:8]))  # fmt: skip


def check_driver(check):
    """the driver fails soft: a decode that fails, and cells whose permuted corpus can't be made,
    are recorded as failed and the run carries on to the next decode"""
    d = tempfile.mkdtemp()
    fake = os.path.join(d, "fake.sh")
    # (k 3 fails; every other decode prints a record)
    open(fake, "w").write(
        '#!/bin/sh\n[ "$3" = 03 ] && { echo boom >&2; exit 2; }\n'
        'echo \'{"kind":"decode","k":\'"${3#0}"\'}\'\n'
    )
    os.chmod(fake, 0o755)
    corpus = os.path.join(d, "c.bz2")
    open(corpus, "wb").write(s2.STREAM + b"x" * 10)  # (one stream: no 12 windows to permute)
    cells = [
        {"section": "screen", "k": 2, "arm": "base", "treat": []},
        {"section": "screen", "k": 3, "arm": "base", "treat": []},
        {"section": "validation", "k": 2, "arm": "permuted", "corpus": "permuted", "treat": []},
    ]
    plan = os.path.join(d, "plan.json")
    json.dump({"cells": cells, "permutation": list(range(12))}, open(plan, "w"))
    a = argparse.Namespace(plan=plan, bin=fake, corpus=corpus, out=d, blocks=2, settle=0,
                           prewarm=0, seed=1)  # fmt: skip
    buf = io.StringIO()
    try:
        with redirect_stdout(buf):
            s2.run(a)
        ok = True
    except BaseException as e:  # noqa: BLE001
        ok, buf = False, io.StringIO(repr(e))
    recs = [json.loads(x) for x in open(os.path.join(d, "runs.jsonl"))]
    kinds = sorted((x["k"], x.get("arm"), x["kind"]) for x in recs)
    want = sorted([(2, "base", "decode"), (3, "base", "failed"), (2, "permuted", "failed")] * 4)
    check("fail soft: the driver records failed decodes and carries on", ok and kinds == want,
          (buf.getvalue()[-300:], kinds))  # fmt: skip


def main():
    res = []

    def check(name, ok, why=""):
        res.append(ok)
        print(f"{'ok  ' if ok else 'FAIL'} {name}{'' if ok else ': ' + str(why)[:600]}")

    code, text = run_report(*scenario())
    line = lambda c: next((l for l in text.splitlines() if l.startswith(f"8   {c} ")), "")  # noqa: E731,E741
    check("whole records: a report", code == 0, text[:400])
    check(
        "A3 (+20% gives +20% wall): CONTROLS",
        "CONTROLS" in line("A3") and "DOSE" not in line("A3"),
        line("A3"),
    )
    check("A9 (no effect): NEGLIGIBLE", "NEGLIGIBLE" in line("A9"), line("A9"))
    m = [l for l in text.splitlines() if "roundness" in l]  # noqa: E741
    r = float(m[0].split(":")[1].split()[0]) if m else float("nan")
    check("an additive surface's roundness near 1", 0.9 < r < 1.1, m)
    g = [l for l in text.splitlines() if "W8k_cap12" in l]  # noqa: E741
    check("the gates: W8k 10% faster, CHANGED", g and "CHANGED" in g[0], g)
    c = [ln for ln in text.splitlines() if ln.startswith("k 8   on/off")]
    check(
        "the cost check: traces 2% slower are PERTURBED, the walls the ledgers off",
        c and "PERTURBED" in c[0],
        c,
    )
    code, text = run_report(*scenario(miss=True))
    check("a missed dose: DOSE-MISSED, not VOID", code == 0 and "DOSE-MISSED" in text, text[-400:])
    code, text = run_report(*scenario(), lscpu="Model name: Intel(R) Xeon(R)\n")
    check("VOID: not the i9", code == 1 and "VOID" in text)
    code, text = run_report(*scenario(fail=lambda r: r.pop(5)))
    check("VOID: a cell's decode missing", code == 1 and "decodes" in text, text[:300])
    code, text = run_report(*scenario(fail=lambda r: r[3].update(bytes=1)))
    check("VOID: wrong bytes", code == 1 and "bytes" in text)
    code, text = run_report(*scenario(fail=lambda r: r[4].update(majflt=2)))
    check("VOID: a major fault", code == 1 and "major" in text)
    code, text = run_report(
        *scenario(
            fail=lambda r: r.append({"kind": "failed", "section": "screen", "arm": "A3", "k": 8})
        )
    )
    check("VOID: a failed decode", code == 1 and "failed" in text)
    # fail soft: a VOID run still reports what it can, and a missing decode fails its row only
    code, text = run_report(*scenario(fail=lambda r: r.pop(3)))  # (block 0, the A3 arm's decode)
    check(
        "fail soft: VOID, yet the report goes on",
        code == 1
        and "diagnosis only" in text
        and "roundness" in text
        and "W8k_cap12" in text
        and text.strip().splitlines()[-1].startswith("VOID (1 problems: cell 1 block 0"),
        text[:600],
    )
    check(
        "fail soft: the row missing a decode is FAILED, the others computed",
        "FAILED" in line("A3") and "NEGLIGIBLE" in line("A9"),
        [line("A3"), line("A9")],
    )
    ph = "build\t0\tt0\tt1\t\nfour-arm\t101\tt0\tt1\terror: could not compile\noracle\tSKIPPED\tt1\tt1\tneeds four-arm\n"
    code, text = run_report(*scenario(), phases=ph, oracle="/nonexistent/oracle.tsv")
    check(
        "fail soft: failed phases listed, the oracle section FAILED, the rest reported",
        code == 0
        and "four-arm     FAILED (exit 101)" in text
        and "oracle       SKIPPED" in text
        and "FAILED (the phase four-arm, oracle failed)" in text
        and "CONTROLS" in line("A3"),
        text[:900],
    )
    check_driver(check)
    # stage 2b: the speed-up read through homogeneity (the smooth max's elasticities known)
    T = lambda s, o: (s**4 + o**4) ** 0.25  # noqa: E731
    want_down = math.log(T(1.2, 0.8) / T(1, 0.8)) / math.log(1.2)
    want_up = -math.log(T(1 / 1.2, 0.8) / T(1, 0.8)) / math.log(1.2)
    code, text = run_report(*scenario2b())
    sp = [ln for ln in text.splitlines() if ln.startswith("8    20%")]
    vals = [float(v) for v in re.findall(r"([+-]\d\.\d+) \[", sp[0])] if sp else []
    check(
        f"2b: H 1, e_down {want_down:.3f}, e_up {want_up:.3f}, ASYMMETRIC, traces in 2 blocks only",
        code == 0
        and len(vals) == 4
        and abs(vals[0] - 1) < 0.01
        and abs(vals[1] - want_down) < 0.02
        and abs(vals[2] - want_up) < 0.02
        and "HOMOGENEOUS" in sp[0]
        and "NOT-HOMOGENEOUS" not in sp[0]
        and "ASYMMETRIC" in sp[0],
        (code, sp, text[:500]),
    )
    code, text = run_report(*scenario2b(homogeneous=False))
    sp = [ln for ln in text.splitlines() if ln.startswith("8    20%")]
    check(
        "2b: an all-paced arm short of its dose is NOT-HOMOGENEOUS",
        sp and "NOT-HOMOGENEOUS" in sp[0],
        sp,
    )
    code, text = run_report(*scenario2b(drop=lambda r: r.pop(0)))  # (block 0's base timed decode)
    check("2b: a missing timed decode VOIDs", code == 1 and "VOID" in text, text[:300])
    plan, recs = scenario2b()
    plan["scanner_cpu"] = 3
    trace0 = next(x for x in recs if x["on"] == 1 and x["cell"] == 0)
    next(t for t in trace0["threads"] if t["role"] == "Scanner")["cpus"] = {"3": 5, "7": 1}
    code, text = run_report(plan, recs)
    pin = [ln for ln in text.splitlines() if ln.startswith("the pin:")]
    check(
        "2b: a pinned trace's scanner off its CPU is PIN-BROKEN (a label, not a VOID)",
        code == 0 and pin and "PIN-BROKEN" in pin[0] and "screen base k 8 block 0" in pin[0],
        pin,
    )
    check_plan2b(check)
    failed = sum(1 for x in res if not x)
    print(f"\n{len(res) - failed} of {len(res)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
