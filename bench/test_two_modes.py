"""bench/two_modes.py on synthetic i9-shaped attempts.

- Each registered signature's synthetic data is called compatible with it and no other.
- Data no signature predicts is reported as no registered signature matching.
- Too few runs in a class, or missing decisive evidence, never yields a compatible signature.
- A broken or contradictory attempt is INVALID.
- The preflight refuses each defect in its collectors' exit statuses.

The Researcher's five counterexamples (REVISE, #145 6757560564) are tests here, each failing
on the source it reviewed (3588d35d) and passing now: the progress estimator, sleeping
siblings, quantised constant input, missing evidence, contradictory identities.

    python3 bench/test_two_modes.py
"""

import contextlib
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import two_modes as tm  # noqa: E402

HZ = 3.7e9  # the TSC's rate
ITERS = 1 << 20
SIBS = {c: {c % 10, c % 10 + 10} for c in range(20)}
NAMES = (
    ["mibz2-0", "mibz2-1", "mibz2-2", "mibz2-scan", "mibz2-read"]
    + [f"mibz2-coop-p{1 + i % 2}" for i in range(6)]
    + ["sampler", "ab"]
)
H = "0123456789abcdef" * 4  # a 64-hex digest
H40 = "0123456789" * 4


def static():
    cpus = {}
    for c in range(20):
        a, b = sorted(SIBS[c])
        cpus[str(c)] = {
            "siblings": f"{a},{b}",
            "core_id": str(a),
            "thermal_throttle": [
                "core_throttle_count",
                "core_throttle_total_time_ms",
                "package_throttle_count",
                "package_throttle_total_time_ms",
                "core_power_limit_count",
                "package_power_limit_count",
            ],
            "tsc_flags": {"constant_tsc": True, "nonstop_tsc": True, "ht": True},
        }
    return {
        "schema": tm.STATIC_SCHEMA,
        "model": "Intel(R) Core(TM) i9-10900K",
        "cpus": cpus,
        "files": {},
    }


def counters(core=0, pkg=0, pl=0, thp=0, power=True):
    tt = {}
    for c in range(20):
        d = {
            "core_throttle_count": core,
            "package_throttle_count": pkg,
            "core_throttle_total_time_ms": 0,
            "package_throttle_total_time_ms": 0,
        }
        if power:
            d["core_power_limit_count"] = pl
            d["package_power_limit_count"] = pl
        tt[str(c)] = d
    return {"thermal_throttle": tt, "vmstat": {"thp_fault_alloc": thp, "thp_fault_fallback": 0}}


def ring(first, until, rate_of, lo, hi):
    """Sample times from `first` until past `until`, each interval's rate (Giter/s) given by
    rate_of(fraction of [lo, hi])"""
    t, dt = first, []
    while t < until:
        frac = min(max((t - lo) / (hi - lo), 0), 1)
        d = int(ITERS * HZ / (rate_of(frac) * 1e9))
        dt.append(d)
        t += d
    return dt


