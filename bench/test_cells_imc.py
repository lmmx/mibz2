"""bench/cells_imc.py's statistic, rule and validity on synthetic readings: each verdict where
the LOG puts it, its boundaries, and each defect the validity guards against leaves no verdict.

    python3 bench/test_cells_imc.py
"""

import json
import math
import random
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import cells_imc as ci  # noqa: E402

OUT = 176_000_000  # output bytes a decode
LOOPS = 4


def readings(growth=1.0, base=10.0, case=None, seed=5, noise=0.01, threads=None, reps=None):
    """Synthetic readings: b(t) is `base` up to 3 threads, rising in log to `base * growth`
    at 10 threads (linear in t from 3 to 10), noise `noise` a reading; idle 0.1 GB/s."""
    threads = threads or ci.REGISTERED_THREADS
    reps = ci.REGISTERED_REPS if reps is None else reps
    rng = random.Random(seed)
    rows = [{"pass": "meta", "threads": threads, "reps": reps, "loops": LOOPS, "pair_seconds": 3}]
    for rep in range(reps):
        for slot, t in enumerate(ci.order(threads, rep)):
            frac = min(max((t - 3) / 7, 0.0), 1.0)
            b = base * math.exp(frac * math.log(growth)) * math.exp(rng.gauss(0, noise))
            span = 2.5 + 0.1 * t
            idle = {"span": 3.0, "reads": 0.06e9, "writes": 0.04e9}
            net = b * OUT * LOOPS
            dec = {
                "err": "",
                "out": OUT,
                "hash": "0123456789abcdef",
                "wall": span / LOOPS,
                "dose_mb": 0,
                "dose_kind": "stream",
                "dose_blocks": ci.BLOCKS,
            }
            rec = {
                "pass": "imc",
                "rep": rep,
                "threads": t,
                "slot": slot,
                "loops": LOOPS,
                "idle": idle,
                "error": None,
                "wall": span + 0.05,
                "span": span,
                "reads": 0.7 * net + idle["reads"] * span,
                "writes": 0.3 * net + idle["writes"] * span,
                "decodes": [dict(dec) for _ in range(LOOPS)],
            }
            key = (t, rep)
            if case == "missing" and key == (10, 4):
                continue
            if case == "no_idle" and key == (3, 2):
                rec["idle"], rec["idle_error"] = None, "perf failed"
            if case == "failed_reading" and key == (6, 1):
                rec.update(error="bench-imc: perf stat failed", decodes=[])
            if case == "failed_decode" and key == (2, 7):
                rec["decodes"][1]["err"] = "crc mismatch"
            if case == "short_chain" and key == (4, 0):
                rec["decodes"] = rec["decodes"][:-1]
            if case == "hash" and key == (8, 3):
                rec["decodes"][0]["hash"] = "fedcba9876543210"
            if case == "dose_mb" and key == (1, 9):
                rec["decodes"][2]["dose_mb"] = 8
            if case == "dose_blocks" and key == (10, 0):
                rec["decodes"][0]["dose_blocks"] = 195
            if case == "net_negative" and key == (1, 5):
                rec["writes"] = idle["writes"] * span * 0.9
            if case == "idle_high" and key == (10, 6):
                # (the idle rate at 10% of the decode reading's rate exactly: not below it)
                rate = (rec["reads"] + rec["writes"]) / span
                idle.update(reads=0.06 * rate, writes=0.04 * rate)
            rows.append(rec)
            if case == "duplicate" and key == (3, 0):
                rows.append(dict(rec))
    if case == "extra":
        rows.append(dict(rows[1], threads=5))
    for rep in range(reps):
        for slot, t in enumerate(ci.order(threads, rep)):
            rec = {
                "pass": "l3",
                "rep": rep,
                "threads": t,
                "slot": slot,
                "loops": LOOPS,
                "error": None,
                "span": 2.5,
                "l3_miss": 0.05 * OUT * LOOPS * (1 + t / 10),
                "decodes": [
                    {"err": "", "out": OUT, "hash": "0123456789abcdef", "wall": 0.5}
                    for _ in range(LOOPS)
                ],
            }
            if case == "l3_uncounted" and (t, rep) == (4, 4):
                rec.update(error="bench-imc: perf could not count", decodes=[])
                del rec["l3_miss"]
            if case == "l3_hash" and (t, rep) == (4, 4):
                rec["decodes"][0]["hash"] = "fedcba9876543210"
            rows.append(rec)
    return rows


