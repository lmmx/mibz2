"""bench/service_guarded.py's verdict on synthetic i9-shaped grids: a clean grid is valid and
read as registered, and each defect the rule guards against makes it invalid (after
bench/test_pipelined_default.py).

    python3 -m pytest bench/test_service_guarded.py
"""

import random
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import service_guarded as sg  # noqa: E402

BASE = "mibz2@/w/bin/ab-base"
TWIN = "mibz2@/w/bin/ab-base-twin"
OK = {"ok": True, "why": []}


def names(plan, busy):
    leaders, plain, piped, svc = plan
    n = [f"mibz2-{i}" for i in range(leaders)]
    n += [f"mibz2-coop-{i + 1}" for i in range(plain)]
    n += [f"mibz2-coop-p{i + 1}" for i in range(piped)]
    n += [f"mibz2-svc-{i}" for i in range(svc)]
    if busy:
        return " ".join(x + "=0.1" for x in n + ["mibz2-scan"])
    extra = ["ab", "mibz2-read", "mibz2-scan", "sampler"] + (["mibz2-svc-feed"] if svc else [])
    return " ".join(sorted(n + extra))


def grid(case=None, speed=1.05, cpu=1.13, twin=1.0, seed=3):
    rng = random.Random(seed)
    rows = []
    for rnd in range(20):
        for t in sg.THREADS:
            for b, arm in (("base", BASE), ("head", "mibz2"), ("twin", TWIN)):
                if case == "missing round" and (b, t, rnd) == ("head", 10, 5):
                    continue
                plan = sg.EXPECT[t][b]
                if case == "head not svc2" and (b, t, rnd) == ("head", 10, 2):
                    plan = sg.PIPED_3X3
                if case == "svc2 at 11" and (b, t, rnd) == ("head", 11, 2):
                    plan = sg.SVC2
                wall = 0.14 * rng.gauss(1, 0.01)
                c = 1.0 * rng.gauss(1, 0.01)
                if b == "head" and t == 10:
                    wall /= speed
                    c /= cpu
                if b == "twin":
                    wall /= twin
                rows.append(
                    {
                        "round": rnd,
                        "corpus": "wikidata",
                        "threads": t,
                        "arm": arm,
                        "wall": wall,
                        "cpu": c,
                        "rss_kb": 70000 * (0.88 if b == "head" and t == 10 else 1.0),
                        "out": 209700196,
                        "hash": "bad" if case == "wrong bytes" and (b, t, rnd) == ("head", 10, 1) else "h",
                        "thread_cpu": names(plan, True),
                        "observed_threads": names(plan, False),
                        "err": "boom" if case == "failed run" and (b, t, rnd) == ("twin", 9, 0) else "",
                    }
                )
    return rows


def test_clean_grid_reads_as_registered():
    v = sg.judge(grid(), OK)
    assert v["valid"], v["problems"]
    assert v["p1"]["speed"] == "GAIN" and v["p1"]["cpu"] == "CPU SAVING"
    assert v["p1"]["reading"] == "THE SERVICE DEFAULT BEATS THE CURRENT DEFAULT"
    assert v["cells"][10]["rss"]["ratio"] < 1
    assert not v["cells"][9]["flag"] and not v["cells"][11]["flag"]


def test_readings():
    assert sg.judge(grid(speed=1.0, cpu=1.0), OK)["p1"]["reading"] == "NOT REPLICATED"
    assert sg.judge(grid(speed=1.06, cpu=0.9), OK)["p1"]["reading"] == "A TRADE"
    assert sg.judge(grid(speed=0.9, cpu=1.0), OK)["p1"]["speed"] == "REGRESSION"


def test_outcome_map_boundaries():
    o = sg.outcome
    assert o({"lo": 1.021, "hi": 1.1}) == "GAIN"
    assert o({"lo": 1.02, "hi": 1.1}) == "SMALL GAIN"
    assert o({"lo": 1.001, "hi": 1.1}) == "SMALL GAIN"
    assert o({"lo": 0.97, "hi": 1.03}) == "NULL"
    assert o({"lo": 0.96, "hi": 1.03}) == "INCONCLUSIVE"
    assert o({"lo": 0.9, "hi": 0.99}) == "REGRESSION"
    assert o({"lo": 0.9, "hi": 0.99}, cpu=True) == "CPU COST"


def test_each_defect_invalidates():
    for case in ("missing round", "head not svc2", "svc2 at 11", "wrong bytes", "failed run"):
        v = sg.judge(grid(case), OK)
        assert not v["valid"] and v["p1"] is None, case


def test_n1_out_of_band_invalidates():
    v = sg.judge(grid(twin=1.05), OK)
    assert not v["valid"] and any("N1" in p for p in v["problems"])


def test_a_failed_or_missing_preflight_invalidates():
    for pf in (None, {"ok": False, "why": ["10 threads: the predicate is false"]}, {}):
        v = sg.judge(grid(), pf)
        assert not v["valid"] and any("preflight" in p for p in v["problems"])


def test_stray_arms_and_threads():
    rows = grid()
    rows.append(dict(rows[0], arm="mibz2@/elsewhere/ab"))
    assert not sg.judge(rows, OK)["valid"]
    rows = grid()
    rows.append(dict(rows[0], threads=12))
    assert not sg.judge(rows, OK)["valid"]


def test_plan_names():
    assert sg.plan_of(names(sg.SVC2, False)) == sg.SVC2
    assert sg.plan_of(names(sg.PIPED_3X3, True)) == sg.PIPED_3X3
    assert sg.plan_of("mibz2-svc-feed mibz2-read") == (0, 0, 0, 0)
