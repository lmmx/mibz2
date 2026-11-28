"""Unit tests for bench/block_size_route.py (C1a block-size-route i9 calibration probe)."""

from __future__ import annotations

import json
import math
from pathlib import Path
import sys
import unittest

BENCH_DIR = Path(__file__).resolve().parent
if str(BENCH_DIR) not in sys.path:
    sys.path.insert(0, str(BENCH_DIR))

import block_size_route as bsr  # noqa: E402


def _make_sweep(
    corpus: str,
    level: int,
    order: str,
    round_idx: int,
    *,
    n_rows: int,
    open_ms_per_blk: float,
    seg_ms_per_blk: float,
    rn: float = 0.095,
    n_blocks: int = 20,
) -> dict:
    blocks = [
        {
            "route": "Open",
            "n": n_rows,
            "rn": rn,
            "open_ms": open_ms_per_blk,
            "seg_ms": seg_ms_per_blk,
            "m_est": 1000,
            "thick": 50,
            "cap_m": 30000,
        }
        for _ in range(n_blocks)
    ]
    agg = bsr.aggregate_sweep_blocks(blocks)
    return {
        "round": round_idx,
        "corpus": corpus,
        "level": level,
        "file": f"{corpus}-L{level}.bz2",
        "shadow_order": order,
        "reps": bsr.REPS,
        "stdout": {
            "file": f"{corpus}-L{level}.bz2",
            "blocks": n_blocks,
            "full": n_blocks,
            "column": 0,
            "open": n_blocks,
            "runs_not_open": 0,
            "bytes": n_rows * n_blocks,
            "inv_ms_open": open_ms_per_blk * n_blocks,
            "inv_ms_runs_not_open": 0.0,
            "inv_ms_column": 0.0,
            "probe_ms": open_ms_per_blk * n_blocks,
            "segmented_ms": seg_ms_per_blk * n_blocks,
            "lazy_ms": seg_ms_per_blk * n_blocks,
            "probe_over_segmented": open_ms_per_blk / seg_ms_per_blk,
        },
        **agg,
    }


def _full_sweeps(level_ratios: dict[tuple[str, int], tuple[int, float, float]]) -> list[dict]:
    sweeps = []
    for r in range(bsr.ROUNDS):
        for order in bsr.SHADOW_ORDERS:
            for (corpus, lv), (n_rows, o_ms, s_ms) in level_ratios.items():
                sweeps.append(
                    _make_sweep(
                        corpus,
                        lv,
                        order,
                        r,
                        n_rows=n_rows,
                        open_ms_per_blk=o_ms,
                        seg_ms_per_blk=s_ms,
                    )
                )
    return sweeps


def _write_valid_archive(out: Path, sweeps: list[dict], *, overrides: dict | None = None) -> None:
    ov = overrides or {}
    prov = {
        "head": "0" * 40,
        "smoke": False,
        "strict_i9": True,
        "rounds": bsr.ROUNDS,
        "reps": bsr.REPS,
        "settle_s": bsr.SETTLE_S,
        "route_full": bsr.ROUTE_FULL,
        "wikidata_levels": list(bsr.WIKIDATA_LEVELS),
        "text_levels": list(bsr.TEXT_LEVELS),
        "shadow_orders": list(bsr.SHADOW_ORDERS),
        "pin_cpu": 0,
        **ov.get("prov", {}),
    }
    host = {
        "n_cores": 10,
        "n_cpus": 20,
        "cpus_allowed": list(range(20)),
        "pin_cpu": 0,
        "require_i9": True,
        **ov.get("host", {}),
    }
    env_facts = ov.get("env_facts", {"cpu_quota": {"state": "unlimited", "cpus": None}})
    parity = ov.get("parity", {"all_ok": True, "checks": []})
    bsr.save(out / "provenance.json", prov)
    bsr.save(out / "host.json", host)
    bsr.save(out / "env-facts.json", env_facts)
    bsr.save(out / "parity.json", parity)
    with (out / "sweeps.jsonl").open("w", encoding="utf-8") as fh:
        for sw in sweeps:
            fh.write(json.dumps(sw, allow_nan=False) + "\n")


