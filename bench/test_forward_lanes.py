"""Frozen unit and shallow-checkout tests for bench/forward_lanes.py (claim forward-emission-run)."""

from __future__ import annotations

import contextlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import forward_lanes as fl  # noqa: E402

GOOD_WIKI_SHA = "a" * 64
GOOD_TEXT_SHA = "b" * 64
BAD_SHA = "c" * 64


def _make_wall(
    corpus: str, k: int, block: int, arm: str, wall_s: float, rss: int = 200_000
) -> dict:
    ref = GOOD_WIKI_SHA if corpus == "wikidata" else GOOD_TEXT_SHA
    return {
        "build": arm,
        "measure": "wall",
        "section": corpus,
        "k": k,
        "block": block,
        "wall_s": wall_s,
        "bytes": 838_800_784 if corpus == "wikidata" else 17_977_996,
        "maxrss_kib": rss,
        "sha256": ref,
        "expected_sha256": ref,
    }


def _make_trace(
    corpus: str,
    k: int,
    block: int,
    arm: str,
    oracle: str,
    wall_s: float,
    scanner_busy_ns: int,
    worker_busy_ns_per_w: int = 80_000_000,
    rss: int = 260_000,
) -> dict:
    ref = GOOD_WIKI_SHA if corpus == "wikidata" else GOOD_TEXT_SHA
    decoded = 1724 if corpus == "wikidata" else 30
    return {
        "kind": "decode",
        "build": arm,
        "measure": oracle,
        "section": corpus,
        "k": k,
        "block": block,
        "wall_s": wall_s,
        "bytes": 838_800_784 if corpus == "wikidata" else 17_977_996,
        "maxrss_kib": rss,
        "scanner_pin_ok": True,
        "expected_sha256": ref,
        "check_pass": {
            "identical": True,
            "bytes": 838_800_784 if corpus == "wikidata" else 17_977_996,
            "sha256": ref,
        },
        "trace_summary": {
            "workers_seen": k,
            "blocks_decoded": decoded,
            "blocks_issued": decoded,
            "blocks_open_pass1": 932 if corpus == "wikidata" else 10,
            "worker_busy_ns_per_w": worker_busy_ns_per_w,
            "scanner_busy_ns": scanner_busy_ns,
        },
        "oracles": {
            "scan_locks": 0,
            "scanned": 0 if oracle == "S" else decoded,
            "replayed": decoded if oracle == "S" else 0,
            "recorded_blocks": decoded if oracle == "S" else 0,
        },
    }


