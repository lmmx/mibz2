"""The stage table across builds whose phases report different stages (bench 2.1.0 added
the segment assembly, within the walk's row): the figures and their comparison read only
what both builds have, a missing stage shows as unavailable, never as zero, and a row
within another is never counted twice.

    python3 bench/test_report.py
"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import report  # noqa: E402

MHZ = 3691.0
# picoseconds a byte, as bench/stages.py records them; the total is phases' own (the walk
# counts its assembly), never a sum of the rows
WITH = {
    "scan": 1000,
    "mtf": 2000,
    "build": 1500,
    "walk": 4000,
    "assemble": 300,
    "rle": 200,
    "crc": 250,
    "total": 8950,
}
WITHOUT = {k: v for k, v in WITH.items() if k != "assemble"}


def runs(base: dict, head: dict, rounds: int = 3) -> list[dict]:
    return [
        {
            "round": r,
            "arm": arm,
            "repeats": 2,
            "wall": 1.0,
            "ps_per_byte": ps,
            "walk": "walk, room (used)",
        }
        for r in range(rounds)
        for arm, ps in (("base", base), ("head", head))
    ]


def row(lines: list[str], title: str) -> list[str]:
    """A stage table row's cells, by its title."""
    for line in lines:
        cells = [c.strip() for c in line.strip("|").split("|")]
        if cells and cells[0].strip("*") == title:
            return cells
    raise AssertionError(f"no row {title!r}")


class StageSchemas(unittest.TestCase):
    def table(self, base: dict, head: dict) -> tuple[dict, list[str]]:
        st = report.stage_figures(runs(base, head), MHZ, True)
        return st, report.stage_lines(st, True)

    def test_base_without_a_stage(self):
        st, lines = self.table(WITHOUT, WITH)
        self.assertNotIn("assemble", st["arms"]["base"]["stages"])
        self.assertIn("assemble", st["arms"]["head"]["stages"])
        self.assertIsNone(st["change"]["assemble"])
        cells = row(lines, "of which: segment assembly")
        self.assertTrue(cells[1].startswith("–"), cells)  # unavailable, not 0.00
        self.assertEqual(cells[3], "–")

    def test_head_without_a_stage(self):
        st, lines = self.table(WITH, WITHOUT)
        self.assertNotIn("assemble", st["arms"]["head"]["stages"])
        self.assertIsNone(st["change"]["assemble"])
        cells = row(lines, "of which: segment assembly")
        self.assertTrue(cells[1].endswith("–"), cells)
        self.assertEqual(cells[2], "")  # no share for a stage head lacks

    def test_shared_stages_compared(self):
        st, _ = self.table(WITHOUT, WITH)
        for key in ("scan", "mtf", "build", "walk", "rle", "crc", "total"):
            self.assertEqual(st["change"][key][0], 1.0, key)

    def test_row_within_the_walk_not_counted_twice(self):
        st, lines = self.table(WITH, WITH)
        head = st["arms"]["head"]["stages"]
        # the total is phases' own, which counts the assembly once, inside the walk
        self.assertAlmostEqual(head["total"]["ns_per_byte"], WITH["total"] / 1000)
        summed = sum(head[k]["ns_per_byte"] for k in ("scan", "mtf", "build", "walk", "rle", "crc"))
        self.assertAlmostEqual(summed, head["total"]["ns_per_byte"])
        share = row(lines, "of which: segment assembly")[2]
        self.assertEqual(share, f"{100 * 300 / 8950:.1f}%")


if __name__ == "__main__":
    unittest.main()
