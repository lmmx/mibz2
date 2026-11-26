#!/usr/bin/env python3
"""bench-kernel-count's report (bench/kernel_count.py) on synthetic records: the paired
ratios and their intervals, the API's built configuration and its twin, COMPLETE, INCOMPLETE
and INVALID."""

from __future__ import annotations

import io
import math
import os
import random
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kernel_count as kc  # noqa: E402


def built(cell, threads, lazy, section="api-built", rnd=0):
    return {
        "section": section,
        "round": rnd,
        "cell": cell,
        "kind": "built",
        "threads": threads,
        "window": 4 * threads,
        "lazy": lazy,
        "service": False,
        "pipelined": False,
        "exit": 0,
    }


def synth(rounds=12, reps=3, lazy_gain=0.9, seed=1, drop=None, fail=None):
    rng = random.Random(seed)
    recs = []
    for k in kc.LAZY_KS:
        recs.append(built(kc.lazy(k), k, True, section="probe", rnd=-1) | {"as_asked": True})
    for k in kc.EAGER_KS:
        recs.append(built(kc.eager(k), k, False, section="probe", rnd=-1) | {"as_asked": True})
    for rnd in range(rounds):
        noise = rng.uniform(0.95, 1.05)  # shared by the round: pairing removes it
        cells = {}
        for k in kc.LAZY_KS:
            cells[kc.lazy(k)] = (1.4 / k * lazy_gain + 0.05) * noise
        for k in kc.EAGER_KS:
            cells[kc.eager(k)] = (1.4 / k + 0.05) * noise
        for t in kc.API_TS:
            n = min(t, 7) if t >= 10 else t
            cells[kc.api(t)] = cells[kc.lazy(n) if n >= 8 or t >= 10 else kc.eager(n)]
            recs.append(built(kc.api(t), n, n >= 8 or t >= 10, rnd=rnd))
        for cell, wall in cells.items():
            if drop and (cell, rnd) == drop:
                continue
            for rep in range(reps):
                ok = not (fail and (cell, rnd, rep) == fail)
                recs.append(
                    {
                        "section": "api" if cell.startswith("api") else "explicit",
                        "round": rnd,
                        "rep": rep,
                        "cell": cell,
                        "ok": ok,
                        "wall": wall * rng.uniform(0.999, 1.001),
                        "cpu": wall * 3,
                        "rss_kb": 50000 if "lazy" in cell else 120000,
                        "out": 838800784,
                        "hash": "abc",
                        "others_busy": 0.1,
                        "err": "" if ok else "boom",
                    }
                )
    return recs


class T(unittest.TestCase):
    def run_report(self, recs, rounds=12, reps=3):
        out = io.StringIO()
        status = kc.report(recs, rounds, reps, out=out)
        return status, out.getvalue()

    def test_ratio_stats(self):
        s = kc.ratio_stats([(0.9, 1.0)] * 5)
        self.assertAlmostEqual(s["gm"], 0.9)
        self.assertAlmostEqual(s["lo"], 0.9)
        self.assertEqual(s["a_lower"], 5)
        s = kc.ratio_stats([(1.1, 1.0), (0.9, 1.0)])
        self.assertAlmostEqual(s["gm"], math.sqrt(0.99))
        self.assertLess(s["lo"], 1.0)
        self.assertGreater(s["hi"], 1.0)
        self.assertEqual(kc.ratio_stats([])["n"], 0)

    def test_complete(self):
        status, text = self.run_report(synth())
        self.assertEqual(status, "COMPLETE", text)
        self.assertIn("decides nothing", text)
        # k = 7: lazy (0.9 * 0.2 + 0.05) / eager (0.2 + 0.05)
        want = (1.4 / 7 * 0.9 + 0.05) / (1.4 / 7 + 0.05)
        row = next(
            line for line in text.splitlines() if line.startswith("| 7 | ") and "/" not in line[:8]
        )
        self.assertIn(f"{want:.3f}", row)
        # the API at 12 asked built 7 lazy workers, window 28, and is paired with lazy-k07
        api12 = next(line for line in text.splitlines() if line.startswith("| 12 | "))
        self.assertIn("7 / 28 / lazy", api12)
        self.assertIn("vs lazy-k07", api12)
        self.assertIn("1.000 [", api12)

    def test_incomplete(self):
        status, text = self.run_report(synth(drop=("lazy-k05", 3)))
        self.assertEqual(status, "INCOMPLETE")
        self.assertIn("lazy-k05: rounds without all 3 decodes: [3]", text)

    def test_invalid_failed_decode(self):
        status, text = self.run_report(synth(fail=("eager-k02", 4, 1)))
        self.assertEqual(status, "INVALID")
        self.assertIn("decodes failed", text)

    def test_invalid_outputs_differ(self):
        recs = synth()
        recs[-1]["hash"] = "other"
        status, text = self.run_report(recs)
        self.assertEqual(status, "INVALID")
        self.assertIn("outputs differ", text)

    def test_invalid_probe(self):
        recs = synth()
        recs[0]["as_asked"] = False
        status, _ = self.run_report(recs)
        self.assertEqual(status, "INVALID")

    def test_two_configurations(self):
        recs = synth()
        recs.append(built(kc.api(9), 6, True, rnd=5))
        status, text = self.run_report(recs)
        self.assertEqual(status, "INCOMPLETE")
        self.assertIn("more than one configuration", text)


if __name__ == "__main__":
    unittest.main()
