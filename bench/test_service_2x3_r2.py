"""The service core's revision-2 registration (docs/journal/2026-11-09-service-2x3-r2-registration.md):
the machine state recorded beside every run changes nothing the registered report reads, and the
recipe keeps the shape the registration gives it. Synthetic records, no decoder run.

    python3 bench/test_service_2x3_r2.py
"""

import copy
import re
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import service_2x3_diag as diag  # noqa: E402
import test_service_2x3 as t  # noqa: E402

ROOT = HERE.parent
RECIPE = "bench-service-2x3-r2"


def telemetry(slow_rounds=(), missing_rounds=()):
    """Every run's machine-state fields, as `ab run --machine-state` (bench 3.4.0) writes them:
    a halved clock in `slow_rounds`, every reading unavailable (null, with its reason) in
    `missing_rounds`"""

    def add(rows, trace, files):
        for r in rows:
            ghz = 2.45 if r["round"] in slow_rounds else 4.9
            st = {
                "freq_mhz": 1000 * ghz, "freq_cpus": 20, "governor": "powersave", "epp": "balance_performance",
                "turbo": True, "pkg_temp_c": 70.0, "temp_src": "thermal_zone1 (x86_pkg_temp)",
                "t_unix_ns": 1_794_200_000_000_000_000 + r["round"], "read_s": 0.0001,
                "cpu_khz": {str(c): int(ghz * 1e6) for c in range(20)},
                "policies": [{"policy": f"policy{c}", "cpus": str(c), "driver": "intel_pstate",
                              "governor": "powersave", "epp": "balance_performance"} for c in range(20)],
                "platform_profile": None, "platform_profile_why": "no sys/firmware/acpi/platform_profile",
            }
            if r["round"] in missing_rounds:
                why = "no cpufreq/scaling_cur_freq for any of the 20 CPUs"
                r.update(
                    cycles=None, cycles_why=why, instructions=None, instructions_why=why,
                    ref_cycles=None, ref_cycles_why=why, task_clock_s=None, task_clock_s_why=why,
                    perf_running=None, eff_ghz=None, eff_ghz_why="no cycles: " + why,
                    machine_pre={"freq_mhz": None, "freq_mhz_why": why},
                    machine_post={"freq_mhz": None, "freq_mhz_why": why},
                    cpus_allowed=None, cpus_allowed_why="no Cpus_allowed_list",
                )
                continue
            r.update(
                cycles=round(r["cpu"] * ghz * 1e9), instructions=round(r["cpu"] * 9e9),
                ref_cycles=round(r["cpu"] * 3.7e9), task_clock_s=r["cpu"], perf_scope="user+kernel",
                perf_times_ns={"cycles": [1, 1], "instructions": [1, 1], "ref_cycles": [1, 1], "task_clock": [1, 1]},
                perf_coverage="this process and every thread it starts after the counters open",
                perf_running=1.0, eff_ghz=ghz, machine_pre=st, machine_post=dict(st, pkg_temp_c=75.0),
                t_spawn_unix_ns=1, t_exit_unix_ns=2, cpus_allowed="0-19", ctxsw_voluntary=400,
                ctxsw_involuntary=20, cpus_seen="0,1,2,3,4,5,6,7,8,9",
            )

    return add


def comparable(d):
    """The report's result, everything it judges and prints"""
    d = copy.deepcopy(d)
    d.pop("md")
    return d


class TelemetryIsNeverRead(unittest.TestCase):
    """Every record enters the registered analysis; the telemetry changes nothing it reads"""

    def test_same_result_with_telemetry(self):
        plain = t.run()
        logged = t.run(telemetry())
        self.assertEqual(plain["verdict"], "VALID", t.failed(plain))
        self.assertEqual(comparable(plain), comparable(logged))
        self.assertEqual(plain["md"], logged["md"])

    def test_slowed_rounds_are_neither_flagged_nor_dropped(self):
        # (a halved clock in a third of the rounds: the report reads the same records, and
        # says nothing of the clock)
        plain = t.run()
        logged = t.run(telemetry(slow_rounds={0, 4, 8, 12, 16, 18}))
        self.assertEqual(comparable(plain), comparable(logged))
        self.assertNotIn("ghz", logged["md"].lower())
        self.assertNotIn("clock", logged["md"].lower())

    def test_missing_telemetry_is_not_a_validity_check(self):
        plain = t.run()
        logged = t.run(telemetry(missing_rounds=set(range(20))))
        self.assertEqual(comparable(plain), comparable(logged))

    def test_telemetry_does_not_rescue_an_invalid_run(self):
        def both(rows, trace, files):
            telemetry()(rows, trace, files)
            t.drop("hash")(rows, trace, files)

        self.assertEqual(t.run(both)["verdict"], "INVALID")


def recipe() -> str:
    """The recipe's body, from .just/bench.just"""
    text = (ROOT / ".just" / "bench.just").read_text()
    m = re.search(rf"^{re.escape(RECIPE)} .*?:\n((?:    .*\n|\n)+)", text, re.M)
    assert m, f"no {RECIPE} in .just/bench.just"
    return m.group(1)


