"""bench/pair_runs_validity.py on fixture runs: a complete one is VALID with its readings, and
each hole the Researcher's REVISE (6730270019) named is NOT VALID with no reading.

    python3 bench/test_pair_runs_validity.py
"""

import hashlib
import json
import math
import random
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import pair_runs_validity as v  # noqa: E402

HEAD = "a" * 40
CORPUS = "corpora/wikidata.bz2"


def make_run(d, nproc=2, rounds=3, ghz=2.8, seed=1):
    """A complete run: plan, corpus and spec, and every planned copy's three files"""
    d = Path(d)
    (d / "corpora").mkdir()
    (d / "bench").mkdir()
    data = b"BZh9 a stand-in corpus"
    (d / CORPUS).write_bytes(data)
    sha = hashlib.sha256(data).hexdigest()
    (d / "bench/wikidata.json").write_text(json.dumps({"corpus_sha256": sha}))
    copies = v.copies_for(nproc)
    plan = {
        "tool": "pair_runs",
        "rounds": rounds,
        "ghz": ghz,
        "nproc": nproc,
        "copies": copies,
        "head": HEAD,
        "frozen": HEAD,
        "corpus": CORPUS,
        "spec": "bench/wikidata.json",
        "corpus_sha256": sha,
    }
    (d / "plan.json").write_text(json.dumps(plan))
    rnd = random.Random(seed)
    files = [CORPUS] + [f"synthetic {s}" for s in v.SYNTHETIC]
    base = {"row": 1.0, "run": 0.8, "one": 0.4, "P": 2.0, "R": 2.2, "O": 2.6}
    for c in copies:
        lines = [",".join(v.HEADER)]
        for f in files:
            for r in range(rounds):
                for arm in v.ARMS:
                    sec = base[arm] * (1 + 0.01 * rnd.random())
                    lines.append(f"{f},{r},{arm},{sec:.9f},233,200000000,900000000")
        (d / f"raw-{c}.csv").write_text("\n".join(lines) + "\n")
        (d / f"status-{c}.txt").write_text("0\n")
        (d / f"sweep-{c}.txt").write_text("runner: x\nVALID (this process's part): ...\n")
    return d


def edit_plan(d, **kw):
    p = Path(d) / "plan.json"
    plan = json.loads(p.read_text())
    plan.update(kw)
    p.write_text(json.dumps(plan))


def edit_raw(d, copy, fn):
    p = Path(d) / f"raw-{copy}.csv"
    lines = p.read_text().splitlines()
    p.write_text("\n".join(fn(lines)) + "\n")


