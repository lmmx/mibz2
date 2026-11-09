"""bench/run_fed_stage.py's admission verdict (schema 2: arms A, B and C31's D) and
bench/copies.sh's exit statuses, on
fixtures: the record examples/run_fed_stage.rs's own writer made (bench/fixtures/
run_fed_stage.json, checked against `record` by that example's test), passing, and each
defect the registration guards against failing it.

    python3 bench/test_run_fed_stage.py
"""

import bz2
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
COUNTEREXAMPLES = HERE / "fixtures" / "run_fed_stage_counterexamples.json"
CENSUS = HERE.parent / "docs" / "journal" / "2026-11-09-run-fed-direct" / "census.json"
HEAD = "0123456789abcdef0123456789abcdef01234567"
# the five synthetic corpora whose inputs don't depend on the head, as the verifier
# regenerates them (and as the binary's own record names them): SHA-256, compressed size,
# decoded size, blocks. `test_the_regeneration_matches` holds the port to them
PINNED = {
    "synthetic random": (
        "69a37c69ea1a77c38127de16424fd6e779b42e264dd964a011141d1505695198",
        1_774_781,
        2_000_000,
        3,
    ),
    "synthetic one byte": (
        "fe83f12dd7b68f263d44f27080c7c21168eaca3ef9c5f54a73a061b262068bb8",
        49,
        5_000_000,
        1,
    ),
    "synthetic pattern": (
        "bc2b45e48ffe998c4bd21a1a1aa653e27a96f7db55fafe26a8296d14da4acfd3",
        314,
        4_900_000,
        6,
    ),
    "synthetic runs": (
        "24141e111ba46d81122e4186124595690944c27ee40f31cc7c4aca52efd451ec",
        9_955,
        2_500_444,
        1,
    ),
    "synthetic records": (
        "616e54bd727b056d7ff6fedb2a13c9eb411f2c512cca52ac4a419c131b135c2b",
        2_397_517,
        2_400_000,
        3,
    ),
}


def base():
    return v.load(FIXTURE)


def evidence(rec=None):
    """The corpus evidence the fixture is checked against: Wikidata's pinned spec, the five
    pinned synthetic corpora, and the text corpus's as the fixture states it (the text is
    made from the head's sources, so its real evidence moves with every head; the CLI
    tests regenerate it)"""
    rec = rec or base()
    out = [v.wikidata_expected()]
    for f in rec["files"][1:]:
        if f["name"] in PINNED:
            sha, b, o, n = PINNED[f["name"]]
            out.append({"name": f["name"], "sha256": sha, "bytes": b, "output": o, "blocks": n})
        else:
            text = base()["files"][1]
            out.append({k: text[k] for k in ("sha256", "bytes", "output", "blocks")})
    return out


def problems(rec, head=HEAD, **kw):
    return v.check(rec, head, evidence(), **kw)


def apply(rec, case):
    for path, value in case.get("set", []):
        node = rec
        for k in path[:-1]:
            node = node[k]
        node[path[-1]] = value
    for path in case.get("delete", []):
        node = rec
        for k in path[:-1]:
            node = node[k]
        del node[path[-1]]
    return rec


