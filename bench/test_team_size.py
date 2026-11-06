"""bench/team_size.py's verdict on synthetic i9-shaped grids: a clean grid passes, and each
defect the Guardian's reviews named (D026, D027) fails the run's predicates.

    python3 -m pytest bench/test_team_size.py
"""

import json
import random
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ARMS = {"mibz2": None, "mibz2-team2": 2, "mibz2-team3": 3, "mibz2-team4": 4}
THREADS = (8, 10, 12, 16)
FAST = {("mibz2-team3", 12): 0.140, ("mibz2-team4", 16): 0.150}


def thread_cpu(teams, k):
    names = [f"mibz2-{i}=0.14" for i in range(teams)]
    return " ".join(names + ["mibz2-coop-1=0.08"] * (teams * (k - 1)) + ["mibz2-scan=0.05"])


def grid(case=None):
    rng = random.Random(2)
    rows = []
    for rnd in range(20):
        for arm, k in ARMS.items():
            for t in THREADS:
                if case == "missing cell" and (arm, t) == ("mibz2-team2", 16):
                    continue
                teams, kk = (4, 2) if k is None else (t // k, k)
                helpers = teams * (kk - 1)
                tc = thread_cpu(teams, kk)
                if case == "4 leaders + 11 helpers" and (arm, t, rnd) == ("mibz2-team4", 16, 3):
                    tc = tc.replace("mibz2-coop-1=0.08 ", "", 1)
                    helpers -= 1
                r = {
                    "round": rnd,
                    "corpus": "wikidata",
                    "threads": t,
                    "arm": arm,
                    "wall": FAST.get((arm, t), 0.161) * rng.uniform(0.99, 1.01),
                    "cpu": 1.05,
                    "rss_kb": 80000,
                    "out": 209700196,
                    "hash": "a",
                    "thread_cpu": tc,
                    "err": "",
                }
                if case == "two output lengths" and (arm, t, rnd) == ("mibz2-team3", 10, 4):
                    r["out"] = 1
                if case == "failed run" and (arm, t, rnd) == ("mibz2-team3", 12, 9):
                    r["err"] = "crc mismatch"
                rows.append(r)
                if case == "duplicate control round" and (arm, t, rnd) == ("mibz2-team2", 8, 7):
                    rows.append(dict(r, wall=0.2))
                if case == "duplicate primary round" and (arm, t, rnd) == ("mibz2-team3", 12, 5):
                    rows.append(dict(r, wall=0.1))
    return rows


def verdict(tmp_path, case=None):
    runs = tmp_path / "runs.jsonl"
    runs.write_text("".join(json.dumps(r) + "\n" for r in grid(case)))
    stem = tmp_path / "rep"
    subprocess.run(
        [sys.executable, str(HERE / "team_size.py"), "report", str(runs), "--out", str(stem)],
        check=True,
        capture_output=True,
    )
    return json.loads(Path(f"{stem}.json").read_text())["verdict"]


def passes(v):
    return {c: p["passes"] for c, p in v["primaries"].items()}


def test_clean_grid_passes(tmp_path):
    v = verdict(tmp_path)
    assert v["same_bytes"] and v["grid_complete"] and not v["control_flagged"]
    assert passes(v) == {"mibz2-team3@12": True, "mibz2-team4@16": True}


def test_each_defect_fails(tmp_path):
    for case in [
        "duplicate control round",
        "duplicate primary round",
        "missing cell",
        "two output lengths",
        "failed run",
    ]:
        d = tmp_path / case.replace(" ", "-")
        d.mkdir()
        v = verdict(d, case)
        assert not any(passes(v).values()), case


def test_uneven_helpers_fail_the_plan(tmp_path):
    v = verdict(tmp_path, "4 leaders + 11 helpers")
    p = v["primaries"]["mibz2-team4@16"]
    assert not p["plan_ok"] and not p["passes"]
    assert v["primaries"]["mibz2-team3@12"]["passes"]
