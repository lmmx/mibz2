"""bench/memlat.py driven for real with stand-ins: a stub `perf` (`record` running the child,
`script` writing samples as perf 6.8's builtin-script.c prints them, `mem report`) and a stub
`ab child`. The report reads latency by data source and the top IPs, and every defect leaves no
report.

    python3 bench/test_memlat.py
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
import memlat  # noqa: E402

PERF = r'''#!/usr/bin/env python3
import os, subprocess, sys
a = sys.argv[1:]
mode = os.environ.get("STUB_PERF", "ok")
if a[0] == "record":
    out = a[a.index("-o") + 1]
    cmd = a[a.index("--") + 1:]
    if cmd[0] == "taskset":
        cmd = cmd[3:]
    p = subprocess.run(cmd)
    open(out, "w").write(cmd[2])
    sys.exit(p.returncode)
if a[0] == "script":
    if mode == "scriptfail":
        sys.stderr.write("failed to process sample\n")
        sys.exit(1)
    arm = open(a[a.index("-i") + 1]).read()
    slow = 2 if "7-28" in arm else 1
    # (data_src, level, weight, ip, sym): perf's data_src__fprintf pads the hex to 16 and the
    # decoded text to the longest so far; the weight is %16 PRIu64; then ' ' and the IP's %16lx
    rows = [(0x68100142, "L1 hit", 35, 0x4a3f2c, "mibz2::walk"),
            (0x6a100142, "L3 hit", 80 * slow, 0x4a3f40, "mibz2::walk"),
            (0x6a100142, "L3 hit", 90 * slow, 0x4a3f40, "mibz2::walk"),
            (0x1a100842, "RAM hit", 300 * slow, 0x4a3f40, "mibz2::walk"),
            (0x6a100142, "LFB/MAB hit", 50, 0x51aa00, "mibz2::cells::induce"),
            (0x44100142, "L2 hit", 40, 0x51aa10, "[unknown]")]
    if mode == "nosamples":
        rows = []
    width = 0
    for src, lvl, w, ip, sym in rows:
        out = f"{src:16x} |OP LOAD|LVL {lvl}|SNP None|TLB L1 or L2 hit|LCK No|BLK  N/A"
        width = max(width, len(out))
        print(f"{out:<{width}}{w:16d} {ip:16x} {sym}")
    print("garbled line")
    sys.exit(0)
if a[0] == "mem":
    print("# perf mem report (stub)")
    sys.exit(0)
sys.exit(2)
'''
CHILD = r'''#!/usr/bin/env python3
import json, os, sys
if os.environ.get("STUB_CHILD") == "fail" and "7-28" in sys.argv[2]:
    sys.exit(3)
print(json.dumps({"wall": 0.5, "out": 209700196, "hash": "h", "err": ""}))
'''
ARMS = ["mibz2-occ-10-40", "mibz2-occ-7-28"]


def setup():
    d = Path(tempfile.mkdtemp())
    for name, body in (("perf", PERF), ("ab", CHILD)):
        p = d / name
        p.write_text(body)
        p.chmod(p.stat().st_mode | stat.S_IEXEC)
    return d


def go(d, name, env, reps=2, report_reps=None):
    out = d / name
    e = {**os.environ, "PATH": f"{d}:{os.environ['PATH']}", **env}
    subprocess.run([sys.executable, str(HERE / "memlat.py"), "run", "--ab", str(d / "ab"),
                    *sum((["--arm", x] for x in ARMS), []), "--threads", "10", "--reps", str(reps),
                    "--file", "x.bz2", "--out", str(out)], env=e, capture_output=True, check=True)
    buf = io.StringIO()
    with redirect_stdout(buf):
        code = memlat.main(["memlat.py", "report", str(out), *sum((["--arm", x] for x in ARMS), []),
                            "--threads", "10", "--reps", str(report_reps or reps)])
    return code, buf.getvalue(), out


def main():
    d = setup()
    results = []
    code, out, run_dir = go(d, "ok", {})
    recs = [json.loads(line) for line in open(run_dir / "memlat.jsonl")]
    files = sorted(p.name for p in run_dir.iterdir())
    blocks = out.split("\n  mibz2-occ-")
    slow = next(b for b in blocks if b.startswith("7-28"))
    want = [
        "12 samples, 2 lines not read",  # 6 a rep, 2 reps, a garbled line each
        "RAM            16.7%",
        "L1             16.7%",
        "LFB/MAB        16.7%",
        "4a3f40",
        "mibz2::cells::induce",
        "[unknown]",
    ]
    ok = (code == 0 and len(recs) == 4 and all(r["cpus"] == "unpinned" for r in recs)
          and not any(f.endswith(".data") for f in files)
          and sum(f.endswith(".txt.gz") for f in files) == 4 and sum(f.endswith("-mem-report.txt") for f in files) == 2
          and all(w in out for w in want)
          # 7/28's stub doubles the L3 and RAM latencies: the RAM median 600, L3's 170
          and "  RAM" in slow and " 600 " in slow.split("RAM", 1)[1].splitlines()[0]
          and " 170 " in slow.split("  L3", 1)[1].splitlines()[0])
    results.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} a complete run: exit {code}, {len(recs)} records, "
          + ", ".join(f"{'found' if w in out else 'missing'} {w!r}" for w in want))
    if not ok:
        print(out, files)
    # the top IP by summed latency is the walk's slow load, mostly from L3
    top = next(ln for ln in out.splitlines() if "4a3f40" in ln)
    ok = top.split()[-2] == "L3" and top.split()[-1] == "mibz2::walk"
    results.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} the top IP, its source and symbol: {top.split()}")
    for name, env, phrase, rr in [
        ("a decode failed", {"STUB_CHILD": "fail"}, "the run failed", None),
        ("perf script failed", {"STUB_PERF": "scriptfail"}, "perf script", None),
        ("no sample parsed", {"STUB_PERF": "nosamples"}, "no sample parsed", None),
        ("runs missing", {}, "runs missing", 3),
    ]:
        code, out, _ = go(d, name.replace(" ", "-"), env, report_reps=rr)
        ok = code == 1 and "VOID" in out and phrase in out
        results.append(ok)
        print(f"{'ok  ' if ok else 'FAIL'} {name}: exit {code}, {'found' if phrase in out else 'missing'} {phrase!r}")
        if not ok:
            print(out)
    ok = memlat.source("Remote RAM (1 hop) hit") == "RAM" and memlat.source("Uncached hit") == "other"
    results.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} the data sources read from perf's levels")
    failed = results.count(False)
    print(f"\n{len(results) - failed} of {len(results)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
