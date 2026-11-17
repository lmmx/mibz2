#!/usr/bin/env python3
"""The knee: why the decoder's wall time stops falling from 8 to 20 threads on the i9-10900K
(`just bench-knee`; the registration, docs/journal/2026-11-17-knee.md, frozen before the data).
A diagnostic: it reports and decides nothing in production. Standard library only.

    knee.py run    --ab AB --out RUNS.jsonl --file NAME=PATH.. --threads 4,8,10,12,20 --pinned 4,8,10
                   --rounds N [--core-events E,..] [--imc-events E,..] [--imc-prefix 'sudo -n']
    knee.py trace  --occupancy OCC --out TRACE.jsonl --file NAME=PATH.. --threads .. --pinned ..
                   --rounds N
    knee.py stream --ab AB --out STREAM.jsonl --threads 1,2,4,10,20 --repeat R [--mib M]
                   [--reps-lo 2 --reps-hi 12] [--core-events ..] [--imc-events ..] [--imc-prefix ..]
    knee.py idle   --out IDLE.jsonl --imc-events E,.. [--imc-prefix ..] [--seconds S] [--count N]
    knee.py report --runs RUNS.jsonl [--trace TRACE.jsonl] [--stream STREAM.jsonl]
                   [--idle IDLE.jsonl] [--main NAME] [--small NAME] [--json OUT.json]
    knee.py estimate [the recipe's grid]   the i9 time the recipe's grid should take

**run** (H1, H2's CPU split, H3, H4's CPU): production's decoder, `ab child mibz2 T FILE`, at
each thread count unpinned, and at each `--pinned` count pinned as `ab run --spread` pins (the
first CPU of each of the first T cores, `taskset`), on each file, every cell once a round, the
cells' order rotated a step each round and reversed every other cycle. Each run is wrapped, from
the inside out, in `taskset` (pinned cells), then `perf stat` of the process's own core and
software events (`--core-events`, as this user), then `perf stat -a` of the memory controller's
events (`--imc-events`, system-wide, behind `--imc-prefix` where that needs `sudo -n`). Either
wrapper is left out where its events are empty. Recorded a run: ab's own record (wall, CPU, peak
RSS, the output's size and hash, each thread's CPU), the process's elapsed time as seen from
here, its minor and major faults (`getrusage`'s children, so a perf wrapper's own are included:
the perf counts are the decode's alone), and every counted event with its enabled share.

**trace** (H2, H3's edges, H4): examples/occupancy.rs built with the `trace` feature, its child
(`occupancy --child FILE T 4T LEVEL`: production's `BzReader::parallel_with` at production's
window, the plan the decoder built) at level 0 (no marks) and level 1 (the timeline), in
alternating order, at the same cells. The level-1 line carries where the workers' idle time went
(`t.w.idle.*`), the scanner's and the reader's states, and the decode's edges (`t.e.*`); the
level-0 wall against level 1's is the timeline's cost, paired by round.

**stream** (H1's ceiling): `ab stream` (scale and triad, `--mib` MiB an array) at each thread
count, pinned as above where the cores allow, each at `--reps-lo` and `--reps-hi` repetitions in
alternating order, `--repeat` times, under the same wrappers. The memory controller's (and the
proxy's) bytes over the two runs' difference, against their kernel time's difference, is the
kernel's own rate: the arrays' first touch and the process's start cancel.

**idle**: the memory controller's bytes over `sleep S`, N times: the machine's idle traffic,
taken off each reading in proportion to its elapsed time.

**report**: the table and the four hypotheses' readings by the registration's rules (below,
`verdicts`), each SUPPORTED, REFUTED, INCONCLUSIVE, UNMEASURED or VOID with the figures it read.
VOID (exit 1) where the runs' outputs differ for a file, or where a cell has no clean run.
"""

import argparse
import json
import os
import resource
import shlex
import statistics
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from counters import spread  # noqa: E402

# perf scales the memory controller's counts to MiB; a bare count is 64-byte lines
SCALE = {"B": 1, "KiB": 2**10, "MiB": 2**20, "GiB": 2**30, "": 64}
LINE = 64
# the proxy for DRAM reads where the memory controller can't be counted: lines the L3 missed
PROXY = ("longest_lat_cache.miss", "LLC-load-misses")
# the registration's thresholds (docs/journal/2026-11-17-knee.md), frozen with it
PLATEAU = (12, 20)
# the count the plateau starts from: the plateau's movement is the speedup from here to 20
BASE = 8
H1_SUPPORT, H1_REFUTE, H1_FLAT = 0.80, 0.50, 0.10
IDLE_MIN = 0.10
SHARE_SUPPORT, SHARE_REFUTE = 0.50, 0.20
MAIN_BUSY_SUPPORT, MAIN_BUSY_REFUTE = 0.90, 0.60
H3_FIXED_SUPPORT, H3_FIXED_REFUTE = 0.25, 0.10
H3_MOVE_SUPPORT, H3_MOVE_REFUTE = 0.05, 0.02
TRACE_COST_MAX = 1.10
# the plateau reproduced: the x1 wall at 20 threads over 8's within this range
PLATEAU_RANGE = (0.90, 1.05)


# ---------------------------------------------------------------------------------------
# parsing


def parse_perf(text, imc=False):
    """perf stat -x, output: {event: {"value": float or None, "pct": enabled %, "raw": str}};
    the memory controller's values in bytes (`imc`), an event counted on several units summed"""
    got = {}
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        f = line.split(",")
        if len(f) < 3:
            continue
        value, unit, event = f[0], f[1], f[2]
        try:
            pct = float(f[4]) if len(f) > 4 and f[4] else 100.0
        except ValueError:
            pct = 0.0
        try:
            v = float(value)
        except ValueError:
            v = None
        if v is not None and imc:
            if unit not in SCALE:
                v = None
                value = f"unknown unit {unit!r}"
            else:
                v *= SCALE[unit]
        if event in got and got[event]["value"] is not None and v is not None:
            got[event]["value"] += v
            got[event]["pct"] = min(got[event]["pct"], pct)
        else:
            got[event] = {"value": v, "pct": pct, "raw": value}
    return got


