"""The workstation's memory-controller and core counters around the decode (#39).

`just bench-imc [ARGS]` runs it (`just bench-quiet` too, then the occupancy curve), after
bench/corpus.sh has built the occupancy example and the corpus:

    python3 bench/imc.py [--threads 1,6,8,10] [--reps 5] [--loops 3] [--idle 30]
                         [--copy 20] [--settle 20] [--note TEXT] [--out PATH] [STEP ...]

Steps, all by default, in this order:
- inventory: what else is on the machine (displays and GPUs, the busiest processes, the
  DIMMs), so the readings carry their own provenance;
- idle: the memory controller's reads and writes each second over --idle seconds of
  nothing: steady, or bursty;
- calibrate: `perf bench mem memcpy` copies --copy GiB, beside an idle reading of its own;
  its traffic less the idle rate's share should be 1-2x the copy in reads (the source, and
  the destination's lines read for ownership unless the copy streams), and 1x in writes;
- core: process-scoped cycles, ref-cycles and instructions of the decode at each thread
  count (IPC, and the clock against the reference clock);
- curve: the memory controller's traffic during the decode at each thread count, each
  reading paired with an idle reading of its own just before it, the thread counts in a
  rotated order each repetition; the decode's own traffic is the reading less its idle
  pair's rate over the same time.

The memory-controller events count the whole machine, so they need `perf stat -a` and
sudo, as the core events are run too (the decode then runs as root, and writes nothing).
Each decode reading runs --loops decodes in one process chain so that a reading spans
seconds, not a fraction of one. Everything goes to one JSON file (imc-<UTC time>.json) and
a summary on stdout.
"""

import argparse
import datetime
import json
import os
import shutil
import statistics
import subprocess
import sys
import tempfile
import time

IMC = ["uncore_imc/data_reads/", "uncore_imc/data_writes/"]
CORE = ["cycles", "ref-cycles", "instructions"]
# perf scales the memory controller's counts to MiB; a bare count is 64-byte lines
SCALE = {"B": 1, "KiB": 2**10, "MiB": 2**20, "GiB": 2**30, "": 64}
GIB = 2**30
EXE = "target/release/examples/occupancy"
CORPUS = "corpora/wikidata.bz2"


def run(argv, check=False):
    r = subprocess.run(argv, capture_output=True, text=True)
    if check and r.returncode:
        sys.exit(f"bench-imc: {' '.join(argv)} failed:\n{r.stderr}")
    return r


def perf(events, cmd, system, interval=None):
    """`cmd` under `sudo perf stat`: (its wall seconds, its stdout, {time: {event: value}})"""
    with tempfile.TemporaryDirectory() as d:
        out = os.path.join(d, "perf.csv")
        argv = ["sudo", "perf", "stat", "-x,", "-o", out, "-e", ",".join(events)]
        argv += ["-a"] if system else []
        argv += ["-I", str(interval)] if interval else []
        t0 = time.monotonic()
        r = subprocess.run([*argv, "--", *cmd], capture_output=True, text=True)
        wall = time.monotonic() - t0
        if r.returncode:
            sys.exit(f"bench-imc: perf stat failed:\n{r.stderr}")
        with open(out) as f:
            rows = parse(f.read(), events)
    return wall, r.stdout, rows


def parse(text, events):
    """perf's CSV: [time,]value,unit,event,...; values summed per event (and interval)"""
    rows = {}
    for line in text.splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        f = line.strip().split(",")
        hit = [(i, e) for i, x in enumerate(f) for e in events if x == e or x.endswith(f"/{e}/")]
        if not hit or hit[0][0] < 2:
            continue
        i, e = hit[0]
        value, unit = f[i - 2], f[i - 1]
        try:
            v = float(value)
        except ValueError:
            sys.exit(f"bench-imc: perf could not count {e}: {value!r}")
        if e in IMC:
            if unit not in SCALE:
                sys.exit(f"bench-imc: {e} in an unknown unit {unit!r}")
            v *= SCALE[unit]
        at = float(f[0]) if i == 3 else None
        rows.setdefault(at, {}).setdefault(e, 0.0)
        rows[at][e] += v
        try:
            run_ns = float(f[i + 1])
        except (IndexError, ValueError):
            run_ns = 0.0
        rows[at]["_run_ns"] = max(rows[at].get("_run_ns", 0.0), run_ns)
    for at, r in rows.items():
        missing = [e for e in events if e not in r]
        if missing:
            sys.exit(f"bench-imc: perf reported no {', '.join(missing)}")
    return rows


