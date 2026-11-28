"""Unit tests for bench/scanner_contention.py (`scanner-contention-residual` PMU probe)."""

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

import scanner_contention as sc  # noqa: E402

REF = {"out": 838_800_784, "sha256": "aa" * 32, "scanned": 1724}


def make_rows(
    *,
    rounds: int = sc.ROUNDS,
    base_84_ratio: float = 1.001,
    lanes_84_ratio: float = 1.060,
    lanes_l3_excess_frac: float = 0.70,
    jitter: float = 0.002,
) -> list[dict]:
    """Generate a synthetic balanced per-thread PMU grid across base/lanes x k={1,4,8}."""
    rows: list[dict] = []
    base_4w_cyc = 820_000_000.0
    base_4w_l3 = 90_000_000.0
    for r in range(rounds):
        delta = jitter * (1.0 if r % 2 == 0 else -1.0)
        for arm in sc.ARMS:
            for k in sc.COUNTS:
                cyc = base_4w_cyc * (1.0 + 0.001 * ((r % 3) - 1))
                l3 = base_4w_l3
                if k == 8:
                    ratio = base_84_ratio if arm == "base" else lanes_84_ratio
                    new_cyc = cyc * (ratio + delta)
                    if arm == "lanes":
                        l3 = base_4w_l3 + (new_cyc - cyc) * lanes_l3_excess_frac
                    else:
                        l3 = base_4w_l3 + max(0.0, new_cyc - cyc) * 0.5
                    cyc = new_cyc
                clock_ratio = 1.427 if k == 1 else 1.327
                ref_cyc = cyc / clock_ratio
                rows.append(
                    {
                        "round": r,
                        "arm": arm,
                        "k": k,
                        "wall_s": 0.19 if k == 8 else (0.34 if k == 4 else 1.25),
                        "peak_kib": 106_000.0,
                        "out": REF["out"],
                        "scanned": REF["scanned"],
                        "scanner_cpu": 0,
                        "bmi2": True,
                        "pmu": {
                            "scanner_thread": "mibz2-scan-4242",
                            "cycles": cyc,
                            "ref_cycles": ref_cyc,
                            "instructions": 1_350_000_000.0,
                            "stalls_l3_miss": l3,
                            "llc_load_misses": l3 / 110.0,
                            "clock_ratio": clock_ratio,
                            "ipc": 1_350_000_000.0 / cyc,
                            "l3_stall_share": l3 / cyc,
                        },
                    }
                )
    return rows


class PerfParsingTests(unittest.TestCase):
    def test_parse_perf_standard_cpu_csv_and_slash_modifiers(self):
        csv_text = "\n".join(
            [
                "# started on Sat Nov 28 05:00:00 2026",
                "850000000,,cycles:u,180000000,100.00,,",
                "640000000,,ref-cycles:u,180000000,100.00,,",
                "1400000000,,instructions:u,180000000,100.00,1.65,insn per cycle",
                "120000000,,cpu/cycle_activity.stalls_l3_miss/u,180000000,100.00,,",
                "1100000,,LLC-load-misses:u,180000000,100.00,,",
            ]
        )
        pmu = sc.parse_perf_scanner_csv(csv_text, scanner_cpu=0)
        self.assertEqual(pmu["scanner_scope"], "CPU0")
        self.assertEqual(pmu["cycles"], 850_000_000.0)
        self.assertEqual(pmu["ref_cycles"], 640_000_000.0)
        self.assertEqual(pmu["stalls_l3_miss"], 120_000_000.0)
        self.assertEqual(pmu["llc_load_misses"], 1_100_000.0)
        self.assertAlmostEqual(pmu["clock_ratio"], 850.0 / 640.0, places=5)

    def test_parse_perf_per_thread_csv_extracts_scanner_only(self):
        csv_text = "\n".join(
            [
                "# started on Sat Nov 28 04:00:00 2026",
                "mibz2-0-1001,900000000,,cycles:u,180000000,100.00,,",
                "mibz2-scan-1002,850000000,,cycles:u,180000000,100.00,,",
                "mibz2-scan-1002,640000000,,ref-cycles:u,180000000,100.00,,",
                "mibz2-scan-1002,1400000000,,instructions:u,180000000,100.00,1.65,insn per cycle",
                "mibz2-scan-1002,120000000,,cycle_activity.stalls_l3_miss:u,180000000,100.00,,",
                "mibz2-scan-1002,1100000,,LLC-load-misses:u,180000000,100.00,,",
            ]
        )
        pmu = sc.parse_perf_per_thread_csv(csv_text)
        self.assertEqual(pmu["scanner_scope"], "mibz2-scan-1002")
        self.assertEqual(pmu["cycles"], 850_000_000.0)
        self.assertEqual(pmu["ref_cycles"], 640_000_000.0)
        self.assertEqual(pmu["stalls_l3_miss"], 120_000_000.0)
        self.assertEqual(pmu["llc_load_misses"], 1_100_000.0)
        self.assertAlmostEqual(pmu["clock_ratio"], 850.0 / 640.0, places=5)

    def test_parse_perf_rejects_missing_thread_or_uncounted_event(self):
        with self.assertRaises(sc.InvalidRun):
            sc.parse_perf_scanner_csv("mibz2-0-1001,100,,cycles:u,10,100.00\n")
        bad_ev = "\n".join(
            [
                "850000000,,cycles:u,180000000,100.00",
                "640000000,,ref-cycles:u,180000000,100.00",
                "1400000000,,instructions:u,180000000,100.00",
                "<not supported>,,cycle_activity.stalls_l3_miss:u,0,100.00",
                "1100000,,LLC-load-misses:u,180000000,100.00",
            ]
        )
        with self.assertRaises(sc.InvalidRun):
            sc.parse_perf_scanner_csv(bad_ev, scanner_cpu=0)

    def test_perf_stat_cmd_uses_cpu_pin_never_per_thread_with_workload(self):
        cmd = sc.build_perf_stat_cmd("perf", 0, Path("/tmp/p.csv"), sc.PERF_EVENTS, ["true"])
        self.assertNotIn("--per-thread", cmd)
        self.assertIn("-C", cmd)
        self.assertEqual(cmd[cmd.index("-C") + 1], "0")
        self.assertIn("-o", cmd)

    def test_preflight_perf_falls_back_from_u_to_plain_events(self):
        with tempfile.TemporaryDirectory() as td:
            out = Path(td)
            seen_cmds: list[list[str]] = []

            def fake_run(cmd, capture_output=False, text=False, timeout=None):
                seen_cmds.append(cmd)
                self.assertNotIn("--per-thread", cmd)
                csv_path = Path(cmd[cmd.index("-o") + 1])
                if len(seen_cmds) == 1:
                    # Simulate :u rejected on a PMU alias
                    return mock.Mock(returncode=1, stdout="", stderr="invalid event modifier")
                csv_path.write_text(
                    "\n".join(
                        [
                            "10000000,,cycles,1000000,100.00",
                            "7000000,,ref-cycles,1000000,100.00",
                            "15000000,,instructions,1000000,100.00",
                            "100000,,cycle_activity.stalls_l3_miss,1000000,100.00",
                            "1000,,LLC-load-misses,1000000,100.00",
                        ]
                    )
                    + "\n",
                    encoding="utf-8",
                )
                return mock.Mock(returncode=0, stdout="", stderr="")

            with mock.patch("subprocess.run", side_effect=fake_run):
                active = sc.preflight_perf(out, scanner_cpu=0)
            self.assertEqual(active, sc.PERF_EVENTS_FALLBACK)
            self.assertEqual(len(seen_cmds), 2)


