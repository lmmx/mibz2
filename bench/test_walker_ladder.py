"""Tests of the walker ladder's validity rules (bench/walker_ladder.py: G0), on synthetic
records: no perf, no timing.

    python3 -m unittest bench/test_walker_ladder.py
"""

import json
import os
import stat
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(__file__))
import walker_ladder as wl  # noqa: E402


def counts(events, value=1000.0, pct=100.0, status="counted", run_time=1000):
    return {e: {"value": value, "status": status, "run_time": run_time, "running_pct": pct}
            for e in events}


def rec(kind="one", group="exec", rung="A", lanes=32, rnd=0, cpu=0, pct=100.0,
        window=(0, 1000), value=1000.0, checked=True, controlled=True, reps=10):
    ev = wl.FIXED + wl.GROUPS[group] if group else []
    out = {"rows": 100, "ns_a_row": 2.0, "checked": checked, "controlled": controlled,
           "window_start_ns": window[0], "window_end_ns": window[1]}
    return wl.record(kind, rnd, rung, lanes, group, cpu, out, counts(ev, value, pct), "", reps)


def clean_four(group="exec", pct=100.0, offsets=(0, 0, 0, 0)):
    return [rec("four", group, "P", 32, cpu=c, pct=pct, window=(o, o + 1000))
            for c, o in enumerate(offsets)]


class Validity(unittest.TestCase):
    def test_clean_capture_passes(self):
        recs = [rec(), rec(group="mlp")] + clean_four()
        recs.append(rec("overhead", reps=0, value=1.0))
        self.assertEqual(wl.validity(recs), [])

    def test_multiplexed_four_core_capture_is_rejected(self):
        recs = [rec()] + clean_four(pct=80.0)
        bad = wl.validity(recs)
        self.assertTrue(any("four" in b and "multiplexed" in b for b in bad), bad)

    def test_one_multiplexed_four_core_copy_is_rejected(self):
        fc = clean_four()
        fc[2]["counts"]["uops_issued.any"]["running_pct"] = 99.0
        bad = wl.validity([rec()] + fc)
        self.assertTrue(any("cpu 2" in b and "multiplexed" in b for b in bad), bad)

    def test_multiplexed_overhead_run_is_rejected(self):
        bad = wl.validity([rec(), rec("overhead", pct=60.0, reps=0, value=1.0)])
        self.assertTrue(any(b.startswith("overhead") and "multiplexed" in b for b in bad), bad)

    def test_multiplexed_one_core_run_is_rejected(self):
        bad = wl.validity([rec(pct=82.0)])
        self.assertTrue(any("multiplexed" in b for b in bad), bad)

    def test_uncounted_event_is_rejected(self):
        r = rec()
        r["counts"]["uops_issued.any"] = {"value": None, "status": "<not counted>",
                                          "run_time": 0, "running_pct": 0.0}
        bad = wl.validity([r])
        self.assertTrue(any("<not counted>" in b for b in bad), bad)

    def test_missing_event_is_rejected(self):
        r = rec()
        del r["counts"]["uops_retired.retire_slots"]
        bad = wl.validity([r])
        self.assertTrue(any("retire_slots missing" in b for b in bad), bad)

    def test_zero_count_is_a_count(self):
        r = rec(value=0.0)
        self.assertEqual(wl.validity([r]), [])

    def test_unchecked_run_is_rejected(self):
        bad = wl.validity([rec(checked=False)])
        self.assertTrue(any("unchecked" in b for b in bad), bad)

    def test_uncontrolled_counts_are_rejected(self):
        bad = wl.validity([rec(controlled=False)])
        self.assertTrue(any("control FIFO" in b for b in bad), bad)

    def test_four_core_windows_must_overlap(self):
        bad = wl.validity([rec()] + clean_four(offsets=(0, 0, 0, 200)))
        self.assertTrue(any("overlap" in b for b in bad), bad)

    def test_four_core_set_must_have_four_copies(self):
        bad = wl.validity([rec()] + clean_four()[:3])
        self.assertTrue(any("windows missing" in b for b in bad), bad)

    def test_large_control_overhead_is_rejected(self):
        bad = wl.validity([rec(value=1000.0), rec("overhead", reps=0, value=100.0)])
        self.assertTrue(any("overhead" in b and "more than" in b for b in bad), bad)

    def test_result_vector_growth_is_rejected(self):
        r = rec()
        r["capacity_unchanged"] = False
        bad = wl.validity([r])
        self.assertTrue(any("grew" in b for b in bad), bad)

    def test_timing_only_records_need_no_counts(self):
        r = rec(group=None, controlled=False)
        self.assertEqual(wl.validity([r]), [])