def _synthetic_dataset(
    *,
    r1: float = 0.90,
    r8: float = 0.98,
    r8s: float = 0.93,
    r_text: float = 0.97,
    scan_residual_base: float = 0.01,
    scan_residual_cand: float = 0.01,
    repeats: int = fl.REPEATS,
) -> list[dict]:
    """Build a complete synthetic dataset with deterministic small paired jitter."""
    rows: list[dict] = []
    for block in range(repeats):
        jitter = 1.0 + ((block - repeats // 2) * 0.0005)
        rows.append(_make_wall("wikidata", 1, block, "base", 1.300 * jitter))
        rows.append(_make_wall("wikidata", 1, block, "cand", 1.300 * jitter * r1))
        rows.append(_make_wall("wikidata", 8, block, "base", 0.210 * jitter))
        rows.append(_make_wall("wikidata", 8, block, "cand", 0.210 * jitter * r8))
        rows.append(_make_wall("text", 1, block, "base", 0.095 * jitter))
        rows.append(_make_wall("text", 1, block, "cand", 0.095 * jitter * r_text))
        rows.append(_make_trace("wikidata", 8, block, "base", "none", 0.220 * jitter, 150_000_000))
        rows.append(
            _make_trace("wikidata", 8, block, "cand", "none", 0.220 * jitter * r8, 150_000_000)
        )
        rows.append(
            _make_trace(
                "wikidata",
                8,
                block,
                "base",
                "S",
                0.195 * jitter,
                int(150_000_000 * scan_residual_base),
            )
        )
        rows.append(
            _make_trace(
                "wikidata",
                8,
                block,
                "cand",
                "S",
                0.195 * jitter * r8s,
                int(150_000_000 * scan_residual_cand),
            )
        )
    return rows


class TestForwardLanesRecipe(unittest.TestCase):
    """Verify frozen §2/§3/§5 classification, shallow checkout, and harness guards."""

    def test_shallow_checkout_fetches_pinned_base_and_cand_without_history(self) -> None:
        """checkout_commit works from a detached `git clone --depth=1 --no-local` shallow runner."""
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            upstream = root / "upstream"
            runner = root / "runner"
            base_dest = root / "src-base"
            cand_dest = root / "src-cand"

            def git(repo: Path, *args: str) -> str:
                return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()

            subprocess.run(["git", "init", "-q", "-b", "master", str(upstream)], check=True)
            git(upstream, "config", "user.name", "Test")
            git(upstream, "config", "user.email", "test@example.invalid")
            (upstream / "src.txt").write_text("base\n", encoding="utf-8")
            git(upstream, "add", "src.txt")
            git(upstream, "commit", "-qm", "base commit")
            base_sha = git(upstream, "rev-parse", "HEAD")

            (upstream / "src.txt").write_text("cand\n", encoding="utf-8")
            git(upstream, "add", "src.txt")
            git(upstream, "commit", "-qm", "cand commit")
            cand_sha = git(upstream, "rev-parse", "HEAD")

            # Add two extra commits so depth=1 shallow runner has NEITHER base nor cand!
            for idx in range(2):
                (upstream / "src.txt").write_text(f"newer-{idx}\n", encoding="utf-8")
                git(upstream, "add", "src.txt")
                git(upstream, "commit", "-qm", f"newer {idx}")

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
                    str(runner),
                ],
                check=True,
            )
            runner_head = git(runner, "rev-parse", "HEAD")
            git(runner, "checkout", "--quiet", "--detach", runner_head)
            self.assertEqual(git(runner, "rev-parse", "--is-shallow-repository"), "true")
            for missing in (base_sha, cand_sha):
                res = subprocess.run(
                    ["git", "-C", str(runner), "cat-file", "-e", missing],
                    capture_output=True,
                )
                self.assertNotEqual(res.returncode, 0)

            fl.checkout_commit(runner, base_dest, base_sha)
            fl.checkout_commit(runner, cand_dest, cand_sha)
            self.assertEqual(git(base_dest, "rev-parse", "HEAD"), base_sha)
            self.assertEqual((base_dest / "src.txt").read_text(encoding="utf-8"), "base\n")
            self.assertEqual(git(cand_dest, "rev-parse", "HEAD"), cand_sha)
            self.assertEqual((cand_dest / "src.txt").read_text(encoding="utf-8"), "cand\n")
            self.assertEqual(git(runner, "rev-parse", "HEAD"), runner_head)

    def test_plan_matches_frozen_registration(self) -> None:
        """build_plan pins exact #1110 commits, 15 alternating AB/BA rounds, and §3 thresholds."""
        plan = fl.build_plan()
        self.assertEqual(plan["base"]["commit"], fl.BASE_SHA)
        self.assertEqual(plan["cand"]["commit"], fl.CANDIDATE_SHA)
        self.assertEqual(plan["cand"]["features"], "forward-anchor-lanes")
        self.assertEqual(plan["repeats"], 15)
        self.assertEqual(len(plan["schedule"]), 15)
        self.assertEqual(plan["schedule"][0]["order"], ["base", "cand"])
        self.assertEqual(plan["schedule"][1]["order"], ["cand", "base"])
        self.assertEqual(plan["thresholds"]["pass"]["r1_hi_max"], 0.950)
        self.assertEqual(plan["thresholds"]["pass"]["r8s_hi_max"], 0.975)
        self.assertEqual(plan["thresholds"]["pass"]["r8_hi_max"], 1.010)
        self.assertEqual(plan["thresholds"]["pass"]["text_hi_max"], 1.010)
        self.assertEqual(plan["thresholds"]["kill"]["r1_lo_min"], 0.980)
        self.assertEqual(plan["thresholds"]["kill"]["r8s_lo_min"], 1.000)
        self.assertEqual(plan["thresholds"]["kill"]["r8_lo_min"], 1.020)

    def test_verdict_pass_when_all_section3_clauses_satisfied(self) -> None:
        """All four PASS upper bounds met produces PASS with finite bootstrap CIs."""
        res = fl.summarise(_synthetic_dataset(r1=0.90, r8=0.98, r8s=0.93, r_text=0.99))
        self.assertEqual(res["verdict"], "PASS", res)
        self.assertTrue(all(res["pass_clauses"].values()))
        self.assertFalse(any(res["kill_clauses"].values()))
        self.assertAlmostEqual(res["scan_busy_residual"]["base"], 0.01, places=4)
        self.assertAlmostEqual(res["scan_busy_residual"]["cand"], 0.01, places=4)

    def test_verdict_kill_clauses_individually(self) -> None:
        """Each §3 KILL clause (r1.lo>0.980, r8S.lo>1.000, r8.lo>1.020) triggers KILL."""
        cases = [
            ("r1_lo_gt_0.980", dict(r1=0.99, r8=0.97, r8s=0.92, r_text=0.98)),
            ("r8s_lo_gt_1.000", dict(r1=0.90, r8=0.97, r8s=1.01, r_text=0.98)),
            ("r8_lo_gt_1.020", dict(r1=0.90, r8=1.03, r8s=0.92, r_text=0.98)),
        ]
        for clause, kwargs in cases:
            with self.subTest(clause=clause):
                res = fl.summarise(_synthetic_dataset(**kwargs))
                self.assertEqual(res["verdict"], "KILL", res)
                self.assertTrue(res["kill_clauses"][clause])
                self.assertIn(clause, res["reason"])

    def test_verdict_rescue_between_pass_and_kill(self) -> None:
        """Intermediate values (or text_1w > 1.010 with non-KILL wikidata) classify as RESCUE."""
        cases = [
            ("r1_mid", dict(r1=0.965, r8=0.98, r8s=0.93, r_text=0.98), "r1_hi_le_0.950"),
            ("r8s_mid", dict(r1=0.90, r8=0.98, r8s=0.985, r_text=0.98), "r8s_hi_le_0.975"),
            ("r8_mid", dict(r1=0.90, r8=1.015, r8s=0.93, r_text=0.98), "r8_hi_le_1.010"),
            ("text_slow", dict(r1=0.90, r8=0.98, r8s=0.93, r_text=1.03), "text_hi_le_1.010"),
        ]
        for label, kwargs, unmet in cases:
            with self.subTest(case=label):
                res = fl.summarise(_synthetic_dataset(**kwargs))
                self.assertEqual(res["verdict"], "RESCUE", res)
                self.assertFalse(res["pass_clauses"][unmet])
                self.assertFalse(any(res["kill_clauses"].values()))

    def test_exactness_failure_on_cand_is_kill_but_on_base_is_void(self) -> None:
        """Candidate SHA/CRC mismatch is KILL; control mismatch is VOID."""
        rows_cand_bad = _synthetic_dataset()
        for r in rows_cand_bad:
            if r["build"] == "cand" and r["measure"] == "wall" and r["block"] == 3:
                r["sha256"] = BAD_SHA
                break
        res_cand = fl.summarise(rows_cand_bad)
        self.assertEqual(res_cand["verdict"], "KILL")
        self.assertFalse(res_cand["exactness_ok"])

        rows_crc = _synthetic_dataset()
        for r in rows_crc:
            if r["build"] == "cand" and r["measure"] == "wall" and r["block"] == 0:
                r["kind"] = "failed"
                r["stderr"] = "block CRC mismatch at block 4"
                break
        self.assertEqual(fl.summarise(rows_crc)["verdict"], "KILL")

        rows_base_bad = _synthetic_dataset()
        for r in rows_base_bad:
            if r["build"] == "base" and r["measure"] == "wall" and r["block"] == 2:
                r["sha256"] = BAD_SHA
                break
        self.assertEqual(fl.summarise(rows_base_bad)["verdict"], "VOID")

    def test_void_conditions_scan_residual_missing_round_open_route_and_malformed_row(self) -> None:
        """Harness violations (scan residual >5%, missing round, Open not hit, failed child) VOID."""
        res_res = fl.summarise(_synthetic_dataset(scan_residual_cand=0.06))
        self.assertEqual(res_res["verdict"], "VOID")
        self.assertIn("scan oracle residual", res_res["reason"])

        rows_missing = _synthetic_dataset()[:-1]
        self.assertEqual(fl.summarise(rows_missing)["verdict"], "VOID")

        rows_no_open = _synthetic_dataset()
        for r in rows_no_open:
            if r["measure"] == "none" and r["build"] == "cand" and r["block"] == 0:
                r["trace_summary"]["blocks_open_pass1"] = 0
                break
        self.assertEqual(fl.summarise(rows_no_open)["verdict"], "VOID")

        rows_lock = _synthetic_dataset()
        for r in rows_lock:
            if r["measure"] == "S" and r["build"] == "cand" and r["block"] == 0:
                r["oracles"]["scan_locks"] = 1
                break
        self.assertEqual(fl.summarise(rows_lock)["verdict"], "VOID")

        # Non-CRC child failure (even with empty stderr) must VOID, never silently pass (#1031)
        rows_err = _synthetic_dataset()
        for r in rows_err:
            if r["build"] == "cand" and r["measure"] == "wall" and r["block"] == 1:
                r["kind"] = "failed"
                r["stderr"] = ""
                r["returncode"] = 1
                break
        self.assertEqual(fl.summarise(rows_err)["verdict"], "VOID")

    def test_paired_bootstrap_is_deterministic_and_reports_ranges(self) -> None:
        """paired_bootstrap_stats is deterministic and reports medians with [min, max]."""
        base = [1.00 + 0.01 * i for i in range(15)]
        cand = [0.92 * x for x in base]
        s1 = fl.paired_bootstrap_stats(base, cand)
        s2 = fl.paired_bootstrap_stats(base, cand)
        self.assertEqual(s1, s2)
        self.assertEqual(s1["base_range"], [min(base), max(base)])
        self.assertEqual(s1["cand_range"], [min(cand), max(cand)])
        self.assertLessEqual(s1["ci95"][0], s1["ratio"])
        self.assertLessEqual(s1["ratio"], s1["ci95"][1])

    def test_cli_plan_and_report_exit_codes(self) -> None:
        """CLI plan and report write summary.json/outcome.txt; VOID exits 1, PASS exits 0."""
        with tempfile.TemporaryDirectory() as td:
            out = Path(td) / "out"
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(fl.main(["plan", str(out)]), 0)
            self.assertTrue((out / "plan.json").exists())

            runs_pass = Path(td) / "pass.jsonl"
            runs_pass.write_text(
                "\n".join(json.dumps(r) for r in _synthetic_dataset()) + "\n", encoding="utf-8"
            )
            rep_pass = Path(td) / "rep-pass"
            with contextlib.redirect_stdout(io.StringIO()):
                rc_pass = fl.main(["report", str(runs_pass), "--out", str(rep_pass)])
            self.assertEqual(rc_pass, 0)
            self.assertTrue(
                (rep_pass / "outcome.txt").read_text(encoding="utf-8").startswith("PASS:")
            )

            runs_void = Path(td) / "void.jsonl"
            runs_void.write_text(
                "\n".join(json.dumps(r) for r in _synthetic_dataset(scan_residual_base=0.09))
                + "\n",
                encoding="utf-8",
            )
            rep_void = Path(td) / "rep-void"
            with contextlib.redirect_stdout(io.StringIO()):
                rc_void = fl.main(["report", str(runs_void), "--out", str(rep_void)])
            self.assertEqual(rc_void, 1)
            self.assertTrue(
                (rep_void / "outcome.txt").read_text(encoding="utf-8").startswith("VOID:")
            )

    def test_just_recipe_enforces_toolchain_bundle_and_no_home_writes(self) -> None:
        """bench-forward-lanes in .just/bench.just sets 1.99.1, bundles on EXIT, avoids $HOME."""
        text = (ROOT / ".just" / "bench.just").read_text(encoding="utf-8")
        self.assertIn("bench-forward-lanes:", text)
        recipe = text.split("bench-forward-lanes:", 1)[1]
        self.assertIn("RUSTUP_TOOLCHAIN=1.99.1", recipe)
        self.assertIn("target/forward-lanes", recipe)
        self.assertIn("INCOMPLETE (exit $rc)", recipe)
        self.assertIn('bash bench/bundle.sh "$out"', recipe)
        self.assertIn('python3 bench/forward_lanes.py run "$out"', recipe)
        self.assertNotIn("$HOME", recipe)
        self.assertNotIn("~/", recipe)

    def test_rejects_identical_base_and_cand_binaries(self) -> None:
        """assert_distinct_binaries accepts distinct digests and VOIDs identical or missing ones."""
        good = {
            "base": {"a6b_endpoint_run": "1" * 64, "mtf_fold": "2" * 64},
            "cand": {"a6b_endpoint_run": "3" * 64, "mtf_fold": "4" * 64},
        }
        fl.assert_distinct_binaries(good)

        same_wall = {
            "base": {"a6b_endpoint_run": "1" * 64, "mtf_fold": "2" * 64},
            "cand": {"a6b_endpoint_run": "1" * 64, "mtf_fold": "4" * 64},
        }
        with self.assertRaisesRegex(fl.VoidRun, "a6b_endpoint_run"):
            fl.assert_distinct_binaries(same_wall)

        same_trace = {
            "base": {"a6b_endpoint_run": "1" * 64, "mtf_fold": "2" * 64},
            "cand": {"a6b_endpoint_run": "3" * 64, "mtf_fold": "2" * 64},
        }
        with self.assertRaisesRegex(fl.VoidRun, "mtf_fold"):
            fl.assert_distinct_binaries(same_trace)

        missing = {"base": {"a6b_endpoint_run": "1" * 64}, "cand": {"a6b_endpoint_run": "3" * 64}}
        with self.assertRaisesRegex(fl.VoidRun, "missing compiled binary digest"):
            fl.assert_distinct_binaries(missing)

    def test_cpu_quota_accepts_unlimited_and_rejects_limited_in_both_schemas(self) -> None:
        """check_cpu_quota handles both `ab env` (None) and `ab env --facts` (dict) schemas."""
        fl.check_cpu_quota(None, source="ab env")
        fl.check_cpu_quota(
            {"state": "unlimited", "cpus": None, "hierarchy": "v2", "why": None},
            source="ab env --facts",
        )
        for bad in (
            4.0,
            {"state": "limited", "cpus": 8.0},
            {"state": "unlimited", "cpus": 4.0},
            {"state": "unknown", "cpus": None},
        ):
            with self.subTest(bad=bad), self.assertRaises(fl.VoidRun):
                fl.check_cpu_quota(bad, source="ab env --facts")

    def test_build_binaries_uses_isolated_per_arm_cargo_target_dirs(self) -> None:
        """build_binaries passes distinct --target-dir paths for base and cand checkouts."""
        from unittest import mock

        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            base_src = root / "src-base"
            cand_src = root / "src-cand"
            base_src.mkdir()
            cand_src.mkdir()
            seen_targets: dict[Path, set[Path]] = {base_src: set(), cand_src: set()}

            def fake_run(cmd, *, cwd, check):
                self.assertTrue(check)
                tidx = cmd.index("--target-dir")
                tdir = Path(cmd[tidx + 1])
                seen_targets[cwd].add(tdir)
                ex_dir = tdir / "release" / "examples"
                ex_dir.mkdir(parents=True, exist_ok=True)
                for i, tok in enumerate(cmd):
                    if tok == "--example":
                        (ex_dir / cmd[i + 1]).write_bytes(f"{cwd.name}-{cmd[i + 1]}".encode())
                return subprocess.CompletedProcess(cmd, 0)

            with mock.patch.object(fl.subprocess, "run", side_effect=fake_run):
                bins = fl.build_binaries(base_src, cand_src, root)

            self.assertEqual(seen_targets[base_src], {root / "cargo-target-base"})
            self.assertEqual(seen_targets[cand_src], {root / "cargo-target-cand"})
            digests = {a: {k: fl.digest(p) for k, p in b.items()} for a, b in bins.items()}
            fl.assert_distinct_binaries(digests)


if __name__ == "__main__":
    unittest.main()
