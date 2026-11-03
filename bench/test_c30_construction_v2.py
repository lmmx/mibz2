"""bench/c30_construction.py's constructor v2 mode (`analyse_v2`, protocol v2, report 211), on
synthetic records (no timing). Run: python3 bench/test_c30_construction_v2.py

Two parts:
1. **Draft 4's suite under v2.** Every test of `test_c30_construction.py` runs again with its
   fixtures carrying v2's identifier and analysed by `analyse_v2`, except the five that test
   draft 4's G2 itself (listed in `DRAFT4_G2_ONLY`), which v2's G2 tests below replace. So every
   schedule defect, wrong denominator, missing calibration, correctness failure, memory or
   evidence violation, major fault and wrong round count stays INVALID under v2, never PASS,
   FAIL or INCONCLUSIVE, and the decision table is unchanged.
2. **v2's registered differences:** G1 (P's warm W4 cost and completion only), G2 (each arm's
   warm W4 all-full steps: aggregate ≤ 2%, at most ⌊N/1000⌋ steps over 2%, none over 10%), and
   the protocol identifier on the records, the dry-run evidence and the RSS records.

The base fixture for G2 is `BIG`: 57 all-full W4 steps a pass, so N = 57 × 20 = 1,140 steps
per arm in the warm regime, and an allowance of ⌊1,140 / 1000⌋ = 1 outlier per arm, as on the
primary corpus (230 full blocks)."""

import json
import os
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(__file__))
import test_c30_construction as base  # noqa: E402

t = base.t
V2 = t.PROTOCOL_V2
BIG = base.BIG
FULL = base.FULL
_make, _rss = base.make, base.rss_records

# draft 4's G2 tests: v2 registers a different G2 (tested in part 2)
DRAFT4_G2_ONLY = {
    "test_staggered_start_invalid",
    "test_big_fixture_passes_without_the_outlier",
    "test_single_staggered_step_invalid",
    "test_single_staggered_step_invalid_at_a_fail_base",
    "test_step_just_under_the_threshold_passes",
}