class RecipeShape(unittest.TestCase):
    """The workstation-recipe shape (CLAUDE.md) and the registration's limits on the recipe"""

    def test_one_new_directory_and_one_bundle(self):
        body = recipe()
        self.assertIn('mkdir "$W" ||', body)
        self.assertNotIn('mkdir -p "$W"', body)
        self.assertIn("W=$B/$(date -u +%Y%m%dT%H%M%SZ)", body)
        self.assertIn('exec > >(tee -a "$W/console.log") 2>&1', body)
        self.assertIn('bash bench/bundle.sh "$W"', body)
        self.assertIn("trap finish EXIT", body)

    def test_registered_cells_rounds_and_report(self):
        body = recipe()
        self.assertIn(
            'arms="mibz2,mibz2-svc2,mibz2-team2x3,mibz2-pipe2x3,mibz2-team3,mibz2-team3-twin"', body
        )
        self.assertIn('rounds={{ROUNDS}}; trace_rounds=6', body)
        self.assertIn('--threads 10 --rounds "$rounds" --machine-state', body)
        self.assertIn("python3 bench/service_2x3.py report", body)
        self.assertIn("bench/service-2x3-r2.frozen", body)

    def test_no_settling_retry_or_setting_change(self):
        body = recipe()
        # (no `ab settle`, no loop that waits on the machine's state, no retry, no writes to
        # sysfs or procfs, no privilege)
        self.assertNotIn("settle", re.sub(r"--settle 0\.2|no settling", "", body))
        self.assertNotRegex(body, r"\bwhile\b|\buntil\b|\bretry\b")
        self.assertNotRegex(body, r">\s*/(sys|proc)/|\btee\b[^\n]*/(sys|proc)/")
        self.assertNotRegex(body, r"\bsudo\b|\bcpupower\b|\bsysctl\b|\bpowerprofilesctl\b")
        self.assertIn("--count 10 --interval 1", body)

    def test_preflight_refuses_before_any_timed_run(self):
        body = recipe()
        first_run = body.index('"$AB" run ')
        for n in ("F1", "F2", "F3", "F4"):
            i = body.index(f'refuse "{n}')
            self.assertLess(i, first_run, n)
        # after the first timed run, nothing refuses
        self.assertNotIn("refuse ", body[first_run:])
        self.assertIn('outcome="REFUSED: PREFLIGHT: $*"', body)
        self.assertIn('"$AB" telemetry-check > "$M/telemetry-check.jsonl"', body)

    def test_diagnostics_never_feed_the_report(self):
        body = recipe()
        report = body.index("python3 bench/service_2x3.py report")
        self.assertGreater(body.index("python3 bench/service_2x3_diag.py"), report)
        self.assertIn("bench/service_2x3_diag.py", body[: body.index("frozen=$(")] + body)


def records(slow=()):
    """The fixture's records with telemetry, a halved clock (cycles below reference cycles)
    for the (round, arm) pairs in `slow`, in the order ab would run them"""
    rows, trace, files = t.fixture()
    telemetry()(rows, trace, files)
    for i, r in enumerate(rows):
        r["t_spawn_unix_ns"] = 1_000_000_000 * i
        r["t_exit_unix_ns"] = 1_000_000_000 * i + 200_000_000
        if (r["round"], r["arm"]) in slow:
            r["cycles"] = r["ref_cycles"] // 2
    return rows


class Diagnostics(unittest.TestCase):
    def test_run_order_position_predecessor_gap(self):
        chron = diag.chronological(records())
        self.assertEqual([c["position"] for c in chron[:7]], [0, 1, 2, 3, 4, 5, 0])
        self.assertIsNone(chron[0]["predecessor"])
        self.assertEqual(chron[1]["predecessor"], chron[0]["cell"])
        self.assertAlmostEqual(chron[1]["gap_s"], 0.8)

    def test_pair_states_are_counted_never_averaged(self):
        rs = records(slow={(0, "mibz2-svc2"), (1, "mibz2"), (2, "mibz2-svc2"), (2, "mibz2")})
        p1 = diag.pairs(rs)["P1"]
        self.assertEqual([x["pair"] for x in p1["rounds"][:4]], ["LH", "HL", "LL", "HH"])
        self.assertEqual(p1["counts"], {"HH": 17, "HL": 1, "LH": 1, "LL": 1, "unknown": 0})
        md = diag.markdown(diag.chronological(rs), diag.pairs(rs))
        self.assertIn("counts only", md)
        tables = md.split("## In run order", 1)[1].lower()
        self.assertNotRegex(tables, r"geometric|interval|\bmean\b")

    def test_unknown_is_never_imputed(self):
        rs = records()
        rs[0]["ref_cycles"] = None
        rs[1]["perf_multiplexed"] = ["cycles"]
        rs[2]["perf_ctl_errors"] = [{"counter": "ref_cycles", "op": "enable", "errno": 25}]
        self.assertEqual([diag.clock_state(r)[0] for r in rs[:4]], ["unknown", "unknown", "unknown", "H"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
