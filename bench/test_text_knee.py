"""Unit and shallow-checkout tests for bench/text_knee.py (text-knee-i9)."""

import math
from pathlib import Path
import subprocess
import tempfile
import unittest

import text_knee as tk


def make_refs():
    return {
        "docs_md": {"base_out": 1000, "base_in": 250, "wall_out": 16000, "ab_hash": "h-docs"},
        "repo_json": {"base_out": 2000, "base_in": 400, "wall_out": 40000, "ab_hash": "h-json"},
        "src_rs": {"base_out": 3000, "base_in": 600, "wall_out": 60000, "ab_hash": "h-src"},
        "wikidata": {"base_out": 4000, "base_in": 225, "wall_out": 16000, "ab_hash": "h-wiki"},
    }


def make_wall_rows(curves, refs, rounds=tk.ROUNDS):
    rows = []
    for r in range(rounds):
        for corpus, by_k in curves.items():
            for k, wall in by_k.items():
                rows.append(
                    {
                        "arm": "mibz2",
                        "corpus": corpus,
                        "threads": k,
                        "round": r,
                        "wall": wall,
                        "rss_kb": 100_000 + 5_000 * k,
                        "out": refs[corpus]["wall_out"],
                        "hash": refs[corpus]["ab_hash"],
                        "err": "",
                        "termination": "success",
                    }
                )
    return rows


def make_station_rows(k_stars, refs, rounds=tk.STATION_ROUNDS):
    rows = []
    for r in range(rounds):
        for corpus, ks in k_stars.items():
            s_ms = 20.0
            w_ms = s_ms * ks
            rows.append(
                {
                    "corpus": corpus,
                    "round": r,
                    "blocks": 12,
                    "full_blocks": 11,
                    "syms": 500_000,
                    "out_bytes": refs[corpus]["base_out"],
                    "in_bytes": refs[corpus]["base_in"],
                    "s_ms": s_ms,
                    "w_ms": w_ms,
                    "k_star": ks,
                    "s_ns_per_sym": 2.5,
                    "w_ns_per_out": 4.0,
                }
            )
    return rows


def curve_with_knee(target_knee, best_wall=0.200):
    """Build a monotone wall curve whose 2% knee is exactly `target_knee`."""
    curve = {}
    for k in tk.COUNTS:
        if k < target_knee:
            curve[k] = best_wall * (1.05 + 0.05 * (target_knee - k))
        elif k == target_knee:
            curve[k] = best_wall * 1.01
        else:
            curve[k] = best_wall
    return curve


