"""bench/block_symbols_tiled.py on synthetic runs: the verdict where the registration puts it
(Wikidata FASTER at 6 threads, and not SLOWER at 1, 2, 4, 8 or 10; text not SLOWER at any
count; the memory gate at every count on both corpora), the preflight's topology and dispatch
checks, and no verdict from incomplete or malformed records.

    python3 bench/test_block_symbols_tiled.py
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
import block_symbols_tiled as bt  # noqa: E402
import test_parscan_verdict as tp  # noqa: E402


def observed(t):
    """The thread names `ab`'s census would see on either build: master's serial scanner
    (`mibz2-scan`) and its workers (7 from 10 threads), none at one thread"""
    if t == 1:
        return "ab"
    n = 7 if t >= 10 else t
    return " ".join(sorted(["ab", "mibz2-scan"] + [f"mibz2-{i}" for i in range(n)]))


def records(step=None, mem=None, noise=0.01, text=None, text_mem=None):
    """Both corpora: Wikidata's steps `step`, text's `text` (level by default)"""
    recs = tp.records(
        cand="mibz2", threads=bt.THREADS, step=step, mem=mem, noise=noise
    ) + tp.records(
        cand="mibz2",
        threads=bt.THREADS,
        step=text,
        mem=text_mem,
        noise=noise,
        corpus="text",
        out_bytes=tp.TEXT_OUT,
        digest="7e57",
    )
    for r in recs:
        r["observed_threads"] = observed(r["threads"])
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
            rc = bt.main(
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
        self.assertTrue(line.startswith("verdict (TILED ZERO-FILL): "), line)
        return line.split(": ", 1)[1]

    def test_adopts(self):
        # #770's run at 6 threads: 0.977
        rc, out = self.run_on(records(step={6: 0.977}))
        self.assertEqual(rc, 0, out)
        self.assertEqual(self.verdict(out), "ADOPTS")
        self.assertNotIn("transfer", out)

    def test_six_not_faster(self):
        # 4 alone FASTER (#770's 0.978) does not adopt: 6 is the judged count
        rc, out = self.run_on(records(step={4: 0.978}))
        self.assertEqual(self.verdict(out), "NOT ADOPTED (6 threads not FASTER)")

    def test_wikidata_slower_at_a_supporting_count(self):
        for t in (1, 2, 4, 8, 10):
            rc, out = self.run_on(records(step={6: 0.977, t: 1.02}))
            self.assertEqual(self.verdict(out), f"NOT ADOPTED (SLOWER at {t} threads)")
        rc, out = self.run_on(records(step={6: 0.977, 1: 1.02, 10: 1.02}))
        self.assertEqual(self.verdict(out), "NOT ADOPTED (SLOWER at 1, 10 threads)")

    def test_text_guard_at_every_count(self):
        for t in bt.THREADS:
            rc, out = self.run_on(records(step={6: 0.977}, text={t: 1.02}))
            self.assertEqual(
                self.verdict(out), f"NOT ADOPTED (text SLOWER at {t} threads: the loss guard)"
            )
        # a text gain decides nothing
        rc, out = self.run_on(records(step={6: 0.977}, text={6: 0.9}))
        self.assertEqual(self.verdict(out), "ADOPTS")

    def test_memory_gate_at_every_count(self):
        for t in bt.THREADS:
            rc, out = self.run_on(records(step={6: 0.977}, mem={t: 1.11}))
            self.assertTrue(self.verdict(out).startswith("HELD ON MEMORY"), out)
            self.assertIn(f"wikidata-x4 at {t} threads 1.1100", self.verdict(out))
            rc, out = self.run_on(records(step={6: 0.977}, text_mem={t: 1.11}))
            self.assertIn(f"text at {t} threads 1.1100", self.verdict(out))
        # the gate exactly does not hold it
        rc, out = self.run_on(records(step={6: 0.977}, mem={1: 1.10}, text_mem={10: 1.10}))
        self.assertEqual(self.verdict(out), "ADOPTS")
        # a loss outranks the memory gate
        rc, out = self.run_on(records(step={6: 0.977, 2: 1.05}, mem={2: 1.5}))
        self.assertEqual(self.verdict(out), "NOT ADOPTED (SLOWER at 2 threads)")

    def test_every_count_has_a_role(self):
        # 6 judged, the rest supporting on Wikidata; every count guarded on text; none reported
        rc, out = self.run_on(records(step={6: 0.977}))
        roles = [ln.split()[:2] for ln in out.splitlines() if ln[:7].strip().isdigit()]
        want = [[str(t), "judged" if t == 6 else "supporting"] for t in bt.THREADS]
        want += [[str(t), "guard"] for t in bt.THREADS]
        self.assertEqual([r for r in roles if r[1] != "base"], want, out)

    def test_text_missing_is_void(self):
        recs = [
            r
            for r in records(step={6: 0.977})
            if not (r["corpus"] == "text" and r["threads"] == 10)
        ]
        rc, out = self.run_on(recs)
        self.assertEqual(rc, 1, out)
        self.assertIn("VOID", out)

    def test_a_count_off_the_grid_is_void(self):
        recs = records(step={6: 0.977}) + [
            dict(r, threads=12) for r in records() if r["threads"] == 10
        ]
        rc, out = self.run_on(recs)
        self.assertEqual(rc, 1, out)
        self.assertIn("unexpected key", out)

    def test_missing_record_is_void(self):
        rc, out = self.run_on(records(step={6: 0.977})[:-1])
        self.assertEqual(rc, 1, out)
        self.assertIn("VOID", out)
        self.assertFalse(any(ln.startswith("verdict") for ln in out.splitlines()))

    def test_text_out_bytes_required(self):
        p = self.tmp / "runs.jsonl"
        p.write_text("".join(json.dumps(r) + "\n" for r in records(step={6: 0.977})))
        with (
            self.assertRaises(SystemExit),
            redirect_stdout(io.StringIO()),
            contextlib.redirect_stderr(io.StringIO()),
        ):
            bt.main(
                [
                    str(p),
                    "--env",
                    str(self.tmp / "env.json"),
                    "--lscpu",
                    str(self.tmp / "lscpu.txt"),
                ]
            )

    def one_round(self):
        return [r for r in records() if r["round"] == 0]

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

    def test_preflight_fails_off_the_i9(self):
        (self.tmp / "lscpu.txt").write_text("Model name: AMD EPYC\n")
        rc, out = self.run_on(self.one_round(), "--preflight")
        self.assertEqual(rc, 1, out)

    def test_preflight_checks_the_topology(self):
        for bad in ({"cores": 8}, {"cpus": 16}, {"cpu_quota": 4.0}, {"core_cpus": ["0,10"]}):
            (self.tmp / "env.json").write_text(json.dumps({**tp.ENV, **bad}))
            rc, out = self.run_on(self.one_round(), "--preflight")
            self.assertEqual(rc, 1, (bad, out))

    def test_preflight_dispatch(self):
        # a helper scanner on either arm: not the serial scanner
        recs = self.one_round()
        next(r for r in recs if r["threads"] == 8 and r["arm"] == "mibz2")["observed_threads"] += (
            " mibz2-scan-1"
        )
        rc, out = self.run_on(recs, "--preflight")
        self.assertEqual(rc, 1, out)
        self.assertIn("a helper scanner", out)
        # no serial scanner above one thread
        recs = self.one_round()
        next(r for r in recs if r["threads"] == 4)["observed_threads"] = "ab mibz2-0"
        rc, out = self.run_on(recs, "--preflight")
        self.assertEqual(rc, 1, out)
        self.assertIn("not the serial scanner", out)
        # no census
        recs = self.one_round()
        next(r for r in recs if r["threads"] == 2).pop("observed_threads")
        rc, out = self.run_on(recs, "--preflight")
        self.assertEqual(rc, 1, out)
        self.assertIn("no thread census", out)


if __name__ == "__main__":
    unittest.main()
