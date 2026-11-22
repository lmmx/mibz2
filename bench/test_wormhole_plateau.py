"""bench/wormhole_plateau.py on synthetic runs: each verdict where the registration puts it,
the wall route before the memory route, and every defect the validity checks guard against
leaves no verdict. bench/test_open_r5.py's tests (#735), the nobi pass's removed, with the
memory route's added.

    python3 bench/test_wormhole_plateau.py
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
import wormhole_plateau as ov  # noqa: E402

R = 12
BASE_EXE = "/w/target/wormhole-plateau/build-base/target/release/examples/ab"
ALL = (6, 8, 10, 12, 16, 20)
ENV = {
    "cpus": 20,
    "cores": 10,
    "core_cpus": [f"{c},{c + 10}" for c in range(10)],
    "cpu_quota": None,
}
LSCPU = "Architecture: x86_64\nModel name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"


def records(arm="mibz2", threads=ALL, step=None, mem=1.0, noise=0.01, mem_noise=0.0):
    """A complete run: `step` maps a thread count to the head's wall over the base's (default
    level); `mem` its peak memory over the base's, a number or a map by count. Rounds alternate
    the noise above and below"""
    step = step or {}
    out = []
    for r in range(R):
        for t in threads:
            for which in ("base", "head"):
                wall = 1.0
                rss = 500_000
                if which == "head":
                    wall = step.get(t, 1.0) * (1 + (noise if r % 2 else -noise))
                    m = mem.get(t, 1.0) if isinstance(mem, dict) else mem
                    rss = int(rss * m * (1 + (mem_noise if r % 2 else -mem_noise)))
                out.append(
                    {
                        "round": r,
                        "corpus": "wikidata-x4",
                        "threads": t,
                        "arm": arm if which == "head" else f"{arm}@{BASE_EXE}",
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
        self.assertIn("verdict: ADOPTS (wall", out)

    def test_wall_route_first(self):
        # both routes hold: the wall route is the one named
        rc, out = self.run_on(records(step={10: 0.95}, mem=0.7))
        self.assertIn("verdict: ADOPTS (wall", out)

    def test_level_at_ten_not_adopted(self):
        rc, out = self.run_on(records(step={8: 0.9, 6: 0.9}))
        self.assertEqual(rc, 0, out)
        self.assertIn("NOT ADOPTED (10 threads not FASTER", out)

    def test_slower_at_a_supporting_count_not_adopted(self):
        for t in (6, 8, 12, 16, 20):
            rc, out = self.run_on(records(step={10: 0.95, t: 1.05}))
            self.assertIn(f"SLOWER at {t} threads", self.verdict(out))
            self.assertIn("NOT ADOPTED", self.verdict(out))

    def test_peak_rss_reported(self):
        rc, out = self.run_on(records(step={10: 0.95}, mem=0.9))
        self.assertIn("verdict: ADOPTS (wall", out)
        line = next(ln for ln in out.splitlines() if ln.strip().startswith("10  base"))
        self.assertIn("base 488.3 MiB, head 439.5 MiB, head - base -48.8 MiB (0.9000)", line)

    def test_held_on_memory(self):
        rc, out = self.run_on(records(step={10: 0.95}, mem=1.2))
        self.assertIn("verdict: HELD ON MEMORY", out)

    def test_memory_at_the_gate_passes(self):
        rc, out = self.run_on(records(step={10: 0.95}, mem=1.10))
        self.assertIn("verdict: ADOPTS (wall", out)

    def test_noisy_gain_not_adopted(self):
        rc, out = self.run_on(records(step={10: 0.99}, noise=0.05))
        self.assertIn("NOT ADOPTED", self.verdict(out))

    def test_memory_route_adopts(self):
        # level wall everywhere, peak RSS 34% lower at 10 (the source's 10-worker figure)
        rc, out = self.run_on(records(mem={10: 0.66, 16: 0.56}, mem_noise=0.02))
        self.assertEqual(rc, 0, out)
        self.assertIn("verdict: ADOPTS (memory", out)

    def test_memory_route_needs_the_judged_count(self):
        # lower at 16 and 20 only: the judged count decides
        rc, out = self.run_on(records(mem={16: 0.5, 20: 0.5}))
        self.assertIn("NOT ADOPTED", self.verdict(out))

    def test_memory_route_needs_beyond_noise(self):
        # 0.89 at the point, but its interval reaches above 0.90
        rc, out = self.run_on(records(mem={10: 0.89}, mem_noise=0.05))
        self.assertIn("NOT ADOPTED", self.verdict(out))
        self.assertIn("not below 0.9 beyond noise", self.verdict(out))

    def test_memory_route_no_count_slower(self):
        for t in (6, 10, 20):
            rc, out = self.run_on(records(step={t: 1.05}, mem={10: 0.6}))
            self.assertIn("NOT ADOPTED", self.verdict(out))
            self.assertIn(f"SLOWER at {t} threads", self.verdict(out))

    def test_memory_route_over_the_gate_elsewhere(self):
        # a memory gain at 10 and growth past 1.10 at 6, wall level: the 1.10 gate holds on the
        # memory route too
        rc, out = self.run_on(records(mem={10: 0.6, 6: 1.2}))
        self.assertIn("NOT ADOPTED", self.verdict(out))
        self.assertIn("peak RSS above 1.1 at a count", self.verdict(out))

    def test_one_judged_count(self):
        self.assertIn(
            "one count",
            self.assert_void(records(), "--judged", "10,12", "--supporting", "6,8,16,20"),
        )

    def test_diagnostic_decides_nothing(self):
        rc, out = self.run_on(records(step={6: 0.9}), "--diagnostic")
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
