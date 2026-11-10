"""bench/two_modes.py on synthetic i9-shaped attempts: each mechanism's signature (§2 of
docs/journal/2026-11-10-two-modes.md) is matched to its column and no other, a signature none
predicts is reported as a sixth mechanism, one mode alone matches nothing, a broken attempt is
INVALID, and the preflight refuses each defect in its collectors' exit statuses.

    python3 bench/test_two_modes.py
"""

import copy
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
DEC = tm.cpu_list(tm.DECODER_CPUS)
NAMES = (
    ["mibz2-0", "mibz2-1", "mibz2-2", "mibz2-scan", "mibz2-read"]
    + [f"mibz2-coop-p{1 + i % 2}" for i in range(6)]
    + ["sampler", "ab"]
)


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
        if slow and sig == "H3":
            return base * 0.5
        if slow and sig == "H5":
            return base * 0.8
        if slow and sig == "H1shared":
            return base * (0.3 + 0.7 * frac)
        if slow and sig == "sixth":
            return base * 0.5
        return base

    def ring(rate_of):
        t = exec_tsc - 2_000_000
        first = t
        dt = []
        while t < exit_tsc + 2_000_000:
            frac = min(max((t - exec_tsc) / (exit_tsc - exec_tsc), 0), 1)
            d = int(ITERS * HZ / (rate_of(frac) * 1e9))
            dt.append(d)
            t += d
        return first, dt

    sched = tm.SCHEDULE[pass_no]
    probed = sched[i % len(sched)] == "P"
    probes = []
    want = [("P-idle", tm.IDLE_CPU)] + ([("P-sib", tm.SIB_CPU)] if pass_no == 2 else [])
    for name, cpu in want if probed else []:
        rate = p_rate if name == "P-idle" else (lambda f: 4.0)
        first, dt = ring(rate if pass_no == 1 else (lambda f, r=rate: r(f) * 0.9))
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
                "first_tsc": first,
                "dtsc": dt,
            }
        )
    # placement: 13 tasks; spread over distinct cores, or packed onto siblings under H2's
    # slow runs
    comm = {str(1000 + k): n for k, n in enumerate(NAMES)}
    packed = slow and sig == "H2"
    spread = [0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13]  # 9 cores: a few pairs share
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
        if slow and sig == "H1":
            ghz = 1.2 + 3.7 * frac
        if slow and sig == "H1shared":
            ghz = 1.2 + 3.7 * frac
        if slow and sig == "H5":
            ghz = 3.9
        freq.append([t, 300, [int(ghz * 1e6)] * 20])
    rise = {"core": 1 if slow and sig == "H3" else 0, "pl": 1 if slow and sig == "H5" else 0}
    thp = 40 if fast and sig == "H4" else 0
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
        "after": counters(5 + rise["core"], 5, 2 + rise["pl"], 100 + thp, power),
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