def probe_record(i, wall, sig, fast, pass_no, t_spawn):
    """One probe.jsonl line for a run of `wall` seconds under signature `sig`"""
    power = not sig.endswith("+nopower")
    sig = sig.split("+")[0]
    span = wall + 0.004
    exec_tsc = 10_000_000_000 + i * 10_000_000_000
    exit_tsc = exec_tsc + int(span * HZ)
    slow = not fast

    def p_rate(frac):  # Giter/s at a fraction of the decode
        base = 4.8
        if slow and sig in ("H3", "sixth"):
            return base * 0.5
        if slow and sig == "H5":
            return base * 0.8
        if slow and sig == "H1shared":
            return base * (0.3 + 0.7 * frac)
        return base

    sched = tm.SCHEDULE[pass_no]
    probed = sched[i % len(sched)] == "P"
    probes = []
    want = [("P-idle", tm.IDLE_CPU)] + ([("P-sib", tm.SIB_CPU)] if pass_no == 2 else [])
    for name, cpu in want if probed else []:
        rate = p_rate if name == "P-idle" else (lambda f: 4.0)
        first = exec_tsc - 2_000_000
        dt = ring(
            first,
            exit_tsc + 2_000_000,
            rate if pass_no == 1 else (lambda f, r=rate: r(f) * 0.9),
            exec_tsc,
            exit_tsc,
        )
        probes.append(
            {
                "name": name,
                "cpu": cpu,
                "readback": str(cpu),
                "pinned": True,
                "err": 0,
                "iters_per_sample": ITERS,
                "n": len(dt) + 1,
                "dropped": 0,
                "cpu_ns": int(sum(dt) / HZ * 1e9),
                "last_tsc": first + sum(dt),
                "first_tsc": first,
                "dtsc": dt,
            }
        )
    # placement: 13 tasks, running; spread over distinct cores (four pairs share a core on
    # the nine), or packed onto siblings under H2's slow runs
    comm = {str(1000 + k): n for k, n in enumerate(NAMES)}
    packed = slow and sig == "H2"
    spread = [0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13]
    pack = [0, 10, 1, 11, 2, 12, 3, 13, 4, 14, 5, 15, 6]
    places = pack if packed else spread
    samples = []
    us = int(span * 1e6)
    for t in range(0, us, 2000):
        samples.append([t, [[1000 + k, places[k], "R"] for k in range(len(NAMES))]])
    freq = []
    for t in range(0, us, 10000):
        frac = t / us
        ghz = 4.9
        if slow and sig in ("H1", "H1shared"):
            ghz = 1.2 + 3.7 * frac
        if slow and sig == "H5":
            ghz = 3.9
        freq.append([t, 300, [int(ghz * 1e6)] * 20])
    rise = {"core": 1 if slow and sig == "H3" else 0, "pl": 1 if slow and sig == "H5" else 0}
    stime = 0.30 if slow and sig == "H4" else 0.05
    return {
        "schema": tm.PROBE_SCHEMA,
        "pass": pass_no,
        "run_index": i,
        "probed": probed,
        "schedule": sched,
        "arm": tm.ARM,
        "threads": 10,
        "t_unix_ns_start": t_spawn + 1_000_000,
        "before": counters(5, 5, 2, 100, power),
        "after": counters(5 + rise["core"], 5, 2 + rise["pl"], 100, power),
        "pid": 4242 + i,
        "exec_errno": 0,
        "decoder_cpus": tm.DECODER_CPUS,
        "decoder_cpus_readback": tm.DECODER_CPUS,
        "decoder_cpus_status": tm.DECODER_CPUS,
        "tsc": {
            "start": exec_tsc - 3_000_000,
            "fork": exec_tsc - 100,
            "exec": exec_tsc,
            "exit": exit_tsc,
            "end": exit_tsc + 100,
        },
        "mono_ns": {
            "start": 0,
            "exec": 1_000_000,
            "exit": 1_000_000 + int(span * 1e9),
            "end": 2_000_000 + int(span * 1e9),
        },
        "tsc_hz": HZ,
        "probes": probes,
        "placement": {"every_ms": 2, "comm": comm, "samples": samples},
        "freq": {"every_ms": 10, "cpus": 20, "samples": freq},
        "rusage": {
            "minflt": 20000,
            "majflt": 0,
            "utime_s": 1.0 if fast else 2.0,
            "stime_s": stime,
            "maxrss_kb": 76000,
            "nvcsw": 1,
            "nivcsw": 1,
        },
        "status": {"exit": 0},
        "smaps": {"reads": 30, "anon_huge_kb_max": 4096 if fast and sig == "H4" else 0},
    }


def run_record(i, wall, t_spawn):
    return {
        "round": i,
        "corpus": "wikidata",
        "threads": 10,
        "arm": f"{tm.ARM}@/w/bin/two_modes_probe",
        "termination": "success",
        "exit": 0,
        "t_spawn_unix_ns": t_spawn,
        "t_exit_unix_ns": t_spawn + int((wall + 0.01) * 1e9),
        "wall": wall,
        "cpu": wall * 7,
        "out": tm.WORKLOAD["out"],
        "hash": tm.WORKLOAD["hash"],
        "err": "",
    }


def pattern(n, fast_runs):
    # fast runs in streaks of three, as the bundle showed
    fast = set()
    i = 0
    while len(fast) < fast_runs and i < n:
        fast.update(range(i, min(i + 3, n)))
        i += 9
    return [k in fast for k in range(n)]


RATIO = tm.WORKLOAD["out"] / tm.CORPUS_BYTES


def sustained(rates, chunk=4 * 1024 * 1024, copies=tm.SUSTAINED_COPIES, phase=0.0):
    """A sustained block's record: its input read in chunks (three ahead), consumed at a
    decoded-equivalent rate (GB/s) that `rates(t)` gives at each time t, the first refill
    `phase` of a chunk along"""
    total = copies * tm.CORPUS_BYTES
    samples, read, due, t = [], 3 * chunk, phase * chunk, 0.0
    while read < total:
        due += rates(t) * 1e9 / RATIO * 0.01
        while due >= chunk and read < total:
            read += chunk
            due -= chunk
        samples.append([int(t * 1e6), min(read, total)])
        t += 0.01
    rec = {
        "schema": tm.SUSTAINED_SCHEMA,
        "span_s": t + 1.0,
        "io": {"every_ms": 10, "samples": samples},
        "smaps": {"anon_huge_kb_max": 0},
        "rusage": {"minflt": 30000, "utime_s": 60.0, "stime_s": 1.0},
        "status": {"exit": 0},
    }
    child = {"out": copies * tm.WORKLOAD["out"], "err": ""}
    return rec, child


