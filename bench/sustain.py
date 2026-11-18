#!/usr/bin/env python3
"""Worker counts under sustained load (`just bench-sustain`; #145 QUEUED 6978994390, CHECKPOINT
6984075662 on #314; the registration docs/journal/2026-11-18-sustain-registration.md). Standard
library only.

MONODROMY's record 58 (REVIEW 6978967731) found the recalibration's bimodal regime to be a
chip-wide clock state with memory of recent load: after a rest a heavy decode runs at full turbo,
after a few heavy CPU-seconds every thread runs about 1.4x slower. Every worker optimum so far was
measured in the first state; a long decode runs in the second. This drives decodes back to back,
with no gap, so each arm is measured in the state its own sustained load puts the chip in, and
counts each decode's package energy, effective clock and DRAM bytes as it goes.

    sustain.py run --ab AB --cell NAME:PLACEMENT:T:ROUNDS:REF,ARM,.. [--cell ..] --main F
                   --guard F [--guard F] --out RUNS.jsonl [--warm 6] [--prewarm 60] [--gate 2]
    sustain.py probe --ab AB --arm ARM --threads T --placement unpinned --seconds 120 --file F
                   --out PROBE.jsonl [--gate 2] [--rest 30]
    sustain.py judged RUNS.jsonl --placement pinned|unpinned --out JUDGED.jsonl
    sustain.py report RUNS.jsonl [--probe PROBE.jsonl]

**A cell** is a placement and a thread count with its arms, the reference (the shipped trim)
first: `pinned` (each decode's affinity one CPU on each of the first T cores, as `ab run
--spread` pins) or `unpinned` (every online CPU, as `ab run --unpinned`). Its run:
1. *the gate:* the machine quiet, nothing of ours running: the other processes keep at most 1.5
   cores busy over `--gate` seconds (2), retried every 10 s up to 30 times, or the cell fails;
2. *the idle baseline:* `perf stat -a sleep 2`, the machine's own DRAM and package rate;
3. *the pre-warm:* the reference decoding the main corpus back to back for `--prewarm` seconds
   (60, as registered), so the chip and its cooler start the rounds in the sustained state;
4. *ROUNDS rounds:* in each, one block per arm, in an order rotated round by round and reversed
   every other cycle of rounds (`ab run`'s `arm_order`: over 2 x arms rounds, each arm in each
   position equally often). A block is the arm decoding the main corpus back to back for
   `--warm` seconds (6), then its measured decodes: the main corpus, each guard, and the main
   corpus again. Blocks follow one another with no gap: the load never stops within a cell.

**A decode** is `perf stat -a -x, -e power/energy-pkg/ -e cycles -e ref-cycles -e
uncore_imc/data_reads/ -e uncore_imc/data_writes/ -- AB child ARM T FILE`, its affinity set
before the exec (the child inherits it). Each event is its own `-e`, never one group (the
events are on three PMUs). Its record: the child's own (`wall`, the decode's; `cpu`, `rss_kb`,
`out`, `hash`, `thread_cpu`), the counts (`energy_j`, `cycles`, `ref_cycles`, `imc_bytes`, each
event's entries summed over its units, with the least enabled share), perf's elapsed time, the
other processes' busy cores over it (`/proc/stat`'s busy time less the child's CPU), and where
it ran (cell, round, position, phase: `idle`, `prewarm`, `warm`, `measure` or `probe`).

**judged** reads one placement's cells into `bench/two_arm.py`'s records, one per (round,
corpus, arm): the mean of its measured decodes' walls and CPU, the largest peak RSS, the output
(every decode's the same), the CPUs and the worker threads (from `thread_cpu`). It writes
nothing and exits 1 (VOID) where:
- a record is not JSON, or not a decode of a registered cell;
- a measured decode is missing, duplicated, failed (its exit, perf's, or the child's `err`), or
  has no output;
- an event of any measured or warm decode is not counted, not supported or multiplexed (enabled
  below 100%), or a count is not positive;
- a corpus's outputs differ, or an arm's decodes name different worker threads;
- **the sustained state not held:** in more than 10% of a cell's blocks, the measured
  main-corpus decodes' mean effective clock (cycles / ref-cycles) differs from the block's last
  warm decode's by more than 3%;
- **the pre-warm missing or failed** (`--prewarm`, 60 s; the Researcher's REVIEW 6985154079): a
  cell with no pre-warm decode, one failed or not fully counted, pre-warm decodes of more than one
  arm or of an arm the cell does not measure, or the last ending before the registered seconds;
- **turbo in a cell registered clamped** (`--clamped`): a measured main-corpus decode at a clock
  of 1.2 or above.

**report** decides nothing. Its diagnostics, each declared before the data:
- *by arm:* the measured main-corpus wall (median), package watts (energy over perf's elapsed
  time), the effective clock (cycles / ref-cycles, and in GHz at the TSC's `--tsc-ghz`, 3.7 on
  the i9), joules a GB of output, and DRAM bytes a byte of output (the IMC's, less the cell's
  idle rate over the decode's elapsed time);
- *order:* by position in the round, each arm's main-corpus wall over its own mean (log mean);
- *drift:* the reference's main-corpus wall and clock round by round, and their least-squares
  slope in % a round;
- *load:* the other processes' busy cores, the largest and the decodes above 1.0;
- *steady state:* the share of blocks within 3% (as judged's check);
- *the probe:* the clock and watts of each decode against time since its start, the first
  decode's clock (after the rest: the turbo state) against the last 60 s's, and the onset: the
  first time from which every decode's clock is within 2% of the last 60 s's mean.
"""
import argparse
import json
import math
import os
import statistics
import subprocess
import sys
import tempfile
import time