class ScoringTests(unittest.TestCase):
    def test_confirmed_when_lanes_8w_over_4w_exceeds_1_and_l3_explains_majority(self):
        rows = make_rows(base_84_ratio=1.002, lanes_84_ratio=1.060, lanes_l3_excess_frac=0.72)
        res = sc.score(rows, REF)
        self.assertEqual(res["verdict"], "CONFIRMED")
        self.assertTrue(res["clauses"]["lanes_8w_over_4w_ci_above_1"])
        self.assertTrue(res["clauses"]["lanes_l3_stall_share_ge_0.50"])
        txt = sc.format_summary(res)
        self.assertTrue(txt.startswith("CONFIRMED:"))
        self.assertIn("lanes/8", txt)

    def test_refuted_when_both_arms_include_1(self):
        rows = make_rows(base_84_ratio=1.000, lanes_84_ratio=1.000, jitter=0.004)
        res = sc.score(rows, REF)
        self.assertEqual(res["verdict"], "REFUTED")
        self.assertTrue(res["clauses"]["base_8w_over_4w_includes_1"])
        self.assertTrue(res["clauses"]["lanes_8w_over_4w_includes_1"])

    def test_inconclusive_when_lanes_exceeds_1_but_l3_below_half(self):
        rows = make_rows(base_84_ratio=1.000, lanes_84_ratio=1.060, lanes_l3_excess_frac=0.20)
        res = sc.score(rows, REF)
        self.assertEqual(res["verdict"], "INCONCLUSIVE")
        self.assertTrue(res["clauses"]["lanes_8w_over_4w_ci_above_1"])
        self.assertFalse(res["clauses"]["lanes_l3_stall_share_ge_0.50"])

    def test_void_conditions(self):
        rows = make_rows()
        self.assertEqual(sc.score(rows, REF, host_ok=False)["verdict"], "VOID")
        self.assertEqual(sc.score(rows, REF, distinct_binaries=False)["verdict"], "VOID")
        self.assertEqual(sc.score(rows[:-1], REF)["verdict"], "VOID")
        self.assertEqual(sc.score([*rows, dict(rows[0])], REF)["verdict"], "VOID")
        bad_scan = [dict(r) for r in rows]
        bad_scan[0]["scanned"] = 999
        self.assertEqual(sc.score(bad_scan, REF)["verdict"], "VOID")


