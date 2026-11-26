"""Tests of bench/squared_trace_verdict.py and the pinned pre-screen source (stdlib only).

python3 bench/test_squared_trace_verdict.py
"""

import hashlib
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import squared_trace_verdict as sv  # noqa: E402

SOURCE = os.path.join(HERE, "squared_trace", "squared_trace_prescreen.c")
# MONODROMY's original: docs/journal/monodromy/2026-11-23-refinement-and-composition/
# squared-trace-prescreen.c.txt at commit 3979c15ee7b8e051f7b076b4273425424f3ae87d
ORIGIN_COMMIT = "3979c15ee7b8e051f7b076b4273425424f3ae87d"
ORIGIN_PATH = (
    "docs/journal/monodromy/2026-11-23-refinement-and-composition/squared-trace-prescreen.c.txt"
)
ORIGIN_BLOB = "124421bf07182a6161752ff797a6eeb9cb3eedf6"
SOURCE_SHA256 = "4f3c62b77d4043d57f8a6bc6201bede82b63ec2109eaef316716d93815b3dfab"

# the record's prescreen.txt, its two invocations of squared-trace-prescreen.c.txt (the
# container, not the i9; the program's "(chk N)" tail left out there)
PRESCREEN_FIRST = """\
m 32768: chase T 2.26 ns/row | gather T^2 0.40 ns/row + chase T^2 1.00 ns/row = 1.40 ns/row | ratio 0.620
m 89874: chase T 3.05 ns/row | gather T^2 0.45 ns/row + chase T^2 1.38 ns/row = 1.83 ns/row | ratio 0.600
m 114000: chase T 3.25 ns/row | gather T^2 0.46 ns/row + chase T^2 1.57 ns/row = 2.03 ns/row | ratio 0.622
m 156404: chase T 4.46 ns/row | gather T^2 0.61 ns/row + chase T^2 3.11 ns/row = 3.72 ns/row | ratio 0.834
m 204281: chase T 7.62 ns/row | gather T^2 0.94 ns/row + chase T^2 5.82 ns/row = 6.77 ns/row | ratio 0.888
"""
PRESCREEN_SECOND = """\
m 89874: chase T 3.06 ns/row | gather T^2 0.44 ns/row + chase T^2 1.49 ns/row = 1.93 ns/row | ratio 0.633
m 114000: chase T 3.29 ns/row | gather T^2 0.46 ns/row + chase T^2 1.68 ns/row = 2.14 ns/row | ratio 0.651
m 156404: chase T 3.72 ns/row | gather T^2 0.50 ns/row + chase T^2 2.23 ns/row = 2.73 ns/row | ratio 0.733
m 204281: chase T 5.16 ns/row | gather T^2 0.62 ns/row + chase T^2 3.77 ns/row = 4.39 ns/row | ratio 0.850
"""
# the fourth-power program's lines: another program, refused
FOURTH = (
    "m 89874: chase T 3.02 ns/row | gather T^2,T^4 0.84 ns/row + chase T^4 1.02 ns/row"
    " = 1.86 ns/row | ratio 0.617\n"
)


def line(m, chase, gather, chase2, chk=None):
    """A pre-screen line as the program prints it, from its three times."""
    total = gather + chase2
    tail = "" if chk is None else f" (chk {chk})"
    return (
        f"m {m}: chase T {chase:.2f} ns/row | gather T^2 {gather:.2f} ns/row + chase T^2"
        f" {chase2:.2f} ns/row = {total:.2f} ns/row | ratio {total / chase:.3f}{tail}\n"
    )


def run_main(args):
    buf = io.StringIO()
    with redirect_stdout(buf):
        status = sv.main(args)
    return status, buf.getvalue()


class Source(unittest.TestCase):
    def test_source_is_monodromys_unchanged(self):
        data = open(SOURCE, "rb").read()
        self.assertEqual(hashlib.sha256(data).hexdigest(), SOURCE_SHA256)
        # the git blob id of the original, computed from the copy's bytes (needs no history)
        blob = hashlib.sha1(b"blob %d\0" % len(data) + data).hexdigest()
        self.assertEqual(blob, ORIGIN_BLOB)

    def test_source_matches_the_origin_commit_where_present(self):
        try:
            got = subprocess.run(
                ["git", "-C", HERE, "rev-parse", f"{ORIGIN_COMMIT}:{ORIGIN_PATH}"],
                capture_output=True,
                text=True,
            )
        except FileNotFoundError:
            self.skipTest("no git")
        if got.returncode != 0:
            self.skipTest(f"{ORIGIN_COMMIT} not in this clone")
        self.assertEqual(got.stdout.strip(), ORIGIN_BLOB)


