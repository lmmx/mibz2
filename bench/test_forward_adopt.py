"""Unit tests for bench/forward_adopt.py (C6 physical i9 full-guard adoption recipe)."""

from __future__ import annotations

import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

BENCH_DIR = Path(__file__).resolve().parent
if str(BENCH_DIR) not in sys.path:
    sys.path.insert(0, str(BENCH_DIR))

import forward_adopt as fa  # noqa: E402

REFS = {
    "wikidata": {"out": 838_800_784, "sha256": "aa" * 32, "hash": "1111"},
    "text": {"out": 51_974_489, "sha256": "bb" * 32, "hash": "2222"},
    "records": {"out": 76_800_000, "sha256": "cc" * 32, "hash": "3333"},
    "random": {"out": 64_000_000, "sha256": "dd" * 32, "hash": "4444"},
    "runs": {"out": 80_001_500, "sha256": "ee" * 32, "hash": "5555"},
    "pattern": {"out": 156_800_000, "sha256": "ff" * 32, "hash": "6666"},
    "onebyte": {"out": 160_000_000, "sha256": "12" * 32, "hash": "7777"},
}

CAND_EXE = "/tmp/cand-ab"


def make_rows(
    *,
    rounds: int = fa.ROUNDS,
    wall_overrides: dict[tuple[str, int], float] | None = None,
    rss_overrides: dict[tuple[str, int], float] | None = None,
    cpr_pair: tuple[float, float] = (1.32, 1.32),
    jitter: float = 0.002,
) -> list[dict]:
    """Generate a full balanced AB/BA synthetic grid for scoring tests."""
    wall_overrides = wall_overrides or {}
    rss_overrides = rss_overrides or {}
    default_wall_ratio = {
        ("wikidata", 1): 0.830,
        ("wikidata", 6): 0.915,
        ("wikidata", 8): 0.943,
        ("wikidata", 10): 0.960,
        ("wikidata", 12): 0.965,
    }
    default_rss_ratio = {
        ("wikidata", 1): 1.012,
        ("wikidata", 6): 1.055,
        ("wikidata", 8): 1.061,
        ("wikidata", 10): 1.062,
        ("wikidata", 12): 1.064,
    }

    rows: list[dict] = []
    for corpus, k in fa.CELLS:
        arm_name = fa.arm_for_threads(k)
        w_ratio = wall_overrides.get((corpus, k), default_wall_ratio.get((corpus, k), 0.998))
        r_ratio = rss_overrides.get((corpus, k), default_rss_ratio.get((corpus, k), 1.002))
        base_w = (
            1.35 if (corpus == "wikidata" and k == 1) else (0.196 if corpus == "wikidata" else 0.14)
        )
        base_r = (
            80_800.0
            if (corpus == "wikidata" and k == 1)
            else (106_400.0 if corpus == "wikidata" else 37_500.0)
        )
        ref = REFS[corpus]

        for r in range(rounds):
            # Deterministic small symmetric jitter around target ratio
            delta = jitter * (1.0 if r % 2 == 0 else -1.0)
            b_wall = base_w * (1.0 + 0.001 * ((r % 3) - 1))
            b_row = {
                "arm": arm_name,
                "threads": k,
                "round": r,
                "corpus": corpus,
                "wall": b_wall,
                "rss_kb": base_r,
                "out": ref["out"],
                "hash": ref["hash"],
                "err": "",
                "cycles_per_ref": cpr_pair[0],
            }
            c_row = {
                "arm": f"{arm_name}@{CAND_EXE}",
                "threads": k,
                "round": r,
                "corpus": corpus,
                "wall": b_wall * (w_ratio + delta),
                "rss_kb": base_r * r_ratio,
                "out": ref["out"],
                "hash": ref["hash"],
                "err": "",
                "cycles_per_ref": cpr_pair[1],
            }
            if r % 2 == 0:
                rows.extend([b_row, c_row])
            else:
                rows.extend([c_row, b_row])
    return rows