def summary(xs):
    xs = list(xs)
    return {
        "n": len(xs),
        "mean": statistics.fmean(xs),
        "sd": statistics.stdev(xs) if len(xs) > 1 else 0.0,
        "min": min(xs),
        "max": max(xs),
    }


def decode(t, loops):
    """--loops decodes at t threads in one chain of processes"""
    child = [os.path.abspath(EXE), "--child", os.path.abspath(CORPUS), str(t), str(2 * t), "0"]
    return ["sh", "-c", 'for i in $(seq "$0"); do "$@" || exit 1; done', str(loops), *child]


def decoded(stdout):
    """the bytes out and the decode's own seconds, summed over the chain's lines"""
    out = wall = 0.0
    lines = [ln for ln in stdout.splitlines() if ln.startswith("wall=")]
    for ln in lines:
        kv = dict(w.split("=", 1) for w in ln.split())
        if kv.get("err", "-") != "-":
            sys.exit(f"bench-imc: the decode failed: {kv['err']}")
        out += float(kv["out"])
        wall += float(kv["wall"])
    return out, wall, len(lines)


def span(wall, rows):
    """the counters' own counting time, else the wall time around perf (sudo's included)"""
    ns = rows[None].get("_run_ns", 0.0)
    return ns / 1e9 if 0 < ns / 1e9 <= 1.5 * wall else wall


def rates(wall, rows):
    r, s = rows[None], span(wall, rows)
    return r[IMC[0]] / s, r[IMC[1]] / s


def inventory():
    def cmd(argv, keep=None):
        if not shutil.which(argv[0]):
            return f"({argv[0]} not on PATH)"
        r = run(argv)
        text = r.stdout if r.returncode == 0 else r.stdout + r.stderr
        lines = text.splitlines()
        return "\n".join(ln for ln in lines if keep is None or any(k in ln for k in keep))

    return {
        "kernel": cmd(["uname", "-a"]),
        "cpu": cmd(["lscpu"], ["Model name", "Thread(s)", "Core(s)", "MHz", "L3"]),
        "loadavg": open("/proc/loadavg").read().strip(),
        "meminfo": cmd(["head", "-5", "/proc/meminfo"]),
        "monitors": cmd(["xrandr", "--listmonitors"]),
        "gpus": cmd(["sh", "-c", "lspci -k | grep -A3 -E 'VGA|3D|Display'"]),
        "dimms": cmd(
            ["sudo", "dmidecode", "-t", "memory"],
            ["Locator:", "Size:", "Speed:", "Type:", "Rank:"],
        ),
        "top_cpu": cmd(["ps", "-eo", "pid,comm,%cpu,%mem,rss", "--sort=-%cpu"]).splitlines()[:16],
        "top_rss": cmd(["ps", "-eo", "pid,comm,%cpu,%mem,rss", "--sort=-rss"]).splitlines()[:16],
        "processes": sum(p.isdigit() for p in os.listdir("/proc")),
    }


def idle(seconds):
    _, _, rows = perf(IMC, ["sleep", str(seconds)], system=True, interval=1000)
    ts = sorted(at for at in rows if at is not None)
    per = []
    for prev, at in zip([0.0, *ts], ts):
        dt = at - prev
        per.append({"t": at, "reads": rows[at][IMC[0]] / dt, "writes": rows[at][IMC[1]] / dt})
    return {
        "seconds": per,
        "reads": summary(p["reads"] for p in per),
        "writes": summary(p["writes"] for p in per),
    }


