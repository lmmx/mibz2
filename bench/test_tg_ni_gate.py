"""bench/tg_ni_gate.py: the target-gated noninferiority workflow's verdict gate (D078). A
qualification dispatch passes its job only as an admitted dispatch, never a verdict; the replicated
programme's verdict (registration §2.11) passes only if the judge's is a valid pass and the
gate's own recomputation of the dispatch-level bound agrees; a smoke run passes only when the
judge refused it as smoke, and says "smoke: not evidence". Also: the workflow
calls the gate last in `aggregate`, after the upload, without masking it, and every external
action in it is pinned to a full commit SHA.

    python3 bench/test_tg_ni_gate.py
"""

import copy
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import tg_ni_gate as g  # noqa: E402

WORKFLOW = HERE.parent / ".github" / "workflows" / "target-gated-ni.yml"


def cells(shift=None):
    """The 21 cells' wall and CPU dispatch estimates as `replicated` writes them: 12 small
    alternating deviations around 0, with `shift` = (cell, metric, amount) added to one"""
    out = {}
    for key in g.CELLS:
        out[key] = {}
        for metric in g.METRICS:
            ds = [0.002 * (-1) ** i for i in range(g.K)]
            if shift and shift[:2] == (key, metric):
                ds = [d + shift[2] for d in ds]
            out[key][metric] = {"d": ds}
    return out


RUNS = list(range(5000, 5000 + g.K))
PASS = {"schema": "mibz2.tg_ni_replicated/1", "valid": True, "passes": True, "failing": [], "rss_regression": [],
        "problems": [], "runs": RUNS, "cells": cells()}
NOT_SHOWN = dict(PASS, passes=False, failing=["pattern/2 wall upper 1.0310"], cells=cells(("pattern/2", "wall", 0.03)))
INVALID = {"schema": "mibz2.tg_ni_replicated/1", "valid": False, "passes": False,
           "problems": ["11 dispatches, not the registered 12"]}
DISPATCH = {"schema": "mibz2.tg_ni_dispatch/1", "valid": True, "problems": [], "run": 5000, "cells": {}}
DISPATCH_INVALID = dict(DISPATCH, valid=False, problems=["random: shard.json rounds 239, not 240"])
SMOKE = {"valid": False, "passes": False,
         "problems": ["the run is labelled 'smoke': not qualification (a smoke check is never evidence)"]}
STRUCT = {"schema": "mibz2.tg_ni_structure/1", "valid": True, "label": "smoke", "rounds": 2, "problems": [],
          "shards": ["onebyte", "pattern", "random", "records", "runs", "text", "wikidata-1", "wikidata-2", "wikidata-4"]}
EMPTY = dict(STRUCT, valid=False, shards=[], problems=["onebyte: no job in the run's history"])


