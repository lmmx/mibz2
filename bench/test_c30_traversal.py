"""bench/c30_traversal.py's schedule checks, gates and decision rules, on synthetic records
(no timing). Run: python3 bench/test_c30_traversal.py

The rejection cases include report 175's five counterexamples (its bundle is kept in
docs/journal/2026-11-02-external-report-175/ on the reports branch, and beside the
protocol), each rebuilt on this fixture's records: one measured pair, workers shifted off
their lists, the candidate's rows doubled, a full step split, and a next-step preparation
inside a prior walk."""

import copy
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
import c30_traversal as t  # noqa: E402

FULL = 900_000
SIZES = [FULL] * 8 + [300, 40]


def make(
    c_cost=0.8,
    c_imbalance=1.0,
    rounds=20,
    noise=0.0,
    dry=False,
    skew=0.0,
    p_drift=0.0,
    c_desched=1.0,
    clock_ns=100,
):
    """The registered schedule on 8 full blocks and 2 short ones, as the harness writes it.
    P walks at 1 ns a row of CPU time. C at `c_cost` on average, with its own per-round
    noise; at W4 its first worker in each all-full step takes `c_imbalance` × the mean and
    the others less, so the completion time is c_cost × c_imbalance while the summed CPU
    time stays c_cost. `c_desched` stretches C's monotonic windows (not its CPU time), as a
    descheduled thread would. `skew` delays worker k's start by skew × k / 3 of its walk.
    Each step: prepare, start barrier, walk, completion barrier, check."""
    lists = t.snake(SIZES)
    meta = {
        "kind": "meta",
        "corpus": "synthetic",
        "blocks": len(SIZES),
        "rows": sum(SIZES),
        "full_blocks": sum(1 for n in SIZES if n >= t.FULL),
        "rounds": rounds,
        "dry_run": dry,
        "inject": None,
        "cpus": [2, 3, 4, 5],
        "w4_lists": lists,
        "w4_rows": [sum(SIZES[b] for b in l) for l in lists],
        "block_rows": SIZES,
        "block_raw_groups": [7] * len(SIZES),
    }
    calib = [
        {
            "kind": "calibration",
            "when": w,
            "cpus": [
                {"cpu": c, "n": 20000, "cpu_ns": [clock_ns] * 5, "wall_ns": [clock_ns + 20] * 5}
                for c in meta["cpus"]
            ],
        }
        for w in ("before", "after")
    ]
    walks = []
    clock = 1000

    def rec(r, mode, arm, k, step, b, prep, start, cpu_d, wall_d, check_start):
        return {
            "kind": "walk", "round": r, "mode": mode, "arm": arm, "worker": k, "step": step,
            "block": b, "rows": SIZES[b], "cpu_before": meta["cpus"][k], "cpu_after": meta["cpus"][k],
            "prep_start": prep[0], "prep_end": prep[1], "start": start, "end": start + wall_d,
            "cpu_start": 10 * start, "cpu_end": 10 * start + cpu_d,
            "check_start": check_start, "check_end": check_start + 10, "exact": True,
        }  # fmt: skip

    for r in range(-1, rounds):
        order = t.arm_order(r)
        for mode in ("w1", "w4"):
            for arm in order:
                if arm == "P":
                    scale, stretch = 1 + p_drift * max(r, 0), 1.0
                else:
                    scale, stretch = c_cost * (1 + noise * math.sin(3 * r + 1)), c_desched
                if mode == "w1":
                    for b, n in enumerate(SIZES):
                        d = int(n * scale)
                        w = int(d * stretch)
                        walks.append(rec(r, "w1", arm, 0, b, b, (clock, clock + 10), clock + 20, d, w,
                                         clock + 20 + w + 5))  # fmt: skip
                        clock += 20 + w + 20
                    continue
                for step in range(max(len(l) for l in lists)):
                    active = [k for k, l in enumerate(lists) if step < len(l)]
                    full = len(active) == 4 and all(SIZES[lists[k][step]] >= t.FULL for k in active)
                    start = clock + 20
                    durs = {}
                    for k in active:
                        n = SIZES[lists[k][step]]
                        m = 1.0
                        if arm == "C" and full:
                            m = c_imbalance if k == 0 else (4 - c_imbalance) / 3
                        d = int(n * scale * m)
                        durs[k] = (d, int(d * stretch), start + int(skew * n * k / 3))
                    last = max(s + w for d, w, s in durs.values())
                    for k in active:
                        d, w, s = durs[k]
                        b = lists[k][step]
                        walks.append(
                            rec(r, "w4", arm, k, step, b, (clock, clock + 10), s, d, w, last + 5)
                        )
                    clock = last + 20
    return meta, walks, calib


