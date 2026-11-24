"""bench/svc_target_check.py on its witnesses: the target's record and a 4-core machine's are
read as they are; each malformation, contradiction or fail-open case is refused (the
Researcher's R2 witnesses among them: ok=true beside an unknown quota, an absent snapshot, no
observations).

    python3 bench/test_svc_target_check.py
"""

import json
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

    def test_the_narrowed_predicate_reported_honestly_is_not_target(self):
        # (another 10-core Comet Lake-S, and the i9 under any quota, as the binary must
        # report them: not ok, the predicate false, the pipelined default built at 10)
        r, _ = record(True)
        for what, f in (
            ("i9-10900", lambda r: r["machine"]["target"]["cpu"].__setitem__(
                "name", "Intel(R) Core(TM) i9-10900 CPU @ 2.80GHz")),
            ("quota 20", lambda r: r["machine"].__setitem__("quota", {"kind": "cpus", "cpus": 20})),
        ):
            with self.subTest(what):
                honest = edit(r, lambda r: (
                    f(r), r.__setitem__("ok", False), r["why"].append("10 threads: the predicate is false"),
                    r["at"][1].__setitem__("predicate", False),
                    r["at"][1]["built"].update(service=False, pipelined=True, threads=9, window=12, counts=None)))
                self.assertEqual(check(honest, 3), ("NOT TARGET", []))

    def test_fail_open_witnesses(self):
        r, _ = record(True)
        cases = {
            "target absent, ok": edit(r, lambda r: r["machine"].__setitem__("target", None)),
            "only ok": {"ok": True},
            "no observations": edit(r, lambda r: r.__setitem__("at", [])),
            "a quota below 10, ok": edit(r, lambda r: r["machine"].__setitem__("quota", {"kind": "cpus", "cpus": 8})),
            "another model, ok": edit(r, lambda r: r["machine"]["target"]["cpu"].__setitem__("model", 166)),
            # (the narrowed predicate, schema 2: the measured SKU's name, and unlimited alone)
            "another 10-core Comet Lake-S, ok": edit(r, lambda r: r["machine"]["target"]["cpu"].__setitem__(
                "name", "Intel(R) Core(TM) i9-10900KF CPU @ 3.70GHz")),
            "a quota of 20, ok": edit(r, lambda r: r["machine"].__setitem__("quota", {"kind": "cpus", "cpus": 20})),
            "a quota of 10, ok": edit(r, lambda r: r["machine"].__setitem__("quota", {"kind": "cpus", "cpus": 10})),
            "scalar kernels, ok": edit(r, lambda r: r["kernels"].__setitem__("avx2", False)),
        }
        for what, bad in cases.items():
            with self.subTest(what):
                self.assertEqual(check(bad, 0)[0], "INVALID RECORD")

    def test_malformed(self):
        r, x = record(True)
        cases = {
            "schema": edit(r, lambda r: r.__setitem__("schema", "mibz2.svc_target/0")),
            "schema 1 (the earlier predicate's)": edit(r, lambda r: r.__setitem__("schema", "mibz2.svc_target/1")),
            "no name": edit(r, lambda r: r["machine"]["target"]["cpu"].pop("name")),
            "a name not a string": edit(r, lambda r: r["machine"]["target"]["cpu"].__setitem__("name", 10900)),
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

    def test_the_researchers_strictness_witnesses(self):
        # REVIEW 6743780326 (probe.py at 05979b91): each returned TARGET at 7f691929
        r, x = record(True)
        inf = edit(r, lambda r: r["machine"].update(quota={"kind": "cpus", "cpus": float("inf")}))
        cases = {
            "an infinite quota": inf,
            "the same through 1e309": json.loads(json.dumps(inf).replace("Infinity", "1e309")),
            "an incomplete plan": edit(r, lambda r: r["at"][0].update(plan={"kind": "teams"})),
            "a float budget": edit(r, lambda r: r["at"][0].update(threads=9.0)),
            "a float service-plan teams": edit(r, lambda r: r["service_plan"].update(teams=2.0)),
            "a negative pooled count": edit(r, lambda r: r["at"][1]["built"]["counts"].update(pooled=-1)),
            "a NaN quota": edit(r, lambda r: r["machine"].update(quota={"kind": "cpus", "cpus": float("nan")})),
            "a float window": edit(r, lambda r: r["at"][1]["built"].update(window=8.0)),
            "a zero thread count": edit(r, lambda r: r["at"][0]["built"].update(threads=0)),
            "a workers plan with no count": edit(r, lambda r: r["at"][0].update(plan={"kind": "workers"})),
            "a plan of another kind": edit(r, lambda r: r["at"][0].update(plan={"kind": "service", "teams": 2, "k": 3})),
            "a plan with an extra field": edit(r, lambda r: r["at"][0]["plan"].update(extra=1)),
            "a float k": edit(r, lambda r: r["at"][0]["plan"].update(k=2.0)),
            "an unknown quota with a count": edit(r, lambda r: r["machine"].update(quota={"kind": "unknown", "cpus": 20})),
            "a bool builds": edit(r, lambda r: r["at"][0].update(builds=True)),
        }
        for what, bad in cases.items():
            with self.subTest(what):
                self.assertEqual(check(bad, x)[0], "INVALID RECORD")
        # the retained controls
        self.assertEqual(check(r, x), ("TARGET", []))
        self.assertEqual(check(edit(r, lambda r: r["at"][0].pop("plan")), x)[0], "INVALID RECORD")
        self.assertEqual(check(edit(r, lambda r: r["machine"].update(quota={"kind": "unknown"})), x)[0], "INVALID RECORD")
        n, nx = record(False)
        self.assertEqual(check(n, nx)[0], "NOT TARGET")

    def test_a_failed_fallback_decode_is_never_not_target(self):
        # REVIEW 6743780326: off the target, a decode failure is FAILED (its own outcome), not
        # NOT TARGET with the defect in its problems
        n, nx = record(False)
        bad = edit(n, lambda r: (r["at"][0].update(decoded=False), r["why"].append("decode mismatch")))
        verdict, problems = check(bad, nx)
        self.assertEqual(verdict, "FAILED")
        self.assertTrue(any("did not give back" in p for p in problems))
        two = edit(n, lambda r: r["at"][2].update(builds=2))
        self.assertEqual(check(two, nx)[0], "FAILED")

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