def scale(rec, file, path, factor, arm=2):
    """An arm's entry time (D's by default) on one path of one file, times `factor`, every
    round"""
    for row in rec["files"][file]["entry"]:
        row[arm][path] *= factor


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
        for row in r["files"][3]["entry"]:  # synthetic one byte, all by records
            row[1][0] = 0.0
        self.assertInvalid(r, "no time for a path with blocks")

    def test_a_zero_total(self):
        r = base()
        r["files"][0]["entry"][4][0][0] = 0
        self.assertInvalid(r, "no time for a path with blocks")

    def test_a_negative_time(self):
        r = base()
        r["files"][0]["entry"][2][1][0] = -0.1
        self.assertInvalid(r, "not 3 arms x 2 finite times")

    def test_a_time_for_an_absent_path(self):
        r = base()
        r["files"][3]["entry"][0][0][1] = 0.01  # synthetic one byte has no fallback blocks
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

    def test_missing_placement_times(self):
        r = base()
        r["files"][0]["phases"][5][2][2] = 0.0  # D's placement
        self.assertInvalid(r, "D direct build: not timed")

    def test_a_sort_time_for_d(self):
        r = base()
        r["files"][0]["phases"][2][2][1] = 0.01  # D has no sort
        self.assertInvalid(r, "D direct sort: a time where none is taken")

    def test_the_old_schema(self):
        r = base()
        r["schema"] = "run_fed_stage/1"
        self.assertInvalid(r, "not run_fed_stage/2")

    def test_the_old_arms(self):
        r = base()
        r["arms"] = ["A column", "B records", "C column+ckpt", "D floor"]
        self.assertInvalid(r, "arms")

    def test_no_output_size(self):
        r = base()
        del r["files"][0]["output"]
        self.assertInvalid(r, "path counts missing")

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
        w = v.wikidata_expected()
        other = {"sha256": "1" * 64, "bytes": w["bytes"], "blocks": w["blocks"]}
        self.assertInvalid(base(), "not the pinned spec's", wikidata_file=other)
        same = {k: w[k] for k in ("sha256", "bytes", "blocks")}
        self.assertEqual(problems(base(), wikidata_file=same), [])

    def test_the_wikidata_files_blocks(self):
        w = v.wikidata_expected()
        other = {"sha256": w["sha256"], "bytes": w["bytes"], "blocks": 430}
        self.assertInvalid(base(), "blocks 430", wikidata_file=other)

    def test_no_corpus_evidence(self):
        p = v.check(base(), HEAD, None)
        self.assertTrue(any("no corpus evidence" in x for x in p), p)

    def test_the_counterexamples(self):
        # each mutation INVALID, by check() and by the CLI, which prints no reading
        cases = json.loads(COUNTEREXAMPLES.read_text())["cases"]
        self.assertGreaterEqual(len(cases), 36)
        for case in cases:
            with self.subTest(case["name"]):
                self.assertInvalid(apply(base(), case), case["expect"])

    def test_the_counterexamples_print_no_reading(self):
        cases = json.loads(COUNTEREXAMPLES.read_text())["cases"]
        for case in cases[:6]:
            with self.subTest(case["name"]):
                rec = apply(base(), case)
                out = self.run_cli(rec, evidence(rec))
                self.assertEqual(out[0], 1)
                self.assertNotIn("VALID;", out[1])
                self.assertNotIn("Q1", out[1])

    def run_cli(self, rec, corpora):
        import contextlib
        import io
        from unittest import mock

        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
            json.dump(rec, f)
        buf = io.StringIO()
        try:
            with mock.patch.object(v, "expected_corpora", lambda root: corpora):
                with contextlib.redirect_stdout(buf):
                    code = v.main(["verify", f.name, "--head", HEAD, "--unjudged"])
        finally:
            os.unlink(f.name)
        return code, buf.getvalue()

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
        r["files"][0]["order"][3] = [2, 1, 0]
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
        self.assertEqual(r["sort_removal"], "D CHEAPER THAN B")
        w = r["files"][0]
        # D 0.85 of A on the records' blocks (0.5 s), equal on the fallback's (0.25 s)
        self.assertAlmostEqual(w["R_direct"][0], (0.85 * 0.5 + 0.25) / 0.75, delta=0.005)
        self.assertGreater(w["R_stage"][0], 1.0)

    def test_q1_reads_d_not_b(self):
        # B made as cheap as D's fixture figure and D as dear as B's: Q1 follows D
        rec = base()
        scale(rec, 0, 0, 1.16 / 0.85)
        scale(rec, 0, 0, 0.85 / 1.16, arm=1)
        r = v.readings(rec)
        self.assertLess(r["files"][0]["R_stage"][0], 0.95)
        self.assertNotEqual(r["Q1"], "FEASIBLE")
        self.assertEqual(r["sort_removal"], "D NOT CHEAPER THAN B")

    def test_the_budget_terms(self):
        # the fixture: D's placement 0.18 against B's fill 0.2 a round, so delta_direct < 0
        w = v.readings(base())["files"][0]
        for key in ("delta_direct", "e", "f", "D_minus_A", "B_minus_A_less_sort", "budget"):
            self.assertIn(key, w)
        self.assertAlmostEqual(w["budget"], v.BUDGET_269 + w["delta_direct"] - w["e"] - w["f"])
        self.assertLess(w["delta_direct"], 0)

    def test_the_fallback_screen_reads_r_fb_not_r_stage(self):
        # the review's counterexample: accepted blocks 20% worse, fallback equal. R_stage
        # rises on a synthetic corpus with both paths, yet R_fb is 1: the fallback screen
        # passes and the synthetic whole-stage screen is what fails
        rec = base()
        f = rec["files"][4]
        # synthetic pattern (6 blocks by records in the census) with one of them made a
        # fallback block: its rows, records, cells and stores taken from the records' path
        # (made-up counts that keep every bound)
        f.update(
            blocks_runs=5,
            blocks_fallback=1,
            rows_runs=4_083_334,
            records=40,
            cells=45,
            slots=4_083_520,
            d_vector=510_420,
            d_scalar=40,
            d_streams=35,
        )
        for row in f["entry"]:
            for a in range(3):
                row[a][1] = 0.5
            row[2][0] = row[0][0] * 1.2
        for row in f["phases"]:
            for a in range(3):
                row[a][3] = 0.25
        self.assertEqual(problems(rec), [])
        r = v.readings(rec)
        pat = next(x for x in r["files"] if x["name"] == "synthetic pattern")
        self.assertAlmostEqual(pat["R_fb_D"][0], 1.0, places=6)
        self.assertGreater(pat["R_direct"][0], 1.02)
        self.assertFalse(r["screen_synthetic"]["pass"])
        self.assertTrue(r["screen_fallback"]["pass"])
        self.assertEqual(r["Q1"], "STOP: a screen failed")

    def test_a_costly_fallback_fails_its_screen(self):
        rec = base()
        scale(rec, 2, 1, 1.05)  # synthetic random: all fallback
        r = v.readings(rec)
        self.assertFalse(r["screen_fallback"]["pass"])
        self.assertEqual(r["Q1"], "STOP: a screen failed")

    def test_a_wide_interval_passes_the_screen_as_a_screen_only(self):
        # mean about 1.01 with a wide interval passes the point-estimate screen: it is no
        # noninferiority claim, and the notes say so
        rec = base()
        for i, row in enumerate(rec["files"][4]["entry"]):  # synthetic pattern
            wide = 1.0 if i == 14 else 1.6 if i % 2 else 1 / 1.6
            row[2][0] = row[0][0] * 1.01 * wide
        r = v.readings(rec)
        pat = next(x for x in r["files"] if x["name"] == "synthetic pattern")
        self.assertAlmostEqual(pat["R_direct"][0], 1.01, places=6)
        self.assertLess(pat["R_direct"][1], 0.9)
        self.assertGreater(pat["R_direct"][2], 1.1)
        self.assertTrue(r["screen_synthetic"]["pass"])
        self.assertTrue(any("not a noninferiority claim" in n for n in v.NOTES))

    def test_no_saving_reads_stop(self):
        rec = base()
        scale(rec, 0, 0, 1 / 0.85)
        self.assertEqual(v.readings(rec)["Q1"], "STOP: no saving of 5% or more")

    def test_a_small_saving_reads_inconclusive(self):
        # a 4% saving at the entry point, with a wide interval: neither the gate nor the stop
        rec = base()
        for i, row in enumerate(rec["files"][0]["entry"]):
            wide = 1.0 if i == 14 else 1.1 if i % 2 else 1 / 1.1
            a = row[0][0] + row[0][1]
            row[2][0] = a * 0.96 * wide - row[2][1]
        r = v.readings(rec)
        g, lo, hi = r["files"][0]["R_direct"]
        self.assertAlmostEqual(g, 0.96, places=6)
        self.assertLess(lo, 0.95)
        self.assertEqual(r["Q1"], "STOP, INCONCLUSIVE")

    def test_the_gate_is_0_95(self):
        # a saving just past the gate reads FEASIBLE; just short of it, INCONCLUSIVE
        r = base()
        scale(r, 0, 0, 0.92 / 0.85)
        g = v.readings(r)
        self.assertLessEqual(g["files"][0]["R_direct"][0], 0.95)
        self.assertEqual(g["Q1"], "FEASIBLE")

    def test_the_cli(self):
        # the verifier's own evidence: the text corpus regenerated from this checkout. The
        # fixture's text is the census's sources' (any later source edit changes it), so
        # the fixture is VALID by the CLI only if those are this checkout's; with its text
        # entry made this checkout's, it is (the census's text is all by the fallback)
        ev = synthetic()
        text = next(x for x in ev if x["name"] == "synthetic text")
        same = all(base()["files"][1][k] == text[k] for k in ("sha256", "bytes", "output"))
        code = v.main(["verify", str(FIXTURE), "--head", HEAD, "--unjudged"])
        self.assertEqual(code, 0 if same else 1)
        rec = base()
        for k in ("sha256", "bytes", "output", "blocks"):
            rec["files"][1][k] = text[k]
        rec["files"][1]["blocks_fallback"] = text["blocks"]
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
            json.dump(rec, f)
        try:
            self.assertEqual(v.main(["verify", f.name, "--head", HEAD, "--unjudged"]), 0)
            self.assertEqual(v.main(["verify", f.name, "--head", "e" * 40]), 1)
        finally:
            os.unlink(f.name)

    def test_the_fixture_counts_are_the_census(self):
        # the passing fixture's counts and identities are the committed count-only census
        census = json.loads(CENSUS.read_text())
        keys = [k for k in census["files"][0] if k != "name"]
        for f, c in zip(base()["files"], census["files"]):
            self.assertEqual({k: f[k] for k in keys}, {k: c[k] for k in keys}, f["name"])
        self.assertEqual(len(census["files"]), 7)
        self.assertNotIn("entry", census["files"][0])
        self.assertNotIn("phases", census["files"][0])

    def test_the_regeneration_matches(self):
        # the port of tests/common's corpora(1) and its compression: the five head-free
        # corpora as pinned (each also the binary's own record's figure when it ran), and
        # the text at least its 3,000,000 bytes, of this checkout's sources
        ev = {x["name"]: x for x in synthetic()}
        for name, (sha, b, o, n) in PINNED.items():
            self.assertEqual(
                (ev[name]["sha256"], ev[name]["bytes"], ev[name]["output"], ev[name]["blocks"]),
                (sha, b, o, n),
                name,
            )
        self.assertGreaterEqual(ev["synthetic text"]["output"], 3_000_000)
        self.assertEqual(v.count_blocks(bz2.compress(b"x" * 10, 9)), 1)
        self.assertEqual(v.count_blocks(bz2.compress(b"x" * 10, 9) * 3), 3)


_SYNTHETIC = []


def synthetic():
    """The verifier's regenerated synthetic evidence, made once for the tests"""
    if not _SYNTHETIC:
        _SYNTHETIC.extend(v.synthetic_expected(str(HERE.parent)))
    return _SYNTHETIC


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
