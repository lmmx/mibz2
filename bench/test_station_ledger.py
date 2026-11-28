"""Unit tests for bench/station_ledger.py per-station CPU ledger reader."""

from __future__ import annotations

import io
import json
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))

import station_ledger as sl  # noqa: E402


def _row(
    corpus: str,
    threads: int,
    arm: str,
    wall_s: float,
    thread_cpu: str | None,
    round_idx: int = 0,
) -> str:
    payload: dict[str, object] = {
        "round": round_idx,
        "corpus": corpus,
        "threads": threads,
        "arm": arm,
        "wall": wall_s,
        "err": "",
    }
    if thread_cpu is not None:
        payload["thread_cpu"] = thread_cpu
    return json.dumps(payload)


class StationLedgerTests(unittest.TestCase):
    def test_six_row_synthetic_jsonl_skips_and_counts_empty_thread_cpu(self):
        """Frozen stop_test: 6-row synthetic JSONL including 1 empty thread_cpu row."""
        lines = [
            # Cell 1 (worker-bound): 3 rounds of wikidata, T=8, mibz2-occ-6-24 (k=6)
            _row(
                "wikidata",
                8,
                "mibz2-occ-6-24",
                0.240,
                "mibz2-0=0.22 mibz2-1=0.22 mibz2-2=0.22 mibz2-3=0.22 mibz2-4=0.22 mibz2-5=0.22 "
                "mibz2-scan=0.14 main=0.06",
                0,
            ),
            _row(
                "wikidata",
                8,
                "mibz2-occ-6-24",
                0.236,
                "mibz2-0=0.23 mibz2-1=0.22 mibz2-2=0.22 mibz2-3=0.22 mibz2-4=0.22 mibz2-5=0.21 "
                "mibz2-scan=0.15 main=0.06",
                1,
            ),
            _row(
                "wikidata",
                8,
                "mibz2-occ-6-24",
                0.238,
                "mibz2-0=0.23 mibz2-1=0.23 mibz2-2=0.22 mibz2-3=0.22 mibz2-4=0.22 mibz2-5=0.22 "
                "mibz2-scan=0.16 main=0.06",
                2,
            ),
            # Cell 2 (scanner-bound): 2 rounds of random, T=8, mibz2-occ-6-24 (k=6)
            _row(
                "random",
                8,
                "mibz2-occ-6-24",
                0.095,
                "mibz2-0=0.04 mibz2-1=0.04 mibz2-2=0.04 mibz2-3=0.04 mibz2-4=0.04 mibz2-5=0.04 "
                "mibz2-scan=0.07 main=0.01",
                0,
            ),
            _row(
                "random",
                8,
                "mibz2-occ-6-24",
                0.091,
                "mibz2-0=0.04 mibz2-1=0.04 mibz2-2=0.04 mibz2-3=0.04 mibz2-4=0.04 mibz2-5=0.04 "
                "mibz2-scan=0.06 main=0.01",
                1,
            ),
            # Row 6: empty thread_cpu row (must be skipped and counted)
            _row("wikidata", 8, "mibz2-occ-6-24", 0.0001, "", 3),
        ]
        self.assertEqual(len(lines), 6)

        rep = sl.analyze_lines(lines, source="synthetic.jsonl")
        self.assertEqual(rep.rows_total, 6)
        self.assertEqual(rep.rows_used, 5)
        self.assertEqual(rep.skipped_empty_thread_cpu, 1)
        self.assertEqual(rep.skipped_invalid, 0)
        self.assertEqual(len(rep.cells), 2)

        by_corpus = {c.corpus: c for c in rep.cells}

        w = by_corpus["wikidata"]
        self.assertEqual(w.n, 3)
        self.assertEqual(w.k, 6)
        self.assertAlmostEqual(w.wall_ms, 238.0, places=6)
        self.assertAlmostEqual(w.s_ms, 150.0, places=6)
        # Round W sums: 1320, 1320, 1340 -> mean = 1326.6667 ms -> W/k = 221.1111 ms
        self.assertAlmostEqual(w.w_ms, 3980.0 / 3.0, places=6)
        self.assertAlmostEqual(w.w_per_k_ms, 3980.0 / 18.0, places=6)
        self.assertAlmostEqual(w.delta_ms, 3980.0 / 18.0 - 150.0, places=6)
        self.assertAlmostEqual(w.residual_ms, 238.0 - 3980.0 / 18.0, places=6)
        self.assertEqual(w.binding, "workers")
        self.assertFalse(w.w_growth_flag)

        r = by_corpus["random"]
        self.assertEqual(r.n, 2)
        self.assertEqual(r.k, 6)
        self.assertAlmostEqual(r.wall_ms, 93.0, places=6)
        self.assertAlmostEqual(r.s_ms, 65.0, places=6)
        self.assertAlmostEqual(r.w_ms, 240.0, places=6)
        self.assertAlmostEqual(r.w_per_k_ms, 40.0, places=6)
        self.assertAlmostEqual(r.delta_ms, -25.0, places=6)
        self.assertAlmostEqual(r.residual_ms, 28.0, places=6)
        self.assertEqual(r.binding, "scanner")

        rendered = sl.format_report(rep)
        self.assertIn("skipped empty thread_cpu: 1", rendered)
        self.assertIn("workers", rendered)
        self.assertIn("scanner", rendered)

    def test_arm_family_w_growth_flag_threshold_and_isolation(self):
        lines = [
            # Family 1 (mibz2-occ): W grows +25% from k=4 (400 ms) to k=8 (500 ms) -> FLAGGED
            _row(
                "text",
                4,
                "mibz2-occ-4-16",
                0.150,
                "mibz2-0=0.10 mibz2-1=0.10 mibz2-2=0.10 mibz2-3=0.10 mibz2-scan=0.03",
            ),
            _row(
                "text",
                8,
                "mibz2-occ-8-32",
                0.150,
                "mibz2-0=0.06 mibz2-1=0.06 mibz2-2=0.06 mibz2-3=0.06 "
                "mibz2-4=0.06 mibz2-5=0.06 mibz2-6=0.07 mibz2-7=0.07 mibz2-scan=0.03",
            ),
            # Family 2 (mibz2-oce): W grows +9.0% from k=2 (200 ms) to k=4 (218 ms) -> NOT flagged
            _row(
                "text",
                2,
                "mibz2-oce-2-8",
                0.140,
                "mibz2-0=0.10 mibz2-1=0.10 mibz2-scan=0.03",
            ),
            _row(
                "text",
                4,
                "mibz2-oce-4-16",
                0.100,
                "mibz2-0=0.055 mibz2-1=0.055 mibz2-2=0.054 mibz2-3=0.054 mibz2-scan=0.03",
            ),
            # Family 3 (distinct @exe binary): W flat -> NOT mixed with mibz2-occ
            _row(
                "text",
                4,
                "mibz2-occ-4-16@/tmp/build-v0.5.17/target/release/examples/ab",
                0.120,
                "mibz2-0=0.10 mibz2-1=0.10 mibz2-2=0.10 mibz2-3=0.10 mibz2-scan=0.03",
            ),
        ]
        rep = sl.analyze_lines(lines, source="growth.jsonl")
        fams = {(f.corpus, f.arm_family): f for f in rep.families}
        self.assertTrue(fams[("text", "mibz2-occ")].flagged)
        self.assertAlmostEqual(fams[("text", "mibz2-occ")].growth_pct, 25.0, places=4)
        self.assertFalse(fams[("text", "mibz2-oce")].flagged)
        self.assertAlmostEqual(fams[("text", "mibz2-oce")].growth_pct, 9.0, places=4)
        self.assertNotIn(
            ("text", "mibz2-occ@/tmp/build-v0.5.17/target/release/examples/ab"),
            fams,
        )

        rendered = sl.format_report(rep)
        self.assertIn("W_GROWTH(+25.0%)", rendered)
        self.assertIn("FLAG text / mibz2-occ: k=4->8, W=400.0->500.0 ms (+25.0%)", rendered)
        self.assertIn("mibz2-occ-4-16@build-v0.5.17", rendered)

    def test_binding_edge_cases_and_missing_or_malformed_rows(self):
        lines = [
            # tie (|W/k - S| <= 1.0 ms, residual < max(S, W/k))
            _row("wikidata", 4, "mibz2", 0.065, "mibz2-0=0.05 mibz2-1=0.05 mibz2-scan=0.0505"),
            # residual-bound (residual > max(S, W/k))
            _row("text", 4, "mibz2", 0.080, "mibz2-0=0.02 mibz2-1=0.02 mibz2-scan=0.02"),
            # single-thread mibz2 (k=0, main only)
            _row("wikidata", 1, "mibz2", 0.350, "main=0.35"),
            # non-mibz2 with duplicate thread names (k=0)
            _row("wikidata", 2, "lbzip2", 1.300, "lbzip2=1.28 lbzip2=1.27"),
            # missing thread_cpu key -> counted as empty thread_cpu
            _row("wikidata", 4, "mibz2", 0.100, None),
            # malformed JSON -> counted as invalid
            "{not valid json",
        ]
        rep = sl.analyze_lines(lines, source="edges.jsonl")
        self.assertEqual(rep.rows_total, 6)
        self.assertEqual(rep.rows_used, 4)
        self.assertEqual(rep.skipped_empty_thread_cpu, 1)
        self.assertEqual(rep.skipped_invalid, 1)

        by_key = {(c.corpus, c.threads, c.arm): c for c in rep.cells}
        self.assertEqual(by_key[("wikidata", 4, "mibz2")].binding, "tie")
        self.assertEqual(by_key[("text", 4, "mibz2")].binding, "residual")
        self.assertEqual(by_key[("wikidata", 1, "mibz2")].binding, "single")
        self.assertIsNone(by_key[("wikidata", 1, "mibz2")].w_per_k_ms)
        self.assertEqual(by_key[("wikidata", 2, "lbzip2")].binding, "other")
        self.assertIsNone(by_key[("wikidata", 2, "lbzip2")].delta_ms)

    def test_tar_bundle_skips_parity_and_duplicate_recorded_and_cli_json(self):
        row = _row(
            "wikidata",
            4,
            "mibz2",
            0.130,
            "mibz2-0=0.10 mibz2-1=0.10 mibz2-2=0.10 mibz2-scan=0.03 main=0.01",
        )
        with tempfile.TemporaryDirectory() as td:
            tar_path = Path(td) / "bundle.tar.gz"
            with tarfile.open(tar_path, "w:gz") as tf:
                for member_name in (
                    "run-1/runs.jsonl",
                    "run-1/recorded/runs.jsonl",
                    "run-1/parity.jsonl",
                ):
                    data = (row + "\n").encode("utf-8")
                    info = tarfile.TarInfo(name=member_name)
                    info.size = len(data)
                    tf.addfile(info, io.BytesIO(data))

            reports = sl.read_input(tar_path)
            self.assertEqual(len(reports), 1)
            self.assertTrue(reports[0].source.endswith("::run-1/runs.jsonl"))

            # Verify CLI --json and missing-path exit code
            buf = io.StringIO()
            old_stdout = sys.stdout
            try:
                sys.stdout = buf
                rc = sl.main([str(tar_path), "--json"])
            finally:
                sys.stdout = old_stdout
            self.assertEqual(rc, 0)
            parsed = json.loads(buf.getvalue())
            self.assertEqual(len(parsed), 1)
            self.assertEqual(parsed[0]["cells"][0]["binding"], "workers")
            self.assertEqual(sl.main([str(Path(td) / "nonexistent.jsonl")]), 2)


if __name__ == "__main__":
    unittest.main()
