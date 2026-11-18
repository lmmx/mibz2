"""bench/sustain.py driven for real with stand-ins: a stub `perf` (perf stat -a's CSV for the
package energy, the clock and the memory controller, counted, not counted, multiplexed or with
a clock that jumps, as told) and a stub `ab child`. A complete run is judged into two_arm's
records and read by bench/two_arm.py; every defect leaves no records.

    python3 bench/test_sustain.py
"""
import io
import json
import os
import stat
import sys
import tempfile
from contextlib import redirect_stdout
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import sustain  # noqa: E402
import two_arm  # noqa: E402

PERF = r'''#!/usr/bin/env python3
import os, subprocess, sys
a = sys.argv[1:]
assert a[0] == "stat" and "-a" in a, a
out = a[a.index("-o") + 1]
events = [a[i + 1] for i, x in enumerate(a) if x == "-e"]
cmd = a[a.index("--") + 1:]
p = subprocess.run(cmd)
mode = os.environ.get("STUB_PERF", "ok")
n_path = os.environ["STUB_COUNTER"]
n = int(open(n_path).read() or 0) if os.path.exists(n_path) else 0
open(n_path, "w").write(str(n + 1))
idle = cmd[0] == "sleep"
clock = 1.3 if (mode == "jumps" and n % 2) or mode == "turbo" else 1.0
with open(out, "w") as f:
    f.write("# started\n\n")
    for e in events:
        if e == "power/energy-pkg/":
            f.write(f"{5.0 if idle else 150.0},Joules,{e},1000,100.00,,\n")
        elif e in ("cycles", "ref-cycles"):
            v = 1e6 if idle else 1e10
            if e == "cycles":
                v *= clock
            pct = "50.00" if (mode == "multiplexed" and e == "cycles" and not idle) else "100.00"
            f.write(f"{v:.0f},,{e},1000,{pct},,\n")
        else:
            if mode == "notcounted" and "writes" in e and not idle:
                f.write(f"<not counted>,MiB,{e},0,0.00,,\n")
                continue
            for unit in (0, 1):
                f.write(f"{(1.0 if idle else 400.0) / 2:.6f},MiB,uncore_imc_{unit}/{e.split('/', 1)[1]},1000,100.00,,\n")
sys.exit(p.returncode)
'''
CHILD = r'''#!/usr/bin/env python3
import json, os, sys
_, _, arm, t, path = sys.argv[:5]
n = int(arm.split("-")[2])
base = os.path.basename(path)
fast = os.environ.get("STUB_FAST", "")
f = 0.9 if fast and fast in arm else 1.0
wall = 0.01 * f
if os.environ.get("STUB_CHILD") == "fail" and "8-32" in arm and "text" in base:
    print(json.dumps({"wall": wall, "err": "boom"}))
    sys.exit(0)
workers = n - 1 if (os.environ.get("STUB_CHILD") == "short" and "occ-8" in arm and "random" in base) else n
print(json.dumps({"wall": wall, "cpu": wall * n, "rss_kb": 1000 * n, "out": 1000 + len(base),
                  "hash": "h-" + base, "err": "",
                  "thread_cpu": " ".join([f"mibz2-{i}=0.1" for i in range(workers)] + ["mibz2-scan=0.05"])}))
'''


def setup():
    d = Path(tempfile.mkdtemp())
    for name, body in (("perf", PERF), ("ab", CHILD)):
        p = d / name
        p.write_text(body)
        p.chmod(p.stat().st_mode | stat.S_IEXEC)
    for c in ("wikidata-x16", "text-x16", "random-x8"):
        (d / f"{c}.bz2").write_bytes(b"x")
    return d


CELLS = ["u10:unpinned:2:4:mibz2-occ-7-28,mibz2-occ-6-24,mibz2-occ-8-32",
         "p2:pinned:1:2:mibz2-oce-3-12,mibz2-oce-2-8"]