class TextKneeScoringTests(unittest.TestCase):
    def test_confirmed_panel_le_7_and_wikidata_ge_8(self):
        refs = make_refs()
        curves = {
            "docs_md": curve_with_knee(5),
            "repo_json": curve_with_knee(6),
            "src_rs": curve_with_knee(7),
            "wikidata": curve_with_knee(8),
        }
        wall = make_wall_rows(curves, refs)
        station = make_station_rows(
            {"docs_md": 4.9, "repo_json": 5.8, "src_rs": 6.4, "wikidata": 8.0}, refs
        )
        res = tk.score(wall, station, refs)
        self.assertEqual(res["verdict"], "CONFIRMED")
        self.assertEqual(res["panel_median_knee"], 6)
        self.assertEqual(res["wikidata_knee"], 8)
        self.assertAlmostEqual(res["panel_median_k_star"], 5.8)
        self.assertAlmostEqual(res["panel_median_r_vs_wikidata"], 5.8 / 8.0)
        text = tk.format_outcome(res)
        self.assertTrue(text.startswith("CONFIRMED:"))
        self.assertIn("docs_md: knee=5", text)
        self.assertIn("wikidata: knee=8", text)

    def test_exact_two_percent_knee_boundary(self):
        # k=7 at exactly 1.02 * best (0.204 vs 0.200) must qualify as knee=7;
        # k=7 at 1.0201 * best must push knee to 8.
        medians_in = {4: 0.30, 5: 0.26, 6: 0.23, 7: 0.204, 8: 0.200, 9: 0.200, 10: 0.200}
        _, _, _, knee_in = tk.compute_knee(medians_in)
        self.assertEqual(knee_in, 7)

        medians_out = {**medians_in, 7: 0.20402}
        _, _, _, knee_out = tk.compute_knee(medians_out)
        self.assertEqual(knee_out, 8)

    def test_refuted_when_panel_knee_ge_8_regardless_of_wikidata(self):
        refs = make_refs()
        station = make_station_rows(
            {"docs_md": 7.5, "repo_json": 8.1, "src_rs": 8.3, "wikidata": 8.2}, refs
        )
        for wiki_knee in (8, 7):
            curves = {
                "docs_md": curve_with_knee(7),
                "repo_json": curve_with_knee(8),
                "src_rs": curve_with_knee(9),
                "wikidata": curve_with_knee(wiki_knee),
            }
            res = tk.score(make_wall_rows(curves, refs), station, refs)
            self.assertEqual(res["verdict"], "REFUTED")
            self.assertEqual(res["panel_median_knee"], 8)
            self.assertEqual(res["wikidata_knee"], wiki_knee)

    def test_inconclusive_when_both_panel_and_wikidata_le_7(self):
        refs = make_refs()
        curves = {
            "docs_md": curve_with_knee(5),
            "repo_json": curve_with_knee(6),
            "src_rs": curve_with_knee(6),
            "wikidata": curve_with_knee(7),
        }
        station = make_station_rows(
            {"docs_md": 4.8, "repo_json": 5.5, "src_rs": 6.1, "wikidata": 7.2}, refs
        )
        res = tk.score(make_wall_rows(curves, refs), station, refs)
        self.assertEqual(res["verdict"], "INCONCLUSIVE")
        self.assertEqual(res["panel_median_knee"], 6)
        self.assertEqual(res["wikidata_knee"], 7)
        self.assertIn("k*=", res["reason"])

    def test_void_conditions(self):
        refs = make_refs()
        curves = {
            "docs_md": curve_with_knee(5),
            "repo_json": curve_with_knee(6),
            "src_rs": curve_with_knee(7),
            "wikidata": curve_with_knee(8),
        }
        base_wall = make_wall_rows(curves, refs)
        base_station = make_station_rows(
            {"docs_md": 5.0, "repo_json": 5.5, "src_rs": 6.0, "wikidata": 8.0}, refs
        )

        # Non-i9 host
        self.assertEqual(tk.score(base_wall, base_station, refs, host_ok=False)["verdict"], "VOID")

        # Missing one wall row
        self.assertEqual(tk.score(base_wall[:-1], base_station, refs)["verdict"], "VOID")

        # Duplicate wall row
        self.assertEqual(
            tk.score([*base_wall, dict(base_wall[0])], base_station, refs)["verdict"], "VOID"
        )

        # Decode error string
        bad_err = [dict(r) for r in base_wall]
        bad_err[0]["err"] = "crc mismatch"
        self.assertEqual(tk.score(bad_err, base_station, refs)["verdict"], "VOID")

        # Non-success termination
        bad_term = [dict(r) for r in base_wall]
        bad_term[0]["termination"] = "timeout"
        self.assertEqual(tk.score(bad_term, base_station, refs)["verdict"], "VOID")

        # Wrong output byte count
        bad_out = [dict(r) for r in base_wall]
        bad_out[0]["out"] += 1
        self.assertEqual(tk.score(bad_out, base_station, refs)["verdict"], "VOID")

        # Wrong output hash
        bad_hash = [dict(r) for r in base_wall]
        bad_hash[0]["hash"] = "wrong"
        self.assertEqual(tk.score(bad_hash, base_station, refs)["verdict"], "VOID")

        # NaN wall time
        bad_nan = [dict(r) for r in base_wall]
        bad_nan[0]["wall"] = math.nan
        self.assertEqual(tk.score(bad_nan, base_station, refs)["verdict"], "VOID")

        # Missing station_balance row
        self.assertEqual(tk.score(base_wall, base_station[:-1], refs)["verdict"], "VOID")

        # Wrong station_balance byte count
        bad_sb = [dict(r) for r in base_station]
        bad_sb[0]["out_bytes"] += 1
        self.assertEqual(tk.score(base_wall, bad_sb, refs)["verdict"], "VOID")

    def test_baseline_env_rejects_rustflags(self):
        tk.check_baseline_env({})
        for var in ("RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
            with self.assertRaises(tk.InvalidRun):
                tk.check_baseline_env({var: "-C target-cpu=native"})

    def test_cpu_quota_unlimited_accepts_facts_and_rejects_limits(self):
        tk.check_cpu_quota_unlimited({"cpu_quota": None})
        tk.check_cpu_quota_unlimited(
            {"cpu_quota": {"state": "unlimited", "cpus": None, "hierarchy": "v2"}}
        )
        for bad in (
            {},
            {"cpu_quota": 4.0},
            {"cpu_quota": {"state": "limit", "cpus": 8.0}},
            {"cpu_quota": {"state": "unknown", "cpus": None}},
        ):
            with self.assertRaises(tk.InvalidRun):
                tk.check_cpu_quota_unlimited(bad)

    def test_wall_plan_is_complete_and_deterministic(self):
        p1 = tk.wall_plan()
        p2 = tk.wall_plan()
        self.assertEqual(p1, p2)
        self.assertEqual(len(p1), tk.ROUNDS * len(tk.ALL_CORPORA) * len(tk.COUNTS))
        # Each round covers every (corpus, k) cell exactly once in non-constant order.
        orders = []
        for r in range(tk.ROUNDS):
            cells = [(c, k) for rr, _, c, k in p1 if rr == r]
            self.assertEqual(len(cells), len(tk.ALL_CORPORA) * len(tk.COUNTS))
            self.assertEqual(len(set(cells)), len(cells))
            orders.append(cells)
        self.assertGreater(len({tuple(o) for o in orders}), 1)

    def test_parse_station_balance_line(self):
        line = "docs.bz2\t13\t12\t6330000\t11480438\t2952993\t16.95\t66.40\t3.92\t2.678\t5.784\n"
        parsed = tk.parse_station_balance_line(line)
        self.assertEqual(parsed["blocks"], 13)
        self.assertEqual(parsed["full_blocks"], 12)
        self.assertEqual(parsed["out_bytes"], 11_480_438)
        self.assertAlmostEqual(parsed["k_star"], 3.92)
        with self.assertRaises(tk.InvalidRun):
            tk.parse_station_balance_line("too\tfew\tfields")

    def test_detached_shallow_runner_fetches_panel_commit(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            upstream = root / "upstream"
            runner = root / "runner"
            work_tmp = root / "work"
            work_tmp.mkdir()

            def git(repo, *args):
                return subprocess.check_output(["git", "-C", repo, *args], text=True).strip()

            subprocess.run(["git", "init", "-q", "-b", "master", upstream], check=True)
            git(upstream, "config", "user.name", "Test")
            git(upstream, "config", "user.email", "test@example.invalid")
            git(upstream, "config", "uploadpack.allowAnySHA1InWant", "true")

            (upstream / "docs").mkdir()
            (upstream / "docs/a.md").write_text("pinned doc alpha\n")
            (upstream / "src").mkdir()
            (upstream / "src/lib.rs").write_text("pub fn x() -> u32 { 1 }\n")
            git(upstream, "add", ".")
            git(upstream, "commit", "-qm", "pinned panel commit")
            pinned = git(upstream, "rev-parse", "HEAD")

            # Advance master with a second commit so a --depth=1 clone does NOT have `pinned`.
            (upstream / "docs/b.md").write_text("later doc beta\n")
            git(upstream, "add", ".")
            git(upstream, "commit", "-qm", "later master commit")

            subprocess.run(
                [
                    "git",
                    "clone",
                    "-q",
                    "--depth=1",
                    "--single-branch",
                    "--no-local",
                    "-b",
                    "master",
                    upstream.as_uri(),
                    runner,
                ],
                check=True,
            )
            runner_head = git(runner, "rev-parse", "HEAD")
            git(runner, "checkout", "-q", "--detach", runner_head)
            self.assertEqual(git(runner, "rev-parse", "--is-shallow-repository"), "true")
            self.assertNotEqual(
                subprocess.run(
                    ["git", "-C", runner, "cat-file", "-e", f"{pinned}^{{commit}}"],
                    capture_output=True,
                ).returncode,
                0,
            )

            fetched_repo = tk.materialize_panel_commit(work_tmp, runner, commit=pinned)
            self.assertNotEqual(fetched_repo, runner)
            blob = tk._read_blobs_batch(fetched_repo, pinned, ["docs/a.md", "src/lib.rs"])
            self.assertEqual(blob, b"pinned doc alpha\npub fn x() -> u32 { 1 }\n")
            # Shallow runner HEAD and history must remain untouched.
            self.assertEqual(git(runner, "rev-parse", "HEAD"), runner_head)

    def test_pinned_text_panel_sha256_when_commit_local(self):
        has_local = (
            subprocess.run(
                ["git", "-C", str(tk.ROOT), "cat-file", "-e", f"{tk.PANEL_COMMIT}^{{commit}}"],
                capture_output=True,
            ).returncode
            == 0
        )
        if not has_local:
            self.skipTest("pinned panel commit not in local shallow clone")
        with tempfile.TemporaryDirectory() as scratch:
            built = tk.build_text_panel(tk.ROOT, Path(scratch), tk.PANEL_COMMIT)
            self.assertEqual(tuple(built.keys()), tk.PANEL_CORPORA)
            for name in tk.PANEL_CORPORA:
                spec = tk.PANEL_SPECS[name]
                self.assertEqual(built[name]["base_bz2_sha256"], spec["base_bz2_sha256"])
                self.assertEqual(built[name]["wall_bz2_sha256"], spec["wall_bz2_sha256"])


if __name__ == "__main__":
    unittest.main()
