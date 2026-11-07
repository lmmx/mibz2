"""bench/plan_diagnostics.py's outcome maps on synthetic i9-shaped attempts: a clean attempt is
VALID and reads as its effects say; each defect the reports guard against makes it INVALID.

    python3 -m unittest bench/test_plan_diagnostics.py      (or python3 -m pytest)
"""

import json
import random
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import plan_diagnostics as pd  # noqa: E402

HEAD = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True, cwd=HERE).stdout.strip()
LSCPU = """Architecture: x86_64
CPU(s): 20
Thread(s) per core: 2
Core(s) per socket: 10
Socket(s): 1
L3 cache: 20 MiB (1 instance)
"""
CORE_CPUS = [f"{c},{c + 10}" for c in range(10)]
BASE = "mibz2@/w/bin/ab-base"


def names(plan, busy=True):
    leaders, plain, piped = plan
    out = [f"mibz2-{i}" for i in range(leaders)]
    out += [f"mibz2-coop-{1 + i % 3}" for i in range(plain)]
    out += [f"mibz2-coop-p{1 + i % 3}" for i in range(piped)]
    return out


def attempt(cells, speed=None, plan_override=None, mask_override=None, dry=False, rounds=20, seed=3, base=False, sums=None, wobble=None):
    """an attempt directory: every cell's arms, `speed[(cell, arm, corpus)]` the arm's speed
    over 1 (its wall divided by it): about 2% noise a round, shared by its arms, and 3% a run;
    `wobble[key]` a further d, alternately faster and slower by round (a wide interval)"""
    speed = speed or {}
    rng = random.Random(seed)
    w = Path(tempfile.mkdtemp(prefix="plan-diag-"))
    (w / "commit.txt").write_text(HEAD + "\n")
    (w / "git-status.txt").write_text("")
    (w / "lscpu.txt").write_text(LSCPU)
    (w / "env.json").write_text(json.dumps({"cpus": 20, "cores": 10, "core_cpus": CORE_CPUS}))
    if dry:
        (w / "DRY-RUN.txt").write_text("dry\n")
    if base:
        (w / "commits.txt").write_text(f"head {HEAD}\nbase {pd.V024}\n")
    corpora = sorted({c for cell in cells for c in cell.corpora})
    sums = sums or {c: pd.LEVELS_SHA[c] for c in corpora}
    (w / "corpus.sha256").write_text("".join(f"{s}  /x/{c}.bz2\n" for c, s in sums.items()))
    masks = []
    for cell in cells:
        reg = pd.mask_cpus(cell.mask, CORE_CPUS)
        mask = "all" if reg is None else ",".join(map(str, reg))
        mask = (mask_override or {}).get(cell.name, mask)
        masks.append(f"{cell.name} {mask}")
        allowed = "0-19" if mask == "all" else mask
        lines = []
        for r in range(rounds):
            noise = rng.uniform(0.98, 1.02)
            for arm in cell.arms:
                for corpus in cell.corpora:
                    plan = (plan_override or {}).get((cell.name, arm, corpus), cell.plan(arm, corpus))
                    wall = 0.15 * noise * rng.uniform(0.97, 1.03) / speed.get((cell.name, arm, corpus), 1.0)
                    wall *= 1 + (wobble or {}).get((cell.name, arm, corpus), 0.0) * (-1) ** r
                    tc = " ".join(f"{n}=0.10" for n in names(plan) + ["mibz2-scan"])
                    lines.append(
                        json.dumps(
                            {
                                "round": r,
                                "corpus": corpus,
                                "threads": cell.threads,
                                "arm": BASE if arm == "base" else arm,
                                "allowed": allowed,
                                "freq_mhz": 4900.0,
                                "wall": wall,
                                "cpu": wall * 6,
                                "rss_kb": 90000,
                                "out": 209700196,
                                "hash": "abc",
                                "thread_cpu": tc,
                                "observed_threads": " ".join(sorted(names(plan) + ["ab", "mibz2-read", "mibz2-scan"])),
                                "err": "",
                            }
                        )
                    )
        (w / f"runs-{cell.name}.jsonl").write_text("\n".join(lines) + "\n")
    (w / "masks.txt").write_text("\n".join(masks) + "\n")
    return w