EVENTS = ["power/energy-pkg/", "cycles", "ref-cycles", "uncore_imc/data_reads/", "uncore_imc/data_writes/"]
FIELD = {"power/energy-pkg/": "energy_j", "cycles": "cycles", "ref-cycles": "ref_cycles",
         "uncore_imc/data_reads/": "imc_read_b", "uncore_imc/data_writes/": "imc_write_b"}
# perf's units, as joules or bytes; a bare memory-controller count is 64-byte lines
UNITS = {"Joules": 1.0, "B": 1, "KiB": 2**10, "MiB": 2**20, "GiB": 2**30}
STEADY, STEADY_SHARE, MAX_BUSY = 0.03, 0.10, 1.5
# the clamp screen (MONODROMY record 64's design note 1): in a cell registered clamped, a measured
# main-corpus decode at this effective clock (cycles / ref-cycles) or above ran in turbo, not clamped
CLAMP = 1.2


def spread(t):
    """The first CPU of each of the first t cores, as `ab run --spread`; None past the cores"""
    allowed = os.sched_getaffinity(0)
    cores = []
    for c in sorted(allowed):
        try:
            sib = open(f"/sys/devices/system/cpu/cpu{c}/topology/thread_siblings_list").read().strip()
        except OSError:
            sib = str(c)
        core = []
        for part in sib.split(","):
            a, _, b = part.partition("-")
            core.extend(x for x in range(int(a), int(b or a) + 1) if x in allowed)
        core = sorted(core) or [c]
        if core not in cores:
            cores.append(core)
    cores.sort()
    return [c[0] for c in cores[:t]] if t <= len(cores) else None


def online():
    """Every online CPU (`/sys/devices/system/cpu/online`), as `ab run --unpinned`"""
    try:
        s = open("/sys/devices/system/cpu/online").read().strip()
    except OSError:
        return sorted(os.sched_getaffinity(0))
    cpus = []
    for part in s.split(","):
        a, _, b = part.partition("-")
        cpus.extend(range(int(a), int(b or a) + 1))
    return cpus


def cpus_for(placement, t):
    return spread(t) if placement == "pinned" else online()


def busy_jiffies():
    """The machine's busy time so far, in seconds (`/proc/stat`'s aggregate line)"""
    try:
        f = open("/proc/stat").readline().split()[1:]
        v = [int(x) for x in f]
    except (OSError, ValueError):
        return None
    # user nice system idle iowait irq softirq steal
    return (v[0] + v[1] + v[2] + v[5] + v[6] + (v[7] if len(v) > 7 else 0)) / os.sysconf("SC_CLK_TCK")


