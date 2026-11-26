"""bench/parscan_stream.py on synthetic runs: the verdict where the registration puts it
(Wikidata FASTER at 10 threads, and not SLOWER at 6, 8, 12, 16 or 20; text not SLOWER at 6, 8,
10, 12, 16 or 20; the memory gate at 8, 10, 12, 16 and 20 on both corpora; 1, 2 and 4
reported), the preflight's topology and activation checks, and no verdict from incomplete or
malformed records.

    python3 bench/test_parscan_stream.py
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
import parscan_stream as ps  # noqa: E402
import test_parscan_verdict as tp  # noqa: E402


def observed(head, t):
    """The thread names `ab`'s census would see: master's serial scanner (`mibz2-scan`) and its
    workers (7 from 10 threads), or the head's front end from 8 threads (8 workers from 10, the
    lead scanner and two helpers)"""
    if head and t >= 8:
        n = 8 if t >= 10 else t
        extra = ["mibz2-scan", "mibz2-scan-1", "mibz2-scan-2"]
    elif t > 1:
        n = 7 if t >= 10 else t
        extra = ["mibz2-scan"]
    else:
        n, extra = 0, []
    return " ".join(sorted(["ab"] + [f"mibz2-{i}" for i in range(n)] + extra))


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
    for r in recs:
        r["observed_threads"] = observed(r["arm"] == "mibz2", r["threads"])
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
        self.assertTrue(line.startswith("verdict (STREAMING FRONT END): "), line)
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

    def test_wikidata_slower_where_the_front_end_runs(self):
        # (the correction: 12, 16 and 20, production's default at 20, are in the loss set)
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
        # #741's figures, 1.28 at 10 and 1.99 at 20: both cells named
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
        # the recalibration's text loss (1.109 at 10) refuses adoption whatever Wikidata gains
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
        # the recalibration's 8/32 over 7/28 on text at 12, 16 and 20 (1.096, 1.057, 1.093):
        # under the draft's rule this was ADOPTS; it is now refused
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
        # (under the draft's rule, text's RSS over 1.10 at 8 or 20 alone was ADOPTS)
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

    def test_census_printed_decides_nothing(self):
        rc, out = self.run_on(records(step={10: 0.93}))
        self.assertIn("the front end's activation by ab's thread census", out)
        self.assertIn(
            "wikidata-x4  10 threads: base 0/16, 0/16, 0/16; head 16/16, 16/16, 0/16", out
        )
        self.assertIn("text          4 threads: base 0/16, 0/16, 0/16; head 0/16, 0/16, 0/16", out)
        # a census against the registered activation is printed, and still decides nothing
        recs = records(step={10: 0.93})
        for r in recs:
            r["observed_threads"] = "ab"
        rc, out = self.run_on(recs)
        self.assertEqual(rc, 0, out)
        self.assertEqual(self.verdict(out), "ADOPTS")

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
                r["arm"] = "mibz2-parscan"
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

    def preflight_with(self, change):
        recs = self.one_round()
        for r in recs:
            change(r)
        return self.run_on(recs, "--preflight")

    def test_preflight_activation(self):
        head = lambda r: r["arm"] == "mibz2"  # noqa: E731

        # the head without its helper from 8 threads: the front end did not run
        def no_helper(r):
            if head(r) and r["threads"] == 8 and r["corpus"] == "text":
                r["observed_threads"] = observed(False, 8)

        rc, out = self.preflight_with(no_helper)
        self.assertEqual(rc, 1, out)
        self.assertIn("no mibz2-scan-1: the front end did not run", out)

        # the head at 10 or more with 7 workers, or with 9
        for n in (7, 9):

            def workers(r, n=n):
                if head(r) and r["threads"] == 16:
                    r["observed_threads"] = " ".join(
                        [f"mibz2-{i}" for i in range(n)] + ["mibz2-scan", "mibz2-scan-1"]
                    )

            rc, out = self.preflight_with(workers)
            self.assertEqual(rc, 1, (n, out))
            self.assertIn("not 8 workers", out)

        # a helper on the base, or on the head below 8
        def base_helper(r):
            if not head(r) and r["threads"] == 20:
                r["observed_threads"] += " mibz2-scan-1"

        rc, out = self.preflight_with(base_helper)
        self.assertEqual(rc, 1, out)
        self.assertIn("mibz2-scan-1 where the front end must not run", out)

        def low_helper(r):
            if head(r) and r["threads"] == 6:
                r["observed_threads"] += " mibz2-scan-1"

        rc, out = self.preflight_with(low_helper)
        self.assertEqual(rc, 1, out)

        # no census at all
        rc, out = self.preflight_with(lambda r: r.pop("observed_threads"))
        self.assertEqual(rc, 1, out)
        self.assertIn("no thread census", out)


if __name__ == "__main__":
    unittest.main()
