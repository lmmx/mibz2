"""bench/parscan_stream.py on synthetic runs: the verdict where the registration puts it
(10 threads judged, 6 and 8 supporting, the memory gate at 10, every other count reported),
and no verdict from incomplete or malformed records.

    python3 bench/test_parscan_stream.py
"""

import io
import json
import shutil
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import parscan_stream as ps  # noqa: E402
import test_parscan_verdict as tp  # noqa: E402


def records(step=None, mem=None, noise=0.01):
    return tp.records(cand="mibz2", threads=ps.THREADS, step=step, mem=mem, noise=noise)


class Verdict(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        (self.tmp / "env.json").write_text(json.dumps(tp.ENV))
        (self.tmp / "lscpu.txt").write_text(tp.LSCPU)

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def run_on(self, recs, *extra):
        p = self.tmp / "runs.jsonl"
        p.write_text("".join(json.dumps(r) + "\n" for r in recs))
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = ps.main(
                [str(p), "--env", str(self.tmp / "env.json"), "--lscpu", str(self.tmp / "lscpu.txt")]
                + list(extra)
            )
        return rc, buf.getvalue()

    def verdict(self, out):
        line = next(ln for ln in out.splitlines() if ln.startswith("verdict"))
        self.assertTrue(line.startswith("verdict (Decision 1, PARSCAN): "), line)
        return line.split(": ", 1)[1]

    def test_adopts(self):
        rc, out = self.run_on(records(step={10: 0.93}))
        self.assertEqual(rc, 0, out)
        self.assertEqual(self.verdict(out), "ADOPTS")

    def test_ten_not_faster(self):
        rc, out = self.run_on(records(step={8: 0.9, 12: 0.9}))
        self.assertTrue(self.verdict(out).startswith("NOT ADOPTED"), out)

    def test_six_slower(self):
        rc, out = self.run_on(records(step={10: 0.9, 6: 1.05}))
        self.assertTrue(self.verdict(out).startswith("NOT ADOPTED"), out)

    def test_eight_slower(self):
        rc, out = self.run_on(records(step={10: 0.9, 8: 1.05}))
        self.assertTrue(self.verdict(out).startswith("NOT ADOPTED"), out)

    def test_reported_counts_decide_nothing(self):
        rc, out = self.run_on(records(step={10: 0.9, 1: 1.2, 2: 1.2, 4: 1.2, 16: 1.2, 20: 1.2}))
        self.assertEqual(self.verdict(out), "ADOPTS")

    def test_memory_gate_at_ten(self):
        rc, out = self.run_on(records(step={10: 0.9}, mem={10: 1.11}))
        self.assertTrue(self.verdict(out).startswith("HELD ON MEMORY"), out)
        rc, out = self.run_on(records(step={10: 0.9}, mem={20: 1.5, 8: 1.3}))
        self.assertEqual(self.verdict(out), "ADOPTS")

    def test_missing_record_is_void(self):
        recs = records(step={10: 0.9})
        rc, out = self.run_on(recs[:-1])
        self.assertEqual(rc, 1, out)
        self.assertIn("VOID", out)
        self.assertFalse(any(ln.startswith("verdict") for ln in out.splitlines()))

    def test_wrong_arm_is_void(self):
        recs = records(step={10: 0.9})
        for r in recs:
            if r["arm"] == "mibz2":
                r["arm"] = "mibz2-parscan"
        rc, out = self.run_on(recs)
        self.assertEqual(rc, 1, out)
        self.assertIn("VOID", out)


    def one_round(self, step=None):
        return [r for r in records(step=step) if r["round"] == 0]

    def test_preflight_passes(self):
        rc, out = self.run_on(self.one_round(), "--preflight")
        self.assertEqual(rc, 0, out)
        self.assertIn("preflight: OK", out)
        self.assertFalse(any(ln.startswith("verdict") for ln in out.splitlines()))

    def test_preflight_fails_on_a_wrong_output(self):
        recs = self.one_round()
        recs[3]["hash"] = "fedcba9876543210"
        rc, out = self.run_on(recs, "--preflight")
        self.assertEqual(rc, 1, out)
        self.assertIn("preflight: FAILED", out)

    def test_preflight_fails_on_a_missing_cell(self):
        rc, out = self.run_on(self.one_round()[:-1], "--preflight")
        self.assertEqual(rc, 1, out)

    def test_preflight_fails_off_the_i9(self):
        (self.tmp / "lscpu.txt").write_text("Model name: AMD EPYC\n")
        rc, out = self.run_on(self.one_round(), "--preflight")
        self.assertEqual(rc, 1, out)


if __name__ == "__main__":
    unittest.main()