def drive(d, env=None, cells=CELLS):
    old = dict(os.environ)
    os.environ.update({"STUB_COUNTER": str(d / "n"), **(env or {})})
    try:
        out = d / "runs.jsonl"
        args = ["sustain.py", "run", "--ab", str(d / "ab"), "--perf", str(d / "perf"), "--main",
                str(d / "wikidata-x16.bz2"), "--guard", str(d / "text-x16.bz2"), "--guard",
                str(d / "random-x8.bz2"), "--out", str(out), "--warm", "0", "--prewarm", "0.01",
                "--gate", "0", "--idle", "0.01"]
        for c in cells:
            args += ["--cell", c]
        with redirect_stdout(io.StringIO()):
            code = sustain.main(args)
        return code, out
    finally:
        os.environ.clear()
        os.environ.update(old)


def judge(runs, placement, extra=()):
    out = runs.parent / f"{placement}-judged.jsonl"
    if out.exists():
        out.unlink()
    buf = io.StringIO()
    with redirect_stdout(buf):
        code = sustain.main(["sustain.py", "judged", str(runs), "--placement", placement, "--out", str(out),
                             *(extra if "--prewarm" in extra else ("--prewarm", "0.01", *extra))])
    return code, buf.getvalue(), out


def check(name, ok, detail=""):
    print(f"{'ok  ' if ok else 'FAIL'} {name}" + (f": {detail}" if detail and not ok else ""))
    return ok