class Gate(unittest.TestCase):
    def test_a_qualification_dispatch_is_never_a_verdict(self):
        status, line = g.gate("qualification", DISPATCH, 0)
        self.assertEqual(status, 0, line)
        self.assertIn("not a verdict", line)
        for verdict, code in ((DISPATCH_INVALID, 2), (None, 2), (None, 0), (DISPATCH, 2), (DISPATCH_INVALID, 0),
                              (SMOKE, 2), (PASS, 0)):
            with self.subTest(verdict=verdict, code=code):
                status, line = g.gate("qualification", verdict, code)
                self.assertEqual(status, 1, line)
        self.assertIn("INVALID", g.gate("qualification", DISPATCH_INVALID, 2)[1])

    def test_replicated(self):
        self.assertEqual(g.gate("replicated", PASS, 0)[0], 0)
        self.assertIn("NOT SHOWN: pattern/2", g.gate("replicated", NOT_SHOWN, 1)[1])
        for verdict, code in ((NOT_SHOWN, 1), (INVALID, 2), (None, 2), (None, 0), (PASS, 1), (PASS, 2),
                              (NOT_SHOWN, 0), (INVALID, 0), (DISPATCH, 0)):
            with self.subTest(verdict=verdict, code=code):
                status, line = g.gate("replicated", verdict, code)
                self.assertEqual(status, 1, line)
        self.assertIn("INVALID", g.gate("replicated", INVALID, 2)[1])

    def test_the_bound_is_recomputed(self):
        # (a judge that passes a cell the estimates fail, or fails one they pass: disagreement)
        status, line = g.gate("replicated", dict(PASS, cells=cells(("text/4", "cpu", 0.03))), 0)
        self.assertEqual(status, 1)
        self.assertIn("disagrees", line)
        status, line = g.gate("replicated", dict(NOT_SHOWN, cells=cells()), 1)
        self.assertEqual(status, 1)
        self.assertIn("disagrees", line)
        # (just inside and just outside the margin, by the gate's own bound)
        for shift, fails in ((0.017, False), (0.019, True)):
            failing, problems = g.recompute(dict(PASS, cells=cells(("runs/1", "wall", shift))))
            self.assertEqual(problems, [])
            self.assertEqual(failing == ["runs/1 wall"], fails, (shift, failing))
        # (11 estimates, a non-finite one, a cell gone, 11 or repeated run ids: INVALID, never a pass)
        short = copy.deepcopy(PASS)
        short["cells"]["onebyte/1"]["wall"]["d"] = short["cells"]["onebyte/1"]["wall"]["d"][:11]
        nan = copy.deepcopy(PASS)
        nan["cells"]["onebyte/1"]["cpu"]["d"][3] = float("nan")
        gone = copy.deepcopy(PASS)
        del gone["cells"]["wikidata/4"]
        for verdict in (short, nan, gone, dict(PASS, runs=RUNS[:11]), dict(PASS, runs=RUNS[:11] + [RUNS[0]]),
                        dict(PASS, runs=RUNS[:11] + [True]), dict(PASS, cells={}), dict(PASS, cells=[]),
                        dict(PASS, cells={"onebyte/1": {"wall": 3}})):
            status, line = g.gate("replicated", verdict, 0)
            self.assertEqual(status, 1, line)
            self.assertIn("INVALID", line)

    def test_smoke(self):
        status, line = g.gate("smoke", SMOKE, 2, STRUCT)
        self.assertEqual(status, 0)
        self.assertIn("smoke: not evidence", line)
        self.assertIn("structurally admitted: all 9 shards", line)
        for verdict, code, structure in ((None, 2, STRUCT), (PASS, 0, STRUCT), (INVALID, 2, STRUCT),
                                         (SMOKE, 0, STRUCT), (SMOKE, 1, STRUCT),
                                         # (the Researcher's §4: the refusal alone, with nothing behind it)
                                         (SMOKE, 2, None), (SMOKE, 2, EMPTY),
                                         (SMOKE, 2, dict(STRUCT, label="pilot"))):
            with self.subTest(verdict=verdict, code=code, structure=structure):
                status, line = g.gate("smoke", verdict, code, structure)
                self.assertEqual(status, 1, line)
                self.assertIn("smoke: not evidence", line)

    def test_unknown_label(self):
        self.assertEqual(g.gate("pilot", PASS, 0)[0], 1)
        self.assertEqual(g.gate("final", PASS, 0)[0], 1)

    def test_the_command_line(self):
        with tempfile.TemporaryDirectory() as d:
            v, summary = Path(d) / "ni.json", Path(d) / "summary.md"
            v.write_text(json.dumps(NOT_SHOWN))
            r = subprocess.run([sys.executable, str(HERE / "tg_ni_gate.py"), "--label", "replicated",
                                "--verdict", str(v), "--exit", "1", "--summary", str(summary)],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
            self.assertIn("NOT SHOWN", summary.read_text())
            v.write_text(json.dumps(SMOKE))
            st = Path(d) / "structure.json"
            st.write_text(json.dumps(STRUCT))
            r = subprocess.run([sys.executable, str(HERE / "tg_ni_gate.py"), "--label", "smoke",
                                "--verdict", str(v), "--exit", "2", "--structure", str(st), "--summary", str(summary)],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
            self.assertIn("smoke: not evidence", summary.read_text())
            r = subprocess.run([sys.executable, str(HERE / "tg_ni_gate.py"), "--label", "smoke",
                                "--verdict", str(v), "--exit", "2"], capture_output=True, text=True)
            self.assertEqual(r.returncode, 1, "a smoke with no structural admission")
            r = subprocess.run([sys.executable, str(HERE / "tg_ni_gate.py"), "--label", "qualification",
                                "--verdict", str(Path(d) / "missing.json"), "--exit", "0"],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 1)

    def test_the_judges_refusal_is_what_the_gate_expects(self):
        # (the negative qualification-label test: the judge's own smoke refusal text, so a
        # reworded judge can't silently break the gate. The refusal is recognised; alone, with
        # an empty run behind it, it is not plumbing OK)
        import service_tg as tg
        root = tempfile.mkdtemp()
        man = {"schema": "mibz2.tg_ni_manifest/2", "label": "smoke", "run": 1, "artifacts": [], "jobs": []}
        v = tg.ni_judge(root, man, "f" * 40, reg={"frozen": {}, "bench": None}, check_git=False)
        self.assertFalse(v["valid"])
        self.assertEqual(g.gate("smoke", v, 2, STRUCT)[0], 0, v["problems"])
        empty = tg.structure(root, man, "f" * 40, "b" * 40, 2, "smoke", "1.99.1")
        self.assertFalse(empty["valid"])
        self.assertEqual(g.gate("smoke", v, 2, empty)[0], 1)


class Workflow(unittest.TestCase):
    def setUp(self):
        self.text = WORKFLOW.read_text()

    def aggregate(self):
        return self.text[self.text.index("\n  aggregate:"):]

    def test_no_verdict_is_masked(self):
        agg = self.aggregate()
        self.assertNotRegex(agg, r"service_tg\.py ni[^\n]*\|\| true")
        self.assertIn("bench/tg_ni_gate.py", agg)
        self.assertIn("service_tg.py structure", agg)
        self.assertIn("--structure structure.json", agg)
        gate_at = agg.index("bench/tg_ni_gate.py")
        upload_at = agg.index("actions/upload-artifact@")
        self.assertLess(upload_at, gate_at, "the evidence is uploaded before the gate can fail the job")
        upload = agg[agg.rindex("- name:", 0, upload_at):gate_at]
        self.assertIn("if: always()", upload)
        gate_line = [l for l in agg.splitlines() if "tg_ni_gate.py" in l][0]
        self.assertNotIn("|| true", gate_line)

    def test_every_action_is_pinned_to_a_commit(self):
        uses = re.findall(r"^\s*-?\s*uses:\s*(\S+)(.*)$", self.text, re.M)
        self.assertTrue(uses)
        for ref, comment in uses:
            with self.subTest(ref):
                self.assertRegex(ref, r"^[\w.-]+/[\w.-]+@[0-9a-f]{40}$")
                self.assertRegex(comment, r"#\s*\S+", "a comment naming the tag or branch it was resolved from")

    def test_cargo_builds_are_locked(self):
        builds = re.findall(r"cargo build[^\n]*", self.text)
        self.assertTrue(builds)
        for b in builds:
            self.assertIn("--locked", b)

    def test_the_final_designs_ceiling_and_rounds(self):
        # (registration §2.9: records 300 minutes, every other shard 240; a qualification refused
        # dispatch at any R but 240)
        shard = self.text[self.text.index("\n  shard:"):self.text.index("\n  aggregate:")]
        self.assertEqual(re.findall(r"timeout-minutes:\s*(.+)", shard), ["${{ matrix.shard.timeout }}"])
        # (the plan's shard list, run as the plan job runs it: records 300, every other 240)
        plan_py = re.search(r"pick=\$\(python3 - <<'EOF'\n(.*?)\n\s*EOF\n", self.text, re.S).group(1)
        import os
        import textwrap
        out = subprocess.run([sys.executable, "-c", textwrap.dedent(plan_py)], capture_output=True, text=True,
                             env=dict(os.environ, SHARDS="all", LABEL="qualification"))
        self.assertEqual(out.returncode, 0, out.stderr)
        shards = json.loads(out.stdout)
        self.assertEqual({s["id"]: s["timeout"] for s in shards},
                         {"onebyte": 240, "pattern": 240, "random": 240, "records": 300, "runs": 240, "text": 240,
                          "wikidata-1": 240, "wikidata-2": 240, "wikidata-4": 240})
        self.assertEqual(sum(s["timeout"] for s in shards), 2220)
        import service_tg as tg
        self.assertEqual(sorted(s["id"] for s in shards), sorted(tg.SHARDS))
        plan = self.text[self.text.index("\n  plan:"):self.text.index("\n  shard:")]
        self.assertIn('[ "$LABEL" = qualification ] && [ "$ROUNDS" != 240 ]', plan)
        import service_tg as tg
        self.assertEqual(tg.NI_ROUNDS, 240)
        self.assertIn('default: "240"', self.text)

    def test_dispatch_only(self):
        on = self.text[self.text.index("\non:"):self.text.index("\npermissions:")]
        self.assertIn("workflow_dispatch:", on)
        for trigger in ("push:", "pull_request:", "schedule:"):
            self.assertNotIn(trigger, on)


if __name__ == "__main__":
    unittest.main(verbosity=2)
