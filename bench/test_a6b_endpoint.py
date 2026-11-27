"""Synthetic checks for the frozen A6b experiment."""

import unittest

from a6b_endpoint import (
    ByteMismatch,
    REPEATS,
    WALL_CELLS,
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
