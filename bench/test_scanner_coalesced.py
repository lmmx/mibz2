"""Frozen scanner CLI score and plan regression checks."""

from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

from scanner_coalesced import CELLS, ROUNDS, InvalidRun, materialize_sources, measure, plan, score


def samples():
    rows = []
    for n, (kind, corpus, k), arm in plan():
        ratio = 0.90 if kind == "phases" else 0.95 if (corpus, k) == ("wikidata", 10) else 1
        wall = (ratio if arm == "head" else 1) * (1 + n / 10000)
        rows.append(
            dict(
                kind=kind,
                corpus=corpus,
                k=k,
                round=n,
                build=arm,
                wall_s=wall,
                scan_s=wall,
                peak_kib=1000,
            )
        )
    return rows


class ScannerSources(unittest.TestCase):
    def test_pins_absent_from_shallow_runner(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)

            def git(cwd, *args):
                return subprocess.check_output(["git", *args], cwd=cwd, text=True).strip()

            origin = root / "origin"
            git(root, "init", "-q", "-b", "main", origin)
            git(origin, "config", "user.email", "fixture@example.invalid")
            git(origin, "config", "user.name", "fixture")

            def commit(text):
                (origin / "data").write_text(text)
                git(origin, "add", "data")
                git(origin, "commit", "-qm", text)
                return git(origin, "rev-parse", "HEAD")

            base, head = commit("baseline"), commit("squash")
            commit("recipe")
            runner = root / "runner"
            git(root, "clone", "-q", "--depth=1", origin.as_uri(), runner)
            self.assertEqual(git(runner, "rev-parse", "--is-shallow-repository"), "true")
            for sha in (head, base):
                self.assertNotEqual(
                    subprocess.run(["git", "cat-file", "-e", sha], cwd=runner).returncode, 0
                )
            materialize_sources(root / "good", runner, head, base)
            for arm, pin, content in (("head", head, "squash"), ("base", base, "baseline")):
                tree = root / "good" / arm
                self.assertEqual(git(tree, "rev-parse", "HEAD"), pin)
                self.assertEqual((tree / "data").read_text(), content)
            with self.assertRaisesRegex(InvalidRun, "first parent"):
                materialize_sources(root / "wrong-base", runner, head, head)


class ScannerVerdict(unittest.TestCase):
    def test_balanced_interleaved_grid(self):
        seq = list(plan())
        self.assertEqual(len(seq), 760)
        self.assertEqual(len(CELLS), 38)
        for i in range(0, len(seq), 2):
            self.assertEqual(seq[i][:2], seq[i + 1][:2])
            self.assertNotEqual(seq[i][2], seq[i + 1][2])
        for cell in CELLS:
            self.assertEqual(sum(c == cell and a == "head" for _, c, a in seq[::2]), ROUNDS // 2)

    def test_passing_frozen_grid(self):
        self.assertEqual(score(samples())["verdict"], "PASS")

    def test_missing_duplicate_and_undefined_measurements(self):
        rows = samples()
        for bad in (rows[:-1], rows + [rows[-1]]):
            with self.assertRaises(InvalidRun):
                score(bad)
        rows[0].update(wall_s=0, scan_s=0)
        with self.assertRaises(InvalidRun):
            score(rows)

    def test_paired_gain_with_overlapping_absolute_spreads_stops(self):
        rows = samples()
        for r in rows:
            if r["kind"] == "phases" and r["round"] % 2:
                r["scan_s"] *= 2
        self.assertEqual(score(rows)["verdict"], "STOP")

    def test_phase_decoder_error_is_void(self):
        with patch(
            "scanner_coalesced.subprocess.run",
            return_value=subprocess.CompletedProcess([], 1, "", "CRC mismatch"),
        ):
            with self.assertRaisesRegex(InvalidRun, "CRC mismatch"):
                measure("exe", ("phases", "wikidata", 1), "file", 0, "2-19", {})

    def test_secondary_loss_and_rss(self):
        rows = samples()
        for r in rows:
            if r["kind"] == "real" and r["build"] == "head" and r["k"] == 8:
                if r["corpus"] == "pattern":
                    r["wall_s"] *= 1.073
                if r["corpus"] == "text":
                    r["peak_kib"] = 1051
        result = score(rows)
        self.assertEqual(result["verdict"], "STOP")
        self.assertTrue(any("pattern/8" in m for m in result["findings"]))
        self.assertTrue(any("text/8" in m for m in result["findings"]))

    def test_ideal_wall_cannot_mask_real_regression_or_replace_scan_gate(self):
        rows = samples()
        for r in rows:
            if r["build"] == "head" and r["kind"] == "C":
                r["wall_s"] *= 0.3
            if r["build"] == "head" and r["kind"] == "phases":
                r["scan_s"] *= 1.1
        self.assertEqual(score(rows)["verdict"], "STOP")
