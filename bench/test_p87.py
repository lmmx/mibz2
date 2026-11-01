"""P87's registered analysis on synthetic trials (docs/journal/2026-11-01-p87-four-core-protocol.md §7-8):
each decision reaches what its rule says, on its own, and neither stands in for the other.

- D material while B is slower at four workers: mechanism MATERIAL, engineering NEGATIVE;
- a single-worker (local) improvement, the same at w = 1 and 4: D NOT MATERIAL, engineering
  IMPROVES;
- a cross-worker improvement: MATERIAL and IMPROVES;
- no effect: NOT MATERIAL and NOT DEMONSTRATED;
- D's interval crossing the threshold: INCONCLUSIVE;
- an A/A failure, a correctness failure, an unbalanced placement, the arms' instructions
  differing: each makes the decisions what the rules say, never a pass;
- a control harmed, and RSS grown: engineering NEGATIVE whatever the primary says;
- the phase attribution from marks, and the counters' idle subtraction.

    python3 bench/test_p87.py
"""

import argparse
import contextlib
import io
import json
import random
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import p87  # noqa: E402

BYTES = {"wikidata-20261026": 207_000_624, "records": 9_600_000, "random": 8_000_000}
ROLES = {"wikidata-20261026": "primary", "records": "control", "random": "control"}


def trial(cpu, noise=0.02, seed=1, rss=None, sha_fail=False, bad_placement=False, aa_shift=0.0):
    """ROW lines for a full design: cpu(name, w, arm) -> mean CPU-s/GB; each observation
    with Gaussian noise (sd `noise`, CPU-s/GB); throughput follows CPU (w = 4: 3.6 busy)"""
    rng = random.Random(seed)
    lines = []
    tl = "mibz2-0|1-4|1;mibz2-scan|1-4|2"
    for rd in range(-1, p87.ROUNDS):
        for c, name in enumerate(BYTES):
            for w in p87.WORKERS:
                for arm in range(3):
                    mean = cpu(name, w, p87.ARMS[arm]) * (1 + (aa_shift if arm == 2 else 0))
                    v = max(0.1, mean + rng.gauss(0, noise))
                    n = BYTES[name]
                    user_us = int(v * n / 1e9 * 1e6)
                    wall_ns = int(v * n / (1 if w == 1 else 3.6))
                    placement = rd % 2 if rd >= 0 else 0
                    if bad_placement and arm == 1 and rd >= 0:
                        placement = 0
                    r = rss(name, w, p87.ARMS[arm]) if rss else 100_000
                    ok = "0" if sha_fail and (rd, c, w, arm) == (3, 0, 4, 1) else "1"
                    f = ["ROW", rd, 0, c, name, w, arm, 0, wall_ns, user_us, 0, n, ok, 0, 0, f"Workers({w})", "1-4", 4,
                         w + 3, tl, w + 3, tl, user_us, 0, r, 1000, 0, 0, 0, int(rd < 0), wall_ns, placement]
                    lines.append("\t".join(map(str, f)))
    return "\n".join(lines) + "\n"


def run_summary(raw):
    with tempfile.TemporaryDirectory() as d:
        d = Path(d)
        (d / "raw.tsv").write_text(raw)
        (d / "manifest.tsv").write_text("".join(f"{n}\t{ROLES[n]}\t/x/{n}.bz2\th\th\t{b}\n" for n, b in BYTES.items()))
        (d / "alloc.json").write_text(json.dumps(dict(alloc=[1, 2, 3, 4], idle_siblings=[11, 12, 13, 14])))
        a = argparse.Namespace(manifest=str(d / "manifest.tsv"), raw=str(d / "raw.tsv"), allocation=str(d / "alloc.json"),
                               out=str(d / "out.json"), stat_before=None, stat_after=None)
        with contextlib.redirect_stdout(io.StringIO()):
            p87.summary(a)
        return json.loads((d / "out.json").read_text())["result"]


def flat(base=4.0, w4=4.6):
    return lambda name, w, arm: base if w == 1 else w4