def parse_perf(path):
    """perf stat -x, output: {field: [value or None, least enabled %, entries]}, each event's
    entries summed (one per memory controller), a failed entry failing the sum"""
    got = {}
    try:
        lines = open(path).read().splitlines()
    except OSError:
        return got
    for line in lines:
        if not line.strip() or line.startswith("#"):
            continue
        f = line.split(",")
        if len(f) < 5:
            continue
        value, unit, event = f[0], f[1], f[2]
        key = FIELD.get(event)
        if key is None:
            # perf names an uncore event per unit (`uncore_imc_0/data_reads/`) without merging
            base = event.split("/")[0].rsplit("_", 1)[0] + "/" + "/".join(event.split("/")[1:])
            key = FIELD.get(base)
            if key is None:
                continue
        try:
            pct = float(f[4]) if f[4] else 100.0
        except ValueError:
            pct = 0.0
        try:
            v = float(value)
        except ValueError:
            v = None
        if v is not None:
            if unit in UNITS:
                v *= UNITS[unit]
            elif key.startswith("imc"):
                v *= 64
        if key in got:
            prev = got[key]
            got[key] = [prev[0] + v if prev[0] is not None and v is not None else None, min(prev[1], pct), prev[2] + 1]
        else:
            got[key] = [v, pct, 1]
    return got


def decode(ab, arm, t, path, cpus, perf="perf"):
    """One decode under perf stat -a, its affinity `cpus`: its record (the child's, the counts)"""
    rec = {"arm": arm, "threads": t, "file": path, "cpus": ",".join(map(str, cpus))}
    with tempfile.NamedTemporaryFile("r", suffix=".csv") as tf:
        cmd = [perf, "stat", "-a", "-x,", "-o", tf.name] + sum((["-e", e] for e in EVENTS), [])
        cmd += ["--", ab, "child", arm, str(t), path]
        b0, t0 = busy_jiffies(), time.monotonic()
        try:
            p = subprocess.run(cmd, capture_output=True, text=True, preexec_fn=lambda: os.sched_setaffinity(0, cpus))
        except OSError as e:
            rec["err"] = f"perf did not start: {e}"
            return rec
        rec["elapsed"] = time.monotonic() - t0
        b1 = busy_jiffies()
        rec["rc"] = p.returncode
        if p.returncode != 0:
            rec["err"] = (p.stderr.strip().splitlines() or ["failed"])[-1][:300]
        try:
            child = json.loads(p.stdout.strip().splitlines()[-1])
            for k in ("wall", "cpu", "rss_kb", "out", "thread_cpu"):
                rec[k] = child.get(k)
            rec["hash"] = child.get("sha256") or child.get("hash")
            rec["child_err"] = child.get("err") or ""
        except (ValueError, IndexError):
            rec.setdefault("err", "no child record")
        rec["counts"] = parse_perf(tf.name)
        if b0 is not None and b1 is not None and isinstance(rec.get("cpu"), (int, float)):
            rec["others_busy"] = max(0.0, (b1 - b0 - rec["cpu"])) / rec["elapsed"]
    return rec


def idle(seconds, perf="perf"):
    """The machine idle, `seconds` long, under the same events: the cell's baseline"""
    with tempfile.NamedTemporaryFile("r", suffix=".csv") as tf:
        cmd = [perf, "stat", "-a", "-x,", "-o", tf.name] + sum((["-e", e] for e in EVENTS), [])
        t0 = time.monotonic()
        p = subprocess.run(cmd + ["--", "sleep", str(seconds)], capture_output=True, text=True)
        return {"phase": "idle", "elapsed": time.monotonic() - t0, "rc": p.returncode, "counts": parse_perf(tf.name)}


def gate(seconds, tries=30, wait=10.0):
    """Wait until the other processes keep at most MAX_BUSY cores busy over `seconds` (nothing of
    ours runs): (busy cores, tries taken), or (busy, None) if it never was"""
    busy = None
    for i in range(tries):
        if seconds <= 0:
            return 0.0, 0
        b0 = busy_jiffies()
        time.sleep(seconds)
        b1 = busy_jiffies()
        if b0 is None or b1 is None:
            return None, i
        busy = (b1 - b0) / seconds
        if busy <= MAX_BUSY:
            return busy, i
        time.sleep(wait)
    return busy, None


