"""bench/w12_stores.py's statistic, rule and validity on synthetic counter records: the lever
where the rule puts it, NOT ESTABLISHED where either event falls short or its interval
reaches 1, and every defect leaves a VOID verdict. Then `bench/counters.py run --events
stores` driven for real with stand-ins for perf and `ab child`, read through to a verdict.

    python3 bench/test_w12_stores.py
"""

import io
import json
import math
import os
import random
import stat
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import w12_stores  # noqa: E402

REF = "mibz2@/w/target/w12/build-master/target/release/examples/ab"
CAND = "mibz2-w12"
OUT = 4 * 209700196
BASE = {
    "cycles": 9e9,
    "instructions": 2e10,
    "l2_lines_out.non_silent": 3e8,
    "resource_stalls.sb": 4e8,
    "mem_inst_retired.all_stores": 2e9,
    "offcore_requests.all_data_rd": 5e8,
    "l2_trans.l2_wb": 3e8,
}


def records(ratios, events="stores", reps=10, noise=0.002, seed=5, drop=None, edit=None):
    """The registered run's records: each event's candidate count over the reference's at
    `ratios[event]` (1 where not given), with log-normal noise of `noise` a run"""
    rng = random.Random(seed)
    rows = []
    for rep in range(reps):
        arms = [REF, CAND] if rep % 2 == 0 else [CAND, REF]
        for arm in arms:
            for g, evs in enumerate(w12_stores.EVENT_SETS[events]):
                if (arm, rep, g) == drop:
                    continue
                counts = {}
                for e in evs:
                    v = BASE[e] * (ratios.get(e, 1.0) if arm == CAND else 1.0)
                    v *= math.exp(rng.gauss(0, noise))
                    counts[e] = [int(v), 100.0, str(int(v))]
                rec = {"arm": arm, "threads": 10, "rep": rep, "group": g, "events": evs, "rc": 0,
                       "cpus": "0,1,2,3,4,5,6,7,8,9", "out": OUT, "hash": "h", "wall": 1.0,
                       "child_err": "", "counts": counts}  # fmt: skip
                if edit:
                    edit(rec)
                rows.append(rec)
    return rows


def judge(rows, events="stores", reps=10):
    with tempfile.TemporaryDirectory() as d:
        p = Path(d) / "counters.jsonl"
        p.write_text("".join(json.dumps(r) + "\n" for r in rows))
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = w12_stores.main(["w12_stores.py", str(p), "--ref", REF, "--cand", CAND,
                                    "--threads", "10", "--reps", str(reps), "--events", events])  # fmt: skip
    out = buf.getvalue()
    verdicts = [line for line in out.splitlines() if line.startswith("VERDICT: ")]
    assert len(verdicts) == 1, out
    return code, verdicts[0], out


LEVER = {"l2_lines_out.non_silent": 0.95, "resource_stalls.sb": 0.96}


class Rule(unittest.TestCase):
    def test_lever(self):
        code, v, out = judge(records(LEVER))
        self.assertEqual(code, 0)
        self.assertEqual(v, "VERDICT: STORE TRAFFIC IS THE LEVER")
        self.assertIn("l2_lines_out.non_silent: 0.9496", out)

    def test_at_the_threshold(self):
        # 0.977 exactly (no noise) meets it
        r = {"l2_lines_out.non_silent": 0.977, "resource_stalls.sb": 0.977}
        self.assertEqual(judge(records(r, noise=0))[1], "VERDICT: STORE TRAFFIC IS THE LEVER")

    def test_stalls_short_of_the_threshold(self):
        r = {**LEVER, "resource_stalls.sb": 0.985}
        code, v, out = judge(records(r))
        self.assertEqual((code, v), (0, "VERDICT: NOT ESTABLISHED"))
        self.assertIn("resource_stalls.sb", out.split("The rule")[1])

    def test_write_back_short_of_the_threshold(self):
        r = {**LEVER, "l2_lines_out.non_silent": 1.02}
        self.assertEqual(judge(records(r))[1], "VERDICT: NOT ESTABLISHED")

    def test_interval_reaching_one(self):
        # both point estimates below 0.977, but too noisy for the interval to clear 1
        r = {"l2_lines_out.non_silent": 0.95, "resource_stalls.sb": 0.95}
        rows = records(r, noise=0.06, seed=4)
        code, v, out = judge(rows)
        self.assertEqual((code, v), (0, "VERDICT: NOT ESTABLISHED"), out)
        rule = out.split("The rule")[1]
        short = [ln for ln in rule.splitlines() if ln.endswith("does not")]
        self.assertEqual(len(short), 2, out)
        for ln in short:
            g, lo, hi = (float(x) for x in ln.split(": ")[1].replace("[", "").replace("]", "")
                         .replace(",", "").split()[:3])  # fmt: skip
            self.assertLessEqual(g, w12_stores.THRESHOLD)
            self.assertGreaterEqual(hi, 1.0)

    def test_substitute(self):
        r = {"l2_trans.l2_wb": 0.95, "resource_stalls.sb": 0.95}
        code, v, _ = judge(records(r, events="stores-alt"), events="stores-alt")
        self.assertEqual(code, 0)
        self.assertTrue(v.startswith("VERDICT: STORE TRAFFIC IS THE LEVER (l2_trans.l2_wb"), v)


