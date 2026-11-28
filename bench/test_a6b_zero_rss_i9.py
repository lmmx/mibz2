"""Unit tests for bench/a6b_zero_rss_i9.py's frozen scoring, quota, env, and source guards."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))

import a6b_zero_rss_i9 as az  # noqa: E402

CAND = "/tmp/cand-ab"
REFS = {name: {"out": 1000, "sha256": "x", "hash": "h"} for name in ("wikidata", *az.GUARD_CORPORA)}


def rows_for(
    wall_ratios=None,
    cpu_ratios=None,
    rss_ratios=None,
    rounds=az.ROUNDS,
    cells=az.CELLS,
    jitter=0.002,
):
    """Synthetic paired rows for the full 20-cell grid."""
    wall_ratios = wall_ratios or {}
    cpu_ratios = cpu_ratios or {}
    rss_ratios = rss_ratios or {}
    out = []
    for arm_mode, corpus, k in cells:
        key = (arm_mode, corpus, k)
        for r in range(rounds):
            base_w = 1.0 + jitter * ((r * 7) % 5 - 2)
            cand_w = base_w * wall_ratios.get(key, 1.0) * (1 + jitter * (-1) ** (r // 2) / 4)
            base_c = 1.0 + jitter * ((r * 5) % 5 - 2)
            cand_c = base_c * cpu_ratios.get(key, wall_ratios.get(key, 1.0))
            arms = [
                (arm_mode, base_w, base_c, 100_000),
                (f"{arm_mode}@{CAND}", cand_w, cand_c, 100_000 * rss_ratios.get(key, 1.0)),
            ]
            if r % 2:
                arms.reverse()
            for arm, wall, cpu, kib in arms:
                out.append(
                    {
                        "arm": arm,
                        "arm_mode": arm_mode,
                        "corpus": corpus,
                        "threads": k,
                        "round": r,
                        "wall": wall,
                        "cpu": cpu,
                        "rss_kb": kib,
                        "thread_cpu": f"mibz2-0={cpu:.2f} mibz2-scan=0.15",
                        "out": 1000,
                        "hash": "h",
                        "err": "",
                    }
                )
    return out


def passing_ratios():
    wall = {
        (az.LAZY_ARM, "wikidata", 1): 0.975,
        (az.LAZY_ARM, "wikidata", 6): 0.992,
        (az.LAZY_ARM, "wikidata", 8): 0.994,
        (az.DEFAULT_ARM, "wikidata", 6): 0.992,
        (az.DEFAULT_ARM, "wikidata", 8): 0.995,
    }
    cpu = {
        (az.LAZY_ARM, "wikidata", 1): 0.975,
        (az.LAZY_ARM, "wikidata", 6): 0.965,
        (az.LAZY_ARM, "wikidata", 8): 0.965,
        (az.DEFAULT_ARM, "wikidata", 6): 0.965,
        (az.DEFAULT_ARM, "wikidata", 8): 0.965,
    }
    return wall, cpu


class Score(unittest.TestCase):
    def verdict(self, wall=None, cpu=None, rss=None):
        return az.score(
            rows_for(wall_ratios=wall, cpu_ratios=cpu, rss_ratios=rss), REFS, cand_exe=CAND
        )

    def test_pass_when_all_four_rules_hold(self):
        wall, cpu = passing_ratios()
        s = self.verdict(wall=wall, cpu=cpu)
        self.assertEqual(s["verdict"], "PASS", s["findings"])

    def test_stop_when_wiki1_gain_too_small(self):
        wall, cpu = passing_ratios()
        wall[(az.LAZY_ARM, "wikidata", 1)] = 0.990
        s = self.verdict(wall=wall, cpu=cpu)
        self.assertEqual(s["verdict"], "STOP")
        self.assertTrue(any("rule 1" in f for f in s["findings"]))

    def test_stop_when_8w_cpu_or_wall_fails(self):
        wall, cpu = passing_ratios()
        cpu[(az.DEFAULT_ARM, "wikidata", 8)] = 0.995
        s = self.verdict(wall=wall, cpu=cpu)
        self.assertEqual(s["verdict"], "STOP")
        self.assertTrue(any("rule 2: mibz2/wikidata/8 cpu" in f for f in s["findings"]))

        wall2, cpu2 = passing_ratios()
        wall2[(az.LAZY_ARM, "wikidata", 8)] = 1.005
        s2 = self.verdict(wall=wall2, cpu=cpu2)
        self.assertEqual(s2["verdict"], "STOP")
        self.assertTrue(
            any("rule 2: mibz2-workers-lazy/wikidata/8 wall" in f for f in s2["findings"])
        )

    def test_stop_on_rss_over_1005(self):
        wall, cpu = passing_ratios()
        ok = self.verdict(wall=wall, cpu=cpu, rss={(az.DEFAULT_ARM, "wikidata", 8): 1.004})
        self.assertEqual(ok["verdict"], "PASS", ok["findings"])
        bad = self.verdict(wall=wall, cpu=cpu, rss={(az.DEFAULT_ARM, "wikidata", 8): 1.006})
        self.assertEqual(bad["verdict"], "STOP")
        self.assertTrue(any("rule 3" in f for f in bad["findings"]))

    def test_stop_on_guard_regression_over_1010(self):
        wall, cpu = passing_ratios()
        wall[(az.DEFAULT_ARM, "text", 8)] = 1.015
        s = self.verdict(wall=wall, cpu=cpu)
        self.assertEqual(s["verdict"], "STOP")
        self.assertTrue(any("rule 4: mibz2/text/8" in f for f in s["findings"]))

    def test_void_on_incomplete_duplicate_or_mismatch(self):
        wall, cpu = passing_ratios()
        rows = rows_for(wall_ratios=wall, cpu_ratios=cpu)
        with self.assertRaises(az.InvalidRun):
            az.score(rows[:-1], REFS, cand_exe=CAND)
        with self.assertRaises(az.InvalidRun):
            az.score(rows + rows[:1], REFS, cand_exe=CAND)
        bad = [dict(r) for r in rows]
        bad[3]["hash"] = "other"
        with self.assertRaises(az.InvalidRun):
            az.score(bad, REFS, cand_exe=CAND)

    def test_score_dir_void_on_smoke_or_wrong_rounds(self):
        with tempfile.TemporaryDirectory() as td:
            out = Path(td)
            wall, cpu = passing_ratios()
            rows = rows_for(wall_ratios=wall, cpu_ratios=cpu, rounds=2)
            (out / "runs.jsonl").write_text("\n".join(json.dumps(r) for r in rows) + "\n")
            (out / "refs.json").write_text(json.dumps(REFS))
            (out / "provenance.json").write_text(
                json.dumps(
                    {
                        "smoke": True,
                        "rounds": 2,
                        "base": az.BASE,
                        "settle_s": "0",
                        "prespin_ms": 0,
                    }
                )
            )
            self.assertEqual(az.score_dir(out)["verdict"], "VOID")


class Guards(unittest.TestCase):
    def test_quota_check_handles_none_dict_and_fails_closed(self):
        az.check_no_quota({"cpu_quota": None})
        az.check_no_quota({"cpu_quota": {"state": "unlimited", "cpus": None}})
        for bad in (
            {},
            {"cpu_quota": 4.0},
            {"cpu_quota": {"state": "limited", "cpus": 8.0}},
            {"cpu_quota": {"state": "unknown", "cpus": None}},
            {"cpu_quota": "unlimited"},
        ):
            with self.assertRaises(az.InvalidRun):
                az.check_no_quota(bad)

    def test_baseline_env_strict_forbids_rustflags_and_profile_overrides(self):
        az.check_baseline_env_strict({})
        for bad_env in (
            {"RUSTFLAGS": "-C target-cpu=native"},
            {"CARGO_BUILD_RUSTFLAGS": "-C target-cpu=native"},
            {"CARGO_PROFILE_RELEASE_LTO": "false"},
            {"CARGO_PROFILE_RELEASE_CODEGEN_UNITS": "16"},
        ):
            with self.assertRaises(az.InvalidRun):
                az.check_baseline_env_strict(bad_env)


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
        (tmp / "src/open").mkdir(parents=True)
        for f in ("src/open.rs", "src/open/endpoint.rs", "src/lib.rs", "Cargo.toml"):
            (tmp / f).write_text("a\n")
        git("add", "-A")
        git("commit", "-qm", "base")
        return tmp, git, git("rev-parse", "HEAD")

    def test_accepts_exactly_candidate_files(self):
        tmp, git, base = self.repo()
        for f in az.CANDIDATE_FILES:
            (tmp / f).write_text("b\n")
        (tmp / "bench").mkdir()
        (tmp / "bench/a6b_zero_rss_i9.py").write_text("recipe\n")
        git("add", "-A")
        git("commit", "-qm", "cand")
        self.assertEqual(az.check_source(tmp, base)["decoder_diff"], sorted(az.CANDIDATE_FILES))

    def test_refuses_other_decoder_changes_and_dirty_tree(self):
        tmp, git, base = self.repo()
        (tmp / "src/open.rs").write_text("b\n")
        (tmp / "src/lib.rs").write_text("b\n")
        git("add", "-A")
        git("commit", "-qm", "cand")
        with self.assertRaises(az.InvalidRun):
            az.check_source(tmp, base)
        tmp2, _, base2 = self.repo()
        (tmp2 / "src/lib.rs").write_text("dirty\n")
        with self.assertRaises(az.InvalidRun):
            az.check_source(tmp2, base2)

    def test_shallow_checkout_fetches_base_and_skips_ancestry(self):
        src, git, base = self.repo()
        for f in az.CANDIDATE_FILES:
            (src / f).write_text("b\n")
        git("add", "-A")
        git("commit", "-qm", "cand")
        git("config", "uploadpack.allowAnySHA1InWant", "true")
        dst = Path(tempfile.mkdtemp()) / "clone"
        subprocess.run(
            ["git", "clone", "-q", "--depth", "1", f"file://{src}", str(dst)],
            check=True,
            capture_output=True,
        )
        info = az.check_source(dst, base)
        self.assertTrue(info["shallow"])
        self.assertIsNone(info["ancestry_checked"])
        self.assertEqual(info["decoder_diff"], sorted(az.CANDIDATE_FILES))
        with self.assertRaises(az.InvalidRun):
            az.check_source(dst, "0" * 40)


class Registration(unittest.TestCase):
    def test_constants_match_registration(self):
        text = az.REGISTRATION.read_text(encoding="utf-8")
        self.assertIn(az.BASE, text)
        self.assertIn("12 paired rounds", text)
        self.assertIn("≤ 0.985", text)
        self.assertIn("≤ 0.990", text)
        self.assertIn("≤ 1.005", text)
        self.assertIn("≤ 1.010", text)
        self.assertEqual(az.ROUNDS, 12)
        self.assertEqual(len(az.CELLS), 20)


if __name__ == "__main__":
    unittest.main()