class ScoringTests(unittest.TestCase):
    def test_adopt_when_all_clauses_pass(self):
        rows = make_rows()
        res = fa.score(rows, REFS, cand_exe=CAND_EXE)
        self.assertEqual(res["verdict"], "ADOPT")
        self.assertTrue(all(res["clauses"].values()))
        self.assertEqual(len(res["cells"]), len(fa.CELLS))
        # Verify position-stratified CI and cycles_per_ref are populated
        w1 = res["cells"]["wikidata/1"]
        self.assertEqual(w1["arm_name"], "mibz2-workers-lazy")
        self.assertEqual(res["cells"]["wikidata/8"]["arm_name"], "mibz2")
        self.assertIsNotNone(w1["wall_s"]["position_stratified"]["combined_ratio"])
        self.assertAlmostEqual(w1["cycles_per_ref"]["ratio"], 1.0, places=4)
        txt = fa.format_summary(res)
        self.assertTrue(txt.startswith("ADOPT:"))
        self.assertIn("wikidata/8", txt)

    def test_do_not_adopt_on_each_wikidata_threshold(self):
        for k, bad_ratio in ((1, 0.905), (8, 0.970), (10, 0.990)):
            with self.subTest(k=k):
                rows = make_rows(wall_overrides={("wikidata", k): bad_ratio})
                res = fa.score(rows, REFS, cand_exe=CAND_EXE)
                self.assertEqual(res["verdict"], "DO NOT ADOPT")
                self.assertIn(f"wikidata/{k}", res["reason"])

    def test_do_not_adopt_on_guard_regression(self):
        for corpus in fa.GUARD_CORPORA:
            with self.subTest(corpus=corpus):
                rows = make_rows(wall_overrides={(corpus, 1): 1.025})
                res = fa.score(rows, REFS, cand_exe=CAND_EXE)
                self.assertEqual(res["verdict"], "DO NOT ADOPT")
                self.assertFalse(res["clauses"]["guards_no_regression"])
                self.assertIn(f"{corpus}/1", res["reason"])

    def test_do_not_adopt_on_rss_limit(self):
        rows_w1 = make_rows(rss_overrides={("wikidata", 1): 1.025})
        res_w1 = fa.score(rows_w1, REFS, cand_exe=CAND_EXE)
        self.assertEqual(res_w1["verdict"], "DO NOT ADOPT")
        self.assertFalse(res_w1["clauses"]["wikidata_rss_ok"])

        rows_w8 = make_rows(rss_overrides={("wikidata", 8): 1.085})
        res_w8 = fa.score(rows_w8, REFS, cand_exe=CAND_EXE)
        self.assertEqual(res_w8["verdict"], "DO NOT ADOPT")
        self.assertFalse(res_w8["clauses"]["wikidata_rss_ok"])

        rows_g = make_rows(rss_overrides={("text", 8): 1.055})
        res_g = fa.score(rows_g, REFS, cand_exe=CAND_EXE)
        self.assertEqual(res_g["verdict"], "DO NOT ADOPT")
        self.assertFalse(res_g["clauses"]["guards_rss_ok"])

    def test_void_conditions(self):
        base = make_rows()
        # Non-i9 host
        self.assertEqual(fa.score(base, REFS, cand_exe=CAND_EXE, host_ok=False)["verdict"], "VOID")
        # Identical binaries
        self.assertEqual(
            fa.score(base, REFS, cand_exe=CAND_EXE, distinct_binaries=False)["verdict"], "VOID"
        )
        # Missing row
        self.assertEqual(fa.score(base[:-1], REFS, cand_exe=CAND_EXE)["verdict"], "VOID")
        # Duplicate row
        self.assertEqual(
            fa.score([*base, dict(base[0])], REFS, cand_exe=CAND_EXE)["verdict"], "VOID"
        )
        # Output hash mismatch
        bad_hash = [dict(r) for r in base]
        bad_hash[0]["hash"] = "deadbeef"
        self.assertEqual(fa.score(bad_hash, REFS, cand_exe=CAND_EXE)["verdict"], "VOID")
        # Decoder error
        bad_err = [dict(r) for r in base]
        bad_err[0]["err"] = "crc mismatch"
        self.assertEqual(fa.score(bad_err, REFS, cand_exe=CAND_EXE)["verdict"], "VOID")