def parse_kv(line):
    """occupancy's child line, `key=value ...`: numbers as floats, the rest as strings"""
    out = {}
    for part in line.split():
        k, eq, v = part.partition("=")
        if not eq:
            continue
        try:
            out[k] = float(v)
        except ValueError:
            out[k] = v
    return out


def thread_shares(thread_cpu, wall):
    """ab's `thread_cpu` ("name=seconds ...") over the wall: (the plan's workers, {role: share}),
    roles scan, read, main and workers (their mean)"""
    if not wall:
        return 0, {}
    workers, shares = [], {}
    for part in (thread_cpu or "").split():
        name, _, s = part.rpartition("=")
        try:
            sec = float(s)
        except ValueError:
            continue
        if name.startswith("mibz2-") and name[6:].isdigit():
            workers.append(sec / wall)
        elif name == "mibz2-scan":
            shares["scan"] = sec / wall
        elif name == "mibz2-read":
            shares["read"] = sec / wall
        elif name == "main":
            shares["main"] = sec / wall
    if workers:
        shares["workers"] = statistics.fmean(workers)
    return len(workers), shares


# ---------------------------------------------------------------------------------------
# running


def cells(files, threads, pinned):
    """(file name, threads, placement) for every file, thread count and placement"""
    out = []
    for name, _ in files:
        for t in threads:
            out.append((name, t, "unpinned"))
            if t in pinned:
                out.append((name, t, "pinned"))
    return out


def order(xs, r):
    """round r's order: rotated a step a round, reversed every other cycle of rounds"""
    n = len(xs)
    rot = xs[r % n :] + xs[: r % n]
    return rot[::-1] if (r // n) % 2 else rot


def wrap(cmd, cpus, core_events, imc_events, imc_prefix, tmp):
    """cmd inside taskset, perf stat of its core events, then perf stat -a of the memory
    controller's; with the files each perf writes"""
    files = {}
    if cpus:
        cmd = ["taskset", "-c", cpus, *cmd]
    perf = os.environ.get("KNEE_PERF", "perf")
    if core_events:
        files["core"] = os.path.join(tmp, "core.csv")
        cmd = [perf, "stat", "-x,", "-o", files["core"], "-e", ",".join(core_events), "--", *cmd]
    if imc_events:
        files["imc"] = os.path.join(tmp, "imc.csv")
        cmd = [
            *shlex.split(imc_prefix or ""),
            perf,
            "stat",
            "-a",
            "-x,",
            "-o",
            files["imc"],
            "-e",
            ",".join(imc_events),
            "--",
            *cmd,
        ]
    return cmd, files


def measured(cmd, cpus, a):
    """cmd run wrapped (`wrap`): (stdout, stderr, returncode, elapsed, faults, perf readings)"""
    with tempfile.TemporaryDirectory() as tmp:
        full, files = wrap(cmd, cpus, a.core_events, a.imc_events, a.imc_prefix, tmp)
        r0 = resource.getrusage(resource.RUSAGE_CHILDREN)
        t0 = time.monotonic()
        p = subprocess.run(full, capture_output=True, text=True)
        elapsed = time.monotonic() - t0
        r1 = resource.getrusage(resource.RUSAGE_CHILDREN)
        faults = {"minflt": r1.ru_minflt - r0.ru_minflt, "majflt": r1.ru_majflt - r0.ru_majflt}
        got = {}
        for k, path in files.items():
            try:
                with open(path) as f:
                    got[k] = parse_perf(f.read(), imc=(k == "imc"))
            except OSError as e:
                got[k] = {"_error": str(e)}
    return p.stdout, p.stderr, p.returncode, elapsed, faults, got


def events(s):
    return [e for e in (s or "").split(",") if e]


def file_args(xs):
    out = []
    for x in xs:
        name, eq, path = x.partition("=")
        if not eq:
            sys.exit(f"knee: --file {x}: NAME=PATH")
        out.append((name, path))
    return out


def ints(s):
    return [int(x) for x in s.split(",") if x]


def cmd_run(a):
    files = dict(file_args(a.file))
    cs = cells(file_args(a.file), ints(a.threads), ints(a.pinned))
    with open(a.out, "a") as out:
        for r in range(a.rounds):
            for name, t, place in order(cs, r):
                cpus = spread(t) if place == "pinned" else None
                if place == "pinned" and cpus is None:
                    rec = {
                        "round": r,
                        "file": name,
                        "threads": t,
                        "placement": place,
                        "err": f"pinned: fewer than {t} cores allowed",
                    }
                else:
                    so, se, rc, el, faults, perf = measured(
                        [a.ab, "child", a.arm, str(t), files[name]], cpus, a
                    )
                    rec = {
                        "round": r,
                        "file": name,
                        "threads": t,
                        "placement": place,
                        "cpus": cpus,
                        "elapsed_s": el,
                        "faults": faults,
                        "perf": perf,
                        "rc": rc,
                    }
                    try:
                        rec["ab"] = json.loads(so.strip().splitlines()[-1])
                        rec["err"] = rec["ab"].get("err", "") or ("" if rc == 0 else f"exit {rc}")
                    except (ValueError, IndexError):
                        rec["err"] = f"exit {rc}, no record: {se.strip()[-300:]}"
                out.write(json.dumps(rec) + "\n")
                out.flush()
                print(
                    f"knee run: round {r} {name} {t} {place}: "
                    f"{rec.get('ab', {}).get('wall', 'NA')} s {rec['err']}",
                    flush=True,
                )


def cmd_trace(a):
    files = dict(file_args(a.file))
    cs = cells(file_args(a.file), ints(a.threads), ints(a.pinned))
    with open(a.out, "a") as out:
        for r in range(a.rounds):
            for name, t, place in order(cs, r):
                cpus = spread(t) if place == "pinned" else None
                levels = (0, 1) if r % 2 == 0 else (1, 0)
                for level in levels:
                    rec = {
                        "round": r,
                        "file": name,
                        "threads": t,
                        "placement": place,
                        "cpus": cpus,
                        "level": level,
                    }
                    if place == "pinned" and cpus is None:
                        rec["err"] = f"pinned: fewer than {t} cores allowed"
                    else:
                        cmd = [
                            a.occupancy,
                            "--child",
                            files[name],
                            str(t),
                            str(a.slots_per_thread * t),
                            str(level),
                        ]
                        if cpus:
                            cmd = ["taskset", "-c", cpus, *cmd]
                        p = subprocess.run(cmd, capture_output=True, text=True)
                        lines = p.stdout.strip().splitlines()
                        rec["kv"] = parse_kv(lines[-1]) if lines else {}
                        e = rec["kv"].get("err", "-")
                        rec["err"] = (
                            ("" if e in ("-", "") else str(e))
                            if p.returncode == 0
                            else f"exit {p.returncode}: {p.stderr.strip()[-300:]}"
                        )
                    out.write(json.dumps(rec) + "\n")
                    out.flush()
                print(f"knee trace: round {r} {name} {t} {place}", flush=True)


def cmd_stream(a):
    with open(a.out, "a") as out:
        for rep in range(a.repeat):
            for t in ints(a.threads):
                cpus = spread(t)
                for kernel in ("scale", "triad"):
                    reps = (a.reps_lo, a.reps_hi) if rep % 2 == 0 else (a.reps_hi, a.reps_lo)
                    for n in reps:
                        cmd = [
                            a.ab,
                            "stream",
                            "--threads",
                            str(t),
                            "--mib",
                            str(a.mib),
                            "--reps",
                            str(n),
                            "--kernel",
                            kernel,
                        ]
                        so, se, rc, el, faults, perf = measured(cmd, cpus, a)
                        rec = {
                            "repeat": rep,
                            "threads": t,
                            "kernel": kernel,
                            "reps": n,
                            "cpus": cpus,
                            "elapsed_s": el,
                            "perf": perf,
                            "rc": rc,
                        }
                        try:
                            rec["stream"] = json.loads(so.strip().splitlines()[-1])
                            rec["err"] = "" if rc == 0 else f"exit {rc}"
                        except (ValueError, IndexError):
                            rec["err"] = f"exit {rc}, no record: {se.strip()[-300:]}"
                        out.write(json.dumps(rec) + "\n")
                        out.flush()
                print(f"knee stream: repeat {rep} {t} threads", flush=True)


def cmd_idle(a):
    a.core_events = []
    with open(a.out, "a") as out:
        for i in range(a.count):
            so, se, rc, el, faults, perf = measured(["sleep", str(a.seconds)], None, a)
            out.write(json.dumps({"i": i, "elapsed_s": el, "perf": perf, "rc": rc}) + "\n")


# ---------------------------------------------------------------------------------------
# reading


def load(path):
    if not path or not os.path.exists(path):
        return []
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]


