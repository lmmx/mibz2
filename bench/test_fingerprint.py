"""bench/fingerprint.py's rules on synthetic records, and its cleanup on a real child.

- the validity screens: a competitor at half a CPU, a victim that waited, throttling, a
  shared core, too few cores, each leads to a screened run or INCONCLUSIVE;
- the paired statistics: per-round ratios, so a drift that moves every run of a round
  cancels;
- C_eff from a synthetic latency curve, and no C_eff where the curve has no knee;
- the stressors' calibration checks;
- the verdicts' wording: never a hardware cause;
- the cleanup: a started competitor dies with the orchestrator, by SIGTERM, SIGINT, or
  (the real stressor) by its stdin closing when the orchestrator is killed outright.

    python3 bench/test_fingerprint.py
"""

import math
import os
import random
import re
import signal
import stat
import subprocess
import sys
import tempfile
import textwrap
import time
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import fingerprint as fp  # noqa: E402
from report import ratio_ci  # noqa: E402

CORES = [[0, 1], [2, 3]]  # CI's runners: SMT siblings 0-1 and 2-3


def rec(cond="compute", wall=1.0, cpu=1.0, comp_cpu=1.0, aff=(2,), **kw):
    r = {"victim": "v", "condition": cond, "wall": wall, "cpu": cpu, "throttled": 0}
    if cond != "none":
        r["competitor"] = {"cpu": comp_cpu, "rate": 1.0, "affinity": list(aff)}
    r.update(kw)
    return r


def records(walls, rounds=8, noise=0.0, drift=0.0, comp_cpu=1.0, seed=1):
    """Synthetic runs: walls {condition: wall}, each round scaled by a drift."""
    rng = random.Random(seed)
    out = []
    for r in range(rounds):
        scale = 1 + drift * r
        for c, w in walls.items():
            x = rec(c, wall=w * scale * (1 + rng.uniform(-noise, noise)), comp_cpu=comp_cpu)
            x["cpu"] = x["wall"]
            x.update(round=r, out=200e6, life=x["wall"] + 0.01, hash="h")
            if c == "twin":
                x["competitor"]["rate"] = 200e6 / x["wall"]
            x["why"] = fp.screen(x, 0, CORES)
            x["valid"] = not x["why"]
            out.append(x)
    return out


CAL = {
    "stressors": {
        "compute": {"valid": True, "rate": 1.0},
        "cache": {"valid": True, "rate": 1.0},
        "stream": {"valid": True, "rate": 1.0},
        "twin": {"valid": True},
    }
}
ALL = ["none", "compute", "cache", "stream", "twin"]


