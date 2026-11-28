"""Unit tests for bench/runs_front_i9.py's frozen scoring and source checks."""

from __future__ import annotations

from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))

import runs_front_i9 as rf  # noqa: E402

CAND = "/tmp/cand-ab"
REFS = {name: {"out": 1000, "sha256": "x", "hash": "h"} for name in ("wikidata", *rf.GUARD_CORPORA)}


def rows_for(ratios=None, rss=None, rounds=rf.ROUNDS, cells=rf.CELLS, jitter=0.004):
    """Synthetic paired rows: candidate wall = base * ratio (cell default 1.0), small jitter."""
    ratios, rss = ratios or {}, rss or {}
    out = []
    for corpus, k in cells:
        for r in range(rounds):
            base = 1.0 + jitter * ((r * 7) % 5 - 2)
            cand = base * ratios.get((corpus, k), 1.0) * (1 + jitter * (-1) ** (r // 2) / 4)
            arms = [
                ("mibz2", base, 100_000),
                (f"mibz2@{CAND}", cand, 100_000 * rss.get((corpus, k), 1.0)),
            ]
            if r % 2:
                arms.reverse()
            for arm, wall, kib in arms:
                out.append(
                    {
                        "arm": arm,
                        "corpus": corpus,
                        "threads": k,
                        "round": r,
                        "wall": wall,
                        "rss_kb": kib,
                        "out": 1000,
                        "hash": "h",
                        "err": "",
                    }
                )
    return out


class Score(unittest.TestCase):
    def verdict(self, **kw):
        return rf.score(rows_for(**kw), REFS, cand_exe=CAND)

    def test_pass_on_primary_gain(self):
        s = self.verdict(ratios={("wikidata", 1): 0.97})
        self.assertEqual(s["verdict"], "PASS", s["findings"])
        self.assertEqual(s["primary_gains"], ["wikidata/1"])

    def test_stop_without_gain(self):
        s = self.verdict()
        self.assertEqual(s["verdict"], "STOP")
        self.assertIn("rule 1", s["findings"][0])

    def test_stop_on_primary_loss(self):
        s = self.verdict(ratios={("wikidata", 1): 0.97, ("text", 8): 1.03})
        self.assertEqual(s["verdict"], "STOP")
        self.assertTrue(any("rule 2: text/8" in f for f in s["findings"]))

    def test_secondary_up_to_five_percent_is_listed(self):
        s = self.verdict(ratios={("wikidata", 1): 0.97, ("runs", 1): 1.04})
        self.assertEqual(s["verdict"], "PASS", s["findings"])
        self.assertTrue(any(x.startswith("runs/1") for x in s["secondary_slower_points"]))

    def test_secondary_trade_needs_twice_the_gain_at_that_count(self):
        paid = self.verdict(ratios={("wikidata", 1): 0.85, ("onebyte", 1): 1.07})
        self.assertEqual(paid["verdict"], "PASS", paid["findings"])
        self.assertEqual(paid["secondary_traded"], ["onebyte/1"])
        unpaid = self.verdict(ratios={("wikidata", 1): 0.97, ("onebyte", 1): 1.07})
        self.assertEqual(unpaid["verdict"], "STOP")
        other_k = self.verdict(ratios={("wikidata", 1): 0.85, ("onebyte", 8): 1.07})
        self.assertEqual(other_k["verdict"], "STOP")

    def test_secondary_above_ten_percent_stops(self):
        s = self.verdict(ratios={("wikidata", 1): 0.70, ("pattern", 1): 1.11})
        self.assertEqual(s["verdict"], "STOP")

    def test_rss_bound(self):
        ok = self.verdict(ratios={("wikidata", 1): 0.97}, rss={("text", 1): 1.09})
        self.assertEqual(ok["verdict"], "PASS", ok["findings"])
        bad = self.verdict(ratios={("wikidata", 1): 0.97}, rss={("text", 1): 1.11})
        self.assertEqual(bad["verdict"], "STOP")

    def test_void_on_incomplete_duplicate_or_mismatch(self):
        rows = rows_for()
        with self.assertRaises(rf.InvalidRun):
            rf.score(rows[:-1], REFS, cand_exe=CAND)
        with self.assertRaises(rf.InvalidRun):
            rf.score(rows + rows[:1], REFS, cand_exe=CAND)
        bad = [dict(r) for r in rows]
        bad[5]["hash"] = "other"
        with self.assertRaises(rf.InvalidRun):
            rf.score(bad, REFS, cand_exe=CAND)
        foreign = [dict(r) for r in rows]
        foreign[1]["arm"] = "mibz2@/elsewhere"
        with self.assertRaises(rf.InvalidRun):
            rf.score(foreign, REFS, cand_exe=CAND)


class Source(unittest.TestCase):
    def repo(self):
        tmp = Path(tempfile.mkdtemp())

        def git(*a):
            return subprocess.run(
                ["git", "-c", "user.name=t", "-c", "user.email=t@t", *a],
                cwd=tmp,
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()

        git("init", "-q")
        (tmp / "src/runs").mkdir(parents=True)
        for f in ("src/runs.rs", "src/runs/tests.rs", "src/lib.rs", "Cargo.toml"):
            (tmp / f).write_text("a\n")
        git("add", "-A")
        git("commit", "-qm", "base")
        return tmp, git, git("rev-parse", "HEAD")

    def test_accepts_exactly_the_candidate_files(self):
        tmp, git, base = self.repo()
        for f in rf.CANDIDATE_FILES:
            (tmp / f).write_text("b\n")
        (tmp / "bench").mkdir()
        (tmp / "bench/x.py").write_text("recipe\n")
        git("add", "-A")
        git("commit", "-qm", "cand")
        self.assertEqual(rf.check_source(tmp, base)["decoder_diff"], sorted(rf.CANDIDATE_FILES))

    def test_refuses_other_decoder_changes_and_missing_candidate(self):
        tmp, git, base = self.repo()
        (tmp / "src/runs.rs").write_text("b\n")
        (tmp / "src/lib.rs").write_text("b\n")
        git("add", "-A")
        git("commit", "-qm", "cand")
        with self.assertRaises(rf.InvalidRun):
            rf.check_source(tmp, base)
        tmp2, git2, base2 = self.repo()
        with self.assertRaises(rf.InvalidRun):
            rf.check_source(tmp2, base2)

    def test_refuses_dirty_tree(self):
        tmp, git, base = self.repo()
        (tmp / "src/lib.rs").write_text("dirty\n")
        with self.assertRaises(rf.InvalidRun):
            rf.check_source(tmp, base)


class Registration(unittest.TestCase):
    def test_constants_match_the_registration(self):
        text = rf.REGISTRATION.read_text(encoding="utf-8")
        self.assertIn(rf.BASE, text)
        self.assertIn("16 paired rounds", text)
        self.assertIn("1.050", text)
        self.assertIn("1.100", text)
        self.assertIn("≤ 1.10", text)
        self.assertEqual(rf.ROUNDS, 16)
        self.assertEqual(rf.WIKIDATA_COUNTS, (1, 6, 8, 10))
        self.assertEqual(rf.GUARD_COUNTS, (1, 8))
        self.assertEqual(len(rf.CELLS), 16)


if __name__ == "__main__":
    unittest.main()
