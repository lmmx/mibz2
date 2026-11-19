#!/usr/bin/env python3
"""Hardware counters for two decoder arms, single-threaded and at T threads: where the time
goes, not how much (`just bench-stop-walk-counters`; #145, the Coordinator's approval of the
Pathfinder's request in REVIEW 6924288835). A diagnostic: it records and decides nothing.
Standard library only.

    counters.py run --ab AB --arm A --arm B --threads 1,10 --reps 10 --file F --out C.jsonl [--events l2|imc] [--unpinned]
    counters.py report C.jsonl --ref A --cand B --threads 1,10 --reps 10 [--events l2|imc]

**Arms.** `NAME` runs `AB child NAME`; `NAME@EXE` runs another build's harness, `EXE child NAME`
(as `ab run`'s arms), its records keeping the arm as given.

**Events.** The default: the three groups below. `--events l2`: one group, cycles, instructions,
L2 demand-request misses and references (`l2_rqsts.miss`, `l2_rqsts.references`), for W12's L2
diagnostic (#145, the Coordinator's ruling on #506).

`--events imc`: the memory controller's read and write bytes (`uncore_imc/data_reads/`,
`data_writes/`), counted system-wide (`perf stat -a`: `kernel.perf_event_paranoid` at 0 or less),
for `just bench-admit`'s DRAM diagnostic (#145 QUEUED 6973253342). Each rep first counts the
machine idle (`sleep`, `COUNTERS_IDLE_S` seconds, 1 by default: the record `idle`), and the report
takes the idle rate's median, times each run's elapsed time, off that run's bytes. perf reports
these counts scaled (MiB and the like): they are read as bytes; an event counted on several
memory controllers is summed.

`--unpinned`: no run is pinned (`just bench-admit` registers its cells unpinned).

**How it counts.** Each rep runs each arm (`ab child ARM T FILE`) once per event group under
`perf stat -x,` as this user (the workstation's `kernel.perf_event_paranoid` at 1 or less),
pinned as `ab run --spread` pins: the first CPU of each of the first T cores. The groups hold
four events each, cycles in every one, so no event is multiplexed. The arms alternate in order
from rep to rep.

**VOID** (exit 1, nothing reported), the report checks before it reads:
- any (arm, threads, rep, group) missing or duplicated;
- a run whose perf or decode failed;
- any event of a group not counted, unsupported or multiplexed (enabled below 100%);
- the arms' outputs differing.

**What it reports.** For each arm and thread count, the median over the reps of each event per
output byte. Then the candidate over the reference, as the median of the reps' paired ratios.
Then derived figures: instructions a cycle, branch misses a thousand instructions, store-forward
blocks a thousand instructions, and the stall fractions of cycles.
"""

import argparse
import json
import os
import statistics
import subprocess
import sys
import tempfile
import time

EVENT_SETS = {
    "default": [
        ["cycles", "instructions", "branches", "branch-misses"],
        ["cycles", "L1-dcache-load-misses", "LLC-load-misses", "ld_blocks.store_forward"],
        [
            "cycles",
            "cycle_activity.stalls_total",
            "cycle_activity.stalls_mem_any",
            "cycle_activity.stalls_l3_miss",
        ],
    ],
    "l2": [["cycles", "instructions", "l2_rqsts.miss", "l2_rqsts.references"]],
    "imc": [["uncore_imc/data_reads/", "uncore_imc/data_writes/"]],
    # the memory controller and the package's energy (RAPL), system-wide: the open-return
    # comparison's diagnostic (#145 QUEUED 6978711740; its settle and power additions)
    "imcpower": [["uncore_imc/data_reads/", "uncore_imc/data_writes/", "power/energy-pkg/"]],
}
# the event sets counted system-wide (`perf stat -a`), with an idle baseline each rep
SYSTEM_WIDE = {"imc", "imcpower"}
# perf's scaled units, as bytes; a bare count of the memory controller's is 64-byte lines
SCALE = {"B": 1, "KiB": 2**10, "MiB": 2**20, "GiB": 2**30, "": 64, "Joules": 1}
GROUPS = EVENT_SETS["default"]


