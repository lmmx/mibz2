"""bench/cells_diet.py's statistic, amended rule and validity on synthetic grids: each verdict
where LOG 6881022819 (amended by LOG 6881074255) puts it, its boundaries, and each defect the
validity guards against leaves no verdict.

    python3 bench/test_cells_diet.py
"""

import json
import math
import random
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import cells_diet as cd  # noqa: E402

ARMS = [
    "mibz2-workers@/w/cells-diet/target-s20/release/examples/ab",
    "mibz2-workers@/w/cells-diet/target-s24/release/examples/ab",
]


def grid(q=1.0, case=None, seed=7, noise=0.003, threads=None, rounds=None):
    """A24's CPU amplification rising to 2.0 at the most threads; A20's to 2.0 * q, the log
    linear in t; noise `noise` a run"""
    threads = threads or cd.REGISTERED_THREADS
    rounds = cd.REGISTERED_ROUNDS if rounds is None else rounds
    rng = random.Random(seed)
    lo, hi = min(threads), max(threads)
    rows = []
    for rnd in range(rounds):
        for t in threads:
            frac = (t - lo) / (hi - lo)
            for arm, b in zip(ARMS, cd.BUILDS):
                key = (b, t, rnd)
                if case == "missing" and key == ("A20", 10, 4):
                    continue
                amp = math.exp(frac * (math.log(2.0) + (math.log(q) if b == "A20" else 0)))
                cpu = 1.0 * amp * math.exp(rng.gauss(0, noise))
                rec = {
                    "round": rnd,
                    "corpus": "wikidata",
                    "threads": t,
                    "arm": arm,
                    "termination": "success",
                    "err": "",
                    "hash": "0123456789abcdef",
                    "out": 209700196,
                    "sha256": None,
                    "wall": cpu / t,
                    "cpu": cpu,
                    "dose_mb": 0,
                    "dose_kind": "stream",
                    "dose_blocks": cd.BLOCKS,
                }
                if case == "failed" and key == ("A24", 4, 2):
                    rec["termination"] = "exit 101"
                if case == "err" and key == ("A20", 6, 3):
                    rec["err"] = "crc"
                if case == "hash" and key == ("A24", 6, 1):
                    rec["hash"] = "fedcba9876543210"
                if case == "engaged20" and key == ("A20", 10, 9):
                    rec["dose_blocks"] = 195
                if case == "engaged24" and key == ("A24", 1, 0):
                    rec["dose_blocks"] = 0
                if case == "dosed" and key == ("A24", 4, 5):
                    rec["dose_mb"] = 16
                if case == "no_cpu" and key == ("A20", 1, 5):
                    del rec["cpu"]
                rows.append(rec)
                if case == "duplicate" and key == ("A20", 1, 0):
                    rows.append(dict(rec))
    if case == "extra":
        rows.append(dict(rows[0], corpus="text"))
    return rows


def run(rows, *extra):
    with tempfile.TemporaryDirectory() as d:
        path = Path(d) / "runs.jsonl"
        path.write_text("".join(json.dumps(r) + "\n" for r in rows))
        stem = str(Path(d) / "diet")
        code = cd.main([str(path), "--arms", ",".join(ARMS), "--out", stem, *extra])
        res = json.loads(Path(stem + ".json").read_text())
        res["_md"] = Path(stem + ".md").read_text()
        return code, res


def test_rule_boundaries():
    assert cd.verdict(0.95, 0.90, 0.9999) == ("MOVES THE PLATEAU", "")
    assert cd.verdict(0.90, 0.85, 1.0) == ("NO SCALING EFFECT", "")  # (upper bound not below 1)
    assert cd.verdict(0.9501, 0.93, 0.97) == (
        "NO SCALING EFFECT",
        "smaller effect, below threshold",
    )
    # (the Optimiser's point forecast, with its interval below 1: the amendment's case)
    assert cd.verdict(0.96, 0.94, 0.98) == ("NO SCALING EFFECT", "smaller effect, below threshold")
    assert cd.verdict(0.97, 0.93, 1.01) == ("NO SCALING EFFECT", "")
    assert cd.verdict(1.02, 1.01, 1.03) == ("NO SCALING EFFECT", "")
    assert cd.verdict(math.nan, 0.9, 0.95)[0] == "NO SCALING EFFECT"


def test_verdicts():
    code, r = run(grid(q=0.90))
    assert code == 0 and r["verdict"] == "MOVES THE PLATEAU", r["verdict_line"]
    assert r["registered"] and abs(r["key"][0] - 0.90) < 0.005
    code, r = run(grid(q=0.96))
    assert code == 0 and r["verdict"] == "NO SCALING EFFECT", r["verdict_line"]
    assert r["key"][2] < 1 and r["note"] == "smaller effect, below threshold", r["verdict_line"]
    assert "smaller effect, below threshold" in r["verdict_line"]
    code, r = run(grid(q=1.0))
    assert r["verdict"] == "NO SCALING EFFECT" and r["note"] == "", r["verdict_line"]
    # (a large effect too noisy to put the upper bound below 1)
    code, r = run(grid(q=0.90, noise=0.15))
    assert r["verdict"] == "NO SCALING EFFECT" and r["key"][2] >= 1, r["verdict_line"]


def test_reported():
    code, r = run(grid(q=0.90, noise=0.0))
    assert code == 0
    # (wall = CPU / t here: the 20/24 wall ratio at the most threads is Q, at one thread 1)
    assert abs(r["t"]["10"]["wall_ratio"][0] - 0.90) < 1e-9
    assert abs(r["t"]["1"]["wall_ratio"][0] - 1.0) < 1e-9
    assert abs(r["t"]["10"]["amp_A24"][0] - 2.0) < 1e-9
    assert "4-thread wall ratio 20 B / 24 B: 0.9" in r["_md"], r["_md"]


def test_validity():
    for case in (
        "missing",
        "failed",
        "err",
        "hash",
        "engaged20",
        "engaged24",
        "dosed",
        "no_cpu",
        "duplicate",
        "extra",
    ):
        code, r = run(grid(q=0.9, case=case))
        assert code == 1 and r["verdict"] == "INVALID", (case, r["verdict_line"])
        assert len(r["problems"]) == 1, (case, r["problems"])


def test_other_grid_is_marked():
    rows = grid(q=0.9, threads=[1, 2], rounds=2)
    code, r = run(rows, "--threads", "1,2", "--rounds", "2")
    assert code == 0 and not r["registered"] and (r["lo"], r["hi"]) == (1, 2)
    assert "not the registered grid" in r["verdict_line"]


if __name__ == "__main__":
    for name, f in list(globals().items()):
        if name.startswith("test_") and callable(f):
            f()
            print(f"ok {name}")