def med(xs):
    xs = [x for x in xs if x is not None]
    return statistics.median(xs) if xs else None


def iqr(xs):
    xs = sorted(x for x in xs if x is not None)
    if len(xs) < 4:
        return None
    q = statistics.quantiles(xs, n=4)
    return (q[0], q[2])


def value(perf, part, names, *, full=False):
    """the first of `names` counted in perf[part] (its value, or None), and which; `full`: only
    a value counted the whole run (not multiplexed)"""
    got = (perf or {}).get(part) or {}
    for n in names:
        e = got.get(n)
        if e and e.get("value") is not None and (not full or e.get("pct", 0) >= 99.0):
            return e["value"], n
    return None, None


def imc_bytes(perf):
    got = (perf or {}).get("imc") or {}
    vals = [e.get("value") for k, e in got.items() if not k.startswith("_")]
    if not vals or any(v is None for v in vals):
        return None
    return sum(vals)


def idle_rate(idle):
    """the memory controller's idle bytes a second (median), or None"""
    rates = [
        imc_bytes(r.get("perf")) / r["elapsed_s"]
        for r in idle
        if r.get("rc") == 0 and imc_bytes(r.get("perf")) is not None and r.get("elapsed_s")
    ]
    return med(rates)


def ceiling(stream, idle_bps):
    """H1's ceilings, by the two runs' difference (`stream`'s docstring): the best (over thread
    counts and kernels) median rate in bytes a second of the memory controller (`imc`) and of
    the proxy (`proxy`), and ab stream's own best rate with write-allocate (`wa`)"""
    by = {}
    for r in stream:
        if r.get("err") or "stream" not in r:
            continue
        by.setdefault((r["threads"], r["kernel"], r["repeat"]), {})[r["reps"]] = r
    rates = {"imc": {}, "proxy": {}, "wa": {}}
    for (t, k, _), pair in by.items():
        for r in pair.values():
            rates["wa"].setdefault((t, k), []).append(r["stream"]["best_gbs_wa"] * 1e9)
        if len(pair) != 2:
            continue
        lo, hi = (pair[n] for n in sorted(pair))
        dt = hi["stream"]["kernel_s"] - lo["stream"]["kernel_s"]
        if dt <= 0:
            continue
        a, b = imc_bytes(lo.get("perf")), imc_bytes(hi.get("perf"))
        if a is not None and b is not None:
            idle_part = (idle_bps or 0.0) * (hi["elapsed_s"] - lo["elapsed_s"])
            rates["imc"].setdefault((t, k), []).append((b - a - idle_part) / dt)
        pa, _ = value(lo.get("perf"), "core", PROXY)
        pb, _ = value(hi.get("perf"), "core", PROXY)
        if pa is not None and pb is not None:
            rates["proxy"].setdefault((t, k), []).append((pb - pa) * LINE / dt)
    out = {}
    for unit, d in rates.items():
        meds = {key: med(v) for key, v in d.items()}
        meds = {key: v for key, v in meds.items() if v is not None}
        if meds:
            best = max(meds, key=meds.get)
            out[unit] = {
                "bps": meds[best],
                "threads": best[0],
                "kernel": best[1],
                "all": {f"{t}/{k}": v for (t, k), v in sorted(meds.items())},
            }
    return out