def harness(ab, arm):
    """The harness and the arm's name for `ab child`: `NAME@EXE` runs EXE"""
    name, at, exe = arm.partition("@")
    return (exe, name) if at else (ab, arm)


def spread(t):
    """The first CPU of each of the first t cores, as `ab run --spread`; None past the cores"""
    status = open("/proc/self/status").read()
    allowed = next(l for l in status.splitlines() if l.startswith("Cpus_allowed_list:")).split(":")[
        1
    ]
    cpus = []
    for part in allowed.strip().split(","):
        a, _, b = part.partition("-")
        cpus.extend(range(int(a), int(b or a) + 1))
    cores = []
    for c in cpus:
        try:
            sib = (
                open(f"/sys/devices/system/cpu/cpu{c}/topology/thread_siblings_list").read().strip()
            )
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


def parse_perf(path, scaled=False):
    """perf stat -x, output: {event: (value or None, enabled percent, raw)}; `scaled` (the memory
    controller's): the value in bytes from perf's unit, an event on several units summed"""
    got = {}
    for line in open(path):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        f = line.split(",")
        if len(f) < 5:
            continue
        value, unit, event = f[0], f[1], f[2]
        try:
            pct = float(f[4]) if f[4] else 100.0
        except ValueError:
            pct = 0.0
        try:
            v = int(value)
        except ValueError:
            try:
                v = int(float(value))
            except ValueError:
                v = None
        if scaled:
            try:
                v = int(float(value) * SCALE[unit])
            except (ValueError, KeyError):
                v = None
            # an event on several controllers: summed, a failed entry failing the sum (never
            # overwritten by a later valid one), and the entries counted (the report checks
            # every run has as many)
            if event in got:
                prev = got[event]
                total = prev[0] + v if prev[0] is not None and v is not None else None
                got[event] = (total, min(prev[1], pct), prev[2] + ";" + value, prev[3] + 1)
            else:
                got[event] = (v, pct, value, 1)
            continue
        got[event] = (v, pct, value)
    return got


def counted(cmd, events, system_wide):
    """cmd under perf stat (system-wide: `-a`): (the record's fields, the counts)"""
    rec = {}
    with tempfile.NamedTemporaryFile("r", suffix=".csv") as tf:
        full = (
            ["perf", "stat"]
            + (["-a"] if system_wide else [])
            + ["-x,", "-o", tf.name, "-e", ",".join(events), "--"]
        )
        t0 = time.monotonic()
        try:
            p = subprocess.run(full + cmd, capture_output=True, text=True)
        except OSError as e:
            return {"err": f"perf did not start: {e}"}, None
        rec["elapsed"] = time.monotonic() - t0
        rec["rc"] = p.returncode
        if p.returncode != 0:
            rec["err"] = (p.stderr.strip().splitlines() or ["failed"])[-1][:300]
        return rec, (p.stdout, parse_perf(tf.name, scaled=system_wide))


