"""bench/counters.py driven for real with stand-ins: a stub `perf` (writing perf stat's CSV,
counted, unsupported, not counted or multiplexed as told) and a stub `ab child`. The run's
records are checked, the report reads a complete run, and every defect leaves no report.

    python3 bench/test_counters.py
"""
import io
import json
import os
import stat
import subprocess
import sys
import tempfile
from contextlib import redirect_stdout
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import counters  # noqa: E402

PERF = r'''#!/usr/bin/env python3
import os, subprocess, sys
a = sys.argv[1:]
out = a[a.index("-o") + 1]
events = a[a.index("-e") + 1].split(",")
cmd = a[a.index("--") + 1:]
if cmd[0] == "taskset":
    cmd = cmd[3:]
p = subprocess.run(cmd)
mode = os.environ.get("STUB_PERF", "ok")
arm = cmd[2]
with open(out, "w") as f:
    for i, e in enumerate(events):
        v = 1000000 * (i + 1) * (2 if "stop" in arm else 1)
        if mode == "unsupported" and e == "ld_blocks.store_forward":
            f.write(f"<not supported>,,{e},0,100.00,,\n")
        elif mode == "notcounted" and e == "branch-misses":
            f.write(f"<not counted>,,{e},0,0.00,,\n")
        elif mode == "multiplexed" and e == "LLC-load-misses":
            f.write(f"{v},,{e},500,50.00,,\n")
        else:
            f.write(f"{v},,{e},1000,100.00,,\n")
sys.exit(p.returncode)
'''
CHILD = r'''#!/usr/bin/env python3
import json, os, sys
if os.environ.get("STUB_CHILD") == "fail" and "stop" in sys.argv[2]:
    sys.exit(3)
print(json.dumps({"wall": 0.5, "out": 209700196, "hash": "h", "err": ""}))
'''


def setup():
    d = Path(tempfile.mkdtemp())
    for name, body in (("perf", PERF), ("ab", CHILD)):
        p = d / name
        p.write_text(body)
        p.chmod(p.stat().st_mode | stat.S_IEXEC)
    return d


def go(d, env, reps=2, report_reps=None):
    out = d / "c.jsonl"
    e = {**os.environ, "PATH": f"{d}:{os.environ['PATH']}", **env}
    subprocess.run([sys.executable, str(HERE / "counters.py"), "run", "--ab", str(d / "ab"),
                    "--arm", "mibz2-workers-lazy", "--arm", "mibz2-workers-stop15", "--threads", "1",
                    "--reps", str(reps), "--file", "x.bz2", "--out", str(out)], env=e, capture_output=True, check=True)
    buf = io.StringIO()
    with redirect_stdout(buf):
        code = counters.main(["counters.py", "report", str(out), "--ref", "mibz2-workers-lazy",
                              "--cand", "mibz2-workers-stop15", "--threads", "1", "--reps", str(report_reps or reps)])
    return code, buf.getvalue()


def main():
    d = setup()
    cases = [
        ("a complete run", {}, 0, "IPC", None),
        ("an unsupported event", {"STUB_PERF": "unsupported"}, 1, "ld_blocks.store_forward not counted", None),
        ("an event not counted", {"STUB_PERF": "notcounted"}, 1, "branch-misses not counted", None),
        ("a multiplexed event", {"STUB_PERF": "multiplexed"}, 1, "multiplexed (50.0% enabled)", None),
        ("a decode failed", {"STUB_CHILD": "fail"}, 1, "the run failed", None),
        ("runs missing", {}, 1, "runs missing", 3),
    ]
    results = []
    for name, env, code, phrase, rr in cases:
        got, out = go(d, env, report_reps=rr)
        ok = got == code and phrase in out
        results.append(ok)
        print(f"{'ok  ' if ok else 'FAIL'} {name}: exit {got}, {'found' if phrase in out else 'missing'} {phrase!r}")
        if not ok:
            print(out)
    got, out = go(d, {})
    ok = "1.0000" not in out.split("cycles", 1)[1].splitlines()[0] and "2.0000" in out
    results.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} the ratio reads the stub's 2 for the candidate")
    failed = results.count(False)
    print(f"\n{len(results) - failed} of {len(results)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
