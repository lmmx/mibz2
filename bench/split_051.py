#!/usr/bin/env python3
"""The 0.5.1 split (`just bench-split-051`, #145 QUEUED 6916536279): which decoder merge between
two builds carries the step between them, thread count by thread count, with each build's LLC
misses and instructions per output byte beside it, recorded and never judged.

    split_051.py counters --arm LABEL=EXE ... --threads 8,10 --file F.bz2 --out C.jsonl [--reps 3]
    split_051.py report RUNS.jsonl --step A:B ... [--total A:B] [--judge 10,12,16,20]
                       [--counters C.jsonl]

**Arms.** A build's arm in `ab run` is `mibz2@EXE`, EXE that build's harness, which decodes with
its own library (`ab child`). Its label is the directory it was built in, `build-LABEL/`.

**The rule** (C1a, posted before the data in the QUEUED item). It is a diagnostic and changes no
default.
- At each thread count, a step A→B is the geometric mean of the rounds' paired ratios of wall
  time, B over A, with its 95% interval (Student's t). It is attributed if its interval excludes 1.
- If both steps are attributed at a thread count, each step's share of the total is its log over
  the total's log. The logs add up exactly, since they are means over the same rounds.
- If no step is attributed at any of the judged thread counts, the total's step is flagged as not
  reproduced.

**Counters** (`counters`): a pass separate from the timed run, untimed. Each build decodes the
file under `perf stat -e instructions,LLC-load-misses`, `--reps` times. It is pinned as `ab run
--spread` pins: one CPU on each of the first t cores while there are t cores, else unpinned. The
median of each count per output byte is recorded. It runs `perf` as this user, else `sudo -n
perf`, and else records that it could not count and why.
"""
import argparse
import collections
import json
import math
import os
import re
import statistics
import subprocess
import sys
import tempfile

# Student's t, two-sided 95%, by degrees of freedom (1..30); 1.96 beyond
T95 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
       2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060,
       2.056, 2.052, 2.048, 2.045, 2.042]


def label(arm):
    m = re.search(r"build-([^/]+)/", arm)
    return m.group(1) if m else arm


def interval(logs):
    """(geometric mean, low, high) of ratios given as logs"""
    n = len(logs)
    mu = statistics.mean(logs)
    if n < 2:
        return math.exp(mu), float("nan"), float("nan")
    t = T95[n - 2] if n - 1 <= len(T95) else 1.96
    h = t * statistics.stdev(logs) / math.sqrt(n)
    return math.exp(mu), math.exp(mu - h), math.exp(mu + h)


def report(args):
    walls = collections.defaultdict(dict)  # (threads, round) -> label -> wall
    outputs = collections.defaultdict(set)
    held = collections.defaultdict(set)
    errors = 0
    with open(args.runs) as f:
        for line in f:
            if not line.strip():
                continue
            r = json.loads(line)
            if r.get("err"):
                errors += 1
                continue
            lab = label(r["arm"])
            walls[(r["threads"], r["round"])][lab] = r["wall"]
            outputs[r.get("corpus", "")].add((r.get("out"), r.get("hash")))
            hs = r.get("held_spinner")
            held[r["threads"]].add(None if not hs else bool(hs.get("overlap")))
    differ = [c for c, o in outputs.items() if len(o) != 1]
    print(f"runs with an error {errors}; corpora whose builds' outputs differ {differ}")
    if errors or differ:
        print("NOT READ: a run failed, or the builds' outputs differ")
        return 1
    threads = sorted({t for t, _ in walls})
    steps = [tuple(s.split(":")) for s in args.step]
    total = tuple(args.total.split(":")) if args.total else None
    judge = {int(t) for t in args.judge.split(",")} if args.judge else set()
    counters = collections.defaultdict(dict)
    if args.counters and os.path.exists(args.counters):
        for line in open(args.counters):
            if line.strip():
                c = json.loads(line)
                counters[(c["label"], c["threads"])] = c

    def step(t, a, b):
        logs = [math.log(v[b] / v[a]) for (tt, _), v in walls.items() if tt == t and a in v and b in v]
        return interval(logs) + (len(logs),)

    print("\nwall, B over A: the geometric mean of the paired rounds' ratios [95% interval]; "
          "* attributed (the interval excludes 1)")
    attributed_any = False
    for t in threads:
        sp = held[t]
        spin = ("spinner on, sharing a decoder core" if True in sp else
                "spinner on, on a free core" if False in sp else "no spinner")
        print(f"\n{t} threads ({spin})")
        res = []
        for a, b in steps:
            g, lo, hi, n = step(t, a, b)
            att = lo > 1 or hi < 1
            res.append((a, b, g, att))
            attributed_any |= att and t in judge
            print(f"  {a} -> {b}: {g:.4f} [{lo:.4f}, {hi:.4f}] over {n} rounds{' *' if att else ''}")
        if total:
            g, lo, hi, n = step(t, *total)
            print(f"  {total[0]} -> {total[1]} (total): {g:.4f} [{lo:.4f}, {hi:.4f}]")
            if len(res) == 2 and all(r[3] for r in res) and abs(math.log(g)) > 0:
                shares = [math.log(r[2]) / math.log(g) for r in res]
                print("  shares of the total: " + ", ".join(
                    f"{r[0]} -> {r[1]} {s:.0%}" for r, s in zip(res, shares)))
        for lab in sorted({l for (l, tt) in counters if tt == t}):
            c = counters[(lab, t)]
            if c.get("why"):
                print(f"  counters {lab}: not counted ({c['why']})")
            else:
                print(f"  counters {lab}: instructions {c['instructions_per_byte']:.3f} a byte, "
                      f"LLC-load-misses {c['llc_misses_per_byte']:.5f} a byte (median of {c['reps']})")
    if 10 in threads:
        print("\nThe 10-thread cell: --spread pins it to one CPU on each of 10 cores, and the held "
              "spinner has no free core, so it shares core 9 with the decoder. Paired steps are read "
              "as at any count (every build runs beside the same spinner); absolute walls at 10 are "
              "not production's.")
    judged = sorted(judge & set(threads))
    if judge and total and not judged:
        print(f"\nNo judged thread count ({sorted(judge)}) is in this run: nothing is flagged")
    elif judge and total and not attributed_any:
        print(f"\nFLAG: no step is attributed at {judged} threads: "
              f"the release-on-release step {total[0]} -> {total[1]} is not reproduced")
    return 0