def sustained(rates, chunk=4_200_000, copies=tm.SUSTAINED_COPIES):
    """A sustained block's record: its input read in chunks, at a decoded-equivalent rate
    (GB/s) that `rates(t)` gives at each time t"""
    total = copies * tm.CORPUS_BYTES
    ratio = tm.WORKLOAD["out"] / tm.CORPUS_BYTES
    samples, read, due, t = [], 3 * chunk, 0.0, 0.0
    while read < total:
        due += rates(t) * 1e9 / ratio * 0.01
        while due >= chunk and read < total:
            read += chunk
            due -= chunk
        samples.append([int(t * 1e6), min(read, total)])
        t += 0.01
    rec = {
        "schema": "two_modes/sustained/1",
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
    "low": lambda t: 0.68 if t > 0.5 else 1.3,
    "alternating": lambda t: 1.3 if int(t / 2) % 2 == 0 else 0.68,
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
    ident = {
        "schema": "two_modes/identity/1",
        "dry": False,
        "rounds": n,
        "layout": {"idle": tm.IDLE_CPU, "sib": tm.SIB_CPU, "decoder": tm.DECODER_CPUS},
        "ab_sha256": tm.SOURCE["ab_sha256"],
        "sustained_copies": tm.SUSTAINED_COPIES,
        "windows_sha256": tm.SOURCE["windows_sha256"],
    }
    with open(os.path.join(d, "identity.json"), "w") as f:
        json.dump(ident, f)
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


def judged(sig, **kw):
    with tempfile.TemporaryDirectory() as d:
        attempt(d, sig, **kw)
        rc = tm.main(["judge", d])
        with open(os.path.join(d, "judgement.json")) as f:
            j = json.load(f)
        assert os.path.exists(os.path.join(d, "judgement.md"))
    return rc, j


class Columns(unittest.TestCase):
    def check(self, sig, want):
        rc, j = judged(sig)
        self.assertEqual(rc, 0, j["outcome"])
        self.assertEqual(j["label"], "VALID")
        self.assertTrue(j["result"].startswith(f"MATCH {want} "), j["result"])
        return j

    def test_h1_ramp(self):
        j = self.check("H1", "H1")
        # (H1 differs from the others by the clock row: the outcome says it rests on it)
        self.assertIn("resting on the supporting reading", j["result"])
        self.assertEqual(j["pass1"]["rows"]["clock"], "low, then rising")
        self.assertEqual(j["pass1"]["rows"]["p_idle"], "full")

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
        self.assertNotIn("resting on", j["result"])

    def test_h4_pages(self):
        j = self.check("H4", "H4")
        self.assertEqual(j["pass1"]["rows"]["thp"], "yes")
        self.assertEqual(j["pass1"]["rows"]["sys_share"], "yes")

    def test_h5_power(self):
        j = self.check("H5", "H5")
        self.assertEqual(j["pass1"]["rows"]["power_limit"], "yes")

    def test_power_limit_unread_is_said(self):
        # (an unread row admits any column, and the outcome names it)
        j = self.check("H3+nopower", "H3")
        self.assertEqual(j["pass1"]["rows"]["power_limit"], tm.UNREAD)
        self.assertIn("(power-limit counts rise unread)", j["result"])

    def test_no_difference_matches_nothing(self):
        rc, j = judged("none")
        self.assertTrue(j["result"].startswith("SIXTH MECHANISM"), j["result"])

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
            with open(os.path.join(d, "identity.json")) as f:
                ident = json.load(f)
            ident.update(
                dry=True,
                rounds=3,
                sustained_copies=tm.SUSTAINED_COPIES,
                layout={"idle": 3, "sib": None, "decoder": "0,1,2"},
            )
            with open(os.path.join(d, "identity.json"), "w") as f:
                json.dump(ident, f)
            self.assertEqual(tm.main(["judge", d, "--dry"]), 0)
            with open(os.path.join(d, "judgement.json")) as f:
                j = json.load(f)
            self.assertEqual(j["label"], "DRY")
            self.assertTrue(j["outcome"].startswith("DRY (not evidence): ONE MODE"), j["outcome"])
            self.assertIsNone(j["pass2"])
            # and without --dry the same attempt is INVALID, never VALID
            self.assertEqual(tm.main(["judge", d]), 2)

    def test_sixth_mechanism(self):
        rc, j = judged("sixth")  # P-idle half with no throttle count rising: no column
        self.assertEqual(rc, 0)
        self.assertTrue(j["result"].startswith("SIXTH MECHANISM"), j["result"])
        self.assertIn("P-idle's rate in slow runs: half", j["result"])

    def test_one_mode(self):
        rc, j = judged("H3", fast_runs=0)
        self.assertTrue(j["result"].startswith("ONE MODE: 0 fast and 40 slow"), j["result"])

    def test_threshold_is_fixed(self):
        self.assertEqual(tm.THRESHOLD_S, 0.21)
        r = tm.readings(run_record(0, 0.2099, 0), probe_record(0, 0.2099, "H3", True, 1, 0), SIBS)
        self.assertTrue(r["fast"])
        r = tm.readings(run_record(0, 0.21, 0), probe_record(0, 0.21, "H3", False, 1, 0), SIBS)
        self.assertFalse(r["fast"])

    def test_sustained_block_agrees(self):
        rc, j = judged("H1")
        self.assertEqual(j["sustained"]["signature"], "fast and steady")
        self.assertIn("sustained block: fast and steady (consistent with H1)", j["result"])

    def test_sustained_block_settles_low(self):
        rc, j = judged("H3", sus="low")
        self.assertEqual(j["sustained"]["signature"], "settles low")
        self.assertIn("(consistent with H3)", j["result"])
        rc, j = judged("H1", sus="low")
        self.assertIn("settles low (not what H1 predicts)", j["result"])

    def test_sustained_block_alternates(self):
        rc, j = judged("H5", sus="alternating")
        self.assertEqual(j["sustained"]["signature"], "alternates")
        self.assertGreaterEqual(j["sustained"]["switches"], 2)

    def test_sustained_block_missing_is_invalid(self):
        def m(p, r, q):
            pass

        with tempfile.TemporaryDirectory() as d:
            attempt(d, "H3", mutate=m)
            os.remove(os.path.join(d, "sustained", "sustained.json"))
            self.assertEqual(tm.main(["judge", d]), 2)
            with open(os.path.join(d, "judgement.json")) as f:
                self.assertIn("the sustained block", json.load(f)["outcome"])

    def test_the_probes_own_effect_on_the_mix(self):
        rc, j = judged("H3")
        self.assertEqual(j["pass1"]["mix"], {"probed": [12, 40], "unprobed": [6, 20]})
        self.assertEqual(j["pass2"]["mix"]["unprobed"], [0, 0])
        self.assertEqual(j["pass1"]["unprobed_table"]["fast"]["n"], 6)
        self.assertIsNone(j["pass1"]["unprobed_table"]["fast"]["p_idle"])

    def test_schedule_is_checked(self):
        def m(p, r, q):
            if p == 1:
                q[2]["probed"] = True

        rc, j = judged("H3", mutate=m)
        self.assertEqual(rc, 2)
        self.assertIn("not slot 2 of 'PPN'", j["outcome"])

    def test_sibling_cost_reported(self):
        rc, j = judged("H3")
        self.assertAlmostEqual(j["sibling_cost"]["fast runs"], 0.9, places=2)

    def test_persistence(self):
        rc, j = judged("H3")
        p = j["pass1"]["persistence"]
        self.assertEqual(p["sequence"][:9], "FFFssssss")
        self.assertEqual(p["fast_after_fast"], 12)


class Invalid(unittest.TestCase):
    def broken(self, mutate, expect):
        rc, j = judged("H3", mutate=mutate)
        self.assertEqual(rc, 2, j["outcome"])
        self.assertTrue(j["outcome"].startswith("INVALID: "), j["outcome"])
        self.assertIn(expect, j["outcome"])

    def test_missing_probe_record(self):
        self.broken(lambda p, r, q: q.pop(), "probe records for 60 runs")

    def test_probe_not_pinned(self):
        def m(p, r, q):
            q[3]["probes"][0]["readback"] = "0-19"

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
            q[1]["probes"][0]["dropped"] = 4

        self.broken(m, "dropped")

    def test_probe_outside_its_run(self):
        def m(p, r, q):
            q[2]["t_unix_ns_start"] = r[3]["t_spawn_unix_ns"] + 5

        self.broken(m, "does not fall inside the run")

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
