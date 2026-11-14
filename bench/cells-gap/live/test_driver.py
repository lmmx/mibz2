"""Untimed harness tests: no candidate performance observations."""

import json
import os
import shutil
from types import SimpleNamespace
import subprocess
import sys
import tempfile
import tarfile
import unittest
from pathlib import Path
from unittest import mock

import driver
import workstation
import build as builder


class DriverTests(unittest.TestCase):
    def test_workstation_installs_only_missing_frozen_toolchain(self):
        for missing in (False, True):
            with mock.patch.object(workstation.subprocess, "run") as run:
                run.return_value.returncode = int(missing)
                workstation.ensure_toolchain()
                self.assertEqual(run.call_count, 3 if missing else 2)
                if missing:
                    self.assertEqual(
                        run.call_args_list[1].args[0],
                        ["rustup", "toolchain", "install", "1.99.1", "--profile", "minimal"],
                    )
                self.assertEqual(run.call_args.args[0], ["rustup", "run", "1.99.1", "rustc", "-Vv"])
                self.assertTrue(run.call_args.kwargs["check"])

    def test_workstation_cpu_selection_is_noninteractive_and_allowed(self):
        with (
            mock.patch.object(workstation.os, "sched_getaffinity", return_value={9, 7}),
            mock.patch.object(workstation.Path, "read_text", return_value="fixture"),
            mock.patch.object(workstation.subprocess, "check_output", return_value="fixture"),
        ):
            record = workstation.machine()
        self.assertEqual(record["selected_cpu"], 7)
        self.assertEqual(record["allowed_cpus"], [7, 9])
        self.assertIn("thread_siblings_list", record["topology"])
        with mock.patch.object(workstation.os, "sched_getaffinity", return_value=set()):
            with self.assertRaisesRegex(ValueError, "no allowed"):
                workstation.machine()

    def test_workstation_staging_validates_copies_and_never_overwrites_attempt(self):
        import hashlib

        fixture = {name: name.encode() for name in driver.DIGESTS}
        identities = {name: hashlib.sha256(data).hexdigest() for name, data in fixture.items()}

        def generate(path):
            path.mkdir()
            for name in ("pattern", "onebyte"):
                (path / f"{name}.bz2").write_bytes(fixture[name])

        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / "corpora").mkdir()
            canonical = root / "corpora/wikidata.bz2"
            canonical.write_bytes(fixture["wikidata"])
            with (
                mock.patch.object(workstation, "DIGESTS", identities),
                mock.patch.object(workstation, "generate", side_effect=generate),
                mock.patch.object(
                    workstation, "ensure_object", side_effect=AssertionError("cache")
                ),
            ):
                result = workstation.stage_inputs(root / "staged", root)
                self.assertEqual(result["sha256"], identities)
                self.assertEqual(
                    (root / "staged/wikidata.bz2").read_bytes(), canonical.read_bytes()
                )
                with self.assertRaises(FileExistsError):
                    workstation.stage_inputs(root / "staged", root)
                fixture["pattern"] = b"corrupt"
                with self.assertRaisesRegex(ValueError, "staged corpus"):
                    workstation.stage_inputs(root / "bad", root)

    def test_workstation_preflight_retained_on_staging_failure(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            with (
                mock.patch.object(workstation, "machine", return_value={"selected_cpu": 7}),
                mock.patch.object(workstation, "stage_inputs", side_effect=ValueError("input")),
                self.assertRaisesRegex(ValueError, "input"),
            ):
                workstation.prepare(root)
            self.assertEqual(json.loads((root / "preflight.json").read_text())["selected_cpu"], 7)
            self.assertFalse((root / "cpu").exists())

    def test_submit_discovers_success_and_failure_bundles(self):
        just = shutil.which("just")
        if just is None:
            if os.environ.get("CELLS_LIVE_REQUIRE_JUST") == "1":
                self.fail("real Just required")
            self.skipTest("real Just unavailable")
        source = driver.HERE.parents[2]
        for queued, prep_exit, child_exit in (
            (False, 0, 0),
            (False, 0, 19),
            (True, 0, 0),
            (True, 0, 19),
            (True, 23, 0),
        ):
            with (
                self.subTest(queued=queued, prep_exit=prep_exit, child_exit=child_exit),
                tempfile.TemporaryDirectory() as d,
            ):
                root = Path(d)
                subprocess.run(["git", "init", "-q", str(root)], check=True)
                shutil.copy2(source / "Justfile", root / "Justfile")
                shutil.copytree(source / ".just", root / ".just")
                live = root / "bench/cells-gap/live"
                (live / ".source").mkdir(parents=True)
                shutil.copy2(driver.HERE / "run.sh", live / "run.sh")
                shutil.copy2(source / "bench/bundle.sh", root / "bench/bundle.sh")
                # Submission is intercepted; no network, decoder, build or clocks.
                (root / "bench/submit.sh").write_text(
                    '#!/bin/sh\nprintf "%s\\n" "$1" > submitted\n'
                )
                tools = root / "tools"
                tools.mkdir()
                (tools / "gh").write_text("#!/bin/sh\nexit 0\n")
                (tools / "python3").write_text(
                    f"#!{sys.executable}\n"
                    "import os, pathlib, sys\n"
                    "if sys.argv[1].endswith('build.py'):\n"
                    "    pathlib.Path(sys.argv[2]).mkdir()\n"
                    "elif sys.argv[1].endswith('driver.py'):\n"
                    "    path = pathlib.Path(sys.argv[5])\n"
                    "    path.mkdir()\n"
                    "    (path / 'stand-in.txt').write_text('no decoder executed')\n"
                    "    sys.exit(int(os.environ['CHILD_EXIT']))\n"
                    "elif sys.argv[1].endswith('workstation.py'):\n"
                    "    assert os.environ['RUSTUP_TOOLCHAIN'] == '1.99.1'\n"
                    "    path = pathlib.Path(sys.argv[2])\n"
                    "    (path / 'preflight.json').write_text('retained preparation')\n"
                    "    if int(os.environ['PREP_EXIT']):\n"
                    "        sys.exit(int(os.environ['PREP_EXIT']))\n"
                    "    (path / 'inputs').mkdir()\n"
                    "    (path / 'cpu').write_text('7')\n"
                    "else:\n"
                    "    raise AssertionError(sys.argv)\n"
                )
                for tool in tools.iterdir():
                    tool.chmod(0o755)
                corpus = root / "corpus"
                corpus.mkdir()
                result = subprocess.run(
                    [just, "submit", "bench-cells-live-path-i9"]
                    if queued
                    else [just, "submit", "bench-cells-live-path", "7", str(corpus)],
                    cwd=root,
                    env=dict(
                        os.environ,
                        PATH=str(tools) + os.pathsep + os.environ["PATH"],
                        CHILD_EXIT=str(child_exit),
                        PREP_EXIT=str(prep_exit),
                    ),
                    capture_output=True,
                    text=True,
                )
                expected_exit = prep_exit or child_exit
                self.assertEqual(result.returncode == 0, expected_exit == 0, result.stderr)
                bundles = list(root.glob("target/*/*.tar.gz"))
                self.assertEqual(len(bundles), 1, result.stdout + result.stderr)
                self.assertEqual(
                    (root / "submitted").read_text().strip(), str(bundles[0].relative_to(root))
                )
                with tarfile.open(bundles[0]) as bundle:
                    stamp = bundles[0].name.removesuffix(".tar.gz")
                    self.assertEqual(
                        bundle.extractfile(stamp + "/exit").read().strip(),
                        str(expected_exit).encode(),
                    )
                    if prep_exit:
                        self.assertNotIn(stamp + "/attempt/stand-in.txt", bundle.getnames())
                        self.assertIn(stamp + "/preflight.json", bundle.getnames())
                    else:
                        self.assertIn(stamp + "/attempt/stand-in.txt", bundle.getnames())

    def test_missing_build_parent_and_no_overwrite(self):
        with (
            tempfile.TemporaryDirectory() as d,
            mock.patch.dict(os.environ, {"RUSTFLAGS": "non-frozen"}),
        ):
            target = Path(d) / "missing-parent" / "binaries"
            with self.assertRaisesRegex(ValueError, "non-frozen"):
                builder.build(target)
            self.assertTrue(target.is_dir())
            with self.assertRaises(FileExistsError):
                builder.build(target)

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

            def child(command, output, error, env, measured=False, job=None):
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
        cases = [
            (0, '{"error":null}', "accepted", None),
            (7 << 8, '{"error":"CRC"}', "nonzero_exit", subprocess.CalledProcessError),
            (0, "not JSON", "malformed_json", ValueError),
            (0, '{"error":"CRC"}', "decoder_error", ValueError),
        ]
        resources = dict(wall_ns=150, cpu_s=0.1 + 0.2, user_s=0.1, system_s=0.2, rss_kib=321)
        job = dict(index=9, **driver.schedule()[9])
        for status, body, validation, exception in cases:
            with self.subTest(validation=validation), tempfile.TemporaryDirectory() as d:
                root = Path(d)
                child = mock.MagicMock(pid=123)
                child.__enter__.return_value = child

                def exited(*args):
                    # Already durable at Popen context exit, before JSON parsing
                    # or the outer exception handler can rescue the record.
                    saved = json.loads((root / "out.resources").read_text())
                    self.assertEqual(saved["resources"], resources)
                    self.assertEqual(saved["wait_status"], status)
                    self.assertEqual(saved["validation"], "pending")
                    return False

                child.__exit__.side_effect = exited

                def launch(command, stdout, stderr, **kwargs):
                    stdout.write(body)
                    stdout.flush()
                    stderr.write("failure retained")
                    stderr.flush()
                    return child

                usage = SimpleNamespace(ru_utime=0.1, ru_stime=0.2, ru_maxrss=321)
                with (
                    mock.patch.object(driver.subprocess, "Popen", side_effect=launch),
                    mock.patch.object(driver.os, "wait4", return_value=(123, status, usage)),
                    mock.patch.object(driver.time, "perf_counter_ns", side_effect=[100, 250]),
                ):
                    if exception:
                        with self.assertRaises(exception):
                            driver.checked_child(
                                ["fixture"], root / "out.json", root / "err", measured=True, job=job
                            )
                    else:
                        row, returned = driver.checked_child(
                            ["fixture"], root / "out.json", root / "err", measured=True, job=job
                        )
                        self.assertEqual(row, {"error": None})
                        self.assertEqual(returned, resources)
                record = json.loads((root / "out.resources").read_text())
                self.assertEqual(record["resources"], resources)
                self.assertEqual(record["command"], ["fixture"])
                self.assertEqual(record["job"], job)
                self.assertEqual(record["pid"], 123)
                self.assertEqual(record["status"], "exited")
                self.assertEqual(record["wait_status"], status)
                self.assertEqual(record["returncode"], os.waitstatus_to_exitcode(status))
                self.assertEqual(record["validation"], validation)
                self.assertEqual("error" in record, exception is not None)
                self.assertEqual((root / "out.json").read_text(), body)
                self.assertEqual((root / "err").read_text(), "failure retained")

    def test_spawn_failure_has_no_child_resources(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            job = dict(index=9, **driver.schedule()[9])
            with (
                mock.patch.object(
                    driver.subprocess, "Popen", side_effect=FileNotFoundError("fixture")
                ),
                mock.patch.object(driver.os, "wait4", side_effect=AssertionError("no child")),
                mock.patch.object(driver.time, "perf_counter_ns", side_effect=[100]),
                self.assertRaises(FileNotFoundError),
            ):
                driver.checked_child(
                    ["fixture"], root / "out.json", root / "err", measured=True, job=job
                )
            record = json.loads((root / "out.resources").read_text())
            self.assertEqual(record["command"], ["fixture"])
            self.assertEqual(record["job"], job)
            self.assertEqual(record["status"], "spawn_failed")
            self.assertEqual(record["validation"], "pending")
            for field in ("pid", "wait_status", "returncode", "resources"):
                self.assertIsNone(record[field])
            self.assertIn("FileNotFoundError", record["error"])
            self.assertEqual((root / "out.json").read_text(), "")
            self.assertEqual((root / "err").read_text(), "")

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

    def test_failed_measured_child_never_enters_successful_rows(self):
        original_child = driver.checked_child
        for case in ("nonzero", "malformed", "decoder_error", "spawn", "reconcile"):
            with self.subTest(case=case), tempfile.TemporaryDirectory() as d:
                root = Path(d)
                (root / "build.json").write_text(
                    json.dumps(
                        dict(
                            pin=driver.PIN,
                            diagnostic_head="a" * 40,
                            tracked_dirty=False,
                            rustc="rustc 1.99.1 (58cf8f9c3 2026-10-13)",
                            binaries={
                                a + "-" + t: "binary"
                                for a in ("baseline", "cells")
                                for t in ("off", "on")
                            },
                            harness={},
                        )
                    )
                )
                jobs = [driver.schedule()[0], driver.schedule()[11]]
                good = dict(
                    arm="baseline",
                    trace=True,
                    timed=False,
                    bytes=1,
                    hash="ok",
                    clock_calls=0,
                    blocks=[],
                    phases={},
                )
                measured_row = dict(
                    good,
                    timed=True,
                    clock_calls=2,
                    phases={"whole_reader_drain": dict(calls=1, inclusive_ns=100, exclusive_ns=99)},
                )
                body = {"malformed": "not JSON", "decoder_error": '{"error":"CRC"}'}.get(
                    case, json.dumps(measured_row)
                )
                child = mock.MagicMock(pid=123)
                child.__enter__.return_value = child

                def launch(command, stdout, stderr, **kwargs):
                    if case == "spawn":
                        raise FileNotFoundError("fixture")
                    stdout.write(body)
                    stdout.flush()
                    stderr.write("retained")
                    stderr.flush()
                    return child

                def checked(command, output, error, env, measured=False, job=None):
                    if not measured:
                        return good.copy()
                    return original_child(command, output, error, env, measured=True, job=job)

                with (
                    mock.patch.object(
                        driver,
                        "digest",
                        side_effect=lambda p: (
                            driver.DIGESTS[p.stem] if p.suffix == ".bz2" else "binary"
                        ),
                    ),
                    mock.patch.object(driver, "schedule", return_value=jobs),
                    mock.patch.object(driver.os, "sched_getaffinity", return_value={7}),
                    mock.patch.object(driver.subprocess, "check_output", return_value="fixture"),
                    mock.patch.object(driver, "checked_child", side_effect=checked),
                    mock.patch.object(driver.subprocess, "Popen", side_effect=launch),
                    mock.patch.object(
                        driver.os,
                        "wait4",
                        return_value=(
                            123,
                            7 << 8 if case == "nonzero" else 0,
                            SimpleNamespace(ru_utime=0.1, ru_stime=0.2, ru_maxrss=321),
                        ),
                    ),
                    mock.patch.object(driver.time, "perf_counter_ns", side_effect=[100, 250]),
                    mock.patch.object(driver.time, "sleep"),
                    self.assertRaises(
                        (ValueError, subprocess.CalledProcessError, FileNotFoundError)
                    ),
                ):
                    driver.run(root, root, root / "attempt", 7)
                attempt = root / "attempt"
                self.assertEqual(
                    json.loads((attempt / "FAILED.json").read_text())["completed_measured"], 0
                )
                for name in ("runs.jsonl", "analysis.json", "COMPLETE"):
                    self.assertFalse((attempt / name).exists())
                record = json.loads((attempt / "001-wikidata-baseline.resources").read_text())
                self.assertEqual(record["job"], dict(index=1, **jobs[1]))
                self.assertEqual(
                    record["resources"]["wall_ns"] if case != "spawn" else record["resources"],
                    150 if case != "spawn" else None,
                )
                self.assertTrue((attempt / "001-wikidata-baseline.json").exists())
                self.assertTrue((attempt / "001-wikidata-baseline.stderr").exists())

    def test_invalid_affinity_leaves_failed_attempt(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            with self.assertRaises(ValueError):
                driver.run(root, root, root / "attempt", -1)
            failure = json.loads((root / "attempt/FAILED.json").read_text())
            self.assertEqual(failure["completed_measured"], 0)


if __name__ == "__main__":
    unittest.main()
