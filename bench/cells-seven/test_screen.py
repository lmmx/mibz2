"""Small fixtures only; never consume or replay a pinned full corpus."""

import json
import bz2
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from prepare import WORK
from screen import check_blocks, controls_ok, read_memory


class Contracts(unittest.TestCase):
    def test_reader_fixtures_and_failed_child_memory(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            good = bz2.compress(b"abacaba" * 1000)
            corrupt = bytearray(good)
            corrupt[10] ^= 1
            fixtures = [
                ("empty", bz2.compress(b""), b""),
                (
                    "joined",
                    good + bz2.compress(b"") + bz2.compress(b"end"),
                    b"abacaba" * 1000 + b"end",
                ),
                ("truncated", good[:-4], None),
                ("crc", bytes(corrupt), None),
                ("missing", None, None),
            ]
            for name, data, expected in fixtures:
                source = root / (name + ".bz2")
                if data is not None:
                    source.write_bytes(data)
                parity = []
                for arm in ("baseline", "cells", "oracle"):
                    stem = root / (name + "-" + arm)
                    output = stem.with_suffix(".decoded")
                    waited_path = stem.with_suffix(".wait4.json")
                    result = subprocess.run(
                        [
                            str(WORK / "bin/launcher"),
                            str(waited_path),
                            str(WORK / "bin" / arm),
                            "check",
                            str(source),
                            str(output),
                        ],
                        capture_output=True,
                        text=True,
                        env={**os.environ, "AB_SHA256": "1"},
                        timeout=40,
                    )
                    stderr = stem.with_suffix(".stderr")
                    stderr.write_text(result.stderr)
                    waited = json.loads(waited_path.read_text())
                    read_memory(stderr, waited)
                    self.assertEqual(result.returncode, waited["exit"])
                    if expected is not None:
                        self.assertEqual(result.returncode, 0)
                        self.assertEqual(output.read_bytes(), expected)
                        row = json.loads(result.stdout)
                        if arm != "oracle":
                            check_blocks(row, arm == "cells")
                    else:
                        self.assertNotEqual(result.returncode, 0)
                    if arm != "oracle" and result.stdout:
                        row = json.loads(result.stdout)
                        parity.append((row["bytes"], row["hash"], row["error"]))
                if parity:
                    self.assertEqual(parity[0], parity[1])

    def test_controls_reject_flat_memory(self):
        sample = {"native": {"end": {"hwm_kib": 41168}}}
        with self.assertRaises(AssertionError):
            controls_ok(sample, sample)

    def test_wrong_pid_and_memory_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "stderr"
            memory = {
                "startup": {"pid": 7, "rss_kib": 100, "hwm_kib": 101},
                "end": {"pid": 7, "rss_kib": 100, "hwm_kib": 101},
            }
            path.write_text(json.dumps({"memory": memory}))
            waited = {"pid": 7, "waited_pid": 7, "wait4_hwm_kib": 101}
            self.assertEqual(read_memory(path, waited), memory)
            for changes in ({"pid": 8}, {"waited_pid": 8}, {"wait4_hwm_kib": 41168}):
                with self.assertRaises(AssertionError):
                    read_memory(path, waited | changes)

    def test_timing_and_crc_rejected(self):
        row = {
            "clock_calls": 0,
            "phases": {},
            "error": None,
            "blocks": [{"stored_crc": 2, "counters": {"computed_crc": 2}}],
        }
        check_blocks(row, False)
        row["blocks"][0]["stored_crc"] = 3
        with self.assertRaises(AssertionError):
            check_blocks(row, False)
        row["blocks"] = []
        row["clock_calls"] = 1
        with self.assertRaises(AssertionError):
            check_blocks(row, False)

    def test_native_launcher_failure_retention(self):
        with tempfile.TemporaryDirectory() as tmp:
            for index, (command, code) in enumerate(
                [
                    (["/bin/false"], 1),
                    (["/nonexistent-cells-seven-fixture"], 127),
                    (["/bin/sh", "-c", "kill -TERM $$"], 143),
                ]
            ):
                path = Path(tmp) / f"{index}.json"
                child = subprocess.run(
                    [str(WORK / "bin/launcher"), str(path), *command], check=False
                )
                record = json.loads(path.read_text())
                self.assertEqual(record["exit"], code)
                self.assertEqual(child.returncode, code)
                self.assertEqual(record["pid"], record["waited_pid"])
                self.assertFalse(record["timeout"])
                self.assertIn("wait4_hwm_kib", record)

    def test_native_launcher_timeout_retention(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "timeout.json"
            child = subprocess.run(
                [str(WORK / "bin/launcher"), str(path), "/bin/sleep", "60"], timeout=40, check=False
            )
            record = json.loads(path.read_text())
            self.assertTrue(record["timeout"])
            self.assertEqual(record["exit"], 137)
            self.assertEqual(child.returncode, 124)
            self.assertEqual(record["pid"], record["waited_pid"])


if __name__ == "__main__":
    unittest.main()