class Screens(unittest.TestCase):
    def test_clean_run(self):
        self.assertEqual(fp.screen(rec(), 0, CORES), [])
        self.assertEqual(fp.screen(rec("none"), 0, CORES), [])

    def test_competitor_at_half_a_cpu(self):
        why = fp.screen(rec(comp_cpu=0.5), 0, CORES)
        self.assertEqual(len(why), 1)
        self.assertIn("less than 0.9 of a CPU", why[0])

    def test_victim_waited(self):
        self.assertIn("waited to be scheduled", fp.screen(rec(cpu=0.9), 0, CORES)[0])
        self.assertIn("run queue", fp.screen(rec(runq_s=0.1), 0, CORES)[0])
        self.assertEqual(fp.screen(rec(runq_s=0.01), 0, CORES), [])

    def test_throttled(self):
        self.assertIn("throttled", fp.screen(rec(throttled=2), 0, CORES)[0])

    def test_shared_core(self):
        # the victim's sibling (1) is the victim's core: never a competitor's place
        self.assertIn("shares the victim's physical core", fp.screen(rec(aff=(1,)), 0, CORES)[0])
        self.assertIn("unreadable", fp.screen(rec(aff=()), 0, CORES)[0])
        # ... except in the labelled sibling arm, where it must be
        self.assertEqual(fp.screen(rec("compute-sib", aff=(1,)), 0, CORES), [])
        self.assertTrue(fp.screen(rec("compute-sib", aff=(2,)), 0, CORES))

    def test_placement(self):
        self.assertEqual(fp.placement(CORES), ((0, 2, 1), None))
        self.assertEqual(fp.placement([[0], [1], [2], [3]]), ((0, 1, None), None))
        place, why = fp.placement([[0, 1]])
        self.assertIsNone(place)
        self.assertIn("1 physical core", why)
        place, why = fp.placement(None)
        self.assertIsNone(place)
        self.assertIn("unreadable", why)

    def test_topology(self):
        with tempfile.TemporaryDirectory() as d:
            for c, sib in ((0, "0-1"), (1, "0-1"), (2, "2-3"), (3, "2-3")):
                os.makedirs(f"{d}/cpu{c}/topology")
                Path(f"{d}/cpu{c}/topology/thread_siblings_list").write_text(sib + "\n")
            self.assertEqual(fp.topology({0, 1, 2, 3}, base=d), CORES)
            self.assertEqual(fp.topology({0, 1}, base=d), [[0, 1]])
            self.assertIsNone(fp.topology({0, 7}, base=d))

    def test_inconclusive_when_every_competitor_is_starved(self):
        res, _ = fp.analyse(
            records({"none": 1, "compute": 1, "cache": 1.3}, comp_cpu=0.5),
            ["v"],
            ["none", "compute", "cache"],
            CAL,
        )
        for c in ("compute", "cache"):
            v = res["v"]["conditions"][c]["verdict"]
            self.assertTrue(v.startswith("INCONCLUSIVE"), v)
            self.assertIn("less than 0.9 of a CPU", v)

    def test_inconclusive_global_and_stressor(self):
        cal = {
            "stressors": {
                **CAL["stressors"],
                "cache": {"valid": False, "check": "it would be a memory streamer"},
            }
        }
        res, _ = fp.analyse(
            records({"none": 1, "compute": 1, "cache": 1.3}),
            ["v"],
            ["none", "compute", "cache"],
            cal,
        )
        self.assertIn("failed its calibration check", res["v"]["conditions"]["cache"]["verdict"])
        self.assertFalse(res["v"]["conditions"]["compute"]["verdict"].startswith("INCONCLUSIVE"))
        res, _ = fp.analyse(
            records({"none": 1, "compute": 1}),
            ["v"],
            ["none", "compute"],
            CAL,
            global_why="1 physical core",
        )
        self.assertEqual(
            res["v"]["conditions"]["compute"]["verdict"], "INCONCLUSIVE: 1 physical core"
        )

    def test_too_few_valid_rounds(self):
        rs = records({"none": 1, "compute": 1, "cache": 1.3})
        for x in rs:
            if x["round"] >= 2 and x["condition"] == "none":
                x["valid"], x["why"] = False, ["the victim waited to be scheduled (0.5)"]
        res, _ = fp.analyse(rs, ["v"], ["none", "compute", "cache"], CAL)
        v = res["v"]["conditions"]["cache"]["verdict"]
        self.assertIn("only 2 of 8 rounds", v)
        self.assertIn("most often: the victim waited to be scheduled", v)


class Statistics(unittest.TestCase):
    def test_paired_by_round(self):
        # every run of a round 0-70% slower (drift): paired ratios still exact
        rs = records({"none": 1.0, "compute": 1.02, "cache": 1.2}, drift=0.1)
        res, _ = fp.analyse(rs, ["v"], ["none", "compute", "cache"], CAL)
        c = res["v"]["conditions"]["cache"]
        self.assertAlmostEqual(c["slowdown"][0], 1.2, places=3)
        self.assertAlmostEqual(c["slowdown"][1], c["slowdown"][2], places=3)
        self.assertAlmostEqual(c["excess"][0], 1.2 / 1.02, places=3)

    def test_matches_ratio_ci(self):
        rs = records({"none": 1.0, "compute": 1.0, "stream": 1.1}, noise=0.05, seed=7)
        res, _ = fp.analyse(rs, ["v"], ["none", "compute", "stream"], CAL)
        by = {}
        for x in rs:
            by.setdefault(x["round"], {})[x["condition"]] = x["wall"]
        want = ratio_ci([r["stream"] / r["none"] for r in by.values()])
        self.assertEqual(res["v"]["conditions"]["stream"]["slowdown"], want)
        want = ratio_ci([r["stream"] / r["compute"] for r in by.values()])
        self.assertEqual(res["v"]["conditions"]["stream"]["excess"], want)
        lo, hi = want[1], want[2]
        self.assertLess(lo, want[0])
        self.assertGreater(hi, want[0])

    def test_twin_aggregate(self):
        # each copy at 1/1.25 of solo speed: together 2/1.25 = 1.6x one copy
        rs = records({"none": 1.0, "compute": 1.0, "twin": 1.25})
        res, _ = fp.analyse(rs, ["v"], ["none", "compute", "twin"], CAL)
        t = res["v"]["conditions"]["twin"]
        self.assertAlmostEqual(t["aggregate"][0], 1.6, places=3)
        self.assertIn("two copies decode 1.60x", t["verdict"])

    def test_across_victims(self):
        a = records({"none": 1.0, "compute": 1.0, "cache": 1.1})
        b = records({"none": 0.8, "compute": 0.8, "cache": 1.0})
        for x in b:
            x["victim"] = "w"
        _, across = fp.analyse(a + b, ["v", "w"], ["none", "compute", "cache"], CAL)
        self.assertAlmostEqual(across["w"]["solo_throughput"][0], 1.25, places=3)
        self.assertAlmostEqual(across["w"]["cache"][0], 1.25 / 1.1, places=3)


