#!/usr/bin/env python3
"""Checks of zero_copy_floor.py on synthetic runs with known walls. All real is 200 ms at every count
(1300 ms at 1 worker); SECMRD is 60 ms (273 at 1), SFCMRD 16 ms (at 1: inside Antigravity's 14-19),
SFCMR 30 ms, and the all-but-X arms (S real 40, E real 56, C real 16.2, M
real 16, R real 20 ms). The handovers and the spelling are in SECMRD's traces. Every measure is recovered, the
prediction is scored both ways, and each VOID and INCOMPLETE fires.

    python3 docs/journal/2026-11-24-algorithm-level/ideal-parts/results/analysis/test_zero_copy_floor.py
"""

import importlib.util
import io
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import zero_copy_floor as zf  # noqa: E402

_spec = importlib.util.spec_from_file_location(
    "ideal_parts_planner",
    os.path.join(HERE, "..", "..", "..", "..", "..", "..", "bench", "ideal_parts.py"),
)
planner = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(planner)

BLOCKS, TB, SCPU = 8, 4, 4
COUNTS = (1, 6, 8, 10)
I9 = "Model name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
ACTS = [
    "A0", "feed_wait", "in_wait", "in_maint", "A2", "A3", "slot_wait", "issue", "job_wait", "A5",
    "pool", "A6a", "A6b", "A6c", "A7", "A8", "A9", "publish", "A10", "read_wait", "lock_state",
    "lock_jobs", "hand_S", zf.HAND_E_DELIVER, zf.E_CONSUME, "hand_C", "hand_M", "hand_R",
    "copy_out", "resid",
]  # fmt: skip
WALL = {"none": 0.200, "SECMRD": 0.060, "SFCMRD": 0.016, "SFCMR": 0.030,
        "FCMRD": 0.040, "SCMRD": 0.056, "SFMRD": 0.0162, "SFCRD": 0.016, "SFCMD": 0.020}  # fmt: skip
WALL1 = {"none": 1.300, "SECMRD": 0.273, "SFCMRD": 0.016, "SFCMR": 0.040,
         "FCMRD": 0.030, "SCMRD": 0.270, "SFMRD": 0.0162, "SFCRD": 0.016, "SFCMD": 0.020}  # fmt: skip


def plan():
    cells = [{"section": "pinned", "k": k, "arm": a, "pinned": 1} for k in COUNTS for a in zf.ARMS]
    return {"cells": cells, "trace_blocks": TB, "scanner_cpu": SCPU}


OWN = {"A2": 5e6, "A3": 6e7, "A7": 1.4e8, "A6a": 2e7, "A6b": 6e7, "A6c": 2e7, "A5": 9e7,
       "A9": 7e7, "copy_out": 4e7}  # fmt: skip


def threads(arm, k, b1_on=None):
    def th(role, acts, cpus=None):
        st = [[0] * len(ACTS) for _ in range(3)]
        for a, ns in acts.items():
            st[1][ACTS.index(a)] = int(ns)
        return {"role": role, "states": st, "cpus": cpus or {}}

    def own(part):
        role, acts = zf.WORK[part]
        return {} if zf.ideal_in(part, arm) else {a: OWN[a] for a in acts}

    work = {**own("E"), **own("C"), **own("M"), **own("R")}
    scan = own("S")
    if arm != "none":
        # zero-copy handovers: small, 1 ms per thread for C, M and R, 2 for E's delivery
        work.update(
            {"hand_C": k * 1e6, "hand_M": k * 1e6, "hand_R": k * 1e6, zf.HAND_E_DELIVER: k * 2e6}
        )
        scan["hand_S"] = 3e6
    if "E" in arm:
        work[zf.E_CONSUME] = k * 40e6  # the spelling, 40 ms per worker, run
    return [th("Scanner", scan, {str(SCPU): 1}), th("Worker", work), th("Reader", own("D"))]