SUSTAINED = {
    "steady": lambda t: 1.3,
    "low": lambda t: 0.68 if t > 4 else 1.3,
    "alternating": lambda t: 1.3 if int(t / 2) % 2 == 0 else 0.68,
}


def identity(n=tm.RUNS, dry=False):
    cat = {
        "schema": tm.CONCAT_SCHEMA,
        "copies": tm.SUSTAINED_COPIES if not dry else tm.DRY_SUSTAINED_COPIES,
        "corpus_sha256": tm.CORPUS_SHA256,
        "expected_sha256": H,
        "sha256": H,
        "ok": True,
    }
    return {
        "schema": tm.IDENTITY_SCHEMA,
        "dry": dry,
        "head": H40,
        "frozen": H40,
        "inputs_match_frozen": True,
        "tree_clean_before": True,
        "tree_clean_after": True,
        "sources": {p: {"head": H, "frozen": H} for p in tm.SOURCES},
        "harness": {
            "evidence_commit": tm.SOURCE["evidence_commit"],
            "path": tm.SOURCE["path"],
            "bundle_sha256": tm.SOURCE["sha256"],
        },
        "ab_sha256": tm.SOURCE["ab_sha256"],
        "ab_sha256_after": tm.SOURCE["ab_sha256"],
        "windows_sha256": tm.SOURCE["windows_sha256"],
        "windows_sha256_after": tm.SOURCE["windows_sha256"],
        "probe_sha256": H,
        "probe_sha256_after": H,
        "corpus_sha256": tm.CORPUS_SHA256,
        "corpus_sha256_after": tm.CORPUS_SHA256,
        "arm": tm.ARM,
        "threads": 10,
        "rounds": n,
        "settle": 0.25,
        "sysfs_state": True,
        "layout": {"idle": tm.IDLE_CPU, "sib": tm.SIB_CPU, "decoder": tm.DECODER_CPUS},
        "schedule": {str(k): v for k, v in tm.SCHEDULE.items()},
        "schedule_seed": tm.SCHEDULE_SEED,
        "preflight_exit": 0,
        "passes": [1, 2],
        "pass_exits": {"1": 0, "2": 0},
        "sustained": {"copies": cat["copies"], "exit": 0, "concat": cat, "sha256_after": H},
    }


def attempt(d, sig, fast_runs=18, n=tm.RUNS, mutate=None, sus="steady"):
    """Write a synthetic attempt into d under signature sig"""
    os.makedirs(os.path.join(d, "sustained"), exist_ok=True)
    rec, child = sustained(SUSTAINED[sus])
    for name, obj in (("sustained.json", rec), ("child.json", child)):
        with open(os.path.join(d, "sustained", name), "w") as f:
            json.dump(obj, f)
    os.makedirs(os.path.join(d, "preflight"), exist_ok=True)
    with open(os.path.join(d, "preflight", "static.json"), "w") as f:
        json.dump(static(), f)
    with open(os.path.join(d, "preflight.json"), "w") as f:
        json.dump({"verdict": "PASS"}, f)
    with open(os.path.join(d, "identity.json"), "w") as f:
        json.dump(identity(n), f)
    fast = pattern(n, fast_runs)
    for pass_no in (1, 2):
        os.makedirs(os.path.join(d, f"pass{pass_no}"), exist_ok=True)
        recs, probes = [], []
        t = 1_794_000_000_000_000_000
        for i in range(n):
            wall = 0.155 if fast[i] else 0.305
            recs.append(run_record(i, wall, t))
            probes.append(probe_record(i, wall, sig, fast[i], pass_no, t))
            t += int((wall + 0.27) * 1e9)
        if mutate:
            mutate(pass_no, recs, probes)
        with open(os.path.join(d, f"pass{pass_no}", "runs.jsonl"), "w") as f:
            f.writelines(json.dumps(r) + "\n" for r in recs)
        with open(os.path.join(d, f"pass{pass_no}", "probe.jsonl"), "w") as f:
            f.writelines(json.dumps(p) + "\n" for p in probes)


def dump(obj, path):
    with open(path, "w") as f:
        json.dump(obj, f)


def load(path):
    with open(path) as f:
        return json.load(f)


def quiet(f, *a):
    with contextlib.redirect_stdout(io.StringIO()):
        return f(*a)


def judged(sig, edit=None, **kw):
    with tempfile.TemporaryDirectory() as d:
        attempt(d, sig, **kw)
        if edit:
            p = os.path.join(d, "identity.json")
            ident = load(p)
            edit(ident)
            dump(ident, p)
        rc = quiet(tm.main, ["judge", d])
        with open(os.path.join(d, "judgement.json")) as f:
            j = json.load(f)
        assert os.path.exists(os.path.join(d, "judgement.md"))
    return rc, j


