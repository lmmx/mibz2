"""bench/c30_construction.py's schedule checks, gates and decision table, on synthetic
records (no timing). Run: python3 bench/test_c30_construction.py

Every defect fixture starts from a record set that would otherwise decide PASS (and, where
it matters, one that would decide FAIL), and must come out INVALID: a defect can never
yield PASS, FAIL or INCONCLUSIVE. The positive fixtures reach PASS, FAIL and INCONCLUSIVE
at the frozen thresholds (b4 1.06714, e4 1.02)."""

import copy
import hashlib
import json
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
import c30_construction as t  # noqa: E402

FULL = 900_000
SIZES = [FULL] * 8 + [300, 40]
MACHINE = {"hostname": "vm", "machine_id": "abc", "kernel": "6", "cpu_model": "x",
           "cpus_online": "0-5", "mem_total": 1 << 34}  # fmt: skip


def make(c_cost=1.0, c_imbalance=1.0, cold_cost=None, rounds=20, noise=0.0, dry=False,
         skew=0.0, p_drift=0.0, clock_ns=100, p_imbalance=1.0, sizes=None, skew_step=None):  # fmt: skip
    """The registered schedule on 8 full blocks and 2 short ones, as `c30walk cbuild`
    writes it. P builds at 1 ns a row of CPU time; C at `c_cost` (warm) or `cold_cost`
    (cold), with its own per-round noise. In each all-full W4 step C's first worker takes
    `c_imbalance` × the mean and the others less, so e4 = c_cost × c_imbalance while b4
    stays c_cost. `p_imbalance` does the same to P's steps, so e4 = c_cost × c_imbalance /
    p_imbalance. `skew` delays worker k's start by skew × k / 3 of its build."""
    cold_cost = c_cost if cold_cost is None else cold_cost
    SIZES = list(sizes or globals()["SIZES"])  # noqa: N806
    lists = t.snake(SIZES)
    cpus = [2, 3, 4, 5]
    meta = {
        "kind": "meta", "corpus": "synthetic", "blocks": len(SIZES), "rows": sum(SIZES),
        "full_blocks": sum(1 for n in SIZES if n >= t.FULL), "rounds": rounds, "dry_run": dry,
        "inject": None, "cpus": cpus, "regimes": ["warm", "cold"], "w4_lists": lists,
        "w4_rows": [sum(SIZES[b] for b in l) for l in lists], "block_rows": SIZES,
        "block_raw_groups": [7] * len(SIZES), "memory_estimate": 10 << 30,
        "corpus_sha256": "c" * 64, "binary_sha256": "b" * 64, "machine": dict(MACHINE),
    }  # fmt: skip
    est = 10 << 30
    peak = 0 if dry else est + (1 << 28)  # a capture's verified dry-run peak, above the estimate
    req = max(est, peak)
    pre = {
        "kind": "preflight", "estimate": est, "required": req, "margin": max(2 << 30, req // 4),
        "verified_peak": peak, "dry_run_evidence": None, "readable": True,
        "mem_available": req + max(2 << 30, req // 4) + 1, "swap_used": 0, "psi_status": "read",
        "psi_present": True, "psi_some_avg10": 0.0, "psi_full_avg10": 0.0, "pswpin": 0,
        "pswpout": 0, "injected": False, "passed": True,
    }  # fmt: skip
    post = {
        "kind": "postflight", "estimate": est, "required": req, "vm_hwm": est // 2,
        "readable": True, "swap_used": 0, "psi_status": "read", "psi_present": True,
        "psi_some_avg10": 0.0, "psi_full_avg10": 0.0, "pswpin_delta": 0, "pswpout_delta": 0,
    }  # fmt: skip
    onepass = [{"kind": "onepass", "block": b, "rows": n, "histogram_equal": True,
                "below_2_24": True} for b, n in enumerate(SIZES)]  # fmt: skip
    counters = [{"kind": "counters", "block": b, "rows": n, "rows_complete": n // 2,
                 "rows_fragment": n // 10, "rows_scalar": n - n // 2 - n // 10,
                 "wide_promotions": b} for b, n in enumerate(SIZES)]  # fmt: skip
    need = lambda n: (n + 29) // 30 + 1  # noqa: E731
    check0 = [{"kind": "check0", "arm": "C", "block": b, "poisoned_capacity": 0 if b == 0 else 30001,
               "needed": need(n), "poison_survived": b != 0, "exact": True,
               "walk": True, "restored": True} for b, n in enumerate(SIZES)] + [
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
                        sk = skew
                        if skew_step is not None:
                            sk = skew_step[1] if step == skew_step[0] else 0.0
                        durs[k] = (d, start + int(sk * n * k / 3))
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
    recs["poisoncheck"] = [{"kind": "poisoncheck", "block": b, "rows": n, "poisoned_capacity": 30001,
                            "needed": need(n), "poison_survived": True, "exact": True}
                           for b, n in enumerate(SIZES)]  # fmt: skip
    recs["calibration"] = calib
    recs["build"] = builds
    if not dry:
        recs["_evidence"] = make_evidence(recs, peak)
    return recs


def make_evidence(recs, peak):
    """A completed, passing dry run's evidence directory's three files, consistent with
    the capture `recs`, and the capture's preflight record pointing at them"""
    meta = dict(recs["meta"][0], dry_run=True, rounds=2)
    jl = "\n".join(json.dumps(r) for r in [
        meta, {"kind": "postflight", "vm_hwm": peak}]).encode() + b"\n"  # fmt: skip
    ana = json.dumps({
        "dry_run": True, "schedule": {"S0_no_failure": True, "S1_meta": True},
        "gates": {k: True for k in t.DRY_GATES}, "memory": {"peak_rss_vm_hwm": peak},
    }).encode()  # fmt: skip
    sha = lambda b: hashlib.sha256(b).hexdigest()  # noqa: E731
    man = json.dumps({
        "exit_status": 0, "source": {"commit": "d" * 40, "tracked_changes": []},
        "binary": {"sha256": meta["binary_sha256"]},
        "corpora": [{"sha256": meta["corpus_sha256"]}],
        "files": [{"path": "c30-construction.jsonl", "sha256": sha(jl)},
                  {"path": "c30-analysis.json", "sha256": sha(ana)}],
    }).encode()  # fmt: skip
    recs["preflight"][0]["dry_run_evidence"] = {
        "dir": "dry", "peak_rss": peak, "commit": "d" * 40, "jsonl_sha256": sha(jl),
        "analysis_sha256": sha(ana), "manifest_sha256": sha(man),
    }  # fmt: skip
    return {"c30-construction.jsonl": (jl, sha(jl)), "c30-analysis.json": (ana, sha(ana)),
            "MANIFEST.json": (man, sha(man))}  # fmt: skip


def reseal(recs):
    """After editing the evidence files' contents in place, recompute the manifest's and
    the preflight's checksums, as a consistent forger would"""
    ev = recs["_evidence"]
    sha = lambda b: hashlib.sha256(b).hexdigest()  # noqa: E731
    jl, ana = ev["c30-construction.jsonl"][0], ev["c30-analysis.json"][0]
    man = json.loads(ev["MANIFEST.json"][0])
    man["files"] = [{"path": "c30-construction.jsonl", "sha256": sha(jl)},
                    {"path": "c30-analysis.json", "sha256": sha(ana)}]  # fmt: skip
    man = json.dumps(man).encode()
    ev["c30-construction.jsonl"] = (jl, sha(jl))
    ev["c30-analysis.json"] = (ana, sha(ana))
    ev["MANIFEST.json"] = (man, sha(man))
    recs["preflight"][0]["dry_run_evidence"].update(
        jsonl_sha256=sha(jl), analysis_sha256=sha(ana), manifest_sha256=sha(man)
    )


def run(recs, identity=True, other=(), rss=(), evidence="auto"):
    ev = recs.get("_evidence") if evidence == "auto" else evidence
    r = {k: v for k, v in recs.items() if k != "_evidence"}
    return t.analyse(r, list(other), identity, list(rss), ev)


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


# the repair (report 197): G2 per step


BIG = [FULL] * 228 + [300, 40]  # 57 all-full W4 steps and one partial


def test_big_fixture_passes_without_the_outlier():
    r = run(make(c_cost=1.0, noise=0.002, sizes=BIG))
    assert r["decision"] == "PASS" and r["start_skew"]["steps_checked"] == 57 * 2 * 2 * 20


def test_single_staggered_step_invalid():
    # one all-full step at 33.3% skew (worker 3 starts half a build late) among 56 clean
    # ones: the row-weighted aggregate stays under 2%, the step does not
    r = run(make(c_cost=1.0, noise=0.002, sizes=BIG, skew_step=(10, 0.5)))
    sk = r["start_skew"]
    assert invalid(r) and r["gates"]["G2_simultaneous_start"] is False, r["gates"]
    assert abs(sk["per_step_max"] - 1 / 3) < 0.001
    assert sk["row_weighted_aggregate_max"] < t.START_SKEW
    assert all(v is True for k, v in r["gates"].items() if k != "G2_simultaneous_start")


def test_single_staggered_step_invalid_at_a_fail_base():
    r = run(make(c_cost=1.2, noise=0.002, sizes=BIG, skew_step=(30, 0.5)))
    assert invalid(r) and r["gates"]["G2_simultaneous_start"] is False


def test_step_just_under_the_threshold_passes():
    # frac 0.02 gives a per-step skew of 0.0196
    r = run(make(c_cost=1.0, noise=0.002, sizes=BIG, skew_step=(10, 0.02)))
    assert r["decision"] == "PASS" and r["start_skew"]["per_step_max"] < t.START_SKEW


# the repair: G9's verified dry-run evidence


def _forged(change, reseal_after=True, base=None, evidence_flag=True):
    for rc in base() if base else _bases():
        change(rc)
        if reseal_after:
            reseal(rc)
        r = run(rc)
        assert invalid(r) and r["gates"]["G9_memory"] is False, (
            r["gates"],
            r["memory"]["problems"],
        )
        if evidence_flag:
            assert r["memory"]["dry_run_evidence_ok"] is False


def _edit_jsonl(rc, fn):
    ev = rc["_evidence"]
    lines = [json.loads(x) for x in ev["c30-construction.jsonl"][0].decode().splitlines()]
    for x in lines:
        fn(x)
    ev["c30-construction.jsonl"] = ("\n".join(json.dumps(x) for x in lines).encode() + b"\n", "")


def _edit_json(rc, f, fn):
    ev = rc["_evidence"]
    d = json.loads(ev[f][0])
    fn(d)
    ev[f] = (json.dumps(d).encode(), "")


def test_capture_without_evidence_files_invalid():
    for rc in _bases():
        r = run(rc, evidence=None)
        assert invalid(r) and r["gates"]["G9_memory"] is False


def test_capture_without_recorded_evidence_invalid():
    _forged(lambda rc: rc["preflight"][0].update(dry_run_evidence=None), reseal_after=False)


def test_capture_with_a_zero_peak_invalid():
    def f(rc):
        p = rc["preflight"][0]
        p.update(verified_peak=0, required=p["estimate"])

    _forged(f, reseal_after=False)


def test_fabricated_tiny_peak_invalid():
    # a consistent forgery: the peak set to 1 byte in every file, checksums recomputed
    def f(rc):
        _edit_jsonl(rc, lambda x: x.update(vm_hwm=1) if x["kind"] == "postflight" else None)
        _edit_json(rc, "c30-analysis.json", lambda d: d["memory"].update(peak_rss_vm_hwm=1))
        p = rc["preflight"][0]
        p.update(verified_peak=1, required=p["estimate"])
        p["margin"] = max(2 << 30, p["required"] // 4)
        p["dry_run_evidence"]["peak_rss"] = 1

    _forged(f)


def test_required_not_max_of_estimate_and_peak_invalid():
    _forged(lambda rc: rc["preflight"][0].update(required=rc["preflight"][0]["estimate"]),
            reseal_after=False, evidence_flag=False)  # fmt: skip


def test_peak_disagreeing_with_the_records_invalid():
    _forged(lambda rc: _edit_json(rc, "c30-analysis.json",
                                  lambda d: d["memory"].update(peak_rss_vm_hwm=12345678901)))  # fmt: skip


def test_stale_binary_invalid():
    def f(rc):
        _edit_jsonl(rc, lambda x: x.update(binary_sha256="e" * 64) if x["kind"] == "meta" else None)
        _edit_json(rc, "MANIFEST.json", lambda d: d["binary"].update(sha256="e" * 64))

    _forged(f)


def test_other_corpus_invalid():
    def f(rc):
        _edit_jsonl(rc, lambda x: x.update(corpus_sha256="f" * 64) if x["kind"] == "meta" else None)
        _edit_json(rc, "MANIFEST.json", lambda d: d["corpora"][0].update(sha256="f" * 64))

    _forged(f)


def test_other_machine_invalid():
    _forged(lambda rc: _edit_jsonl(
        rc, lambda x: x["machine"].update(hostname="i9") if x["kind"] == "meta" else None))  # fmt: skip


def test_other_cpus_invalid():
    _forged(lambda rc: _edit_jsonl(
        rc, lambda x: x.update(cpus=[0, 1, 2, 3]) if x["kind"] == "meta" else None))  # fmt: skip


def test_injected_dry_run_invalid():
    _forged(lambda rc: _edit_jsonl(
        rc, lambda x: x.update(inject="pin:w4:1") if x["kind"] == "meta" else None))  # fmt: skip


def test_dry_run_with_a_failed_gate_invalid():
    _forged(lambda rc: _edit_json(rc, "c30-analysis.json",
                                  lambda d: d["gates"].update(G7_round_minus_one=False)))  # fmt: skip


def test_dry_run_with_a_failed_schedule_invalid():
    _forged(lambda rc: _edit_json(rc, "c30-analysis.json",
                                  lambda d: d["schedule"].update(S3_record_set=False)))  # fmt: skip


def test_dry_run_with_a_failure_record_invalid():
    def f(rc):
        ev = rc["_evidence"]
        jl = ev["c30-construction.jsonl"][0] + json.dumps({"kind": "failure"}).encode() + b"\n"
        ev["c30-construction.jsonl"] = (jl, "")

    _forged(f)


def test_dry_run_exit_nonzero_invalid():
    _forged(lambda rc: _edit_json(rc, "MANIFEST.json", lambda d: d.update(exit_status=3)))


def test_dry_run_with_tracked_changes_invalid():
    _forged(lambda rc: _edit_json(
        rc, "MANIFEST.json", lambda d: d["source"].update(tracked_changes=["M src/bwt.rs"])))  # fmt: skip


def test_evidence_edited_after_verification_invalid():
    # the records changed without the checksums the collector recorded
    _forged(lambda rc: _edit_jsonl(
        rc, lambda x: x.update(vm_hwm=x["vm_hwm"] + 1) if x["kind"] == "postflight" else None),
        reseal_after=False)  # fmt: skip


def test_dry_run_carrying_evidence_invalid():
    rc = make(rounds=2, dry=True)
    rc["preflight"][0]["dry_run_evidence"] = {"dir": "x"}
    r = run(rc)
    assert r["gates"]["G9_memory"] is False


def test_unreadable_memory_sources_invalid():
    for change, where in (
        (lambda p: p.update(psi_status="unreadable", psi_some_avg10=None), "preflight"),
        (lambda p: p.update(psi_status="unreadable"), "postflight"),
        (lambda p: p.update(readable=False), "preflight"),
        (lambda p: p.update(readable=False), "postflight"),
        (lambda p: p.update(pswpout_delta=None), "postflight"),
        (lambda p: p.update(swap_used=None), "preflight"),
    ):
        _memory(change, where)


def test_absent_psi_is_accepted_and_recorded():
    for rc in _bases()[:1]:
        for w in ("preflight", "postflight"):
            rc[w][0].update(psi_status="absent", psi_present=False, psi_some_avg10=None,
                            psi_full_avg10=None)  # fmt: skip
        assert run(rc)["decision"] == "PASS"


# the repair: G7's recorded poison coverage


def test_poison_claimed_without_capacity_invalid():
    for rc in _bases():
        c = next(o for o in rc["check0"] if o["arm"] == "C" and o["block"] == 0)
        c["poison_survived"] = True  # with poisoned_capacity 0
        r = run(rc)
        assert invalid(r) and r["gates"]["G7_round_minus_one"] is False


def test_old_unqualified_poison_flag_invalid():
    for rc in _bases():
        for o in rc["check0"]:
            if o["arm"] == "C":
                for k in ("poisoned_capacity", "needed", "poison_survived"):
                    del o[k]
                o["poisoned"] = True
        r = run(rc)
        assert invalid(r) and r["gates"]["G7_round_minus_one"] is False


def test_poison_pass_not_surviving_invalid():
    for rc in _bases():
        rc["poisoncheck"][4]["poison_survived"] = False
        r = run(rc)
        assert invalid(r) and r["gates"]["G7_round_minus_one"] is False


def test_poison_pass_missing_rejected():
    for rc in _bases():
        rc["poisoncheck"] = rc["poisoncheck"][1:]
        assert before_figures(run(rc))


def test_poison_coverage_reported():
    r = run(base_pass())
    pc = r["poison_coverage"]
    assert pc["round_minus_one_blocks_poison_survived"] == len(SIZES) - 1
    assert pc["poison_pass_blocks_survived"] == len(SIZES)


# the repair: the peak-RSS records


def rss_records(meta):
    out = []
    for k in range(3):
        for a in ("P", "C"):
            out.append({"kind": "rss", "arm": a, "corpus": meta["corpus"],
                        "corpus_sha256": meta["corpus_sha256"], "blocks": meta["blocks"],
                        "cpus": meta["cpus"], "pid": 100 + 2 * k + (a == "C"),
                        "started_unix_ns": 10**18 + k, "baseline_rss": 1 << 20,
                        "hwm_after_load": 1 << 21, "hwm_reset": True, "hwm_after_reset": 1 << 20,
                        "hwm_after_pass": 1 << 22, "growth_over_baseline": 3 << 20,
                        "arena_capacity_bytes": 1 << 21})  # fmt: skip
    return out


def test_rss_records_summarised_when_valid():
    rc = base_pass()
    r = run(rc, rss=rss_records(rc["meta"][0]))
    assert r["peak_rss_processes"]["P"]["n"] == 3 and r["decision"] == "PASS"


def test_rss_records_invalid_are_unavailable_and_decide_nothing():
    for change in (
        lambda v: v.pop(),
        lambda v: v[1].update(pid=v[0]["pid"], started_unix_ns=v[0]["started_unix_ns"]),
        lambda v: v[2].update(hwm_reset=False),
        lambda v: v[3].update(corpus="other.bz2"),
        lambda v: v[4].update(cpus=[0, 1, 2, 3]),
        lambda v: v[5].update(growth_over_baseline=-1),
        lambda v: v[0].update(arm="C"),
        lambda v: v[1].update(hwm_after_pass=None),
    ):
        rc = base_pass()
        v = rss_records(rc["meta"][0])
        change(v)
        r = run(rc, rss=v)
        assert r["peak_rss_processes"].get("unavailable") is True, change
        assert r["decision"] == "PASS"


def test_no_rss_records_are_unavailable():
    assert run(base_pass())["peak_rss_processes"]["unavailable"] is True


# the repair: A7, the mixed step reported apart


def test_mixed_step_reported_separately():
    r = run(make(c_cost=1.0, noise=0.002, sizes=[FULL] * 9 + [300, 40, 20]))
    econ = r["block_economics"]
    assert econ["mixed_steps"]["steps"] and "e4" in econ["mixed_steps"]
    assert r["decision"] == "PASS"


if __name__ == "__main__":
    n = 0
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print("ok", name)
            n += 1
    print(f"{n} tests passed")