def records(fail=None, seed=3):
    rnd = random.Random(seed)
    p = plan()
    out = []
    for b in range(BLOCKS):
        for ci, c in enumerate(p["cells"]):
            for on in (0, 1) if b < TB else (0,):
                w = (WALL1 if c["k"] == 1 else WALL)[c["arm"]]
                d = "D" in c["arm"]
                x = {"kind": "decode", "k": c["k"], "on": on, "on_arm": on, "cell": ci, "block": b,
                     "section": "pinned", "arm": c["arm"], "wall_s": w * math.exp(rnd.gauss(0, 0.005)),
                     "bytes": 0 if d else zf.OUT_BYTES, "replays": {"D_discarded_bytes": zf.OUT_BYTES if d else 0},
                     "majflt": 0, "peak_kb": 800_000, "identities_ok": True, "check_pass": {"identical": True},
                     "pmu": {"cycles": [4.9e8], "ref-cycles": [3.7e8], "LLC-load-misses": [2e6]},
                     "imc_mib": {"why": "perf_event_paranoid 2"}}  # fmt: skip
                if on:
                    x["activities"] = ACTS
                    x["threads"] = threads(c["arm"], c["k"])
                out.append(x)
    if fail:
        fail(out)
    return p, out


def run(p, recs, lscpu=I9):
    buf = io.StringIO()
    code, rows = zf.analyse(p, recs, lscpu, BLOCKS, out=buf)
    return code, buf.getvalue(), rows


def get(rows, k, m, key="value"):
    return next((r[key] for r in rows if r["k"] == k and r["measure"] == m), None)


