"""Unit tests for bench/forward_adopt_rescue.py (Clause 3 C6 physical i9 rescue recipe)."""

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

import forward_adopt_rescue as far  # noqa: E402

REFS = {
    "wikidata": {"out": 838_800_784, "sha256": "aa" * 32, "hash": "1111"},
    "text": {"out": 51_974_489, "sha256": "bb" * 32, "hash": "2222"},
    "records": {"out": 76_800_000, "sha256": "cc" * 32, "hash": "3333"},
    "random": {"out": 64_000_000, "sha256": "dd" * 32, "hash": "4444"},
    "runs": {"out": 80_001_500, "sha256": "ee" * 32, "hash": "5555"},
    "pattern": {"out": 156_800_000, "sha256": "ff" * 32, "hash": "6666"},
    "onebyte": {"out": 160_000_000, "sha256": "12" * 32, "hash": "7777"},
}

CAND_EXE = "/tmp/cand-ab-rescue"


def make_rows(
    *,
    rounds: int = far.ROUNDS,
    wall_overrides: dict[tuple[str, int], float] | None = None,
    wall_series_overrides: dict[tuple[str, int], list[float]] | None = None,
    cpu_overrides: dict[tuple[str, int], float] | None = None,
    rss_overrides: dict[tuple[str, int], float] | None = None,
    base_rss_overrides: dict[tuple[str, int], float] | None = None,
    cpr_pair: tuple[float, float] = (1.326, 1.326),
    jitter: float = 0.002,
) -> list[dict]:
    """Generate a full balanced AB/BA synthetic grid for Clause 3 scoring tests."""
    wall_overrides = wall_overrides or {}
    wall_series_overrides = wall_series_overrides or {}
    cpu_overrides = cpu_overrides or {}
    rss_overrides = rss_overrides or {}
    base_rss_overrides = base_rss_overrides or {}

    default_wall_ratio = {
        ("wikidata", 1): 0.850,
        ("wikidata", 6): 0.875,
        ("wikidata", 8): 0.945,
        ("wikidata", 10): 0.960,
        ("wikidata", 12): 0.965,
    }
    default_cpu_ratio = {
        ("wikidata", 1): 0.850,
        ("wikidata", 6): 0.875,
        ("wikidata", 8): 0.910,
        ("wikidata", 10): 0.915,
        ("wikidata", 12): 0.915,
    }
    default_rss_ratio = {
        ("wikidata", 1): 1.003,
        ("wikidata", 6): 1.010,
        ("wikidata", 8): 1.008,
        ("wikidata", 10): 1.012,
        ("wikidata", 12): 1.010,
    }

    rows: list[dict] = []
    for corpus, k in far.CELLS:
        arm_name = far.arm_for_threads(k)
        w_ratio = wall_overrides.get((corpus, k), default_wall_ratio.get((corpus, k), 0.998))
        c_ratio = cpu_overrides.get((corpus, k), default_cpu_ratio.get((corpus, k), 0.998))
        r_ratio = rss_overrides.get((corpus, k), default_rss_ratio.get((corpus, k), 1.002))
        base_w = (
            1.35 if (corpus == "wikidata" and k == 1) else (0.192 if corpus == "wikidata" else 0.14)
        )
        base_c = base_w * (1.0 if k == 1 else min(k, 8) * 1.1)
        default_base_r = (
            32_890.0
            if (corpus == "wikidata" and k == 1)
            else (62_400.0 if corpus == "wikidata" else 37_500.0)
        )
        base_r = base_rss_overrides.get((corpus, k), default_base_r)
        ref = REFS[corpus]
        series = wall_series_overrides.get((corpus, k))

        for r in range(rounds):
            delta = jitter * (1.0 if r % 2 == 0 else -1.0)
            b_wall = base_w * (1.0 + 0.001 * ((r % 3) - 1))
            b_cpu = base_c * (1.0 + 0.001 * ((r % 3) - 1))
            eff_w_ratio = series[r] if series is not None else (w_ratio + delta)
            b_row = {
                "arm": arm_name,
                "threads": k,
                "round": r,
                "corpus": corpus,
                "wall": b_wall,
                "cpu": b_cpu,
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
                "wall": b_wall * eff_w_ratio,
                "cpu": b_cpu * (c_ratio + delta),
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
    def test_adopt_when_all_clause3_gates_pass(self):
        # Full production BOOT_SAMPLES = 10_000 exercised end-to-end here.
        rows = make_rows()
        res = far.score(rows, REFS, cand_exe=CAND_EXE)
        self.assertEqual(res["verdict"], "ADOPT")
        self.assertTrue(all(res["clauses"].values()))
        self.assertEqual(len(res["cells"]), len(far.CELLS))
        w1 = res["cells"]["wikidata/1"]
        self.assertEqual(w1["arm_name"], "mibz2-workers-lazy")
        self.assertEqual(res["cells"]["wikidata/8"]["arm_name"], "mibz2")
        self.assertIsNotNone(w1["wall_s"]["position_stratified"]["combined_ratio"])
        self.assertAlmostEqual(w1["cycles_per_ref"]["ratio"], 1.0, places=4)
        self.assertLessEqual(res["cells"]["wikidata/8"]["cpu_s"]["t_ratio"], 0.940)
        self.assertLessEqual(w1["rss_kib"]["delta_median_kib"], 512.0)
        txt = far.format_summary(res)
        self.assertTrue(txt.startswith("ADOPT:"))
        self.assertIn("wikidata/6", txt)
        self.assertIn("wikidata/8", txt)
        self.assertIn("rss_gm", txt)

    def test_do_not_adopt_on_each_wikidata_wall_ci_threshold(self):
        with mock.patch.object(far, "BOOT_SAMPLES", 200):
            for k, bad_ratio in ((1, 0.905), (6, 0.905), (8, 0.980), (10, 0.990)):
                with self.subTest(k=k):
                    rows = make_rows(wall_overrides={("wikidata", k): bad_ratio})
                    res = far.score(rows, REFS, cand_exe=CAND_EXE)
                    self.assertEqual(res["verdict"], "DO NOT ADOPT")
                    self.assertIn(f"wikidata/{k}", res["reason"])

    def test_do_not_adopt_on_wikidata_8_median_even_when_ci_hi_passes(self):
        # Paired median = 0.965 > 0.960 while t_ci95_hi ~ 0.967 <= 0.975 and CPU passes.
        rows = make_rows(wall_overrides={("wikidata", 8): 0.965}, jitter=0.001)
        res = far.score(rows, REFS, cand_exe=CAND_EXE)
        self.assertEqual(res["verdict"], "DO NOT ADOPT")
        self.assertTrue(res["clauses"]["wikidata_8_hi_le_0.975"])
        self.assertFalse(res["clauses"]["wikidata_8_med_le_0.960"])
        self.assertTrue(res["clauses"]["wikidata_8_cpu_le_0.940"])
        self.assertIn("wikidata/8 paired_median=", res["reason"])

    def test_do_not_adopt_on_wikidata_8_ci_hi_even_when_median_passes(self):
        # 13 rounds at 0.950 (median=0.950 <= 0.960) plus 3 tail rounds at 1.070 push t_ci95_hi > 0.975.
        series = [0.950] * 13 + [1.070, 1.070, 1.070]
        rows = make_rows(wall_series_overrides={("wikidata", 8): series})
        res = far.score(rows, REFS, cand_exe=CAND_EXE)
        self.assertEqual(res["verdict"], "DO NOT ADOPT")
        self.assertFalse(res["clauses"]["wikidata_8_hi_le_0.975"])
        self.assertTrue(res["clauses"]["wikidata_8_med_le_0.960"])
        self.assertTrue(res["clauses"]["wikidata_8_cpu_le_0.940"])

    def test_do_not_adopt_on_wikidata_8_cpu_threshold(self):
        rows = make_rows(cpu_overrides={("wikidata", 8): 0.945})
        res = far.score(rows, REFS, cand_exe=CAND_EXE)
        self.assertEqual(res["verdict"], "DO NOT ADOPT")
        self.assertTrue(res["clauses"]["wikidata_8_hi_le_0.975"])
        self.assertTrue(res["clauses"]["wikidata_8_med_le_0.960"])
        self.assertFalse(res["clauses"]["wikidata_8_cpu_le_0.940"])
        self.assertIn("wikidata/8 cpu_t_ratio=", res["reason"])

    def test_do_not_adopt_on_guard_regression(self):
        with mock.patch.object(far, "BOOT_SAMPLES", 200):
            for corpus in far.GUARD_CORPORA:
                with self.subTest(corpus=corpus):
                    rows = make_rows(wall_overrides={(corpus, 1): 1.025})
                    res = far.score(rows, REFS, cand_exe=CAND_EXE)
                    self.assertEqual(res["verdict"], "DO NOT ADOPT")
                    self.assertFalse(res["clauses"]["guards_no_regression"])
                    self.assertIn(f"{corpus}/1", res["reason"])

    def test_do_not_adopt_on_calibrated_rss_limits(self):
        with mock.patch.object(far, "BOOT_SAMPLES", 200):
            # wikidata/1 paired GM > 1.020
            rows_w1_gm = make_rows(rss_overrides={("wikidata", 1): 1.025})
            res_w1_gm = far.score(rows_w1_gm, REFS, cand_exe=CAND_EXE)
            self.assertEqual(res_w1_gm["verdict"], "DO NOT ADOPT")
            self.assertFalse(res_w1_gm["clauses"]["wikidata_rss_ok"])

            # wikidata/1 absolute delta > +512 KiB (+0.5 MiB) even with paired GM <= 1.020
            rows_w1_abs = make_rows(
                base_rss_overrides={("wikidata", 1): 40_000.0},
                rss_overrides={("wikidata", 1): 1.015},  # +600 KiB > +512 KiB, GM 1.015 <= 1.020
            )
            res_w1_abs = far.score(rows_w1_abs, REFS, cand_exe=CAND_EXE)
            self.assertEqual(res_w1_abs["verdict"], "DO NOT ADOPT")
            self.assertFalse(res_w1_abs["clauses"]["wikidata_rss_ok"])
            self.assertIn("delta_rss=", res_w1_abs["reason"])

            # wikidata/{6,8,10,12} calibrated paired GM limit <= 1.030 (tighter than dropped 1.080)
            for k in (6, 8, 10, 12):
                with self.subTest(k=k):
                    rows_wk = make_rows(rss_overrides={("wikidata", k): 1.035})
                    res_wk = far.score(rows_wk, REFS, cand_exe=CAND_EXE)
                    self.assertEqual(res_wk["verdict"], "DO NOT ADOPT")
                    self.assertFalse(res_wk["clauses"]["wikidata_rss_ok"])
                    self.assertIn(f"wikidata/{k} rss_gm=", res_wk["reason"])

            # guard paired GM limit <= 1.050
            rows_g = make_rows(rss_overrides={("text", 8): 1.055})
            res_g = far.score(rows_g, REFS, cand_exe=CAND_EXE)
            self.assertEqual(res_g["verdict"], "DO NOT ADOPT")
            self.assertFalse(res_g["clauses"]["guards_rss_ok"])
            self.assertIn("text/8 rss_gm=", res_g["reason"])

    def test_void_conditions(self):
        base = make_rows()
        self.assertEqual(far.score(base, REFS, cand_exe=CAND_EXE, host_ok=False)["verdict"], "VOID")
        self.assertEqual(
            far.score(base, REFS, cand_exe=CAND_EXE, distinct_binaries=False)["verdict"], "VOID"
        )
        self.assertEqual(far.score(base[:-1], REFS, cand_exe=CAND_EXE)["verdict"], "VOID")
        self.assertEqual(
            far.score([*base, dict(base[0])], REFS, cand_exe=CAND_EXE)["verdict"], "VOID"
        )
        bad_hash = [dict(r) for r in base]
        bad_hash[0]["hash"] = "deadbeef"
        self.assertEqual(far.score(bad_hash, REFS, cand_exe=CAND_EXE)["verdict"], "VOID")
        bad_err = [dict(r) for r in base]
        bad_err[0]["err"] = "crc mismatch"
        self.assertEqual(far.score(bad_err, REFS, cand_exe=CAND_EXE)["verdict"], "VOID")
        bad_cpu = [dict(r) for r in base]
        bad_cpu[0]["cpu"] = 0.0
        self.assertEqual(far.score(bad_cpu, REFS, cand_exe=CAND_EXE)["verdict"], "VOID")
        missing_cpu = [dict(r) for r in base]
        del missing_cpu[0]["cpu"]
        self.assertEqual(far.score(missing_cpu, REFS, cand_exe=CAND_EXE)["verdict"], "VOID")
        # Auxiliary telemetry NaN (e.g. ab pre_busy: NaN) sanitizes to null without breaking JSON
        sanitized = far._sanitize_json(
            {"pre_busy": float("nan"), "wall": 0.5, "nested": [float("inf")]}
        )
        self.assertIsNone(sanitized["pre_busy"])
        self.assertEqual(sanitized["wall"], 0.5)
        self.assertEqual(sanitized["nested"], [None])
        json.dumps(sanitized, allow_nan=False)


class IsolationAndWiringTests(unittest.TestCase):
    def test_check_baseline_env_rejects_rustflags_and_profile_overrides(self):
        far.check_baseline_env({})
        for var in (*far.RUSTFLAGS_ENV_VARS, *far.PROFILE_ENV_VARS):
            with self.subTest(var=var), self.assertRaises(far.InvalidRun):
                far.check_baseline_env({var: "1"})

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
                (ex_dir / "ab").write_bytes(f"bin-{tdir.name}".encode())
                if "windows" in cmd:
                    (ex_dir / "windows").write_bytes(b"win-bin")

            with mock.patch("subprocess.run", side_effect=fake_run):
                bins, meta = far.build_arms(out, tmp)
            self.assertEqual(len(calls), 2)
            self.assertNotEqual(calls[0][1], calls[1][1])
            self.assertIn("--features", calls[1][0])
            self.assertIn("forward-anchor-lanes", calls[1][0])
            self.assertTrue(meta["distinct_binaries"])
            self.assertNotEqual(meta["base"]["sha256"], meta["cand"]["sha256"])
            self.assertTrue(bins["base_ab"].exists())
            self.assertTrue(bins["cand_ab"].exists())

            with self.assertRaises(far.InvalidRun):
                far.build_arms(
                    out,
                    tmp,
                    prebuilt_base=bins["base_ab"],
                    prebuilt_cand=bins["base_ab"],
                    prebuilt_windows=bins["windows"],
                )

    def _write_run_dir(
        self,
        out: Path,
        prov: dict,
        *,
        env_facts: dict | None = None,
        write_env_facts: bool = True,
        write_parity: bool = True,
    ) -> None:
        rows = make_rows(rounds=int(prov["rounds"]))
        (out / "provenance.json").write_text(json.dumps(prov))
        (out / "binaries.json").write_text(
            json.dumps(
                {
                    "base": {"path": "/tmp/base-ab", "sha256": "11" * 32},
                    "cand": {"path": CAND_EXE, "sha256": "22" * 32},
                    "distinct_binaries": True,
                }
            )
        )
        # As a real run writes them: inputs.json before parity (no 64-bit hash), then
        # parity.jsonl carrying each corpus's hash.
        inputs = {c: {k: v for k, v in ref.items() if k != "hash"} for c, ref in REFS.items()}
        (out / "inputs.json").write_text(json.dumps(inputs))
        if write_parity:
            (out / "parity.jsonl").write_text(
                "".join(
                    json.dumps({"arm": arm, "corpus": c, "hash": ref["hash"]}) + "\n"
                    for c, ref in REFS.items()
                    for arm in ("base", "cand")
                )
            )
        (out / "runs.jsonl").write_text("\n".join(json.dumps(r) for r in rows) + "\n")
        if write_env_facts:
            facts = (
                env_facts
                if env_facts is not None
                else {
                    "facts": "ab env --facts/1",
                    "cpu_quota": {
                        "state": "unlimited",
                        "cpus": None,
                        "hierarchy": "v2",
                        "levels": ["/sys/fs/cgroup"],
                        "why": None,
                    },
                }
            )
            (out / "env-facts.json").write_text(json.dumps(facts))

    def registered_prov(self, **changes) -> dict:
        return {
            "strict_i9": True,
            "base_features": far.BASE_FEATURES,
            "cand_features": far.CAND_FEATURES,
            "rounds": far.ROUNDS,
            "settle_s": far.SETTLE_S,
            "prespin_ms": far.PRESPIN_MS,
            "wikidata_counts": list(far.WIKIDATA_COUNTS),
            "guard_corpora": list(far.GUARD_CORPORA),
            "guard_counts": list(far.GUARD_COUNTS),
            **changes,
        }

    def test_score_dir_roundtrip(self):
        with tempfile.TemporaryDirectory() as td:
            out = Path(td)
            self._write_run_dir(out, self.registered_prov())
            res = far.score_dir(out)
            self.assertEqual(res["verdict"], "ADOPT")
            self.assertTrue((out / "summary.json").exists())
            self.assertTrue((out / "summary.txt").exists())
            self.assertTrue((out / "outcome.txt").read_text().startswith("ADOPT:"))

    def test_score_dir_unregistered_run_is_void(self):
        with mock.patch.object(far, "BOOT_SAMPLES", 200):
            for changes in (
                {"strict_i9": False},
                {"rounds": 4},
                {"settle_s": "0"},
                {"prespin_ms": 0},
                {"base_features": "other"},
                {"cand_features": "default"},
            ):
                with self.subTest(changes=changes), tempfile.TemporaryDirectory() as td:
                    out = Path(td)
                    self._write_run_dir(out, self.registered_prov(**changes))
                    res = far.score_dir(out)
                    self.assertEqual(res["verdict"], "VOID")
                    self.assertIn("not the registered run", res["reason"])
                    self.assertTrue((out / "outcome.txt").read_text().startswith("VOID:"))

    def test_score_dir_registered_without_parity_is_void(self):
        with mock.patch.object(far, "BOOT_SAMPLES", 200), tempfile.TemporaryDirectory() as td:
            out = Path(td)
            self._write_run_dir(out, self.registered_prov(), write_parity=False)
            res = far.score_dir(out)
            self.assertEqual(res["verdict"], "VOID")
            self.assertIn("parity.jsonl", res["reason"])

    def test_unregistered_run_outcome_is_void(self):
        live = {"verdict": "ADOPT", "reason": "all frozen Clause 3 C6 adoption clauses passed"}
        self.assertEqual(far.mark_unregistered(live, True)["verdict"], "ADOPT")
        marked = far.mark_unregistered(live, False)
        self.assertEqual(marked["verdict"], "VOID")
        self.assertIn("not the registered run", marked["reason"])
        void = {"verdict": "VOID", "reason": "x"}
        self.assertEqual(far.mark_unregistered(void, False), void)

    def test_non_strict_run_adoption_writes_void(self):
        # `run --allow-any-host` (or `--smoke`) scores a full grid that passes every clause:
        # its outcome must still read VOID, never a live ADOPT.
        rows = make_rows(rounds=far.ROUNDS)
        host = {"pin_cpu": 0}
        bins = {"base_ab": Path("/tmp/base-ab"), "cand_ab": Path(CAND_EXE)}
        meta = {"distinct_binaries": True}
        files = {c: Path(f"/tmp/{c}.bz2") for c in REFS}
        with (
            mock.patch.object(far, "BOOT_SAMPLES", 200),
            mock.patch.object(far, "check_host_i9", return_value=host),
            mock.patch.object(far, "build_arms", return_value=(bins, meta)),
            mock.patch.object(far, "prepare_inputs", return_value=(files, dict(REFS))),
            mock.patch.object(far, "verify_parity", return_value=[]),
            mock.patch.object(far, "output", return_value="{}"),
            mock.patch.object(far, "run_ab_group", side_effect=[rows, [], [], []]),
            tempfile.TemporaryDirectory() as td,
        ):
            out = Path(td)
            res = far.run_adoption(out, allow_any_host=True)
            self.assertEqual(res["verdict"], "VOID")
            self.assertIn("not the registered run", res["reason"])
            self.assertIn("all frozen Clause 3", res["reason"])
            self.assertTrue((out / "outcome.txt").read_text().startswith("VOID:"))

    def test_score_dir_missing_or_limited_quota_is_void(self):
        with tempfile.TemporaryDirectory() as td:
            out = Path(td)
            self._write_run_dir(out, self.registered_prov(), write_env_facts=False)
            res = far.score_dir(out)
            self.assertEqual(res["verdict"], "VOID")
            self.assertIn("env-facts.json", res["reason"])

        with tempfile.TemporaryDirectory() as td:
            out = Path(td)
            self._write_run_dir(
                out,
                self.registered_prov(),
                env_facts={"cpu_quota": {"state": "limited", "cpus": 8.0}},
            )
            res = far.score_dir(out)
            self.assertEqual(res["verdict"], "VOID")
            self.assertIn("quota", res["reason"])

    def test_quota_is_void(self):
        far.check_no_quota({"cpu_quota": None})
        far.check_no_quota(
            {
                "facts": "ab env --facts/1",
                "cpu_quota": {
                    "state": "unlimited",
                    "cpus": None,
                    "hierarchy": "v2",
                    "levels": ["/sys/fs/cgroup"],
                    "why": None,
                },
            }
        )
        for bad in (
            {},
            {"cpu_quota": 8.0},
            {"cpu_quota": "unlimited"},
            {"cpu_quota": {"state": "limited", "cpus": 4.0}},
            {"cpu_quota": {"state": "unknown", "cpus": None}},
            {"cpu_quota": {"state": "unlimited", "cpus": 8.0}},
        ):
            with self.subTest(bad=bad), self.assertRaises(far.InvalidRun):
                far.check_no_quota(bad)

    def test_strict_run_refuses_all_overrides(self):
        for kwargs, expected_flag in (
            ({"rounds": 8}, "--rounds"),
            ({"settle_s": "0"}, "--settle"),
            ({"prespin_ms": 0}, "--prespin-ms"),
            ({"prebuilt_base": Path("/tmp/a")}, "--base-ab"),
            ({"prebuilt_cand": Path("/tmp/b")}, "--cand-ab"),
            ({"prebuilt_windows": Path("/tmp/w")}, "--windows-exe"),
        ):
            with (
                self.subTest(flag=expected_flag),
                tempfile.TemporaryDirectory() as td,
                self.assertRaises(far.InvalidRun) as ctx,
            ):
                far.run_adoption(Path(td), **kwargs)
            self.assertIn(expected_flag, str(ctx.exception))

    def test_just_recipe_registration_and_defaults(self):
        self.assertEqual(far.SETTLE_S, "2")
        self.assertEqual(far.ROUNDS, 16)
        self.assertEqual(far.PRESPIN_MS, 100)
        self.assertEqual(far.WIKIDATA_COUNTS, (1, 6, 8, 10, 12))
        self.assertEqual(
            far.GUARD_CORPORA, ("text", "records", "random", "runs", "pattern", "onebyte")
        )
        self.assertEqual(far.GUARD_COUNTS, (1, 8))
        self.assertEqual(far.WIKIDATA_WALL_HI_MAX, {1: 0.900, 6: 0.900, 8: 0.975, 10: 0.985})
        self.assertEqual(far.WIKIDATA_8_MEDIAN_MAX, 0.960)
        self.assertEqual(far.WIKIDATA_8_CPU_MAX, 0.940)
        self.assertEqual(
            far.WIKIDATA_RSS_GM_MAX, {1: 1.020, 6: 1.030, 8: 1.030, 10: 1.030, 12: 1.030}
        )
        self.assertEqual(far.WIKIDATA_1_RSS_DELTA_KIB_MAX, 512.0)
        self.assertEqual(far.GUARD_RSS_GM_MAX, 1.050)
        self.assertTrue(far.REGISTRATION.exists())
        just_file = (far.ROOT / ".just/forward-adopt-rescue.just").read_text(encoding="utf-8")
        self.assertIn("bench-forward-adopt-rescue", just_file)
        self.assertIn("bench/bundle.sh", just_file)
        self.assertIn("python3 bench/forward_adopt_rescue.py run", just_file)
        root_just = (far.ROOT / "Justfile").read_text(encoding="utf-8")
        self.assertIn('import ".just/forward-adopt-rescue.just"', root_just)


if __name__ == "__main__":
    unittest.main()
