#!/usr/bin/env python3
"""Hardware counters for two decoder arms, single-threaded and at T threads: where the time
goes, not how much (`just bench-stop-walk-counters`; #145, the Coordinator's approval of the
Pathfinder's request in REVIEW 6924288835). A diagnostic: it records and decides nothing.
Standard library only.

    counters.py run --ab AB --arm A --arm B --threads 1,10 --reps 10 --file F --out C.jsonl [--events l2]
    counters.py report C.jsonl --ref A --cand B --threads 1,10 --reps 10 [--events l2]

**Arms.** `NAME` runs `AB child NAME`; `NAME@EXE` runs another build's harness, `EXE child NAME`
(as `ab run`'s arms), its records keeping the arm as given.

**Events.** The default: the three groups below. `--events l2`: one group, cycles, instructions,
L2 demand-request misses and references (`l2_rqsts.miss`, `l2_rqsts.references`), for W12's L2
diagnostic (#145, the Coordinator's ruling on #506).

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

EVENT_SETS = {
    "default": [
        ["cycles", "instructions", "branches", "branch-misses"],
        ["cycles", "L1-dcache-load-misses", "LLC-load-misses", "ld_blocks.store_forward"],
        ["cycles", "cycle_activity.stalls_total", "cycle_activity.stalls_mem_any", "cycle_activity.stalls_l3_miss"],
    ],
    "l2": [["cycles", "instructions", "l2_rqsts.miss", "l2_rqsts.references"]],
}
GROUPS = EVENT_SETS["default"]


def harness(ab, arm):
    """The harness and the arm's name for `ab child`: `NAME@EXE` runs EXE"""
    name, at, exe = arm.partition("@")
    return (exe, name) if at else (ab, arm)


def spread(t):
    """The first CPU of each of the first t cores, as `ab run --spread`; None past the cores"""
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


def parse_perf(path):
    """perf stat -x, output: {event: (value or None, enabled percent, raw)}"""
    got = {}
    for line in open(path):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        f = line.split(",")
        if len(f) < 5:
            continue
        value, event = f[0], f[2]
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
        got[event] = (v, pct, value)
    return got


def run(a):
    global GROUPS
    GROUPS = EVENT_SETS[a.events]
    threads = [int(t) for t in a.threads.split(",")]
    with open(a.out, "w") as out:
        for rep in range(a.reps):
            arms = a.arm if rep % 2 == 0 else list(reversed(a.arm))
            for t in threads:
                for arm in arms:
                    for g, events in enumerate(GROUPS):
                        rec = {"arm": arm, "threads": t, "rep": rep, "group": g, "events": events}
                        with tempfile.NamedTemporaryFile("r", suffix=".csv") as tf:
                            cmd = ["perf", "stat", "-x,", "-o", tf.name, "-e", ",".join(events), "--"]
                            cpus = spread(t)
                            if cpus:
                                cmd += ["taskset", "-c", cpus]
                            exe, name = harness(a.ab, arm)
                            cmd += [exe, "child", name, str(t), a.file]
                            try:
                                p = subprocess.run(cmd, capture_output=True, text=True)
                            except OSError as e:
                                rec["err"] = f"perf did not start: {e}"
                                out.write(json.dumps(rec) + "\n")
                                continue
                            rec["rc"] = p.returncode
                            rec["cpus"] = cpus or "unpinned"
                            if p.returncode != 0:
                                rec["err"] = (p.stderr.strip().splitlines() or ["failed"])[-1][:300]
                            else:
                                try:
                                    child = json.loads(p.stdout.strip().splitlines()[-1])
                                    rec.update(out=child.get("out"), hash=child.get("sha256") or child.get("hash"),
                                               wall=child.get("wall"), child_err=child.get("err"))
                                except (ValueError, IndexError):
                                    rec["err"] = "no child record"
                                rec["counts"] = {e: list(v) for e, v in parse_perf(tf.name).items()}
                        out.write(json.dumps(rec) + "\n")
                        print(f"  rep {rep} t={t} {arm} group {g}: {rec.get('err') or 'counted'}", flush=True)
    return 0


def report(a):
    global GROUPS
    GROUPS = EVENT_SETS[a.events]
    threads = [int(t) for t in a.threads.split(",")]
    arms = [a.ref, a.cand]
    roster = {(arm, t, r, g) for arm in arms for t in threads for r in range(a.reps) for g in range(len(GROUPS))}
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
                errors.append(f"{key}: the run failed: {r.get('err') or r.get('child_err') or r.get('rc')}")
                continue
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
    if len(outputs) > 1:
        errors.append(f"the arms' outputs differ: {sorted(outputs, key=str)[:3]}")
    if errors:
        print(f"VOID: the counters do not make the registered run ({len(errors)} problems); nothing reported")
        for e in errors[:25]:
            print(f"  {e}")
        return 1

    def per_byte(arm, t, r, e):
        g = next(i for i, ev in enumerate(GROUPS) if e in ev)
        rec = rows[(arm, t, r, g)]
        return rec["counts"][e][0] / rec["out"]

    events = []
    for ev in GROUPS:
        events += [e for e in ev if e not in events]
    for t in threads:
        print(f"\n{t} thread{'s' if t > 1 else ''}: the median over {a.reps} reps, a byte of output; "
              f"{a.cand} over {a.ref} as the median of the reps' paired ratios")
        print(f"  {'event':32s} {a.ref:>22s} {a.cand:>22s}   ratio")
        for e in events:
            ref = [per_byte(a.ref, t, r, e) for r in range(a.reps)]
            cand = [per_byte(a.cand, t, r, e) for r in range(a.reps)]
            ratio = statistics.median(c / x for c, x in zip(cand, ref) if x > 0) if any(ref) else float("nan")
            print(f"  {e:32s} {statistics.median(ref):22.6f} {statistics.median(cand):22.6f}   {ratio:.4f}")
        for arm in arms:
            def med(e):
                return statistics.median(per_byte(arm, t, r, e) for r in range(a.reps))
            cyc, ins = med("cycles"), med("instructions")
            if a.events == "l2":
                refs = med("l2_rqsts.references")
                print(f"  {arm}: IPC {ins / cyc:.3f}; L2 misses {med('l2_rqsts.miss'):.5f} a byte, "
                      f"{med('l2_rqsts.miss') / refs:.1%} of L2 references")
                continue
            stall = med("cycle_activity.stalls_total")
            print(f"  {arm}: IPC {ins / cyc:.3f}; branch misses {1000 * med('branch-misses') / ins:.2f} a thousand "
                  f"instructions; store-forward blocks {1000 * med('ld_blocks.store_forward') / ins:.3f}; stalls "
                  f"{stall / cyc:.1%} of cycles (memory {med('cycle_activity.stalls_mem_any') / cyc:.1%}, "
                  f"L3 miss {med('cycle_activity.stalls_l3_miss') / cyc:.1%})")
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