def run(meta, walks, calib, listing=True, failures=()):
    return t.analyse(meta, walks, listing, calib, failures)


def find(walks, **kw):
    return next(w for w in walks if all(w[k] == v for k, v in kw.items()))


def rejected(r):
    """An invalid record set: no ratio, no ADVANCE/STOP/INCONCLUSIVE"""
    return r["valid"] is False and r["decision"].startswith("INVALID") and "ratios" not in r


# decisions on a valid run


def test_advance():
    r = run(*make(c_cost=0.80, noise=0.002))
    assert r["valid"] and r["decision"] == "ADVANCE", (r["schedule"], r["gates"])
    assert abs(r["ratios"]["w4"]["point"] - 0.80) < 0.01
    assert abs(r["constructor_budget"]["b_max_at_w4"] - 1.20) < 0.02


def test_stop_on_cost():
    assert run(*make(c_cost=0.95, noise=0.002))["decision"] == "STOP"


def test_stop_on_throughput_harm():
    # the cost passes the screen, but uneven walks make completion 5% slower
    r = run(*make(c_cost=0.80, c_imbalance=1.3125, noise=0.002))
    assert r["valid"], r["gates"]
    assert r["ratios"]["w4"]["hi"] <= t.SCREEN and r["ratios"]["e4"]["lo"] > t.THROUGHPUT_TOL
    assert r["decision"] == "STOP"


def test_inconclusive_straddles_screen():
    r = run(*make(c_cost=0.865, noise=0.03))
    assert r["ratios"]["w4"]["lo"] <= t.SCREEN <= r["ratios"]["w4"]["hi"]
    assert r["decision"] == "INCONCLUSIVE"


def test_inconclusive_throughput():
    r = run(*make(c_cost=0.80, c_imbalance=1.275, noise=0.03))
    assert r["ratios"]["e4"]["lo"] <= t.THROUGHPUT_TOL <= r["ratios"]["e4"]["hi"]
    assert r["decision"] == "INCONCLUSIVE"


def test_uneven_schedule_is_a_result_and_checks_wait():
    # one worker finishes far earlier than another; the checks still start after the
    # step's last walk, and the run is valid
    m, w, c = make(c_cost=0.80, c_imbalance=1.6, noise=0.002)
    step = [
        x for x in w if x["round"] == 0 and x["mode"] == "w4" and x["arm"] == "C" and x["step"] == 0
    ]
    ends = sorted(x["end"] for x in step)
    assert ends[0] < ends[-1] - 300_000
    assert all(x["check_start"] > ends[-1] for x in step)
    r = run(m, w, c)
    assert (
        r["schedule"]["S7_completion_barrier"]
        and r["gates"]["G2_simultaneous_start"]
        and r["valid"]
    )


def test_cost_is_cpu_time_not_elapsed():
    # a descheduled candidate: its monotonic windows stretch 10%, its CPU time does not
    r = run(*make(c_cost=0.80, c_desched=1.10, noise=0.002))
    assert abs(r["ratios"]["w4"]["point"] - 0.80) < 0.01
    assert abs(r["ratios"]["w4_wall"]["point"] - 0.88) < 0.01
    assert r["rounds"][0]["C"]["w4_running_fraction"] < 0.92


def test_budget_at_both_ends():
    r = run(*make(c_cost=0.80, noise=0.02))
    b = r["constructor_budget"]
    assert b["b_max_at_w4_hi"] < b["b_max_at_w4"] < b["b_max_at_w4_lo"]