def curve(c, l2=4.0, l3=14.0, dram=100.0, sizes=None):
    """A latency curve: L2 to 1 MB, then a cache of `c` bytes, then random misses."""
    sizes = sizes or [int(256 * 1024 * 2 ** (k / 2)) for k in range(17)]
    out = []
    for s in sizes:
        if s <= 1 << 20:
            ns = l2
        elif s <= c:
            ns = l3
        else:
            ns = l3 + (dram - l3) * (1 - c / s)
        out.append({"bytes": s, "ns": ns})
    return out


class CEff(unittest.TestCase):
    def test_knee(self):
        for mbs in (4, 8, 16, 24, 32):
            c, d = fp.c_eff(curve(mbs << 20))
            self.assertIsNotNone(c, d)
            self.assertGreater(c, (mbs << 20) / 1.5, (mbs, c / 2**20))
            self.assertLess(c, (mbs << 20) * 1.5, (mbs, c / 2**20))

    def test_outlier_inside_the_cache(self):
        sw = curve(16 << 20)
        sw[6]["ns"] = 300.0  # a preempted measurement at 2 MB, slower than memory
        c, _ = fp.c_eff(sw)
        self.assertGreater(c, 8 << 20)

    def test_no_knee(self):
        c, d = fp.c_eff(curve(1 << 30))
        self.assertIsNone(c)
        self.assertIn("no knee", d["reason"])
        c, d = fp.c_eff(curve(16 << 20) + [{"bytes": 1 << 27, "ns": None, "preempted": True}])
        self.assertIsNone(c)
        self.assertIn("preempted", d["reason"])


class Checks(unittest.TestCase):
    def test_cache(self):
        self.assertTrue(fp.validate_cache(1.3, 1.5, 19.0)[0])
        ok, why = fp.validate_cache(1.3, 12.0, 19.0)
        self.assertFalse(ok)
        self.assertIn("memory streamer", why)
        self.assertFalse(fp.validate_cache(10.0, 11.0, 12.0)[0])
        self.assertFalse(fp.validate_cache(None, 1.0, 2.0)[0])

    def test_stream(self):
        self.assertTrue(fp.validate_stream(80.0, 10.0, 128 << 20, 16 << 20)[0])
        self.assertFalse(fp.validate_stream(80.0, 10.0, 128 << 20, 100 << 20)[0])  # footprint
        self.assertFalse(fp.validate_stream(11.0, 10.0, 128 << 20, 16 << 20)[0])  # not apart
        self.assertFalse(fp.validate_stream(80.0, 0.5, 128 << 20, 16 << 20)[0])  # too slow
        self.assertFalse(fp.validate_stream(80.0, 10.0, 128 << 20, None)[0])


# what a verdict must never name as the cause
CAUSES = re.compile(
    r"\b(LLC|L[123]|DRAM|bandwidth|channel|queue|prefetch|TLB|because|due to|caused by|contention)\b",
    re.I,
)