class Validity(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = make_run(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def invalid(self, why):
        ok, lines = v.validate(self.d)
        text = "\n".join(lines)
        self.assertFalse(ok, text)
        self.assertIn("NOT VALID (no readings)", text)
        self.assertNotIn("readings (UNJUDGED", text, "a verdict before validity")
        for word in ["POSITIVE", "NEGATIVE", "INCONCLUSIVE", "MET"]:
            self.assertNotIn(f": {word}", text)
        self.assertIn(why, text)

    def test_complete_run_is_valid_with_readings(self):
        ok, lines = v.validate(self.d)
        text = "\n".join(lines)
        self.assertTrue(ok, text)
        self.assertIn("Q1 R_build", text)
        self.assertIn("M1 run - one-hop", text)
        self.assertIn("Q2 R_whole", text)

    def test_wholly_missing_child(self):
        for kind in ["sweep-2-cpu1.txt", "status-2-cpu1.txt", "raw-2-cpu1.csv"]:
            (self.d / kind).unlink()
        self.invalid("copy 2-cpu1: sweep-2-cpu1.txt missing")

    def test_all_concurrent_copies_missing(self):
        for c in ["2-cpu0", "2-cpu1"]:
            for kind in [f"sweep-{c}.txt", f"status-{c}.txt", f"raw-{c}.csv"]:
                (self.d / kind).unlink()
        self.invalid("copy 2-cpu0: raw-2-cpu0.csv missing")

    def test_unplanned_copy(self):
        (self.d / "sweep-2-cpu7.txt").write_text("VALID (this process\n")
        self.invalid("sweep-2-cpu7.txt: not a planned copy")

    def test_plan_copies_must_match_the_cpus(self):
        edit_plan(self.d, copies=["1"])
        self.invalid("not ['1', '2-cpu0', '2-cpu1'] for 2 CPUs")

    def test_missing_plan(self):
        (self.d / "plan.json").unlink()
        self.invalid("plan.json")

    def test_failed_child(self):
        (self.d / "status-2-cpu0.txt").write_text("3\n")
        self.invalid("copy 2-cpu0: exit status '3'")

    def test_missing_status(self):
        (self.d / "status-1.txt").unlink()
        self.invalid("copy 1: status-1.txt missing")

    def test_no_valid_line(self):
        (self.d / "sweep-1.txt").write_text("runner: x\nNOT VALID (no readings):\n")
        self.invalid("copy 1: no VALID line")

    def test_empty_raw(self):
        (self.d / "raw-1.csv").write_text("")
        self.invalid("raw-1.csv: header None")

    def test_header_only_raw(self):
        (self.d / "raw-2-cpu1.csv").write_text(",".join(v.HEADER) + "\n")
        self.invalid("raw-2-cpu1.csv: corpora/wikidata.bz2: round 0 arm row missing")

    def test_truncated_raw(self):
        edit_raw(self.d, "1", lambda ls: ls[:-1])
        self.invalid("round 2 arm O missing")

    def test_truncated_row(self):
        edit_raw(self.d, "1", lambda ls: ls[:-1] + [ls[-1][:12]])
        self.invalid("raw-1.csv: a row of")

    def test_duplicate_round(self):
        edit_raw(self.d, "1", lambda ls: ls + [ls[1]])
        self.invalid("round 0 arm row twice")

    def test_round_outside_the_matrix(self):
        edit_raw(self.d, "1", lambda ls: ls + [ls[1].replace(",0,row,", ",9,row,")])
        self.invalid("round 9 arm 'row' outside the matrix")

    def test_unregistered_file(self):
        edit_raw(self.d, "1", lambda ls: ls + ["other.bz2" + ls[1][len(CORPUS):]])
        self.invalid("file 'other.bz2' not registered")

    def test_nonfinite_total(self):
        edit_raw(self.d, "1", lambda ls: [ls[0], ls[1].replace(ls[1].split(",")[3], "nan")] + ls[2:])
        self.invalid("round 0 arm row total nan")

    def test_zero_total_on_a_paired_file(self):
        edit_raw(self.d, "1", lambda ls: [ls[0], ls[1].replace(ls[1].split(",")[3], "0")] + ls[2:])
        self.invalid("round 0 arm row total 0.0")

    def test_zero_build_arms_only_with_no_paired_block(self):
        # a synthetic file with no paired block: its build arms exactly 0 is VALID
        def nopaired(ls):
            out = [ls[0]]
            for line in ls[1:]:
                f, r, arm, sec, pb, pr, ob = line.split(",")
                if f == "synthetic one byte":
                    pb, pr = "0", "0"
                    if arm in v.BUILD_ARMS:
                        sec = "0"
                out.append(",".join([f, r, arm, sec, pb, pr, ob]))
            return out

        for c in v.copies_for(2):
            edit_raw(self.d, c, nopaired)
        ok, lines = v.validate(self.d)
        self.assertTrue(ok, "\n".join(lines))
        # and Wikidata itself with no paired block is not
        for c in v.copies_for(2):
            edit_raw(self.d, c, lambda ls: [ls[0]] + [
                ",".join(x.split(",")[:4] + ["0", "0"] + x.split(",")[6:]) if x.startswith(CORPUS) else x
                for x in ls[1:]
            ])
        self.invalid("no paired block")

    def test_counts_change_between_rows(self):
        edit_raw(self.d, "1", lambda ls: [ls[0], ls[1].replace(",233,", ",232,")] + ls[2:])
        self.invalid("change between rows")

    def test_bad_clock_nan(self):
        edit_plan(self.d, ghz=math.nan)
        self.invalid("GHZ nan is not finite and positive")

    def test_bad_clock_zero(self):
        edit_plan(self.d, ghz=0)
        self.invalid("GHZ 0 is not finite and positive")

    def test_missing_m1(self):
        # M1's registered difference: missing when the paired rows make no c/B
        for c in v.copies_for(2):
            edit_raw(self.d, c, lambda ls: [ls[0]] + [
                x.replace(",233,200000000,", ",233,0,") if x.startswith(CORPUS) else x for x in ls[1:]
            ])
        self.invalid("interval M1 difference missing")

    def test_m1_difference_nonfinite_endpoints(self):
        self.assertIsNone(v.diff([[1, 2], [1, 2]], 0, 1, math.nan))
        self.assertIsNone(v.diff([[1, 2], [1, 2]], 0, 1, 0.0))
        self.assertIsNone(v.diff([[1e308, -1e308], [-1e308, 1e308]], 0, 1, 10.0))
        self.assertIsNotNone(v.diff([[1, 2], [1.1, 2]], 0, 1, 1.0))

    def test_head_not_frozen(self):
        edit_plan(self.d, frozen="b" * 40)
        self.invalid("is not the frozen head")

    def test_no_frozen_head(self):
        edit_plan(self.d, frozen="")
        self.invalid("none named at dispatch")

    def test_corpus_identity(self):
        (self.d / CORPUS).write_bytes(b"another corpus")
        self.invalid("not the spec's")

    def test_plan_corpus_identity(self):
        edit_plan(self.d, corpus_sha256="0" * 64)
        self.invalid("not the plan's")


if __name__ == "__main__":
    unittest.main()
