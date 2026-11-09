"""bench/run_fed_stage.py's admission verdict and bench/copies.sh's exit statuses, on
fixtures: the record examples/run_fed_stage.rs's own writer made (bench/fixtures/
run_fed_stage.json, checked against `record` by that example's test), passing, and each
defect the registration guards against failing it.

    python3 bench/test_run_fed_stage.py
"""

import copy
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import run_fed_stage as v  # noqa: E402

FIXTURE = HERE / "fixtures" / "run_fed_stage.json"
HEAD = "0123456789abcdef0123456789abcdef01234567"


def base():
    return v.load(FIXTURE)


def problems(rec, head=HEAD, **kw):
    return v.check(rec, head, **kw)


def scale_b(rec, file, path, factor):
    """B's entry time on one path of one file, times `factor`, every round"""
    for row in rec["files"][file]["entry"]:
        row[1][path] *= factor


class Admission(unittest.TestCase):
    def assertInvalid(self, rec, needle, **kw):
        p = problems(rec, **kw)
        self.assertTrue(p, "accepted")
        self.assertTrue(any(needle in x for x in p), f"{needle!r} not in {p}")

    def test_the_fixture_is_valid(self):
        self.assertEqual(problems(base()), [])

    def test_a_missing_synthetic_corpus(self):
        r = base()
        del r["files"][3]
        self.assertInvalid(r, "not the registered 7")

    def test_every_synthetic_corpus_absent(self):
        r = base()
        r["files"] = r["files"][:1]
        self.assertInvalid(r, "not the registered 7")

    def test_a_renamed_corpus(self):
        r = base()
        r["files"][2]["name"] = "synthetic other"
        self.assertInvalid(r, "not the registered 'synthetic random'")

    def test_an_absent_interval(self):
        # a corpus's whole-stage times all 0: no interval, never worst = 0
        r = base()
        for row in r["files"][1]["entry"]:
            row[1][0] = 0.0
        self.assertInvalid(r, "no time for a path with blocks")

    def test_a_zero_total(self):
        r = base()
        r["files"][0]["entry"][4][0][0] = 0
        self.assertInvalid(r, "no time for a path with blocks")

    def test_a_negative_time(self):
        r = base()
        r["files"][0]["entry"][2][1][0] = -0.1
        self.assertInvalid(r, "not 4 arms x 2 finite times")

    def test_a_time_for_an_absent_path(self):
        r = base()
        r["files"][1]["entry"][0][0][1] = 0.01  # synthetic text has no fallback blocks
        self.assertInvalid(r, "a time for a path with no blocks")

    def test_a_nonfinite_time_does_not_load(self):
        text = FIXTURE.read_text().replace("4.96e-1", "NaN", 1)
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
            f.write(text)
        try:
            with self.assertRaises(ValueError):
                v.load(f.name)
            self.assertEqual(v.main(["verify", f.name, "--head", HEAD]), 1)
        finally:
            os.unlink(f.name)

    def test_a_missing_record_is_invalid(self):
        self.assertEqual(v.main(["verify", "/nonexistent/record.json", "--head", HEAD]), 1)

    def test_a_missing_round(self):
        r = base()
        del r["files"][0]["entry"][7]
        self.assertInvalid(r, "entry times for 14 rounds")

    def test_missing_phase_times(self):
        r = base()
        r["files"][0]["phases"][3][1][1] = 0.0  # B's sort
        self.assertInvalid(r, "B records sort: not timed")

    def test_the_wrong_rounds(self):
        r = base()
        r["rounds"] = 14
        self.assertInvalid(r, "rounds 14")

    def test_rounds_as_a_bool(self):
        r = base()
        r["rounds"] = True
        self.assertInvalid(r, "rounds True")

    def test_no_avx2(self):
        r = base()
        r["kernels"]["avx2"] = False
        self.assertInvalid(r, "AVX2 not selected")

    def test_a_short_head(self):
        r = base()
        r["head"] = HEAD[:7]
        self.assertInvalid(r, "not a full 40-character commit")

    def test_a_short_registered_head(self):
        self.assertInvalid(base(), "registered head", head=HEAD[:12])

    def test_another_head(self):
        self.assertInvalid(base(), "not the registered", head="f" * 40)

    def test_no_head(self):
        r = base()
        r["head"] = None
        self.assertInvalid(r, "not a full 40-character commit")

    def test_another_wikidata_file(self):
        r = base()
        r["files"][0]["sha256"] = "0" * 64
        self.assertInvalid(r, "not the registered 82f65ace")

    def test_the_wikidata_files_own_digest(self):
        self.assertInvalid(base(), "not the file's", wikidata_sha256="1" * 64)

    def test_a_synthetic_digest_missing(self):
        r = base()
        r["files"][4]["sha256"] = ""
        self.assertInvalid(r, "SHA-256 '' not recorded")

    def test_correctness_not_passed(self):
        r = base()
        r["correctness"] = "failed"
        self.assertInvalid(r, "correctness 'failed'")

    def test_the_arms_order_not_rotated(self):
        r = base()
        r["files"][0]["order"][3] = [0, 1, 2, 3]
        self.assertInvalid(r, "not the registered rotation")

    def test_blocks_that_dont_add_up(self):
        r = base()
        r["files"][0]["blocks_fallback"] += 1
        self.assertInvalid(r, "blocks, not")

    def test_an_unpinned_primary(self):
        r = base()
        r["affinity"] = "0-3"
        self.assertInvalid(r, "one copy pinned to one CPU")
        self.assertEqual(problems(r, primary=False), [])

    def test_no_host(self):
        r = base()
        r["host"] = "unknown"
        self.assertInvalid(r, "host 'unknown'")