class Decisions(unittest.TestCase):
    def test_D_material_but_B_slower_at_four_workers(self):
        # B costs 0.40 CPU-s/GB at w = 1 and only 0.20 at w = 4: D = +0.20, B still slower
        def cpu(name, w, arm):
            if name != p87.PRIMARY:
                return 4.0
            return {1: 4.0, 4: 4.6}[w] + ({1: 0.40, 4: 0.20}[w] if arm == "B" else 0)
        r = run_summary(trial(cpu))
        d = r["decisions"]
        self.assertAlmostEqual(r["mechanism"]["D"]["mean"], 0.20, delta=0.03)
        self.assertEqual(d["mechanism"], "MATERIAL")
        self.assertEqual(d["engineering"], "NEGATIVE")
        self.assertTrue(r["engineering"]["primary_w4"]["worse"])
        self.assertIn("outweighed", d["matrix"])

    def test_a_local_improvement_is_not_the_mechanism(self):
        # B saves 0.30 CPU-s/GB at both w = 1 and 4: D = 0, engineering improves
        def cpu(name, w, arm):
            base = {1: 4.0, 4: 4.6}[w] if name == p87.PRIMARY else 4.0
            return base - (0.30 if arm == "B" and name == p87.PRIMARY else 0)
        r = run_summary(trial(cpu))
        d = r["decisions"]
        self.assertEqual(d["mechanism"], "NOT MATERIAL")
        self.assertEqual(d["engineering"], "IMPROVES")
        self.assertTrue(d["practical"])
        self.assertIn("local or persistent", d["matrix"])

    def test_a_cross_worker_improvement(self):
        def cpu(name, w, arm):
            base = {1: 4.0, 4: 4.6}[w] if name == p87.PRIMARY else 4.0
            return base - (0.30 if arm == "B" and name == p87.PRIMARY and w == 4 else 0)
        d = run_summary(trial(cpu))["decisions"]
        self.assertEqual((d["mechanism"], d["engineering"]), ("MATERIAL", "IMPROVES"))

    def test_no_effect(self):
        d = run_summary(trial(flat()))["decisions"]
        self.assertEqual((d["mechanism"], d["engineering"]), ("NOT MATERIAL", "NOT DEMONSTRATED"))
        self.assertIn("closes negative", d["matrix"])

    def test_D_crossing_the_threshold_is_inconclusive(self):
        def cpu(name, w, arm):
            base = {1: 4.0, 4: 4.6}[w] if name == p87.PRIMARY else 4.0
            return base - (0.10 if arm == "B" and name == p87.PRIMARY and w == 4 else 0)
        r = run_summary(trial(cpu, noise=0.15, seed=1))
        D = r["mechanism"]["D"]
        self.assertLess(D["lower95"], p87.D_THRESHOLD)
        self.assertGreaterEqual(D["upper95"], p87.D_THRESHOLD)
        self.assertEqual(r["decisions"]["mechanism"], "INCONCLUSIVE")

    def test_an_aa_failure_decides_nothing(self):
        # A2, the same arm as A, 3% dearer everywhere: the calibration fails
        def cpu(name, w, arm):
            base = {1: 4.0, 4: 4.6}[w] if name == p87.PRIMARY else 4.0
            return base - (0.30 if arm == "B" and name == p87.PRIMARY and w == 4 else 0)
        r = run_summary(trial(cpu, aa_shift=0.03))
        self.assertFalse(r["aa"]["ok"])
        self.assertEqual(r["decisions"]["mechanism"], "INCONCLUSIVE (A/A calibration)")
        self.assertEqual(r["decisions"]["engineering"], "INCONCLUSIVE (A/A calibration)")

    def test_a_correctness_failure(self):
        d = run_summary(trial(flat(), sha_fail=True))["decisions"]
        self.assertEqual(d["mechanism"], "NEGATIVE (correctness)")
        self.assertEqual(d["engineering"], "NEGATIVE (correctness)")

    def test_an_unbalanced_placement_aborts(self):
        d = run_summary(trial(flat(), bad_placement=True))["decisions"]
        self.assertTrue(d["mechanism"].startswith("ABORTED (placement"))

    def test_the_instruction_check_only_withholds_the_mechanism(self):
        def cpu(name, w, arm):
            base = {1: 4.0, 4: 4.6}[w] if name == p87.PRIMARY else 4.0
            return base - (0.30 if arm == "B" and name == p87.PRIMARY and w == 4 else 0)
        r = run_summary(trial(cpu))
        d = p87.decisions(r, instr=False)
        self.assertEqual(d["mechanism"], "INCONCLUSIVE (the arms' instructions differ at w = 1)")
        self.assertEqual(d["engineering"], "IMPROVES")
        self.assertEqual(p87.decisions(r, instr=None)["mechanism"], "MATERIAL")

    def test_a_harmed_control_or_grown_rss_is_negative(self):
        def cpu(name, w, arm):
            base = {1: 4.0, 4: 4.6}[w] if name == p87.PRIMARY else 4.0
            if arm == "B" and name == p87.PRIMARY and w == 4:
                return base - 0.30
            return base + (0.20 if arm == "B" and name == "random" else 0)
        self.assertEqual(run_summary(trial(cpu))["decisions"]["engineering"], "NEGATIVE")

        def better(name, w, arm):
            base = {1: 4.0, 4: 4.6}[w] if name == p87.PRIMARY else 4.0
            return base - (0.30 if arm == "B" and name == p87.PRIMARY and w == 4 else 0)
        r = run_summary(trial(better, rss=lambda n, w, arm: 103_000 if arm == "B" else 100_000))
        self.assertEqual(r["decisions"]["engineering"], "NEGATIVE")
        self.assertEqual(r["decisions"]["mechanism"], "MATERIAL")