def to_v2(recs):
    """Draft 4's synthetic records as v2's collector writes them: the identifier in the meta
    record, and a capture's dry-run evidence a v2 dry run's (records and analysis), resealed"""
    recs["meta"][0].update(protocol=V2, candidate=t.CANDIDATE_V2, candidate_scratch_bytes=16384)
    for o in recs["counters"]:
        o["fin_wide"] = o.pop("wide_promotions")
        o.update(fin_ordinary=1, fin_crossing=2, norm_fields=o["rows"] // 3)
    if "_evidence" in recs:
        ev = recs["_evidence"]
        lines = [json.loads(x) for x in ev["c30-construction.jsonl"][0].splitlines() if x.strip()]
        for r in lines:
            if r.get("kind") == "meta":
                r.update(protocol=V2, candidate=t.CANDIDATE_V2)
        jl = "\n".join(json.dumps(r) for r in lines).encode() + b"\n"
        ana = json.loads(ev["c30-analysis.json"][0])
        ana["protocol"] = V2
        ev["c30-construction.jsonl"] = (jl, None)
        ev["c30-analysis.json"] = (json.dumps(ana).encode(), None)
        base.reseal(recs)
    return recs


def make(**kw):
    return to_v2(_make(**kw))


def rss_records(meta):
    return [dict(r, protocol=V2) for r in _rss(meta)]


def run(recs, identity=True, other=(), rss=(), evidence="auto"):
    ev = recs.get("_evidence") if evidence == "auto" else evidence
    r = {k: v for k, v in recs.items() if k != "_evidence"}
    return t.analyse_v2(r, list(other), identity, list(rss), ev)


invalid = base.invalid


def builds(recs, **kw):
    return [w for w in recs["build"] if all(w[k] == v for k, v in kw.items())]


def stagger(recs, rnd, regime, arm, step, skew):
    """Make one W4 step's start skew `skew` exactly: its worker 3 starts `skew` × the step's
    span later and ends where it did (its CPU window shrinks with it), so the span and every
    barrier ordering are unchanged"""
    v = builds(recs, round=rnd, regime=regime, mode="w4", arm=arm, step=step)
    assert len(v) == 4 and all(w["rows"] >= t.FULL for w in v), (rnd, regime, arm, step)
    s0 = min(w["start"] for w in v)
    span = max(w["end"] for w in v) - s0
    w = next(w for w in v if w["worker"] == 3)
    w["start"] = s0 + round(skew * span)
    assert w["start"] < w["end"]
    w["cpu_start"] = w["cpu_end"] - (w["end"] - w["start"])
    return (w["start"] - s0) / span


def scale_cpu(recs, f, **kw):
    """Shorten the CPU windows of the matching builds by the factor f ≤ 1 (their monotonic
    windows unchanged, so every schedule check still holds)"""
    for w in builds(recs, **kw):
        w["cpu_end"] = w["cpu_start"] + int((w["cpu_end"] - w["cpu_start"]) * f)


def big(**kw):
    return make(c_cost=1.0, noise=0.002, sizes=BIG, **kw)


def g2(r, arm="P"):
    return r["start_skew"]["warm_gated"][arm]


def only_failed(r):
    return sorted(k for k, v in r["gates"].items() if v is not True)


# ---------------------------------------------------------------------------------------
# the base: PASS under v2, its G2 population as registered
# ---------------------------------------------------------------------------------------


def test_v2_base_passes_with_the_registered_population():
    r = run(big())
    assert r["valid"] is True and r["decision"] == "PASS", only_failed(r)
    assert r["protocol"] == V2 and r["schedule"]["S1_protocol"] is True
    for a in ("P", "C"):
        assert g2(r, a)["steps"] == 57 * 20 and g2(r, a)["outlier_allowance"] == 1
        assert g2(r, a)["outliers_over_2pct"] == 0
    assert set(k for k in r["gates"] if k.startswith(("G1_", "G2_"))) == {
        "G1_warm_w4_cpu_ns_row", "G1_warm_w4_completion_ns", "G2_aggregate",
        "G2_outlier_allowance", "G2_max_step"}  # fmt: skip


# ---------------------------------------------------------------------------------------
# G2: isolated interruptions, widespread staggering, the 33.3% step, the boundaries
# ---------------------------------------------------------------------------------------


def test_isolated_moderate_interruptions_stay_valid():
    # one warm step per arm at 5% and 8% (each within its arm's allowance of one), a 3% one
    # in the other arm's cold pass and a 6% one in cold P: cold is reported, not gated
    rc = big()
    stagger(rc, 3, "warm", "P", 10, 0.05)
    stagger(rc, 7, "warm", "C", 20, 0.08)
    stagger(rc, 5, "cold", "P", 5, 0.06)
    stagger(rc, 9, "cold", "C", 30, 0.03)
    stagger(rc, 9, "cold", "C", 31, 0.07)
    r = run(rc)
    assert r["valid"] is True and r["decision"] == "PASS", only_failed(r)
    assert [(o["round"], o["step"], o["arm"]) for o in g2(r, "P")["outliers"]] == [(3, 10, "P")]
    assert [(o["round"], o["step"], o["arm"]) for o in g2(r, "C")["outliers"]] == [(7, 20, "C")]
    assert abs(g2(r, "C")["max_step_skew"] - 0.08) < 1e-5
    assert r["start_skew"]["cold_reported"]["C"]["outliers_over_2pct"] == 2
    assert r["start_skew"]["cold_reported"]["P"]["max_step_skew"] > 0.059


def test_a_three_percent_step_stays_valid():
    rc = big()
    k = stagger(rc, 12, "warm", "P", 40, 0.03)
    r = run(rc)
    assert r["decision"] == "PASS" and abs(g2(r)["max_step_skew"] - k) < 1e-12


def test_widespread_staggered_starts_invalid():
    # every step of every pass at 9.1% (worker k starts 0.1·k/3 of a build late)
    r = run(make(c_cost=1.0, noise=0.002, sizes=BIG, skew=0.1))
    assert invalid(r) and r["gates"]["G2_aggregate"] is False
    assert r["gates"]["G2_outlier_allowance"] is False
    assert g2(r)["row_weighted_aggregate"] > t.G2_AGGREGATE


def test_widespread_staggering_at_a_fail_base_invalid():
    r = run(make(c_cost=1.2, noise=0.002, sizes=BIG, skew=0.1))
    assert invalid(r) and "decision" in r and r["decision"].startswith("INVALID")


def test_outliers_above_the_allowance_invalid():
    # three warm P steps at 3%: the aggregate stays far under 2%, the allowance is one
    rc = big()
    for rnd, s in ((2, 4), (8, 9), (15, 33)):
        stagger(rc, rnd, "warm", "P", s, 0.03)
    r = run(rc)
    assert invalid(r) and only_failed(r) == ["G2_outlier_allowance"], only_failed(r)
    assert g2(r)["outliers_over_2pct"] == 3 and g2(r)["row_weighted_aggregate"] < 0.001
    assert [(o["round"], o["step"]) for o in g2(r)["outliers"]] == [(2, 4), (8, 9), (15, 33)]


def test_the_allowance_at_its_limit_and_one_over():
    rc = big()
    stagger(rc, 4, "warm", "P", 7, 0.05)
    r = run(rc)
    assert r["valid"] is True and g2(r)["outliers_over_2pct"] == g2(r)["outlier_allowance"] == 1
    stagger(rc, 11, "warm", "P", 50, 0.05)
    r = run(rc)
    assert invalid(r) and only_failed(r) == ["G2_outlier_allowance"]
    assert g2(r)["outliers_over_2pct"] == 2


def test_the_allowance_is_per_arm():
    rc = big()
    stagger(rc, 4, "warm", "P", 7, 0.05)
    stagger(rc, 4, "warm", "C", 7, 0.05)
    r = run(rc)
    assert r["valid"] is True and g2(r, "P")["outliers_over_2pct"] == 1
    assert g2(r, "C")["outliers_over_2pct"] == 1


def test_a_step_at_exactly_two_percent_is_not_an_outlier():
    rc = big()
    for rnd in range(3):
        stagger(rc, rnd, "warm", "P", 3, 0.02)
    r = run(rc)
    assert r["valid"] is True and g2(r)["outliers_over_2pct"] == 0
    assert g2(r)["max_step_skew"] == 0.02


def test_single_33_percent_step_invalid():
    # the case report 211 names: within the allowance of one, but over the 10% maximum
    rc = big()
    k = stagger(rc, 6, "warm", "P", 10, 1 / 3)
    r = run(rc)
    assert abs(k - 1 / 3) < 1e-6
    assert invalid(r) and only_failed(r) == ["G2_max_step"], only_failed(r)
    assert g2(r)["outliers_over_2pct"] == 1 and g2(r)["row_weighted_aggregate"] < t.G2_AGGREGATE
    assert g2(r)["outliers"] == [{"round": 6, "step": 10, "skew": k, "arm": "P"}]


def test_single_33_percent_step_invalid_in_the_candidate_and_at_a_fail_base():
    rc = make(c_cost=1.2, noise=0.002, sizes=BIG)
    stagger(rc, 13, "warm", "C", 30, 1 / 3)
    r = run(rc)
    assert invalid(r) and only_failed(r) == ["G2_max_step"]


def test_one_step_just_above_ten_percent_invalid():
    rc = big()
    k = stagger(rc, 1, "warm", "P", 22, 0.1001)
    r = run(rc)
    assert k > t.G2_MAX and invalid(r) and only_failed(r) == ["G2_max_step"]


def test_one_step_at_exactly_ten_percent_valid():
    rc = big()
    k = stagger(rc, 1, "warm", "P", 22, 0.10)
    r = run(rc)
    assert k == 0.10 and r["valid"] is True and g2(r)["steps_over_10pct"] == 0


def test_the_aggregate_binds_on_its_own():
    # every warm P step just under 2% (none an outlier) and one at exactly 10% (within the
    # maximum and the allowance): the row-weighted aggregate alone exceeds 2%
    rc = big()
    for rnd in range(20):
        for s in range(57):
            if (rnd, s) != (0, 0):
                stagger(rc, rnd, "warm", "P", s, 0.019993)
    stagger(rc, 0, "warm", "P", 0, 0.10)
    r = run(rc)
    assert g2(r)["outliers_over_2pct"] == 1 and g2(r)["max_step_skew"] == 0.10
    assert g2(r)["row_weighted_aggregate"] > t.G2_AGGREGATE
    assert invalid(r) and only_failed(r) == ["G2_aggregate"], only_failed(r)


def test_the_aggregate_is_the_row_weighted_mean_of_draft_4s_per_step_skew():
    rc = big()
    ks = [stagger(rc, rnd, "warm", "C", 5, 0.004 * (rnd + 1)) for rnd in range(5)]
    r = run(rc)
    n = 57 * 20
    assert abs(g2(r, "C")["row_weighted_aggregate"] - sum(ks) / n) < 1e-12
    # equal rows in every step, so it is also the mean of the per-round aggregates
    per = [r["rounds"][x]["warm"]["C"]["w4_start_skew"] for x in range(20)]
    assert abs(g2(r, "C")["row_weighted_aggregate"] - sum(per) / 20) < 1e-12


def test_cold_staggering_alone_is_reported_not_gated():
    rc = make(c_cost=1.0, noise=0.002, sizes=BIG)
    for rnd in range(20):
        stagger(rc, rnd, "cold", "P", 10, 1 / 3)
        stagger(rc, rnd, "cold", "C", 11, 0.25)
    r = run(rc)
    assert r["valid"] is True and r["decision"] == "PASS"
    assert r["start_skew"]["cold_reported"]["P"]["steps_over_10pct"] == 20


# ---------------------------------------------------------------------------------------
# G1: P's warm W4 gates; W1 and cold are reported
# ---------------------------------------------------------------------------------------


def test_w1_instability_alone_is_reported_not_invalid():
    # P's warm and cold W1 CPU cost bimodal across rounds (max/min 1.25), as v1's capture
    rc = big()
    for rnd in (6, 8, 12, 14, 16):
        scale_cpu(rc, 0.8, round=rnd, mode="w1", arm="P")
    r = run(rc)
    assert r["valid"] is True and r["decision"] == "PASS", only_failed(r)
    st = r["stability"]
    assert abs(st["warm_P_w1_cpu_ns_row"]["max_over_min"] - 1.25) < 0.01
    assert st["warm_P_w1_cpu_ns_row"]["gates"] is False
    assert abs(st["cold_P_w1_cpu_ns_row"]["max_over_min"] - 1.25) < 0.01


def test_cold_instability_alone_is_reported_not_invalid():
    rc = big()
    for rnd in (3, 9):
        scale_cpu(rc, 0.85, round=rnd, regime="cold", mode="w4", arm="P")
    r = run(rc)
    assert r["valid"] is True and r["decision"] == "PASS", only_failed(r)
    assert r["stability"]["cold_P_w4_cpu_ns_row"]["max_over_min"] > 1.15


def test_candidate_instability_is_reported_not_gated():
    rc = big()
    scale_cpu(rc, 0.9, round=5, regime="warm", mode="w4", arm="C")
    r = run(rc)
    assert r["valid"] is True and r["stability"]["warm_C_w4_cpu_ns_row"]["max_over_min"] > 1.1


def test_production_warm_w4_cost_instability_invalid():
    rc = big()
    scale_cpu(rc, 0.9, round=4, regime="warm", mode="w4", arm="P")
    r = run(rc)
    assert invalid(r) and only_failed(r) == ["G1_warm_w4_cpu_ns_row"], only_failed(r)
    assert r["stability"]["warm_P_w4_cpu_ns_row"]["min_round"] == 4


def test_production_warm_w4_cost_just_inside_the_limit_valid():
    rc = big()
    scale_cpu(rc, 0.955, round=4, regime="warm", mode="w4", arm="P")
    r = run(rc)
    assert r["valid"] is True
    assert 1.04 < r["stability"]["warm_P_w4_cpu_ns_row"]["max_over_min"] <= t.STABILITY


def test_production_warm_w4_completion_instability_invalid():
    # P's worker 0 takes 1.1× the step mean in every all-full step (completion 1.1× the
    # balanced span) except in round 6, where it takes the others' share: that round's
    # completion is 12% shorter while its CPU cost moves 3%
    rc = make(c_cost=1.0, noise=0.002, sizes=BIG, p_imbalance=1.1)
    for w in builds(rc, round=6, regime="warm", mode="w4", arm="P", worker=0):
        if w["rows"] >= t.FULL:
            d = int(w["rows"] * (4 - 1.1) / 3)
            w["end"] = w["start"] + d
            w["cpu_end"] = w["cpu_start"] + d
    r = run(rc)
    assert invalid(r) and only_failed(r) == ["G1_warm_w4_completion_ns"], only_failed(r)
    assert r["stability"]["warm_P_w4_cpu_ns_row"]["max_over_min"] < t.STABILITY


# ---------------------------------------------------------------------------------------
# the protocol identifier
# ---------------------------------------------------------------------------------------


def test_v1_records_are_refused_by_v2():
    r = run(_make(c_cost=1.0, noise=0.002))
    assert invalid(r) and r["schedule"] == {"S1_protocol": False} and "ratios" not in r


def test_another_candidate_is_refused():
    rc = big()
    rc["meta"][0]["candidate"] = "ctor"
    r = run(rc)
    assert invalid(r) and r["schedule"]["S1_protocol"] is False and "ratios" not in r


def test_v2_records_are_refused_by_draft_4s_mode():
    rc = make(c_cost=1.0, noise=0.002, rounds=2, dry=True)
    with tempfile.TemporaryDirectory() as d:
        p = os.path.join(d, "run.jsonl")
        with open(p, "w") as f:
            for k, v in rc.items():
                if k != "_evidence":
                    for x in v:
                        f.write(json.dumps(x) + "\n")
        here = os.path.join(os.path.dirname(os.path.abspath(__file__)), "c30_construction.py")
        v1 = subprocess.run([sys.executable, "-I", here, p], capture_output=True, text=True)
        v2 = subprocess.run([sys.executable, "-I", here, p, "--protocol", "v2"],
                            capture_output=True, text=True)  # fmt: skip
    assert v1.returncode == 2 and "--protocol v2" in v1.stdout, v1.stdout
    assert v2.returncode == 0 and "protocol: c30-construction-v2" in v2.stdout, v2.stdout


def _evidence_edit(fn_meta=None, fn_ana=None):
    rc = big()
    ev = rc["_evidence"]
    if fn_meta:
        lines = [json.loads(x) for x in ev["c30-construction.jsonl"][0].splitlines() if x.strip()]
        for x in lines:
            if x.get("kind") == "meta":
                fn_meta(x)
        ev["c30-construction.jsonl"] = (
            "\n".join(json.dumps(x) for x in lines).encode() + b"\n",
            None,
        )
    if fn_ana:
        a = json.loads(ev["c30-analysis.json"][0])
        fn_ana(a)
        ev["c30-analysis.json"] = (json.dumps(a).encode(), None)
    base.reseal(rc)
    return run(rc)


def test_a_v1_dry_runs_evidence_is_refused():
    for r in (
        _evidence_edit(fn_ana=lambda a: a.pop("protocol")),
        _evidence_edit(fn_meta=lambda m: (m.pop("protocol"), m.pop("candidate"))),
        _evidence_edit(fn_meta=lambda m: m.update(candidate="ctor")),
    ):
        assert invalid(r) and only_failed(r) == ["G9_memory"], only_failed(r)
        assert r["memory"]["dry_run_evidence_ok"] is False
        assert any("protocol v2" in p for p in r["memory"]["problems"])


def test_rss_records_without_the_identifier_are_unavailable():
    rc = big()
    good = rss_records(rc["meta"][0])
    assert run(rc, rss=good)["peak_rss_processes"]["P"]["n"] == 3
    bad = [dict(x) for x in good]
    bad[2].pop("protocol")
    r = run(rc, rss=bad)
    assert r["peak_rss_processes"]["unavailable"] is True and r["decision"] == "PASS"


def test_v2_dry_run_suppresses_g1_g2_and_records_the_protocol():
    r = run(make(rounds=2, dry=True))
    assert r["valid"] is None and r["protocol"] == V2 and "ratios" not in r
    assert not any(k.startswith(("G1_", "G2_")) for k in r["gates"])
    assert all(v is True for v in r["schedule"].values()) and r["schedule"]["S1_protocol"]


def test_v2_slow_path_table_reported():
    r = run(big())
    dec = r["slow_paths_v2_by_one_row_decile"]
    assert dec and all("wide_finishes_mean" in d and "warm_w1_ratio_median" in d for d in dec)
    assert r["decision"] == "PASS"


# ---------------------------------------------------------------------------------------
# part 1: draft 4's suite, under v2
# ---------------------------------------------------------------------------------------


def draft4_suite():
    names = sorted(k for k, v in vars(base).items() if k.startswith("test_") and callable(v))
    saved = (base.make, base.run, base.rss_records)
    base.make, base.run, base.rss_records = make, run, rss_records
    try:
        n = 0
        for name in names:
            if name in DRAFT4_G2_ONLY:
                print("replaced", name, "(draft 4's G2)")
                continue
            getattr(base, name)()
            print("ok under v2", name)
            n += 1
        return n
    finally:
        base.make, base.run, base.rss_records = saved


if __name__ == "__main__":
    n = draft4_suite()
    m = 0
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print("ok", name)
            m += 1
    print(f"{n} draft 4 tests passed under v2, {len(DRAFT4_G2_ONLY)} replaced; {m} v2 tests passed")