class Signatures(unittest.TestCase):
    def check(self, sig, want, **kw):
        rc, j = judged(sig, **kw)
        self.assertEqual(rc, 0, j["outcome"])
        self.assertEqual(j["label"], "VALID")
        self.assertTrue(j["result"].startswith(f"COMPATIBLE WITH {want}"), j["result"])
        self.assertFalse(j["result"].startswith(f"COMPATIBLE WITH {want},"), j["result"])
        return j

    def test_h1_ramp(self):
        j = self.check("H1", "H1")
        self.assertEqual(j["pass1"]["rows"]["clock"], "low, then rising")
        self.assertIn("H1 rests on the supporting clock reading", j["result"])

    def test_h1_ramp_on_a_shared_clock(self):
        j = self.check("H1shared", "H1")
        self.assertEqual(j["pass1"]["rows"]["p_idle"], "rising")

    def test_h2_packing(self):
        j = self.check("H2", "H2")
        self.assertEqual(j["pass1"]["rows"]["sibling"], "most")

    def test_h3_stop_clock(self):
        j = self.check("H3", "H3")
        self.assertEqual(j["pass1"]["rows"]["p_idle"], "half")
        self.assertEqual(j["pass1"]["rows"]["throttle"], "yes")
        self.assertNotIn("rests on", j["result"])

    def test_h4_pages(self):
        j = self.check("H4", "H4")
        self.assertEqual(j["pass1"]["rows"]["thp"], "yes")

    def test_h5_power(self):
        j = self.check("H5", "H5")
        self.assertEqual(j["pass1"]["signatures"]["H5"]["status"], "compatible")

    def test_h5_without_power_counts_is_only_in_part(self):
        j = self.check("H5+nopower", "H5")
        self.assertEqual(j["pass1"]["signatures"]["H5"]["status"], "compatible in part")
        self.assertIn("H5 in part: power-limit counts rise missing", j["result"])

    def test_no_difference_matches_nothing(self):
        rc, j = judged("none")
        self.assertTrue(j["result"].startswith("NO REGISTERED SIGNATURE MATCHES"), j["result"])

    def test_a_signature_none_predicts(self):
        rc, j = judged("sixth")  # P-idle half with no throttle count rising
        self.assertTrue(j["result"].startswith("NO REGISTERED SIGNATURE MATCHES"), j["result"])
        self.assertIn("P-idle's elapsed-time progress in slow runs: half", j["result"])
        self.assertNotIn("SIXTH", j["result"])

    def test_insufficient_class_support(self):
        rc, j = judged("H3", fast_runs=0)
        self.assertTrue(j["result"].startswith("INSUFFICIENT CLASS SUPPORT: 0 fast"), j["result"])
        self.assertNotIn("ONE MODE", j["result"])

    def test_four_fast_runs_are_not_one_mode(self):
        rc, j = judged("H3", fast_runs=6)  # (four of them probed, at most)
        self.assertLess(j["pass1"]["table"]["fast"]["n"], 5)
        self.assertTrue(j["result"].startswith("INSUFFICIENT CLASS SUPPORT"), j["result"])

    def test_threshold_is_fixed(self):
        self.assertEqual(tm.THRESHOLD_S, 0.21)
        r = tm.readings(run_record(0, 0.2099, 0), probe_record(0, 0.2099, "H3", True, 2, 0), SIBS)
        self.assertTrue(r["fast"])
        r = tm.readings(run_record(0, 0.21, 0), probe_record(0, 0.21, "H3", False, 2, 0), SIBS)
        self.assertFalse(r["fast"])

    def test_serial_contrast_reported(self):
        rc, j = judged("H3")
        self.assertAlmostEqual(j["serial_contrast"]["fast runs"], 0.9, places=2)

    def test_persistence(self):
        rc, j = judged("H3")
        p = j["pass1"]["persistence"]
        self.assertEqual(p["sequence"][:9], "FFFssssss")
        self.assertEqual(p["fast_after_fast"], 12)

    def test_the_probes_own_effect_on_the_mix(self):
        rc, j = judged("H3")
        m = j["pass1"]["mix"]
        self.assertEqual((m["probed"][1], m["unprobed"][1]), (40, 20))
        self.assertEqual(m["probed"][0] + m["unprobed"][0], 18)
        self.assertIsNone(j["pass1"]["unprobed_table"]["fast"]["p_idle"])

    def test_the_schedule_is_randomised_and_fixed(self):
        s = tm.SCHEDULE[1]
        self.assertEqual((len(s), s.count("P")), (60, 40))
        self.assertEqual(s, tm._schedule(tm.SCHEDULE_SEED))
        # a period-three nuisance (fast, fast, slow) no longer aligns with the slots
        fast = [i % 3 != 2 for i in range(60)]
        p = sum(f for f, c in zip(fast, s) if c == "P")
        n = sum(f for f, c in zip(fast, s) if c == "N")
        self.assertNotEqual((p, n), (40, 0))

    def test_schedule_is_checked(self):
        def m(p, r, q):
            if p == 1:
                q[2]["probed"] = not q[2]["probed"]

        rc, j = judged("H3", mutate=m)
        self.assertEqual(rc, 2)
        self.assertIn("not slot 2 of the registered schedule", j["outcome"])