class Parse(unittest.TestCase):
    def test_prescreen_lines(self):
        rs = sv.parse(PRESCREEN_FIRST + PRESCREEN_SECOND)
        self.assertEqual(len(rs), 9)
        self.assertEqual(rs[0].m, 32768)
        self.assertEqual(rs[4].ratio, 0.888)
        self.assertEqual(rs[-1].chase2, 3.77)

    def test_chk_tail(self):
        rs = sv.parse(line(89874, 3.05, 0.45, 1.38, chk=123456789))
        self.assertEqual(rs[0].ratio, 0.6)

    def test_blank_lines_skipped(self):
        self.assertEqual(len(sv.parse("\n" + PRESCREEN_SECOND + "\n\n")), 4)

    def test_other_program_refused(self):
        with self.assertRaises(sv.Invalid):
            sv.parse(FOURTH)

    def test_stray_line_refused(self):
        with self.assertRaises(sv.Invalid):
            sv.parse(PRESCREEN_SECOND + "Segmentation fault\n")

    def test_inconsistent_total_refused(self):
        bad = PRESCREEN_SECOND.replace("= 1.93 ns/row", "= 2.93 ns/row")
        with self.assertRaises(sv.Invalid):
            sv.parse(bad)

    def test_inconsistent_ratio_refused(self):
        bad = PRESCREEN_SECOND.replace("ratio 0.850", "ratio 0.950")
        with self.assertRaises(sv.Invalid):
            sv.parse(bad)


class Judge(unittest.TestCase):
    def test_prescreen_is_alive(self):
        v = sv.judge(sv.parse(PRESCREEN_FIRST + PRESCREEN_SECOND), expect=2)
        self.assertEqual(v["verdict"], "ALIVE")
        med = {r["m"]: r["median"] for r in v["rows"]}
        self.assertAlmostEqual(med[89874], (0.600 + 0.633) / 2)
        self.assertAlmostEqual(med[204281], (0.888 + 0.850) / 2)
        # m = 32768 is reported, outside the rule
        self.assertFalse([r for r in v["rows"] if r["m"] == 32768][0]["in_rule"])
        self.assertTrue(v["unanimous"])

    def test_dead_needs_every_m(self):
        text = "".join(line(m, 3.0, 0.6, 2.4) for m in sv.RULE_MS)  # ratio 1.000 everywhere
        self.assertEqual(sv.judge(sv.parse(text))["verdict"], "DEAD")
        text = "".join(line(m, 3.0, 0.6, 2.4 if m != 114000 else 2.0) for m in sv.RULE_MS)
        v = sv.judge(sv.parse(text))
        self.assertEqual(v["verdict"], "ALIVE")
        self.assertIn("[114000]", v["verdict_line"])

    def test_threshold_is_inclusive(self):
        # 0.57 + 2.28 = 2.85 over 3.00: exactly 0.950
        text = "".join(line(m, 3.00, 0.57, 2.28) for m in sv.RULE_MS)
        rs = sv.parse(text)
        self.assertEqual(rs[0].ratio, 0.95)
        self.assertEqual(sv.judge(rs)["verdict"], "DEAD")

    def test_median_decides_and_disagreement_is_noted(self):
        text = ""
        for m in sv.RULE_MS:
            # three invocations: 1.000, 1.000, 0.800 (the median 1.000, at or above)
            text += line(m, 3.0, 0.6, 2.4) * 2 + line(m, 3.0, 0.6, 1.8)
        v = sv.judge(sv.parse(text), expect=3)
        self.assertEqual(v["verdict"], "DEAD")
        self.assertFalse(v["unanimous"])
        self.assertIn("invocations disagree", sv.report(v))

    def test_missing_m_invalid(self):
        with self.assertRaises(sv.Invalid):
            sv.judge(sv.parse(PRESCREEN_FIRST.replace("m 156404", "m 156405")))

    def test_too_few_invocations_invalid(self):
        with self.assertRaises(sv.Invalid):
            sv.judge(sv.parse(PRESCREEN_FIRST + PRESCREEN_SECOND), expect=3)


class Main(unittest.TestCase):
    def test_main_verdict_and_json(self):
        with tempfile.TemporaryDirectory() as d:
            out = os.path.join(d, "out.txt")
            js = os.path.join(d, "verdict.json")
            open(out, "w").write(PRESCREEN_FIRST + PRESCREEN_SECOND)
            status, text = run_main([out, "--expect", "2", "--json", js])
            self.assertEqual(status, 0)
            self.assertTrue(text.rstrip().splitlines()[-1].startswith("VERDICT: ALIVE"))
            self.assertEqual(json.load(open(js))["verdict"], "ALIVE")

    def test_main_invalid(self):
        with tempfile.TemporaryDirectory() as d:
            out = os.path.join(d, "out.txt")
            open(out, "w").write(FOURTH)
            status, text = run_main([out])
            self.assertEqual(status, 1)
            self.assertTrue(text.startswith("INVALID: "))


if __name__ == "__main__":
    unittest.main()