def test_b_max_table():
    for w, b in ((0.70, 1.51), (0.80, 1.20), (0.865, 1.00), (0.90, 0.89), (1.00, 0.58)):
        assert abs(t.b_max(w) - b) < 0.006
    assert t.b_max(0.865) < 1 < t.b_max(1 - 0.25 / 1.847 - 1e-9)  # 0.865 is rounded


# gates


def test_unstable_control_is_invalid():
    r = run(*make(c_cost=0.80, p_drift=0.01))
    assert not r["gates"]["G1_w1_cpu_ns_row"] and r["decision"].startswith("INVALID")


def test_staggered_start_is_invalid():
    r = run(*make(skew=0.2))
    assert not r["gates"]["G2_simultaneous_start"] and not r["valid"]


def test_listing_mismatch_is_invalid():
    assert run(*make(), listing=False)["decision"].startswith("INVALID")


def test_clock_overhead_bound():
    r = run(*make(clock_ns=6000))  # 10 walks x 6 us against 7.2 ms
    assert not r["gates"]["G5_clock_overhead"] and r["decision"].startswith("INVALID")
    r = run(*make())
    assert r["gates"]["G5_clock_overhead"] and r["clock"]["short_blocks_share_max"] > 0.1


# the calibration evidence (report 178): checked complete before any median is read


def expensive_cpu(c, cpu=4, ns=7500, base=125):
    """Each batch's CPU `cpu` with a larger empty-window median than the others"""
    for k in c:
        for e in k["cpus"]:
            q = ns if e["cpu"] == cpu else base
            e["cpu_ns"] = [q] * 5
            e["wall_ns"] = [q + 20] * 5
    return c


def test_r178_deleting_the_expensive_cpu_rejected():
    # complete: the expensive CPU's estimate fails G5. Deleted from both batches, the
    # pinned c0f8d38 analyser passed G5 and gave ADVANCE; now the evidence is rejected
    m, w, c = make(c_cost=0.8, noise=0.002)
    c = expensive_cpu(c)
    r = run(m, w, c)
    assert r["schedule"]["S9_calibration"] and not r["gates"]["G5_clock_overhead"]
    assert r["clock"]["share_max"] > t.CLOCK_SHARE and r["decision"].startswith("INVALID")
    for k in c:
        k["cpus"] = [e for e in k["cpus"] if e["cpu"] != 4]
    r = run(m, w, c)
    assert not r["schedule"]["S9_calibration"] and rejected(r)


def calib_case(f):
    m, w, c = make(c_cost=0.8, noise=0.002)
    f(m, c)
    return run(m, w, c)


def test_r178_malformed_calibration_rejected():
    def set_(i, cpu, key, value):
        def f(m, c):
            c[i]["cpus"][cpu][key] = value

        return f

    def drop_key(i, cpu, key):
        def f(m, c):
            del c[i]["cpus"][cpu][key]

        return f

    cases = {
        "missing batch": lambda m, c: c.pop(1),
        "empty after batch": lambda m, c: c[1].update(cpus=[]),
        "three batches": lambda m, c: c.append(copy.deepcopy(c[0])),
        "two before batches": lambda m, c: c[1].update(when="before"),
        "unknown when": lambda m, c: c[1].update(when="later"),
        "cpus not a list": lambda m, c: c[0].update(cpus={"cpu": 2}),
        "one CPU missing": lambda m, c: c[0]["cpus"].pop(2),
        "a CPU twice": lambda m, c: c[0]["cpus"].__setitem__(3, copy.deepcopy(c[0]["cpus"][0])),
        "an extra CPU": lambda m, c: c[1]["cpus"].append(dict(c[1]["cpus"][0], cpu=9)),
        "a wrong CPU": set_(0, 1, "cpu", 9),
        "wrong n": set_(1, 2, "n", 2000),
        "n as bool": set_(1, 2, "n", True),
        "four quantiles": set_(0, 0, "cpu_ns", [1, 2, 3, 4]),
        "negative": set_(0, 0, "cpu_ns", [-1, 2, 3, 4, 5]),
        "NaN": set_(0, 0, "wall_ns", [1, float("nan"), 3, 4, 5]),
        "infinite": set_(1, 3, "cpu_ns", [1, 2, 3, 4, float("inf")]),
        "unordered": set_(1, 3, "wall_ns", [1, 5, 3, 4, 6]),
        "a string": set_(0, 2, "cpu_ns", [1, 2, "3", 4, 5]),
        "a bool": set_(0, 2, "cpu_ns", [0, 1, True, 4, 5]),
        "no cpu_ns": drop_key(0, 0, "cpu_ns"),
        "no wall_ns": drop_key(1, 1, "wall_ns"),
        "no cpu id": drop_key(1, 1, "cpu"),
        "an entry not a dict": lambda m, c: c[0]["cpus"].__setitem__(0, [2, 20000]),
        "a batch not a dict": lambda m, c: c.__setitem__(0, ["before"]),
    }
    for name, f in cases.items():
        r = calib_case(f)
        assert not r["schedule"]["S9_calibration"] and rejected(r), name


