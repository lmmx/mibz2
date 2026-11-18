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
arm = cmd[2] if len(cmd) > 2 else "idle"
with open(out, "w") as f:
    if events[0].startswith("uncore_imc"):
        # the memory controller, system-wide only, on two controllers, in MiB: 7 B read and 4 B
        # written a byte of output (0.8 of that for a `wide` arm); idle, 1 MiB each
        assert "-a" in a, "the memory controller is counted system-wide"
        for e in events:
            per = (7 if "reads" in e else 4) * 209700196 * (0.8 if "wide" in arm else 1.0)
            v = 2**20 if arm == "idle" else per
            if mode == "imcnotcounted" and arm != "idle" and "writes" in e:
                f.write(f"<not counted>,MiB,{e},0,0.00,,\n")
                continue
            for unit in range(2):
                f.write(f"{v / 2 / 2**20:.6f},MiB,{e},1000,100.00,,\n")
        sys.exit(p.returncode)
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
    for name, body in (("perf", PERF), ("ab", CHILD), ("ab2", CHILD)):
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
    # the L2 events, the reference from another build's harness (`NAME@EXE`)
    out_file = d / "l2.jsonl"
    ref = f"mibz2-workers-lazy@{d / 'ab2'}"
    e = {**os.environ, "PATH": f"{d}:{os.environ['PATH']}"}
    subprocess.run([sys.executable, str(HERE / "counters.py"), "run", "--ab", str(d / "ab"), "--arm", ref,
                    "--arm", "mibz2-workers-stop15", "--threads", "10", "--reps", "2", "--file", "x.bz2",
                    "--out", str(out_file), "--events", "l2"], env=e, capture_output=True, check=True)
    recs = [json.loads(line) for line in open(out_file)]
    buf = io.StringIO()
    with redirect_stdout(buf):
        code = counters.main(["counters.py", "report", str(out_file), "--ref", ref, "--cand", "mibz2-workers-stop15",
                              "--threads", "10", "--reps", "2", "--events", "l2"])
    out = buf.getvalue()
    ok = (code == 0 and len(recs) == 4 and {r["arm"] for r in recs} == {ref, "mibz2-workers-stop15"}
          and all(set(r["counts"]) == {"cycles", "instructions", "l2_rqsts.miss", "l2_rqsts.references"} for r in recs)
          and "L2 misses" in out and "2.0000" in out)
    results.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} the L2 events, the reference another build's: exit {code}, {len(recs)} records")
    if not ok:
        print(out)
    # the memory controller (`--events imc`): system-wide, unpinned, an idle record each rep, the
    # idle rate taken off, the bytes read in perf's MiB
    def imc(env, drop_idle=False):
        out_file = d / "imc.jsonl"
        e = {**os.environ, "PATH": f"{d}:{os.environ['PATH']}", "COUNTERS_IDLE_S": "0.05", **env}
        ref = f"mibz2@{d / 'ab2'}"
        subprocess.run([sys.executable, str(HERE / "counters.py"), "run", "--ab", str(d / "ab"), "--arm", ref,
                        "--arm", "mibz2-wide", "--threads", "8,10", "--reps", "3", "--file", "x.bz2",
                        "--out", str(out_file), "--events", "imc", "--unpinned"], env=e, capture_output=True, check=True)
        recs = [json.loads(line) for line in open(out_file)]
        if drop_idle:
            with open(out_file, "w") as f:
                for r in recs:
                    if not (r["arm"] == "idle" and r["rep"] == 1):
                        f.write(json.dumps(r) + "\n")
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = counters.main(["counters.py", "report", str(out_file), "--ref", ref, "--cand", "mibz2-wide",
                                  "--threads", "8,10", "--reps", "3", "--events", "imc"])
        return code, buf.getvalue(), recs

    code, out, recs = imc({})
    lines = {ln.split()[0]: ln.split() for ln in out.splitlines() if ln.strip().split()[:1] in (["read"], ["written"], ["total"])}
    ok = (code == 0 and len(recs) == 3 + 2 * 2 * 3 and sum(r["arm"] == "idle" for r in recs) == 3
          and all(r["cpus"] == "unpinned" for r in recs if r["arm"] != "idle")
          and abs(float(lines["read"][1]) - 7.0) < 0.01 and abs(float(lines["written"][1]) - 4.0) < 0.01
          and abs(float(lines["total"][-1]) - 0.8) < 0.001)
    results.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} the memory controller: 7 and 4 B a byte read in MiB from two controllers, "
          f"the idle rate off, the candidate at 0.8: exit {code}")
    if not ok:
        print(out)
    for name, env, drop, phrase in (("an IMC event not counted", {"STUB_PERF": "imcnotcounted"}, False, "not counted"),
                                    ("an idle record missing", {}, True, "runs missing")):
        code, out, _ = imc(env, drop)
        ok = code == 1 and phrase in out
        results.append(ok)
        print(f"{'ok  ' if ok else 'FAIL'} {name}: exit {code}, {'found' if phrase in out else 'missing'} {phrase!r}")
        if not ok:
            print(out)
    failed = results.count(False)
    print(f"\n{len(results) - failed} of {len(results)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