def run(rows, *extra):
    with tempfile.TemporaryDirectory() as d:
        path = Path(d) / "readings.jsonl"
        path.write_text("".join(json.dumps(r) + "\n" for r in rows))
        stem = str(Path(d) / "cells-imc")
        code = ci.main(["analyse", str(path), "--out", stem, *extra])
        return code, json.loads(Path(stem + ".json").read_text())


def test_rule_boundaries():
    assert ci.verdict(1.25, 1.0001, 1.5) == "PREMISE HOLDS"
    assert ci.verdict(1.2499, 1.1, 1.4) == "INCONCLUSIVE"
    assert ci.verdict(1.30, 1.0, 1.6) == "INCONCLUSIVE"  # (lower bound not above 1)
    assert ci.verdict(1.10, 0.9, 1.15) == "PREMISE FAILS"
    assert ci.verdict(1.1001, 0.9, 1.15) == "INCONCLUSIVE"
    assert ci.verdict(1.05, 0.9, 1.1501) == "INCONCLUSIVE"
    assert ci.verdict(0.80, 0.70, 0.90) == "PREMISE FAILS"  # (traffic falling: fails too)
    assert ci.verdict(1.15, 1.05, 1.25) == "INCONCLUSIVE"
    assert ci.verdict(math.nan, 1, 2) == "INCONCLUSIVE"


def test_verdicts():
    code, r = run(readings(growth=1.5))
    assert code == 0 and r["verdict"] == "PREMISE HOLDS", r["verdict_line"]
    assert r["registered"] and (r["hi"], r["lo"]) == (10, 3)
    assert abs(r["key"][0] - 1.5) < 0.03, r["key"]
    code, r = run(readings(growth=1.0))
    assert code == 0 and r["verdict"] == "PREMISE FAILS", r["verdict_line"]
    code, r = run(readings(growth=1.17))
    assert code == 0 and r["verdict"] == "INCONCLUSIVE", r["verdict_line"]
    # (a large ratio too noisy to exclude 1)
    code, r = run(readings(growth=1.3, base=40.0, noise=0.4))
    assert r["verdict"] == "INCONCLUSIVE", r["verdict_line"]


def test_measure():
    # (b(t) recovers the bytes per output byte after the idle subtraction)
    code, r = run(readings(growth=1.0, base=12.0, noise=0.0))
    assert code == 0
    for t in ci.REGISTERED_THREADS:
        assert abs(r["curve"][str(t)]["b"][0] - 12.0) < 1e-6, (t, r["curve"][str(t)]["b"])
        assert abs(r["curve"][str(t)]["reads"] - 8.4) < 1e-6
    assert abs(r["b1_over_census"] - 12 / 25) < 1e-6
    assert abs(r["b10_over_dose_log"] - 12 / 24) < 1e-6
    assert set(r["reported"]) == {"b(10)/b(2)", "b(10)/b(4)"}
    assert r["curve"]["10"]["l3_miss_per_byte"][0] > r["curve"]["1"]["l3_miss_per_byte"][0]


def test_validity():
    for case in (
        "missing",
        "no_idle",
        "failed_reading",
        "failed_decode",
        "short_chain",
        "hash",
        "dose_mb",
        "dose_blocks",
        "net_negative",
        "idle_high",
        "duplicate",
        "extra",
        "l3_hash",
    ):
        code, r = run(readings(growth=1.5, case=case))
        assert code == 1 and r["verdict"] == "INVALID", (case, r["verdict_line"])
        assert r["problems"], case


