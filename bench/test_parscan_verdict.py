"""bench/parscan_verdict.py (bench-parscan-c37's verdict, copied) on synthetic runs: each
verdict where the registration puts it, the memory gate at the judged count alone, the reported counts and the transfer deciding nothing,
and every defect the validity checks guard against leaving no verdict.

    python3 bench/test_parscan_verdict.py
"""

import io
import json
import math
import shutil
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import parscan_verdict as pv  # noqa: E402

R = 16
THREADS = (1, 2, 4, 6, 8, 10, 12, 16, 20)
BASE_EXE = "/w/target/parscan-c37/build-base/target/release/examples/ab"
ENV = {
    "cpus": 20,
    "cores": 10,
    "core_cpus": [f"{c},{c + 10}" for c in range(10)],
    "cpu_quota": None,
}
LSCPU = "Architecture: x86_64\nModel name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"


TEXT_OUT = 48_123_456


def records(
    ref="mibz2",
    cand="mibz2-parscan",
    threads=THREADS,
    step=None,
    mem=None,
    noise=0.01,
    corpus="wikidata-x4",
    out_bytes=pv.WIKIDATA_X4_OUT,
    digest="0123456789abcdef",
):
    """A complete run: `step` maps a thread count to the head's wall over the base's (default
    level); `mem` a count to its peak memory over the base's (default 1). Rounds alternate the
    noise above and below"""
    step, mem = step or {}, mem or {}
    out = []
    for r in range(R):
        for t in threads:
            for which in ("base", "head"):
                wall, rss = 1.0, 500_000
                if which == "head":
                    wall = step.get(t, 1.0) * (1 + (noise if r % 2 else -noise))
                    rss = int(rss * mem.get(t, 1.0))
                out.append(
                    {
                        "round": r,
                        "corpus": corpus,
                        "threads": t,
                        "arm": cand if which == "head" else f"{ref}@{BASE_EXE}",
                        "cpus": "1" if t == 1 else "all",
                        "termination": "success",
                        "wall": wall,
                        "cpu": wall * t,
                        "rss_kb": rss,
                        "out": out_bytes,
                        "hash": digest,
                        "err": "",
                    }
                )
    return out