def table(runs, idle_bps):
    """per cell (file, threads, placement): the medians over its clean runs"""
    by = {}
    for r in runs:
        if r.get("err") or "ab" not in r:
            continue
        by.setdefault((r["file"], r["threads"], r["placement"]), []).append(r)
    out = {}
    for key, rs in by.items():
        wall = [r["ab"]["wall"] for r in rs]
        outb = rs[0]["ab"]["out"]
        row = {
            "n": len(rs),
            "wall": med(wall),
            "wall_iqr": iqr(wall),
            "out": outb,
            "elapsed": med(r["elapsed_s"] for r in rs),
        }
        row["gbs"] = outb / row["wall"] / 1e9 if row["wall"] else None
        plans = sorted({thread_shares(r["ab"].get("thread_cpu"), r["ab"]["wall"])[0] for r in rs})
        row["workers"] = plans
        shares = [thread_shares(r["ab"].get("thread_cpu"), r["ab"]["wall"])[1] for r in rs]
        row["busy"] = {
            k: med(s.get(k) for s in shares) for k in ("scan", "read", "main", "workers")
        }
        row["cpu_over_wall"] = med(r["ab"]["cpu"] / r["ab"]["wall"] for r in rs)
        row["rss_mb"] = med(r["ab"].get("rss_kb", 0) / 1024 for r in rs)
        pf = [value(r.get("perf"), "core", ("minor-faults", "page-faults"))[0] for r in rs]
        row["minflt"] = (
            med(pf) if any(x is not None for x in pf) else med(r["faults"]["minflt"] for r in rs)
        )
        row["minflt_source"] = (
            "perf" if any(x is not None for x in pf) else "rusage (perf's own included)"
        )
        row["minflt_per_mb"] = row["minflt"] / (outb / 1e6) if row["minflt"] is not None else None
        imc = []
        for r in rs:
            b = imc_bytes(r.get("perf"))
            if b is not None:
                imc.append(b - (idle_bps or 0.0) * r["elapsed_s"])
        row["imc_bytes"] = med(imc)
        row["imc_bps"] = row["imc_bytes"] / row["wall"] if imc else None
        row["imc_per_out"] = row["imc_bytes"] / outb if imc else None
        px = [value(r.get("perf"), "core", PROXY)[0] for r in rs]
        px = [x * LINE for x in px if x is not None]
        row["proxy_event"] = next(
            (
                value(r.get("perf"), "core", PROXY)[1]
                for r in rs
                if value(r.get("perf"), "core", PROXY)[1]
            ),
            None,
        )
        row["proxy_bps"] = med(px) / row["wall"] if px else None
        row["proxy_per_out"] = med(px) / outb if px else None
        out[key] = row
    return out


def trace_table(trace):
    """per cell: the level-1 runs' medians of the trace's shares and edges, over complete
    traces, and the timeline's cost (level 1's wall over level 0's, paired by round)"""
    by, walls = {}, {}
    for r in trace:
        if r.get("err") or "kv" not in r:
            continue
        key = (r["file"], r["threads"], r["placement"])
        walls.setdefault(key, {}).setdefault(r["round"], {})[r["level"]] = r["kv"].get("wall")
        if r["level"] == 1 and r["kv"].get("t.complete") == "yes":
            by.setdefault(key, []).append(r["kv"])
    keys = [
        "t.w.idle",
        "t.w.idle.reading",
        "t.w.idle.input",
        "t.w.idle.slot_earliest_decoding",
        "t.w.idle.slot_earliest_ready",
        "t.w.idle.outside",
        "t.w.idle.jobs_queued",
        "t.w.decoding",
        "t.s.reading",
        "t.s.input",
        "t.r.waiting",
        "t.r.consuming",
        "t.e.head",
        "t.e.tail",
        "t.e.end",
        "t.e.last_issued",
        "t.e.first_decode",
    ]
    out = {}
    for key in set(by) | set(walls):
        kvs = by.get(key, [])
        row = {"n": len(kvs), "plan": sorted({kv.get("plan") for kv in kvs if kv.get("plan")})}
        for k in keys:
            row[k] = med(kv.get(k) for kv in kvs if isinstance(kv.get(k), float))
        ratios = [
            w[1] / w[0]
            for w in walls.get(key, {}).values()
            if isinstance(w.get(0), float) and isinstance(w.get(1), float) and w[0] > 0
        ]
        row["cost"] = med(ratios)
        row["n_rounds"] = len(walls.get(key, {}))
        out[key] = row
    return out


def verdict(name, status, why, **figures):
    return {"hypothesis": name, "status": status, "why": why, "figures": figures}