def main():
    res = []

    def check(name, ok, why=""):
        res.append(ok)
        print(f"{'ok  ' if ok else 'FAIL'} {name}{'' if ok else ': ' + str(why)[:900]}")

    check(
        "the planner's floor design is the analysis's",
        tuple(planner.FLOOR_ARMS) == zf.ARMS and planner.FLOOR_COUNTS == (1, 6, 7, 8, 9, 10),
        (planner.FLOOR_ARMS, planner.FLOOR_COUNTS),
    )
    code, text, rows = run(*records())
    check(
        "a whole run: exit 0, no VOID",
        code == 0 and not any(ln.startswith("VOID") for ln in text.splitlines()),
        text[:900],
    )
    check(
        "k 8: the floor is 60/200", abs(get(rows, 8, "floor") - 0.30) < 0.005, get(rows, 8, "floor")
    )
    check(
        "k 8: Residual A 60 ms, B 16 ms",
        abs(get(rows, 8, "residual A ms") - 60) < 1
        and abs(get(rows, 8, "residual B ms") - 16) < 0.5,
    )
    check(
        "k 8: E_consume on the wall 44 ms",
        abs(get(rows, 8, "E_consume wall ms") - 44) < 1,
        get(rows, 8, "E_consume wall ms"),
    )
    check(
        "k 8: D's share 14 ms", abs(get(rows, 8, "D share ms") - 14) < 1, get(rows, 8, "D share ms")
    )
    check(
        "k 8: the ledger's spelling 40 ms per worker",
        abs(get(rows, 8, "E_consume ledger ms per thread") - 40) < 1e-6,
    )
    check(
        "k 8: the workers' handovers 5 ms per thread",
        abs(get(rows, 8, "workers' handover ms per thread") - 5) < 1e-6,
    )
    check(
        "k 8: S's handover 3 ms on its one thread",
        abs(get(rows, 8, "handover S ms per thread") - 3) < 1e-6,
    )
    room = lambda k, p, key="value": get(rows, k, f"room {p} against the floor", key)  # noqa: E731
    check(
        "k 8: S's room against the floor 16/40, ROOM",
        abs(room(8, "S") - 0.40) < 0.01 and room(8, "S", "label") == "ROOM",
    )
    check(
        "k 8: C's room 16/16.2, NO ROOM; M's NO ROOM",
        room(8, "C", "label") == "NO ROOM" and room(8, "M", "label") == "NO ROOM",
    )
    check(
        "k 8: R's room 0.80 and D's 16/30, both ROOM",
        abs(room(8, "R") - 0.80) < 0.01 and room(8, "D", "label") == "ROOM",
    )
    check(
        "the prediction: Residual B 16 ms at 1 worker is a HIT",
        get(rows, 1, "prediction residual B", "label") == "hit" and "HIT" in text,
    )
    check(
        "the removed parts read 0%",
        all(f"{p}: 0.00%" in text for p in list(zf.WORK) + ["E_consume"]),
        text[:1200],
    )
    check(
        "the IMC's absence is reported with its reason",
        "memory n/a (perf_event_paranoid 2)" in text,
        text[-900:],
    )

    def b_slow(r):
        for x in r:
            if x["k"] == 1 and x["arm"] == "SFCMRD":
                x["wall_s"] = 0.040 * math.exp(random.gauss(0, 0.01))

    code, text, rows = run(*records(fail=b_slow))
    check(
        "the prediction: Residual B 40 ms at 1 worker is a MISS",
        get(rows, 1, "prediction residual B", "label") == "miss" and "MISS" in text,
    )

    def spelling_left(r):
        for x in r:
            if x.get("on") == 1 and x["arm"] in ("SFCMRD", "SFCMR") and x["k"] == 6:
                x["threads"][1]["states"][1][ACTS.index(zf.E_CONSUME)] = int(6 * 20e6)

    code, text, _ = run(*records(fail=spelling_left))
    check(
        "VOID: E_full leaves the spelling (over 5% of SECMRD's)",
        code == 1 and "E_consume is" in text,
        text[:700],
    )

    def c_left(r):
        for x in r:
            if x.get("on") == 1 and "C" in x["arm"] and x["k"] == 10:
                x["threads"][1]["states"][1][ACTS.index("A6b")] = int(30e6)

    code, text, _ = run(*records(fail=c_left))
    check("VOID: construction's own work over 5%", code == 1 and "C's work" in text, text[:700])

    def big_hand(r):
        for x in r:
            if x.get("on") == 1 and x["arm"] == "SECMRD":
                x["threads"][1]["states"][1][ACTS.index("hand_R")] = int(9e8)

    code, text, _ = run(*records(fail=big_hand))
    check("a large handover is no VOID (its own entry)", code == 0, text[:600])

    def timeout(r):
        r[9] = {"kind": "failed", "timeout": True, "section": "pinned", "arm": r[9]["arm"], "k": r[9]["k"],
                "block": r[9]["block"], "cell": r[9]["cell"], "on_arm": r[9]["on_arm"]}  # fmt: skip

    code, text, _ = run(*records(fail=timeout))
    check("VOID: a decode killed at the timeout", code == 1 and "a decode failed (timeout)" in text)

    def bad(r):
        r[4]["check_pass"] = {"identical": False}

    code, text, _ = run(*records(fail=bad))
    check("VOID: a check pass not byte-identical", code == 1 and "not byte-identical" in text)

    def lost(r):
        next(x for x in r if x["arm"] == "SECMRD" and x["on"] == 0)["replays"][
            "D_discarded_bytes"
        ] = 0

    code, text, _ = run(*records(fail=lost))
    check("VOID: a D arm that delivered nothing", code == 1 and "0 bytes delivered" in text)

    def pin_off(r):
        for x in r:
            if x.get("on") == 1 and x["k"] in (8, 10):
                x["threads"][0]["cpus"] = {"9": 1}

    code, text, _ = run(*records(fail=pin_off))
    check("VOID: the pin broken in over 10% of traces", code == 1 and "pin broken" in text)
    code, text, _ = run(*records(fail=lambda r: r.pop(2)))
    check("VOID: a decode missing", code == 1 and "0 decodes" in text)
    code, text, _ = run(*records(), lscpu="Model name: Xeon\n")
    check("VOID: not the i9", code == 1 and "not the i9" in text)

    def ident(r):
        next(x for x in r if x.get("on") == 1)["identities_ok"] = False

    code, text, _ = run(*records(fail=ident))
    check("INCOMPLETE: an identity failed (no VOID)", code == 0 and "INCOMPLETE" in text)
    failed = sum(1 for x in res if not x)
    print(f"\n{len(res) - failed} of {len(res)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