class Boundaries(unittest.TestCase):
    def run_it(self, **kw):
        w = attempt(pd.BOUNDARIES, **kw)
        try:
            return pd.boundaries(str(w))[0]
        finally:
            shutil.rmtree(w)

    def test_no_effects_neither_resolved(self):
        v = self.run_it()
        self.assertEqual(v["verdict"], "VALID", [c for c in v["checks"] if not c[1]])
        r = v["readings"]
        self.assertTrue(r["nine"].startswith("LEAVES BOTH OPEN"))
        self.assertEqual(r["rule_a"], "NOT FALSIFIED AT 8")
        self.assertTrue(r["rule_b"].startswith("FALSIFIED AT 8"), r["rule_b"])  # 3x2's upper bounds below 1.03
        self.assertTrue(r["nine_core_mask"].startswith("UNRESOLVED"))

    def test_rule_b_supported_and_rule_a_falsified(self):
        sp = {(c, "mibz2-plan-3x2", "wikidata"): 1.08 for c in pd.EIGHT}
        sp[("U9", "mibz2-plan-3x3p", "wikidata")] = 1.16
        v = self.run_it(speed=sp)
        r = v["readings"]
        self.assertTrue(r["nine"].startswith("SUPPORTS BOTH"))
        self.assertTrue(r["rule_a"].startswith("FALSIFIED AT 8"))
        self.assertTrue(r["rule_b"].startswith("SUPPORTED AT 8"))
        self.assertIn("rule A FALSIFIED", r["eight"])

    def test_neither_resolved(self):
        # 3x2 at about 1 at every 8-thread cell with wide intervals: never FASTER (rule A
        # stands), upper bounds above 1.03 (rule B neither falsified nor supported)
        wob = {(c, "mibz2-plan-3x2", "wikidata"): 0.06 for c in pd.EIGHT}
        v = self.run_it(wobble=wob)
        r = v["readings"]
        self.assertEqual(r["rule_a"], "NOT FALSIFIED AT 8")
        self.assertTrue(r["rule_b"].startswith("NEITHER"), r["rule_b"])
        self.assertTrue(r["eight"].startswith("NEITHER RESOLVED"))
        self.assertIn("does not favour rule A", r["eight"])

    def test_two_by_four_falsifies_a_alone(self):
        v = self.run_it(speed={("U8", "mibz2-plan-2x4p", "wikidata"): 1.14})
        self.assertIn("U8 mibz2-plan-2x4p", v["readings"]["rule_a"])

    def test_nine_shared_slower_falsifies_both(self):
        v = self.run_it(speed={("U9", "mibz2-plan-3x3p", "wikidata"): 0.9})
        self.assertTrue(v["readings"]["nine"].startswith("FALSIFIES BOTH"))

    def test_nine_core_mask_information_only(self):
        v = self.run_it(speed={("M9", "mibz2-plan-3x3p", "wikidata"): 1.1})
        self.assertTrue(v["readings"]["nine_core_mask"].startswith("FASTER: the core-left-over"))
        self.assertEqual(v["readings"]["rule_a"], "NOT FALSIFIED AT 8")

    def test_wrong_plan_is_invalid(self):
        v = self.run_it(plan_override={("M8", "mibz2-plan-3x2", "wikidata"): (4, 4, 0)})
        self.assertEqual(v["verdict"], "INVALID")
        self.assertIsNone(v["readings"])

    def test_wrong_mask_is_invalid(self):
        v = self.run_it(mask_override={"M8S": "0-7"})
        self.assertEqual(v["verdict"], "INVALID")

    def test_dry_and_short_runs_are_invalid(self):
        self.assertEqual(self.run_it(dry=True)["verdict"], "INVALID")
        self.assertEqual(self.run_it(rounds=19)["verdict"], "INVALID")

    def test_masks(self):
        self.assertEqual(pd.mask_cpus((8, "one"), CORE_CPUS), list(range(8)))
        self.assertEqual(pd.mask_cpus((8, "both"), CORE_CPUS), list(range(8)) + list(range(10, 18)))
        self.assertIsNone(pd.mask_cpus((12, "one"), CORE_CPUS))