def run(a):
    global GROUPS
    GROUPS = EVENT_SETS[a.events]
    system_wide = a.events in SYSTEM_WIDE
    threads = [int(t) for t in a.threads.split(",")]
    with open(a.out, "w") as out:
        for rep in range(a.reps):
            if system_wide:
                # the machine idle, this rep: the baseline the report takes off each run
                secs = os.environ.get("COUNTERS_IDLE_S", "1")
                rec, got = counted(["sleep", secs], GROUPS[0], True)
                rec.update(arm="idle", threads=0, rep=rep, group=0, events=GROUPS[0])
                if got is not None:
                    rec["counts"] = {e: list(v) for e, v in got[1].items()}
                out.write(json.dumps(rec) + "\n")
                print(f"  rep {rep} idle: {rec.get('err') or 'counted'}", flush=True)
            arms = a.arm if rep % 2 == 0 else list(reversed(a.arm))
            for t in threads:
                for arm in arms:
                    for g, events in enumerate(GROUPS):
                        cpus = None if a.unpinned else spread(t)
                        exe, name = harness(a.ab, arm)
                        cmd = (["taskset", "-c", cpus] if cpus else []) + [
                            exe,
                            "child",
                            name,
                            str(t),
                            a.file,
                        ]
                        rec, got = counted(cmd, events, system_wide)
                        rec.update(
                            arm=arm,
                            threads=t,
                            rep=rep,
                            group=g,
                            events=events,
                            cpus=cpus or "unpinned",
                        )
                        if got is not None and "err" not in rec:
                            try:
                                child = json.loads(got[0].strip().splitlines()[-1])
                                rec.update(
                                    out=child.get("out"),
                                    hash=child.get("sha256") or child.get("hash"),
                                    wall=child.get("wall"),
                                    child_err=child.get("err"),
                                )
                            except (ValueError, IndexError):
                                rec["err"] = "no child record"
                            rec["counts"] = {e: list(v) for e, v in got[1].items()}
                        out.write(json.dumps(rec) + "\n")
                        print(
                            f"  rep {rep} t={t} {arm} group {g}: {rec.get('err') or 'counted'}",
                            flush=True,
                        )
    return 0


