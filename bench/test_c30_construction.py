"""bench/c30_construction.py's schedule checks, gates and decision table, on synthetic
records (no timing). Run: python3 bench/test_c30_construction.py

Every defect fixture starts from a record set that would otherwise decide PASS (and, where
it matters, one that would decide FAIL), and must come out INVALID: a defect can never
yield PASS, FAIL or INCONCLUSIVE. The positive fixtures reach PASS, FAIL and INCONCLUSIVE
at the frozen thresholds (b4 1.06714, e4 1.02)."""

import copy
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
import c30_construction as t  # noqa: E402

FULL = 900_000
SIZES = [FULL] * 8 + [300, 40]


def make(c_cost=1.0, c_imbalance=1.0, cold_cost=None, rounds=20, noise=0.0, dry=False,
         skew=0.0, p_drift=0.0, clock_ns=100, p_imbalance=1.0):  # fmt: skip
    """The registered schedule on 8 full blocks and 2 short ones, as `c30walk cbuild`
    writes it. P builds at 1 ns a row of CPU time; C at `c_cost` (warm) or `cold_cost`
    (cold), with its own per-round noise. In each all-full W4 step C's first worker takes
    `c_imbalance` × the mean and the others less, so e4 = c_cost × c_imbalance while b4
    stays c_cost. `p_imbalance` does the same to P's steps, so e4 = c_cost × c_imbalance /
    p_imbalance. `skew` delays worker k's start by skew × k / 3 of its build."""
    cold_cost = c_cost if cold_cost is None else cold_cost
    lists = t.snake(SIZES)
    cpus = [2, 3, 4, 5]
    meta = {
        "kind": "meta", "corpus": "synthetic", "blocks": len(SIZES), "rows": sum(SIZES),
        "full_blocks": sum(1 for n in SIZES if n >= t.FULL), "rounds": rounds, "dry_run": dry,
        "inject": None, "cpus": cpus, "regimes": ["warm", "cold"], "w4_lists": lists,
        "w4_rows": [sum(SIZES[b] for b in l) for l in lists], "block_rows": SIZES,
        "block_raw_groups": [7] * len(SIZES), "memory_estimate": 10 << 30,
    }  # fmt: skip
    est = 10 << 30
    pre = {
        "kind": "preflight", "estimate": est, "required": est, "margin": max(2 << 30, est // 4),
        "mem_available": est + max(2 << 30, est // 4) + 1, "swap_used": 0, "psi_present": True,
        "psi_some_avg10": 0.0, "psi_full_avg10": 0.0, "pswpin": 0, "pswpout": 0,
        "injected": False, "passed": True,
    }  # fmt: skip
    post = {
        "kind": "postflight", "estimate": est, "required": est, "vm_hwm": est // 2,
        "swap_used": 0, "psi_present": True, "psi_some_avg10": 0.0, "psi_full_avg10": 0.0,
        "pswpin_delta": 0, "pswpout_delta": 0,
    }  # fmt: skip
    onepass = [{"kind": "onepass", "block": b, "rows": n, "histogram_equal": True,
                "below_2_24": True} for b, n in enumerate(SIZES)]  # fmt: skip
    counters = [{"kind": "counters", "block": b, "rows": n, "rows_complete": n // 2,
                 "rows_fragment": n // 10, "rows_scalar": n - n // 2 - n // 10,
                 "wide_promotions": b} for b, n in enumerate(SIZES)]  # fmt: skip
    check0 = [{"kind": "check0", "arm": "C", "block": b, "poisoned": True, "exact": True,
               "walk": True, "restored": True} for b in range(len(SIZES))] + [
              {"kind": "check0", "arm": "P", "block": b, "definition": True, "walk": True,
               "restored": True} for b in range(len(SIZES))]  # fmt: skip
    calib = [
        {"kind": "calibration", "when": w,
         "cpus": [{"cpu": c, "n": 20000, "cpu_ns": [clock_ns] * 5,
                   "wall_ns": [clock_ns + 20] * 5} for c in cpus]}
        for w in ("before", "after")
    ]  # fmt: skip
    builds = []
    clock = 1000

    def rec(r, rg, mode, arm, k, step, b, prep, start, cpu_d, check_start):
        return {
            "kind": "build", "round": r, "regime": rg, "mode": mode, "arm": arm, "worker": k,
            "step": step, "block": b, "rows": SIZES[b], "cpu_before": cpus[k],
            "cpu_after": cpus[k], "prep_start": prep[0], "prep_end": prep[1], "start": start,
            "end": start + cpu_d, "cpu_start": 10 * start, "cpu_end": 10 * start + cpu_d,
            "minflt": 3, "majflt": 0, "check_start": check_start, "check_end": check_start + 10,
            "cap_bytes": 4 * SIZES[b], "usable_bytes": 4 * SIZES[b] + 8, "exact": True,
        }  # fmt: skip

    for r in range(-1, rounds):
        for rg, mode in t.PASSES:
            for arm in t.arm_order(r):
                if arm == "P":
                    scale = 1 + p_drift * max(r, 0)
                else:
                    base = c_cost if rg == "warm" else cold_cost
                    scale = base * (1 + noise * math.sin(3 * r + 1))
                if mode == "w1":
                    for b, n in enumerate(SIZES):
                        d = max(int(n * scale), 1)
                        builds.append(rec(r, rg, "w1", arm, 0, b, b, (clock, clock + 10),
                                          clock + 20, d, clock + 20 + d + 5))  # fmt: skip
                        clock += 20 + d + 20
                    continue
                for step in range(max(len(l) for l in lists)):
                    active = [k for k, l in enumerate(lists) if step < len(l)]
                    full = len(active) == 4 and all(SIZES[lists[k][step]] >= t.FULL for k in active)
                    start = clock + 20
                    durs = {}
                    for k in active:
                        n = SIZES[lists[k][step]]
                        m = 1.0
                        imb = c_imbalance if arm == "C" else p_imbalance
                        if full:
                            m = imb if k == 0 else (4 - imb) / 3
                        d = max(int(n * scale * m), 1)
                        durs[k] = (d, start + int(skew * n * k / 3))
                    last = max(s + d for d, s in durs.values())
                    for k in active:
                        d, s = durs[k]
                        builds.append(rec(r, rg, "w4", arm, k, step, lists[k][step],
                                          (clock, clock + 10), s, d, last + 5))  # fmt: skip
                    clock = last + 40
    recs = {k: [] for k in t.KNOWN}
    recs["meta"] = [meta]
    recs["preflight"] = [pre]
    recs["postflight"] = [post]
    recs["onepass"] = onepass
    recs["counters"] = counters
    recs["check0"] = check0
    recs["calibration"] = calib
    recs["build"] = builds
    return recs


def run(recs, identity=True, other=()):
    return t.analyse(recs, list(other), identity)


def invalid(r):
    """Rejected: INVALID, never a decision"""
    return r["valid"] is False and r["decision"].startswith("INVALID")


def before_figures(r):
    """Rejected before any figure: no ratio was computed"""
    return invalid(r) and "ratios" not in r


def find(builds, **kw):
    return next(w for w in builds if all(w[k] == v for k, v in kw.items()))


def base_pass():
    return make(c_cost=1.00, noise=0.002)


def base_fail():
    return make(c_cost=1.20, noise=0.002)


# positive fixtures: the decision table at the frozen thresholds


def test_pass():
    r = run(base_pass())
    assert r["valid"] and r["decision"] == "PASS", (r["schedule"], r["gates"])
    assert abs(r["ratios"]["warm_b4"]["point"] - 1.0) < 0.01


def test_pass_at_the_ceiling_edge():
    # b4 ≈ 1.06 with a narrow interval stays under 1.06714 (P's steps uneven, so e4 ≈ 0.96)
    r = run(make(c_cost=1.06, p_imbalance=1.1, noise=0.0005))
    assert r["decision"] == "PASS" and r["ratios"]["warm_b4"]["hi"] <= t.CEILING


def test_fail_on_cost():
    r = run(base_fail())
    assert r["valid"] and r["decision"] == "FAIL"
    assert r["ratios"]["warm_b4"]["lo"] > t.CEILING


def test_fail_on_completion():
    # cheaper CPU, but one worker's build 10% longer than the mean: e4 ≈ 1.1
    r = run(make(c_cost=1.0, c_imbalance=1.1, noise=0.002))
    assert r["valid"] and r["decision"] == "FAIL", r["ratios"]
    assert (
        r["ratios"]["warm_b4"]["hi"] <= t.CEILING
        and r["ratios"]["warm_e4"]["lo"] > t.COMPLETION_TOL
    )


def test_inconclusive_on_cost():
    # P's steps 10% uneven, so e4 ≈ 0.97 stays under 1.02 while b4 straddles 1.06714
    r = run(make(c_cost=1.067, p_imbalance=1.1, noise=0.03))
    b4 = r["ratios"]["warm_b4"]
    assert r["valid"] and r["decision"] == "INCONCLUSIVE"
    assert b4["lo"] <= t.CEILING < b4["hi"]


def test_inconclusive_on_completion():
    r = run(make(c_cost=0.9, c_imbalance=1.0 / 0.9 * 1.02, noise=0.02))
    e4 = r["ratios"]["warm_e4"]
    assert r["valid"] and r["decision"] == "INCONCLUSIVE", r["ratios"]
    assert e4["lo"] <= t.COMPLETION_TOL < e4["hi"]


def test_cold_regime_never_decides():
    # cold twice as dear, warm passes: PASS, the cold ratio reported
    r = run(make(c_cost=1.0, cold_cost=2.0, noise=0.002))
    assert r["decision"] == "PASS" and r["ratios"]["cold_b4"]["point"] > 1.9


def test_one_core_never_decides():
    # the one-core ratio is reported only; it equals b4 here, so a FAIL on cost is from b4
    r = run(base_pass())
    assert "warm_b1" in r["ratios"] and r["decision"] == "PASS"


def test_dry_run_suppresses_timing_and_checks_the_rest():
    r = run(make(rounds=2, dry=True))
    assert r["valid"] is None and "ratios" not in r and "decision" not in r
    assert all(v is True for v in r["schedule"].values())
    assert all(r["gates"][k] is True for k in ("G4_allocation", "G6_exact_output",
               "G7_round_minus_one", "G8_one_pass", "G9_memory", "G10_major_faults"))  # fmt: skip


def test_dry_run_still_rejects_a_memory_violation():
    rc = make(rounds=2, dry=True)
    rc["postflight"][0]["pswpout_delta"] = 1
    r = run(rc)
    assert r["gates"]["G9_memory"] is False


# malformed schedules


def _bases():
    return (base_pass(), base_fail())


def test_missing_build_rejected():
    for rc in _bases():
        rc["build"].remove(find(rc["build"], round=3, regime="warm", mode="w4", arm="C", worker=2))
        assert before_figures(run(rc))


def test_duplicate_build_rejected():
    for rc in _bases():
        rc["build"].append(
            copy.deepcopy(find(rc["build"], round=0, regime="warm", mode="w1", arm="P"))
        )
        assert before_figures(run(rc))


def test_extra_round_rejected():
    for rc in _bases():
        extra = copy.deepcopy(find(rc["build"], round=19, regime="cold", mode="w1", arm="C"))
        extra["round"] = 20
        rc["build"].append(extra)
        assert before_figures(run(rc))


def test_lists_not_the_snake_rejected():
    for rc in _bases():
        m = rc["meta"][0]
        m["w4_lists"] = [m["w4_lists"][1], m["w4_lists"][0]] + m["w4_lists"][2:]
        assert before_figures(run(rc))


def test_workers_shifted_rejected():
    for rc in _bases():
        for w in rc["build"]:
            if w["mode"] == "w4" and w["round"] == 4:
                w["worker"] = (w["worker"] + 1) % 4
        assert before_figures(run(rc))


def test_wrong_cpu_rejected():
    for rc in _bases():
        find(rc["build"], round=5, regime="warm", mode="w4", arm="C", worker=1)["cpu_after"] = 9
        assert before_figures(run(rc))


def test_arm_order_swapped_rejected():
    for rc in _bases():
        # round 2 (P first) laid out as C first: shift P's W1 warm pass after C's
        v = [
            w
            for w in rc["build"]
            if (w["round"], w["regime"], w["mode"], w["arm"]) == (2, "warm", "w1", "P")
        ]
        c = [
            w
            for w in rc["build"]
            if (w["round"], w["regime"], w["mode"], w["arm"]) == (2, "warm", "w1", "C")
        ]
        shift = max(x["check_end"] for x in c) - min(x["prep_start"] for x in v) + 1
        for w in v:
            for f in ("prep_start", "prep_end", "start", "end", "check_start", "check_end"):
                w[f] += shift
        assert before_figures(run(rc))


def test_start_before_a_peers_preparation_rejected():
    for rc in _bases():
        w = find(rc["build"], round=1, regime="cold", mode="w4", arm="P", worker=3, step=0)
        w["prep_end"] = w["start"] + 1
        w["prep_start"] = w["prep_end"] - 5
        assert before_figures(run(rc))


def test_check_inside_a_peers_build_rejected():
    for rc in _bases():
        v = [w for w in rc["build"] if (w["round"], w["regime"], w["mode"], w["arm"], w["step"])
             == (6, "warm", "w4", "C", 1)]  # fmt: skip
        # worker 0 finishes early and checks while its peers still build
        first = v[0]
        first["end"] -= 100
        first["cpu_end"] -= 100
        first["check_start"] = first["end"]
        assert before_figures(run(rc))


def test_w1_overlap_rejected():
    for rc in _bases():
        a = find(rc["build"], round=0, regime="warm", mode="w1", arm="C", step=2)
        b = find(rc["build"], round=0, regime="warm", mode="w1", arm="C", step=3)
        a["check_end"] = b["prep_start"] + 1
        assert before_figures(run(rc))


def test_cpu_window_longer_than_its_build_rejected():
    for rc in _bases():
        w = find(rc["build"], round=7, regime="warm", mode="w1", arm="C", step=0)
        w["cpu_end"] = w["cpu_start"] + (w["end"] - w["start"]) + t.CPU_SLACK_NS + 1
        assert before_figures(run(rc))


def test_negative_faults_rejected():
    for rc in _bases():
        find(rc["build"], round=0, regime="cold", mode="w1", arm="P")["minflt"] = -1
        assert before_figures(run(rc))


def test_unknown_record_rejected():
    for rc in _bases():
        assert before_figures(run(rc, other=[{"kind": "mystery"}]))


def test_no_meta_rejected():
    rc = base_pass()
    rc["meta"] = []
    assert invalid(run(rc))


# wrong denominators


def test_candidate_rows_doubled_rejected():
    for rc in _bases():
        for w in rc["build"]:
            if w["arm"] == "C":
                w["rows"] *= 2
        assert before_figures(run(rc))


def test_meta_rows_wrong_rejected():
    for rc in _bases():
        rc["meta"][0]["rows"] += 1
        assert before_figures(run(rc))


def test_block_rows_rewritten_rejected():
    # a meta record whose block rows (and so its denominators and snake) changed, with the
    # records left as they were
    for rc in _bases():
        m = rc["meta"][0]
        m["block_rows"] = [n * 2 for n in m["block_rows"]]
        m["rows"] = sum(m["block_rows"])
        m["w4_lists"] = t.snake(m["block_rows"])
        m["w4_rows"] = [sum(m["block_rows"][b] for b in l) for l in m["w4_lists"]]
        assert before_figures(run(rc))


def test_full_block_count_wrong_rejected():
    for rc in _bases():
        rc["meta"][0]["full_blocks"] -= 1
        assert before_figures(run(rc))


# missing or malformed calibration


def test_no_calibration_rejected():
    for rc in _bases():
        rc["calibration"] = []
        assert before_figures(run(rc))


def test_one_calibration_rejected():
    for rc in _bases():
        rc["calibration"] = rc["calibration"][:1]
        assert before_figures(run(rc))


def test_calibration_missing_a_cpu_rejected():
    for rc in _bases():
        rc["calibration"][1]["cpus"] = rc["calibration"][1]["cpus"][:3]
        assert before_figures(run(rc))


def test_calibration_short_or_malformed_rejected():
    for change in (
        lambda c: c.update(n=19_999),
        lambda c: c.update(cpu_ns=[100, 90, 100, 100, 100]),
        lambda c: c.update(wall_ns=[100, 100, 100, float("nan"), 100]),
        lambda c: c.update(cpu_ns=[100] * 4),
        lambda c: c.update(n=True),
    ):
        for rc in _bases():
            change(rc["calibration"][0]["cpus"][2])
            assert before_figures(run(rc))


def test_clock_overhead_too_large_invalid():
    rc = make(c_cost=1.0, noise=0.002, clock_ns=10_000)
    r = run(rc)
    assert invalid(r) and r["gates"]["G5_clock_overhead"] is False


# correctness failures


def test_failure_record_rejected():
    for rc in _bases():
        rc["failure"] = [{"kind": "failure", "what": "output", "detail": "round 3 ..."}]
        r = run(rc)
        assert before_figures(r) and r["failures"]


def test_failed_run_records_rejected():
    # what an injected failure leaves: the meta, the preamble, a calibration, the failure
    rc = make(rounds=2, dry=True)
    rc["build"] = rc["build"][:5]
    rc["calibration"] = rc["calibration"][:1]
    rc["postflight"] = []
    rc["failure"] = [{"kind": "failure", "what": "pin", "detail": "W4 worker 1"}]
    assert before_figures(run(rc))


def test_inexact_build_invalid():
    for rc in _bases():
        find(rc["build"], round=9, regime="warm", mode="w4", arm="C", worker=0)["exact"] = False
        r = run(rc)
        assert invalid(r) and r["gates"]["G6_exact_output"] is False


def test_round_minus_one_check_failed_invalid():
    for rc in _bases():
        rc["check0"][3]["walk"] = False
        r = run(rc)
        assert invalid(r) and r["gates"]["G7_round_minus_one"] is False


def test_round_minus_one_check_missing_rejected():
    for rc in _bases():
        rc["check0"] = rc["check0"][1:]
        assert before_figures(run(rc))


def test_one_pass_precondition_failed_invalid():
    for rc in _bases():
        rc["onepass"][2]["histogram_equal"] = False
        r = run(rc)
        assert invalid(r) and r["gates"]["G8_one_pass"] is False


def test_one_pass_records_missing_rejected():
    for rc in _bases():
        rc["onepass"] = rc["onepass"][:-1]
        assert before_figures(run(rc))


def test_counter_pass_missing_rejected():
    for rc in _bases():
        rc["counters"] = rc["counters"][1:]
        assert before_figures(run(rc))


def test_identity_failed_or_unknown_invalid():
    for ident in (False, None):
        r = run(base_pass(), identity=ident)
        assert invalid(r)


# memory-gate violations


def _memory(change, where="preflight"):
    for rc in _bases():
        change(rc[where][0])
        r = run(rc)
        assert invalid(r) and r["gates"]["G9_memory"] is False, r["gates"]


def test_memory_margin_short_invalid():
    _memory(lambda p: p.update(mem_available=p["required"] + p["margin"] - 1))


def test_memory_margin_misstated_invalid():
    _memory(lambda p: p.update(margin=1 << 30, mem_available=p["required"] + (1 << 30)))


def test_memory_required_below_estimate_invalid():
    _memory(lambda p: p.update(required=p["estimate"] - 1))


def test_swap_in_use_before_invalid():
    _memory(lambda p: p.update(swap_used=4096))


def test_pressure_before_invalid():
    _memory(lambda p: p.update(psi_some_avg10=0.5))


def test_preflight_not_passed_invalid():
    _memory(lambda p: p.update(passed=False))


def test_preflight_injected_invalid():
    _memory(lambda p: p.update(injected=True))


def test_swap_in_use_after_invalid():
    _memory(lambda p: p.update(swap_used=1), "postflight")


def test_swap_activity_invalid():
    _memory(lambda p: p.update(pswpin_delta=2), "postflight")
    _memory(lambda p: p.update(pswpout_delta=1), "postflight")


def test_pressure_after_invalid():
    _memory(lambda p: p.update(psi_full_avg10=0.01), "postflight")


def test_preflight_or_postflight_missing_rejected():
    for where in ("preflight", "postflight"):
        for rc in _bases():
            rc[where] = []
            assert before_figures(run(rc))


# major faults


def test_major_fault_in_a_measured_build_invalid():
    for rc in _bases():
        find(rc["build"], round=11, regime="cold", mode="w4", arm="P", worker=2)["majflt"] = 1
        r = run(rc)
        assert invalid(r) and r["gates"]["G10_major_faults"] is False


def test_major_fault_in_round_minus_one_reported_not_gated():
    rc = base_pass()
    find(rc["build"], round=-1, regime="warm", mode="w1", arm="C")["majflt"] = 2
    r = run(rc)
    assert r["decision"] == "PASS" and r["faults"]["major_in_round_minus_one"] == 2


# insufficient measured rounds


def test_too_few_rounds_rejected():
    for n in (1, 2, 19):
        for base in (dict(c_cost=1.0), dict(c_cost=1.2)):
            r = run(make(rounds=n, noise=0.002, **base))
            assert before_figures(r) and r["schedule"]["S2_rounds"] is False


def test_dry_run_label_with_twenty_rounds_rejected():
    rc = base_pass()
    rc["meta"][0]["dry_run"] = True
    assert before_figures(run(rc))


def test_capture_with_injection_rejected():
    rc = base_pass()
    rc["meta"][0]["inject"] = "output:w4:C:2"
    assert before_figures(run(rc))


def test_interval_refuses_other_sample_sizes():
    for n in (1, 2, 19, 21):
        try:
            t.interval([1.0] * n)
        except ValueError:
            continue
        raise AssertionError(f"interval accepted {n} rounds")


# validity gates on figures


def test_unstable_control_invalid():
    r = run(make(c_cost=1.0, noise=0.002, p_drift=0.01))
    assert invalid(r) and any(k.startswith("G1_") and v is False for k, v in r["gates"].items())


def test_staggered_start_invalid():
    r = run(make(c_cost=1.0, noise=0.002, skew=0.1))
    assert invalid(r) and r["gates"]["G2_simultaneous_start"] is False


if __name__ == "__main__":
    n = 0
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print("ok", name)
            n += 1
    print(f"{n} tests passed")
