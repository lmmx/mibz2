"""bench/svc_target_check.py on its witnesses: the target's record and a 4-core machine's are
read as they are; each malformation, contradiction or fail-open case is refused (the
Researcher's R2 witnesses among them: ok=true beside an unknown quota, an absent snapshot, no
observations).

    python3 bench/test_svc_target_check.py
"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from svc_target_check import check  # noqa: E402
from svc_target_fixtures import edit, record  # noqa: E402


class Check(unittest.TestCase):
    def test_the_target(self):
        r, x = record(True)
        self.assertEqual(check(r, x), ("TARGET", []))

    def test_not_the_target(self):
        r, x = record(False)
        self.assertEqual(check(r, x)[0], "NOT TARGET")

    def test_unknown_quota_is_never_target(self):
        r, _ = record(True)
        unknown = edit(r, lambda r: r["machine"].__setitem__("quota", {"kind": "unknown"}))
        # as the binary must report it: not ok, the predicate false, no service core
        honest = edit(unknown, lambda r: (
            r.__setitem__("ok", False), r["why"].append("quota unknown"),
            r["at"][1].__setitem__("predicate", False),
            r["at"][1]["built"].update(service=False, pipelined=True, threads=9, window=12, counts=None)))
        self.assertEqual(check(honest, 3)[0], "NOT TARGET")
        # ok=true beside it (the R2 witness): refused
        self.assertEqual(check(unknown, 0)[0], "INVALID RECORD")

    def test_fail_open_witnesses(self):
        r, _ = record(True)
        cases = {
            "target absent, ok": edit(r, lambda r: r["machine"].__setitem__("target", None)),
            "only ok": {"ok": True},
            "no observations": edit(r, lambda r: r.__setitem__("at", [])),
            "a quota below 10, ok": edit(r, lambda r: r["machine"].__setitem__("quota", {"kind": "cpus", "cpus": 8})),
            "another model, ok": edit(r, lambda r: r["machine"]["target"]["cpu"].__setitem__("model", 166)),
            "scalar kernels, ok": edit(r, lambda r: r["kernels"].__setitem__("avx2", False)),
        }
        for what, bad in cases.items():
            with self.subTest(what):
                self.assertEqual(check(bad, 0)[0], "INVALID RECORD")

    def test_malformed(self):
        r, x = record(True)
        cases = {
            "schema": edit(r, lambda r: r.__setitem__("schema", "mibz2.svc_target/0")),
            "ok as 1": edit(r, lambda r: r.__setitem__("ok", 1)),
            "predicate as 0": edit(r, lambda r: r["at"][0].__setitem__("predicate", 0)),
            "threads as a string": edit(r, lambda r: r["at"][1]["built"].__setitem__("threads", "8")),
            "a budget missing": edit(r, lambda r: r["at"].pop()),
            "budgets reordered": edit(r, lambda r: r["at"].reverse()),
            "quota kind": edit(r, lambda r: r["machine"]["quota"].__setitem__("kind", "maybe")),
            "service plan": edit(r, lambda r: r["service_plan"].__setitem__("services", 1)),
            "machine missing": edit(r, lambda r: r.pop("machine")),
        }
        for what, bad in cases.items():
            with self.subTest(what):
                self.assertEqual(check(bad, x)[0], "INVALID RECORD")
        self.assertEqual(check(r, None)[0], "INVALID RECORD")

    def test_contradictions(self):
        r, x = record(True)
        cases = {
            "exit 3 with ok": (r, 3),
            "exit 101": (r, 101),
            "predicate at 9": (edit(r, lambda r: r["at"][0].__setitem__("predicate", True)), 0),
            "service at 11": (edit(r, lambda r: r["at"][2]["built"].__setitem__("service", True)), 0),
            "ok with reasons": (edit(r, lambda r: r["why"].append("x")), 0),
        }
        n, nx = record(False)
        cases["service core off the target"] = (edit(n, lambda r: r["at"][1]["built"].__setitem__("service", True)), nx)
        cases["predicate true off the target"] = (edit(n, lambda r: r["at"][1].__setitem__("predicate", True)), nx)
        cases["not ok, no reason"] = (edit(n, lambda r: r.__setitem__("why", [])), nx)
        for what, (bad, ex) in cases.items():
            with self.subTest(what):
                self.assertEqual(check(bad, ex)[0], "INVALID RECORD")

    def test_failed_dispatch_on_the_target(self):
        r, _ = record(True)
        for what, f in {
            "window 12 at 10": lambda r: r["at"][1]["built"].__setitem__("window", 12),
            "not ended": lambda r: r["at"][1]["built"]["counts"].__setitem__("ended", False),
            "a bad decode": lambda r: r["at"][0].__setitem__("decoded", False),
            "two decoders": lambda r: r["at"][2].__setitem__("builds", 2),
        }.items():
            with self.subTest(what):
                bad = edit(r, lambda r: (f(r), r.__setitem__("ok", False), r["why"].append(what)))
                self.assertEqual(check(bad, 3)[0], "FAILED")
                # and with ok kept true: refused
                self.assertEqual(check(edit(r, f), 0)[0], "INVALID RECORD")


if __name__ == "__main__":
    unittest.main(verbosity=2)
