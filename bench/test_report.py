"""The stage table across builds whose phases report different stages (bench 2.1.0 added
the segment assembly, within the walk's row): the figures and their comparison read only
what both builds have, a missing stage shows as unavailable, never as zero, and a row
within another is never counted twice.

    python3 bench/test_report.py
"""

import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import report  # noqa: E402

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "docs" / "scripts"))
import bench_history  # noqa: E402

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


class Version(unittest.TestCase):
    def test_bench_matches_the_changelog(self):
        # BENCH and bench/CHANGELOG.md's newest entry agree: a bump comes with its entry, an
        # entry with its bump. Whether a change needed a new version is the reviewer's call
        text = (Path(__file__).resolve().parent / "CHANGELOG.md").read_text()
        newest = re.search(r"^## \[(\d+\.\d+\.\d+)\]", text, re.M)
        self.assertIsNotNone(newest, "no version entry in bench/CHANGELOG.md")
        self.assertEqual(report.BENCH, newest.group(1))

    def test_meaning_joins_only_what_kept_its_meaning(self):
        self.assertEqual(report.meaning_major("decoder", "1.2.0"), 1)
        self.assertEqual(report.meaning_major("decoder", "2.1.1"), 1)
        self.assertEqual(report.meaning_major("stages", "1.2.0"), 1)
        self.assertEqual(report.meaning_major("stages", "2.0.0"), 2)
        major = int(report.BENCH.split(".")[0])
        for family, majors in report.MEANING.items():
            self.assertEqual(majors, sorted(set(majors)), family)
            self.assertLessEqual(majors[-1], major, family)

    def test_history_joins_versions_never_runners_or_corpora(self):
        corpora = {"records": {}, "text": {}, "wikidata": {}}

        def res(bench="1.0.0", cpu="AMD EPYC 7763 64-Core Processor", cores=4, names=corpora):
            return {
                "bench": bench,
                "machine": {"cpu": cpu, "cores": cores},
                "corpora": {k: {} for k in names},
            }

        def key(r):
            return bench_history.line_class(r)[0]

        # the decoder's figures kept their meaning from 1.0.0, so its 1.x and 2.x points
        # on one runner and one set of corpora share a line
        self.assertEqual(key(res("1.0.0")), key(res("2.1.1")))
        self.assertEqual(key(res("1.2.0")), key(res("2.0.0")))
        # and never across runner models, core counts or corpora, whatever the version
        for bench in ("1.0.0", "2.1.1"):
            self.assertNotEqual(
                key(res(bench)), key(res(bench, cpu="AMD EPYC 9V74 80-Core Processor"))
            )
            self.assertNotEqual(key(res(bench)), key(res(bench, cores=2)))
            self.assertNotEqual(key(res(bench)), key(res(bench, names=["records", "text"])))
            # as many corpora, not the same ones
            self.assertNotEqual(key(res(bench)), key(res(bench, names=["records", "text", "runs"])))
            self.assertNotEqual(
                key(res(bench)), key(res(bench, names=["records", "text", "wikidata", "runs"]))
            )
        # the order the corpora are stored in doesn't split a line
        self.assertEqual(key(res(names=["wikidata", "text", "records"])), key(res()))

    def test_history_joins_a_corpus_only_with_the_same_content(self):
        def res(bench, sha):
            return {
                "bench": bench,
                "machine": {"cpu": "AMD EPYC 7763 64-Core Processor", "cores": 4},
                "corpora": {"text": {"sha256": sha}, "wikidata": {"sha256": "82f65ace" * 8}},
            }

        a, b = res("1.0.0", "05591211" * 8), res("2.1.1", "677bf073" * 8)
        # one name, one class, different bytes: two workloads, never one line (`text` is
        # this crate's sources, so its bytes change with them)
        self.assertEqual(bench_history.line_class(a), bench_history.line_class(b))
        self.assertNotEqual(bench_history.line_key(a, "text"), bench_history.line_key(b, "text"))
        # the same bytes join across the versions
        self.assertEqual(
            bench_history.line_key(a, "wikidata"), bench_history.line_key(b, "wikidata")
        )


if __name__ == "__main__":
    unittest.main()