def calibrate(gib, settle):
    wall0, _, rows0 = perf(IMC, ["sleep", str(settle)], system=True)
    r0, w0 = rates(wall0, rows0)
    # (-f default: without it perf bench runs every memcpy it has, three on x86-64)
    cmd = ["perf", "bench", "mem", "memcpy", "-f", "default", "-s", "1GB", "-l", str(gib)]
    wall, out, rows = perf(IMC, cmd, system=True)
    wall = span(wall, rows)
    reads = rows[None][IMC[0]] - r0 * wall
    writes = rows[None][IMC[1]] - w0 * wall
    copied = gib * GIB
    return {
        "copied_bytes": copied,
        "wall": wall,
        "idle_reads": r0,
        "idle_writes": w0,
        "net_reads": reads,
        "net_writes": writes,
        "reads_per_copied": reads / copied,
        "writes_per_copied": writes / copied,
        "perf_bench": out.strip().splitlines()[-3:],
    }


def core(threads, reps, loops):
    res = []
    for rep in range(reps):
        for t in threads[rep % len(threads) :] + threads[: rep % len(threads)]:
            wall, out, rows = perf(CORE, decode(t, loops), system=False)
            n, dwall, k = decoded(out)
            r = rows[None]
            res.append(
                {
                    "rep": rep,
                    "threads": t,
                    "decodes": k,
                    "out": n,
                    "decode_wall": dwall,
                    **{e: r[e] for e in CORE},
                    "ipc": r["instructions"] / r["cycles"],
                    "clock_ratio": r["cycles"] / r["ref-cycles"],
                }
            )
    return res


def curve(threads, reps, loops, pair):
    res = []
    for rep in range(reps):
        for t in threads[rep % len(threads) :] + threads[: rep % len(threads)]:
            iw, _, irows = perf(IMC, ["sleep", str(pair)], system=True)
            ir, iwr = rates(iw, irows)
            wall, out, rows = perf(IMC, decode(t, loops), system=True)
            wall = span(wall, rows)
            n, dwall, k = decoded(out)
            reads, writes = rows[None][IMC[0]], rows[None][IMC[1]]
            res.append(
                {
                    "rep": rep,
                    "threads": t,
                    "decodes": k,
                    "out": n,
                    "decode_wall": dwall,
                    "wall": wall,
                    "throughput_mb_s": n / dwall / 1e6,
                    "reads": reads / wall,
                    "writes": writes / wall,
                    "idle_reads": ir,
                    "idle_writes": iwr,
                    "net_reads_per_byte": (reads - ir * wall) / n,
                    "net_writes_per_byte": (writes - iwr * wall) / n,
                    "net_read_gib_s": (reads - ir * wall) / wall / GIB,
                }
            )
    return res


def by_threads(rows, keys):
    out = {}
    for t in sorted({r["threads"] for r in rows}):
        mine = [r for r in rows if r["threads"] == t]
        out[t] = {k: summary(r[k] for r in mine) for k in keys}
    return out


