"""bench/open_route.py on synthetic runs: the verdict where the registration puts it
(Wikidata FASTER at 10 threads, and not SLOWER at 6, 8, 12, 16 or 20; text not SLOWER at 6, 8,
10, 12, 16 or 20; the memory gate at 8, 10, 12, 16 and 20 on both corpora; 1, 2 and 4
reported), the preflight's machine, topology and record checks, and no verdict from incomplete
or malformed records.

    python3 bench/test_open_route.py
"""

import contextlib
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
import open_route as ps  # noqa: E402
import test_parscan_verdict as tp  # noqa: E402


def records(step=None, mem=None, noise=0.01, text=None, text_mem=None):
    """Both corpora: Wikidata's steps `step`, text's `text` (level by default)"""
    recs = tp.records(
        cand="mibz2", threads=ps.THREADS, step=step, mem=mem, noise=noise
    ) + tp.records(
        cand="mibz2",
        threads=ps.THREADS,
        step=text,
        mem=text_mem,
        noise=noise,
        corpus="text",
        out_bytes=tp.TEXT_OUT,
        digest="7e57",
    )
    return recs


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
                [
                    str(p),
                    "--env",
                    str(self.tmp / "env.json"),
                    "--lscpu",
                    str(self.tmp / "lscpu.txt"),
                    "--text-out-bytes",
                    str(tp.TEXT_OUT),
                ]
                + list(extra)
            )
        return rc, buf.getvalue()

    def verdict(self, out):
        line = next(ln for ln in out.splitlines() if ln.startswith("verdict"))
        self.assertTrue(line.startswith("verdict (OPEN KERNEL ROUTE): "), line)
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
        # 1, 2 and 4 alone are reported (CI's paired bench gates them), on both corpora
        slow = {1: 1.2, 2: 1.2, 4: 1.2}
        rc, out = self.run_on(records(step={10: 0.9, **slow}, text=slow))
        self.assertEqual(self.verdict(out), "ADOPTS")

    def test_wikidata_slower_past_the_cores(self):
        # 12, 16 and 20 (production's default at 20) are in the loss set
        for t in (12, 16, 20):
            rc, out = self.run_on(records(step={10: 0.9, t: 1.05}))
            self.assertEqual(self.verdict(out), f"NOT ADOPTED (SLOWER at {t} threads)")
        rc, out = self.run_on(records(step={10: 0.9, 6: 1.05, 20: 1.05}))
        self.assertEqual(self.verdict(out), "NOT ADOPTED (SLOWER at 6, 20 threads)")

    def test_memory_gate_at_eight_to_twenty(self):
        for t in (8, 10, 12, 16, 20):
            rc, out = self.run_on(records(step={10: 0.9}, mem={t: 1.11}))
            self.assertEqual(
                self.verdict(out),
                f"HELD ON MEMORY (peak RSS above 1.10: wikidata-x4 at {t} threads 1.1100: the "
                "Coordinator weighs it, PERFORMANCE-POLICY rule 4)",
            )
        # two cells over: both named
        rc, out = self.run_on(records(step={10: 0.9}, mem={10: 1.28, 20: 1.99}))
        self.assertIn(
            "wikidata-x4 at 10 threads 1.2800, wikidata-x4 at 20 threads 1.9900:", self.verdict(out)
        )
        # 1, 2, 4 and 6, and the gate exactly, do not hold it
        rc, out = self.run_on(
            records(step={10: 0.9}, mem={1: 1.5, 2: 1.5, 4: 1.5, 6: 1.5, 20: 1.10})
        )
        self.assertEqual(self.verdict(out), "ADOPTS")
        # a loss outranks the memory gate
        rc, out = self.run_on(records(step={10: 0.9, 16: 1.05}, mem={20: 1.5}))
        self.assertEqual(self.verdict(out), "NOT ADOPTED (SLOWER at 16 threads)")

    def test_text_guard(self):
        # a text loss at 10 refuses adoption whatever Wikidata gains
        rc, out = self.run_on(records(step={10: 0.93}, text={10: 1.109}))
        self.assertEqual(rc, 0, out)
        self.assertEqual(
            self.verdict(out), "NOT ADOPTED (text SLOWER at 10 threads: the loss guard)"
        )
        for t in (6, 8, 12, 16, 20):
            rc, out = self.run_on(records(step={10: 0.93}, text={t: 1.05}))
            self.assertEqual(
                self.verdict(out), f"NOT ADOPTED (text SLOWER at {t} threads: the loss guard)"
            )
        # several text losses past the cores: all named
        rc, out = self.run_on(records(step={10: 0.93}, text={12: 1.096, 16: 1.057, 20: 1.093}))
        self.assertEqual(
            self.verdict(out), "NOT ADOPTED (text SLOWER at 12, 16, 20 threads: the loss guard)"
        )
        # text at 1, 2 and 4 is reported, deciding nothing
        rc, out = self.run_on(records(step={10: 0.93}, text={1: 1.08, 2: 1.08, 4: 1.08}))
        self.assertEqual(self.verdict(out), "ADOPTS")

    def test_text_memory_gate_at_eight_to_twenty(self):
        rc, out = self.run_on(records(step={10: 0.93}, text_mem={10: 1.11}))
        self.assertTrue(
            self.verdict(out).startswith(
                "HELD ON MEMORY (peak RSS above 1.10: text at 10 threads 1.1100"
            ),
            out,
        )
        for t in (8, 20):
            rc, out = self.run_on(records(step={10: 0.93}, text_mem={t: 1.5}))
            self.assertIn(f"text at {t} threads 1.5000", self.verdict(out))
            self.assertTrue(self.verdict(out).startswith("HELD ON MEMORY"), out)
        rc, out = self.run_on(records(step={10: 0.93}, mem={12: 1.2}, text_mem={16: 1.3}))
        self.assertIn(
            "peak RSS above 1.10: wikidata-x4 at 12 threads 1.2000, text at 16 threads 1.3000:",
            self.verdict(out),
        )
        rc, out = self.run_on(records(step={10: 0.93}, text_mem={6: 1.5, 4: 1.5}))
        self.assertEqual(self.verdict(out), "ADOPTS")
        # a text loss outranks the memory gate
        rc, out = self.run_on(records(step={10: 0.93}, text={20: 1.05}, text_mem={20: 1.5}))
        self.assertEqual(
            self.verdict(out), "NOT ADOPTED (text SLOWER at 20 threads: the loss guard)"
        )

    def test_text_missing_is_void(self):
        recs = [
            r
            for r in records(step={10: 0.93})
            if not (r["corpus"] == "text" and r["threads"] == 20)
        ]
        rc, out = self.run_on(recs)
        self.assertEqual(rc, 1, out)
        self.assertIn("VOID", out)

    def test_text_out_bytes_required(self):
        p = self.tmp / "runs.jsonl"
        p.write_text("".join(json.dumps(r) + "\n" for r in records(step={10: 0.93})))
        with (
            self.assertRaises(SystemExit),
            redirect_stdout(io.StringIO()),
            contextlib.redirect_stderr(io.StringIO()),
        ):
            ps.main(
                [
                    str(p),
                    "--env",
                    str(self.tmp / "env.json"),
                    "--lscpu",
                    str(self.tmp / "lscpu.txt"),
                ]
            )

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
                r["arm"] = "mibz2-workers-lazy"
        rc, out = self.run_on(recs)
        self.assertEqual(rc, 1, out)
        self.assertIn("VOID", out)

    def test_preflight_needs_text(self):
        rc, out = self.run_on([r for r in self.one_round() if r["corpus"] != "text"], "--preflight")
        self.assertEqual(rc, 1, out)
        recs = self.one_round()
        next(r for r in recs if r["corpus"] == "text")["out"] = 7
        rc, out = self.run_on(recs, "--preflight")
        self.assertEqual(rc, 1, out)

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

    def test_preflight_checks_the_topology(self):
        for bad in ({"cores": 8}, {"cpus": 16}, {"cpu_quota": 4.0}, {"core_cpus": ["0,10"]}):
            (self.tmp / "env.json").write_text(json.dumps({**tp.ENV, **bad}))
            rc, out = self.run_on(self.one_round(), "--preflight")
            self.assertEqual(rc, 1, (bad, out))
            self.assertIn("preflight: FAILED", out)

    def test_preflight_fails_on_arms_that_disagree(self):
        # the head's output differs from the base's on one cell: one hash a corpus fails
        recs = self.one_round()
        next(r for r in recs if r["arm"] == "mibz2" and r["threads"] == 10)["hash"] = "0badc0de"
        rc, out = self.run_on(recs, "--preflight")
        self.assertEqual(rc, 1, out)
        self.assertIn("preflight: FAILED", out)


if __name__ == "__main__":
    unittest.main()
