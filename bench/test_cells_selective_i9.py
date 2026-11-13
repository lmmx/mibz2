"""Untimed schedule, refusal, source, and real Just/bundle/submission checks."""

import collections
import copy
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from test_submit import Submit, git

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "cells_i9", ROOT / "bench/cells-selective-i9/workstation.py"
)
W = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(W)


def rows(out):
    return [
        dict(
            round=rnd,
            corpus=corpus,
            arm=arm,
            threads=1,
            cpus="0",
            termination="success",
            out=W.PINS[corpus][1],
            hash="same-" + corpus,
            wall=1.0,
            cpu=0.9,
            rss_kb=1000,
            sampler_s=0,
            sampler_cpus="none",
            sha256=None,
            machine_pre=dict(governor="powersave", epp="balance_performance", turbo=True),
            machine_post=dict(governor="powersave", epp="balance_performance", turbo=True),
        )
        for rnd, corpus, arm in W.schedule(out)
    ]


class Registration(unittest.TestCase):
    def test_balanced_schedule(self):
        scheduled = W.schedule(Path("/attempt"))
        self.assertEqual(len(scheduled), 252)
        for corpus in W.PINS:
            subset = [arm for _, name, arm in scheduled if name == corpus]
            for arm in W.arms(Path("/attempt")):
                self.assertEqual([subset[i::3].count(arm) for i in range(3)], [4, 4, 4])
        self.assertEqual(
            collections.Counter(r for r, _, _ in scheduled), dict.fromkeys(range(12), 21)
        )

    def test_cpu_binding_rejects_incomplete_and_shared_core(self):
        topology = {
            "0": dict(socket=0, core=0, siblings=[0, 10]),
            "10": dict(socket=0, core=0, siblings=[0, 10]),
            "1": dict(socket=0, core=1, siblings=[1]),
        }
        self.assertEqual(
            W.cpu_binding([0, 1, 10], topology),
            dict(cpu=0, siblings=[0, 10], parent=[1], allowed=[0, 1, 10]),
        )
        for allowed in ([], [0], [0, 10]):
            with self.assertRaises(ValueError):
                W.cpu_binding(allowed, topology)
        topology["10"]["core"] = 1
        with self.assertRaises(ValueError):
            W.cpu_binding([0, 1, 10], topology)

    def test_measurement_rows_are_not_filtered(self):
        out = Path("/attempt")
        good = rows(out)
        W.validate(good, out, 0)
        for key, value in (
            ("termination", "exit 17"),
            ("err", "CRC"),
            ("broken", 1),
            ("threads", 4),
            ("cpus", "1"),
            ("out", 2),
            ("hash", "wrong"),
            ("wall", float("nan")),
            ("cpu", 0),
            ("rss_kb", -1),
            ("sampler_s", 0.1),
            ("sha256", "active"),
            ("machine_post", {}),
        ):
            bad = copy.deepcopy(good)
            bad[0][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                W.validate(bad, out, 0)
        for bad in (good[:-1], good + good[:1], list(reversed(good))):
            with self.assertRaises(ValueError):
                W.validate(bad, out, 0)

    def test_existing_estimator(self):
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp)
            W.summarise(rows(out), out)
            data = json.loads((out / "summary.json").read_text())
            self.assertEqual(data["wikidata"]["ratios"]["selective/original"]["wall"], [1, 1, 1])
            self.assertEqual(data["text"]["absolute"]["original"]["max_rss_kb"], 1000)

    def test_build_overrides_refused(self):
        for key in (
            "RUSTFLAGS",
            "CARGO_ENCODED_RUSTFLAGS",
            "CARGO_PROFILE_RELEASE_LTO",
            "CARGO_TARGET_DIR",
            "RUSTC_WRAPPER",
        ):
            with (
                patch.dict(os.environ, {key: "unexpected"}, clear=True),
                self.assertRaises(ValueError),
            ):
                W.build_env(Path("/target"))

    def test_input_identity_refusal(self):
        with tempfile.TemporaryDirectory() as tmp:
            inputs = Path(tmp)
            (inputs / "wikidata.bz2").write_bytes(b"wrong")
            with self.assertRaises(ValueError):
                W.check_inputs(inputs)

    def test_exact_source_comparison(self):
        # Offline test of pins when the objects exist; full build check is separate.
        for arm, pin in (("original", W.ORIGINAL), ("selective", W.COMMON)):
            result = subprocess.run(
                ["git", "show", pin + ":src/cells.rs"], cwd=ROOT, capture_output=True
            )
            if result.returncode:
                self.skipTest("pinned source not fetched in shallow CI; native dry run checks it")
            self.assertEqual(W.hashlib.sha256(result.stdout).hexdigest(), W.CELLS_HASHES[arm])

    def test_preflight_preserves_collector_failures(self):
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp)

            def failed(*args, **kwargs):
                return subprocess.CompletedProcess(args[0], 17)

            with patch.object(W.subprocess, "run", failed), self.assertRaises(ValueError):
                W.preflight(out)
            record = (out / "preflight/exits.txt").read_text()
            self.assertIn("ab sample: exit 17", record)
            self.assertIn("static facts: exit 17", record)
            self.assertTrue(json.loads((out / "preflight/problems.json").read_text()))

    def test_prepare_has_no_measurement_call(self):
        # Fails if a later edit smuggles measured runs into the cloud dry-run path.
        source = (ROOT / "bench/cells-selective-i9/workstation.py").read_text()
        body = source[source.index("def prepare(") : source.index("def cpu_binding(")]
        self.assertNotIn('"run", str(out / "runs.jsonl")', body)
        self.assertNotIn("measure(", body)