def h1(tab, ceil, main):
    """DRAM bandwidth: the decode's mean memory traffic at the plateau's counts against the
    measured ceiling, in the memory controller's bytes where counted, else the proxy's"""
    for unit, key, label in (
        ("imc", "imc_bps", "the memory controller"),
        ("proxy", "proxy_bps", "the proxy (lines the L3 missed x 64 B: reads only)"),
    ):
        if unit not in ceil:
            continue
        rows = [tab.get((main, t, "unpinned")) for t in PLATEAU]
        if any(r is None or r.get(key) is None for r in rows):
            continue
        peak = ceil[unit]["bps"]
        fr = [r[key] / peak for r in rows]
        flat = abs(rows[1][key] / rows[0][key] - 1) <= H1_FLAT
        figs = {
            "unit": label,
            "peak_gbs": peak / 1e9,
            "peak_at": f"{ceil[unit]['threads']} threads, {ceil[unit]['kernel']}",
            **{f"gbs_{t}": r[key] / 1e9 for t, r in zip(PLATEAU, rows)},
            **{f"of_peak_{t}": f for t, f in zip(PLATEAU, fr)},
            "flat": flat,
        }
        if all(f >= H1_SUPPORT for f in fr) and flat:
            return verdict(
                "H1",
                "SUPPORTED" + ("" if unit == "imc" else " (proxy)"),
                f"at {PLATEAU} threads the decode's traffic is at least {H1_SUPPORT:.0%} of the ceiling, flat",
                **figs,
            )
        if all(f < H1_REFUTE for f in fr):
            return verdict(
                "H1",
                "REFUTED" + ("" if unit == "imc" else " (proxy)"),
                f"at {PLATEAU} threads the decode's traffic is below {H1_REFUTE:.0%} of the ceiling",
                **figs,
            )
        return verdict(
            "H1",
            "INCONCLUSIVE" + ("" if unit == "imc" else " (proxy)"),
            "between the thresholds, or not flat",
            **figs,
        )
    return verdict(
        "H1",
        "UNMEASURED",
        "neither the memory controller nor the proxy was counted for both the decode and the ceiling",
    )


def shares(tr, main, keys):
    rows = [tr.get((main, t, "unpinned")) for t in PLATEAU]
    if any(r is None or r["n"] == 0 or r.get("t.w.idle") is None for r in rows):
        return None, rows
    return [(r["t.w.idle"], sum(r.get(k) or 0.0 for k in keys)) for r in rows], rows


def trace_void(rows):
    costs = [r.get("cost") for r in rows]
    if any(c is None or c > TRACE_COST_MAX for c in costs):
        return f"the timeline costs more than {TRACE_COST_MAX - 1:.0%} of the wall at the plateau ({costs})"
    return None


def wait_verdict(name, tr, main, keys, label, extra=None):
    got, rows = shares(tr, main, keys)
    if got is None:
        return verdict(name, "UNMEASURED", "no complete level-1 trace at the plateau's counts")
    void = trace_void(rows)
    figs = {
        **{f"idle_{t}": i for t, (i, _) in zip(PLATEAU, got)},
        **{f"{label}_{t}": s for t, (_, s) in zip(PLATEAU, got)},
        **{f"{label}_of_idle_{t}": (s / i if i else 0.0) for t, (i, s) in zip(PLATEAU, got)},
        "timeline_cost": [r.get("cost") for r in rows],
    }
    if extra:
        figs.update(extra(rows))
    if void:
        return verdict(name, "VOID", void, **figs)
    of = [s / i if i else 0.0 for i, s in got]
    sup = all(i >= IDLE_MIN and f >= SHARE_SUPPORT for (i, _), f in zip(got, of))
    ref = all(f < SHARE_REFUTE for f in of) or all(i < IDLE_MIN / 2 for i, _ in got)
    return figs, sup, ref


def h2(tr, main):
    """a serial front end: workers idle while the scanner reads or waits for input"""
    got = wait_verdict(
        "H2",
        tr,
        main,
        ["t.w.idle.reading", "t.w.idle.input"],
        "front",
        extra=lambda rows: {
            f"scan_reading_{t}": r.get("t.s.reading") for t, r in zip(PLATEAU, rows)
        },
    )
    if isinstance(got, dict):
        return got
    figs, sup, ref = got
    if sup:
        return verdict(
            "H2",
            "SUPPORTED",
            f"at least {SHARE_SUPPORT:.0%} of the workers' idle time is the scanner reading or waiting for input",
            **figs,
        )
    if ref:
        return verdict(
            "H2",
            "REFUTED",
            f"under {SHARE_REFUTE:.0%} of the idle time is the front end's (or the workers are hardly idle)",
            **figs,
        )
    return verdict("H2", "INCONCLUSIVE", "between the thresholds", **figs)


def h2w(tr, main):
    """the in-order window: workers idle while the scanner waits for a slot and the earliest
    untaken block is still decoding (head of line)"""
    got = wait_verdict("H2w", tr, main, ["t.w.idle.slot_earliest_decoding"], "window")
    if isinstance(got, dict):
        return got
    figs, sup, ref = got
    if sup:
        return verdict(
            "H2w",
            "SUPPORTED",
            f"at least {SHARE_SUPPORT:.0%} of the idle time is the window full behind a block still decoding",
            **figs,
        )
    if ref:
        return verdict(
            "H2w", "REFUTED", f"under {SHARE_REFUTE:.0%} of the idle time is the window's", **figs
        )
    return verdict("H2w", "INCONCLUSIVE", "between the thresholds", **figs)