class Phases(unittest.TestCase):
    def test_phase_totals(self):
        head = "thread\tname\tns\tkind\tplace\tcpu_ns\trun_ns\twait_ns\tslices\tlen\tcap\n"
        rows = []
        # a segmented block, then a short one, on one worker; a scanner mark ignored
        for kind, ns, cpu, ln in [("DecodeStart", 0, 0, "-"), ("InvertStart", 10, 10, "900000"), ("WalkStart", 30, 30, "-"),
                                  ("AssembleStart", 70, 70, "-"), ("InvertEnd", 75, 75, "-"), ("DecodeEnd", 90, 90, "1000000"),
                                  ("DecodeStart", 100, 100, "-"), ("InvertStart", 101, 101, "1000"), ("WalkStart", 102, 102, "-"),
                                  ("InvertEnd", 105, 105, "-"), ("DecodeEnd", 106, 106, "1200")]:
            rows.append(f"1\tmibz2-0\t{ns}\t{kind}\t0\t{cpu}\t-\t-\t-\t{ln}\t-\n")
        rows.append("2\tmibz2-scan\t5\tIssued\t0\t-\t-\t-\t-\t10\t20\n")
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "m.tsv"
            p.write_text(head + "".join(rows))
            t = p87.phase_totals(p)
        self.assertEqual(t["n_errors"], 0)
        self.assertEqual((t["blocks"], t["segmented"], t["rows"], t["segmented_rows"], t["decoded"]), (2, 1, 901000, 900000, 1001200))
        self.assertEqual(t["cpu_ns"], dict(mtf=11, build=21, traversal=40, short_walk=3, assembly=5, post=16))

    def test_out_of_order_marks_are_errors(self):
        head = "thread\tname\tns\tkind\tplace\tcpu_ns\trun_ns\twait_ns\tslices\tlen\tcap\n"
        rows = [f"1\tmibz2-0\t{i}\t{k}\t0\t{i}\t-\t-\t-\t1\t-\n" for i, k in enumerate(["DecodeStart", "WalkStart", "InvertStart", "InvertEnd", "DecodeEnd"])]
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "m.tsv"
            p.write_text(head + "".join(rows))
            t = p87.phase_totals(p)
        self.assertEqual(t["blocks"], 0)
        self.assertEqual(t["n_errors"], 1)


class Counters(unittest.TestCase):
    def test_imc_idle_subtracted_per_decoded_byte(self):
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            runs = []
            for rep in range(3):
                for arm, reads in (("A", 3_000_000), ("B", 2_000_000)):
                    for w in p87.WORKERS:
                        (d / f"{arm}{w}{rep}.csv").write_text(f"{reads},,uncore_imc/data_reads/,1,100.00,,\n{reads},,uncore_imc/data_writes/,1,100.00,,\n")
                        (d / f"{arm}{w}{rep}.idle.csv").write_text("100000,,uncore_imc/data_reads/,1,100.00,,\n100000,,uncore_imc/data_writes/,1,100.00,,\n")
                        runs.append(dict(w=w, rep=rep, arm=arm, **{"pass": "imc"}, csv=f"{arm}{w}{rep}.csv", idle_csv=f"{arm}{w}{rep}.idle.csv",
                                         rc=0, wall_s=2.0, idle_wall_s=1.0, out=64_000_000))
                        (d / f"c{arm}{w}{rep}.csv").write_text("4e9,,cycles,1,100.00,,\n4e9,,ref-cycles,1,100.00,,\n"
                                                               + f"{1e10 * (1.001 if arm == 'B' else 1)},,instructions,1,100.00,,\n")
                        runs.append(dict(w=w, rep=rep, arm=arm, **{"pass": "core"}, csv=f"c{arm}{w}{rep}.csv", rc=0, wall_s=1.0, out=64_000_000))
            (d / "runs.jsonl").write_text("".join(json.dumps(r) + "\n" for r in runs))
            (d / "probe.json").write_text("{}")
            a = argparse.Namespace(runs=str(d / "runs.jsonl"), dir=str(d), probe=str(d / "probe.json"), out=str(d / "c.json"))
            with contextlib.redirect_stdout(io.StringIO()):
                p87.counters(a)
            res = json.loads((d / "c.json").read_text())["result"]
        # (3e6 - 2 x 1e5) lines x 64 B / 64e6 B = 2.8 bytes a decoded byte; B 1.8
        r = res["imc"]["w=1"]["uncore_imc/data_reads/"]
        self.assertAlmostEqual(r["A"], 2.8)
        self.assertAlmostEqual(r["B_minus_A"]["mean"], -1.0)
        self.assertTrue(res["instructions_w1_ok"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