def test_calibration_in_any_cpu_order_accepted():
    r = calib_case(lambda m, c: c[1]["cpus"].reverse())
    assert r["schedule"]["S9_calibration"] and r["valid"] and r["decision"] == "ADVANCE"


def test_dry_run_checks_calibration_too():
    m, w, c = make(rounds=2, dry=True)
    c[0]["cpus"].pop()
    r = run(m, w, c)
    assert not r["schedule"]["S9_calibration"] and rejected(r)


def test_dry_run_suppresses_timing():
    r = run(*make(rounds=2, dry=True))
    assert "ratios" not in r and "decision" not in r and "rounds" not in r
    assert all(r["schedule"].values()) and r["gates"]["G4_allocation"]


# report 175's five cases: rejected before any ratio


def test_r175_one_pair_rejected():
    r = run(*make(rounds=1, c_cost=0.8))
    assert not r["schedule"]["S2_rounds"] and rejected(r)


def test_r175_workers_shifted_rejected():
    m, w, c = make(c_cost=0.8)
    for x in w:
        if x["mode"] == "w4":
            x["worker"] = (x["worker"] + 1) % 4
            x["cpu_before"] = x["cpu_after"] = m["cpus"][x["worker"]]
    r = run(m, w, c)
    assert not r["schedule"]["S3_record_set"] and rejected(r)


def test_r175_rows_doubled_rejected():
    m, w, c = make(c_cost=0.95)
    assert run(m, w, c)["decision"] == "STOP"
    for x in w:
        if x["mode"] == "w4" and x["arm"] == "C":
            x["rows"] *= 2
    r = run(m, w, c)
    assert not r["schedule"]["S3_record_set"] and rejected(r)


def test_r175_split_step_rejected():
    m, w, c = make(c_cost=0.6)
    for x in w:
        if x["mode"] == "w4" and x["arm"] == "C" and x["step"] == 0 and x["worker"] == 0:
            x["step"] = 99
    r = run(m, w, c)
    assert not r["schedule"]["S3_record_set"] and rejected(r)


def test_r175_next_preparation_inside_prior_walk_rejected():
    m, w, c = make(c_cost=0.8)
    prev = find(w, round=0, arm="C", mode="w4", step=0, worker=0)
    nxt = find(w, round=0, arm="C", mode="w4", step=1, worker=0)
    nxt["prep_start"] = prev["start"] + 1
    nxt["prep_end"] = prev["end"] - 1
    r = run(m, w, c)
    assert not r["schedule"]["S7_cross_step"] and rejected(r)


# the measurement boundary


def test_check_inside_a_peers_walk_rejected():
    m, w, c = make(c_cost=0.8, c_imbalance=1.3)
    early = find(w, round=3, arm="C", mode="w4", step=1, worker=1)
    early["check_start"] = early["end"] + 1  # before worker 0's longer walk ends
    early["check_end"] = early["check_start"] + 10
    r = run(m, w, c)
    assert not r["schedule"]["S7_completion_barrier"] and rejected(r)


def test_prep_after_a_peers_start_rejected():
    m, w, c = make()
    x = find(w, round=0, arm="P", mode="w4", step=1, worker=1)
    x["prep_end"] = x["start"] + 5
    x["prep_start"] = x["prep_end"] - 1
    x["start"] = x["prep_end"]
    r = run(m, w, c)
    assert not r["schedule"]["S7_start_barrier"] and rejected(r)