def h4(tr, tab, main):
    """the in-order consumer (main): workers idle while the window is full of blocks ready
    and not yet taken, or main busy consuming"""
    got = wait_verdict(
        "H4",
        tr,
        main,
        ["t.w.idle.slot_earliest_ready"],
        "consumer",
        extra=lambda rows: {
            f"main_consuming_{t}": r.get("t.r.consuming") for t, r in zip(PLATEAU, rows)
        },
    )
    if isinstance(got, dict):
        return got
    figs, sup, ref = got
    rows = [tr.get((main, t, "unpinned")) for t in PLATEAU]
    cons = [r.get("t.r.consuming") or 0.0 for r in rows]
    for t in PLATEAU:
        row = tab.get((main, t, "unpinned"))
        figs[f"main_cpu_{t}"] = row["busy"].get("main") if row else None
    if sup or all(c >= MAIN_BUSY_SUPPORT for c in cons):
        return verdict(
            "H4",
            "SUPPORTED",
            f"the consumer binds: at least {SHARE_SUPPORT:.0%} of the idle time is the window full of ready blocks, or main consumes {MAIN_BUSY_SUPPORT:.0%} of the time",
            **figs,
        )
    if ref and all(c < MAIN_BUSY_REFUTE for c in cons):
        return verdict(
            "H4",
            "REFUTED",
            f"under {SHARE_REFUTE:.0%} of the idle time is the consumer's and main consumes under {MAIN_BUSY_REFUTE:.0%}",
            **figs,
        )
    return verdict("H4", "INCONCLUSIVE", "between the thresholds", **figs)


def h3(tab, tr, main, small, copies):
    """fixed costs: the wall's fixed part from the small file and the main (copies x) file,
    and whether the plateau moves with the file's size"""

    def w(f, t):
        r = tab.get((f, t, "unpinned"))
        return (r["wall"], r["out"]) if r and r.get("wall") else (None, None)

    figs, fixed = {}, []
    for t in PLATEAU:
        (w1, o1), (w4, o4) = w(small, t), w(main, t)
        if None in (w1, w4):
            return verdict("H3", "UNMEASURED", f"no clean runs of both files at {t} threads")
        k = o4 / o1 if o1 else copies
        per = (w4 - w1) / (k - 1) if k > 1 else None
        if per is None:
            return verdict("H3", "UNMEASURED", "the two files are the same size")
        f = w1 - per
        fixed.append(f / w1)
        figs[f"fixed_s_{t}"] = f
        figs[f"fixed_of_small_{t}"] = f / w1
    base = BASE
    if (small, base, "unpinned") not in tab or (main, base, "unpinned") not in tab:
        return verdict(
            "H3", "UNMEASURED", "no count below the plateau to measure its movement", **figs
        )
    s1 = w(small, base)[0] / w(small, PLATEAU[1])[0]
    s4 = w(main, base)[0] / w(main, PLATEAU[1])[0]
    figs.update(
        {
            f"speedup_small_{base}_to_{PLATEAU[1]}": s1,
            f"speedup_main_{base}_to_{PLATEAU[1]}": s4,
            "moves": s4 - s1,
        }
    )
    for t in PLATEAU:
        r = tr.get((small, t, "unpinned")) or {}
        figs[f"trace_head_s_{t}"] = r.get("t.e.head")
        figs[f"trace_tail_s_{t}"] = r.get("t.e.tail")
        figs[f"idle_outside_{t}"] = r.get("t.w.idle.outside")
        row = tab.get((small, t, "unpinned")) or {}
        figs[f"minflt_per_mb_{t}"] = row.get("minflt_per_mb")
        figs[f"start_and_exit_s_{t}"] = (
            (row["elapsed"] - row["wall"]) if row.get("elapsed") else None
        )
    if all(f >= H3_FIXED_SUPPORT for f in fixed) and s4 - s1 >= H3_MOVE_SUPPORT:
        return verdict(
            "H3",
            "SUPPORTED",
            f"the fixed part is at least {H3_FIXED_SUPPORT:.0%} of the small file's wall and the plateau moves with size",
            **figs,
        )
    if all(f < H3_FIXED_REFUTE for f in fixed) and s4 - s1 < H3_MOVE_REFUTE:
        return verdict(
            "H3",
            "REFUTED",
            f"the fixed part is under {H3_FIXED_REFUTE:.0%} and the plateau doesn't move",
            **figs,
        )
    return verdict("H3", "INCONCLUSIVE", "between the thresholds", **figs)


def plateau(tab, small):
    """whether the release-scaling plateau is reproduced: the x1 wall at 20 threads over 8's
    within PLATEAU_RANGE (unpinned); None where either is missing"""
    a, b = tab.get((small, BASE, "unpinned")), tab.get((small, PLATEAU[1], "unpinned"))
    if not a or not b or not a.get("wall") or not b.get("wall"):
        return None
    ratio = b["wall"] / a["wall"]
    return {"ratio": ratio, "reproduced": PLATEAU_RANGE[0] <= ratio <= PLATEAU_RANGE[1]}


FOLLOW_UP = {
    "H1": "fewer DRAM bytes per output byte (fused passes, the output's stores, the reader's copy); worker-count work deprioritised",
    "H2": "the scanner: a parallel or faster block scan (the par-scan line)",
    "H2w": "the window's ordering: a larger window, or dispatching blocks so the head of line finishes first",
    "H4": "the consumer: main's copy and hash (a zero-copy read path; larger reads)",
    "H3": "fixed costs: pre-faulting or reusing buffers, the first block's latency, splitting the tail",
}