def order(arms, r):
    """`ab run`'s arm order: rotated by the round, reversed every other cycle of rounds"""
    k = len(arms)
    rot = arms[r % k:] + arms[:r % k]
    return list(reversed(rot)) if (r // k) % 2 else rot


def parse_cell(spec):
    name, place, t, rounds, arms = spec.split(":")
    if place not in ("pinned", "unpinned"):
        raise ValueError(f"--cell {spec}: placement pinned or unpinned")
    arms = arms.split(",")
    if len(arms) < 2 or len(set(arms)) != len(arms):
        raise ValueError(f"--cell {spec}: the reference and one arm or more, distinct")
    return {"name": name, "placement": place, "threads": int(t), "rounds": int(rounds), "arms": arms}


def corpus(path):
    return os.path.basename(path).removesuffix(".bz2")


def run(a):
    cells = [parse_cell(s) for s in a.cell]
    seq = 0
    with open(a.out, "w") as out:
        prev = {"arm": None, "block": None}

        def emit(rec, **kw):
            nonlocal seq
            rec.update(kw, seq=seq, corpus=corpus(rec["file"]) if "file" in rec else None,
                       prev_arm=prev["arm"], prev_block=prev["block"])
            seq += 1
            if "arm" in rec:
                prev["arm"] = rec["arm"]
            out.write(json.dumps(rec) + "\n")
            out.flush()
            return rec

        for cell in cells:
            t, place = cell["threads"], cell["placement"]
            cpus = cpus_for(place, t)
            where = dict(cell=cell["name"], placement=place)
            if cpus is None:
                print(f"sustain: {cell['name']}: {t} cores not available to pin", file=sys.stderr)
                return 1
            busy, tries = gate(a.gate)
            out.write(json.dumps({"phase": "gate", **where, "busy": busy, "tries": tries}) + "\n")
            if tries is None:
                print(f"sustain: {cell['name']}: the machine never quiet (busy {busy})", file=sys.stderr)
                return 1
            emit(idle(a.idle, a.perf), **where)
            prev.update(arm=None, block=None)
            t_cell = time.monotonic()
            t0 = time.monotonic()
            while time.monotonic() - t0 < a.prewarm:
                rec = decode(a.ab, cell["arms"][0], t, a.main, cpus, a.perf)
                emit(rec, **where, phase="prewarm", since=time.monotonic() - t_cell)
            print(f"{cell['name']}: pre-warmed {a.prewarm} s on {cell['arms'][0]}", flush=True)
            for r in range(cell["rounds"]):
                for pos, arm in enumerate(order(cell["arms"], r)):
                    b = dict(where, round=r, position=pos)
                    t0 = time.monotonic()
                    while True:
                        rec = decode(a.ab, arm, t, a.main, cpus, a.perf)
                        emit(rec, **b, phase="warm", since=time.monotonic() - t_cell)
                        if time.monotonic() - t0 >= a.warm:
                            break
                    for i, f in enumerate([a.main, *a.guard, a.main]):
                        rec = decode(a.ab, arm, t, f, cpus, a.perf)
                        emit(rec, **b, phase="measure", index=i, since=time.monotonic() - t_cell)
                    prev["block"] = arm
                    print(f"{cell['name']} round {r} {arm}: "
                          f"{'failed: ' + rec['err'] if rec.get('err') else 'measured'}", flush=True)
    return 0


def probe(a):
    cpus = cpus_for(a.placement, a.threads)
    busy, tries = gate(a.gate)
    with open(a.out, "w") as out:
        out.write(json.dumps({"phase": "gate", "busy": busy, "tries": tries}) + "\n")
        if tries is None:
            return 1
        time.sleep(a.rest)
        t0, seq = time.monotonic(), 0
        while time.monotonic() - t0 < a.seconds:
            rec = decode(a.ab, a.arm, a.threads, a.file, cpus, a.perf)
            rec.update(phase="probe", since=time.monotonic() - t0, seq=seq, placement=a.placement)
            seq += 1
            out.write(json.dumps(rec) + "\n")
            out.flush()
    return 0


def load(path, errors):
    rows = []
    if not os.path.exists(path):
        errors.append(f"{path}: missing")
        return rows
    for ln, line in enumerate(open(path), 1):
        if not line.strip():
            continue
        try:
            r = json.loads(line)
        except ValueError:
            errors.append(f"line {ln}: not JSON")
            continue
        if not isinstance(r, dict):
            errors.append(f"line {ln}: not a record")
            continue
        rows.append(r)
    return rows


def clock(r):
    c = r.get("counts") or {}
    try:
        return c["cycles"][0] / c["ref_cycles"][0]
    except (KeyError, TypeError, ZeroDivisionError):
        return None


def failed(r):
    return r.get("err") or r.get("rc") != 0 or r.get("child_err")


def check_counts(r, key, errors):
    counts = r.get("counts") or {}
    for e, field in FIELD.items():
        c = counts.get(field)
        if c is None:
            errors.append(f"{key}: {e} missing")
        elif c[0] is None:
            errors.append(f"{key}: {e} not counted")
        elif c[1] < 100.0:
            errors.append(f"{key}: {e} multiplexed ({c[1]:.1f}% enabled)")
        elif c[0] <= 0:
            errors.append(f"{key}: {e} not positive ({c[0]})")


def workers(r):
    names = [p.partition("=")[0] for p in str(r.get("thread_cpu") or "").split()]
    return sorted(n for n in names if n.startswith("mibz2-") and n[6:].isdigit())


def blocks(rows, placement, errors):
    """The placement's measured blocks {(cell, threads, round, arm): {"measure": [..], "warm": [..]}},
    after checking every decode"""
    out, outputs = {}, {}
    for r in rows:
        if r.get("placement") != placement or r.get("phase") not in ("warm", "measure"):
            continue
        try:
            key = (r["cell"], int(r["threads"]), int(r["round"]), r["arm"])
        except (KeyError, TypeError, ValueError):
            errors.append(f"seq {r.get('seq')}: not a decode of a cell")
            continue
        b = out.setdefault(key, {"measure": {}, "warm": []})
        if r["phase"] == "warm":
            b["warm"].append(r)
        else:
            i = r.get("index")
            if i in b["measure"]:
                errors.append(f"{key} measured decode {i} duplicated")
            b["measure"][i] = r
        if failed(r):
            errors.append(f"{key} {r['phase']} seq {r.get('seq')}: failed: {r.get('err') or r.get('child_err') or r.get('rc')}")
            continue
        if not isinstance(r.get("out"), int) or r["out"] <= 0 or not r.get("hash"):
            errors.append(f"{key} seq {r.get('seq')}: no output")
        else:
            outputs.setdefault(r.get("corpus"), set()).add((r["out"], r["hash"]))
        check_counts(r, f"{key} seq {r.get('seq')}", errors)
    for c, outs in sorted(outputs.items()):
        if len(outs) != 1:
            errors.append(f"{c}: the decodes' outputs differ: {sorted(outs)[:3]}")
    return out


def steady(bs, n_measure, errors, report=None):
    """The sustained state held: the share of each cell's blocks whose measured main-corpus
    decodes' mean clock is within STEADY of the last warm decode's"""
    by_cell = {}
    for key, b in sorted(bs.items()):
        ms = [b["measure"].get(i) for i in range(n_measure)]
        if any(m is None for m in ms) or not b["warm"]:
            errors.append(f"{key}: {sum(m is None for m in ms)} measured decodes missing, {len(b['warm'])} warm")
            continue
        main = [clock(ms[0]), clock(ms[-1])]
        last = clock(max(b["warm"], key=lambda r: r.get("seq", 0)))
        if None in main or last is None:
            continue
        dev = statistics.mean(main) / last - 1
        by_cell.setdefault(key[0], []).append(abs(dev) <= STEADY)
        if report is not None:
            report.append((key, dev))
    for cell, ok in sorted(by_cell.items()):
        share = 1 - sum(ok) / len(ok)
        if share > STEADY_SHARE:
            errors.append(f"{cell}: the sustained state not held: {share:.0%} of blocks off by more than "
                          f"{STEADY:.0%} (at most {STEADY_SHARE:.0%})")
    return by_cell


def clamp_screen(bs, clamped, errors):
    """In the cells registered clamped, every measured main-corpus decode below CLAMP"""
    for key, b in sorted(bs.items()):
        if key[0] not in clamped or not b["measure"]:
            continue
        main = b["measure"][min(b["measure"])].get("corpus")
        for i, m in sorted(b["measure"].items()):
            c = clock(m)
            if m.get("corpus") == main and c is not None and c >= CLAMP:
                errors.append(f"{key} measured decode {i}: clock {c:.3f}, at or above {CLAMP} in a cell "
                              f"registered clamped (turbo, not the sustained state)")


def prewarm_check(rows, placement, seconds, errors):
    """Each cell of the placement pre-warmed: at least one pre-warm decode, every one of them a
    success with every event counted, all of one arm that the cell also measures, and the last
    ending at least `seconds` after the cell's start (the Researcher's REVIEW 6985154079)"""
    cells = {}
    for r in rows:
        if r.get("placement") == placement and r.get("phase") in ("prewarm", "measure") and "cell" in r:
            cells.setdefault(r["cell"], {"prewarm": [], "arms": set()})
            if r["phase"] == "prewarm":
                cells[r["cell"]]["prewarm"].append(r)
            else:
                cells[r["cell"]]["arms"].add(r.get("arm"))
    for cell, c in sorted(cells.items()):
        pw = c["prewarm"]
        if not pw:
            errors.append(f"{cell}: no pre-warm")
            continue
        for r in pw:
            key = f"{cell} prewarm seq {r.get('seq')}"
            if failed(r):
                errors.append(f"{key}: failed: {r.get('err') or r.get('child_err') or r.get('rc')}")
            else:
                check_counts(r, key, errors)
        arms = {r.get("arm") for r in pw}
        if len(arms) != 1 or not arms <= c["arms"]:
            errors.append(f"{cell}: the pre-warm's arms {sorted(map(str, arms))}: not one arm the cell measures")
        end = max((r.get("since") for r in pw if isinstance(r.get("since"), (int, float))), default=None)
        if end is None or end < seconds:
            errors.append(f"{cell}: the pre-warm ended at {end if end is None else round(end, 1)} s, "
                          f"short of the registered {seconds:g} s")


def judged(a):
    errors = []
    rows = load(a.runs, errors)
    prewarm_check(rows, a.placement, a.prewarm, errors)
    bs = blocks(rows, a.placement, errors)
    clamp_screen(bs, set(c for c in a.clamped.split(",") if c), errors)
    n_measure = 1 + max((r.get("index", 0) for r in rows if r.get("phase") == "measure"), default=0)
    steady(bs, n_measure, errors)
    recs = []
    if not bs:
        errors.append(f"no {a.placement} decodes")
    for key, b in sorted(bs.items()):
        cell, t, rnd, arm = key
        ms = [b["measure"][i] for i in sorted(b["measure"])]
        names = {tuple(workers(m)) for m in ms + b["warm"]}
        if len(names) != 1:
            errors.append(f"{key}: its decodes name different worker threads {sorted(names)}")
        by_corpus = {}
        for m in ms:
            by_corpus.setdefault(m["corpus"], []).append(m)
        for c, ds in by_corpus.items():
            if any(failed(d) or not isinstance(d.get("wall"), (int, float)) for d in ds):
                continue
            w = list(names)[0] if names else ()
            recs.append({"round": rnd, "corpus": c, "threads": t, "arm": arm, "termination": "success",
                         "wall": statistics.mean(d["wall"] for d in ds),
                         "cpu": statistics.mean(d["cpu"] for d in ds), "rss_kb": max(d["rss_kb"] for d in ds),
                         "out": ds[0]["out"], "hash": ds[0]["hash"], "cpus": ds[0]["cpus"],
                         "observed_threads": " ".join(["ab", *w, "mibz2-read", "mibz2-scan"]),
                         "decodes": len(ds), "cell": cell})
    if errors:
        print(f"VOID: the {a.placement} decodes do not make the registered run ({len(errors)} problems); "
              f"no records written")
        for e in errors[:25]:
            print(f"  {e}")
        return 1
    with open(a.out, "w") as f:
        for r in recs:
            f.write(json.dumps(r) + "\n")
    print(f"{a.placement}: {len(recs)} records from {len(bs)} blocks, the sustained state held")
    return 0


def mean_or(xs):
    return statistics.mean(xs) if xs else float("nan")


def safe(f):
    try:
        return f()
    except (KeyError, TypeError, ZeroDivisionError, IndexError):
        return None


def slope(ys):
    n = len(ys)
    if n < 2:
        return float("nan")
    xm, ym = (n - 1) / 2, statistics.mean(ys)
    den = sum((i - xm) ** 2 for i in range(n))
    return sum((i - xm) * (y - ym) for i, y in enumerate(ys)) / den


def watts(r):
    try:
        return r["counts"]["energy_j"][0] / r["elapsed"]
    except (KeyError, TypeError, ZeroDivisionError):
        return None


def report(a):
    errors = []
    rows = load(a.runs, errors)
    idle_rate = {}
    for r in rows:
        if r.get("phase") == "idle" and r.get("counts"):
            c = r["counts"]
            try:
                idle_rate[r["cell"]] = (c["imc_read_b"][0] + c["imc_write_b"][0]) / r["elapsed"]
            except (KeyError, TypeError):
                pass
    cells = list(dict.fromkeys(r["cell"] for r in rows if r.get("phase") == "measure"))
    for cell in cells:
        ms = [r for r in rows if r.get("cell") == cell and r.get("phase") == "measure" and not failed(r)]
        if not ms:
            print(f"\n== {cell}: no measured decode succeeded")
            continue
        main = ms[0]["corpus"]
        mains = [r for r in ms if r["corpus"] == main]
        arms = list(dict.fromkeys(r["arm"] for r in mains))
        ref = next((r["arm"] for r in rows if r.get("cell") == cell and r.get("phase") == "prewarm"), arms[0] if arms else None)
        print(f"\n== {cell} ({ms[0]['placement']}, {ms[0]['threads']} threads; reference {ref}; main corpus {main})")
        print("  arm: wall (median s) | package W | clock (cycles/ref-cycles; GHz at TSC) | J a GB out | DRAM B a byte out")
        for arm in arms:
            ds = [r for r in mains if r["arm"] == arm]
            w = statistics.median(d["wall"] for d in ds)
            pw = [watts(d) for d in ds if watts(d) is not None]
            ck = [clock(d) for d in ds if clock(d) is not None]
            jgb = [x for x in (safe(lambda: d["counts"]["energy_j"][0] / (d["out"] / 1e9)) for d in ds) if x is not None]
            dram = [x for x in (safe(lambda: ((d["counts"]["imc_read_b"][0] + d["counts"]["imc_write_b"][0])
                                              - idle_rate.get(cell, 0.0) * d["elapsed"]) / d["out"]) for d in ds)
                    if x is not None]
            med = lambda xs, f: f.format(statistics.median(xs)) if xs else "?"  # noqa: E731
            print(f"  {arm}: {w:.3f} | {med(pw, '{:.1f}')} | {med(ck, '{:.3f}')} "
                  f"({med([c * a.tsc_ghz for c in ck], '{:.2f}')}) | {med(jgb, '{:.2f}')} | {med(dram, '{:.2f}')}")
        print("  order (each arm's main-corpus wall over its own mean, log mean by position):")
        pos = {}
        for arm in arms:
            ds = [r for r in mains if r["arm"] == arm]
            m = statistics.mean(math.log(d["wall"]) for d in ds)
            for d in ds:
                pos.setdefault(d["position"], []).append(math.log(d["wall"]) - m)
        print("   " + "  ".join(f"position {p}: {math.exp(statistics.mean(v)):.4f}" for p, v in sorted(pos.items())))
        rr = sorted({r["round"] for r in mains})
        rw = [statistics.mean(d["wall"] for d in mains if d["arm"] == ref and d["round"] == i) for i in rr]
        rc = [mean_or([c for d in mains if d["arm"] == ref and d["round"] == i
                       for c in [clock(d)] if c is not None]) for i in rr]
        print(f"  drift ({ref}, round by round): wall " + " ".join(f"{w:.3f}" for w in rw)
              + f" (slope {100 * slope(rw) / statistics.mean(rw):+.2f}% a round); clock "
              + " ".join(f"{c:.3f}" for c in rc) + f" (slope {100 * slope(rc) / statistics.mean(rc):+.2f}% a round)")
        ob = [r["others_busy"] for r in rows if r.get("cell") == cell and isinstance(r.get("others_busy"), (int, float))]
        if ob:
            print(f"  load: other processes' busy cores, largest {max(ob):.2f}, decodes above 1.0: "
                  f"{sum(o > 1.0 for o in ob)} of {len(ob)}")
        mc = [clock(r) for r in mains if clock(r) is not None]
        state = ("clamped" if mc and all(c < CLAMP for c in mc) else "turbo" if mc and all(c >= CLAMP for c in mc)
                 else "mixed")
        print(f"  the cell's state (its measured main-corpus decodes): {state}, "
              f"{sum(c < CLAMP for c in mc)} of {len(mc)} below {CLAMP}")
        print(f"  clock state (main-corpus decodes, warm and measured; at or above {CLAMP} is turbo):")
        for arm in arms:
            cs = [clock(r) for r in rows if r.get("cell") == cell and r.get("arm") == arm and r.get("corpus") == main
                  and r.get("phase") in ("warm", "measure") and clock(r) is not None]
            gs = [clock(r) for r in ms if r["arm"] == arm and r["corpus"] != main and clock(r) is not None]
            if cs:
                print(f"   {arm}: {sum(c < CLAMP for c in cs)} of {len(cs)} clamped (median {statistics.median(cs):.3f}); "
                      f"guards' median {statistics.median(gs) if gs else float('nan'):.3f}")
        print("  carry-over (each arm's measured main-corpus wall over its own mean, by the block before it):")
        for arm in arms:
            ds = [r for r in mains if r["arm"] == arm]
            m = statistics.mean(math.log(d["wall"]) for d in ds)
            by = {}
            for d in ds:
                by.setdefault(d.get("prev_block"), []).append(math.log(d["wall"]) - m)
            print(f"   {arm}: " + "  ".join(f"after {k or 'the pre-warm'}: {math.exp(statistics.mean(v)):.4f}"
                                           for k, v in sorted(by.items(), key=lambda kv: str(kv[0]))))
        devs = []
        bs = {k: v for k, v in blocks([r for r in rows if r.get("cell") == cell], ms[0]["placement"], []).items()}
        steady(bs, 1 + max(r.get("index", 0) for r in ms), [], devs)
        if devs:
            off = sum(abs(d) > STEADY for _, d in devs)
            print(f"  steady state: {len(devs) - off} of {len(devs)} blocks within {STEADY:.0%} "
                  f"(largest {max(abs(d) for _, d in devs):.1%})")
    if a.probe:
        pr = [r for r in load(a.probe, errors) if r.get("phase") == "probe" and not failed(r)]
        if pr:
            end = pr[-1]["since"]
            last = [clock(r) for r in pr if r["since"] >= end - 60 and clock(r) is not None]
            level = statistics.mean(last) if last else float("nan")
            print(f"\n== the probe ({pr[0]['arm']}, {pr[0]['threads']} threads, {pr[0].get('placement')}): "
                  f"{len(pr)} decodes over {end:.0f} s")
            print("  since s: clock (GHz at TSC), package W, wall s")
            for r in pr:
                c, w = clock(r), watts(r)
                print(f"  {r['since']:6.1f}: {c if c is None else round(c, 3)} "
                      f"({'?' if c is None else f'{c * a.tsc_ghz:.2f}'}), {'?' if w is None else f'{w:.1f}'}, {r['wall']:.3f}")
            first = clock(pr[0])
            onset = None
            for i, r in enumerate(pr):
                if all(clock(x) is not None and abs(clock(x) / level - 1) <= 0.02 for x in pr[i:]):
                    onset = r["since"]
                    break
            print(f"  the first decode's clock {first:.3f} against the last 60 s's {level:.3f} "
                  f"({first / level:.3f}x); onset (every later decode within 2%): "
                  f"{'never' if onset is None else f'{onset:.1f} s'}")
    for e in errors[:10]:
        print(f"  (unreadable: {e})")
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("--ab", required=True)
    r.add_argument("--cell", action="append", required=True)
    r.add_argument("--main", required=True)
    r.add_argument("--guard", action="append", default=[])
    r.add_argument("--out", required=True)
    r.add_argument("--warm", type=float, default=6.0)
    r.add_argument("--prewarm", type=float, default=60.0)
    r.add_argument("--gate", type=float, default=2.0)
    r.add_argument("--idle", type=float, default=2.0)
    r.add_argument("--perf", default="perf")
    p = sub.add_parser("probe")
    p.add_argument("--ab", required=True)
    p.add_argument("--arm", required=True)
    p.add_argument("--threads", type=int, required=True)
    p.add_argument("--placement", choices=("pinned", "unpinned"), default="unpinned")
    p.add_argument("--seconds", type=float, default=120.0)
    p.add_argument("--file", required=True)
    p.add_argument("--out", required=True)
    p.add_argument("--gate", type=float, default=2.0)
    p.add_argument("--rest", type=float, default=30.0)
    p.add_argument("--perf", default="perf")
    j = sub.add_parser("judged")
    j.add_argument("runs")
    j.add_argument("--placement", choices=("pinned", "unpinned"), required=True)
    j.add_argument("--out", required=True)
    j.add_argument("--clamped", default="", help="the cells registered clamped, comma-separated (the clock screen)")
    j.add_argument("--prewarm", type=float, default=60.0,
                   help="the registered pre-warm in seconds: each cell's must have lasted this long, and succeeded")
    q = sub.add_parser("report")
    q.add_argument("runs")
    q.add_argument("--probe")
    q.add_argument("--tsc-ghz", type=float, default=3.7)
    a = ap.parse_args(argv[1:])
    return {"run": run, "probe": probe, "judged": judged, "report": report}[a.cmd](a)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