class IsolationAndWiringTests(unittest.TestCase):
    def test_check_baseline_env_rejects_rustflags(self):
        fa.check_baseline_env({})
        for var in fa.RUSTFLAGS_ENV_VARS:
            with self.assertRaises(fa.InvalidRun):
                fa.check_baseline_env({var: "-C target-cpu=native"})

    def test_build_arms_uses_isolated_target_dirs_and_enforces_distinct_binaries(self):
        with tempfile.TemporaryDirectory() as td:
            out = Path(td) / "out"
            out.mkdir()
            tmp = Path(td) / "tmp"
            tmp.mkdir()
            calls: list[tuple[list[str], str]] = []

            def fake_run(cmd, cwd=None, check=False, env=None):
                tdir = Path(env["CARGO_TARGET_DIR"])
                calls.append((cmd, str(tdir)))
                ex_dir = tdir / "release/examples"
                ex_dir.mkdir(parents=True, exist_ok=True)
                # Write distinct bytes per target dir
                (ex_dir / "ab").write_bytes(f"bin-{tdir.name}".encode())
                if "windows" in cmd:
                    (ex_dir / "windows").write_bytes(b"win-bin")

            with mock.patch("subprocess.run", side_effect=fake_run):
                bins, meta = fa.build_arms(out, tmp)
            self.assertEqual(len(calls), 2)
            self.assertNotEqual(calls[0][1], calls[1][1])
            self.assertIn("--features", calls[1][0])
            self.assertIn("forward-anchor-lanes", calls[1][0])
            self.assertTrue(meta["distinct_binaries"])
            self.assertNotEqual(meta["base"]["sha256"], meta["cand"]["sha256"])
            self.assertTrue(bins["base_ab"].exists())
            self.assertTrue(bins["cand_ab"].exists())

            # Identical prebuilt binaries MUST fail VOID
            with self.assertRaises(fa.InvalidRun):
                fa.build_arms(
                    out,
                    tmp,
                    prebuilt_base=bins["base_ab"],
                    prebuilt_cand=bins["base_ab"],
                    prebuilt_windows=bins["windows"],
                )

    def test_score_dir_roundtrip(self):
        with tempfile.TemporaryDirectory() as td:
            out = Path(td)
            rows = make_rows(rounds=4)
            (out / "provenance.json").write_text(
                json.dumps(
                    {
                        "rounds": 4,
                        "wikidata_counts": list(fa.WIKIDATA_COUNTS),
                        "guard_corpora": list(fa.GUARD_CORPORA),
                        "guard_counts": list(fa.GUARD_COUNTS),
                    }
                )
            )
            (out / "binaries.json").write_text(
                json.dumps(
                    {
                        "base": {"path": "/tmp/base-ab", "sha256": "11" * 32},
                        "cand": {"path": CAND_EXE, "sha256": "22" * 32},
                        "distinct_binaries": True,
                    }
                )
            )
            (out / "inputs.json").write_text(json.dumps(REFS))
            (out / "runs.jsonl").write_text("\n".join(json.dumps(r) for r in rows) + "\n")
            res = fa.score_dir(out)
            self.assertEqual(res["verdict"], "ADOPT")
            self.assertTrue((out / "summary.json").exists())
            self.assertTrue((out / "summary.txt").exists())
            self.assertTrue((out / "outcome.txt").read_text().startswith("ADOPT:"))

    def test_just_recipe_registration_and_defaults(self):
        self.assertEqual(fa.SETTLE_S, "2")
        self.assertEqual(fa.ROUNDS, 16)
        self.assertEqual(fa.WIKIDATA_COUNTS, (1, 6, 8, 10, 12))
        self.assertEqual(
            fa.GUARD_CORPORA, ("text", "records", "random", "runs", "pattern", "onebyte")
        )
        self.assertEqual(fa.GUARD_COUNTS, (1, 8))
        self.assertTrue(fa.REGISTRATION.exists())
        just_file = (fa.ROOT / ".just/forward-adopt.just").read_text(encoding="utf-8")
        self.assertIn("bench-forward-adopt", just_file)
        self.assertIn("bench/bundle.sh", just_file)
        self.assertIn("python3 bench/forward_adopt.py run", just_file)
        root_just = (fa.ROOT / "Justfile").read_text(encoding="utf-8")
        self.assertIn('import ".just/forward-adopt.just"', root_just)


if __name__ == "__main__":
    unittest.main()
