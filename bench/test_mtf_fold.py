#!/usr/bin/env python3
"""Unit tests for bench/mtf_fold.py (frozen C2 gates K1-K4 and VOID §3.5)."""

from __future__ import annotations

import io
import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import mtf_fold as mf  # noqa: E402


LSCPU_I9 = "Model name:                      Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
LSCPU_OTHER = "Model name:                      AMD EPYC 7763 64-Core Processor\n"
LSCPU_E = "\n".join(
    [
        "CPU NODE SOCKET CORE L1d:L1i:L2:L3 ONLINE",
        "0 0 0 0 0:0:0:0 yes",
        "1 0 0 1 1:1:1:0 yes",
        "2 0 0 2 2:2:2:0 yes",
        "3 0 0 0 0:0:0:0 yes",
        "4 0 0 1 1:1:1:0 yes",
        "5 0 0 2 2:2:2:0 yes",
    ]
)


def make_plan(blocks=6, trace_blocks=2):
    cells = []
    for k in mf.COUNTS:
        arms = list(mf.ARMS_ALL_K) + (list(mf.ARMS_K8_EXTRA) if k == 8 else [])
        for arm in arms:
            cells.append(
                {
                    "section": "wikidata",
                    "k": k,
                    "arm": arm,
                    "pinned": True,
                    "scanner_cpu": 1,
                    "mask": [0, 2, 3, 5],
                    "prespin_ms": 100,
                }
            )
    return {
        "scanner_cpu": 1,
        "blocks": blocks,
        "trace_blocks": trace_blocks,
        "cells": cells,
    }


