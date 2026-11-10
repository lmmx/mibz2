"""bench/team_ports_adjudicate.py's tests: §8a on every §12a case, and its controls.

docs/journal/2026-11-10-team-ports-combined-registration.md §8a, §12a. Synthetic ledgers
and simulated outcomes only: no decoder. The real cases run the pinned checker blob
eb7c392d2d766e870de3e14a31b5fe940787dd3f, copied out of this repository's object store; a
shallow clone must fetch it first (`git fetch --depth=1 origin 88768ac15924b08ceb039169690c942542791f7e`).

    python3 bench/test_team_ports_adjudicate.py
"""

import contextlib
import io
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import team_ports_adjudicate as adj  # noqa: E402
import team_ports_dry_run as dry  # noqa: E402

BALANCED = (
    "VALID\n"
    "  (a) holds: every helper walked every block it built; 0 zero-work helper calls\n"
    "  (b) holds: team 0: helper bytes [100, 100], largest share 0.5\n"
    "  (c) holds: 1 leader calls of traverse\n"
    "  (d) holds: 0 calls of another walker\n"
    "ENGAGED\n"
).encode()
BALANCE_ONLY = (
    "VALID\n"
    "  (a) holds: every helper walked every block it built; 0 zero-work helper calls\n"
    "  (b) fails: team 0: helper bytes [1, 99], largest share 0.99\n"
    "  (c) holds: 1 leader calls of traverse\n"
    "  (d) holds: 0 calls of another walker\n"
    "NOT ENGAGED\n"
).encode()


def decide(code, out, err=b"", timed_out=False, sig=None):
    d = adj.adjudicate(code, out, err, timed_out, sig)
    return d["decision"], d["rule"]


class Rules(unittest.TestCase):
    """§8a on hand-written outcomes (the dry run below runs the checker itself)"""

    def test_accepts(self):
        self.assertEqual(decide(0, BALANCED), ("accepted", 3))
        self.assertEqual(decide(1, BALANCE_ONLY), ("accepted", 3))

    def test_outcome_and_exit_reported_never_relabelled(self):
        d = adj.adjudicate(1, BALANCE_ONLY, b"")
        rec = adj.record(
            "E1", dict(exit=1, stdout=BALANCE_ONLY, stderr=b"", timed_out=False, signal=None), d
        )
        self.assertEqual(rec["checker_outcome"], "NOT ENGAGED")
        self.assertEqual(rec["checker_exit"], 1)
        self.assertEqual(rec["decision"], "accepted")
        self.assertFalse(rec["reported"]["b"]["holds"])
        self.assertIn("claim limit", rec["claim_limit"])

    def test_rule1_status(self):
        for code, kw in [
            (2, {}),
            (3, {}),
            (124, {}),
            (137, {}),
            (-9, {}),
            (-15, {}),
            (0, dict(timed_out=True)),
            (1, dict(timed_out=True)),
            (0, dict(sig=9)),
        ]:
            with self.subTest(code=code, **kw):
                self.assertEqual(decide(code, BALANCED, **kw), ("rejected", 1))

    def test_rule2_shape(self):
        lines = BALANCE_ONLY.decode().split("\n")[:-1]
        cases = {
            "stderr": (1, BALANCE_ONLY, b"x"),
            "stderr newline only": (1, BALANCE_ONLY, b"\n"),
            "no trailing newline": (1, BALANCE_ONLY[:-1], b""),
            "empty": (1, b"", b""),
            "blank line": (1, BALANCE_ONLY + b"\n", b""),
            "crlf": (1, BALANCE_ONLY.replace(b"\n", b"\r\n"), b""),
            "not utf-8": (1, b"\xff" + BALANCE_ONLY, b""),
            "leading space": (1, b" " + BALANCE_ONLY, b""),
            "INVALID": (1, b"INVALID\n  no single terminal record at the end\n", b""),
            "wrong letter": (1, BALANCE_ONLY.replace(b"(c) holds", b"(e) holds"), b""),
            "empty detail": (
                1,
                BALANCE_ONLY.replace(b"(d) holds: 0 calls of another walker", b"(d) holds: "),
                b"",
            ),
            "verb": (1, BALANCE_ONLY.replace(b"(a) holds", b"(a) held"), b""),
            "indent": (1, BALANCE_ONLY.replace(b"  (a)", b" (a)"), b""),
            "duplicate": (1, ("\n".join(lines[:2] + lines[1:]) + "\n").encode(), b""),
            "missing (d)": (1, ("\n".join(lines[:4] + lines[5:]) + "\n").encode(), b""),
            "outcome lower": (1, BALANCE_ONLY.replace(b"NOT ENGAGED", b"not engaged"), b""),
            "reference": (
                0,
                BALANCED.replace(b"ENGAGED\n", b"reference (the old arm): no outcome\n"),
                b"",
            ),
            "outcome contradicts": (0, BALANCE_ONLY.replace(b"NOT ENGAGED", b"ENGAGED"), b""),
            "all hold but NOT ENGAGED": (1, BALANCED.replace(b"ENGAGED", b"NOT ENGAGED"), b""),
            "exit 1 for ENGAGED": (1, BALANCED, b""),
            "exit 0 for NOT ENGAGED": (0, BALANCE_ONLY, b""),
        }
        for name, (code, out, err) in cases.items():
            with self.subTest(name):
                self.assertEqual(decide(code, out, err), ("rejected", 2))

    def test_rule3_required(self):
        for k in "acd":
            out = BALANCE_ONLY.replace(f"({k}) holds".encode(), f"({k}) fails".encode())
            with self.subTest(k):
                self.assertEqual(decide(1, out), ("rejected", 3))

    def test_command(self):
        cmd = adj.checker_command("walk_stats_check.py", "e1.jsonl", "wm-classes.txt")
        self.assertEqual(
            cmd,
            [
                "python3",
                "walk_stats_check.py",
                "e1.jsonl",
                "--walker",
                "traverse",
                "--helpers",
                "2",
                "--classes",
                "wm-classes.txt",
            ],
        )
        src = (HERE / "team_ports_adjudicate.py").read_text()
        self.assertNotIn("|| true", src)
        self.assertNotIn('"--reference"', src)


