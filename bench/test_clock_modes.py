"""Exhaustive pytest / unittest suite for bench/clock_modes.py (i9-clock-modes)."""

from __future__ import annotations

import bz2
import hashlib
import json
import math
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

BENCH_DIR = Path(__file__).resolve().parent
if str(BENCH_DIR) not in sys.path:
    sys.path.insert(0, str(BENCH_DIR))

import clock_modes as cm  # noqa: E402

REF = {"wall_out": 838_800_784, "ab_hash": "deadbeef12345678"}

# Compact real-data fixtures from origin/evidence (bce3914d 1T and 77236472 #1119 6T)
# so mode-robust estimator tests run deterministically even in shallow CI checkouts.
BCE3914D_1T_WALLS = [
    # (round, pos0_arm, pos0_wall, pos1_arm, pos1_wall) — mibz2=0.5.15, old=0.5.14
    (0, "mibz2", 0.3952, "old", 0.4031),
    (1, "old", 0.3673, "mibz2", 0.3955),
    (2, "mibz2", 0.3563, "old", 0.3999),
    (3, "old", 0.3898, "mibz2", 0.3636),
    (4, "mibz2", 0.3504, "old", 0.3955),
    (5, "old", 0.3915, "mibz2", 0.3557),
    (6, "mibz2", 0.3580, "old", 0.3970),
    (7, "old", 0.3610, "mibz2", 0.3540),
    (8, "mibz2", 0.3960, "old", 0.3940),
    (9, "old", 0.3980, "mibz2", 0.3590),
]

JCC1119_6T_PAIRS = [
    # (round, pos0_arm, pos0_wall, pos1_arm, pos1_wall) — exact alternating pos1 slow (~1.45x)
    (0, "mibz2", 0.2445, "aligned", 0.3554),
    (1, "aligned", 0.2452, "mibz2", 0.3565),
    (2, "aligned", 0.2501, "mibz2", 0.3630),
    (3, "mibz2", 0.2417, "aligned", 0.3589),
    (4, "mibz2", 0.2409, "aligned", 0.3657),
    (5, "aligned", 0.2435, "mibz2", 0.3550),
    (6, "mibz2", 0.2420, "aligned", 0.3575),
    (7, "aligned", 0.2448, "mibz2", 0.3568),
    (8, "aligned", 0.2455, "mibz2", 0.3580),
    (9, "mibz2", 0.2415, "aligned", 0.3560),
    (10, "mibz2", 0.2422, "aligned", 0.3592),
    (11, "aligned", 0.2440, "mibz2", 0.3558),
]


def make_factorial_rows(
    *,
    rounds: int = cm.ROUNDS,
    threads: tuple[int, ...] = cm.THREADS,
    pinnings: tuple[str, ...] = cm.PINNINGS,
    settles: tuple[float, ...] = cm.SETTLES,
    prespin_ms_values: tuple[int, ...] = cm.DEFAULT_PRESPIN_MS,
    wall_fn=None,
    cycles_fn=None,
) -> list[dict]:
    """Generate a complete synthetic factorial dataset matching `factorial_plan`."""
    plan = cm.factorial_plan(
        rounds=rounds,
        threads=threads,
        pinnings=pinnings,
        settles=settles,
        prespin_ms_values=prespin_ms_values,
    )
    rows = []
    for item in plan:
        t = item["threads"]
        base_wall = 1.380 if t == 1 else 0.244
        base_cycles = 7_000_000_000 if t == 1 else 7_200_000_000
        wall = wall_fn(item, base_wall) if wall_fn else base_wall
        cycles = cycles_fn(item, base_cycles) if cycles_fn else base_cycles
        cpb = (cycles / REF["wall_out"]) if cycles is not None else None
        rows.append(
            {
                **item,
                "arm": "mibz2",
                "corpus": "wikidata",
                "wall": wall,
                "cpu": wall * (1.0 if t == 1 else 5.5),
                "rss_kb": 40_000,
                "out": REF["wall_out"],
                "hash": REF["ab_hash"],
                "err": "",
                "termination": "success",
                "cycles": cycles,
                "instructions": int(cycles * 1.4) if cycles is not None else None,
                "ref_cycles": int(wall * 3_700_000_000) if cycles is not None else None,
                "cycles_per_ref": round(cycles / (wall * 3_700_000_000), 4)
                if cycles is not None
                else None,
                "cycles_per_byte": round(cpb, 6) if cpb is not None else None,
                "perf_available": cycles is not None,
                "perf_source": "ab_perf" if cycles is not None else "none",
                "placement": {
                    "sample_count": 10,
                    "cpus_sampled": [0] if (t == 1 and item["pinning"] == "pinned") else [0, 1, 2],
                    "n_distinct_cpus": 1 if (t == 1 and item["pinning"] == "pinned") else 3,
                    "n_distinct_cores": 1 if (t == 1 and item["pinning"] == "pinned") else 3,
                    "smt_sibling_overlap_samples": 0,
                    "migrations": 0,
                    "per_tid_cpus": {"1": [0]},
                },
            }
        )
    return rows