class ControlComparison(unittest.TestCase):
    """report 146: a control-only count comparable to a window's count is flagged, never
    subtracted"""

    def recs(self, window, control, event="cycle_activity.stalls_l3_miss", group="stalls_l2"):
        w = rec(group=group, rung="P", lanes=32)
        w["counts"][event]["value"] = window
        o = rec("overhead", group=group, rung="A", lanes=32, reps=0)
        o["counts"][event]["value"] = control
        return [w, o], f"P32 {group} {event}"

    def test_the_probes_stalls_l3_miss_is_comparable(self):
        recs, key = self.recs(28608.0, 17636.0)
        t = wl.control_comparison(recs, [("P", 32)])
        self.assertEqual(t[key]["state"], "comparable")
        self.assertAlmostEqual(t[key]["control_over_window"], 17636 / 28608)
        self.assertEqual(t[key]["window"], 28608.0, "the window's count is kept, not subtracted")

    def test_small_control_is_small(self):
        recs, key = self.recs(1_000_000.0, 500.0)
        self.assertEqual(wl.control_comparison(recs, [("P", 32)])[key]["state"], "small")

    def test_both_zero_is_zero(self):
        recs, key = self.recs(0.0, 0.0, "sw_prefetch_access.t0", "pf")
        self.assertEqual(wl.control_comparison(recs, [("P", 32)])[key]["state"], "zero")

    def test_control_against_zero_window_is_comparable(self):
        recs, key = self.recs(0.0, 5.0, "sw_prefetch_access.t0", "pf")
        self.assertEqual(wl.control_comparison(recs, [("P", 32)])[key]["state"], "comparable")


class Parse(unittest.TestCase):
    def test_perf_csv(self):
        err = ("Events disabled\nEvents enabled\n1234,,cycles,100,100.00,,\n"
               "<not counted>,,uops_issued.any,0,0.00,,\n"
               "77,,instructions:u,100,82.00,,\nnoise line\n")
        c = wl.parse_perf(err)
        self.assertEqual(c["cycles"], {"value": 1234.0, "status": "counted", "run_time": 100,
                                       "running_pct": 100.0})
        self.assertEqual(c["uops_issued.any"]["status"], "<not counted>")
        self.assertEqual(c["instructions"]["running_pct"], 82.0)
        self.assertNotIn("noise line", c)


class Groups(unittest.TestCase):
    def test_groups_fit_four_programmable_counters_with_cycles(self):
        for g, evs in wl.GROUPS.items():
            self.assertLessEqual(len(evs), 3, g)

    def test_decision_pairs_share_a_group(self):
        self.assertIn("l1d_pend_miss.pending", wl.GROUPS[wl.NEEDS["D4"]])
        self.assertIn("l1d_pend_miss.pending_cycles", wl.GROUPS[wl.NEEDS["D4"]])
        self.assertIn("l1d_pend_miss.fb_full", wl.GROUPS[wl.NEEDS["D4"]])
        self.assertIn("uops_issued.any", wl.GROUPS[wl.NEEDS["D3"]])


# the i9 probe of 2026-11-02 00:06 UTC (a245a45): every event, every run, exactly this shape
FAILED_I9 = ("Events disabled\n<not counted>,,cycles,0,100.00,,\n"
             "<not counted>,,instructions,0,100.00,,\n"
             "<not counted>,,uops_issued.any,0,100.00,,\n"
             "<not counted>,,uops_retired.retire_slots,0,100.00,,\n"
             "<not counted>,,idq_uops_not_delivered.core,0,100.00,,\n")
GOOD = ("Events disabled\nEvents enabled\n9000,,cycles,5000,100.00,,\n"
        "8000,,instructions,5000,100.00,,\n7000,,uops_issued.any,5000,100.00,,\n"
        "0,,uops_retired.retire_slots,5000,100.00,,\n10,,idq_uops_not_delivered.core,5000,100.00,,\n")
OUT = {"rows": 100, "ns_a_row": 1.0, "checked": True, "controlled": True,
       "window_start_ns": 0, "window_end_ns": 10}
EV = wl.FIXED + wl.GROUPS["exec"]


def probe_rec(raw, controlled=True, reps=20):
    out = dict(OUT, controlled=controlled)
    return wl.record("probe", 0, "A", 32, None, 0, out, wl.parse_perf(raw), raw, reps, EV)