def make_records(
    plan,
    *,
    f_ratio_1=0.94,
    fs_ratio_8=0.94,
    f_ratio_6=1.00,
    f_ratio_8=1.00,
    rss_ratio=0.98,
    fs_folded=932,
    fs_bailed=0,
    s_scan_residual=0.01,
    pin_ok=True,
    identical=True,
):
    recs = []
    blocks = plan["blocks"]
    tb = plan["trace_blocks"]
    for ci, c in enumerate(plan["cells"]):
        k, arm = c["k"], c["arm"]
        base_wall = 1.30 / max(1, min(k, 8))
        if arm == "none":
            wall = base_wall
        elif arm == "F":
            mult = {1: f_ratio_1, 6: f_ratio_6, 8: f_ratio_8}.get(k, 0.99)
            wall = base_wall * mult
        elif arm == "S":
            wall = base_wall * 0.95
        elif arm == "FS":
            mult = fs_ratio_8 if k == 8 else 0.94
            wall = base_wall * 0.95 * mult
        elif arm == "SM":
            wall = base_wall * 0.95 * 0.88
        elif arm == "SC":
            wall = base_wall * 0.95 * 0.78
        else:
            wall = base_wall

        rss = int(100_000 * (rss_ratio if arm == "F" else 1.0))
        for b in range(blocks):
            jitter = 1.0 + 0.002 * ((b % 3) - 1)
            for tr in (0, 1) if b < tb else (0,):
                is_fold = arm in ("F", "FS")
                fold = {
                    "folded": (fs_folded if arm == "FS" else 932) if is_fold else 0,
                    "bailed": (fs_bailed if arm == "FS" else 0) if is_fold else 0,
                    "not_open": 100 if is_fold else 0,
                }
                oracles = {
                    "recorded_blocks": 1724 if "S" in arm else 0,
                    "syms_store_bytes": 52_000_000 if "S" in arm else 0,
                    "scanned": 0 if "S" in arm else 1724,
                    "replayed": 1724 if "S" in arm else 0,
                    "scan_locks": 0,
                    "mtf_executed": 0 if arm == "SM" else 1724,
                    "mtf_replayed": 1724 if arm == "SM" else 0,
                    "mtf_locks": 0,
                    "constr_constructed": 0 if arm == "SC" else 932,
                    "constr_replayed": 932 if arm == "SC" else 0,
                    "constr_refused_replayed": 0,
                    "constr_locks": 0,
                }
                ts = None
                if tr == 1:
                    scan_ns = int(180_000_000 * (s_scan_residual if "S" in arm else 1.0))
                    ts = {
                        "scanner_busy_ns": scan_ns,
                        "scanner_cpu_ns": scan_ns,
                        "workers_seen": k,
                        "worker_busy_ns_total": 1_200_000_000,
                        "worker_busy_ns_per_w": 1_200_000_000 / k,
                        "worker_cpu_ns_total": 1_200_000_000,
                        "worker_cpu_ns_per_w": 1_200_000_000 / k,
                        "a5_ns_total": 0 if is_fold else 132_000_000,
                        "a5_ns_per_w": 0 if is_fold else 132_000_000 / k,
                        "a6a_ns_total": 0 if is_fold else 200_000_000,
                        "a6a_ns_per_w": 0 if is_fold else 200_000_000 / k,
                        "fused_a5_a6a_ns_total": 260_000_000 if is_fold else 0,
                        "fused_a5_a6a_ns_per_w": 260_000_000 / k if is_fold else 0,
                        "fold_fallback_ns_total": 0,
                        "a6bc_a7_ns_total": 760_000_000,
                        "a6bc_a7_ns_per_w": 760_000_000 / k,
                        "a9_ns_total": 100_000_000,
                        "a9_ns_per_w": 100_000_000 / k,
                        "copy_out_ns": 34_000_000,
                        "reader_wait_ns": 150_000_000,
                        "reader_cpu_ns": 35_000_000,
                        "ready_bytes_hwm": 8 * 1024 * 1024,
                        "ready_bytes_sweep_hwm": 8 * 1024 * 1024,
                        "blocks_issued": 1724,
                        "blocks_decoded": 1724,
                        "blocks_open_pass1": 932,
                    }
                recs.append(
                    {
                        "kind": "decode",
                        "section": "wikidata",
                        "cell": ci,
                        "block": b,
                        "arm": arm,
                        "k": k,
                        "trace": tr,
                        "wall_s": wall * jitter,
                        "maxrss_kib": rss,
                        "majflt": 0,
                        "bytes": mf.WIKIDATA_X4_BYTES,
                        "scanner_cpu": 1,
                        "scanner_pin_ok": pin_ok,
                        "fold": fold,
                        "oracles": oracles,
                        "check_pass": {"identical": identical},
                        "trace_summary": ts,
                    }
                )
    return recs