class TestBlockSizeRoute(unittest.TestCase):
    def test_baseline_env_guard(self) -> None:
        bsr.check_baseline_env({})
        for var in (*bsr.RUSTFLAGS_ENV_VARS, *bsr.PROFILE_ENV_VARS):
            with self.assertRaises(bsr.InvalidRun):
                bsr.check_baseline_env({var: "1"})

    def test_check_no_quota_scalar_and_dict(self) -> None:
        bsr.check_no_quota({"cpu_quota": None})
        bsr.check_no_quota({"cpu_quota": {"state": "unlimited", "cpus": None}})
        for bad in (
            {},
            {"cpu_quota": 2.0},
            {"cpu_quota": "limited"},
            {"cpu_quota": {"state": "limited", "cpus": 2.0}},
            {"cpu_quota": {"state": "unlimited", "cpus": 4.0}},
        ):
            with self.assertRaises(bsr.InvalidRun):
                bsr.check_no_quota(bad)

    def test_sanitize_json_forbids_nan(self) -> None:
        data = {"ok": 1.5, "nan": float("nan"), "inf": float("inf"), "nested": [float("-inf"), 2.0]}
        clean = bsr._sanitize_json(data)
        self.assertEqual(clean, {"ok": 1.5, "nan": None, "inf": None, "nested": [None, 2.0]})
        encoded = json.dumps(clean, allow_nan=False)
        self.assertIn("null", encoded)

    def test_parse_route_census_output(self) -> None:
        stdout = (
            "file\tblocks\tfull\tcolumn\topen\truns_not_open\trn_lt015\trn_015_028\trn_ge028"
            "\trn_p10\trn_p50\trn_p90\tbytes\tinv_ms_open\tinv_ms_runs_not_open\tinv_ms_column"
            "\tprobe_ms\tsegmented_ms\tlazy_ms\tprobe_over_segmented\n"
            "wikidata-L1.bz2\t3\t2\t0\t2\t0\t2\t0\t0\t0.090\t0.095\t0.100\t200000\t1.2\t0.0\t0.0"
            "\t2.4\t2.0\t2.2\t1.200\n"
        )
        stderr = (
            "wikidata-L1.bz2\tOpen\t99981\t0.0950\t0.600\t0.500\t9000\t200\t29994\n"
            "wikidata-L1.bz2\tOpen\t99981\t0.1050\t0.650\t0.500\t9500\t210\t29994\n"
            "wikidata-L1.bz2\tRunsNotOpen\t99981\t0.2500\t0.500\t0.500\t20000\t10\t29994\n"
        )
        parsed = bsr.parse_route_census_output(stdout, stderr)
        self.assertEqual(parsed["summary"]["bytes"], 200000)
        self.assertEqual(len(parsed["blocks"]), 3)
        agg = bsr.aggregate_sweep_blocks(parsed["blocks"])
        self.assertEqual(agg["open_blocks"], 2)
        self.assertAlmostEqual(agg["open_ms"], 1.25)
        self.assertAlmostEqual(agg["seg_ms"], 1.00)
        self.assertIn("<150k", agg["row_bins"])

        with self.assertRaises(bsr.InvalidRun):
            bsr.parse_route_census_output("file\tblocks\n", stderr)

    def test_score_confirmed_and_stop(self) -> None:
        confirmed_sweeps = _full_sweeps(
            {
                ("wikidata", 1): (99_981, 1.18, 1.00),  # 1.18x >= 1.05 -> CONFIRMED
                ("wikidata", 2): (199_981, 1.08, 1.00),
                ("wikidata", 3): (299_981, 1.00, 1.00),
                ("wikidata", 5): (499_981, 0.75, 1.00),
                ("wikidata", 9): (899_981, 0.50, 1.00),
                ("text", 1): (99_981, 1.02, 1.00),
                ("text", 9): (899_981, 0.90, 1.00),
            }
        )
        res = bsr.score(confirmed_sweeps, expected_rounds=bsr.ROUNDS)
        self.assertEqual(res["verdict"], "CONFIRMED")
        self.assertGreaterEqual(len(res["confirmed_bins"]), 2)
        self.assertTrue(res["candidate_rules_top5"])
        self.assertLess(res["candidate_rules_top5"][0]["wikidata_inv_pct"], 0.0)

        stop_sweeps = _full_sweeps(
            {
                ("wikidata", 1): (99_981, 0.96, 1.00),  # < 1.05
                ("wikidata", 2): (199_981, 0.85, 1.00),
                ("wikidata", 3): (299_981, 0.78, 1.00),
                ("wikidata", 5): (499_981, 0.65, 1.00),
                ("wikidata", 9): (899_981, 0.50, 1.00),
                ("text", 1): (99_981, 1.01, 1.00),
                ("text", 9): (899_981, 0.92, 1.00),
            }
        )
        res_stop = bsr.score(stop_sweeps, expected_rounds=bsr.ROUNDS)
        self.assertEqual(res_stop["verdict"], "STOP")
        self.assertEqual(res_stop["confirmed_bins"], [])

    def test_score_dir_fail_closed_provenance(self) -> None:
        import tempfile

        sweeps = _full_sweeps(
            {
                ("wikidata", 1): (99_981, 1.15, 1.00),
                ("wikidata", 2): (199_981, 1.02, 1.00),
                ("wikidata", 3): (299_981, 0.95, 1.00),
                ("wikidata", 5): (499_981, 0.75, 1.00),
                ("wikidata", 9): (899_981, 0.50, 1.00),
                ("text", 1): (99_981, 0.98, 1.00),
                ("text", 9): (899_981, 0.90, 1.00),
            }
        )
        with tempfile.TemporaryDirectory() as td:
            base = Path(td)
            d_ok = base / "ok"
            d_ok.mkdir()
            _write_valid_archive(d_ok, sweeps)
            self.assertEqual(bsr.score_dir(d_ok)["verdict"], "CONFIRMED")

            # Smoke -> VOID
            d_smoke = base / "smoke"
            d_smoke.mkdir()
            _write_valid_archive(d_smoke, sweeps, overrides={"prov": {"smoke": True}})
            self.assertEqual(bsr.score_dir(d_smoke)["verdict"], "VOID")

            # Quota -> VOID (dict and scalar)
            for tag, q in (
                ("q_dict", {"cpu_quota": {"state": "limited", "cpus": 2.0}}),
                ("q_scalar", {"cpu_quota": 2.0}),
            ):
                dq = base / tag
                dq.mkdir()
                _write_valid_archive(dq, sweeps, overrides={"env_facts": q})
                self.assertEqual(bsr.score_dir(dq)["verdict"], "VOID")

            # Non-i9 host -> VOID
            d_host = base / "host"
            d_host.mkdir()
            _write_valid_archive(d_host, sweeps, overrides={"host": {"n_cores": 2, "n_cpus": 2}})
            self.assertEqual(bsr.score_dir(d_host)["verdict"], "VOID")

            # Parity failure -> VOID
            d_par = base / "par"
            d_par.mkdir()
            _write_valid_archive(d_par, sweeps, overrides={"parity": {"all_ok": False}})
            self.assertEqual(bsr.score_dir(d_par)["verdict"], "VOID")

            # Missing sweeps -> VOID
            d_inc = base / "inc"
            d_inc.mkdir()
            _write_valid_archive(d_inc, sweeps[:-1])
            self.assertEqual(bsr.score_dir(d_inc)["verdict"], "VOID")

    def test_reencode_bz2_exact_parity(self) -> None:
        import hashlib
        import tempfile

        payload = b"alpha beta gamma delta epsilon 0123456789\n" * 6000
        expected_sha = hashlib.sha256(payload).hexdigest()
        with tempfile.TemporaryDirectory() as td:
            raw = Path(td) / "raw.bin"
            raw.write_bytes(payload)
            for lv in (1, 9):
                dst = Path(td) / f"out-L{lv}.bz2"
                bsr.reencode_bz2(raw, dst, lv)
                ref = bsr.reference(dst)
                self.assertEqual(ref["out"], len(payload))
                self.assertEqual(ref["sha256"], expected_sha)
                self.assertTrue(math.isfinite(float(ref["out"])))


if __name__ == "__main__":
    unittest.main()
