"""Frozen AVX2 ring CRC rescue registration: the decision rule, scored on synthetic rows."""

import unittest

from crc32_ring_rescue import CELLS, ROUNDS, InvalidRun, canonical_arm, score

REFS = {name: {"out": 100, "hash": "h"} for name, _ in CELLS}


def rows(wall=None, rss=None, drop=None):
    """Paired rows; `wall`/`rss` map a cell to the head's ratio, with a small round jitter."""
    wall, rss, out = wall or {}, rss or {}, []
    for cell in CELLS:
        corpus, k = cell
        for r in range(ROUNDS):
            jitter = 1 + (0.002 if r % 2 else -0.002)
            for arm, label in (("base", "mibz2"), ("head", "mibz2@/x/head-ab")):
                if (cell, r, arm) == drop:
                    continue
                w = 2.0 * (wall.get(cell, 1.0) * jitter if arm == "head" else 1.0)
                m = 50_000 * (rss.get(cell, 1.0) if arm == "head" else 1.0)
                out.append(dict(arm=label, corpus=corpus, threads=k, round=r, out=100,
                                hash="h", wall=w, rss_kb=m))
    return out


class Rule(unittest.TestCase):
    def test_paid_gain_and_flat_guards_pass(self):
        result = score(rows(wall={("wikidata", 1): 0.97}), REFS, head_exe="/x/head-ab")
        self.assertEqual(result["verdict"], "PASS", result["findings"])

    def test_no_gain_at_one_worker_stops(self):
        result = score(rows(), REFS, head_exe="/x/head-ab")
        self.assertEqual(result["verdict"], "STOP")
        self.assertIn("no paid gain", result["findings"][0])

    def test_primary_loss_beyond_noise_stops(self):
        wall = {("wikidata", 1): 0.97, ("text", 8): 1.006}
        result = score(rows(wall=wall), REFS, head_exe="/x/head-ab")
        self.assertEqual(result["verdict"], "STOP")
        self.assertTrue(any("text/8" in f for f in result["findings"]))

    def test_secondary_loss_needs_both_point_and_noise(self):
        small = {("wikidata", 1): 0.97, ("random", 8): 1.015}
        self.assertEqual(score(rows(wall=small), REFS, head_exe="/x/head-ab")["verdict"], "PASS")
        big = {("wikidata", 1): 0.97, ("random", 8): 1.03}
        self.assertEqual(score(rows(wall=big), REFS, head_exe="/x/head-ab")["verdict"], "STOP")

    def test_calibrated_memory_bands(self):
        gain = {("wikidata", 1): 0.97}
        # k = 1: 1.006x of 50 MB is +300 KiB, over both limbs; 1.004x passes.
        for ratio, verdict in ((1.004, "PASS"), (1.006, "STOP")):
            got = score(rows(gain, {("pattern", 1): ratio}), REFS, head_exe="/x/head-ab")
            self.assertEqual(got["verdict"], verdict, (ratio, got["findings"]))
        # Multi-thread bands: 1.03 primary, 1.05 secondary (the predecessor's 1.005 is gone).
        for cell, ratio, verdict in (
            (("wikidata", 8), 1.025, "PASS"),
            (("text", 8), 1.035, "STOP"),
            (("pattern", 8), 1.045, "PASS"),
            (("records", 8), 1.055, "STOP"),
        ):
            got = score(rows(gain, {cell: ratio}), REFS, head_exe="/x/head-ab")
            self.assertEqual(got["verdict"], verdict, (cell, ratio, got["findings"]))

    def test_incomplete_or_foreign_grid_is_void(self):
        with self.assertRaises(InvalidRun):
            score(rows(drop=(CELLS[0], 0, "head")), REFS, head_exe="/x/head-ab")
        with self.assertRaises(InvalidRun):
            canonical_arm("mibz2@/elsewhere", head_exe="/x/head-ab")
        bad = rows(wall={("wikidata", 1): 0.97})
        bad[0]["hash"] = "other"
        with self.assertRaises(InvalidRun):
            score(bad, REFS, head_exe="/x/head-ab")


if __name__ == "__main__":
    unittest.main()