class Bytes(unittest.TestCase):
    def run_it(self, **kw):
        w = attempt(pd.BYTES, **kw)
        try:
            return pd.bytes_report(str(w))[0]
        finally:
            shutil.rmtree(w)

    def sp(self, l6, l7, l9):
        return {("B10", "mibz2-plan-3x3", f"wikidata-l{lv}"): s for lv, s in ((6, l6), (7, l7), (9, l9))}

    def test_contradicted(self):
        v = self.run_it(speed=self.sp(1.08, 1.08, 1.08))
        self.assertEqual(v["verdict"], "VALID", [c for c in v["checks"] if not c[1]])
        self.assertTrue(v["reading"].startswith("PREMISE CONTRADICTED"))

    def test_supported(self):
        self.assertTrue(self.run_it(speed=self.sp(0.99, 1.03, 1.08))["reading"].startswith("PREMISE SUPPORTED"))

    def test_inconclusive_without_level_nine(self):
        self.assertTrue(self.run_it(speed=self.sp(1.0, 1.0, 1.0))["reading"].startswith("INCONCLUSIVE"))

    def test_unresolved(self):
        # level 6 near 1.03 with a wide interval: neither above 1.03 nor below it
        wob = {("B10", "mibz2-plan-3x3", "wikidata-l6"): 0.06}
        self.assertEqual(self.run_it(speed=self.sp(1.03, 1.05, 1.08), wobble=wob)["reading"], "UNRESOLVED")

    def test_wrong_corpus_is_invalid(self):
        sums = {c: pd.LEVELS_SHA[c] for c in ("wikidata-l6", "wikidata-l7")}
        sums["wikidata-l9"] = "0" * 64
        self.assertEqual(self.run_it(sums=sums)["verdict"], "INVALID")


class Levels(unittest.TestCase):
    def run_it(self, **kw):
        w = attempt(pd.LEVELS, base=True, **kw)
        try:
            return pd.levels(str(w))[0]
        finally:
            shutil.rmtree(w)

    def test_decoder_count(self):
        # 0.2.6's three decoders at 0.8 of 0.2.4's four; four forced on 0.2.6 as fast as 0.2.4
        sp = {}
        for t in (10, 20):
            for lv in (1, 2):
                sp[(f"L{t}", "mibz2", f"wikidata-l{lv}")] = 0.8
        v = self.run_it(speed=sp)
        self.assertEqual(v["verdict"], "VALID", [c for c in v["checks"] if not c[1]])
        for lv in ("wikidata-l1", "wikidata-l2"):
            self.assertIn("SLOWDOWN SHOWN", v["readings"][lv])
            self.assertIn("THE DECODER COUNT", v["readings"][lv])

    def test_other_changes(self):
        sp = {}
        for t in (10, 20):
            sp[(f"L{t}", "mibz2", "wikidata-l1")] = 0.8
            sp[(f"L{t}", "mibz2-plan-4x2", "wikidata-l1")] = 0.8
        v = self.run_it(speed=sp)
        self.assertIn("OTHER CHANGES", v["readings"]["wikidata-l1"])
        self.assertTrue(v["readings"]["wikidata-l2"].startswith("SLOWDOWN NOT SHOWN"))

    def test_both_and_neither(self):
        sp = {}
        for t in (10, 20):
            sp[(f"L{t}", "mibz2", "wikidata-l1")] = 0.7
            sp[(f"L{t}", "mibz2-plan-4x2", "wikidata-l1")] = 0.85
            sp[(f"L{t}", "mibz2", "wikidata-l2")] = 0.8
        # level 2: at 10, four forced decoders as fast as 0.2.4 (the count); at 20, as slow as
        # 0.2.6's default (the rest): neither at both thread counts
        sp[("L20", "mibz2-plan-4x2", "wikidata-l2")] = 0.8
        v = self.run_it(speed=sp)
        self.assertIn("BOTH", v["readings"]["wikidata-l1"])
        self.assertIn("NEITHER RESOLVED", v["readings"]["wikidata-l2"])

    def test_not_shown_beyond_the_inference(self):
        v = self.run_it()
        self.assertIn("wholly above the inferred", v["readings"]["wikidata-l1"])

    def test_helpers_at_level_one_are_invalid(self):
        # a team at level 1 that started helpers is not the registered plan
        v = self.run_it(plan_override={("L10", "mibz2", "wikidata-l1"): (3, 0, 6)})
        self.assertEqual(v["verdict"], "INVALID")

    def test_base_must_be_v024(self):
        w = attempt(pd.LEVELS, base=True)
        try:
            (w / "commits.txt").write_text(f"head {HEAD}\nbase {'1' * 40}\n")
            self.assertEqual(pd.levels(str(w))[0]["verdict"], "INVALID")
        finally:
            shutil.rmtree(w)


if __name__ == "__main__":
    unittest.main()