def priority(vs, tr, tab, main):
    """the registration's decision rule: H1 first where SUPPORTED (a bandwidth ceiling caps every
    other gain); then the other SUPPORTED hypotheses by the share of time each could recover at
    20 threads (a wait's share of the workers' time; H3's fixed share of the small file's wall);
    with none, the per-worker contention model (MONODROMY's lane) stays the lead"""
    sup = {v["hypothesis"]: v for v in vs if v["status"].startswith("SUPPORTED")}
    ranked = []
    if "H1" in sup:
        ranked.append(("H1", None))
    t = PLATEAU[1]
    recover = {
        "H2": sup.get("H2", {}).get("figures", {}).get(f"front_{t}"),
        "H2w": sup.get("H2w", {}).get("figures", {}).get(f"window_{t}"),
        "H4": sup.get("H4", {}).get("figures", {}).get(f"consumer_{t}"),
        "H3": sup.get("H3", {}).get("figures", {}).get(f"fixed_of_small_{t}"),
    }
    rest = sorted(((h, s) for h, s in recover.items() if h in sup), key=lambda x: -(x[1] or 0.0))
    ranked += rest
    if not ranked:
        return [
            {
                "rank": 1,
                "hypothesis": None,
                "recoverable": None,
                "follow_up": "none supported: the per-worker contention model (c(W) = a + b W^2, MONODROMY's records) stays the lead; next, cycles per byte decomposed by thread count",
            }
        ]
    return [
        {"rank": i + 1, "hypothesis": h, "recoverable": s, "follow_up": FOLLOW_UP[h]}
        for i, (h, s) in enumerate(ranked)
    ]


def cmd_report(a):
    runs, trace, stream, idle = load(a.runs), load(a.trace), load(a.stream), load(a.idle)
    void = []
    hashes = {}
    for r in runs:
        if not r.get("err") and "ab" in r:
            hashes.setdefault(r["file"], set()).add(r["ab"]["hash"])
    for f, hs in hashes.items():
        if len(hs) != 1:
            void.append(f"{f}: the runs' outputs differ ({len(hs)} hashes)")
    expect = {(r["file"], r["threads"], r["placement"]) for r in runs}
    ib = idle_rate(idle)
    tab = table(runs, ib)
    for key in sorted(expect):
        if key not in tab:
            void.append(f"{key}: no clean run")
    if void:
        print("bench-knee: VOID, no reading:\n- " + "\n- ".join(void))
        return 1
    errs = [r for r in runs if r.get("err")]
    tr = trace_table(trace)
    ceil = ceiling(stream, ib)
    print(
        "bench-knee: a diagnostic, reported, decides nothing in production "
        "(docs/journal/2026-11-17-knee.md)"
    )
    print(
        f"runs: {len(runs)}, failed: {len(errs)}; trace records: {len(trace)}; stream records: "
        f"{len(stream)}; idle readings: {len(idle)}"
        + (f" (idle traffic {ib / 1e9:.3f} GB/s, taken off)" if ib is not None else "")
    )
    print(
        "\nfile threads placement  n  workers  wall_s   GB/s  scan  read  main  workers  "
        "minflt/MB  DRAM_GB/s  DRAM_B/out  proxy_GB/s  proxy_B/out"
    )
    fmt = lambda x, p=3: "NA" if x is None else f"{x:.{p}f}"  # noqa: E731
    for key in sorted(tab):
        r = tab[key]
        b = r["busy"]
        print(
            f"{key[0]} {key[1]} {key[2]} {r['n']} {','.join(map(str, r['workers']))} {fmt(r['wall'], 4)} "
            f"{fmt(r['gbs'], 2)} {fmt(b.get('scan'), 2)} {fmt(b.get('read'), 2)} {fmt(b.get('main'), 2)} "
            f"{fmt(b.get('workers'), 2)} {fmt(r['minflt_per_mb'], 1)} "
            f"{fmt(r['imc_bps'] and r['imc_bps'] / 1e9, 2)} {fmt(r['imc_per_out'], 2)} "
            f"{fmt(r['proxy_bps'] and r['proxy_bps'] / 1e9, 2)} {fmt(r['proxy_per_out'], 2)}"
        )
    if tr:
        print(
            "\nthe timeline (level 1; shares of the workers' time, the scanner's and main's of the span)"
        )
        print(
            "file threads placement  n  plan  cost  idle  front(read+input)  window(slot,decoding)  "
            "consumer(slot,ready)  outside  queued  scan_reading  main_waiting  main_consuming  head_s  tail_s"
        )
        for key in sorted(tr):
            r = tr[key]
            front = (
                None
                if r["t.w.idle.reading"] is None
                else (r["t.w.idle.reading"] + (r["t.w.idle.input"] or 0))
            )
            print(
                f"{key[0]} {key[1]} {key[2]} {r['n']} {','.join(r['plan']) or 'NA'} {fmt(r['cost'])} "
                f"{fmt(r['t.w.idle'])} {fmt(front)} {fmt(r['t.w.idle.slot_earliest_decoding'])} "
                f"{fmt(r['t.w.idle.slot_earliest_ready'])} {fmt(r['t.w.idle.outside'])} "
                f"{fmt(r['t.w.idle.jobs_queued'])} {fmt(r['t.s.reading'])} {fmt(r['t.r.waiting'])} "
                f"{fmt(r['t.r.consuming'])} {fmt(r['t.e.head'], 4)} {fmt(r['t.e.tail'], 4)}"
            )
    print("\nthe ceilings (best median over thread counts and kernels)")
    for unit in ("imc", "proxy", "wa"):
        c = ceil.get(unit)
        print(
            f"  {unit}: "
            + (
                "UNMEASURED"
                if not c
                else f"{c['bps'] / 1e9:.2f} GB/s at {c['threads']} threads, {c['kernel']}"
            )
        )
    vs = [
        h1(tab, ceil, a.main),
        h2(tr, a.main),
        h2w(tr, a.main),
        h3(tab, tr, a.main, a.small, a.copies),
        h4(tr, tab, a.main),
    ]
    print("\nthe hypotheses (the registration's rules)")
    for v in vs:
        figs = ", ".join(
            f"{k} {fmt(x) if isinstance(x, float) else x}" for k, x in v["figures"].items()
        )
        print(f"  {v['hypothesis']}: {v['status']}: {v['why']}" + (f" [{figs}]" if figs else ""))
    pl = plateau(tab, a.small)
    print(
        "\nthe plateau: "
        + (
            "not measured (no x1 runs at 8 and 20 threads unpinned)"
            if pl is None
            else f"x1 wall at {PLATEAU[1]} over {BASE} threads {pl['ratio']:.3f}: "
            + ("reproduced" if pl["reproduced"] else "NOT reproduced; the readings stand, but say so")
        )
    )
    pr = priority(vs, tr, tab, a.main)
    print("\nthe follow-up, by the registration's decision rule")
    for p in pr:
        print(
            f"  {p['rank']}. {p['hypothesis'] or '-'}: {p['follow_up']}"
            + (f" (recoverable {p['recoverable']:.3f})" if p["recoverable"] is not None else "")
        )
    if a.json:
        with open(a.json, "w") as f:
            json.dump(
                {
                    "table": {"|".join(map(str, k)): v for k, v in tab.items()},
                    "trace": {"|".join(map(str, k)): v for k, v in tr.items()},
                    "ceilings": ceil,
                    "idle_bps": ib,
                    "verdicts": vs,
                    "plateau": pl,
                    "priority": pr,
                },
                f,
                indent=1,
            )
    return 0


