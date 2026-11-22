"""bench/open_graded.py on synthetic runs: each verdict where the registration puts it, and every
defect the validity checks guard against leaves no verdict.

    python3 bench/test_open_graded.py
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
import open_graded as ov  # noqa: E402

R = 16
BASE_EXE = "/w/target/open-graded/build-base/target/release/examples/ab"
ENV = {
    "cpus": 20,
    "cores": 10,
    "core_cpus": [f"{c},{c + 10}" for c in range(10)],
    "cpu_quota": None,
}
LSCPU = "Architecture: x86_64\nModel name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"


def records(arm="mibz2", threads=(6, 8, 10), step=None, mem=1.0, noise=0.01):
    """A complete run: `step` maps a thread count to the head's wall over the base's (default
    level); `mem` its peak memory over the base's. Rounds alternate the noise above and below"""
    step = step or {}
    out = []
    for r in range(R):
        for t in threads:
            for which in ("base", "head"):
                wall = 1.0
                rss = 500_000
                if which == "head":
                    wall = step.get(t, 1.0) * (1 + (noise if r % 2 else -noise))
                    rss = int(rss * mem)
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
                        "out": ov.WIKIDATA_X4_OUT,
                        "hash": "0123456789abcdef",
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
            rc = ov.main(
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
        return next(line for line in out.splitlines() if line.startswith("verdict:"))

    def test_adopts(self):
        rc, out = self.run_on(records(step={10: 0.95, 8: 0.97, 6: 1.0}))
        self.assertEqual(rc, 0, out)
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")

    def test_level_at_ten_not_adopted(self):
        rc, out = self.run_on(records(step={8: 0.9, 6: 0.9}))
        self.assertEqual(rc, 0, out)
        self.assertIn("NOT ADOPTED (10 threads not FASTER)", out)

    def test_slower_at_six_not_adopted(self):
        rc, out = self.run_on(records(step={10: 0.95, 6: 1.05}))
        self.assertIn("NOT ADOPTED (SLOWER at 6 threads)", self.verdict(out))

    def test_peak_rss_reported(self):
        rc, out = self.run_on(records(step={10: 0.95}, mem=0.9))
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")
        line = next(ln for ln in out.splitlines() if ln.strip().startswith("10  base"))
        self.assertIn("base 488.3 MiB, head 439.5 MiB, head - base -48.8 MiB (0.9000)", line)

    def test_held_on_memory(self):
        rc, out = self.run_on(records(step={10: 0.95}, mem=1.2))
        self.assertIn("verdict: HELD ON MEMORY", out)

    def test_memory_at_the_gate_passes(self):
        rc, out = self.run_on(records(step={10: 0.95}, mem=1.10))
        self.assertEqual(self.verdict(out), "verdict: ADOPTS")

    def test_noisy_gain_not_adopted(self):
        rc, out = self.run_on(records(step={10: 0.99}, noise=0.05))
        self.assertIn("NOT ADOPTED", self.verdict(out))

    def test_diagnostic_decides_nothing(self):
        recs = records(arm="mibz2-workers-lazy", threads=(6, 8), step={6: 0.9})
        rc, out = self.run_on(
            recs,
            "--arm",
            "mibz2-workers-lazy",
            "--judged",
            "",
            "--supporting",
            "6,8",
            "--diagnostic",
        )
        self.assertEqual(rc, 0, out)
        self.assertNotIn("verdict:", out)
        self.assertIn("FASTER", out)

    def assert_void(self, recs, *extra, **kw):
        rc, out = self.run_on(recs, *extra, **kw)
        self.assertEqual(rc, 1, out)
        self.assertTrue(out.startswith("VOID"), out)
        self.assertNotIn("verdict:", out)
        return out

    def test_void_missing_record(self):
        recs = records(step={10: 0.9})
        self.assertIn("missing", self.assert_void(recs[:-1]))

    def test_void_duplicate(self):
        recs = records(step={10: 0.9})
        self.assertIn("duplicated", self.assert_void(recs + recs[:1]))

    def test_void_unexpected_arm(self):
        recs = records(step={10: 0.9})
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
            recs = records(step={10: 0.9})
            recs[5][field] = bad
            self.assert_void(recs)

    def test_void_pinned_or_spinner(self):
        recs = records(step={10: 0.9})
        recs[3]["cpus"] = "0,1,2,3,4,5"
        self.assertIn("not unpinned", self.assert_void(recs))
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
            rc = ov.main(
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