class RecipeSubmission(Submit):
    """Real Just entry, wrapper, bounded supervisor, bundle and scratch submission."""

    def setUp(self):
        super().setUp()
        self.just = shutil.which("just")
        if self.just is None:
            self.fail("install just1.38.0: actual recipe/submission checks must not silently skip")
        for path in (
            ".just/submit.just",
            ".just/cells-selective-i9.just",
            "bench/bounded.py",
            "bench/bundle.sh",
            "bench/submit.sh",
            "bench/cells-selective-i9/run.sh",
        ):
            dest = self.work / path
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / path, dest)
        (self.work / "Justfile").write_text(
            'import ".just/submit.just"\nimport ".just/cells-selective-i9.just"\n'
        )
        # Only preparation/decoder work is a stand-in. The production recipe has
        # no skip-host, skip-checks or fake-measurements environment switch.
        (self.work / "bench/cells-selective-i9/workstation.py").write_text("""
import os, sys
from pathlib import Path
mode, path = sys.argv[1:]
out = Path(path)
with (out / "calls").open("a") as f: f.write(mode + "\\n")
(out / "commits.txt").write_text("head " + "a" * 40 + "\\n")
(out / "partial.jsonl").write_text("retained stand-in evidence\\n")
if os.environ.get("FAIL_STAGE") == mode: sys.exit(17)
if mode == "measure": (out / "outcome.txt").write_text("STAND-IN COMPLETE\\n")
""")
        self.env["PATH"] = str(self.tmp) + os.pathsep + os.environ["PATH"]
        self.env["GH"] = str(self.tmp / "gh")
        git(self.work, "add", ".")
        git(self.work, "commit", "-qm", "fixture")

    def exercise(self, failure=None):
        if failure:
            self.env["FAIL_STAGE"] = failure
        result = subprocess.run(
            [self.just, "submit", "bench-cells-selective-i9"],
            cwd=self.work,
            env=self.env,
            capture_output=True,
            text=True,
            timeout=60,
        )
        self.assertEqual(
            result.returncode, 0 if failure is None else 17, result.stdout + result.stderr
        )
        bundles = list((self.work / "target/cells-selective-i9").glob("*.tar.gz"))
        self.assertEqual(len(bundles), 1)
        self.assertEqual(len(self.comments()), 1)
        self.assertIn("INCOMPLETE" if failure else "STAND-IN COMPLETE", self.comments()[0]["body"])
        attempt = bundles[0].with_suffix("").with_suffix("")
        self.assertEqual(
            (attempt / "calls").read_text(),
            "prepare\n" if failure == "prepare" else "prepare\nmeasure\n",
        )
        self.assertTrue((attempt / "partial.jsonl").exists())
        # Resubmit the same archive without rerunning the job: no extra comment.
        self.assertEqual(self.submit(bundles[0]).returncode, 0)
        self.assertEqual(len(self.comments()), 1)

    def test_successful_recipe_submission(self):
        self.exercise()

    def test_preparation_failure_still_submitted(self):
        self.exercise("prepare")

    def test_measurement_failure_still_submitted(self):
        self.exercise("measure")


if __name__ == "__main__":
    unittest.main()