class Verdict(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        (self.tmp / "env.json").write_text(json.dumps(ENV))
        (self.tmp / "lscpu.txt").write_text(LSCPU)

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def run_on(self, recs, *extra, env=None, lscpu=None):
        p = self.tmp / "runs.jsonl"
        p.write_text("".join(json.dumps(r) + "\n" for r in recs))
        if env is not None:
            (self.tmp / "env.json").write_text(json.dumps(env))
        if lscpu is not None:
            (self.tmp / "lscpu.txt").write_text(lscpu)
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = pv.main(
                [
                    str(p),
                    "--env",
                    str(self.tmp / "env.json"),
                    "--lscpu",
                    str(self.tmp / "lscpu.txt"),
                    *extra,
                ]
            )
        return rc, buf.getvalue()

    def verdict(self, out):
        return next(line for line in out.splitlines() if line.startswith("verdict"))

    def row(self, out, t):
        return next(ln for ln in out.splitlines() if ln.split()[:1] == [str(t)] and "[" in ln)

    def test_adopts(self):
        rc, out = self.run_on(records(step={10: 0.95, 8: 0.97, 6: 1.0}))
        self.assertEqual(rc, 0, out)
        self.assertEqual(self.verdict(out), "verdict (Decision 1, PARSCAN): ADOPTS")
        self.assertIn("judged", self.row(out, 10))
        self.assertIn("supporting", self.row(out, 6))
        for t in (1, 2, 4, 12, 16, 20):
            self.assertIn("reported", self.row(out, t))

    def test_level_at_ten_not_adopted(self):
        rc, out = self.run_on(records(step={8: 0.9, 6: 0.9, 12: 0.9}))
        self.assertEqual(rc, 0, out)
        self.assertIn("NOT ADOPTED (10 threads not FASTER)", out)

    def test_slower_at_supporting_not_adopted(self):
        for t in (6, 8):
            rc, out = self.run_on(records(step={10: 0.95, t: 1.05}))
            self.assertIn(f"NOT ADOPTED (SLOWER at {t} threads)", self.verdict(out))

    def test_reported_counts_decide_nothing(self):
        # SLOWER at every reported count, and far above the memory gate there: still ADOPTS
        slow = {t: 1.10 for t in (1, 2, 4, 12, 16, 20)}
        rc, out = self.run_on(records(step={10: 0.95, **slow}, mem={t: 1.5 for t in slow}))
        self.assertEqual(self.verdict(out), "verdict (Decision 1, PARSCAN): ADOPTS")
        self.assertIn("SLOWER", self.row(out, 20))

    def test_memory_gate_at_ten_only(self):
        rc, out = self.run_on(records(step={10: 0.95}, mem={6: 1.3, 8: 1.3}))
        self.assertEqual(self.verdict(out), "verdict (Decision 1, PARSCAN): ADOPTS")
        rc, out = self.run_on(records(step={10: 0.95}, mem={10: 1.2}))
        self.assertIn(
            "verdict (Decision 1, PARSCAN): HELD ON MEMORY (peak RSS 1.2000 at 10 threads", out
        )

    def test_memory_at_the_gate_passes(self):
        rc, out = self.run_on(records(step={10: 0.95}, mem={10: 1.10}))
        self.assertEqual(self.verdict(out), "verdict (Decision 1, PARSCAN): ADOPTS")

    def test_memory_not_weighed_without_a_gain(self):
        rc, out = self.run_on(records(step={10: 1.0}, mem={10: 1.2}))
        self.assertIn("NOT ADOPTED (10 threads not FASTER)", self.verdict(out))

    def test_peak_rss_reported(self):
        rc, out = self.run_on(records(step={10: 0.95}, mem={10: 0.9}))
        line = next(ln for ln in out.splitlines() if ln.strip().startswith("10  base"))
        self.assertIn("base 488.3 MiB, head 439.5 MiB, head - base -48.8 MiB (0.9000)", line)

    def test_transfer_reported(self):
        rc, out = self.run_on(records(step={10: 0.9333}, noise=0.0))
        line = next(ln for ln in out.splitlines() if ln.startswith("transfer at 10 threads"))
        self.assertIn("= 1.000 [1.000, 1.000]", line)
        self.assertIn("decides nothing", line)
        self.assertAlmostEqual(pv.transfer(math.sqrt(0.9333), 0.9333), 0.5)
        # a loss transfers below 0
        self.assertLess(pv.transfer(1.02, 0.9333), 0)

    def test_noisy_gain_not_adopted(self):
        rc, out = self.run_on(records(step={10: 0.99}, noise=0.05))
        self.assertIn("NOT ADOPTED", self.verdict(out))

    def test_diagnostic_decides_nothing(self):
        recs = records(cand="mibz2", threads=(1, 6, 8, 10), step={10: 0.9})
        rc, out = self.run_on(
            recs,
            "--arm",
            "mibz2",
            "--judged",
            "",
            "--supporting",
            "",
            "--reported",
            "1,6,8,10",
            "--diagnostic",
        )
        self.assertEqual(rc, 0, out)
        self.assertNotIn("verdict (", out)
        self.assertNotIn("transfer", out)
        self.assertIn("FASTER", self.row(out, 10))
        self.assertIn("diagnostic", self.row(out, 1))

    # ---- Decision 2, BLOCK-READER: the head's mibz2 against the base's at 1, 6, 8 and 10 ----

    BR = ("--decision", "block-reader")

    def br(self, step=None, mem=None, noise=0.01):
        return records(cand="mibz2", threads=(1, 6, 8, 10), step=step, mem=mem, noise=noise)

    def br_verdict(self, out):
        line = self.verdict(out)
        self.assertTrue(line.startswith("verdict (Decision 2, BLOCK-READER): "), line)
        return line.split(": ", 1)[1]

    def test_block_reader_adopts(self):
        rc, out = self.run_on(self.br(step={1: 0.97}), *self.BR)
        self.assertEqual(rc, 0, out)
        self.assertEqual(self.br_verdict(out), "ADOPTS")
        self.assertIn("judged", self.row(out, 1))
        for t in (6, 8, 10):
            self.assertIn("supporting", self.row(out, t))
        # no transfer: the handoff's claim is Decision 1's
        self.assertNotIn("transfer", out)

    def test_block_reader_one_thread_must_be_faster(self):
        # faster at every parallel count is not enough: 1 thread level
        rc, out = self.run_on(self.br(step={6: 0.9, 8: 0.9, 10: 0.9}), *self.BR)
        self.assertEqual(self.br_verdict(out), "NOT ADOPTED (1 threads not FASTER)")

    def test_block_reader_slower_at_any_parallel_count(self):
        for t in (6, 8, 10):
            rc, out = self.run_on(self.br(step={1: 0.97, t: 1.05}), *self.BR)
            self.assertEqual(self.br_verdict(out), f"NOT ADOPTED (SLOWER at {t} threads)")

    def test_block_reader_memory_at_ten(self):
        rc, out = self.run_on(self.br(step={1: 0.97}, mem={10: 1.2}), *self.BR)
        self.assertTrue(self.br_verdict(out).startswith("HELD ON MEMORY (peak RSS 1.2000 at 10"))
        # memory above the gate at 1, 6 or 8 alone does not hold it
        rc, out = self.run_on(self.br(step={1: 0.97}, mem={1: 1.5, 6: 1.5, 8: 1.5}), *self.BR)
        self.assertEqual(self.br_verdict(out), "ADOPTS")
        rc, out = self.run_on(self.br(step={1: 0.97}, mem={10: 1.10}), *self.BR)
        self.assertEqual(self.br_verdict(out), "ADOPTS")

    def test_block_reader_roster(self):
        # the parscan arm is not this decision's roster, nor are the other counts
        self.assertIn("unexpected", self.assert_void(records(threads=(1, 6, 8, 10)), *self.BR))
        self.assertIn("unexpected", self.assert_void(records(cand="mibz2"), *self.BR))
        self.assertIn("missing", self.assert_void(self.br()[:-1], *self.BR))
        # and Decision 1 does not read Decision 2's records (the head's arm is not its roster)
        self.assertIn("unexpected key ('mibz2', 1, 0)", self.assert_void(self.br()))

    def test_decisions_are_independent(self):
        # the same machine, two files: each decision's verdict reads its own records alone
        rc, out1 = self.run_on(records(step={10: 1.05}))
        rc, out2 = self.run_on(self.br(step={1: 0.97}), *self.BR)
        self.assertIn("NOT ADOPTED", self.verdict(out1))
        self.assertEqual(self.br_verdict(out2), "ADOPTS")

    # ---- the loss guard on a second corpus (bench-parscan-stream's amendment) ----

    GUARD = ("--guard", "text", "--guard-out-bytes", str(TEXT_OUT), "--guard-loss", "6,8,10")

    def text(self, step=None, mem=None, noise=0.01):
        return records(
            step=step, mem=mem, noise=noise, corpus="text", out_bytes=TEXT_OUT, digest="7e57"
        )

    def test_guard_adopts(self):
        rc, out = self.run_on(records(step={10: 0.95}) + self.text(), *self.GUARD)
        self.assertEqual(rc, 0, out)
        self.assertEqual(self.verdict(out), "verdict (Decision 1, PARSCAN): ADOPTS")
        self.assertIn("text (the loss guard: SLOWER at 6, 8, 10 threads", out)

    def test_guard_slower_not_adopted(self):
        for t in (6, 8, 10):
            rc, out = self.run_on(records(step={10: 0.95}) + self.text(step={t: 1.05}), *self.GUARD)
            self.assertEqual(rc, 0, out)
            self.assertEqual(
                self.verdict(out),
                f"verdict (Decision 1, PARSCAN): NOT ADOPTED (text SLOWER at {t} threads: the loss guard)",
            )

    def test_guard_with_no_gain(self):
        rc, out = self.run_on(records(step={10: 1.0}) + self.text(step={10: 1.1}), *self.GUARD)
        self.assertIn("NOT ADOPTED (10 threads not FASTER; text SLOWER at 10 threads", out)

    def test_guard_other_counts_decide_nothing(self):
        # text SLOWER at every count outside 6, 8 and 10, and a text gain at 10: still ADOPTS
        slow = {t: 1.10 for t in (1, 2, 4, 12, 16, 20)}
        rc, out = self.run_on(
            records(step={10: 0.95}) + self.text(step={**slow, 10: 0.8}), *self.GUARD
        )
        self.assertEqual(self.verdict(out), "verdict (Decision 1, PARSCAN): ADOPTS")
        # an inconclusive text step at 10 does not count against adoption
        rc, out = self.run_on(
            records(step={10: 0.95}) + self.text(step={10: 1.01}, noise=0.05), *self.GUARD
        )
        self.assertEqual(self.verdict(out), "verdict (Decision 1, PARSCAN): ADOPTS")

    def test_guard_memory_at_ten(self):
        rc, out = self.run_on(records(step={10: 0.95}) + self.text(mem={10: 1.2}), *self.GUARD)
        self.assertIn("HELD ON MEMORY (peak RSS text 1.2000 at 10 threads", self.verdict(out))
        rc, out = self.run_on(
            records(step={10: 0.95}, mem={10: 1.3}) + self.text(mem={10: 1.2}), *self.GUARD
        )
        self.assertIn("HELD ON MEMORY (peak RSS wikidata-x4 1.3000, text 1.2000 at 10", out)
        # text's memory elsewhere, and at the gate exactly, does not hold it
        rc, out = self.run_on(
            records(step={10: 0.95}) + self.text(mem={8: 1.5, 20: 1.5, 10: 1.10}), *self.GUARD
        )
        self.assertEqual(self.verdict(out), "verdict (Decision 1, PARSCAN): ADOPTS")
        # a text loss outranks the memory gate
        rc, out = self.run_on(
            records(step={10: 0.95}) + self.text(step={8: 1.05}, mem={10: 1.2}), *self.GUARD
        )
        self.assertIn("NOT ADOPTED (text SLOWER at 8 threads: the loss guard)", out)

    def test_guard_checked_whole(self):
        both = records(step={10: 0.95}) + self.text()
        # a text record missing, duplicated, failed or of the wrong size voids the run
        self.assertIn("missing", self.assert_void(both[:-1], *self.GUARD))
        self.assertIn("on text", self.assert_void(both + both[-1:], *self.GUARD))
        recs = records(step={10: 0.95}) + self.text()
        recs[-3]["out"] = 1
        self.assertIn("wrote", self.assert_void(recs, *self.GUARD))
        recs = records(step={10: 0.95}) + self.text()
        recs[-3]["termination"] = "signal 9"
        self.assert_void(recs, *self.GUARD)
        # each corpus its own one hash; two hashes within text void it
        recs = records(step={10: 0.95}) + self.text()
        recs[-3]["hash"] = "ffff"
        self.assertIn("outputs differ on text", self.assert_void(recs, *self.GUARD))
        # no text at all, or a third corpus, voids it
        self.assertIn("missing", self.assert_void(records(step={10: 0.95}), *self.GUARD))
        recs = both + [dict(both[0], corpus="random")]
        self.assertIn("corpus 'random'", self.assert_void(recs, *self.GUARD))

    def test_guard_arguments(self):
        both = records(step={10: 0.95}) + self.text()
        self.assert_void(both, "--guard", "text", "--guard-loss", "6,8,10")
        self.assert_void(both, "--guard", "text", "--guard-out-bytes", str(TEXT_OUT))
        self.assert_void(
            both, "--guard", "wikidata-x4", "--guard-out-bytes", "1", "--guard-loss", "10"
        )
        self.assert_void(
            both, "--guard", "text", "--guard-out-bytes", str(TEXT_OUT), "--guard-loss", "9"
        )
        self.assert_void(records(step={10: 0.95}), "--guard-loss", "6,8,10")
        # without a guard, text records are not the roster
        self.assertIn("corpus 'text'", self.assert_void(both))

    def assert_void(self, recs, *extra, **kw):
        rc, out = self.run_on(recs, *extra, **kw)
        self.assertEqual(rc, 1, out)
        self.assertTrue(out.startswith("VOID"), out)
        self.assertNotIn("verdict (", out)
        return out

    def test_void_roles(self):
        recs = records(step={10: 0.9})
        self.assertIn("two roles", self.assert_void(recs, "--supporting", "6,8,10"))
        self.assertIn("two roles", self.assert_void(recs, "--reported", "1,2,4,8,12,16,20"))
        self.assertIn("one count", self.assert_void(recs, "--judged", ""))

    def test_void_missing_record(self):
        recs = records(step={10: 0.9})
        self.assertIn("missing", self.assert_void(recs[:-1]))
        # a reported count missing voids the run too
        recs = [r for r in records(step={10: 0.9}) if not (r["threads"] == 20 and r["round"] == 3)]
        self.assertIn("missing", self.assert_void(recs))

    def test_void_duplicate(self):
        recs = records(step={10: 0.9})
        self.assertIn("duplicated", self.assert_void(recs + recs[:1]))

    def test_void_unexpected_arm(self):
        recs = records(step={10: 0.9})
        recs[0]["arm"] = "mibz2@/elsewhere/ab"
        self.assert_void(recs)
        # the base's harness running the candidate arm is not the roster's
        recs = records(step={10: 0.9})
        recs[1]["arm"] = f"mibz2-parscan@{BASE_EXE}"
        self.assertIn("unexpected", self.assert_void(recs))

    def test_void_failed_run(self):
        for field, bad in (
            ("termination", "signal 9"),
            ("err", "boom"),
            ("wall", 0),
            ("rss_kb", float("nan")),
            ("cpu", None),
        ):
            recs = records(step={10: 0.9})
            recs[5][field] = bad
            self.assert_void(recs)

    def test_void_pinning_or_spinner(self):
        recs = records(step={10: 0.9})
        i = next(i for i, r in enumerate(recs) if r["threads"] == 10)
        recs[i]["cpus"] = "0,1,2,3,4,5,6,7,8,9"
        self.assertIn("not unpinned", self.assert_void(recs))
        recs = records(step={10: 0.9})
        i = next(i for i, r in enumerate(recs) if r["threads"] == 1)
        recs[i]["cpus"] = "all"
        self.assertIn("not one CPU", self.assert_void(recs))
        recs = records(step={10: 0.9})
        recs[3]["held_spinner"] = {"cpu": 19}
        self.assertIn("spinner", self.assert_void(recs))

    def test_void_outputs(self):
        recs = records(step={10: 0.9})
        recs[7]["hash"] = "ffff"
        self.assertIn("outputs differ", self.assert_void(recs))
        recs = records(step={10: 0.9})
        recs[7]["out"] = 1
        self.assertIn("wrote", self.assert_void(recs))
        recs = records(step={10: 0.9})
        recs[7]["corpus"] = "text"
        self.assert_void(recs)

    def test_void_machine(self):
        self.assert_void(records(step={10: 0.9}), lscpu="Model name: AMD Ryzen 9 7950X\n")
        self.assert_void(records(step={10: 0.9}), env={**ENV, "cpus": 16})
        self.assert_void(records(step={10: 0.9}), env={**ENV, "cpu_quota": 4.0})

    def test_void_unreadable(self):
        rc, out = self.run_on([])
        self.assertEqual(rc, 1)
        p = self.tmp / "runs.jsonl"
        p.write_text("not json\n")
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = pv.main(
                [
                    str(p),
                    "--env",
                    str(self.tmp / "env.json"),
                    "--lscpu",
                    str(self.tmp / "lscpu.txt"),
                ]
            )
        self.assertEqual(rc, 1)


if __name__ == "__main__":
    unittest.main()