class TestMtfFoldAnalysis(unittest.TestCase):
    def test_topology_and_fastest_core(self):
        with tempfile.TemporaryDirectory() as td:
            lp = os.path.join(td, "lscpu-e.txt")
            cp = os.path.join(td, "cores.json")
            with open(lp, "w", encoding="utf-8") as f:
                f.write(LSCPU_E)
            with open(cp, "w", encoding="utf-8") as f:
                json.dump(
                    {
                        "0": {"repeats": [[0, 100], [1, 102]]},
                        "1": {"repeats": [[0, 88], [1, 90]]},
                        "2": {"repeats": [[0, 95], [1, 96]]},
                    },
                    f,
                )
            byc = mf.topology(lp)
            self.assertEqual(byc, [[0, 3], [1, 4], [2, 5]])
            scpu, scs, _ = mf.fastest_core(byc, cp)
            self.assertEqual(scpu, 1)
            self.assertEqual(scs, [1, 4])

    def test_continue_when_all_gates_pass(self):
        plan = make_plan()
        recs = make_records(plan)
        buf = io.StringIO()
        verdict, summary = mf.analyse(plan, recs, [], LSCPU_I9, 6, 2, mf.WIKIDATA_X4_BYTES, out=buf)
        self.assertEqual(verdict, "CONTINUE", buf.getvalue())
        self.assertEqual(summary["kills"], [])
        self.assertEqual(summary["void_reasons"], [])

    def test_kill_k1(self):
        plan = make_plan()
        recs = make_records(plan, f_ratio_1=0.985)
        verdict, summary = mf.analyse(
            plan, recs, [], LSCPU_I9, 6, 2, mf.WIKIDATA_X4_BYTES, out=io.StringIO()
        )
        self.assertEqual(verdict, "KILL")
        self.assertTrue(any("K1:" in k for k in summary["kills"]))

    def test_kill_k2(self):
        plan = make_plan()
        recs = make_records(plan, fs_ratio_8=0.985)
        verdict, summary = mf.analyse(
            plan, recs, [], LSCPU_I9, 6, 2, mf.WIKIDATA_X4_BYTES, out=io.StringIO()
        )
        self.assertEqual(verdict, "KILL")
        self.assertTrue(any("K2:" in k for k in summary["kills"]))

    def test_kill_k3(self):
        plan = make_plan()
        recs = make_records(plan, f_ratio_6=1.02)
        verdict, summary = mf.analyse(
            plan, recs, [], LSCPU_I9, 6, 2, mf.WIKIDATA_X4_BYTES, out=io.StringIO()
        )
        self.assertEqual(verdict, "KILL")
        self.assertTrue(any("K3:" in k for k in summary["kills"]))

    def test_kill_k4_rss(self):
        plan = make_plan()
        recs = make_records(plan, rss_ratio=1.06)
        verdict, summary = mf.analyse(
            plan, recs, [], LSCPU_I9, 6, 2, mf.WIKIDATA_X4_BYTES, out=io.StringIO()
        )
        self.assertEqual(verdict, "KILL")
        self.assertTrue(any("K4:" in k for k in summary["kills"]))

    def test_void_non_i9(self):
        plan = make_plan()
        recs = make_records(plan)
        verdict, _ = mf.analyse(
            plan, recs, [], LSCPU_OTHER, 6, 2, mf.WIKIDATA_X4_BYTES, out=io.StringIO()
        )
        self.assertEqual(verdict, "VOID")

    def test_void_fs_did_not_fold(self):
        plan = make_plan()
        recs = make_records(plan, fs_folded=0)
        verdict, summary = mf.analyse(
            plan, recs, [], LSCPU_I9, 6, 2, mf.WIKIDATA_X4_BYTES, out=io.StringIO()
        )
        self.assertEqual(verdict, "VOID")
        self.assertTrue(any("did not fold" in v for v in summary["void_reasons"]))

    def test_void_bailed_gt_zero(self):
        plan = make_plan()
        recs = make_records(plan, fs_bailed=1)
        verdict, summary = mf.analyse(
            plan, recs, [], LSCPU_I9, 6, 2, mf.WIKIDATA_X4_BYTES, out=io.StringIO()
        )
        self.assertEqual(verdict, "VOID")
        self.assertTrue(any("bailed=" in v for v in summary["void_reasons"]))

    def test_void_scan_residual_gt_5pct(self):
        plan = make_plan()
        recs = make_records(plan, s_scan_residual=0.06)
        verdict, summary = mf.analyse(
            plan, recs, [], LSCPU_I9, 6, 2, mf.WIKIDATA_X4_BYTES, out=io.StringIO()
        )
        self.assertEqual(verdict, "VOID")
        self.assertTrue(any("residual" in v for v in summary["void_reasons"]))

    def test_void_pin_failed(self):
        plan = make_plan()
        recs = make_records(plan, pin_ok=False)
        verdict, summary = mf.analyse(
            plan, recs, [], LSCPU_I9, 6, 2, mf.WIKIDATA_X4_BYTES, out=io.StringIO()
        )
        self.assertEqual(verdict, "VOID")
        self.assertTrue(any("pin failed" in v for v in summary["void_reasons"]))

    def test_run_child_malformed_json_returns_failed_record(self):
        with tempfile.TemporaryDirectory() as td:
            bad_bin = os.path.join(td, "bad.sh")
            with open(bad_bin, "w", encoding="utf-8") as f:
                f.write("#!/bin/sh\necho 'not json {'\n")
            os.chmod(bad_bin, 0o755)
            cell = {"section": "wikidata", "arm": "SC", "k": 1, "prespin_ms": 0, "pinned": False}
            rec = mf.run_child(bad_bin, "dummy.bz2", cell, 0, 0, False, 10)
            self.assertEqual(rec["kind"], "failed")
            self.assertIn("malformed JSON", rec["stderr"])

    def test_cmd_report_and_shallow_checkout_pipeline(self):
        import contextlib
        from pathlib import Path
        import subprocess

        repo_root = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            upstream = root / "upstream"
            runner = root / "runner"

            def git(repo: Path, *args: str) -> str:
                return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()

            subprocess.run(["git", "init", "-q", "-b", "master", str(upstream)], check=True)
            git(upstream, "config", "user.name", "Test")
            git(upstream, "config", "user.email", "test@example.invalid")
            (upstream / "bench").mkdir()
            (upstream / "bench" / "bundle.sh").write_text(
                (repo_root / "bench" / "bundle.sh").read_text(encoding="utf-8"), encoding="utf-8"
            )
            os.chmod(upstream / "bench" / "bundle.sh", 0o755)
            git(upstream, "add", ".")
            git(upstream, "commit", "-qm", "master")

            git(upstream, "checkout", "-q", "--orphan", "bench-data")
            git(upstream, "rm", "-rfq", ".")
            (upstream / "windows" / "demo").mkdir(parents=True)
            (upstream / "windows" / "demo" / "w00.bin").write_bytes(b"window-bytes")
            git(upstream, "add", ".")
            git(upstream, "commit", "-qm", "bench-data")
            git(upstream, "checkout", "-q", "master")

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
            head = git(runner, "rev-parse", "HEAD")
            git(runner, "checkout", "--quiet", "--detach", head)
            self.assertEqual(git(runner, "rev-parse", "--is-shallow-repository"), "true")

            # Verify bench-mtf-fold's bench-data fetch works inside a detached depth=1 single-branch clone
            git(runner, "fetch", "-q", "origin", "bench-data:refs/remotes/origin/bench-data")
            arc = subprocess.check_output(
                ["git", "-C", str(runner), "archive", "origin/bench-data", "windows/demo"]
            )
            subprocess.run(["tar", "-x"], cwd=runner, input=arc, check=True)
            self.assertEqual(
                (runner / "windows" / "demo" / "w00.bin").read_bytes(), b"window-bytes"
            )

            # Verify cmd_report runs cleanly with no ResourceWarnings and bundles via bench/bundle.sh
            out_dir = runner / "target" / "mtf-fold" / "run-test"
            out_dir.mkdir(parents=True)
            plan = make_plan()
            recs = make_records(plan)
            plan_p = out_dir / "plan.json"
            runs_p = out_dir / "runs.jsonl"
            lscpu_p = out_dir / "lscpu.txt"
            plan_p.write_text(json.dumps(plan) + "\n", encoding="utf-8")
            runs_p.write_text("\n".join(json.dumps(r) for r in recs) + "\n", encoding="utf-8")
            lscpu_p.write_text(LSCPU_I9, encoding="utf-8")
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                rc = mf.main(
                    [
                        "report",
                        str(plan_p),
                        str(runs_p),
                        "--lscpu",
                        str(lscpu_p),
                        "--out",
                        str(out_dir),
                    ]
                )
            self.assertEqual(rc, 0, buf.getvalue())
            self.assertTrue((out_dir / "summary.json").exists())
            subprocess.run(
                ["bash", "bench/bundle.sh", str(out_dir)],
                cwd=runner,
                check=True,
                capture_output=True,
            )
            self.assertTrue(out_dir.with_suffix(".tar.gz").exists())


if __name__ == "__main__":
    unittest.main(verbosity=2)
