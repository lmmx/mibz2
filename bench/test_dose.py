"""bench/dose.py's statistic, rule and validity on synthetic grids: each verdict where the
LOG puts it, and each defect the validity guards against leaves no verdict.

    python3 bench/test_dose.py
"""

import json
import math
import random
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import dose  # noqa: E402

ARMS = [f"mibz2-workers@/w/dose-{p}/target/release/examples/ab" for p in dose.DOSES]


def grid(growth=1.0, base=1.0, case=None, seed=3, noise=0.003):
    """The registered grid: P=16's wall over P=0's is `base` at 1 thread, rising in log to
    `base * growth` at 10 threads; P=8's half that in log; noise of `noise` a run"""
    rng = random.Random(seed)
    rows = []
    for rnd in range(dose.REGISTERED_ROUNDS):
        for t in dose.REGISTERED_THREADS:
            frac = (t - 1) / 9
            for arm, p in zip(ARMS, dose.DOSES):
                if case == "missing" and (p, t, rnd) == (16, 10, 4):
                    continue
                c = math.exp(p / 16 * (math.log(base) + frac * math.log(growth)))
                rec = {
                    "round": rnd,
                    "corpus": "wikidata",
                    "threads": t,
                    "arm": arm,
                    "termination": "success",
                    "err": "",
                    "hash": "0123456789abcdef",
                    "out": 1000,
                    "sha256": None,
                    "wall": 2.0 / t * c * rng.gauss(1, noise),
                    "cpu": 2.0 * c,
                    "dose_mb": p,
                    "dose_blocks": 40,
                }
                if case == "failed" and (p, t, rnd) == (8, 4, 2):
                    rec["termination"] = "exit 101"
                if case == "hash" and (p, t, rnd) == (0, 6, 1):
                    rec["hash"] = "fedcba9876543210"
                if case == "dose" and p == 8:
                    rec["dose_mb"] = 16
                rows.append(rec)
                if case == "duplicate" and (p, t, rnd) == (0, 1, 0):
                    rows.append(dict(rec))
    if case == "extra":
        rows.append(dict(rows[0], corpus="text"))
    return rows


def run(rows, *extra):
    with tempfile.TemporaryDirectory() as d:
        path = Path(d) / "runs.jsonl"
        path.write_text("".join(json.dumps(r) + "\n" for r in rows))
        stem = str(Path(d) / "dose")
        code = dose.main([str(path), "--arms", ",".join(ARMS), "--out", stem, *extra])
        return code, json.loads(Path(stem + ".json").read_text())


def test_rule():
    assert dose.verdict(1.10, 1.001, 1.2) == "LEVER"
    assert dose.verdict(1.12, 0.99, 1.3) == "INCONCLUSIVE"
    assert dose.verdict(1.03, 0.97, 1.06) == "NOT THE LEVER"
    assert dose.verdict(1.00, 0.96, 1.04) == "INCONCLUSIVE"
    assert dose.verdict(1.00, 0.98, 1.07) == "INCONCLUSIVE"
    assert dose.verdict(1.05, 1.01, 1.09) == "INCONCLUSIVE"


def test_verdicts():
    code, r = run(grid(growth=1.20, base=1.02))
    assert code == 0 and r["verdict"] == "LEVER", r["verdict_line"]
    assert r["registered"] and r["monotone"]
    # (a constant cost of the dose, not one that grows with the threads: not the lever)
    code, r = run(grid(growth=1.0, base=1.08))
    assert code == 0 and r["verdict"] == "NOT THE LEVER", r["verdict_line"]
    assert abs(r["doses"]["16"]["c"]["1"][0] - 1.08) < 0.01
    code, r = run(grid(growth=1.06))
    assert r["verdict"] == "INCONCLUSIVE", r["verdict_line"]
    code, r = run(grid(growth=1.0, noise=0.1))
    assert r["verdict"] == "INCONCLUSIVE", r["verdict_line"]


def test_validity():
    for case in ("missing", "failed", "hash", "dose", "duplicate", "extra"):
        code, r = run(grid(growth=1.2, case=case))
        assert code == 1 and r["verdict"] == "INVALID", (case, r["verdict_line"])
        assert r["problems"], case


def test_other_grid_is_marked():
    rows = [r for r in grid(growth=1.2) if r["threads"] in (1, 4) and r["round"] < 2]
    code, r = run(rows, "--threads", "1,4", "--rounds", "2")
    assert code == 0 and not r["registered"]
    assert "not the registered grid" in r["verdict_line"]


if __name__ == "__main__":
    for name, f in list(globals().items()):
        if name.startswith("test_") and callable(f):
            f()
            print(f"ok {name}")
