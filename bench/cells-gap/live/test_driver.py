"""Untimed harness tests: no candidate performance observations."""

import json
import os
import shutil
from types import SimpleNamespace
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import driver


class DriverTests(unittest.TestCase):
    def test_fixed_balanced_budget(self):
        jobs = driver.schedule()
        self.assertEqual(len(jobs), 129)
        self.assertEqual(sum(j["mode"] == "check" for j in jobs), 9)
        for corpus in driver.CORPORA:
            measured = [j for j in jobs if j["corpus"] == corpus and j["mode"] == "measure"]
            self.assertEqual(len(measured), 40)
            self.assertEqual(sum(measured[i]["arm"] == "baseline" for i in range(0, 40, 2)), 10)
            self.assertEqual(sum(measured[i]["trace"] for i in range(0, 40, 4)), 5)
            self.assertEqual({j["settle_s"] for j in measured}, {0.25})

    def test_dry_run_has_no_child_clock_or_sleep(self):
        with (
            mock.patch.object(driver.subprocess, "run", side_effect=AssertionError("child")),
            mock.patch.object(driver.time, "perf_counter_ns", side_effect=AssertionError("clock")),
            mock.patch.object(driver.time, "sleep", side_effect=AssertionError("settle")),
        ):
            self.assertEqual(len(driver.schedule()), 129)

    def test_child_failure_preserved(self):
        with tempfile.TemporaryDirectory() as d:
            out, err = Path(d) / "out", Path(d) / "err"
            with self.assertRaises(subprocess.CalledProcessError):
                driver.checked_child(
                    [
                        sys.executable,
                        "-c",
                        "import sys; print('failed fixture',file=sys.stderr); sys.exit(7)",
                    ],
                    out,
                    err,
                )
            self.assertIn("failed fixture", err.read_text())
            with self.assertRaises(FileExistsError):
                driver.checked_child([sys.executable, "-c", "pass"], out, err)

    def test_malformed_and_decoder_error(self):
        for text in ["not JSON", json.dumps({"error": "CRC mismatch"})]:
            with tempfile.TemporaryDirectory() as d, self.assertRaises(ValueError):
                driver.checked_child(
                    [sys.executable, "-c", f"print({text!r})"], Path(d) / "out", Path(d) / "err"
                )

    def test_reconciliation_rejects_clock_and_crc(self):
        good = dict(
            trace=True,
            timed=False,
            clock_calls=0,
            phases={"whole_reader_drain": dict(calls=1, inclusive_ns=0, exclusive_ns=0)},
            blocks=[],
        )
        driver.reconcile(good)
        with self.assertRaises(ValueError):
            driver.reconcile(dict(good, clock_calls=1))
        with self.assertRaises(ValueError):
            driver.reconcile(dict(good, blocks=[dict(stored_crc=4, counters={"computed_crc": 3})]))
        with self.assertRaises(ValueError):
            driver.reconcile(
                dict(
                    good,
                    timed=True,
                    clock_calls=2,
                    phases={"whole_reader_drain": dict(calls=1, inclusive_ns=3, exclusive_ns=2)},
                )
            )

    def test_attribution_stops_if_instrumentation_changes_conclusion(self):
        rows = []
        for j in driver.schedule()[9:]:
            ratio = (0.8 if not j["trace"] else 1.2) if j["arm"] == "cells" else 1
            rows.append(
                dict(
                    j,
                    timed=True,
                    wall_ns=1000 * ratio,
                    clock_calls=2,
                    blocks=[],
                    phases={
                        "whole_reader_drain": dict(calls=1, inclusive_ns=100, exclusive_ns=100)
                    },
                )
                if j["trace"]
                else dict(j, timed=True, wall_ns=1000 * ratio)
            )
        result = driver.analyse(rows)
        self.assertTrue(all(not c["quantitative_attribution_valid"] for c in result.values()))
        with self.assertRaises(ValueError):
            driver.analyse(rows[:-1])
        with self.assertRaises(ValueError):
            driver.analyse(rows[:-1] + [rows[0]])

    def test_exact_run_wiring_without_decodes_or_clocks(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            keys = {a + "-" + t: "binary" for a in ("baseline", "cells") for t in ("off", "on")}
            (root / "build.json").write_text(
                json.dumps(
                    dict(
                        pin=driver.PIN,
                        diagnostic_head="a" * 40,
                        tracked_dirty=False,
                        rustc="rustc 1.99.1 (58cf8f9c3 2026-10-13)",
                        binaries=keys,
                        harness={},
                    )
                )
            )

            def identity(path):
                return driver.DIGESTS[path.stem] if path.suffix == ".bz2" else "binary"

            def child(command, output, error, env, measured=False):
                corpus = Path(command[5]).stem
                key = Path(command[3]).name
                trace = key.endswith("-on")
                row = dict(
                    arm=key.split("-")[0],
                    trace=trace,
                    timed=measured,
                    bytes=driver.SIZES[corpus],
                    sha256="oracle",
                    hash="checked",
                    error=None,
                )
                if trace:
                    ns = 100 if measured else 0
                    row.update(
                        clock_calls=2 if measured else 0,
                        blocks=[],
                        phases={
                            "whole_reader_drain": dict(calls=1, inclusive_ns=ns, exclusive_ns=ns)
                        },
                    )
                return (row, dict(wall_ns=1000, cpu_s=0.001, rss_kib=100)) if measured else row

            with (
                mock.patch.object(driver, "digest", side_effect=identity),
                mock.patch.object(driver.os, "sched_getaffinity", return_value={7}),
                mock.patch.object(
                    driver.subprocess, "check_output", return_value="fixture topology"
                ),
                mock.patch.object(driver, "checked_child", side_effect=child) as children,
                mock.patch.object(
                    driver,
                    "oracle",
                    side_effect=lambda path, output: dict(
                        bytes=driver.SIZES[path.stem], sha256="oracle"
                    ),
                ) as oracle,
                mock.patch.object(driver, "equal_files"),
                mock.patch.object(driver.time, "sleep") as settle,
                mock.patch.object(
                    driver.time, "perf_counter_ns", side_effect=AssertionError("real clock")
                ),
            ):
                driver.run(root, root, root / "attempt", 7)
            self.assertEqual(children.call_count, 126)
            self.assertEqual(oracle.call_count, 3)
            self.assertEqual(settle.call_count, 120)
            self.assertTrue((root / "attempt/COMPLETE").exists())
            self.assertEqual(len((root / "attempt/runs.jsonl").read_text().splitlines()), 120)

    def test_wait4_resources_and_failure_with_fake_clock(self):
        for status in (0, 7 << 8):
            with tempfile.TemporaryDirectory() as d:
                root = Path(d)
                child = mock.MagicMock(pid=123)
                child.__enter__.return_value = child

                def launch(command, stdout, **kwargs):
                    stdout.write('{"error":null}')
                    stdout.flush()
                    return child

                usage = SimpleNamespace(ru_utime=0.1, ru_stime=0.2, ru_maxrss=321)
                with (
                    mock.patch.object(driver.subprocess, "Popen", side_effect=launch),
                    mock.patch.object(driver.os, "wait4", return_value=(123, status, usage)),
                    mock.patch.object(driver.time, "perf_counter_ns", side_effect=[100, 250]),
                ):
                    if status:
                        with self.assertRaises(subprocess.CalledProcessError):
                            driver.checked_child(
                                ["fixture"], root / "out", root / "err", measured=True
                            )
                    else:
                        _, resources = driver.checked_child(
                            ["fixture"], root / "out", root / "err", measured=True
                        )
                        self.assertEqual(resources["wall_ns"], 150)
                        self.assertEqual(resources["rss_kib"], 321)
                        self.assertAlmostEqual(resources["cpu_s"], 0.3)

    def test_real_just_wiring_and_failure(self):
        just = shutil.which("just")
        if just is None:
            if os.environ.get("CELLS_LIVE_REQUIRE_JUST") == "1":
                self.fail("real Just required")
            self.skipTest("real Just unavailable")
        root = driver.HERE.parents[2]
        for code in (0, 19):
            with tempfile.TemporaryDirectory() as d:
                temp = Path(d)
                capture = temp / "args"
                stub = temp / "bash"
                stub.write_text(
                    '#!/bin/sh\nprintf "%s\\n" "$@" > "$CAPTURE"\nexit ' + str(code) + "\n"
                )
                stub.chmod(0o755)
                env = dict(
                    os.environ,
                    PATH=str(temp) + os.pathsep + os.environ["PATH"],
                    CAPTURE=str(capture),
                )
                result = subprocess.run(
                    [
                        just,
                        "--justfile",
                        str(root / "Justfile"),
                        "bench-cells-live-path",
                        "7",
                        "/frozen corpus",
                    ],
                    env=env,
                    capture_output=True,
                    text=True,
                )
                self.assertEqual(result.returncode == 0, code == 0, result.stderr)
                self.assertEqual(
                    capture.read_text().splitlines(),
                    ["bench/cells-gap/live/run.sh", "7", "/frozen corpus"],
                )

    def test_invalid_affinity_leaves_failed_attempt(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            with self.assertRaises(ValueError):
                driver.run(root, root, root / "attempt", -1)
            failure = json.loads((root / "attempt/FAILED.json").read_text())
            self.assertEqual(failure["completed_measured"], 0)


if __name__ == "__main__":
    unittest.main()
