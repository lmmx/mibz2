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
        # 3.0.0's sampler, in each run's process, redefined the decoder's CPU figures
        self.assertEqual(report.meaning_major("decoder", "3.0.0"), 3)
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


def decodes(cpu: dict, wall: dict, rounds: int = 6, rss: dict | None = None, **extra) -> list[dict]:
    """`ab run`-shaped records on the Wikidata corpus: for each (arm, threads), each
    round's CPU and wall as cpu[(arm, t)] * (1 + jitter); the jitter is the same for every
    arm in a round, as a runner's drift is, plus a small spread of its own."""
    out = []
    for r in range(rounds):
        drift = 1 + 0.02 * ((r % 3) - 1)
        for (arm, t), c in cpu.items():
            own = 1 + 0.004 * ((r * 7 + t) % 5 - 2)
            w = wall[(arm, t)] * drift * own
            out.append(
                {
                    "round": r,
                    "corpus": "wikidata",
                    "threads": t,
                    "arm": arm,
                    "parallel": True,
                    "cpus": {1: "1", 2: "0,2"}.get(t, "all"),
                    "wall": w,
                    "cpu": c * drift * own,
                    "rss_kb": (rss or {}).get((arm, t), 50_000),
                    "out": 209_700_196,
                    "hash": "h",
                    "threads_busy": t + 2,
                    "thread_cpu": f"main={0.9 * c * drift * own:.2f}",
                    "runq_s": 0.01,
                    "throttled_n": 0,
                    "err": "",
                    **extra,
                }
            )
    return out


def pr_result(runs: list[dict]) -> dict:
    """The parts of `summary`'s result the assessment reads."""
    res = {
        "threads": sorted({r["threads"] for r in runs}),
        "figures": report.figures(runs, "mibz2@base/ab", None, True),
        "parity": True,
        "accounting": report.accounting(runs),
        "resources": report.resources(runs, True),
        "paired_resources": report.paired_resources(runs, "mibz2", "mibz2@base/ab"),
    }
    res["assessment"] = report.assessment(res)
    return res


# CPU seconds and wall at 1, 2 and 4 threads: 1.0 s alone, then 1.2x and 1.6x the CPU
BASE_CPU = {1: 1.0, 2: 1.2, 4: 1.6}
BASE_WALL = {1: 1.0, 2: 0.6, 4: 0.45}


def builds(head_cpu=BASE_CPU, head_wall=BASE_WALL) -> tuple[dict, dict]:
    cpu = {("mibz2@base/ab", t): v for t, v in BASE_CPU.items()}
    cpu |= {("mibz2", t): v for t, v in head_cpu.items()}
    wall = {("mibz2@base/ab", t): v for t, v in BASE_WALL.items()}
    wall |= {("mibz2", t): v for t, v in head_wall.items()}
    return cpu, wall


class Resources(unittest.TestCase):
    def test_amplification_is_cpu_over_the_fewest_threads(self):
        res = pr_result(decodes(*builds()))
        head = res["resources"]["wikidata"]["head"]["threads"]
        self.assertAlmostEqual(head["2"]["amplification"][0], 1.2, places=2)
        self.assertAlmostEqual(head["4"]["amplification"][0], 1.6, places=2)
        # efficiency: the speedup (1 / 0.45) over 4 threads
        self.assertAlmostEqual(head["4"]["efficiency"][0], 1 / 0.45 / 4, places=2)
        self.assertEqual(res["resources"]["wikidata"]["head"]["best"], 4)

    def test_the_same_build_twice_moves_nothing(self):
        a = pr_result(decodes(*builds()))["assessment"]
        self.assertEqual(a["better"], [])
        self.assertEqual(a["worse"], [])
        self.assertEqual(a["broader_hardware"], [])
        self.assertTrue(a["verdict"].startswith("No change"))

    def test_faster_for_more_cpu_a_thread_is_a_trade_off(self):
        # head: 10% faster at 4 threads, for 15% more CPU there (amplification up)
        cpu = dict(BASE_CPU) | {4: 1.6 * 1.15}
        wall = dict(BASE_WALL) | {4: 0.45 * 0.9}
        a = pr_result(decodes(*builds(cpu, wall)))["assessment"]
        self.assertIn("speed", a["better"])
        self.assertIn("CPU cost", a["worse"])
        self.assertIn("contention sensitivity", a["worse"])
        self.assertTrue(a["verdict"].startswith("A trade-off"))
        self.assertIn("CPU amplification rose", a["broader_hardware"])

    def test_cheaper_at_one_thread_only_is_more_sensitive(self):
        # head: 20% less CPU at 1 thread, the same at 2 and 4: its amplification rose,
        # though no figure at 4 threads moved
        cpu = dict(BASE_CPU) | {1: 0.8}
        wall = dict(BASE_WALL) | {1: 0.8}
        a = pr_result(decodes(*builds(cpu, wall)))["assessment"]
        self.assertEqual(a["signs"]["contention sensitivity"], 1)
        self.assertNotIn("CPU cost", a["worse"])
        self.assertIn("CPU amplification rose", a["broader_hardware"])

    def test_throttling_makes_it_inconclusive(self):
        runs = decodes(*builds())
        runs[0]["throttled_n"] = 3
        a = pr_result(runs)["assessment"]
        self.assertTrue(a["verdict"].startswith("Inconclusive"))
        self.assertIn("throttled", a["verdict"])


class Accounting(unittest.TestCase):
    def test_a_thread_counted_twice_fails(self):
        runs = decodes(*builds())
        good = report.accounting(runs)
        self.assertEqual(good, [])
        # this build's run whose threads hold more than the process's total
        head = next(r for r in runs if r["arm"] == "mibz2")
        bad = dict(head, thread_cpu="mibz2-0=0.60 mibz2-1=0.60 =0.60 main=0.04")
        self.assertEqual(len(report.accounting([bad])), 1)

    def test_a_base_built_before_3_0_0_is_not_held_to_it(self):
        # (its child is the base's own harness: no runq_s, its split from 50 ms samples)
        runs = decodes(*builds())
        old = {k: v for k, v in runs[0].items() if k != "runq_s"}
        old["thread_cpu"] = "main=9.99"
        self.assertEqual(report.accounting([old]), [])

    def test_the_split_reports_what_it_leaves_unattributed(self):
        run = decodes(*builds())[0]
        run = dict(run, cpu=1.0, thread_cpu="mibz2-scan=0.1 mibz2-0=0.4 mibz2-1=0.4 main=0.02")
        s = report.split_cpu(run)
        self.assertAlmostEqual(s["unattributed"], 0.08, places=6)


if __name__ == "__main__":
    unittest.main()