class Readings(unittest.TestCase):
    def test_the_fixture_reads_feasible(self):
        r = v.readings(base())
        self.assertEqual(r["Q1"], "FEASIBLE")
        self.assertTrue(r["screen_fallback"]["pass"])

    def test_the_fallback_screen_reads_r_fb_not_r_stage(self):
        # the review's counterexample: accepted blocks 20% worse, fallback equal. R_stage
        # rises on a synthetic corpus with both paths, yet R_fb is 1: the fallback screen
        # passes and the synthetic whole-stage screen is what fails
        rec = base()
        f = rec["files"][1]
        f["blocks_fallback"], f["blocks"] = 1, f["blocks"] + 1
        for row in f["entry"]:
            for a in range(4):
                row[a][1] = 0.5
            row[1][0] = row[0][0] * 1.2
        for row in f["phases"]:
            for a in range(4):
                row[a][3] = 0.25
        self.assertEqual(v.check(rec, HEAD), [])
        r = v.readings(rec)
        text = next(x for x in r["files"] if x["name"] == "synthetic text")
        self.assertAlmostEqual(text["R_fb"][0], 1.0, places=6)
        self.assertGreater(text["R_stage"][0], 1.02)
        self.assertFalse(r["screen_synthetic"]["pass"])
        self.assertTrue(r["screen_fallback"]["pass"])
        self.assertEqual(r["Q1"], "STOP: a screen failed")

    def test_a_costly_fallback_fails_its_screen(self):
        rec = base()
        scale_b(rec, 2, 1, 1.05)  # synthetic random: all fallback
        r = v.readings(rec)
        self.assertFalse(r["screen_fallback"]["pass"])
        self.assertEqual(r["Q1"], "STOP: a screen failed")

    def test_a_wide_interval_passes_the_screen_as_a_screen_only(self):
        # mean about 1.01 with a wide interval passes the point-estimate screen: it is no
        # noninferiority claim, and the notes say so
        rec = base()
        for i, row in enumerate(rec["files"][4]["entry"]):  # synthetic pattern
            wide = 1.0 if i == 14 else 1.6 if i % 2 else 1 / 1.6
            row[1][0] = row[0][0] * 1.01 * wide
        r = v.readings(rec)
        pat = next(x for x in r["files"] if x["name"] == "synthetic pattern")
        self.assertAlmostEqual(pat["R_stage"][0], 1.01, places=6)
        self.assertLess(pat["R_stage"][1], 0.9)
        self.assertGreater(pat["R_stage"][2], 1.1)
        self.assertTrue(r["screen_synthetic"]["pass"])
        self.assertTrue(any("not a noninferiority claim" in n for n in v.NOTES))

    def test_no_saving_reads_stop(self):
        rec = base()
        scale_b(rec, 0, 0, 1 / 0.85)
        self.assertEqual(v.readings(rec)["Q1"], "STOP: no saving of 5% or more")

    def test_a_small_saving_reads_inconclusive(self):
        rec = base()
        scale_b(rec, 0, 0, 0.93 / 0.85)
        self.assertEqual(v.readings(rec)["Q1"], "STOP, INCONCLUSIVE")

    def test_the_cli(self):
        self.assertEqual(v.main(["verify", str(FIXTURE), "--head", HEAD, "--unjudged"]), 0)
        self.assertEqual(v.main(["verify", str(FIXTURE), "--head", "e" * 40]), 1)


class Copies(unittest.TestCase):
    """bench/copies.sh: each copy's status collected, a failure never masked"""

    def run_copies(self, fail_cpu):
        d = tempfile.mkdtemp()
        cmd = [
            "bash",
            str(HERE / "copies.sh"),
            f"{d}/out",
            "header",
            "--",
            "bash",
            "-c",
            'echo ran; [ "$COPY_CPU" != "$FAIL_CPU" ]',
        ]
        env = dict(os.environ, FAIL_CPU=str(fail_cpu))
        p = subprocess.run(cmd, env=env, capture_output=True, text=True)
        return p, d

    def test_all_copies_succeed(self):
        p, d = self.run_copies(-1)
        self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
        self.assertFalse(os.path.exists(f"{d}/out-INCOMPLETE.txt"))
        self.assertIn("ran", Path(f"{d}/out-cpu0.txt").read_text())

    def test_one_failed_copy_fails_the_run(self):
        p, d = self.run_copies(0)
        self.assertNotEqual(p.returncode, 0)
        self.assertIn("CPU 0: exit 1", p.stdout)
        self.assertIn("INCOMPLETE", Path(f"{d}/out-INCOMPLETE.txt").read_text())

    def test_a_bare_wait_would_have_masked_it(self):
        # the defect itself: `false & true & wait` succeeds
        p = subprocess.run(["bash", "-c", "false & true & wait"])
        self.assertEqual(p.returncode, 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