def spread(t):
    """`ab run --spread`'s CPUs for t threads: the first CPU of each of the first t cores"""
    if t <= 1:
        return None
    status = open("/proc/self/status").read()
    allowed = next(l for l in status.splitlines() if l.startswith("Cpus_allowed_list:")).split(":")[1]
    cpus = []
    for part in allowed.strip().split(","):
        a, _, b = part.partition("-")
        cpus.extend(range(int(a), int(b or a) + 1))
    cores = []
    for c in cpus:
        try:
            sib = open(f"/sys/devices/system/cpu/cpu{c}/topology/thread_siblings_list").read().strip()
        except OSError:
            sib = str(c)
        core = []
        for part in sib.split(","):
            a, _, b = part.partition("-")
            core.extend(x for x in range(int(a), int(b or a) + 1) if x in cpus)
        core = sorted(core) or [c]
        if core not in cores:
            cores.append(core)
    cores.sort()
    return ",".join(str(c[0]) for c in cores[:t]) if t <= len(cores) else None


def perf_prefix():
    """How perf can count here: [] as this user, ['sudo', '-n'], or None, with why each failed"""
    whys = []
    for pre, who in (([], "as this user"), (["sudo", "-n"], "under sudo -n")):
        try:
            p = subprocess.run(pre + ["perf", "stat", "-x,", "-e", "instructions", "--", "true"],
                               capture_output=True, text=True, timeout=30)
        except (OSError, subprocess.TimeoutExpired) as e:
            whys.append(f"{who}: {e}")
            continue
        if p.returncode == 0 and "<not" not in p.stderr:
            return pre, None
        whys.append(f"{who}: " + (p.stderr.strip().splitlines() or ["perf failed"])[-1][:160])
    return None, "; ".join(whys)


def counters(args):
    pre, why = perf_prefix()
    out = open(args.out, "w")
    arms = [a.split("=", 1) for a in args.arm]
    threads = [int(t) for t in args.threads.split(",")]
    print(f"counters: perf {'as this user' if pre == [] else 'under sudo -n' if pre else 'unavailable: ' + why}")
    for t in threads:
        for lab, exe in arms:
            rec = {"label": lab, "threads": t}
            if pre is None:
                rec["why"] = why
                out.write(json.dumps(rec) + "\n")
                continue
            ins, llc, outb = [], [], None
            for _ in range(args.reps):
                with tempfile.NamedTemporaryFile("r", suffix=".csv") as tf:
                    cmd = pre + ["perf", "stat", "-x,", "-o", tf.name, "-e", "instructions,LLC-load-misses", "--"]
                    cpus = spread(t)
                    if cpus:
                        cmd += ["taskset", "-c", cpus]
                    cmd += [exe, "child", "mibz2", str(t), args.file]
                    p = subprocess.run(cmd, capture_output=True, text=True)
                    if p.returncode != 0:
                        rec["why"] = (p.stderr.strip().splitlines() or ["the decode failed"])[-1][:200]
                        break
                    vals = {}
                    for line in open(tf.name):
                        f = line.strip().split(",")
                        if len(f) > 2 and f[0] and f[0][0].isdigit():
                            vals[f[2]] = int(f[0])
                    child = json.loads(p.stdout.strip().splitlines()[-1])
                    outb = child.get("out")
                    if "instructions" not in vals or "LLC-load-misses" not in vals or not outb:
                        rec["why"] = f"counts not read: {sorted(vals)}"
                        break
                    ins.append(vals["instructions"] / outb)
                    llc.append(vals["LLC-load-misses"] / outb)
            if "why" not in rec:
                rec.update(instructions_per_byte=statistics.median(ins),
                           llc_misses_per_byte=statistics.median(llc), reps=len(ins), out=outb,
                           cpus=spread(t) or "unpinned", perf="sudo -n" if pre else "user")
            out.write(json.dumps(rec) + "\n")
            print(f"  {lab} at {t}: {rec.get('why') or 'counted'}", flush=True)
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("counters")
    c.add_argument("--arm", action="append", required=True)
    c.add_argument("--threads", required=True)
    c.add_argument("--file", required=True)
    c.add_argument("--out", required=True)
    c.add_argument("--reps", type=int, default=3)
    r = sub.add_parser("report")
    r.add_argument("runs")
    r.add_argument("--step", action="append", required=True)
    r.add_argument("--total")
    r.add_argument("--judge")
    r.add_argument("--counters")
    a = ap.parse_args(argv[1:])
    return counters(a) if a.cmd == "counters" else report(a)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