class Wording(unittest.TestCase):
    def test_never_a_cause(self):
        cis = [
            [1.0, 0.99, 1.01],
            [1.18, 1.15, 1.21],
            [1.02, 1.01, 1.03],
            [0.95, 0.93, 0.97],
            [1.5, 0.8, 2.5],
        ]
        for cond in ("compute", "cache", "stream", "twin", "compute-sib", "cache-sib"):
            for slow in cis:
                for ex in [None, *cis]:
                    for comp in (None, [1.03, 1.0, 1.06]):
                        v = fp.verdict(cond, slow, ex, comp, 8, 8)
                        self.assertNotRegex(v, CAUSES)

    def test_lines(self):
        v = fp.verdict("cache", [1.18, 1.15, 1.21], [1.15, 1.12, 1.18], [1.03, 1.01, 1.05], 8, 8)
        self.assertEqual(
            v,
            "slows 1.18x (1.15-1.21) beside the cache stressor, 1.03x beside compute: "
            "sensitive to a co-runner's cache footprint, beyond the compute control "
            "(1.15x (1.12-1.18) over it)",
        )
        v = fp.verdict("stream", [1.04, 1.0, 1.08], [1.01, 0.98, 1.04], [1.03, 1.0, 1.06], 8, 8)
        self.assertIn("not distinguishable from the compute control at this run's precision", v)
        v = fp.verdict("stream", [1.04, 1.03, 1.05], [1.01, 1.005, 1.015], [1.03, 1.0, 1.06], 8, 8)
        self.assertIn("a small excess", v)
        self.assertTrue(
            fp.verdict("cache", [1.2, 1.1, 1.3], None, None, 2, 8).startswith("INCONCLUSIVE")
        )


# a stand-in for examples/stress.rs: answers the protocol, ignores its stdin closing and
# SIGTERM (so only the process group's SIGKILL ends it)
FAKE = textwrap.dedent(
    """\
    #!/usr/bin/env python3
    import json, signal, sys, time
    signal.signal(signal.SIGTERM, signal.SIG_IGN)
    print(json.dumps({"ready": sys.argv[1], "bytes": 0, "unit": "iterations"}), flush=True)
    t0 = time.time()
    while True:
        time.sleep(0.01)
    """
)
DRIVER = textwrap.dedent(
    """\
    import sys, time
    sys.path.insert(0, {here!r})
    import fingerprint as fp
    fp.install_handlers()
    s = fp.Stressor({exe!r}, "compute", {cpu}, timeout=60).start()
    print(s.pid, flush=True)
    time.sleep(60)
    """
)


def gone(pid):
    """The process is gone, or a zombie (killed, not yet reaped by an init that doesn't reap)."""
    try:
        state = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[0]
    except (OSError, IndexError):
        return True
    return state in ("Z", "X")


def stress_binary():
    for p in (os.environ.get("FINGERPRINT_STRESS"), HERE.parent / "target/release/examples/stress"):
        if p and os.access(p, os.X_OK):
            return str(p)
    return None


class Cleanup(unittest.TestCase):
    def driver(self, exe):
        cpu = min(os.sched_getaffinity(0))
        code = DRIVER.format(here=str(HERE), exe=exe, cpu=cpu)
        d = subprocess.Popen([sys.executable, "-c", code], stdout=subprocess.PIPE, text=True)
        line = d.stdout.readline()
        self.assertTrue(line.strip().isdigit(), line)
        pid = int(line)
        self.assertFalse(gone(pid))
        return d, pid

    def wait_gone(self, pid, seconds=10):
        t = time.time() + seconds
        while time.time() < t and not gone(pid):
            time.sleep(0.05)
        return gone(pid)

    def fake(self, tmp):
        p = Path(tmp) / "fake-stress"
        p.write_text(FAKE)
        p.chmod(p.stat().st_mode | stat.S_IXUSR)
        return str(p)

    def test_signals_kill_the_competitor(self):
        with tempfile.TemporaryDirectory() as tmp:
            for exe in filter(None, (stress_binary(), self.fake(tmp))):
                for sig in (signal.SIGTERM, signal.SIGINT):
                    with self.subTest(exe=exe, sig=sig):
                        d, pid = self.driver(exe)
                        d.send_signal(sig)
                        d.wait(15)
                        d.stdout.close()
                        self.assertEqual(d.returncode, 128 + sig)
                        self.assertTrue(self.wait_gone(pid), f"{exe} survived {sig!r}")

    def test_stressor_exits_when_the_orchestrator_is_killed(self):
        exe = stress_binary()
        if not exe:
            self.skipTest(
                "no target/release/examples/stress (cargo build --release --example stress)"
            )
        d, pid = self.driver(exe)
        d.kill()  # (no handler runs: the stressor's stdin closes)
        d.wait(15)
        d.stdout.close()
        self.assertTrue(self.wait_gone(pid), "the stressor outlived a SIGKILLed orchestrator")


if __name__ == "__main__":
    unittest.main()