class FailedProbe(unittest.TestCase):
    """report 140: perf's 100.00% beside <not counted> and zero run time is no measurement"""

    def test_the_i9_failure_is_invalid_and_unassessed(self):
        for controlled in (False, True):
            pr = wl.probe_pair(probe_rec(FAILED_I9, controlled), probe_rec(FAILED_I9, controlled, 0))
            self.assertFalse(pr["valid"])
            self.assertEqual(pr["multiplexed"], "unassessed")
            self.assertTrue(all(st == "<not counted>" for st in pr["states"].values()))

    def test_good_window_with_failed_control_is_rejected(self):
        pr = wl.probe_pair(probe_rec(GOOD), probe_rec(FAILED_I9, reps=0))
        self.assertFalse(pr["valid"])
        self.assertTrue(any("<not counted>" in f for f in pr["failures"]))

    def test_good_pair_is_valid_and_zero_count_that_ran_is_a_count(self):
        pr = wl.probe_pair(probe_rec(GOOD), probe_rec(GOOD, reps=0))
        self.assertTrue(pr["valid"], pr["failures"])
        self.assertIs(pr["multiplexed"], False)
        self.assertEqual(pr["states"]["uops_retired.retire_slots"], "valid")

    def test_counted_with_zero_run_time_is_invalid(self):
        raw = GOOD.replace("9000,,cycles,5000", "9000,,cycles,0")
        self.assertEqual(wl.event_state(wl.parse_perf(raw)["cycles"]), "never ran (zero run time)")
        self.assertFalse(wl.probe_pair(probe_rec(raw), probe_rec(GOOD, reps=0))["valid"])

    def test_uncontrolled_good_counts_are_rejected(self):
        pr = wl.probe_pair(probe_rec(GOOD, controlled=False), probe_rec(GOOD, reps=0))
        self.assertFalse(pr["valid"])


def script(d, name, body):
    p = os.path.join(d, name)
    with open(p, "w") as f:
        f.write(body)
    os.chmod(p, os.stat(p).st_mode | stat.S_IEXEC)
    return p


class Transport(unittest.TestCase):
    """report 140: the real command constructor through an environment-filtering wrapper
    (sudo's default policy) reaches the workload with its FIFOs and barrier intact"""

    def run_through_filter(self, barrier):
        with tempfile.TemporaryDirectory() as d:
            strip = script(d, "strip", '#!/bin/sh\nexec env -i PATH="$PATH" "$@"\n')
            # a perf stand-in that runs what follows `--`, as perf stat does
            passperf = script(d, "passperf", "#!" + sys.executable + "\n"
                              "import os, sys\na = sys.argv[1:]\n"
                              "os.execvp(a[a.index('--') + 1], a[a.index('--') + 1:])\n")
            child = script(d, "child", "#!" + sys.executable + "\n"
                           "import json, os, sys\n"
                           "print(json.dumps({'argv': sys.argv[1:], 'env': sorted(k for k in os.environ if k.startswith('WALKER'))}))\n")
            saved = wl.PERF
            wl.PERF = [strip, passperf]
            try:
                cmd, env = wl.command("corpus.bz2", "A", 32, 20, None, ["cycles"], d,
                                      barrier=barrier, binary=child)
                env = dict(env, WALKER_CTL="stale", WALKER_BARRIER="1")
                p = subprocess.run(cmd, capture_output=True, text=True, env=env)
            finally:
                wl.PERF = saved
            self.assertEqual(p.returncode, 0, p.stderr)
            return json.loads(p.stdout), d

    def test_fifos_reach_the_child_through_the_filter(self):
        got, d = self.run_through_filter(False)
        a = got["argv"]
        self.assertEqual(a[a.index("--ctl") + 1], os.path.join(d, "ctl"))
        self.assertEqual(a[a.index("--ack") + 1], os.path.join(d, "ack"))
        self.assertNotIn("--barrier", a)
        self.assertEqual(got["env"], [], "the filter stripped every WALKER variable")

    def test_barrier_reaches_the_child_through_the_filter(self):
        got, _ = self.run_through_filter(True)
        self.assertIn("--barrier", got["argv"])
        self.assertIn("--ctl", got["argv"])

    def test_perf_is_told_the_same_fifos(self):
        with tempfile.TemporaryDirectory() as d:
            cmd, _ = wl.command("c.bz2", "A", 32, 20, 3, ["cycles"], d)
            ctl, ack = os.path.join(d, "ctl"), os.path.join(d, "ack")
            self.assertIn(f"fifo:{ctl},{ack}", cmd)
            self.assertEqual(cmd[cmd.index("-D") + 1], "-1")
            tail = cmd[cmd.index("--") + 1:]
            self.assertEqual(tail[:3], ["taskset", "-c", "3"])
            self.assertEqual(tail[tail.index("--ctl") + 1], ctl)


if __name__ == "__main__":
    unittest.main()
