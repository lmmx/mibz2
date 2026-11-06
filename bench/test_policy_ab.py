"""bench/policy_ab.py's verdict on synthetic i9-shaped grids: a clean grid passes, and each
defect the rule guards against fails it.

    python3 -m pytest bench/test_policy_ab.py
"""

import json
import random
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import policy_ab  # noqa: E402

BASE = "mibz2@/tmp/base/target/release/examples/ab"


def tc(teams, k):
    names = [f"mibz2-{i}=0.14" for i in range(teams)]
    return " ".join(names + ["mibz2-coop-1=0.08"] * (teams * (k - 1)) + ["mibz2-scan=0.05"])


def grid(case=None, speedup=1.08, harm=1.0, seed=7):
    """20 rounds of base and head at every registered thread count; `speedup` at 10 threads,
    `harm` (head wall over base) at 11-20, noise of about 1% a run"""
    rng = random.Random(seed)
    rows = []
    for rnd in range(20):
        for t in policy_ab.THREADS:
            for build in ("base", "head"):
                if case == "missing round" and (build, t, rnd) == ("head", 12, 7):
                    continue
                new = build == "head" and t >= 10
                plan = (3, 3) if new else (4, 2)
                if case == "wrong plan" and (build, t, rnd) == ("head", 10, 4):
                    plan = (4, 2)
                wall = 0.165 * rng.gauss(1, 0.01)
                if build == "head" and t == 10:
                    wall /= speedup
                if build == "head" and t in policy_ab.NO_HARM:
                    wall *= harm
                if case == "control drift" and build == "head" and t == 8:
                    wall /= 1.05
                rows.append(
                    {
                        "round": rnd,
                        "threads": t,
                        "arm": "mibz2" if build == "head" else BASE,
                        "wall": wall,
                        "cpu": 1.0 * (0.9 if new else 1.0),
                        "rss_kb": 82000 * (0.82 if new else 1.0),
                        "out": 209700196,
                        "hash": "bad" if case == "wrong bytes" and (rnd, t) == (3, 16) else "ok",
                        "thread_cpu": tc(*plan),
                        "err": "boom" if case == "failed run" and (rnd, t) == (5, 9) else "",
                    }
                )
    if case == "duplicate round":
        rows.append(dict(rows[0]))
    return rows


def test_clean_grid_passes():
    v = policy_ab.judge(grid())
    assert v["valid"] and v["pass"], v["problems"]
    assert 1.06 < v["cells"][10]["speed"] < 1.10


def test_plans_read_from_thread_names():
    assert policy_ab.plan_of(tc(3, 3)) == (3, 6)
    assert policy_ab.plan_of(tc(4, 2)) == (4, 4)


def test_each_defect_fails():
    for case in ("missing round", "wrong plan", "wrong bytes", "failed run", "duplicate round"):
        v = policy_ab.judge(grid(case))
        assert not v["valid"] and not v["pass"], case


def test_a_flagged_control_fails():
    v = policy_ab.judge(grid("control drift"))
    assert v["valid"] and not v["controls"][8] and not v["pass"]


def test_too_small_a_gain_fails():
    v = policy_ab.judge(grid(speedup=1.02))
    assert not v["primary"]["speed"] and not v["pass"]


def test_harm_beyond_two_percent_fails():
    v = policy_ab.judge(grid(harm=1.04))
    assert not any(v["no_harm"].values()) and not v["pass"]


def test_noise_too_wide_is_inconclusive(monkeypatch):
    rows = grid()
    rng = random.Random(1)
    for r in rows:
        if r["threads"] == 16:
            r["wall"] *= rng.gauss(1, 0.08)
    v = policy_ab.judge(rows)
    assert not v["no_harm"][16] and not v["pass"]


def test_cli_writes_both_reports(tmp_path):
    runs = tmp_path / "runs.jsonl"
    runs.write_text("".join(json.dumps(r) + "\n" for r in grid()))
    out = tmp_path / "policy-ab"
    r = subprocess.run(
        [sys.executable, str(HERE / "policy_ab.py"), "report", str(runs), "--out", str(out)],
        capture_output=True,
        text=True,
    )
    assert r.returncode == 0, r.stderr
    assert json.loads((tmp_path / "policy-ab.json").read_text())["pass"]
    assert "**This run: PASS.**" in (tmp_path / "policy-ab.md").read_text()