def report(a):
    global GROUPS
    GROUPS = EVENT_SETS[a.events]
    threads = [int(t) for t in a.threads.split(",")]
    arms = [a.ref, a.cand]
    system_wide = a.events in SYSTEM_WIDE
    roster = {
        (arm, t, r, g)
        for arm in arms
        for t in threads
        for r in range(a.reps)
        for g in range(len(GROUPS))
    }
    if system_wide:
        roster |= {("idle", 0, r, 0) for r in range(a.reps)}
    rows, errors, outputs = {}, [], set()
    if not os.path.exists(a.runs):
        errors.append(f"{a.runs}: missing")
    else:
        for ln, line in enumerate(open(a.runs), 1):
            if not line.strip():
                continue
            try:
                r = json.loads(line)
                key = (r["arm"], int(r["threads"]), int(r["rep"]), int(r["group"]))
            except (ValueError, KeyError, TypeError):
                errors.append(f"line {ln}: not a record")
                continue
            if key in rows:
                errors.append(f"{key} duplicated")
                continue
            if key not in roster:
                errors.append(f"{key} not in the roster")
            rows[key] = r
            if r.get("err") or r.get("rc") != 0 or r.get("child_err"):
                errors.append(
                    f"{key}: the run failed: {r.get('err') or r.get('child_err') or r.get('rc')}"
                )
                continue
            if system_wide and not (
                isinstance(r.get("elapsed"), (int, float)) and r["elapsed"] > 0
            ):
                errors.append(f"{key}: no elapsed time")
            if key[0] != "idle":
                outputs.add((r.get("out"), r.get("hash")))
                if not isinstance(r.get("out"), int) or r["out"] <= 0:
                    errors.append(f"{key}: no output size")
            counts = r.get("counts") or {}
            for e in GROUPS[key[3]]:
                c = counts.get(e)
                if c is None:
                    errors.append(f"{key}: {e} missing")
                elif c[0] is None or c[0] < 0:
                    errors.append(f"{key}: {e} not counted ({c[2]!r})")
                elif c[1] < 100.0:
                    errors.append(f"{key}: {e} multiplexed ({c[1]}% enabled)")
    missing = sorted(roster - set(rows))
    if missing:
        errors.append(f"{len(missing)} of {len(roster)} runs missing, e.g. {missing[:3]}")
    if system_wide:
        # every controller in every run: each event summed over as many entries everywhere
        units = {
            (e, len(c) > 3 and c[3])
            for r in rows.values()
            for e, c in (r.get("counts") or {}).items()
        }
        per = {}
        for e, n in units:
            per.setdefault(e, set()).add(n)
        for e, ns in sorted(per.items()):
            if len(ns) != 1 or not all(isinstance(n, int) and n >= 1 for n in ns):
                errors.append(
                    f"{e}: the runs count it on differing controllers ({sorted(ns, key=str)}): unscorable"
                )
    if len(outputs) > 1:
        errors.append(f"the arms' outputs differ: {sorted(outputs, key=str)[:3]}")
    if errors:
        print(
            f"VOID: the counters do not make the registered run ({len(errors)} problems); nothing reported"
        )
        for e in errors[:25]:
            print(f"  {e}")
        return 1

    if system_wide:
        return report_dram(a, rows, threads, arms)

    def per_byte(arm, t, r, e):
        g = next(i for i, ev in enumerate(GROUPS) if e in ev)
        rec = rows[(arm, t, r, g)]
        return rec["counts"][e][0] / rec["out"]

    events = []
    for ev in GROUPS:
        events += [e for e in ev if e not in events]
    for t in threads:
        print(
            f"\n{t} thread{'s' if t > 1 else ''}: the median over {a.reps} reps, a byte of output; "
            f"{a.cand} over {a.ref} as the median of the reps' paired ratios"
        )
        print(f"  {'event':32s} {a.ref:>22s} {a.cand:>22s}   ratio")
        for e in events:
            ref = [per_byte(a.ref, t, r, e) for r in range(a.reps)]
            cand = [per_byte(a.cand, t, r, e) for r in range(a.reps)]
            ratio = (
                statistics.median(c / x for c, x in zip(cand, ref) if x > 0)
                if any(ref)
                else float("nan")
            )
            print(
                f"  {e:32s} {statistics.median(ref):22.6f} {statistics.median(cand):22.6f}   {ratio:.4f}"
            )
        for arm in arms:

            def med(e):
                return statistics.median(per_byte(arm, t, r, e) for r in range(a.reps))

            cyc, ins = med("cycles"), med("instructions")
            if a.events == "l2":
                refs = med("l2_rqsts.references")
                print(
                    f"  {arm}: IPC {ins / cyc:.3f}; L2 misses {med('l2_rqsts.miss'):.5f} a byte, "
                    f"{med('l2_rqsts.miss') / refs:.1%} of L2 references"
                )
                continue
            stall = med("cycle_activity.stalls_total")
            print(
                f"  {arm}: IPC {ins / cyc:.3f}; branch misses {1000 * med('branch-misses') / ins:.2f} a thousand "
                f"instructions; store-forward blocks {1000 * med('ld_blocks.store_forward') / ins:.3f}; stalls "
                f"{stall / cyc:.1%} of cycles (memory {med('cycle_activity.stalls_mem_any') / cyc:.1%}, "
                f"L3 miss {med('cycle_activity.stalls_l3_miss') / cyc:.1%})"
            )
    return 0