class FactorialDesignTests(unittest.TestCase):
    def test_default_2x2x2_plan_is_balanced_and_deterministic(self):
        p1 = cm.factorial_plan()
        p2 = cm.factorial_plan()
        self.assertEqual(p1, p2)
        # 12 rounds * 2 threads * 2 pinnings * 2 settles * 2 pair_pos = 192 runs
        self.assertEqual(len(p1), 192)
        round_orders = []
        for r in range(cm.ROUNDS):
            r_items = [x for x in p1 if x["round"] == r]
            self.assertEqual(len(r_items), 16)
            # Every consecutive pair shares pair_id and goes pos 0 -> pos 1.
            pair_seq = []
            for idx in range(0, len(r_items), 2):
                a, b = r_items[idx], r_items[idx + 1]
                self.assertEqual(a["pair_id"], b["pair_id"])
                self.assertEqual(a["pair_pos"], 0)
                self.assertEqual(b["pair_pos"], 1)
                for key in ("threads", "pinning", "settle_s", "prespin_ms"):
                    self.assertEqual(a[key], b[key])
                pair_seq.append((a["threads"], a["pinning"], a["settle_s"]))
            self.assertEqual(len(set(pair_seq)), 8)
            round_orders.append(tuple(pair_seq))
        # Shuffling must vary across rounds.
        self.assertGreater(len(set(round_orders)), 1)

    def test_prespin_arm_expands_factorial_cleanly(self):
        plan = cm.factorial_plan(rounds=4, prespin_ms_values=(0, 100))
        self.assertEqual(len(plan), 4 * 2 * 2 * 2 * 2 * 2)
        counts = {0: 0, 100: 0}
        for item in plan:
            counts[item["prespin_ms"]] += 1
        self.assertEqual(counts[0], counts[100])