class Counterexamples(unittest.TestCase):
    """The Researcher's five, each failing on 3588d35d and passing now"""

    def test_1_a_pause_halves_the_progress(self):
        k, n, base = ITERS, 700, 808277
        normal = [base] * n
        stalled = normal.copy()
        stalled[n // 2] += n * base
        for ds, want in ((normal, 4.8), (stalled, 2.4)):
            p = {"first_tsc": 0, "iters_per_sample": k, "dtsc": ds}
            r = tm.progress(p, 0, sum(ds), HZ)
            self.assertAlmostEqual(r["progress"], want, places=2)
            self.assertAlmostEqual(r["active_median"], 4.8, places=2)
        self.assertAlmostEqual(tm.progress(p, 0, sum(stalled), HZ)["longest_ms"], 153.1, places=0)

    def test_1_a_pause_across_a_boundary_lowers_coverage(self):
        k, base = ITERS, 808277
        ds = [base] * 700
        ds[0] += 300 * base  # a pause straddling the window's start
        p = {"first_tsc": 0, "iters_per_sample": k, "dtsc": ds}
        r = tm.progress(p, 100 * base, sum(ds), HZ)
        self.assertLess(r["coverage"], tm.MIN_COVERAGE)
        self.assertGreater(r["gap_start_ms"], 40)

    def test_1_a_pause_inside_a_run_lowers_its_progress(self):
        probe = probe_record(0, 0.305, "H3", False, 2, 0)
        p = probe["probes"][0]
        p["dtsc"][len(p["dtsc"]) // 3] += int(0.05 * HZ)  # (a 50 ms pause inside)
        r = tm.readings(run_record(0, 0.305, 0), probe, SIBS)
        self.assertGreaterEqual(r["p_idle_coverage"], tm.MIN_COVERAGE)
        self.assertLess(r["p_idle"], 0.9 * r["p_idle_active"])
        self.assertGreater(r["p_idle_longest_ms"], 49)

    def test_1_an_uncovered_run_has_no_progress_reading(self):
        probe = probe_record(0, 0.305, "H3", False, 2, 0)
        p = probe["probes"][0]
        p["dtsc"][len(p["dtsc"]) // 3] += int(0.25 * HZ)  # (a pause past the run's end)
        r = tm.readings(run_record(0, 0.305, 0), probe, SIBS)
        self.assertLess(r["p_idle_coverage"], tm.MIN_COVERAGE)
        self.assertIsNone(r["p_idle"])

    def test_2_sleeping_siblings_are_not_busy(self):
        def sleeping(p, recs, probes):
            for x in probes:
                for _, tasks in x["placement"]["samples"]:
                    for task in tasks:
                        if task[1] >= 10:
                            task[2] = "S"

        rc, j = judged("H2", mutate=sleeping)
        self.assertFalse(j["result"].startswith("COMPATIBLE WITH H2"), j["result"])
        self.assertEqual(j["pass1"]["table"]["slow"]["sibling_running"], 0)
        self.assertGreater(j["pass1"]["table"]["slow"]["sibling_last_cpu"], 0.9)
        self.assertEqual(j["pass1"]["signatures"]["H2"]["status"], "contradicted")

    def test_3_constant_input_is_constant_within_error(self):
        for rate in (0.68, 0.98, 1.00, 1.01, 1.02, 1.05, 1.10, 1.3):
            for phase in (0.0, 0.25, 0.5, 0.75):
                rec, _ = sustained(lambda t, r=rate: r, phase=phase)
                s = tm.sustained_reading(rec)
                self.assertEqual(s["shape"], "constant within error", (rate, phase, s))

    def test_3_alternating_and_settling_are_seen(self):
        for phase in (0.0, 0.5):
            rec, _ = sustained(SUSTAINED["alternating"], phase=phase)
            self.assertEqual(tm.sustained_reading(rec)["shape"], "alternating beyond error")
            rec, _ = sustained(SUSTAINED["low"], phase=phase)
            self.assertEqual(tm.sustained_reading(rec)["shape"], "a sustained change, down")

    def test_3_no_burst_threshold_or_ratio_classifies(self):
        self.assertFalse(hasattr(tm, "FAST_RATE"))
        rec, _ = sustained(lambda t: 1.0)
        s = tm.sustained_reading(rec)
        self.assertIn("windows_100ms_bytes", s)
        self.assertNotIn("blocks_gbps", s)

    def test_4_missing_decisive_evidence_is_not_a_match(self):
        v = dict(tm.BASELINE)
        v["clock"] = tm.UNREAD
        o = tm.outcome({"fast": {"n": 20}, "slow": {"n": 20}}, v)
        self.assertTrue(o.startswith("NO REGISTERED SIGNATURE MATCHES"), o)
        self.assertIn("untestable: H1", o)
        self.assertEqual(tm.signatures(v)["H1"]["status"], "untestable")

    def test_4_every_decisive_row_missing(self):
        v = {r: tm.UNREAD for r in tm.ROWS}
        o = tm.outcome({"fast": {"n": 20}, "slow": {"n": 20}}, v)
        self.assertTrue(o.startswith("INSUFFICIENT MEASUREMENT SUPPORT"), o)

    def test_4_a_row_with_too_few_observations_is_insufficient(self):
        def few(p, recs, probes):
            for x in probes[5:]:
                x["freq"]["samples"] = []

        rc, j = judged("H1", mutate=few)
        self.assertEqual(j["pass1"]["rows"]["clock"], tm.INSUFFICIENT)
        self.assertEqual(j["pass1"]["signatures"]["H1"]["status"], "untestable")

    def test_5_each_contradiction_alone_is_invalid(self):
        edits = {
            "frozen": lambda i: i.update(frozen=None),
            "inputs": lambda i: i.update(inputs_match_frozen=False),
            "preflight": lambda i: i.update(preflight_exit=1),
            "sustained": lambda i: i["sustained"].update(exit=124),
            "ab after": lambda i: i.update(ab_sha256_after="0" * 64),
            "probe after": lambda i: i.update(probe_sha256_after="b" * 64),
            "corpus after": lambda i: i.update(corpus_sha256_after="d" * 64),
            "corpus": lambda i: i.update(corpus_sha256="c" * 64, corpus_sha256_after="c" * 64),
            "source": lambda i: i["sources"]["bench/two_modes.py"].update(frozen="e" * 64),
            "tree": lambda i: i.update(tree_clean_after=False),
            "harness": lambda i: i["harness"].update(bundle_sha256="f" * 64),
            "pass timeout": lambda i: i["pass_exits"].update({"2": 124}),
            "concat": lambda i: i["sustained"]["concat"].update(sha256="9" * 64),
            "concat after": lambda i: i["sustained"].update(sha256_after="8" * 64),
            "seed": lambda i: i.update(schedule_seed=1),
            "layout": lambda i: i.update(layout={"idle": 8, "sib": 18, "decoder": "0-7"}),
        }
        for what, edit in edits.items():
            rc, j = judged("H3", edit=edit)
            self.assertEqual((rc, j["label"]), (2, "INVALID"), what)
            self.assertTrue(j["problems"], what)

    def test_5_the_preflight_judgement_must_be_pass(self):
        with tempfile.TemporaryDirectory() as d:
            attempt(d, "H3")
            dump({"verdict": "REFUSED"}, os.path.join(d, "preflight.json"))
            self.assertEqual(quiet(tm.main, ["judge", d]), 2)

    def test_5_the_clean_attempt_is_valid(self):
        rc, j = judged("H3")
        self.assertEqual((rc, j["label"], j["problems"]), (0, "VALID", []))


class Invalid(unittest.TestCase):
    def broken(self, mutate, expect):
        rc, j = judged("H3", mutate=mutate)
        self.assertEqual(rc, 2, j["outcome"])
        self.assertTrue(j["outcome"].startswith("INVALID: "), j["outcome"])
        self.assertIn(expect, j["outcome"])

    def test_missing_probe_record(self):
        self.broken(lambda p, r, q: q.pop(), "probe records for 60 runs")

    def first_probed(self, q):
        return next(x for x in q if x["probes"])

    def test_probe_not_pinned(self):
        def m(p, r, q):
            self.first_probed(q)["probes"][0]["readback"] = "0-19"

        self.broken(m, "not pinned to CPU 9")

    def test_decoder_mask_not_read_back(self):
        def m(p, r, q):
            q[5]["decoder_cpus_readback"] = "0-19"

        self.broken(m, "the decoder's mask read back")

    def test_failed_run(self):
        def m(p, r, q):
            r[7]["termination"] = "signal 9"

        self.broken(m, "did not succeed")

    def test_other_workload(self):
        def m(p, r, q):
            r[0]["hash"] = "0000"

        self.broken(m, "not the registered workload")

    def test_ring_dropped(self):
        def m(p, r, q):
            self.first_probed(q)["probes"][0]["dropped"] = 4

        self.broken(m, "dropped")

    def test_probe_outside_its_run(self):
        def m(p, r, q):
            q[2]["t_unix_ns_start"] = r[3]["t_spawn_unix_ns"] + 5

        self.broken(m, "does not fall inside the run")

    def test_runs_out_of_order(self):
        def m(p, r, q):
            r[4]["t_spawn_unix_ns"], r[5]["t_spawn_unix_ns"] = (
                r[5]["t_spawn_unix_ns"],
                r[4]["t_spawn_unix_ns"],
            )

        self.broken(m, "are not ordered after the run before")

    def test_probe_timestamps_out_of_order(self):
        def m(p, r, q):
            q[6]["tsc"]["exit"] = q[6]["tsc"]["exec"] - 1

        self.broken(m, "its timestamps are not ordered")

    def test_no_pass2_psib(self):
        def m(p, r, q):
            if p == 2:
                for x in q:
                    x["probes"] = x["probes"][:1]

        self.broken(m, "probes [('P-idle', 9)], not")

    def test_short_attempt(self):
        def m(p, r, q):
            del r[-1]
            del q[-1]

        self.broken(m, "59 runs recorded, not 60")

    def test_sustained_block_missing(self):
        with tempfile.TemporaryDirectory() as d:
            attempt(d, "H3")
            os.remove(os.path.join(d, "sustained", "sustained.json"))
            self.assertEqual(quiet(tm.main, ["judge", d]), 2)
            with open(os.path.join(d, "judgement.json")) as f:
                self.assertIn("the sustained block", json.load(f)["outcome"])

    def test_dry_without_a_sibling(self):
        def m(p, r, q):
            for x in q:
                x["probes"] = x["probes"][:1]
                x["decoder_cpus"] = x["decoder_cpus_readback"] = x["decoder_cpus_status"] = "0,1,2"
                for y in x["probes"]:
                    y["cpu"] = 3
                    y["readback"] = "3"

        with tempfile.TemporaryDirectory() as d:
            attempt(d, "H3", n=3, fast_runs=1, mutate=m)
            rec, child = sustained(SUSTAINED["steady"], copies=tm.DRY_SUSTAINED_COPIES)
            dump(rec, os.path.join(d, "sustained", "sustained.json"))
            dump(child, os.path.join(d, "sustained", "child.json"))
            ident = identity(3, dry=True)
            ident.update(layout={"idle": 3, "sib": None, "decoder": "0,1,2"}, passes=[1])
            ident["pass_exits"] = {"1": 0}
            dump(ident, os.path.join(d, "identity.json"))
            self.assertEqual(quiet(tm.main, ["judge", d, "--dry"]), 0)
            j = load(os.path.join(d, "judgement.json"))
            self.assertEqual(j["label"], "DRY")
            self.assertTrue(
                j["outcome"].startswith("DRY (not evidence): INSUFFICIENT"), j["outcome"]
            )
            self.assertIsNone(j["pass2"])
            # and without --dry the same attempt is INVALID, never VALID
            self.assertEqual(quiet(tm.main, ["judge", d]), 2)


class Concat(unittest.TestCase):
    def test_built_by_content_and_a_wrong_cache_rebuilt(self):
        with tempfile.TemporaryDirectory() as d:
            c, o = os.path.join(d, "c.bz2"), os.path.join(d, "x.bz2")
            Path(c).write_bytes(b"abc" * 1000)
            r = tm.concat(c, 3, o)
            self.assertTrue(r["ok"] and not r["cached"])
            self.assertEqual(Path(o).read_bytes(), b"abc" * 3000)
            self.assertTrue(tm.concat(c, 3, o)["cached"])
            Path(o).write_bytes(b"abd" * 3000)  # same size, wrong content
            r = tm.concat(c, 3, o)
            self.assertTrue(r["ok"] and not r["cached"])
            self.assertEqual(Path(o).read_bytes(), b"abc" * 3000)


GOOD_EXITS = "".join(f"{n}: exit 0\n" for n in tm.PREFLIGHT_COLLECTORS)


def env():
    return {
        "facts": tm.FACTS_SCHEMA,
        "cpus": 20,
        "cpus_allowed": list(range(20)),
        "cores": 10,
        "core_cpus": [f"{c},{c + 10}" for c in range(10)],
        "cpu_quota": {
            "state": "unlimited",
            "cpus": None,
            "hierarchy": "v2",
            "levels": ["/sys/fs/cgroup"],
            "why": None,
        },
        "spin_threads": 0,
        "perf_probed": False,
        "machine": dict(tm.DECLARED),
    }


def preflight_dir(
    d,
    exits=GOOD_EXITS,
    lscpu="Model name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n",
    e=None,
    st=None,
    sel=None,
):
    p = os.path.join(d, "preflight")
    os.makedirs(p, exist_ok=True)
    if exits is not None:
        with open(os.path.join(p, "exits.txt"), "w") as f:
            f.write(exits)
    with open(os.path.join(p, "lscpu.txt"), "w") as f:
        f.write(lscpu)
    with open(os.path.join(p, "env.json"), "w") as f:
        json.dump(e or env(), f)
    state = {"governor": "powersave", "epp": "balance_performance", "turbo": True}
    with open(os.path.join(p, "sample.jsonl"), "w") as f:
        f.writelines(json.dumps({"state": state}) + "\n" for _ in range(3))
    with open(os.path.join(p, "static.json"), "w") as f:
        json.dump(st or static(), f)
    with open(os.path.join(p, "selftest.json"), "w") as f:
        json.dump(sel or {"cpu": 9, "pinned": True, "readback": "9", "samples": 900}, f)


def pf(**kw):
    with tempfile.TemporaryDirectory() as d:
        preflight_dir(d, **kw)
        rc = tm.main(["preflight", d])
        with open(os.path.join(d, "preflight.json")) as f:
            return rc, json.load(f)


class Preflight(unittest.TestCase):
    def refused(self, expect, **kw):
        rc, j = pf(**kw)
        self.assertEqual(rc, 1)
        self.assertEqual(j["verdict"], "REFUSED")
        self.assertTrue(any(expect in x for x in j["problems"]), j["problems"])

    def test_pass(self):
        rc, j = pf()
        self.assertEqual((rc, j["verdict"]), (0, "PASS"), j["problems"])

    def test_missing_exits(self):
        self.refused("exits.txt) are missing", exits=None)

    def test_nonzero_exit(self):
        self.refused(
            "probe selftest exited 1",
            exits=GOOD_EXITS.replace("probe selftest: exit 0", "probe selftest: exit 1"),
        )

    def test_timeout(self):
        self.refused(
            "(timed out)", exits=GOOD_EXITS.replace("ab sample: exit 0", "ab sample: exit 124")
        )

    def test_collector_missing(self):
        self.refused(
            "static facts: no exit status", exits=GOOD_EXITS.replace("static facts: exit 0\n", "")
        )

    def test_collector_twice(self):
        self.refused("recorded twice", exits=GOOD_EXITS + "lscpu: exit 0\n")

    def test_malformed(self):
        self.refused("malformed", exits=GOOD_EXITS + "lscpu: exited fine\n")

    def test_other_host(self):
        self.refused("not the registered i9-10900K", lscpu="Model name: Intel(R) Xeon(R)\n")

    def test_other_governor(self):
        e = env()
        e["machine"]["governor"] = "performance"
        self.refused("governor 'performance'", e=e)

    def test_no_invariant_tsc(self):
        s = static()
        s["cpus"]["4"]["tsc_flags"]["nonstop_tsc"] = False
        self.refused("lack constant_tsc or nonstop_tsc", st=s)

    def test_throttle_counts_unread(self):
        s = static()
        s["cpus"]["7"]["thermal_throttle"] = None
        self.refused("no core and package throttle counts", st=s)

    def test_selftest_elsewhere(self):
        self.refused(
            "selftest was not pinned",
            sel={"cpu": 9, "pinned": True, "readback": "0-19", "samples": 9},
        )

    def test_cli_exit_through_recipe_shape(self):
        # the recipe reads the first line and the exit status
        with tempfile.TemporaryDirectory() as d:
            preflight_dir(d, exits=GOOD_EXITS.replace("lscpu: exit 0", "lscpu: exit 137"))
            r = subprocess.run(
                [sys.executable, str(HERE / "two_modes.py"), "preflight", d],
                capture_output=True,
                text=True,
            )
            self.assertEqual(r.returncode, 1)
            self.assertTrue(r.stdout.startswith("REFUSED: "), r.stdout)
            self.assertIn("(killed)", r.stdout)


class Layout(unittest.TestCase):
    def test_registered(self):
        self.assertEqual(tm.layout(env(), False), (9, 19, "0-8,10-18"))
        self.assertEqual(tm.cpu_list(tm.DECODER_CPUS), [c for c in range(20) if c not in (9, 19)])

    def test_dry_takes_the_last_core(self):
        e = env()
        self.assertEqual(
            tm.layout(e, True), (9, 19, ",".join(str(c) for c in range(20) if c not in (9, 19)))
        )
        e = {"core_cpus": ["0", "1", "2", "3"], "cpus_allowed": [0, 1, 2, 3]}
        self.assertEqual(tm.layout(e, True), (3, None, "0,1,2"))


if __name__ == "__main__":
    unittest.main(verbosity=1)
