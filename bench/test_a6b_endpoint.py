"""Synthetic checks for the frozen A6b experiment."""

from pathlib import Path
import subprocess
import tempfile
import unittest

from a6b_endpoint import (
    ByteMismatch,
    REPEATS,
    WALL_CELLS,
    checkout_candidate,
    paired,
    summarise,
    validate_pair,
    validate_trace,
)


def observations():
    rows = []
    for block in range(REPEATS):
        for arm in ("base", "candidate"):
            for corpus, k in WALL_CELLS:
                wall = 1.3 if arm == "base" or (corpus, k) != ("wikidata", 1) else 1.25
                rows.append(
                    dict(
                        measure="wall",
                        build=arm,
                        section=corpus,
                        k=k,
                        block=block,
                        wall_s=wall,
                        maxrss_kib=1000,
                    )
                )
            for oracle in ("none", "S"):
                trace = dict(
                    worker_busy_ns_per_w=200e6 if arm == "base" else 191e6,
                    scanner_busy_ns=1000 if oracle == "none" else 15,
                    worker_cpu_ns_per_w=160e6,
                )
                rows.append(
                    dict(
                        measure=oracle,
                        build=arm,
                        section="wikidata",
                        k=8,
                        block=block,
                        wall_s=0.2,
                        trace_summary=trace,
                    )
                )
    return rows


class EndpointRecipe(unittest.TestCase):
    def test_source_checkout_from_detached_shallow_recipe(self):
        with tempfile.TemporaryDirectory() as scratch:
            folder = Path(scratch)
            upstream = folder / "upstream"
            recipe = folder / "runner"
            source = folder / "build"

            def git(repo, *args):
                return subprocess.check_output(["git", "-C", repo, *args], text=True).strip()

            subprocess.run(["git", "init", "-q", "-b", "recipe", upstream], check=True)
            git(upstream, "config", "user.name", "Test")
            git(upstream, "config", "user.email", "test@example.invalid")
            git(upstream, "commit", "-q", "--allow-empty", "-m", "base")
            git(upstream, "switch", "-qc", "candidate")
            (upstream / "decoder").write_text("registered source")
            git(upstream, "add", "decoder")
            git(upstream, "commit", "-qm", "candidate")
            pinned = git(upstream, "rev-parse", "HEAD")
            git(upstream, "switch", "-q", "recipe")
            git(upstream, "commit", "-q", "--allow-empty", "-m", "recipe")
            subprocess.run(
                [
                    "git",
                    "clone",
                    "-q",
                    "--depth=1",
                    "--single-branch",
                    "-b",
                    "recipe",
                    upstream.as_uri(),
                    recipe,
                ],
                check=True,
            )
            recipe_head = git(recipe, "rev-parse", "HEAD")
            git(recipe, "checkout", "--quiet", "--detach", recipe_head)
            self.assertEqual(git(recipe, "rev-parse", "--is-shallow-repository"), "true")
            self.assertNotEqual(
                subprocess.run(
                    ["git", "-C", recipe, "cat-file", "-e", pinned], capture_output=True
                ).returncode,
                0,
            )
            checkout_candidate(recipe, source, pinned)
            self.assertEqual(git(source, "rev-parse", "HEAD"), pinned)
            self.assertEqual((source / "decoder").read_text(), "registered source")
            self.assertEqual(git(recipe, "rev-parse", "HEAD"), recipe_head)

    def test_frozen_thresholds(self):
        rows = observations()
        self.assertTrue(summarise(rows)["hardware_pass"])
        for r in rows:
            if r["measure"] == "none" and r["build"] == "candidate":
                r["trace_summary"]["worker_cpu_ns_per_w"] = 1e6
                r["trace_summary"]["worker_busy_ns_per_w"] = 198e6
            if (r["measure"], r["k"], r["build"]) == ("wall", 10, "candidate"):
                r["wall_s"] *= 1.011
        checks = summarise(rows)["checks"]
        self.assertFalse(checks["worker_busy"])
        self.assertFalse(checks["wikidata/10"])
        self.assertTrue(checks["wikidata/8"])

    def test_missing_or_duplicate_round(self):
        rows = [
            r for r in observations() if (r["measure"], r["section"], r["k"]) == ("wall", "text", 1)
        ]
        with self.assertRaises(KeyError):
            paired(rows[:-1], lambda r: r["wall_s"])
        with self.assertRaises(ValueError):
            paired(rows + rows[:1], lambda r: r["wall_s"])

    def test_missing_activation_and_wrong_hash(self):
        r = dict(
            kind="decode",
            scanner_pin_ok=True,
            k=8,
            section="wikidata",
            bytes=30,
            maxrss_kib=1,
            check_pass=dict(identical=True, sha256="ok", bytes=30),
            trace_summary=dict(
                workers_seen=8, blocks_decoded=5, blocks_open_pass1=3, worker_busy_ns_per_w=1
            ),
            oracles=dict(scanned=0, replayed=5, scan_locks=0, recorded_blocks=5),
        )
        r["trace_summary"]["blocks_issued"] = 5
        validate_trace(r, "S")
        r["trace_summary"]["blocks_issued"] = 99
        with self.assertRaisesRegex(ValueError, "incomplete"):
            validate_trace(r, "S")
        r["trace_summary"]["blocks_issued"] = 5
        r["trace_summary"]["blocks_open_pass1"] = 0
        with self.assertRaisesRegex(ValueError, "Open"):
            validate_trace(r, "S")

    def test_classify_candidate_and_control_byte_failures(self):
        good = dict(k=8, maxrss_kib=20, measure="wall", sha256="reference")
        wrong = {**good, "sha256": "wrong"}
        error = {"kind": "failed", "stderr": "block CRC mismatch (stored 0, computed 1)"}
        for invalid in (wrong, error):
            with self.assertRaises(ByteMismatch):
                validate_pair(
                    {"base": good, "candidate": invalid}, "text", 8, 0, "wall", "reference"
                )
            with self.assertRaisesRegex(ValueError, "control failed") as exc:
                validate_pair(
                    {"base": invalid, "candidate": wrong}, "text", 8, 0, "wall", "reference"
                )
            self.assertNotIsInstance(exc.exception, ByteMismatch)
