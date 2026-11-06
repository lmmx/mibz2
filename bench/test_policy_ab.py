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


def tn(teams, k):
    """ab's `thread_names`: every thread seen, sorted, the reader and scanner included"""
    names = [f"mibz2-{i}" for i in range(teams)] + ["mibz2-coop-1"] * (teams * (k - 1))
    return " ".join(sorted(names + ["main", "mibz2-read", "mibz2-scan"]))


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
                        "corpus": "wikidata",
                        "threads": t,
                        "arm": "mibz2" if build == "head" else BASE,
                        "wall": wall,
                        "cpu": 1.0 * (0.9 if new else 1.0),
                        "rss_kb": 82000 * (0.82 if new else 1.0),
                        "out": 209700196,
                        "hash": "bad" if case == "wrong bytes" and (rnd, t) == (3, 16) else "ok",
                        "thread_cpu": tc(*plan),
                        "thread_names": tn(
                            *(
                                (3, 4)
                                if case == "hidden helpers" and (build, t, rnd) == ("head", 11, 2)
                                else plan
                            )
                        ),
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
    assert policy_ab.plan_of(tn(3, 3)) == (3, 6)
    # ab's own census of a 2 × 3 run on this branch (examples/ab.rs, `thread_names`)
    seen = "main mibz2-0 mibz2-1 mibz2-coop-1 mibz2-coop-1 mibz2-coop-2 mibz2-coop-2 mibz2-read mibz2-scan"
    assert policy_ab.plan_of(seen) == (2, 4)


def test_an_idle_helper_still_fails_the_busy_check():
    """the busy list must show the plan as before: the census adds a check, it doesn't
    replace one"""
    rows = grid()
    r = next(r for r in rows if r["arm"] == "mibz2" and r["threads"] == 10)
    r["thread_cpu"] = r["thread_cpu"].replace("mibz2-coop-1=0.08 ", "", 1)
    v = policy_ab.judge(rows)
    assert not v["valid"] and not v["pass"]


def test_each_defect_fails():
    for case in (
        "missing round",
        "wrong plan",
        "wrong bytes",
        "failed run",
        "duplicate round",
        "hidden helpers",
    ):
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


def test_rows_outside_the_registration_are_rejected():
    cases = {
        "no hash": lambda r: r.pop("hash"),
        "empty hash": lambda r: r.update(hash=""),
        "no length": lambda r: r.pop("out"),
        "zero length": lambda r: r.update(out=0),
        "nan wall": lambda r: r.update(wall=float("nan")),
        "zero cpu": lambda r: r.update(cpu=0.0),
        "inf rss": lambda r: r.update(rss_kb=float("inf")),
        "string wall": lambda r: r.update(wall="0.16"),
        "other corpus": lambda r: r.update(corpus="enwiki"),
        "second base": lambda r: r.update(arm="mibz2@/elsewhere/ab"),
        "undeclared arm": lambda r: r.update(arm="libbzip2"),
        "unregistered threads": lambda r: r.update(threads=14),
        "no census": lambda r: r.pop("thread_names"),
    }
    for name, spoil in cases.items():
        rows = grid()
        spoil(rows[37])
        v = policy_ab.judge(rows)
        assert not v["valid"] and not v["pass"], name
        assert v["problems"], name


def test_a_failed_run_without_figures_is_rejected_not_a_crash():
    rows = grid()
    rows[11] = {k: rows[11][k] for k in ("round", "threads", "arm", "corpus")} | {"err": "oom"}
    v = policy_ab.judge(rows)
    assert not v["valid"] and "1 failed run(s)" in v["problems"]
    assert "This run: FAIL" in policy_ab.markdown(v)