def main():
    results = []
    d = setup()
    code, runs = drive(d)
    rows = [json.loads(line) for line in open(runs)]
    phases = {}
    for r in rows:
        phases[r.get("phase")] = phases.get(r.get("phase"), 0) + 1
    results.append(check("a run completes", code == 0, str(code)))
    results.append(check("each phase recorded", set(phases) >= {"gate", "idle", "prewarm", "warm", "measure"}, str(phases)))
    m = [r for r in rows if r.get("phase") == "measure"]
    # 4 rounds x 3 arms + 2 rounds x 2 arms blocks, 4 measured decodes each
    results.append(check("every block measured: main, each guard, main", len(m) == (4 * 3 + 2 * 2) * 4, len(m)))
    results.append(check("each decode counted: energy, clock, both IMC events summed over controllers",
                         all(r["counts"]["imc_read_b"][2] == 2 and r["counts"]["energy_j"][0] == 150.0 for r in m)))
    u = [r for r in m if r["cell"] == "u10"]
    results.append(check("unpinned on every online CPU", all(r["cpus"] == ",".join(map(str, sustain.online())) for r in u)))
    pos = {}
    for r in u:
        if r["index"] == 0:
            pos.setdefault(r["arm"], []).append(r["position"])
    results.append(check("the order rotates: each arm in more than one position",
                         all(len(set(v)) >= 2 for v in pos.values()), str(pos)))
    results.append(check("order(): reversed every other cycle", sustain.order(["a", "b", "c"], 3) == ["c", "b", "a"]
                         and sustain.order(["a", "b", "c"], 1) == ["b", "c", "a"]))

    code, out, jf = judge(runs, "unpinned")
    recs = [json.loads(line) for line in open(jf)] if jf.exists() else []
    results.append(check("judged: unpinned records", code == 0 and len(recs) == 4 * 3 * 3, out))
    w = [r for r in recs if r["corpus"] == "wikidata-x16"]
    results.append(check("judged: the main corpus's two decodes averaged", all(r["decodes"] == 2 for r in w)))
    results.append(check("judged: worker threads from thread_cpu",
                         all(two_arm.worker_names(r) == [f"mibz2-{i}" for i in range(two_arm.occupancy_workers(r["arm"]))]
                             for r in recs)))
    env = {"cpus": len(sustain.online()), "cores": len(sustain.online()),
           "core_cpus": [str(c) for c in sustain.online()], "cpu_quota": None}
    (d / "env.json").write_text(json.dumps(env))
    buf = io.StringIO()
    with redirect_stdout(buf):
        c2 = two_arm.main(["two_arm.py", str(jf), "--main", "wikidata-x16", "--guard", "text-x16,random-x8",
                           "--controls", "", "--rounds", "4", "--env", str(d / "env.json"),
                           "--cores", str(env["cores"]), "--cpus", str(env["cpus"]), "--sparse", "--alpha", "0.025",
                           "--primary", "wikidata-x16,text-x16", "--check-workers", "--judged", "2",
                           "--hold-max-threads", "0", "--unpinned", "--ref-at", "mibz2-occ-7-28=2",
                           "--cand", "mibz2-occ-6-24", "--judge-at", "mibz2-occ-6-24=2",
                           "--cand", "mibz2-occ-8-32", "--judge-at", "mibz2-occ-8-32=2"])
    results.append(check("two_arm reads the judged records", c2 == 0 and "THE CHOICE at 2 threads" in buf.getvalue(),
                         buf.getvalue()[-600:]))
    code, out, jf = judge(runs, "pinned")
    results.append(check("judged: pinned records", code == 0 and len(open(jf).readlines()) == 2 * 2 * 3, out))

    buf = io.StringIO()
    with redirect_stdout(buf):
        sustain.main(["sustain.py", "report", str(runs)])
    rep = buf.getvalue()
    results.append(check("report: the declared diagnostics",
                         all(s in rep for s in ("package W", "order (", "drift (", "load:",
                                                "steady state: 12 of 12 blocks", "mibz2-occ-8-32")), rep[:400]))
    results.append(check("report: watts, clock and DRAM a byte", "| 1.000 (3.70) |" in rep, rep[:600]))

    # a faster arm: two_arm sees it
    d2 = setup()
    _, runs2 = drive(d2, {"STUB_FAST": "occ-8-32"}, cells=CELLS[:1])
    code, _, jf2 = judge(runs2, "unpinned")
    walls = {}
    for r in map(json.loads, open(jf2)):
        if r["corpus"] == "wikidata-x16":
            walls.setdefault(r["arm"], []).append(r["wall"])
    results.append(check("a faster arm's walls carried through", code == 0
                         and max(walls["mibz2-occ-8-32"]) < min(walls["mibz2-occ-7-28"]), str(walls)))

    for mode, phrase in (("multiplexed", "multiplexed (50.0% enabled)"), ("notcounted", "not counted"),
                         ("jumps", "the sustained state not held")):
        dm = setup()
        _, rm = drive(dm, {"STUB_PERF": mode}, cells=CELLS[:1])
        code, out, jf = judge(rm, "unpinned")
        results.append(check(f"VOID: perf {mode}", code == 1 and phrase in out and not jf.exists(), out[:400]))
    for mode, phrase in (("fail", "failed: boom"), ("short", "name different worker threads")):
        dm = setup()
        _, rm = drive(dm, {"STUB_CHILD": mode}, cells=CELLS[:1])
        code, out, jf = judge(rm, "unpinned")
        results.append(check(f"VOID: child {mode}", code == 1 and phrase in out and not jf.exists(), out[:400]))
    # a measured decode missing, one duplicated, outputs differing
    lines = open(runs).read().splitlines()
    mi = next(i for i, line in enumerate(lines) if '"phase": "measure"' in line and '"u10"' in line)
    for name, edit, phrase in (
            ("a measured decode missing", lambda ls: ls[:mi] + ls[mi + 1:], "measured decodes missing"),
            ("a measured decode duplicated", lambda ls: ls[:mi + 1] + [ls[mi]] + ls[mi + 1:], "duplicated"),
            ("outputs differ", lambda ls: ls[:mi] + [ls[mi].replace('"hash": "h-', '"hash": "x-')] + ls[mi + 1:],
             "outputs differ"),
            ("not JSON", lambda ls: ls + ["{"], "not JSON")):
        f = d / "edited.jsonl"
        f.write_text("\n".join(edit(lines)) + "\n")
        code, out, jf = judge(f, "unpinned")
        results.append(check(f"VOID: {name}", code == 1 and phrase in out, out[:400]))
    # the clamp screen (record 64's note 1): a cell registered clamped whose decodes ran in turbo
    dt = setup()
    _, rt = drive(dt, {"STUB_PERF": "turbo"}, cells=CELLS[:1])
    code, out, jf = judge(rt, "unpinned", ["--clamped", "u10"])
    results.append(check("VOID: a clamped cell's main-corpus decodes in turbo",
                         code == 1 and "at or above 1.2 in a cell registered clamped" in out and not jf.exists(), out[:400]))
    code, out, jf = judge(rt, "unpinned", ["--clamped", "p10"])
    results.append(check("the clamp screen only in the cells registered clamped", code == 0, out[:400]))
    code, out, jf = judge(runs, "unpinned", ["--clamped", "u10"])
    results.append(check("a clamped cell's decodes below 1.2 pass the screen", code == 0, out[:400]))
    m2 = [r for r in rows if r.get("phase") == "measure" and r["cell"] == "u10"]
    results.append(check("each decode records the arm before it and the block before it",
                         all("prev_arm" in r and "prev_block" in r for r in m2)
                         and all(r["prev_arm"] == r["arm"] for r in m2 if r["index"] > 0)
                         and any(r["prev_block"] not in (None, r["arm"]) for r in m2), str(m2[:2])))
    results.append(check("report: the clock state and the carry-over", "the cell's state (its measured main-corpus decodes): clamped" in rep
                         and "clock state (" in rep
                         and "12 of 12 clamped" in rep and "carry-over (" in rep and "after " in rep, rep[:800]))
    # the pre-warm (the Researcher's REVIEW 6985154079): missing, failed, uncounted or short VOIDs
    pw = [i for i, line in enumerate(lines) if '"phase": "prewarm"' in line and '"u10"' in line]
    for name, edit, extra, phrase in (
            ("the pre-warm missing", lambda ls: [x for i, x in enumerate(ls) if i not in pw], (), "u10: no pre-warm"),
            ("the pre-warm failed with no counters",
             lambda ls: [json.dumps({**json.loads(x), "rc": 1, "err": "perf: boom", "counts": {}}) if i in pw else x
                         for i, x in enumerate(ls)], (), "failed: perf: boom"),
            ("the pre-warm uncounted", lambda ls: [json.dumps({**json.loads(x), "counts": {}}) if i in pw else x
                                                   for i, x in enumerate(ls)], (), "power/energy-pkg/ missing"),
            ("the pre-warm short of its seconds", lambda ls: ls, ("--prewarm", "60"), "short of the registered 60 s"),
            ("the pre-warm on an arm the cell does not measure",
             lambda ls: [json.dumps({**json.loads(x), "arm": "mibz2-occ-9-36"}) if i in pw else x
                         for i, x in enumerate(ls)], (), "not one arm the cell measures")):
        f = d / "edited.jsonl"
        f.write_text("\n".join(edit(lines)) + "\n")
        code, out, jf = judge(f, "unpinned", ("--prewarm", "0.01") + tuple(extra) if not extra else extra)
        results.append(check(f"VOID: {name}", code == 1 and phrase in out and not jf.exists(), out[:400]))
    code, out, jf = judge(runs, "unpinned", ("--prewarm", "0.01"))
    results.append(check("the pre-warm whole and long enough passes", code == 0, out[:400]))
    # the parser: per-controller events merged, units read, a failed entry failing the sum
    tf = d / "p.csv"
    tf.write_text("1.5,Joules,power/energy-pkg/,1,100.00,,\n2,MiB,uncore_imc_0/data_reads/,1,100.00,,\n"
                  "<not counted>,MiB,uncore_imc_1/data_reads/,0,0.00,,\n10,,uncore_imc/data_writes/,1,100.00,,\n")
    got = sustain.parse_perf(str(tf))
    results.append(check("parse_perf: joules, MiB, a bare count as 64-byte lines, a failed controller fails the sum",
                         got["energy_j"][0] == 1.5 and got["imc_read_b"][0] is None and got["imc_read_b"][2] == 2
                         and got["imc_write_b"][0] == 640, str(got)))
    failed = results.count(False)
    print(f"\n{len(results) - failed} of {len(results)} checks pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
