#!/usr/bin/env python3
"""Unit tests for bench/scanner_seam_window.py against the frozen C6 adoption registration
(docs/journal/2026-11-27-scanner-seam-window/registration.md)."""

import bz2
import hashlib
import io
import json
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import scanner_seam_identity as ident  # noqa: E402
import scanner_seam_window as ssw  # noqa: E402

GOOD_ENV = {
    "cores": 10,
    "cpus": 20,
    "cpu_quota": None,
    "core_cpus": [f"{i},{i + 10}" for i in range(10)],
}
GOOD_LSCPU = "Architecture: x86_64\nModel name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz\n"
WORKERS_FOR = {6: 6, 8: 8, 9: 9, 10: 8, 12: 8}


def make_threads(n_workers):
    return " ".join(["mibz2-scan", "mibz2-feed"] + [f"mibz2-{i}" for i in range(n_workers)])


def write_fixture(
    td,
    *,
    rounds=16,
    wall_ratios=None,
    rss_ratios=None,
    threads=(6, 8, 9, 10, 12),
    arm="mibz2",
    env=None,
    lscpu=GOOD_LSCPU,
    mutate=None,
):
    wall_ratios = wall_ratios or {t: 0.97 for t in threads}
    rss_ratios = rss_ratios or {t: 0.98 for t in threads}
    env_p = os.path.join(td, "env.json")
    lscpu_p = os.path.join(td, "lscpu.txt")
    runs_p = os.path.join(td, "runs.jsonl")
    with open(env_p, "w") as f:
        json.dump(GOOD_ENV if env is None else env, f)
    with open(lscpu_p, "w") as f:
        f.write(lscpu)
    with open(os.path.join(td, "identity.json"), "w") as f:
        json.dump({"out": ssw.WIKIDATA_X4_OUT, "sha256": "ab" * 32, "hash": "deadbeef" * 2}, f)
    rows = []
    for t in threads:
        w = WORKERS_FOR.get(t, t)
        for r in range(rounds):
            # Small deterministic per-round jitter so Student's t CI is well-defined and tight.
            jitter = 1.0 + ((r % 5) - 2) * 0.001
            base_wall = 0.200 * (1.0 + (r % 3) * 0.002)
            head_wall = base_wall * wall_ratios[t] * jitter
            base_rss = 60_000
            head_rss = int(round(base_rss * rss_ratios[t]))
            for name, wall, rss in (
                (f"{arm}@/work/target/scanner-seam-window/build-base/target/release/examples/ab", base_wall, base_rss),
                (arm, head_wall, head_rss),
            ):
                rows.append(
                    {
                        "corpus": "wikidata-x4",
                        "arm": name,
                        "threads": t,
                        "round": r,
                        "termination": "success",
                        "err": "",
                        "wall": wall,
                        "cpu": wall * w,
                        "rss_kb": rss,
                        "cpus": "all",
                        "held_spinner": None,
                        "out": ssw.WIKIDATA_X4_OUT,
                        "hash": "deadbeef" * 2,
                        "observed_threads": make_threads(w),
                        "cycles_per_ref": 1.32,
                    }
                )
    if mutate:
        mutate(rows)
    with open(runs_p, "w") as f:
        for row in rows:
            f.write(json.dumps(row) + "\n")
    return runs_p, env_p, lscpu_p


def run_verdict(runs_p, env_p, lscpu_p, extra=()):
    buf = io.StringIO()
    with redirect_stdout(buf):
        identity = os.path.join(os.path.dirname(runs_p), "identity.json")
        rc = ssw.main([runs_p, "--identity", identity, "--env", env_p, "--lscpu", lscpu_p, *extra])
    return rc, buf.getvalue()


