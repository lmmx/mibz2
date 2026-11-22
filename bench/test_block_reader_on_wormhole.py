"""bench/block_reader_on_wormhole.py on synthetic runs: Decision 2's verdict where the
registration puts it, its VOID checks, and the composition report deciding nothing.

    python3 bench/test_block_reader_on_wormhole.py
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
import block_reader_on_wormhole as bw  # noqa: E402
import test_parscan_c37 as tp  # noqa: E402


def records(step=None, mem=None, noise=0.01):
    return tp.records(cand="mibz2", threads=(1, 6, 8, 10), step=step, mem=mem, noise=noise)


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
            rc = bw.main(
                [
                    str(p),
                    "--env",
                    str(self.tmp / "env.json"),
                    "--lscpu",
                    str(self.tmp / "lscpu.txt"),
                ]
                + list(extra)
            )
        return rc, buf.getvalue()

    def verdict(self, out):
        line = next(ln for ln in out.splitlines() if ln.startswith("verdict"))
        self.assertTrue(line.startswith("verdict (Decision 2, BLOCK-READER): "), line)
        return line.split(": ", 1)[1]

    def composition(self, out):
        return next(ln for ln in out.splitlines() if ln.startswith("composition"))

    def test_adopts(self):
        rc, out = self.run_on(records(step={1: 0.97, 10: 0.93}))
        self.assertEqual(rc, 0, out)
        self.assertEqual(self.verdict(out), "ADOPTS")

    def test_one_thread_must_be_faster(self):
        rc, out = self.run_on(records(step={6: 0.9, 8: 0.9, 10: 0.9}))
        self.assertEqual(self.verdict(out), "NOT ADOPTED (1 threads not FASTER)")

    def test_slower_at_a_supporting_count(self):
        for t in (6, 8, 10):
            rc, out = self.run_on(records(step={1: 0.97, t: 1.05}))
            self.assertEqual(self.verdict(out), f"NOT ADOPTED (SLOWER at {t} threads)")

    def test_memory_at_ten(self):
        rc, out = self.run_on(records(step={1: 0.97}, mem={10: 1.2}))
        self.assertTrue(self.verdict(out).startswith("HELD ON MEMORY (peak RSS 1.2000 at 10"))
        rc, out = self.run_on(records(step={1: 0.97}, mem={1: 1.5, 6: 1.5, 8: 1.5}))
        self.assertEqual(self.verdict(out), "ADOPTS")

    def test_composition_reported(self):
        # the same gain as before: here / there 1.0000
        rc, out = self.run_on(records(step={1: 0.97, 10: 0.9320}, noise=0.0))
        line = self.composition(out)
        self.assertIn("decides nothing", line)
        self.assertIn("here 0.9320 [0.9320, 0.9320]", line)
        self.assertIn("here / there 1.0000 [1.0000, 1.0000]", line)

    def test_composition_decides_nothing(self):
        # no gain at 10 left on Wormhole's master: the composition says so, the verdict stands
        rc, out = self.run_on(records(step={1: 0.97, 10: 1.0}))
        self.assertEqual(self.verdict(out), "ADOPTS")
        self.assertIn("here / there 1.07", self.composition(out))

    def assert_void(self, recs):
        rc, out = self.run_on(recs)
        self.assertEqual(rc, 1, out)
        self.assertIn("VOID", out)
        self.assertNotIn("verdict (", out)
        self.assertNotIn("composition", out)
        return out

    def test_void(self):
        self.assertIn("missing", self.assert_void(records()[:-1]))
        recs = records()
        recs[3]["hash"] = "ffff"
        self.assertIn("outputs differ", self.assert_void(recs))
        # the parscan arm, or other counts, are not this run's roster
        self.assertIn("unexpected", self.assert_void(tp.records(threads=(1, 6, 8, 10))))
        self.assertIn("unexpected", self.assert_void(tp.records(cand="mibz2")))


if __name__ == "__main__":
    unittest.main()