def test_l3_is_reported_not_gating():
    code, r = run(readings(growth=1.5, case="l3_uncounted"))
    assert code == 0 and r["verdict"] == "PREMISE HOLDS", r["verdict_line"]
    assert r["l3_problems"] and r["curve"]["4"]["l3_miss_per_byte"][1] == 9


def test_other_grid_is_marked():
    rows = readings(growth=1.5, threads=[1, 2], reps=2)
    code, r = run(rows, "--threads", "1,2", "--reps", "2")
    assert code == 0 and not r["registered"] and (r["hi"], r["lo"]) == (2, 1)
    assert "not the registered grid" in r["verdict_line"]


def test_collect_with_stand_ins():
    """collect end to end, then analyse: a stand-in `ab` (a JSON record a decode, as `ab
    child` prints) and a stand-in for imc.perf (runs the command; counts made up from the
    thread count), so the roster, rotation, idle pairing and records are what analyse reads"""
    import os
    import subprocess

    with tempfile.TemporaryDirectory() as d:
        ab = Path(d) / "ab"
        ab.write_text(
            "#!/bin/sh\n"
            'echo \'{"wall":0.01,"out":1000,"hash":"ab","err":"","dose_mb":0,'
            '"dose_kind":"stream","dose_blocks":196}\'\n'
        )
        ab.chmod(0o755)
        seen = []

        def perf(events, cmd, system, interval=None):
            r = subprocess.run(cmd, capture_output=True, text=True)
            t = int(cmd[-2]) if cmd[0] == "sh" else 0
            if cmd[0] == "sh":
                seen.append((system, t, events == [ci.L3]))
            n = 1000 * int(cmd[3]) if cmd[0] == "sh" else 0
            row = {e: 1e3 + (10 + t) * n for e in events}
            row["_run_ns"] = 1e9
            return 1.0, r.stdout, {None: row}

        real, ci.imc.perf = ci.imc.perf, perf
        real_run, ci.imc.run = (
            ci.imc.run,
            lambda argv, check=False: subprocess.CompletedProcess(argv, 0, "", ""),
        )
        try:
            out = str(Path(d) / "readings.jsonl")
            argv = ["collect", "--ab", str(ab), "--corpus", os.devnull, "--threads", "1,3,10"]
            assert (
                ci.main([*argv, "--reps", "3", "--loops", "2", "--settle", "0", "--out", out]) == 0
            )
        finally:
            ci.imc.perf, ci.imc.run = real, real_run
        recs = [json.loads(x) for x in Path(out).read_text().splitlines()]
        assert recs[0]["pass"] == "meta" and recs[0]["loops"] == 2
        imcs = [r for r in recs if r["pass"] == "imc"]
        assert [r["threads"] for r in imcs] == [1, 3, 10, 3, 10, 1, 10, 1, 3]
        assert all(r["idle"] and len(r["decodes"]) == 2 for r in imcs)
        assert len([r for r in recs if r["pass"] == "l3"]) == 9
        assert {s for s in seen if not s[2]} == {(True, t, False) for t in (1, 3, 10)}
        assert {s for s in seen if s[2]} == {(False, t, True) for t in (1, 3, 10)}
        code, r = run(
            [json.loads(x) for x in Path(out).read_text().splitlines()],
            "--threads",
            "1,3,10",
            "--reps",
            "3",
        )
        assert code == 0 and (r["hi"], r["lo"]) == (10, 3), r["verdict_line"]
        assert abs(r["key"][0] - 20 / 13) < 1e-6, r["key"]


if __name__ == "__main__":
    for name, f in list(globals().items()):
        if name.startswith("test_") and callable(f):
            f()
            print(f"ok {name}")