class TestScannerSeamWindow(unittest.TestCase):
    def test_adopts_when_10_and_12_faster_none_slower_and_rss_ok(self):
        with tempfile.TemporaryDirectory() as td:
            p = write_fixture(
                td,
                wall_ratios={6: 1.000, 8: 0.992, 9: 0.985, 10: 0.975, 12: 0.974},
                rss_ratios={6: 0.98, 8: 0.99, 9: 1.00, 10: 0.97, 12: 0.97},
            )
            rc, out = run_verdict(*p)
            self.assertEqual(rc, 0, out)
            self.assertIn("verdict: ADOPTS", out)

    def test_not_adopted_when_only_one_judged_count_faster(self):
        with tempfile.TemporaryDirectory() as td:
            p = write_fixture(
                td,
                wall_ratios={6: 1.000, 8: 0.990, 9: 0.980, 10: 0.975, 12: 1.000},
            )
            rc, out = run_verdict(*p)
            self.assertEqual(rc, 0, out)
            self.assertIn("verdict: NOT ADOPTED (12 threads not FASTER)", out)

    def test_not_adopted_when_supporting_count_slower(self):
        with tempfile.TemporaryDirectory() as td:
            p = write_fixture(
                td,
                wall_ratios={6: 1.020, 8: 0.990, 9: 0.980, 10: 0.975, 12: 0.975},
            )
            rc, out = run_verdict(*p)
            self.assertEqual(rc, 0, out)
            self.assertIn("verdict: NOT ADOPTED (SLOWER at 6 threads)", out)

    def test_not_adopted_when_peak_rss_exceeds_1_10(self):
        with tempfile.TemporaryDirectory() as td:
            p = write_fixture(
                td,
                wall_ratios={6: 1.000, 8: 0.990, 9: 0.980, 10: 0.975, 12: 0.975},
                rss_ratios={6: 1.00, 8: 1.12, 9: 1.00, 10: 1.00, 12: 1.00},
            )
            rc, out = run_verdict(*p)
            self.assertEqual(rc, 0, out)
            self.assertIn("verdict: NOT ADOPTED (peak RSS > 1.10 at 8", out)

    def test_void_on_wrong_machine(self):
        with tempfile.TemporaryDirectory() as td:
            p = write_fixture(td, lscpu="Model name: Intel(R) Xeon(R) CPU\n")
            rc, out = run_verdict(*p)
            self.assertEqual(rc, 1, out)
            self.assertIn("VOID: no verdict", out)

    def test_void_on_hash_mismatch(self):
        def bad_hash(rows):
            rows[0]["hash"] = "00000000" * 8

        with tempfile.TemporaryDirectory() as td:
            p = write_fixture(td, mutate=bad_hash)
            rc, out = run_verdict(*p)
            self.assertEqual(rc, 1, out)
            self.assertIn("the outputs differ", out)

    def test_success_still_rejects_decoder_errors(self):
        with tempfile.TemporaryDirectory() as td:
            p = write_fixture(td, mutate=lambda rows: rows[0].update(err="bad CRC"))
            rc, out = run_verdict(*p)
            self.assertEqual(rc, 1, out)
            self.assertIn("bad CRC", out)

    def test_equal_but_wrong_outputs_fail_the_independent_reference(self):
        with tempfile.TemporaryDirectory() as td:
            def wrong(rows):
                for row in rows:
                    row["hash"] = "0123456789abcdef"

            rc, out = run_verdict(*write_fixture(td, mutate=wrong))
            self.assertEqual(rc, 1, out)
            self.assertIn("the outputs differ", out)

    def test_reference_checks_every_child_against_libbz2(self):
        data = b"first stream" + b"second stream"
        record = {"err": "", "out": len(data), "hash": "01" * 8,
                  "sha256": hashlib.sha256(data).hexdigest()}
        with tempfile.TemporaryDirectory() as td:
            corpus = Path(td) / "fixture.bz2"
            corpus.write_bytes(bz2.compress(b"first stream") + bz2.compress(b"second stream"))
            with patch.object(ident, "OUTPUT_BYTES", len(data)), patch.object(
                ident.subprocess, "run"
            ) as call:
                call.return_value.stdout = json.dumps(record)
                checked = ident.reference(corpus, "base", "head")
                self.assertEqual(len(checked["checks"]), 16)
                call.return_value.stdout = json.dumps({**record, "sha256": "ab" * 32})
                with self.assertRaisesRegex(ValueError, "differs from reference"):
                    ident.reference(corpus, "base", "head")

    def test_trace_hash_checked_across_reports_and_modes(self):
        ref = {"hash": "01" * 8, "out": 42}
        rows = [{"round": 0, "threads": 10, "mode": mode,
                 "line": f"err=- out=42 hash={ref['hash']}"}
                for mode in ("off", "timeline", "full")]
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "trace-base.jsonl"
            report = {"parity": True, "output_bytes": 42, "runs": rows}
            path.write_text(json.dumps(report))
            ident.check_trace(path, ref, 1)
            rows[-1]["line"] = "err=- out=42 hash=ffffffffffffffff"
            path.write_text(json.dumps(report))
            with self.assertRaisesRegex(ValueError, "invalid trace output"):
                ident.check_trace(path, ref, 1)

    def test_void_on_worker_plan_drift(self):
        def bad_plan(rows):
            for row in rows:
                if row["arm"] == "mibz2" and row["threads"] == 10:
                    row["observed_threads"] = make_threads(7)

        with tempfile.TemporaryDirectory() as td:
            p = write_fixture(td, mutate=bad_plan)
            rc, out = run_verdict(*p)
            self.assertEqual(rc, 1, out)
            self.assertIn("the plan moved", out)

    def test_dry_and_diagnostic_modes(self):
        with tempfile.TemporaryDirectory() as td:
            p = write_fixture(
                td,
                rounds=2,
                threads=(8, 9, 10),
                arm="mibz2-workers-lazy",
                env={"cores": 4, "cpus": 4, "quota": None},
                lscpu="Model name: Test VM\n",
            )
            rc, out = run_verdict(
                *p,
                extra=(
                    "--arm",
                    "mibz2-workers-lazy",
                    "--judged",
                    "",
                    "--supporting",
                    "8,9,10",
                    "--rounds",
                    "2",
                    "--diagnostic",
                    "--dry",
                ),
            )
            self.assertEqual(rc, 0, out)
            self.assertIn("DRY (untimed, not evidence)", out)
            self.assertNotIn("verdict:", out)


if __name__ == "__main__":
    unittest.main()