class Identity(unittest.TestCase):
    """§2: the copy is the pinned blob, or the checker is not run"""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def test_missing_blob(self):
        _, rec = adj.materialise_checker(self.tmp, HERE.parent, blob="0" * 40)
        self.assertFalse(rec["matches"])

    def test_tampered_copy_not_run(self):
        path, rec = adj.materialise_checker(self.tmp, HERE.parent)
        self.assertTrue(rec["matches"], rec)
        path.write_text(path.read_text() + "\n# tampered\n")
        (self.tmp / "r.jsonl").write_text(dry.ledger(dry.team(100, 100, 100)))
        (self.tmp / "c.txt").write_text("T\n")
        out = adj.run_step(self.tmp, "E1", "r.jsonl", "c.txt")
        self.assertEqual((out["decision"], out["rule"]), ("rejected", 0))
        self.assertIsNone(out["command"])
        self.assertEqual((self.tmp / "E1" / "checker-stdout.txt").read_bytes(), b"")
        with contextlib.redirect_stdout(io.StringIO()):
            code = adj.main(
                [
                    "run",
                    "--dir",
                    str(self.tmp),
                    "--step",
                    "E2",
                    "--records",
                    "r.jsonl",
                    "--classes",
                    "c.txt",
                ]
            )
        self.assertEqual(code, 2)


class DryRun(unittest.TestCase):
    """Every §12a case, the Researcher's and the Guardian's controls: expected = actual"""

    @classmethod
    def setUpClass(cls):
        cls.tmp = Path(tempfile.mkdtemp())
        cls.summary = dry.dry_run(cls.tmp / "out")

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmp)

    def test_every_case(self):
        names = [c[0] for c in dry.real_cases()] + [c[0] for c in dry.simulated_cases()]
        self.assertEqual([r["case"] for r in self.summary["cases"]], names)
        for r in self.summary["cases"]:
            with self.subTest(r["case"]):
                self.assertTrue(r["match"], r)
        self.assertTrue(self.summary["all_match"])

    def test_checker_side(self):
        by = {r["case"]: r for r in self.summary["cases"]}
        self.assertEqual(
            (by["balanced"]["checker_exit"], by["balanced"]["checker_outcome"]), (0, "ENGAGED")
        )
        for k in ("balance-only-1-99", "zero-work-0-0", "a-fails", "c-fails", "d-fails"):
            self.assertEqual(
                (by[k]["checker_exit"], by[k]["checker_outcome"]), (1, "NOT ENGAGED"), k
            )
        self.assertEqual(by["zero-work-0-0"]["zero_work_helper_calls"], 2)
        self.assertEqual(by["invalid-no-terminal"]["checker_exit"], 2)
        self.assertTrue(by["timeout"]["timed_out"])
        self.assertEqual(by["signal-kill"]["signal"], 9)
        self.assertEqual(by["signal-term"]["signal"], 15)
        crash = self.tmp / "out" / "cases" / "crash-traceback-exit1"
        self.assertEqual(by["crash-traceback-exit1"]["checker_exit"], 1)
        self.assertIn(b"Traceback", (crash / "checker-stderr.txt").read_bytes())

    def test_archive(self):
        for r in self.summary["cases"]:
            d = self.tmp / "out" / "cases" / r["case"]
            rec = json.loads((d / "decision.json").read_text())
            self.assertEqual(
                (rec["decision"], rec["rule"]), (r["actual"]["decision"], r["actual"]["rule"])
            )
            for f in ("checker-stdout.txt", "checker-stderr.txt", "checker-exit.txt"):
                self.assertTrue((d / f).is_file(), (r["case"], f))
            self.assertEqual(
                adj.sha256((d / "checker-stdout.txt").read_bytes()), rec["stdout"]["sha256"]
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
