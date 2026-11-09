"""bench/run_fed_leader.py's VALID, on synthetic runs: a clean set passes, and each failure
it must refuse is refused (a missing round, a failed child, a different output, a fallback,
a control that took records, a non-finite interval).

    python3 bench/test_run_fed_leader.py
"""

import copy
import json
import os
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import run_fed_leader as rfl  # noqa: E402

ROUNDS = 3


def clean():
    recs = []
    for rnd in range(ROUNDS):
        for i, arm in enumerate([rfl.REF] + rfl.ARMS):
            rec = {
                "round": rnd,
                "threads": 9,
                "arm": arm,
                "exit": 0,
                "err": "",
                "wall": 1.0 + 0.01 * i + 0.003 * rnd,
                "cpu": 3.0 + 0.02 * i + 0.001 * rnd,
                "hash": "abc",
                "out": 100,
            }
            if "runfed" in arm:
                records = 5 if arm in rfl.RECORD_ARMS else 0
                rec["phases"] = (
                    f"runs=10 team_records={records} team_column={10 - records} fallbacks=0 "
                    f"lead_front_ns=1000000 rule_blocks=4 rule_front_ns=800000 rule_sort_ns=0 "
                    f"rows=1000 stores=1500 records=70"
                )
            recs.append(rec)
    return recs


def run(recs):
    with tempfile.TemporaryDirectory() as d:
        p = os.path.join(d, "runs.jsonl")
        with open(p, "w") as f:
            for r in recs:
                f.write(json.dumps(r) + "\n")
        out = os.path.join(d, "rfl")
        r = subprocess.run(
            [sys.executable, "-I", os.path.join(HERE, "run_fed_leader.py"), "report", p,
             "--rounds", str(ROUNDS), "--out", out],
            capture_output=True,
            text=True,
        )
        raw = json.load(open(out + ".json"))
        return r.returncode, r.stdout, raw


class Valid(unittest.TestCase):
    def test_clean_passes_and_archives_raw(self):
        code, out, raw = run(clean())
        self.assertEqual(code, 0, out)
        self.assertIn("VALID", out)
        self.assertEqual(len(raw["raw"][rfl.PRIMARY]), ROUNDS)

    def refused(self, recs, what):
        code, out, raw = run(recs)
        self.assertEqual(code, 2, out)
        self.assertIn("INVALID", out)
        self.assertTrue(raw["invalid"])
        return out

    def test_missing_round(self):
        recs = [r for r in clean() if not (r["arm"] == rfl.PRIMARY and r["round"] == 1)]
        self.refused(recs, "a missing round")

    def test_failed_child(self):
        for key, val in (("exit", 1), ("exit", None), ("err", "boom"), ("broken", 1)):
            recs = clean()
            recs[3][key] = val
            self.refused(recs, key)

    def test_different_output(self):
        recs = clean()
        next(r for r in recs if r["arm"] == rfl.PRIMARY)["hash"] = "zzz"
        self.refused(recs, "a hash")

    def test_fallback_and_controls(self):
        recs = clean()
        r = next(r for r in recs if r["arm"] == rfl.PRIMARY)
        r["phases"] = r["phases"].replace("fallbacks=0", "fallbacks=1")
        self.refused(recs, "a fallback")
        recs = clean()
        r = next(r for r in recs if r["arm"].endswith("-ctl"))
        r["phases"] = r["phases"].replace("team_records=0", "team_records=2")
        self.refused(recs, "a control with records")
        recs = clean()
        del next(r for r in recs if r["arm"].endswith("-dir"))["phases"]
        self.refused(recs, "no phases")

    def test_non_finite_interval(self):
        recs = clean()
        for r in recs:
            if r["arm"] == rfl.PRIMARY:
                r["wall"] = 0.0
        self.refused(recs, "a zero wall")

    def test_geo_ci_refuses(self):
        self.assertIsNone(rfl.geo_ci([1.0]))
        self.assertIsNone(rfl.geo_ci([1.0, float("nan")]))
        self.assertIsNone(rfl.geo_ci([1.0, 0.0]))
        self.assertIsNotNone(rfl.geo_ci([1.0, 1.1]))


if __name__ == "__main__":
    unittest.main()
