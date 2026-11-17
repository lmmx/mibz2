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
        elif e == "task-clock":
            f.write(f"{4000.25 * (2 if 'stop' in arm else 1):.2f},msec,{e},1000,100.00,,\n")
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
    # named groups and several candidates (`just bench-workers-low`'s occupancy diagnostic): the
    # reference 8/32, production 9/36 and 7/28 and 6/24 its candidates, at 12 threads (unpinned)
    occ_groups = "cycles,instructions,l2_rqsts.miss,l2_lines_out.non_silent;cycles,LLC-load-misses"
    occ_ref, occ_cands = "mibz2-occ-8-32", ["mibz2", "mibz2-occ-7-28", "mibz2-occ-6-24"]

    def occ_run(arms, out_file):
        subprocess.run([sys.executable, str(HERE / "counters.py"), "run", "--ab", str(d / "ab"),
                        *sum((["--arm", x] for x in arms), []), "--threads", "12", "--reps", "2",
                        "--file", "x.bz2", "--out", str(out_file), "--groups", occ_groups],
                       env=e, capture_output=True, check=True)

    def occ_report(out_file, groups=occ_groups):
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = counters.main(["counters.py", "report", str(out_file), "--ref", occ_ref,
                                  *sum((["--cand", c] for c in occ_cands), []), "--threads", "12",
                                  "--reps", "2", "--groups", groups])
        return code, buf.getvalue()

    occ_file = d / "occ.jsonl"
    occ_run([occ_ref, *occ_cands], occ_file)
    recs = [json.loads(line) for line in open(occ_file)]
    code, out = occ_report(occ_file)
    ok = (code == 0 and len(recs) == 16 and {r["arm"] for r in recs} == {occ_ref, *occ_cands}
          and all(r["cpus"] == "unpinned" for r in recs)
          and {frozenset(r["counts"]) for r in recs} == {
              frozenset({"cycles", "instructions", "l2_rqsts.miss", "l2_lines_out.non_silent"}),
              frozenset({"cycles", "LLC-load-misses"})}
          and all(f"{c}: IPC" in out for c in [occ_ref, *occ_cands])
          and all(ev in out for ev in ("l2_rqsts.miss", "l2_lines_out.non_silent", "LLC-load-misses"))
          and "mibz2-occ-8-32 over" not in out and "and mibz2-occ-6-24 over mibz2-occ-8-32" in out)
    results.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} named groups, three candidates against 8/32: exit {code}, {len(recs)} records")
    if not ok:
        print(out)
    code, out = occ_report(occ_file, groups="cycles,instructions,l2_rqsts.miss,l2_rqsts.references")
    ok = code == 1 and "VOID" in out
    results.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} the report naming other groups than the run's: exit {code}, VOID")
    part = d / "occ-part.jsonl"
    occ_run([occ_ref, *occ_cands[:2]], part)
    code, out = occ_report(part)
    ok = code == 1 and "runs missing" in out
    results.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} a candidate's runs missing: exit {code}, {'found' if 'runs missing' in out else 'missing'} 'runs missing'")
    # the at-cores diagnostic (`just bench-workers-at-cores`): production's `mibz2` (10 workers
    # at 10 threads, given) the reference, 9/36 to 6/24 the candidates (their workers read from
    # their names); cycles beside ref-cycles for the clock, task-clock for the busy share
    atc_groups = "cycles,ref-cycles,instructions,l2_rqsts.miss,task-clock;cycles,LLC-load-misses"
    atc_cands = ["mibz2-occ-9-36", "mibz2-occ-8-32", "mibz2-occ-7-28", "mibz2-occ-6-24"]
    atc_file = d / "atc.jsonl"
    subprocess.run([sys.executable, str(HERE / "counters.py"), "run", "--ab", str(d / "ab"),
                    *sum((["--arm", x] for x in ["mibz2", *atc_cands]), []), "--threads", "10", "--reps", "2",
                    "--file", "x.bz2", "--out", str(atc_file), "--groups", atc_groups],
                   env=e, capture_output=True, check=True)

    def atc_report(extra=()):
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = counters.main(["counters.py", "report", str(atc_file), "--ref", "mibz2",
                                  *sum((["--cand", c] for c in atc_cands), []), "--threads", "10", "--reps", "2",
                                  "--groups", atc_groups, *extra])
        return code, buf.getvalue()

    code, out = atc_report(("--workers", "mibz2=10"))
    # the stub: cycles 1e6 and ref-cycles 2e6 in the first group; task-clock 4000.25 msec, the wall 0.5 s
    want = ["mibz2: IPC 3.000; cycles over ref-cycles 0.500; busy share 0.800 a worker (task-clock over wall x 10)",
            "mibz2-occ-8-32: IPC 3.000; cycles over ref-cycles 0.500; busy share 1.000 a worker",
            "mibz2-occ-6-24: IPC 3.000; cycles over ref-cycles 0.500; busy share 1.333 a worker", "ref-cycles",
            "task-clock"]
    ok = code == 0 and all(w in out for w in want)
    results.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} the clock and the busy share, W 10 to 6: exit {code}, "
          + ", ".join(f"{'found' if w in out else 'missing'} {w!r}" for w in want))
    if not ok:
        print(out)
    code, out = atc_report()
    ok = code == 0 and "mibz2: IPC 3.000; cycles over ref-cycles 0.500\n" in out and "x 9)" in out
    results.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} production's busy share left out without its workers given: exit {code}")
    if not ok:
        print(out)
    code, out = atc_report(("--workers", "mibz2=ten"))
    ok = code == 1 and "VOID" in out and "not ARM=N" in out
    results.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} --workers malformed: exit {code}, VOID")
    # the sweep's diagnostic (`just bench-workers-sweep-low`), as at 4 threads: one group (the
    # clock, cycles and task-clock), production given 4 workers, 4/16 to 2/8 read from their names
    swp_group = "cycles,ref-cycles,instructions,l2_rqsts.miss,task-clock"
    swp_cands = ["mibz2-occ-4-16", "mibz2-occ-3-12", "mibz2-occ-2-8"]
    swp_file = d / "swp.jsonl"
    subprocess.run([sys.executable, str(HERE / "counters.py"), "run", "--ab", str(d / "ab"),
                    *sum((["--arm", x] for x in ["mibz2", *swp_cands]), []), "--threads", "4", "--reps", "2",
                    "--file", "x.bz2", "--out", str(swp_file), "--groups", swp_group],
                   env=e, capture_output=True, check=True)
    buf = io.StringIO()
    with redirect_stdout(buf):
        code = counters.main(["counters.py", "report", str(swp_file), "--ref", "mibz2",
                              *sum((["--cand", c] for c in swp_cands), []), "--threads", "4", "--reps", "2",
                              "--groups", swp_group, "--workers", "mibz2=4"])
    out = buf.getvalue()
    # task-clock 4000.25 msec over the wall 0.5 s: 8.0 CPUs busy, over W
    want = ["mibz2: IPC 3.000; cycles over ref-cycles 0.500; busy share 2.000 a worker (task-clock over wall x 4)",
            "mibz2-occ-3-12: IPC 3.000; cycles over ref-cycles 0.500; busy share 2.667 a worker",
            "mibz2-occ-2-8: IPC 3.000; cycles over ref-cycles 0.500; busy share 4.000 a worker"]
    ok = code == 0 and all(w in out for w in want) and "LLC-load-misses" not in out
    results.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} the sweep's busy share at 4 threads, W 4 to 2, one group: exit {code}, "
          + ", ".join(f"{'found' if w in out else 'missing'} {w!r}" for w in want))
    if not ok:
        print(out)
    failed = results.count(False)
    print(f"\n{len(results) - failed} of {len(results)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