def cmd_estimate(a):
    """the grid's i9 time from the 0.5.3 release-scaling walls (Wikidata x1: about 0.11 s at 8
    to 20 threads, 0.2 s at 4) and a process's start and perf's (0.08 s a run, assumed)"""
    th, pin = ints(a.threads), ints(a.pinned)
    n_cells = len(th) + len([t for t in pin if t in th])
    w1 = {t: (0.20 if t <= 4 else 0.12) for t in th}
    per_round = sum(w1[t] * (1 + a.copies) + 2 * 0.08 for t in th) + sum(
        w1[t] * (1 + a.copies) + 2 * 0.08 for t in pin if t in th
    )
    run = a.rounds * per_round
    trace = a.trace_rounds * 2 * per_round * 1.05
    # stream: 3 x 256 MiB arrays' first touch (about 0.4 s) and reps at about 30 GB/s
    s_rep = 3 * a.mib * 2**20 * 4 / 3 / 30e9
    stream = (
        a.repeat
        * len(ints(a.stream_threads))
        * 2
        * (2 * (0.4 + 0.08) + (a.reps_lo + a.reps_hi) * s_rep)
    )
    idle = 2 * a.idle_count * (a.idle_seconds + 0.1)
    build, corpus = 150, 30
    total = run + trace + stream + idle + build + corpus
    print(
        json.dumps(
            {
                "cells": n_cells * 2,
                "run_s": round(run),
                "trace_s": round(trace),
                "stream_s": round(stream),
                "idle_s": round(idle),
                "build_s": build,
                "corpus_s": corpus,
                "total_min": round(total / 60, 1),
            }
        )
    )


def main(argv=None):
    p = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = p.add_subparsers(dest="cmd", required=True)

    def common(s):
        s.add_argument("--core-events", type=events, default=[])
        s.add_argument("--imc-events", type=events, default=[])
        s.add_argument("--imc-prefix", default="")

    s = sub.add_parser("run")
    s.add_argument("--ab", required=True)
    s.add_argument("--arm", default="mibz2")
    s.add_argument("--out", required=True)
    s.add_argument("--file", action="append", required=True)
    s.add_argument("--threads", required=True)
    s.add_argument("--pinned", default="")
    s.add_argument("--rounds", type=int, default=10)
    common(s)
    s = sub.add_parser("trace")
    s.add_argument("--occupancy", required=True)
    s.add_argument("--out", required=True)
    s.add_argument("--file", action="append", required=True)
    s.add_argument("--threads", required=True)
    s.add_argument("--pinned", default="")
    s.add_argument("--rounds", type=int, default=5)
    s.add_argument("--slots-per-thread", type=int, default=4)
    s = sub.add_parser("stream")
    s.add_argument("--ab", required=True)
    s.add_argument("--out", required=True)
    s.add_argument("--threads", required=True)
    s.add_argument("--repeat", type=int, default=3)
    s.add_argument("--mib", type=int, default=256)
    s.add_argument("--reps-lo", type=int, default=2)
    s.add_argument("--reps-hi", type=int, default=12)
    common(s)
    s = sub.add_parser("idle")
    s.add_argument("--out", required=True)
    s.add_argument("--seconds", type=float, default=2.0)
    s.add_argument("--count", type=int, default=5)
    common(s)
    s = sub.add_parser("report")
    s.add_argument("--runs", required=True)
    s.add_argument("--trace")
    s.add_argument("--stream")
    s.add_argument("--idle")
    s.add_argument("--main", default="wikidata-x4")
    s.add_argument("--small", default="wikidata")
    s.add_argument("--copies", type=int, default=4)
    s.add_argument("--json")
    s = sub.add_parser("estimate")
    s.add_argument("--threads", default="4,8,10,12,20")
    s.add_argument("--pinned", default="4,8,10")
    s.add_argument("--rounds", type=int, default=10)
    s.add_argument("--trace-rounds", type=int, default=10)
    s.add_argument("--copies", type=int, default=4)
    s.add_argument("--stream-threads", default="1,2,4,10,20")
    s.add_argument("--repeat", type=int, default=3)
    s.add_argument("--mib", type=int, default=256)
    s.add_argument("--reps-lo", type=int, default=2)
    s.add_argument("--reps-hi", type=int, default=12)
    s.add_argument("--idle-count", type=int, default=5)
    s.add_argument("--idle-seconds", type=float, default=2.0)
    a = p.parse_args(argv)
    return {
        "run": cmd_run,
        "trace": cmd_trace,
        "stream": cmd_stream,
        "idle": cmd_idle,
        "report": cmd_report,
        "estimate": cmd_estimate,
    }[a.cmd](a) or 0


if __name__ == "__main__":
    sys.exit(main())