def report(res):
    g = lambda x: f"{x / GIB:5.1f}"  # noqa: E731
    print(f"\n== bench-imc {res['meta']['utc']}  note: {res['meta'].get('note') or '-'}")
    if "idle" in res:
        i = res["idle"]
        print(
            f"idle {len(i['seconds'])} s: reads {g(i['reads']['mean'])} GiB/s"
            f" (sd {g(i['reads']['sd'])}, {g(i['reads']['min'])}-{g(i['reads']['max'])}),"
            f" writes {g(i['writes']['mean'])} GiB/s (sd {g(i['writes']['sd'])})"
        )
    if "calibrate" in res:
        c = res["calibrate"]
        print(
            f"calibrate: copied {c['copied_bytes'] / GIB:.0f} GiB; net reads"
            f" {c['reads_per_copied']:.2f}x (expect 1-2x), net writes"
            f" {c['writes_per_copied']:.2f}x (expect ~1x); idle beside it"
            f" {g(c['idle_reads'])} / {g(c['idle_writes'])} GiB/s"
        )
    if "core" in res:
        print("core (process-scoped)   threads  instructions     cycles   IPC  cycles/ref")
        for t, s in by_threads(
            res["core"], ["instructions", "cycles", "ipc", "clock_ratio"]
        ).items():
            k = res["meta"]["loops"]
            print(
                f"{'':24}{t:7}  {s['instructions']['mean'] / k:12.3e}"
                f" {s['cycles']['mean'] / k:10.3e}  {s['ipc']['mean']:4.2f}"
                f"  {s['clock_ratio']['mean']:.3f}"
            )
    if "curve" in res:
        print(
            "curve   threads  MB/s (sd)    reads  writes  idle reads  net read GiB/s"
            "  net B read/B (sd)  net B written/B"
        )
        keys = [
            "throughput_mb_s",
            "reads",
            "writes",
            "idle_reads",
            "net_read_gib_s",
            "net_reads_per_byte",
            "net_writes_per_byte",
        ]
        for t, s in by_threads(res["curve"], keys).items():
            print(
                f"{'':8}{t:7}  {s['throughput_mb_s']['mean']:4.0f} ({s['throughput_mb_s']['sd']:3.0f})"
                f"  {g(s['reads']['mean'])}  {g(s['writes']['mean'])}"
                f"       {g(s['idle_reads']['mean'])}           {s['net_read_gib_s']['mean']:5.2f}"
                f"       {s['net_reads_per_byte']['mean']:5.1f} ({s['net_reads_per_byte']['sd']:4.1f})"
                f"            {s['net_writes_per_byte']['mean']:5.1f}"
            )


def main():
    steps = ["inventory", "idle", "calibrate", "core", "curve"]
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("steps", nargs="*", choices=[[], *steps], default=[])
    ap.add_argument("--threads", default="1,6,8,10")
    ap.add_argument("--reps", type=int, default=5)
    ap.add_argument("--loops", type=int, default=3, help="decodes in each reading")
    ap.add_argument("--idle", type=int, default=30, help="seconds of idle readings")
    ap.add_argument("--copy", type=int, default=20, help="GiB the calibration copies")
    ap.add_argument("--pair", type=float, default=3, help="seconds of each paired idle")
    ap.add_argument("--settle", type=float, default=20, help="seconds to wait first")
    ap.add_argument("--note", default="", help="what was closed or running, for the record")
    ap.add_argument("--out", default=None)
    a = ap.parse_args()
    todo = a.steps or steps
    threads = [int(t) for t in a.threads.split(",")]
    root = run(["git", "rev-parse", "--show-toplevel"], check=True).stdout.strip()
    os.chdir(root)
    for p in [EXE, CORPUS]:
        if not os.path.exists(p):
            sys.exit(f"bench-imc: no {p}; run bash bench/corpus.sh (just bench-imc does)")
    if not shutil.which("perf"):
        sys.exit("bench-imc: needs perf on PATH")
    utc = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out = a.out or f"imc-{utc}.json"
    print("bench-imc: perf stat -a needs sudo for the memory controller's events")
    run(["sudo", "-v"])
    res = {
        "meta": {
            "utc": utc,
            "note": a.note,
            "commit": run(["git", "rev-parse", "HEAD"]).stdout.strip(),
            "perf": run(["perf", "--version"]).stdout.strip(),
            "threads": threads,
            "reps": a.reps,
            "loops": a.loops,
            "pair_seconds": a.pair,
            "steps": todo,
        }
    }
    if a.settle:
        print(f"bench-imc: settling for {a.settle:g} s")
        time.sleep(a.settle)
    for step in todo:
        print(f"bench-imc: {step}", flush=True)
        run(["sudo", "-v"])
        if step == "inventory":
            res[step] = inventory()
        elif step == "idle":
            res[step] = idle(a.idle)
        elif step == "calibrate":
            res[step] = calibrate(a.copy, max(a.pair, 5))
        elif step == "core":
            res[step] = core(threads, a.reps, a.loops)
        elif step == "curve":
            res[step] = curve(threads, a.reps, a.loops, a.pair)
        with open(out, "w") as f:
            json.dump(res, f, indent=1)
    report(res)
    print(f"\nbench-imc: wrote {out}")


if __name__ == "__main__":
    main()