def unsupported(rec):
    if rec["arm"] == CAND and rec["rep"] == 3:
        rec["counts"]["resource_stalls.sb"] = [None, 100.0, "<not supported>"]


def multiplexed(rec):
    if rec["rep"] == 0:
        rec["counts"]["mem_inst_retired.all_stores"][1] = 61.5


def failed(rec):
    if rec["arm"] == REF and rec["rep"] == 7:
        rec.update(rc=255, err="perf: failed")


def differ(rec):
    if rec["arm"] == CAND:
        rec["hash"] = "other"


def zero(rec):
    if rec["arm"] == REF and rec["rep"] == 2:
        rec["counts"]["l2_lines_out.non_silent"] = [0, 100.0, "0"]


def wrong_events(rec):
    rec["events"] = ["cycles", "instructions", "l2_rqsts.miss", "l2_rqsts.references"]


class Void(unittest.TestCase):
    def check(self, rows, why, **kw):
        code, v, out = judge(rows, **kw)
        self.assertEqual(code, 1, out)
        self.assertTrue(v.startswith("VERDICT: VOID"), v)
        self.assertIn(why, out)
        self.assertNotIn("The rule", out)

    def test_missing_record(self):
        self.check(records(LEVER, drop=(CAND, 4, 0)), "1 of 20 runs missing")

    def test_missing_file(self):
        code, v, _ = judge([])
        self.assertEqual((code, v[:13]), (1, "VERDICT: VOID"))

    def test_perf_unsupported(self):
        self.check(records(LEVER, edit=unsupported), "resource_stalls.sb not counted")

    def test_multiplexed(self):
        self.check(records(LEVER, edit=multiplexed), "multiplexed")

    def test_perf_failed(self):
        self.check(records(LEVER, edit=failed), "the run failed")

    def test_outputs_differ(self):
        self.check(records(LEVER, edit=differ), "outputs differ")

    def test_zero_count(self):
        self.check(records(LEVER, edit=zero), "counted zero")

    def test_other_events(self):
        self.check(records(LEVER, edit=wrong_events), "not the stores set's group")

    def test_duplicate(self):
        rows = records(LEVER)
        self.check(rows + rows[:1], "duplicated")

    def test_one_rep(self):
        self.check(records(LEVER, reps=1), "at least 2", reps=1)


# perf's stand-in: perf stat's CSV for the events asked, the candidate's 0.95 of the reference's
PERF = r"""#!/usr/bin/env python3
import os, subprocess, sys
a = sys.argv[1:]
out = a[a.index("-o") + 1]
events = a[a.index("-e") + 1].split(",")
cmd = a[a.index("--") + 1:]
if cmd[0] == "taskset":
    cmd = cmd[3:]
p = subprocess.run(cmd)
with open(out, "w") as f:
    for i, e in enumerate(events):
        v = int(1000000 * (i + 1) * (0.95 if "w12" in cmd[2] else 1))
        if os.environ.get("STUB_PERF") == "unsupported" and e == "l2_lines_out.non_silent":
            f.write(f"<not supported>,,{e},0,100.00,,\n")
        else:
            f.write(f"{v},,{e},1000,100.00,,\n")
sys.exit(p.returncode)
"""
CHILD = r"""#!/usr/bin/env python3
import json
print(json.dumps({"wall": 0.5, "out": 838800784, "hash": "h", "err": ""}))
"""


class EndToEnd(unittest.TestCase):
    def go(self, env):
        d = Path(tempfile.mkdtemp())
        for name, body in (("perf", PERF), ("ab", CHILD), ("ab-master", CHILD)):
            (d / name).write_text(body)
            (d / name).chmod((d / name).stat().st_mode | stat.S_IEXEC)
        out = d / "counters.jsonl"
        e = {**os.environ, "PATH": f"{d}:{os.environ['PATH']}", **env}
        ref = f"mibz2@{d / 'ab-master'}"
        subprocess.run([sys.executable, str(HERE / "counters.py"), "run", "--ab", str(d / "ab"),
                        "--arm", ref, "--arm", CAND, "--events", "stores", "--threads", "10",
                        "--reps", "3", "--file", "x.bz2", "--out", str(out)],
                       env=e, capture_output=True, check=True)  # fmt: skip
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = w12_stores.main(["w12_stores.py", str(out), "--ref", ref, "--cand", CAND,
                                    "--threads", "10", "--reps", "3"])  # fmt: skip
        return code, buf.getvalue()

    def test_counted(self):
        code, out = self.go({})
        self.assertEqual(code, 0, out)
        self.assertIn("VERDICT: STORE TRAFFIC IS THE LEVER", out)

    def test_unsupported(self):
        code, out = self.go({"STUB_PERF": "unsupported"})
        self.assertEqual(code, 1, out)
        self.assertIn("VERDICT: VOID", out)
        self.assertIn("l2_lines_out.non_silent not counted", out)


if __name__ == "__main__":
    unittest.main()