class StopTestClassifierTests(unittest.TestCase):
    def test_pass_mechanism_found_two_way_interaction_6t(self):
        # Exact #1119 signature: 6T unpinned & pair_pos==1 runs 1.45x slower at constant cycles.
        def wall_fn(item, base):
            if item["threads"] == 6 and item["pinning"] == "unpinned" and item["pair_pos"] == 1:
                return base * 1.45
            return base

        rows = make_factorial_rows(wall_fn=wall_fn)
        res = cm.score(rows, REF)
        self.assertEqual(res["verdict"], "PASS (mechanism found)")
        pt6 = res["per_thread"]["6"]
        self.assertEqual(pt6["verdict"], "PASS (mechanism found)")
        self.assertEqual(pt6["mode_counts"]["slow_clock"], 24)
        self.assertEqual(pt6["mode_counts"]["slow_work"], 0)
        self.assertEqual(pt6["best_explainer"]["label"], "pinning=unpinned & pair_pos=1")
        self.assertAlmostEqual(pt6["best_explainer"]["explained_share"], 1.0)
        self.assertAlmostEqual(pt6["best_explainer"]["slow_rate"], 1.0)
        txt = cm.format_summary(res)
        self.assertTrue(txt.startswith("PASS (mechanism found):"))
        self.assertIn("pinning=unpinned & pair_pos=1", txt)

    def test_pass_mechanism_found_single_factor_1t(self):
        # Mode (a): 1T pair_pos==0 runs 1.055x slower at constant cycles (within 3% cpb).
        def wall_fn(item, base):
            if item["threads"] == 1 and item["pair_pos"] == 0:
                return base * 1.055
            return base

        rows = make_factorial_rows(wall_fn=wall_fn)
        res = cm.score(rows, REF)
        self.assertEqual(res["verdict"], "PASS (mechanism found)")
        pt1 = res["per_thread"]["1"]
        self.assertEqual(pt1["verdict"], "PASS (mechanism found)")
        self.assertEqual(pt1["best_explainer"]["label"], "pair_pos=0")
        self.assertAlmostEqual(pt1["best_explainer"]["explained_share"], 1.0)

    def test_exact_80_percent_explanation_boundary(self):
        # 20 slow_clock runs at 6T: 16 in pinning=unpinned (80.0%) -> PASS; 15/20 (75.0%) -> STOP.
        def build_with_unpinned_count(n_unpinned_slow: int):
            unpinned_left = [n_unpinned_slow]
            pinned_left = [20 - n_unpinned_slow]

            def wall_fn(item, base):
                if item["threads"] != 6:
                    return base
                if item["pinning"] == "unpinned" and unpinned_left[0] > 0:
                    unpinned_left[0] -= 1
                    return base * 1.45
                if item["pinning"] == "pinned" and pinned_left[0] > 0:
                    pinned_left[0] -= 1
                    return base * 1.45
                return base

            return make_factorial_rows(wall_fn=wall_fn)

        res_80 = cm.score(build_with_unpinned_count(16), REF)
        self.assertEqual(res_80["verdict"], "PASS (mechanism found)")
        self.assertAlmostEqual(res_80["per_thread"]["6"]["best_explainer"]["explained_share"], 0.80)

        res_75 = cm.score(build_with_unpinned_count(15), REF)
        self.assertEqual(res_75["verdict"], "STOP")

    def test_pass_mode_is_real_work_when_cycles_ge_1_2x(self):
        # Slow 6T unpinned runs execute 1.28x cycles -> contention/placement, not clock.
        def wall_fn(item, base):
            if item["threads"] == 6 and item["pinning"] == "unpinned":
                return base * 1.40
            return base

        def cycles_fn(item, base_c):
            if item["threads"] == 6 and item["pinning"] == "unpinned":
                return int(base_c * 1.28)
            return base_c

        rows = make_factorial_rows(wall_fn=wall_fn, cycles_fn=cycles_fn)
        res = cm.score(rows, REF)
        self.assertEqual(res["verdict"], "PASS (mode is real work)")
        pt6 = res["per_thread"]["6"]
        self.assertEqual(pt6["mode_counts"]["slow_work"], 48)
        self.assertEqual(pt6["mode_counts"]["slow_clock"], 0)
        self.assertGreaterEqual(pt6["median_slow_cpb_ratio"], 1.20)

    def test_stop_when_unimodal_or_uniform_or_no_perf_or_single_outlier(self):
        # 1. Completely unimodal -> STOP
        self.assertEqual(cm.score(make_factorial_rows(), REF)["verdict"], "STOP")

        # 2. Single stray outlier -> STOP (guarded by min_slow_runs=3)
        def one_outlier(item, base):
            if (
                item["threads"] == 6
                and item["round"] == 0
                and item["pinning"] == "unpinned"
                and item["settle_s"] == 0.0
                and item["pair_pos"] == 1
            ):
                return base * 1.50
            return base

        self.assertEqual(cm.score(make_factorial_rows(wall_fn=one_outlier), REF)["verdict"], "STOP")

        # 3. Slow runs spread uniformly across every cell (round < 4) -> 50%/25% < 80% -> STOP
        def uniform_slow(item, base):
            if item["threads"] == 6 and item["round"] < 4:
                return base * 1.45
            return base

        res_unif = cm.score(make_factorial_rows(wall_fn=uniform_slow), REF)
        self.assertEqual(res_unif["verdict"], "STOP")
        self.assertEqual(res_unif["per_thread"]["6"]["mode_counts"]["slow_clock"], 32)

        # 4. Wall-slow runs without PMU cycles -> slow_unclassified -> STOP (cannot claim clock)
        def unpinned_slow(item, base):
            return base * 1.45 if (item["threads"] == 6 and item["pinning"] == "unpinned") else base

        rows_noperf = make_factorial_rows(wall_fn=unpinned_slow, cycles_fn=lambda _i, _b: None)
        res_noperf = cm.score(rows_noperf, REF)
        self.assertEqual(res_noperf["verdict"], "STOP")
        self.assertEqual(res_noperf["per_thread"]["6"]["mode_counts"]["slow_unclassified"], 48)

    def test_void_conditions(self):
        base = make_factorial_rows()
        self.assertEqual(cm.score(base, REF, host_ok=False)["verdict"], "VOID")
        self.assertEqual(cm.score(base[:-1], REF)["verdict"], "VOID")
        self.assertEqual(cm.score([*base, dict(base[0])], REF)["verdict"], "VOID")

        bad_err = [dict(r) for r in base]
        bad_err[0]["err"] = "crc mismatch"
        self.assertEqual(cm.score(bad_err, REF)["verdict"], "VOID")

        bad_out = [dict(r) for r in base]
        bad_out[0]["out"] += 1
        self.assertEqual(cm.score(bad_out, REF)["verdict"], "VOID")

        bad_hash = [dict(r) for r in base]
        bad_hash[0]["hash"] = "wrong"
        self.assertEqual(cm.score(bad_hash, REF)["verdict"], "VOID")

        bad_nan = [dict(r) for r in base]
        bad_nan[0]["wall"] = math.nan
        self.assertEqual(cm.score(bad_nan, REF)["verdict"], "VOID")

    def test_baseline_env_and_cpu_quota_checks(self):
        cm.check_baseline_env({})
        for var in ("RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
            with self.assertRaises(cm.InvalidRun):
                cm.check_baseline_env({var: "-C target-cpu=native"})
        cm.check_cpu_quota_unlimited({"cpu_quota": None})
        cm.check_cpu_quota_unlimited({"cpu_quota": {"state": "unlimited", "cpus": None}})
        with self.assertRaises(cm.InvalidRun):
            cm.check_cpu_quota_unlimited({"cpu_quota": {"state": "limit", "cpus": 4.0}})


class TelemetryAndPlacementTests(unittest.TestCase):
    def test_parse_proc_stat_cpu_handles_comm_with_spaces_and_parens(self):
        # Real /proc/self/stat must parse cleanly to a non-negative integer CPU id.
        self_stat = Path("/proc/self/stat").read_text(encoding="utf-8")
        cpu = cm.parse_proc_stat_cpu(self_stat)
        self.assertIsNotNone(cpu)
        self.assertGreaterEqual(cpu, 0)

        # Synthetic comm containing spaces and nested parens: field 39 is index 36 after last `)`.
        fields = ["R"] + ["0"] * 35 + ["17", "0", "0"]
        synthetic = f"4242 (mibz2 worker (0)) {' '.join(fields)}\n"
        self.assertEqual(cm.parse_proc_stat_comm_cpu(synthetic), ("mibz2 worker (0)", 17))
        self.assertEqual(cm.parse_proc_stat_cpu(synthetic), 17)
        taskset_line = f"4243 (taskset) {' '.join(fields)}\n"
        self.assertEqual(cm.parse_proc_stat_comm_cpu(taskset_line), ("taskset", 17))
        self.assertIn("taskset", cm.PlacementSampler._WRAPPER_COMMS)
        self.assertIsNone(cm.parse_proc_stat_cpu("malformed"))

    def test_placement_sampler_live_child(self):
        proc = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(0.06)"])
        sampler = cm.PlacementSampler(proc.pid, core_map={0: 0, 1: 1, 2: 0, 3: 1}, interval_s=0.005)
        sampler.start()
        proc.wait(timeout=5)
        info = sampler.stop()
        self.assertGreaterEqual(info["sample_count"], 1)
        self.assertGreaterEqual(info["n_distinct_cpus"], 1)

    def test_parse_perf_stat_csv_and_msr_wrap(self):
        sample_err = (
            "123456789,,cycles:u,1000000,100.00\n"
            "234567890,,instructions:u,1000000,100.00\n"
            "98765432,,ref-cycles:u,1000000,100.00\n"
            "<not counted>,,cache-misses,0,0.00\n"
        )
        parsed = cm.parse_perf_stat_csv(sample_err)
        self.assertEqual(parsed["cycles"], 123456789)
        self.assertEqual(parsed["instructions"], 234567890)
        self.assertEqual(parsed["ref_cycles"], 98765432)

        # Test 64-bit unsigned wrap-around in MSR APERF/MPERF delta.
        top = (1 << 64) - 10
        ratio = cm.msr_aperf_mperf_ratio({0: (top, top)}, {0: (140, 90)})
        self.assertIsNotNone(ratio)
        self.assertEqual(ratio["delta_aperf"], 150)
        self.assertEqual(ratio["delta_mperf"], 100)
        self.assertAlmostEqual(ratio["aperf_per_mperf"], 1.5)


class ModeRobustEstimatorTests(unittest.TestCase):
    def test_synthetic_cycles_per_ref_separates_mode_bias(self):
        # True speedup of arm vs ref is 1.01x, but arm lands in slow mode 8/10 vs ref 2/10.
        rows = []
        for r in range(10):
            arm_slow = r < 8
            ref_slow = r < 2
            ref_wall = 0.350 * (1.10 if ref_slow else 1.0)
            arm_wall = 0.3535 * (1.10 if arm_slow else 1.0)
            rows.append(
                {
                    "round": r,
                    "corpus": "wikidata",
                    "threads": 1,
                    "arm": "mibz2",
                    "wall": ref_wall,
                    "cycles_per_ref": 1.29 if ref_slow else 1.43,
                    "err": "",
                }
            )
            rows.append(
                {
                    "round": r,
                    "corpus": "wikidata",
                    "threads": 1,
                    "arm": "old",
                    "wall": arm_wall,
                    "cycles_per_ref": 1.29 if arm_slow else 1.43,
                    "err": "",
                }
            )
        res = cm.analyze_ab_runs(rows, ref_arm="mibz2", target_arm="old")
        c = res["cells"][0]
        # Pooled is inflated by mode count imbalance (~1.069), while within-mode recovers ~1.010!
        self.assertGreater(c["pooled"]["ratio"], 1.05)
        self.assertEqual(c["within_mode"]["basis"], "cycles_per_ref")
        self.assertEqual(c["within_mode"]["arm_mode_counts"], {"fast": 2, "slow": 8})
        self.assertEqual(c["within_mode"]["ref_mode_counts"], {"fast": 8, "slow": 2})
        self.assertAlmostEqual(c["within_mode"]["by_mode"]["fast"]["ratio"], 1.01, places=3)
        self.assertAlmostEqual(c["within_mode"]["by_mode"]["slow"]["ratio"], 1.01, places=3)

    def test_real_bce3914d_1t_fixture_recovers_within_mode_ratios(self):
        rows = []
        for r, a0, w0, a1, w1 in BCE3914D_1T_WALLS:
            rows.append(
                {"round": r, "corpus": "wikidata", "threads": 1, "arm": a0, "wall": w0, "err": ""}
            )
            rows.append(
                {"round": r, "corpus": "wikidata", "threads": 1, "arm": a1, "wall": w1, "err": ""}
            )
        res = cm.analyze_ab_runs(rows, ref_arm="mibz2", target_arm="old")
        c = res["cells"][0]
        self.assertGreater(c["pooled"]["ratio"], 1.04)
        self.assertEqual(c["within_mode"]["arm_mode_counts"]["slow"], 8)
        self.assertEqual(c["within_mode"]["ref_mode_counts"]["slow"], 3)
        self.assertLess(abs(c["within_mode"]["by_mode"]["fast"]["ratio"] - 1.018), 0.015)
        self.assertLess(abs(c["within_mode"]["by_mode"]["slow"]["ratio"] - 1.002), 0.015)

    def test_real_77236472_6t_fixture_collapses_ci_via_position_stratification(self):
        rows = []
        for r, a0, w0, a1, w1 in JCC1119_6T_PAIRS:
            rows.append(
                {"round": r, "corpus": "wikidata", "threads": 6, "arm": a0, "wall": w0, "err": ""}
            )
            rows.append(
                {"round": r, "corpus": "wikidata", "threads": 6, "arm": a1, "wall": w1, "err": ""}
            )
        res = cm.analyze_ab_runs(rows, ref_arm="mibz2", target_arm="aligned")
        c = res["cells"][0]
        pooled_width = c["pooled"]["ci_hi"] - c["pooled"]["ci_lo"]
        pos0 = c["position_stratified"]["by_pos"]["0"]
        pos1 = c["position_stratified"]["by_pos"]["1"]
        pos0_width = pos0["ci_hi"] - pos0["ci_lo"]
        pos1_width = pos1["ci_hi"] - pos1["ci_lo"]
        self.assertGreater(pooled_width, 0.40)
        self.assertLess(pos0_width, 0.04)
        self.assertLess(pos1_width, 0.04)
        self.assertAlmostEqual(pos1["ratio"], 1.000, places=2)


class ShallowCloneAndIsolationTests(unittest.TestCase):
    def test_materialize_wikidata_is_shallow_safe_on_depth1_checkout(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            upstream = root / "upstream"
            runner = root / "runner"
            work_tmp = root / "work"
            work_tmp.mkdir()

            def git(repo: Path, *args: str) -> str:
                return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()

            subprocess.run(["git", "init", "-q", "-b", "master", str(upstream)], check=True)
            git(upstream, "config", "user.name", "Test")
            git(upstream, "config", "user.email", "test@example.invalid")

            # Create a bench-data orphan branch holding one tiny verified window blob.
            window_bytes = b"synthetic-wikidata-window-payload"
            window_sha = hashlib.sha256(window_bytes).hexdigest()
            base_bz2 = bz2.compress(b"synthetic-base" * 200, compresslevel=9)
            base_sha = hashlib.sha256(base_bz2).hexdigest()
            x4_sha = hashlib.sha256(base_bz2 * 4).hexdigest()

            (upstream / "bench").mkdir()
            spec = {
                "windows_dir": "windows",
                "corpus_sha256": base_sha,
                "windows": [{"name": "w0", "sha256": window_sha}],
            }
            (upstream / "bench/wikidata.json").write_text(json.dumps(spec), encoding="utf-8")
            git(upstream, "add", ".")
            git(upstream, "commit", "-qm", "master spec")

            git(upstream, "checkout", "--orphan", "bench-data")
            git(upstream, "rm", "-rfq", ".")
            (upstream / "windows").mkdir()
            (upstream / "windows/w0.bin").write_bytes(window_bytes)
            git(upstream, "add", ".")
            git(upstream, "commit", "-qm", "bench-data window")
            git(upstream, "checkout", "-q", "master")

            # Shallow --depth=1 --single-branch clone has NO bench-data ref locally.
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
            git(runner, "checkout", "-q", "--detach", runner_head)
            self.assertEqual(git(runner, "rev-parse", "--is-shallow-repository"), "true")

            def fake_build(_spec, windows_dir, _windows_bin, dest):
                self.assertEqual((Path(windows_dir) / "w0.bin").read_bytes(), window_bytes)
                Path(dest).write_bytes(base_bz2)

            with (
                mock.patch.object(cm.wikidata, "build", side_effect=fake_build),
                mock.patch.object(cm, "WIKIDATA_X4_SHA256", x4_sha),
            ):
                info = cm.materialize_wikidata(work_tmp, runner, Path("/bin/true"))

            self.assertEqual(info["base_bz2_sha256"], base_sha)
            self.assertEqual(info["wall_bz2_sha256"], x4_sha)
            # Shallow runner HEAD and refs must remain completely untouched.
            self.assertEqual(git(runner, "rev-parse", "HEAD"), runner_head)
            self.assertNotIn("bench-data", git(runner, "branch", "-a"))

    def test_build_binaries_uses_isolated_cargo_target_dir(self):
        with tempfile.TemporaryDirectory() as scratch:
            tmp = Path(scratch)
            seen_env = {}

            def fake_run(cmd, cwd, env, check):
                seen_env["CARGO_TARGET_DIR"] = env.get("CARGO_TARGET_DIR")
                seen_env["cmd"] = cmd
                rel = Path(env["CARGO_TARGET_DIR"]) / "release/examples"
                rel.mkdir(parents=True, exist_ok=True)
                (rel / "ab").write_text("bin-ab", encoding="utf-8")
                (rel / "windows").write_text("bin-win", encoding="utf-8")

            with mock.patch.object(cm.subprocess, "run", side_effect=fake_run):
                bins = cm.build_binaries(cm.ROOT, tmp, include_windows=True, check_env=False)

            self.assertEqual(seen_env["CARGO_TARGET_DIR"], str(tmp / "target"))
            self.assertIn("--locked", seen_env["cmd"])
            self.assertTrue(bins["ab"].exists())
            self.assertTrue(bins["windows"].exists())


class CliAndRecipeTests(unittest.TestCase):
    def test_cli_parsing_and_score_analyze_subcommands(self):
        parser = cm.build_parser()
        ns = parser.parse_args(
            [
                "run",
                "/tmp/out",
                "--smoke",
                "--rounds",
                "2",
                "--prespin-ms",
                "0,100",
                "--threads",
                "1,6",
            ]
        )
        self.assertTrue(ns.smoke)
        self.assertEqual(ns.rounds, 2)
        self.assertEqual(ns.prespin_ms, (0, 100))
        self.assertEqual(ns.threads, (1, 6))

        with tempfile.TemporaryDirectory() as scratch:
            tmp = Path(scratch)
            # Write synthetic factorial rows -> score subcommand
            rows = make_factorial_rows(
                rounds=4,
                wall_fn=lambda item, base: (
                    base * 1.45
                    if (item["threads"] == 6 and item["pinning"] == "unpinned")
                    else base
                ),
            )
            jsonl = tmp / "runs.jsonl"
            jsonl.write_text("\n".join(json.dumps(r) for r in rows) + "\n", encoding="utf-8")
            out_dir = tmp / "scored"
            rc = cm.main(["score", str(jsonl), "--out", str(out_dir)])
            self.assertEqual(rc, 0)
            self.assertTrue((out_dir / "summary.json").exists())
            self.assertTrue((out_dir / "summary.txt").exists())
            self.assertTrue(
                (out_dir / "outcome.txt").read_text().startswith("PASS (mechanism found):")
            )

            # Write bce3914d fixture -> analyze subcommand
            ab_rows = []
            for r, a0, w0, a1, w1 in BCE3914D_1T_WALLS:
                ab_rows.append(
                    {
                        "round": r,
                        "corpus": "wikidata",
                        "threads": 1,
                        "arm": a0,
                        "wall": w0,
                        "err": "",
                    }
                )
                ab_rows.append(
                    {
                        "round": r,
                        "corpus": "wikidata",
                        "threads": 1,
                        "arm": a1,
                        "wall": w1,
                        "err": "",
                    }
                )
            ab_jsonl = tmp / "ab.jsonl"
            ab_jsonl.write_text("\n".join(json.dumps(r) for r in ab_rows) + "\n", encoding="utf-8")
            rc2 = cm.main(["analyze", str(ab_jsonl), "--ref", "mibz2", "--arm", "old", "--json"])
            self.assertEqual(rc2, 0)

    def test_just_recipe_and_registration_wiring(self):
        just_recipe = (cm.ROOT / ".just/clock-modes.just").read_text(encoding="utf-8")
        self.assertIn("bench-clock-modes", just_recipe)
        self.assertIn("RUSTUP_TOOLCHAIN=1.99.1", just_recipe)
        self.assertIn("bash bench/bundle.sh", just_recipe)
        self.assertIn("python3 bench/clock_modes.py run", just_recipe)
        justfile = (cm.ROOT / "Justfile").read_text(encoding="utf-8")
        self.assertIn('import ".just/clock-modes.just"', justfile)
        self.assertTrue(cm.REGISTRATION.exists())

    def test_bundle_sh_smoke_with_submit_bundles(self):
        with tempfile.TemporaryDirectory() as scratch:
            tmp = Path(scratch)
            run_dir = tmp / "clock-modes" / "run-20261127T000000Z"
            run_dir.mkdir(parents=True)
            (run_dir / "outcome.txt").write_text("STOP: smoke\n", encoding="utf-8")
            (run_dir / "summary.txt").write_text("summary\n", encoding="utf-8")
            (run_dir / "runs.jsonl").write_text("{}\n", encoding="utf-8")
            submit_list = tmp / "submit_bundles.txt"
            env = {**cm.os.environ, "SUBMIT_BUNDLES": str(submit_list)}
            subprocess.run(
                ["bash", str(cm.ROOT / "bench/bundle.sh"), str(run_dir)],
                cwd=cm.ROOT,
                env=env,
                check=True,
                capture_output=True,
                text=True,
            )
            recorded = [ln.strip() for ln in submit_list.read_text().splitlines() if ln.strip()]
            self.assertEqual(len(recorded), 1)
            tarball = Path(recorded[0])
            self.assertTrue(tarball.exists())
            self.assertEqual(tarball.name, "run-20261127T000000Z.tar.gz")


if __name__ == "__main__":
    unittest.main()
