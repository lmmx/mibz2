"""bench/tg_ni_gate.py: the target-gated noninferiority workflow's verdict gate (D078). A
qualification run fails its job unless the judge's verdict is a valid pass; a smoke run passes
only when the judge refused it as smoke, and says "smoke: not evidence". Also: the workflow
calls the gate last in `aggregate`, after the upload, without masking it, and every external
action in it is pinned to a full commit SHA.

    python3 bench/test_tg_ni_gate.py
"""

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
PASS = {"valid": True, "passes": True, "failing": [], "rss_regression": [], "problems": []}
NOT_SHOWN = {"valid": True, "passes": False, "failing": ["pattern/2 wall upper 1.0310"], "rss_regression": [],
             "problems": []}
INVALID = {"valid": False, "passes": False, "problems": ["random: shard.json rounds 599, not 600"]}
SMOKE = {"valid": False, "passes": False,
         "problems": ["the run is labelled 'smoke': not qualification (a smoke check is never evidence)"]}


class Gate(unittest.TestCase):
    def test_qualification(self):
        self.assertEqual(g.gate("qualification", PASS, 0)[0], 0)
        for verdict, code in ((NOT_SHOWN, 1), (INVALID, 2), (None, 2), (None, 0), (PASS, 1), (NOT_SHOWN, 0),
                              (INVALID, 0), (SMOKE, 2)):
            with self.subTest(verdict=verdict, code=code):
                status, line = g.gate("qualification", verdict, code)
                self.assertEqual(status, 1, line)
        self.assertIn("NOT SHOWN: pattern/2", g.gate("qualification", NOT_SHOWN, 1)[1])
        self.assertIn("INVALID", g.gate("qualification", INVALID, 2)[1])

    def test_smoke(self):
        status, line = g.gate("smoke", SMOKE, 2)
        self.assertEqual(status, 0)
        self.assertIn("smoke: not evidence", line)
        for verdict, code in ((None, 2), (PASS, 0), (INVALID, 2), (SMOKE, 0), (SMOKE, 1)):
            with self.subTest(verdict=verdict, code=code):
                status, line = g.gate("smoke", verdict, code)
                self.assertEqual(status, 1, line)
                self.assertIn("smoke: not evidence", line)

    def test_unknown_label(self):
        self.assertEqual(g.gate("pilot", PASS, 0)[0], 1)

    def test_the_command_line(self):
        with tempfile.TemporaryDirectory() as d:
            v, summary = Path(d) / "ni.json", Path(d) / "summary.md"
            v.write_text(json.dumps(NOT_SHOWN))
            r = subprocess.run([sys.executable, str(HERE / "tg_ni_gate.py"), "--label", "qualification",
                                "--verdict", str(v), "--exit", "1", "--summary", str(summary)],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
            self.assertIn("NOT SHOWN", summary.read_text())
            v.write_text(json.dumps(SMOKE))
            r = subprocess.run([sys.executable, str(HERE / "tg_ni_gate.py"), "--label", "smoke",
                                "--verdict", str(v), "--exit", "2", "--summary", str(summary)],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
            self.assertIn("smoke: not evidence", summary.read_text())
            r = subprocess.run([sys.executable, str(HERE / "tg_ni_gate.py"), "--label", "qualification",
                                "--verdict", str(Path(d) / "missing.json"), "--exit", "0"],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 1)

    def test_the_judges_refusal_is_what_the_gate_expects(self):
        # (the judge's own smoke refusal text, so a reworded judge can't silently break the gate)
        import service_tg as tg
        v = tg.ni_judge(tempfile.mkdtemp(), {"label": "smoke", "artifacts": []}, "f" * 40,
                        reg={"frozen": {}, "bench": None}, check_git=False)
        self.assertFalse(v["valid"])
        self.assertEqual(g.gate("smoke", v, 2)[0], 0, v["problems"])


class Workflow(unittest.TestCase):
    def setUp(self):
        self.text = WORKFLOW.read_text()

    def aggregate(self):
        return self.text[self.text.index("\n  aggregate:"):]

    def test_no_verdict_is_masked(self):
        agg = self.aggregate()
        self.assertNotRegex(agg, r"service_tg\.py ni[^\n]*\|\| true")
        self.assertIn("bench/tg_ni_gate.py", agg)
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

    def test_dispatch_only(self):
        on = self.text[self.text.index("\non:"):self.text.index("\npermissions:")]
        self.assertIn("workflow_dispatch:", on)
        for trigger in ("push:", "pull_request:", "schedule:"):
            self.assertNotIn(trigger, on)


if __name__ == "__main__":
    unittest.main(verbosity=2)