def report_dram(a, rows, threads, arms):
    """The memory controller's report: each arm's DRAM bytes a byte of output, read and written,
    the machine's idle rate (the median over the reps) times each run's elapsed time taken off"""
    reads, writes = GROUPS[0][:2]
    power = "power/energy-pkg/" if "power/energy-pkg/" in GROUPS[0] else None
    idle = {
        e: statistics.median(
            rows[("idle", 0, r, 0)]["counts"][e][0] / rows[("idle", 0, r, 0)]["elapsed"]
            for r in range(a.reps)
        )
        for e in (reads, writes)
    }
    print(
        f"the machine idle: {idle[reads] / 1e9:.3f} GB/s read, {idle[writes] / 1e9:.3f} GB/s written "
        f"(the median over {a.reps} reps), taken off each run"
    )
    idle_power = (
        statistics.median(
            rows[("idle", 0, r, 0)]["counts"][power][0] / rows[("idle", 0, r, 0)]["elapsed"]
            for r in range(a.reps)
        )
        if power
        else None
    )

    def dram(arm, t, r, e):
        rec = rows[(arm, t, r, 0)]
        return max(0.0, rec["counts"][e][0] - idle[e] * rec["elapsed"]) / rec["out"]

    for t in threads:
        print(
            f"\n{t} thread{'s' if t > 1 else ''}: DRAM bytes a byte of output, the median over {a.reps} reps; "
            f"{a.cand} over {a.ref} as the median of the reps' paired ratios"
        )
        print(f"  {'':10s} {a.ref:>24s} {a.cand:>24s}   ratio")
        for what, f in (
            ("read", lambda arm, r: dram(arm, t, r, reads)),
            ("written", lambda arm, r: dram(arm, t, r, writes)),
            ("total", lambda arm, r: dram(arm, t, r, reads) + dram(arm, t, r, writes)),
        ):
            ref = [f(a.ref, r) for r in range(a.reps)]
            cand = [f(a.cand, r) for r in range(a.reps)]
            ratio = (
                statistics.median(c / x for c, x in zip(cand, ref) if x > 0)
                if any(ref)
                else float("nan")
            )
            print(
                f"  {what:10s} {statistics.median(ref):24.3f} {statistics.median(cand):24.3f}   {ratio:.4f}"
            )
        if power:
            # the package's energy: watts over each run's elapsed time, and joules a GB of
            # output (the machine's whole package, idle included)
            def joules(arm, r):
                return rows[(arm, t, r, 0)]["counts"][power][0] / (
                    rows[(arm, t, r, 0)]["out"] / 1e9
                )

            ref = [joules(a.ref, r) for r in range(a.reps)]
            cand = [joules(a.cand, r) for r in range(a.reps)]
            ratio = statistics.median(c / x for c, x in zip(cand, ref) if x > 0)
            print(
                f"  {'J a GB':10s} {statistics.median(ref):24.3f} {statistics.median(cand):24.3f}   {ratio:.4f}"
            )
        for arm in arms:
            rate = statistics.median(
                (
                    rows[(arm, t, r, 0)]["counts"][reads][0]
                    + rows[(arm, t, r, 0)]["counts"][writes][0]
                )
                / rows[(arm, t, r, 0)]["elapsed"]
                for r in range(a.reps)
            )
            watts = (
                f", package {statistics.median(rows[(arm, t, r, 0)]['counts'][power][0] / rows[(arm, t, r, 0)]['elapsed'] for r in range(a.reps)):.1f} W"
                if power
                else ""
            )
            print(
                f"  {arm}: {rate / 1e9:.2f} GB/s at the memory controller (idle included){watts}, "
                f"wall {statistics.median(rows[(arm, t, r, 0)]['wall'] for r in range(a.reps)):.3f} s"
            )
    if power:
        print(
            f"the package idle: {idle_power:.1f} W (the median over {a.reps} reps; not taken off)"
        )
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("--ab", required=True)
    r.add_argument("--arm", action="append", required=True)
    r.add_argument("--threads", required=True)
    r.add_argument("--reps", type=int, default=10)
    r.add_argument("--file", required=True)
    r.add_argument("--out", required=True)
    r.add_argument("--events", choices=sorted(EVENT_SETS), default="default")
    r.add_argument("--unpinned", action="store_true", help="no run pinned")
    p = sub.add_parser("report")
    p.add_argument("runs")
    p.add_argument("--ref", required=True)
    p.add_argument("--cand", required=True)
    p.add_argument("--threads", required=True)
    p.add_argument("--reps", type=int, default=10)
    p.add_argument("--events", choices=sorted(EVENT_SETS), default="default")
    a = ap.parse_args(argv[1:])
    return run(a) if a.cmd == "run" else report(a)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
