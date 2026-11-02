"""Tests of the walker ladder's validity rules (bench/walker_ladder.py: G0), on synthetic
records: no perf, no timing.

    python3 -m unittest bench/test_walker_ladder.py
"""

import copy
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(__file__))
import walker_ladder as wl  # noqa: E402


def counts(events, value=1000.0, pct=100.0, status="counted"):
    return {e: {"value": value, "status": status, "running_pct": pct} for e in events}


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
                                          "running_pct": 0.0}
        bad = wl.validity([r])
        self.assertTrue(any("<not counted>" in b for b in bad), bad)

    def test_missing_event_is_rejected(self):
        r = rec()
        del r["counts"]["uops_retired.retire_slots"]
        bad = wl.validity([r])
        self.assertTrue(any("retire_slots None" in b for b in bad), bad)

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

    def test_timing_only_records_need_no_counts(self):
        r = rec(group=None, controlled=False)
        self.assertEqual(wl.validity([r]), [])


class Parse(unittest.TestCase):
    def test_perf_csv(self):
        err = ("Events disabled\nEvents enabled\n1234,,cycles,100,100.00,,\n"
               "<not counted>,,uops_issued.any,0,0.00,,\n"
               "77,,instructions:u,100,82.00,,\nnoise line\n")
        c = wl.parse_perf(err)
        self.assertEqual(c["cycles"], {"value": 1234.0, "status": "counted", "running_pct": 100.0})
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


if __name__ == "__main__":
    unittest.main()
