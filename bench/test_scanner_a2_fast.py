"""Frozen fast-A2 scanner registration: plan, score and source checks."""

from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
import unittest

from scanner_a2_fast import CELLS, GUARDS, ROUNDS, InvalidRun, materialize_sources, plan, score


def samples(scan=0.95):
    rows = []
    for n, (kind, corpus, k), arm in plan():
        ratio = scan if (kind, corpus) == ("phases", "wikidata") else 1
        wall = (ratio if arm == "head" else 1) * (1 + n / 10000)
        rows.append(
            dict(kind=kind, corpus=corpus, k=k, round=n, build=arm, wall_s=wall, scan_s=wall,
                 peak_kib=1000)
        )
    return rows


def git(cwd, *args):
    return subprocess.check_output(["git", *args], cwd=cwd, text=True).strip()


class Sources(unittest.TestCase):
    def test_merge_and_master_parent_from_shallow_runner(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            origin = root / "origin"
            git(root, "init", "-q", "-b", "master", origin)
            git(origin, "config", "user.email", "fixture@example.invalid")
            git(origin, "config", "user.name", "fixture")

            def commit(text):
                (origin / text).write_text(text)
                git(origin, "add", text)
                git(origin, "commit", "-qm", text)
                return git(origin, "rev-parse", "HEAD")

            commit("root")
            git(origin, "switch", "-qc", "pr")
            commit("change")
            git(origin, "switch", "-q", "master")
            base = commit("master")
            git(origin, "switch", "-q", "pr")
            git(origin, "merge", "-q", "--no-edit", "master")
            head = git(origin, "rev-parse", "HEAD")
            commit("recipe")
            runner = root / "runner"
            git(root, "clone", "-q", "--depth=1", "-b", "pr", origin.as_uri(), runner)
            materialize_sources(root / "good", runner, head, base)
            self.assertEqual(git(root / "good" / "head", "rev-parse", "HEAD"), head)
            self.assertEqual(git(root / "good" / "base", "rev-parse", "HEAD"), base)
            self.assertTrue((root / "good" / "head" / "change").exists())
            self.assertFalse((root / "good" / "base" / "change").exists())
            with self.assertRaisesRegex(InvalidRun, "master parent"):
                materialize_sources(root / "wrong", runner, head, head)


class Verdict(unittest.TestCase):
    def test_balanced_interleaved_grid(self):
        seq = list(plan())
        self.assertEqual(len(CELLS), 15)
        self.assertEqual(len(seq), 15 * ROUNDS * 2)
        for i in range(0, len(seq), 2):
            self.assertEqual(seq[i][:2], seq[i + 1][:2])
            self.assertNotEqual(seq[i][2], seq[i + 1][2])
        for cell in CELLS:
            self.assertEqual(sum(c == cell and a == "head" for _, c, a in seq[::2]), ROUNDS // 2)

    def test_pass_and_scan_bound(self):
        self.assertEqual(score(samples(0.95))["verdict"], "PASS")
        # A 2% cut misses the 3.0% bound.
        self.assertEqual(score(samples(0.98))["verdict"], "STOP")

    def test_overlapping_ranges_stop(self):
        rows = samples(0.95)
        for r in rows:
            if (r["kind"], r["corpus"]) == ("phases", "wikidata") and r["round"] % 2:
                r["scan_s"] *= 2
        self.assertEqual(score(rows)["verdict"], "STOP")

    def test_guard_rss_and_wall(self):
        for corpus in GUARDS:
            rows = samples()
            for r in rows:
                if (r["kind"], r["corpus"], r["build"]) == ("phases", corpus, "head"):
                    r["scan_s"] *= 1.021
            result = score(rows)
            self.assertEqual(result["verdict"], "STOP")
            self.assertTrue(any(corpus in m for m in result["findings"]))
        rows = samples()
        for r in rows:
            if (r["kind"], r["corpus"], r["build"]) == ("phases", "records", "head"):
                r["scan_s"] *= 1.019
        self.assertEqual(score(rows)["verdict"], "PASS")
        rows = samples()
        for r in rows:
            if r["kind"] == "real" and r["build"] == "head" and r["corpus"] == "runs":
                r["peak_kib"] = 1021
        self.assertEqual(score(rows)["verdict"], "STOP")
        rows = samples()
        for r in rows:
            if r["kind"] == "real" and r["build"] == "head" and (r["corpus"], r["k"]) == (
                "wikidata",
                9,
            ):
                r["wall_s"] *= 1.03
        self.assertEqual(score(rows)["verdict"], "STOP")

    def test_missing_duplicate_and_undefined(self):
        rows = samples()
        for bad in (rows[:-1], rows + [rows[-1]]):
            with self.assertRaises(InvalidRun):
                score(bad)
        rows[0].update(wall_s=0, scan_s=0)
        with self.assertRaises(InvalidRun):
            score(rows)


if __name__ == "__main__":
    unittest.main()
