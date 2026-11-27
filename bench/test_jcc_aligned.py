"""Unit tests for the frozen jcc-aligned-release-build i9 recipe and C2 scorer."""

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import jcc_aligned as ja


ALIGNED_EXE = "/tmp/fake-run/aligned-ab"


def make_refs():
    return {
        "wikidata": {"out": 838_800_784, "hash": "hash-wiki", "sha256": "sha-wiki"},
        "text": {"out": 71_964_160, "hash": "hash-text", "sha256": "sha-text"},
        "records": {"out": 38_400_000, "hash": "hash-rec", "sha256": "sha-rec"},
        "random": {"out": 32_000_000, "hash": "hash-rnd", "sha256": "sha-rnd"},
    }


def make_passing_rows(refs=None):
    refs = refs or make_refs()
    rows = []
    for r in range(ja.ROUNDS):
        for corpus, k in ja.CELLS:
            base_wall = 1.0 + r * 0.001
            if (corpus, k) == ("wikidata", 1):
                ratio = 0.94
            elif (corpus, k) == ("wikidata", 8):
                ratio = 0.96
            elif corpus == "wikidata" and k in (4, 6):
                ratio = 0.95
            else:
                ratio = 1.002
            for arm_label, wall in (
                ("mibz2", base_wall),
                (f"mibz2@{ALIGNED_EXE}", base_wall * ratio),
            ):
                rows.append(
                    {
                        "round": r,
                        "corpus": corpus,
                        "threads": k,
                        "arm": arm_label,
                        "termination": "success",
                        "wall": wall,
                        "cpu": wall * k,
                        "rss_kb": 40_000,
                        "out": refs[corpus]["out"],
                        "hash": refs[corpus]["hash"],
                        "err": "",
                    }
                )
    return rows


def make_station_rows():
    rows = []
    for r in range(ja.STATION_ROUNDS):
        for corpus in ja.STATION_CORPORA:
            s_def, w_def = 75.0 + r * 0.1, 750.0 + r * 0.5
            s_al, w_al = s_def * 0.97, w_def * 0.87
            for arm, s_ms, w_ms in (("default", s_def, w_def), ("aligned", s_al, w_al)):
                rows.append(
                    {
                        "round": r,
                        "corpus": corpus,
                        "arm": arm,
                        "file": f"{corpus}.bz2",
                        "blocks": 431,
                        "full_blocks": 233,
                        "symbols": 24_979_299,
                        "out": 209_700_196,
                        "in_bytes": 11_854_933,
                        "s_ms": s_ms,
                        "w_ms": w_ms,
                        "k_star": w_ms / s_ms,
                        "s_ns_sym": 3.0,
                        "w_ns_byte": 3.5,
                    }
                )
    return rows


def make_valid_audit():
    return {
        "default": {
            "ab": {
                "sha256": "aaa",
                "text_bytes": 640_000,
                "mibz2_jcc_total": 8_900,
                "mibz2_unfused_32b_crossings": 1_195,
            },
            "station_balance": {
                "sha256": "bbb",
                "text_bytes": 508_000,
                "mibz2_jcc_total": 6_200,
                "mibz2_unfused_32b_crossings": 810,
            },
        },
        "aligned": {
            "ab": {
                "sha256": "ccc",
                "text_bytes": 662_000,
                "mibz2_jcc_total": 8_900,
                "mibz2_unfused_32b_crossings": 0,
            },
            "station_balance": {
                "sha256": "ddd",
                "text_bytes": 523_000,
                "mibz2_jcc_total": 6_200,
                "mibz2_unfused_32b_crossings": 0,
            },
        },
    }