def test_w1_check_overlapping_next_prep_rejected():
    m, w, c = make()
    a = find(w, round=0, arm="C", mode="w1", step=2)
    b = find(w, round=0, arm="C", mode="w1", step=3)
    b["prep_start"] = a["check_end"] - 5
    r = run(m, w, c)
    assert not r["schedule"]["S6_w1_sequence"] and rejected(r)


def test_cpu_window_longer_than_its_walk_rejected():
    m, w, c = make()
    x = find(w, round=0, arm="C", mode="w4", step=0, worker=2)
    x["cpu_end"] = x["cpu_start"] + (x["end"] - x["start"]) + 10_000
    r = run(m, w, c)
    assert not r["schedule"]["S5_walk_windows"] and rejected(r)


# the record set


def test_missing_walk_rejected():
    m, w, c = make()
    r = run(m, w[:5] + w[6:], c)
    assert not r["schedule"]["S3_record_set"] and rejected(r)


def test_duplicate_walk_rejected():
    m, w, c = make()
    r = run(m, w + [copy.deepcopy(w[7])], c)
    assert not r["schedule"]["S3_record_set"] and rejected(r)


def test_unexpected_walk_rejected():
    m, w, c = make()
    x = copy.deepcopy(find(w, round=0, arm="P", mode="w4", step=2, worker=0))
    x["step"], x["worker"] = 2, 2  # worker 2 has no third block
    x["cpu_before"] = x["cpu_after"] = m["cpus"][2]
    r = run(m, w + [x], c)
    assert not r["schedule"]["S3_record_set"] and rejected(r)


def test_meta_lists_not_the_snake_rejected():
    m, w, c = make()
    m["w4_lists"] = [[0, 7, 8, 1], [6, 9], [2, 5], [3, 4]]
    m["w4_rows"] = [sum(SIZES[b] for b in l) for l in m["w4_lists"]]
    r = run(m, w, c)
    assert not r["schedule"]["S1_meta"] and rejected(r)


def test_wrong_cpu_rejected():
    m, w, c = make()
    find(w, round=4, arm="C", mode="w4", step=0, worker=3)["cpu_after"] = 9
    r = run(m, w, c)
    assert not r["schedule"]["S4_placement"] and rejected(r)


def test_arm_order_swapped_rejected():
    m, w, c = make()
    for x in w:  # round 0 is registered P first; relabel its W4 passes
        if x["round"] == 0 and x["mode"] == "w4":
            x["arm"] = "C" if x["arm"] == "P" else "P"
    r = run(m, w, c)
    assert not r["schedule"]["S8_pass_order"] and rejected(r)


def test_labels_rejected():
    # a dry run must have the dry rounds; a capture the registered 20 and no injection
    assert not run(*make(rounds=20, dry=True))["schedule"]["S2_rounds"]
    assert not run(*make(rounds=2, dry=False))["schedule"]["S2_rounds"]
    m, w, c = make()
    m["inject"] = "output:w4:C:2"
    assert rejected(run(m, w, c))


# failures


def test_failure_record_rejected():
    m, w, c = make()
    f = [{"kind": "failure", "what": "output", "detail": "round -1 w4 arm C worker 2 ..."}]
    r = run(m, w, c, failures=f)
    assert not r["schedule"]["S0_no_failure"] and rejected(r) and r["failures"] == f


def test_failed_run_records_rejected():
    # what an injected failure leaves: the meta, a calibration, the failure, no walks
    m, _, c = make(rounds=2, dry=True)
    f = [{"kind": "failure", "what": "pin", "detail": "W4 worker 1: could not pin to CPU 1023"}]
    r = run(m, [], c[:1], failures=f)
    assert rejected(r) and not r["schedule"]["S3_record_set"]


def test_interval_refuses_other_sample_sizes():
    for n in (1, 2, 19, 21):
        try:
            t.interval([0.8] * n)
        except ValueError:
            continue
        raise AssertionError(f"interval accepted {n} rounds")


if __name__ == "__main__":
    n = 0
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print("ok", name)
            n += 1
    print(f"{n} tests passed")
