"""bench/c30_traversal.py's decision rules and gates, on synthetic records (no timing)"""

import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
import c30_traversal as t  # noqa: E402

FULL = 900_000


def make(
    c_cost=0.8, c_imbalance=1.0, rounds=20, noise=0.0, dry=False, skew=0.0, p_drift=0.0, drop=None
):
    """8 full blocks, 2 short. P walks at 1 ns a row. C walks at `c_cost` a row on average,
    with per-round noise of its own; at W4 its first worker in each step takes
    `c_imbalance` × the mean and the others less, so the elapsed time is c_cost × c_imbalance
    while the summed worker time stays c_cost. `skew` delays worker k's start by
    skew × k / 3 of the step's walk."""
    sizes = [FULL] * 8 + [300, 40]
    lists = [[0, 7, 8], [1, 6, 9], [2, 5], [3, 4]]
    rows = [sum(sizes[b] for b in l) for l in lists]
    meta = {
        "kind": "meta",
        "blocks": len(sizes),
        "rows": sum(sizes),
        "rounds": rounds,
        "dry_run": dry,
        "cpus": [0, 1, 2, 3],
        "w4_lists": lists,
        "w4_rows": rows,
        "block_rows": sizes,
        "block_raw_groups": [7] * len(sizes),
    }
    walks = []
    clock = 0
    for r in range(-1, rounds):
        for arm in ("P", "C"):
            if arm == "P":
                scale = 1 + p_drift * max(r, 0)
            else:
                scale = c_cost * (1 + noise * math.sin(3 * r + 1))
            for b, n in enumerate(sizes):
                d = int(n * scale)
                walks.append(
                    {
                        "kind": "walk",
                        "round": r,
                        "mode": "w1",
                        "arm": arm,
                        "worker": 0,
                        "step": b,
                        "block": b,
                        "rows": n,
                        "cpu": 0,
                        "prep_start": clock,
                        "prep_end": clock + 10,
                        "start": clock + 20,
                        "end": clock + 20 + d,
                        "exact": True,
                    }
                )
                clock += 30 + d
            for step in range(3):
                start = clock + 20
                active = [k for k, l in enumerate(lists) if step < len(l)]
                for k in active:
                    b = lists[k][step]
                    n = sizes[b]
                    m = 1.0
                    if arm == "C" and len(active) == 4:
                        m = c_imbalance if k == 0 else (4 - c_imbalance) / 3
                    d = int(n * scale * m)
                    s = start + int(skew * n * k / 3)
                    walks.append(
                        {
                            "kind": "walk",
                            "round": r,
                            "mode": "w4",
                            "arm": arm,
                            "worker": k,
                            "step": step,
                            "block": b,
                            "rows": n,
                            "cpu": k,
                            "prep_start": clock,
                            "prep_end": clock + 10,
                            "start": s,
                            "end": s + d,
                            "exact": True,
                        }
                    )
                clock = start + 4 * FULL * 2
    if drop is not None:
        walks = [w for i, w in enumerate(walks) if i != drop]
    return meta, walks


def test_advance():
    m, w = make(c_cost=0.80, noise=0.002)
    r = t.analyse(m, w, True)
    assert r["valid"] and r["decision"] == "ADVANCE"
    assert abs(r["ratios"]["w4"]["point"] - 0.80) < 0.01
    assert abs(r["constructor_budget"]["b_max_at_w4"] - 1.20) < 0.02


def test_stop_on_cost():
    m, w = make(c_cost=0.95, noise=0.002)
    assert t.analyse(m, w, True)["decision"] == "STOP"


def test_stop_on_throughput_harm():
    # the cost ratio passes the screen, but uneven walks make completed work 5% slower
    m, w = make(c_cost=0.80, c_imbalance=1.3125, noise=0.002)
    r = t.analyse(m, w, True)
    assert r["valid"], r["gates"]
    assert r["ratios"]["w4"]["hi"] <= t.SCREEN and r["ratios"]["e4"]["lo"] > t.THROUGHPUT_TOL
    assert r["decision"] == "STOP"


def test_inconclusive_straddles_screen():
    m, w = make(c_cost=0.865, noise=0.03)
    r = t.analyse(m, w, True)
    assert r["ratios"]["w4"]["lo"] <= t.SCREEN <= r["ratios"]["w4"]["hi"]
    assert r["decision"] == "INCONCLUSIVE"


def test_inconclusive_throughput():
    m, w = make(c_cost=0.80, c_imbalance=1.275, noise=0.03)
    r = t.analyse(m, w, True)
    assert r["ratios"]["e4"]["lo"] <= t.THROUGHPUT_TOL <= r["ratios"]["e4"]["hi"]
    assert r["decision"] == "INCONCLUSIVE"


def test_unstable_control_is_invalid():
    m, w = make(c_cost=0.80, p_drift=0.01)
    r = t.analyse(m, w, True)
    assert not r["gates"]["G1_w1_ns_row"] and r["decision"].startswith("INVALID")


def test_staggered_start_is_invalid():
    m, w = make(skew=0.2)
    r = t.analyse(m, w, True)
    assert not r["gates"]["G2_simultaneous_start"] and not r["valid"]


def test_uneven_walks_are_a_result_not_invalid():
    m, w = make(c_cost=0.80, c_imbalance=1.2)
    r = t.analyse(m, w, True)
    assert r["gates"]["G2_simultaneous_start"] and r["valid"]


def test_missing_walk_is_invalid():
    m, w = make(drop=5)
    r = t.analyse(m, w, True)
    assert not r["gates"]["G0_outputs"] and not r["valid"]


def test_listing_mismatch_is_invalid():
    m, w = make()
    assert t.analyse(m, w, False)["decision"].startswith("INVALID")


def test_dry_run_suppresses_timing():
    m, w = make(rounds=2, dry=True)
    r = t.analyse(m, w, True)
    assert "ratios" not in r and "decision" not in r and "rounds" not in r
    assert r["gates"]["G0_outputs"] and r["gates"]["G4_allocation"] and all(r["path"].values())


def test_barrier_violation_detected():
    m, w = make()
    for x in w:
        if x["mode"] == "w4" and x["worker"] == 1:
            x["prep_end"] = x["start"] + 5  # prepared after another worker started
            x["prep_start"] = x["prep_end"] - 1
            x["start"] = x["prep_end"]
            x["end"] = x["start"] + 1
            break
    assert not t.analyse(m, w, True)["path"]["barrier"]


def test_b_max_table():
    for w, b in ((0.70, 1.51), (0.80, 1.20), (0.865, 1.00), (0.90, 0.89), (1.00, 0.58)):
        assert abs(t.b_max(w) - b) < 0.006


def test_allocation_bound_is_one_block():
    m, w = make()
    assert t.analyse(m, w, True)["gates"]["G4_allocation"]
    m["w4_lists"] = [[0, 7, 8, 1], [6, 9], [2, 5], [3, 4]]  # one worker two full blocks over
    m["w4_rows"] = [sum(m["block_rows"][b] for b in l) for l in m["w4_lists"]]
    assert not t.analyse(m, w, True)["gates"]["G4_allocation"]


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print("ok", name)
