"""bench/scanner_plumbing.py on synthetic runs: each verdict where the registration puts it, and
every defect the validity checks guard against leaves no verdict. bench/test_open_relay8.py's
cases with the counts moved, plus the workers check, rule 4's trade report and `--dry`.

    python3 bench/test_scanner_plumbing.py
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
import scanner_plumbing as sp  # noqa: E402

R = 16
BASE_EXE = "/w/target/scanner-plumbing/build-base/target/release/examples/ab"
ENV = {
    "cpus": 20,
    "cores": 10,
    "core_cpus": [f"{c},{c + 10}" for c in range(10)],
    "cpu_quota": None,
}
LSCPU = "Architecture: x86_64\nModel name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
COUNTS = (6, 8, 9, 10, 12)


def prod_workers(t):
    """production's workers on the i9 unpinned: the plan's own below 10 threads, 7 from 10"""
    return t if t < 10 else 7


def records(arm="mibz2", threads=COUNTS, step=None, mem=1.0, noise=0.01, workers=None, clock=None):
    """A complete run: `step` maps a thread count to the head's wall over the base's (default
    level); `mem` its peak memory over the base's; `workers` maps a count to the head's workers
    (default the base's). Rounds alternate the noise above and below"""
    step = step or {}
    workers = workers or {}
    out = []
    for r in range(R):
        for t in threads:
            for which in ("base", "head"):
                wall = 1.0
                rss = 500_000
                n = prod_workers(t)
                if which == "head":
                    wall = step.get(t, 1.0) * (1 + (noise if r % 2 else -noise))
                    rss = int(rss * mem)
                    n = workers.get(t, n)
                names = ["ab"] + [f"mibz2-{i}" for i in range(n)] + ["mibz2-read", "mibz2-scan"]
                out.append(
                    {
                        "round": r,
                        "corpus": "wikidata-x4",
                        "threads": t,
                        "arm": f"{arm}@{BASE_EXE}" if which == "base" else arm,
                        "cpus": "all",
                        "termination": "success",
                        "wall": wall,
                        "cpu": wall * t,
                        "rss_kb": rss,
                        "out": sp.WIKIDATA_X4_OUT,
                        "hash": "0123456789abcdef",
                        "err": "",
                        "observed_threads": " ".join(names + ["sampler"]),
                        **(
                            {}
                            if clock is None
                            else {
                                "cycles_per_ref": 4.9
                                / 3.7
                                * (clock.get(t, 1.0) if which == "head" else 1.0)
                            }
                        ),
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
            rc = sp.main(
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
        return next(line for line in out.splitlines() if "verdict" in line.split(":")[0])

    def test_adopts(self):
        rc, out = self.run_on(records(step={9: 0.95, 10: 0.99, 8: 0.99}))
        self.assertEqual(rc, 0, out)
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")

    def test_level_at_nine_not_adopted(self):
        rc, out = self.run_on(records(step={8: 0.9, 10: 0.9, 12: 0.9}))
        self.assertEqual(rc, 0, out)
        self.assertIn("NOT ADOPTED (9 threads not FASTER)", out)

    def test_slower_at_a_supporting_count_not_adopted(self):
        for t in (6, 8, 10, 12):
            rc, out = self.run_on(records(step={9: 0.95, t: 1.05}))
            self.assertIn(f"NOT ADOPTED (SLOWER at {t} threads)", self.verdict(out))

    def test_peak_rss_reported(self):
        rc, out = self.run_on(records(step={9: 0.95}, mem=0.9))
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")
        line = next(ln for ln in out.splitlines() if ln.strip().startswith("9  base"))
        self.assertIn("base 488.3 MiB, head 439.5 MiB, head - base -48.8 MiB (0.9000)", line)

    def test_workers_reported(self):
        rc, out = self.run_on(records(step={9: 0.95}))
        row = next(ln for ln in out.splitlines() if ln.strip().startswith("12  supporting"))
        self.assertEqual(row.split()[2], "7")
        row = next(ln for ln in out.splitlines() if ln.strip().startswith("9  judged"))
        self.assertEqual(row.split()[2], "9")

    def test_held_on_memory_trade_not_met(self):
        # 5% faster at 9 for 20% more memory: more than twice the gain
        rc, out = self.run_on(records(step={9: 0.95}, mem=1.2))
        self.assertIn("verdict: HELD ON MEMORY (peak RSS above 1.1 at 6, 8, 9, 10, 12 threads", out)
        self.assertIn(
            "rule 4's trade at 9 threads (reported; the Coordinator weighs it): not met", out
        )

    def test_held_on_memory_trade_met(self):
        # 12% faster everywhere for 15% more memory (+75 MiB here: over 64 MB, so not met; the
        # small-process case below is met)
        recs = records(step={t: 0.88 for t in COUNTS}, mem=1.15)
        for r in recs:
            r["rss_kb"] = r["rss_kb"] // 10  # 48.8 MiB base, +7.3 MiB
        rc, out = self.run_on(recs)
        self.assertIn("verdict: HELD ON MEMORY", out)
        self.assertIn(
            "rule 4's trade at 9 threads (reported; the Coordinator weighs it): met:", out
        )
        rc, out = self.run_on(records(step={t: 0.88 for t in COUNTS}, mem=1.15))
        self.assertIn(
            "rule 4's trade at 9 threads (reported; the Coordinator weighs it): not met", out
        )

    def test_memory_at_the_gate_passes(self):
        rc, out = self.run_on(records(step={9: 0.95}, mem=1.10))
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")

    def test_noisy_gain_not_adopted(self):
        rc, out = self.run_on(records(step={9: 0.99}, noise=0.05))
        self.assertIn("NOT ADOPTED", self.verdict(out))

    def test_diagnostic_decides_nothing(self):
        recs = records(arm="mibz2-workers-lazy", threads=(8, 9, 10), step={10: 0.9})
        rc, out = self.run_on(
            recs,
            "--arm",
            "mibz2-workers-lazy",
            "--judged",
            "",
            "--supporting",
            "8,9,10",
            "--diagnostic",
        )
        self.assertEqual(rc, 0, out)
        self.assertNotIn("verdict:", out)
        self.assertIn("FASTER", out)

    def test_dry_labels_and_skips_the_machine(self):
        rc, out = self.run_on(
            records(step={9: 0.95}),
            "--dry",
            lscpu="Model name: Intel Xeon\n",
            env={**ENV, "cpus": 4},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("DRY: not applied: lscpu", out)
        self.assertIn("DRY verdict (not evidence): ADOPTS", out)
        self.assertNotIn("\nverdict:", out)

    def assert_void(self, recs, *extra, **kw):
        rc, out = self.run_on(recs, *extra, **kw)
        self.assertEqual(rc, 1, out)
        self.assertTrue(out.startswith("VOID") or out.startswith("DRY: VOID"), out)
        self.assertNotIn("verdict:", out)
        self.assertNotIn("verdict (not evidence):", out)
        return out

    def test_void_plan_moved(self):
        self.assertIn("plan moved", self.assert_void(records(step={9: 0.9}, workers={10: 10})))

    def test_void_workers_unobserved(self):
        recs = records(step={9: 0.9})
        del recs[4]["observed_threads"]
        self.assertIn("observed_threads", self.assert_void(recs))

    def test_void_missing_record(self):
        recs = records(step={9: 0.9})
        self.assertIn("missing", self.assert_void(recs[:-1]))

    def test_void_duplicate(self):
        recs = records(step={9: 0.9})
        self.assertIn("duplicated", self.assert_void(recs + recs[:1]))

    def test_void_unexpected_arm(self):
        recs = records(step={9: 0.9})
        recs[0]["arm"] = "mibz2@/elsewhere/ab"
        self.assert_void(recs)

    def test_void_failed_run(self):
        for field, bad in (
            ("termination", "signal 9"),
            ("err", "boom"),
            ("wall", 0),
            ("rss_kb", float("nan")),
            ("cpu", None),
        ):
            recs = records(step={9: 0.9})
            recs[5][field] = bad
            self.assert_void(recs)

    def test_void_pinned_or_spinner(self):
        recs = records(step={9: 0.9})
        recs[3]["cpus"] = "0,1,2,3,4,5"
        self.assertIn("not unpinned", self.assert_void(recs))
        recs = records(step={9: 0.9})
        recs[3]["held_spinner"] = {"cpu": 19}
        self.assertIn("spinner", self.assert_void(recs))

    def test_void_outputs(self):
        recs = records(step={9: 0.9})
        recs[7]["hash"] = "ffff"
        self.assertIn("outputs differ", self.assert_void(recs))
        recs = records(step={9: 0.9})
        recs[7]["out"] = 1
        self.assertIn("wrote", self.assert_void(recs))
        recs = records(step={9: 0.9})
        recs[7]["corpus"] = "text"
        self.assert_void(recs)

    def test_void_machine(self):
        self.assert_void(records(step={9: 0.9}), lscpu="Model name: AMD Ryzen 9 7950X\n")
        self.assert_void(records(step={9: 0.9}), env={**ENV, "cpus": 16})
        self.assert_void(records(step={9: 0.9}), env={**ENV, "cpu_quota": 4.0})

    def test_dry_still_voids_the_records(self):
        recs = records(step={9: 0.9})
        self.assert_void(recs[:-1], "--dry")

    def test_void_unreadable(self):
        rc, out = self.run_on([])
        self.assertEqual(rc, 1)
        p = self.tmp / "runs.jsonl"
        p.write_text("not json\n")
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = sp.main(
                [
                    str(p),
                    "--env",
                    str(self.tmp / "env.json"),
                    "--lscpu",
                    str(self.tmp / "lscpu.txt"),
                ]
            )
        self.assertEqual(rc, 1)


# Journal §84's test 2 (the combined fix, master merged in, the spin warm-up): the judged counts
# moved to production's defaults, 10 and 12, the rest supporting, as the recipe passes them
S84 = ("--judged", "10,12", "--supporting", "6,8,9")
GOOD84 = {10: 0.97, 12: 0.97}


class Section84(Verdict):
    def test_adopts(self):
        rc, out = self.run_on(records(step=GOOD84), *S84)
        self.assertEqual(rc, 0, out)
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")
        row = next(ln for ln in out.splitlines() if ln.strip().startswith("10  judged"))
        self.assertIn("FASTER", row)

    def test_nine_level_is_enough(self):
        # 9 is supporting now: level there adopts
        rc, out = self.run_on(records(step=GOOD84), *S84)
        row = next(ln for ln in out.splitlines() if ln.strip().startswith("9  supporting"))
        self.assertIn("INCONCLUSIVE", row)
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")

    def test_each_judged_count_must_be_faster(self):
        rc, out = self.run_on(records(step={10: 0.97}), *S84)
        self.assertIn("NOT ADOPTED (10/12 threads not FASTER)", self.verdict(out))

    def test_slower_at_a_supporting_count_not_adopted(self):
        for t in (6, 8, 9):
            rc, out = self.run_on(records(step={**GOOD84, t: 1.05}), *S84)
            self.assertIn(f"NOT ADOPTED (SLOWER at {t} threads)", self.verdict(out))

    def test_clock_split_reported_not_judged(self):
        rc, out = self.run_on(records(step=GOOD84, clock={10: 0.98}), *S84)
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")
        row = next(ln for ln in out.splitlines() if ln.strip().startswith("10  clock"))
        self.assertIn("clock 0.9800", row)
        common = float(row.split("wall at a common clock ")[1].split()[0])
        self.assertAlmostEqual(common, 0.97 * 0.98, delta=0.0005)
        rc, out = self.run_on(records(step=GOOD84), *S84)
        self.assertIn("not recorded (a run without the clock", out)


if __name__ == "__main__":
    unittest.main()