class JccAlignedVerdictTests(unittest.TestCase):
    def test_passing_grid_with_abs_path_arm_and_empty_err(self):
        refs = make_refs()
        res = ja.score(make_passing_rows(refs), refs, aligned_exe=ALIGNED_EXE)
        self.assertEqual(res["verdict"], "PASS")
        self.assertEqual(res["findings"], [])
        self.assertEqual(res["amd_ci_gate"], "pending_release_pr_ci_bench")
        self.assertEqual(len(res["cells"]), len(ja.CELLS))

    def test_canonical_arm_handles_abs_path_and_rejects_mismatch(self):
        self.assertEqual(ja.canonical_arm("mibz2", ALIGNED_EXE), "default")
        self.assertEqual(ja.canonical_arm(f"mibz2@{ALIGNED_EXE}", ALIGNED_EXE), "aligned")
        with self.assertRaises(ja.InvalidRun):
            ja.canonical_arm("mibz2@/tmp/other/ab", ALIGNED_EXE)
        with self.assertRaises(ja.InvalidRun):
            ja.canonical_arm("pbz2", ALIGNED_EXE)

    def test_stop_on_k1_below_3_percent_or_overlapping_ci(self):
        refs = make_refs()
        rows = make_passing_rows(refs)
        for row in rows:
            if row["corpus"] == "wikidata" and row["threads"] == 1 and "@" in row["arm"]:
                row["wall"] = (1.0 + row["round"] * 0.001) * 0.975
        res = ja.score(rows, refs, aligned_exe=ALIGNED_EXE)
        self.assertEqual(res["verdict"], "STOP")
        self.assertTrue(any("wikidata/1" in f for f in res["findings"]))

        # Overlapping 95% CI on k=1 despite 3.5% mean gain
        rows2 = make_passing_rows(refs)
        for row in rows2:
            if row["corpus"] == "wikidata" and row["threads"] == 1 and "@" in row["arm"]:
                jitter = 0.90 if row["round"] % 2 == 0 else 1.03
                row["wall"] = (1.0 + row["round"] * 0.001) * jitter
        res2 = ja.score(rows2, refs, aligned_exe=ALIGNED_EXE)
        self.assertEqual(res2["verdict"], "STOP")
        self.assertTrue(any("wikidata/1" in f for f in res2["findings"]))

    def test_stop_on_k8_below_1_5_percent(self):
        refs = make_refs()
        rows = make_passing_rows(refs)
        for row in rows:
            if row["corpus"] == "wikidata" and row["threads"] == 8 and "@" in row["arm"]:
                row["wall"] = (1.0 + row["round"] * 0.001) * 0.990
        res = ja.score(rows, refs, aligned_exe=ALIGNED_EXE)
        self.assertEqual(res["verdict"], "STOP")
        self.assertTrue(any("wikidata/8" in f for f in res["findings"]))

    def test_stop_on_k10_regression_over_1_percent(self):
        refs = make_refs()
        rows = make_passing_rows(refs)
        for row in rows:
            if row["corpus"] == "wikidata" and row["threads"] == 10 and "@" in row["arm"]:
                row["wall"] = (1.0 + row["round"] * 0.001) * 1.015
        res = ja.score(rows, refs, aligned_exe=ALIGNED_EXE)
        self.assertEqual(res["verdict"], "STOP")
        self.assertTrue(any("wikidata/10" in f for f in res["findings"]))

    def test_stop_on_guard_corpus_regression_over_1_percent(self):
        refs = make_refs()
        rows = make_passing_rows(refs)
        for row in rows:
            if row["corpus"] == "records" and row["threads"] == 8 and "@" in row["arm"]:
                row["wall"] = (1.0 + row["round"] * 0.001) * 1.012
        res = ja.score(rows, refs, aligned_exe=ALIGNED_EXE)
        self.assertEqual(res["verdict"], "STOP")
        self.assertTrue(any("records/8" in f for f in res["findings"]))

    def test_stop_on_rss_over_1_02x(self):
        refs = make_refs()
        rows = make_passing_rows(refs)
        for row in rows:
            if row["corpus"] == "wikidata" and row["threads"] == 6 and "@" in row["arm"]:
                row["rss_kb"] = 41_000  # 1.025x > 1.020
        res = ja.score(rows, refs, aligned_exe=ALIGNED_EXE)
        self.assertEqual(res["verdict"], "STOP")
        self.assertTrue(any("wikidata/6" in f and "RSS" in f for f in res["findings"]))

    def test_void_on_err_termination_hash_byte_missing_or_dup(self):
        refs = make_refs()
        for mutate in (
            lambda r: r[0].update(err="CRC mismatch"),
            lambda r: r[0].update(termination="exit 1"),
            lambda r: r[0].update(hash="corrupt-hash"),
            lambda r: r[0].update(out=123),
            lambda r: r[0].update(wall=0.0),
            lambda r: r[0].update(rss_kb=0),
            lambda r: r.pop(),
            lambda r: r.append(dict(r[0])),
        ):
            rows = make_passing_rows(refs)
            mutate(rows)
            with self.assertRaises(ja.InvalidRun):
                ja.score(rows, refs, aligned_exe=ALIGNED_EXE)

    def test_codegen_audit_catches_identical_or_unaligned_binary(self):
        ja.validate_codegen_audit(make_valid_audit())

        same = make_valid_audit()
        same["aligned"]["ab"]["sha256"] = same["default"]["ab"]["sha256"]
        with self.assertRaises(ja.InvalidRun):
            ja.validate_codegen_audit(same)

        no_growth = make_valid_audit()
        no_growth["aligned"]["ab"]["text_bytes"] = no_growth["default"]["ab"]["text_bytes"]
        with self.assertRaises(ja.InvalidRun):
            ja.validate_codegen_audit(no_growth)

        leftover = make_valid_audit()
        leftover["aligned"]["station_balance"]["mibz2_unfused_32b_crossings"] = 2
        with self.assertRaises(ja.InvalidRun):
            ja.validate_codegen_audit(leftover)

    def test_station_balance_summary_and_void_on_missing(self):
        rows = make_station_rows()
        summary = ja.summarize_stations(rows)
        self.assertAlmostEqual(summary["wikidata"]["r_aligned"], 0.87 / 0.97, places=4)
        with self.assertRaises(ja.InvalidRun):
            ja.summarize_stations(rows[:-1])

    def test_cli_writes_void_outcome_on_non_i9(self):
        with tempfile.TemporaryDirectory() as d:
            out = Path(d)
            fake_lscpu = "Model name: AMD EPYC 7763 64-Core Processor\n"
            with patch("jcc_aligned.output", return_value=fake_lscpu):
                with self.assertRaises(ja.InvalidRun):
                    ja.machine(out)
            # Also exercise real CLI exit-code and outcome.txt write path
            proc = subprocess.run(
                [sys.executable, str(Path(__file__).parent / "jcc_aligned.py"), str(out)],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(proc.returncode, 0)
            outcome = (out / "outcome.txt").read_text().strip()
            self.assertTrue(outcome.startswith("VOID: "), outcome)


if __name__ == "__main__":
    unittest.main()