class IsolationAndWiringTests(unittest.TestCase):
    def test_check_baseline_env_rejects_rustflags_and_profile(self):
        sc.check_baseline_env({})
        for var in (*sc.RUSTFLAGS_ENV_VARS, *sc.PROFILE_ENV_VARS):
            with self.assertRaises(sc.InvalidRun):
                sc.check_baseline_env({var: "1"})

    def test_quota_is_void(self):
        sc.check_no_quota({"cpu_quota": None})
        sc.check_no_quota(
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
            {"cpu_quota": {"state": "limited", "cpus": 4.0}},
            {"cpu_quota": {"state": "unknown", "cpus": None}},
            {"cpu_quota": {"state": "unlimited", "cpus": 8.0}},
        ):
            with self.subTest(bad=bad), self.assertRaises(sc.InvalidRun):
                sc.check_no_quota(bad)

    def test_strict_run_refuses_overrides(self):
        for kwargs, flag in (
            ({"rounds": 9}, "--rounds"),
            ({"settle_s": "0"}, "--settle"),
            ({"stub_pmu": True}, "--stub-pmu"),
            ({"perf_cmd": "/bin/true"}, "--perf-cmd"),
        ):
            with (
                self.subTest(flag=flag),
                tempfile.TemporaryDirectory() as td,
                self.assertRaises(sc.InvalidRun) as ctx,
            ):
                sc.run_probe(Path(td), **kwargs)
            self.assertIn(flag, str(ctx.exception))

    def test_build_arms_uses_isolated_target_dirs_and_distinct_binaries(self):
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
                (ex_dir / "scanner_coalesced_run").write_bytes(f"run-{tdir.name}".encode())
                if "ab" in cmd:
                    (ex_dir / "ab").write_bytes(b"ab-bin")
                if "windows" in cmd:
                    (ex_dir / "windows").write_bytes(b"win-bin")

            with mock.patch("subprocess.run", side_effect=fake_run):
                bins, meta = sc.build_arms(out, tmp)
            self.assertEqual(len(calls), 2)
            self.assertNotEqual(calls[0][1], calls[1][1])
            self.assertIn("scan-oracle forward-anchor-lanes", calls[1][0])
            self.assertTrue(meta["distinct_binaries"])

            with self.assertRaises(sc.InvalidRun):
                sc.build_arms(
                    out,
                    tmp,
                    prebuilt_base=bins["base"],
                    prebuilt_lanes=bins["base"],
                    prebuilt_ab=bins["ab"],
                    prebuilt_windows=bins["windows"],
                )

    def _write_run_dir(self, out: Path, prov: dict) -> None:
        rows = make_rows(rounds=int(prov["rounds"]))
        (out / "provenance.json").write_text(json.dumps(prov), encoding="utf-8")
        (out / "binaries.json").write_text(
            json.dumps(
                {
                    "base": {"path": "/tmp/base-run", "sha256": "11" * 32},
                    "lanes": {"path": "/tmp/lanes-run", "sha256": "22" * 32},
                    "distinct_binaries": True,
                }
            ),
            encoding="utf-8",
        )
        (out / "inputs.json").write_text(json.dumps({"wikidata": REF}), encoding="utf-8")
        (out / "runs.jsonl").write_text(
            "\n".join(json.dumps(r) for r in rows) + "\n", encoding="utf-8"
        )

    def registered_prov(self, **changes) -> dict:
        return {
            "strict_i9": True,
            "stub_pmu": False,
            "rounds": sc.ROUNDS,
            "settle_s": sc.SETTLE_S,
            "arms": list(sc.ARMS),
            "counts": list(sc.COUNTS),
            **changes,
        }

    def test_score_dir_roundtrip_and_unregistered_void(self):
        with tempfile.TemporaryDirectory() as td:
            out = Path(td)
            self._write_run_dir(out, self.registered_prov())
            res = sc.score_dir(out)
            self.assertEqual(res["verdict"], "CONFIRMED")
            self.assertTrue(
                (out / "outcome.txt").read_text(encoding="utf-8").startswith("CONFIRMED:")
            )

        for changes in (
            {"strict_i9": False},
            {"stub_pmu": True},
            {"rounds": 4},
            {"settle_s": "0"},
        ):
            with self.subTest(changes=changes), tempfile.TemporaryDirectory() as td:
                out = Path(td)
                self._write_run_dir(out, self.registered_prov(**changes))
                res = sc.score_dir(out)
                self.assertEqual(res["verdict"], "VOID")
                self.assertIn("not the registered run", res["reason"])

    def test_just_recipe_registration_and_defaults(self):
        self.assertGreaterEqual(sc.ROUNDS, 9)
        self.assertEqual(sc.SETTLE_S, "2")
        self.assertEqual(sc.ARMS, ("base", "lanes"))
        self.assertEqual(sc.COUNTS, (1, 4, 8))
        self.assertTrue(sc.REGISTRATION.exists())
        just_file = (sc.ROOT / ".just/scanner-contention.just").read_text(encoding="utf-8")
        self.assertIn("bench-scanner-contention", just_file)
        self.assertIn("bench/bundle.sh", just_file)
        self.assertIn("python3 bench/scanner_contention.py run", just_file)
        root_just = (sc.ROOT / "Justfile").read_text(encoding="utf-8")
        self.assertIn('import ".just/scanner-contention.just"', root_just)


if __name__ == "__main__":
    unittest.main()
