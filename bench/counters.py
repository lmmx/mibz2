#!/usr/bin/env python3
"""Hardware counters for two decoder arms, single-threaded and at T threads: where the time
goes, not how much (`just bench-stop-walk-counters`; #145, the Coordinator's approval of the
Pathfinder's request in REVIEW 6924288835). A diagnostic: it records and decides nothing.
Standard library only.

    counters.py run --ab AB --arm A --arm B --threads 1,10 --reps 10 --file F --out C.jsonl [--events l2]
    counters.py report C.jsonl --ref A --cand B [--cand C ...] --threads 1,10 --reps 10 [--events l2]
                       [--workers ARM=N ...]
    (either: --groups 'cycles,instructions,l2_rqsts.miss;cycles,LLC-load-misses' in place of --events)

**Arms.** `NAME` runs `AB child NAME`; `NAME@EXE` runs another build's harness, `EXE child NAME`
(as `ab run`'s arms), its records keeping the arm as given. `NAME+thp` runs `NAME` with glibc's
malloc asking for transparent huge pages (`GLIBC_TUNABLES=glibc.malloc.hugetlb=1`: glibc 2.35 or
later madvises its heaps and mappings `MADV_HUGEPAGE`), no code changed (`just bench-miss-cost`'s
side arm). Every run records `thp_faults`, the rise of `/proc/vmstat`'s `thp_fault_alloc` over it
(system-wide, so context only), and the report prints its median for each arm: whether the
tunable took effect is read there, not assumed.

**Events.** The default: the three groups below. `--events l2`: one group, cycles, instructions,
L2 demand-request misses and references (`l2_rqsts.miss`, `l2_rqsts.references`), for W12's L2
diagnostic (#145, the Coordinator's ruling on #506). `--groups`: the groups named, `;` between
groups and `,` between a group's events, in place of a set (`just bench-workers-low`'s occupancy
diagnostic, its events those of `perf list` on the machine that runs it); the run and the report
must name the same groups. A group written `system:EV,EV` is counted system-wide (`perf stat -a`,
the uncore's memory-controller events, which perf will not count for one process; the workstation's
`kernel.perf_event_paranoid` at 0 or less); a count perf gives in MiB is read in bytes.

**The miss cost** (`just bench-miss-cost`): where the groups name them, the report adds a table
of derived figures for each arm: the stall shares of cycles (`cycle_activity.*` over cycles), the
L3 hit share of retired loads that missed L2, the page walks' share of cycles and cycles a walk,
the average latency of offcore demand data reads and of those that missed L3 (Little's law: the
outstanding count's sum over cycles over the requests), the reads in flight while any is, and
bandwidth in GB/s against the child's wall (64 bytes a request or retired miss; the IMC's own
bytes where counted). The counts cover the whole process and the wall the decode alone, so each
GB/s is an upper bound on the decode's.

**The clock and the busy share** (`just bench-workers-at-cores`): with named groups, where
`ref-cycles` is counted beside `cycles`, the report gives cycles over ref-cycles (the core clock
over the reference clock, the turbo and frequency residual). Where the software event
`task-clock` is counted (msec, kept as perf prints it), it gives a busy share a worker: the
median over the reps of task-clock over (the child's wall × W). W is read from an occupancy
arm's name (`mibz2-occ-N-S`: N) or given (`--workers ARM=N`, production's `mibz2` at its own
plan); an arm of unknown W has no share printed. task-clock counts every thread of the process
(the reader and the scanner beside the workers, and the start-up outside the child's wall), so
the share is an upper bound on the workers' own.

**Several candidates** (`--cand` repeated, `bench-workers-low`): each read against the one
reference, a column and a ratio each; the run's arms are the reference and every candidate.

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


THP_ENV = {"GLIBC_TUNABLES": "glibc.malloc.hugetlb=1"}
SYSTEM = set()


def harness(ab, arm):
    """The harness, the arm's name for `ab child` and the child's extra environment: `NAME@EXE`
    runs EXE, `NAME+thp` asks glibc's malloc for transparent huge pages"""
    name, at, exe = arm.partition("@")
    exe = exe if at else ab
    if name.endswith("+thp"):
        return exe, name.removesuffix("+thp"), THP_ENV
    return exe, name, {}


def thp_faults():
    """`/proc/vmstat`'s thp_fault_alloc, else None"""
    try:
        for line in open("/proc/vmstat"):
            k, _, v = line.partition(" ")
            if k == "thp_fault_alloc":
                return int(v)
    except (OSError, ValueError):
        pass
    return None


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
    """perf stat -x, output: {event: (value or None, enabled percent, raw, unit)}"""
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
                v = float(value)
            except ValueError:
                v = None
        got[event] = (v, pct, value, f[1])
    return got


def groups(a):
    """The event groups `--groups` names, else `--events`' set; SYSTEM, those counted system-wide"""
    SYSTEM.clear()
    if a.groups:
        out = []
        for g in a.groups.split(";"):
            g = g.strip()
            system = g.startswith("system:")
            g = g.removeprefix("system:")
            if not g.strip(","):
                continue
            if system:
                SYSTEM.add(len(out))
            out.append([e for e in g.split(",") if e])
        return out
    return EVENT_SETS[a.events]


def occ_workers(arm):
    """An occupancy arm's workers (`mibz2-occ-N-S`: N), else None"""
    n, _, s = arm.removeprefix("mibz2-occ-").partition("-") if arm.startswith("mibz2-occ-") else ("", "", "")
    return int(n) if n.isdigit() and s.isdigit() and int(n) > 0 else None


def run(a):
    global GROUPS
    GROUPS = groups(a)
    threads = [int(t) for t in a.threads.split(",")]
    with open(a.out, "w") as out:
        for rep in range(a.reps):
            arms = a.arm if rep % 2 == 0 else list(reversed(a.arm))
            for t in threads:
                for arm in arms:
                    for g, events in enumerate(GROUPS):
                        rec = {"arm": arm, "threads": t, "rep": rep, "group": g, "events": events}
                        if g in SYSTEM:
                            rec["system"] = True
                        with tempfile.NamedTemporaryFile("r", suffix=".csv") as tf:
                            cmd = ["perf", "stat", "-x,", "-o", tf.name, *(["-a"] if g in SYSTEM else []),
                                   "-e", ",".join(events), "--"]
                            cpus = spread(t)
                            if cpus:
                                cmd += ["taskset", "-c", cpus]
                            exe, name, env = harness(a.ab, arm)
                            cmd += [exe, "child", name, str(t), a.file]
                            try:
                                before = thp_faults()
                                p = subprocess.run(cmd, capture_output=True, text=True,
                                                   env={**os.environ, **env} if env else None)
                                after = thp_faults()
                                rec["thp_faults"] = after - before if None not in (before, after) else None
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
    GROUPS = groups(a)
    threads = [int(t) for t in a.threads.split(",")]
    cands = a.cand
    arms = [a.ref, *cands]
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
                continue
            if "events" in r and r["events"] != GROUPS[key[3]]:
                errors.append(f"{key}: counted {r['events']}, not the group named {GROUPS[key[3]]}")
                continue
            if bool(r.get("system")) != (key[3] in SYSTEM):
                errors.append(f"{key}: counted {'system-wide' if r.get('system') else 'for the process'}, "
                              f"not as the group is named")
                continue
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
    workers = {}
    for spec in a.workers:
        arm, _, n = spec.rpartition("=")
        if not arm or not n.isdigit() or int(n) < 1:
            errors.append(f"--workers {spec!r}: not ARM=N")
        else:
            workers[arm] = int(n)
    walled = {"task-clock"} | {num for _, kind, num, _ in DERIVED if kind == "bw"}
    for k, r in rows.items():
        if walled & set(GROUPS[k[3]]) and not (isinstance(r.get("wall"), (int, float)) and r["wall"] > 0):
            errors.append(f"{k}: no wall for the busy share or a bandwidth")
    if errors:
        print(f"VOID: the counters do not make the registered run ({len(errors)} problems); nothing reported")
        for e in errors[:25]:
            print(f"  {e}")
        return 1

    def record(arm, t, r, e):
        return rows[(arm, t, r, next(i for i, ev in enumerate(GROUPS) if e in ev))]

    def count(rec, e):
        c = rec["counts"][e]
        return c[0] * 2 ** 20 if len(c) > 3 and c[3] == "MiB" else c[0]

    def per_byte(arm, t, r, e):
        rec = record(arm, t, r, e)
        return count(rec, e) / rec["out"]

    events = []
    for ev in GROUPS:
        events += [e for e in ev if e not in events]
    for t in threads:
        print(f"\n{t} thread{'s' if t > 1 else ''}: the median over {a.reps} reps, a byte of output; "
              f"{' and '.join(cands)} over {a.ref} as the median of the reps' paired ratios")
        print(f"  {'event':32s} {a.ref:>22s} " + " ".join(f"{c:>22s}" for c in cands) + "   "
              + ("ratio" if len(cands) == 1 else " ".join(f"{c.removeprefix('mibz2-occ-')[-12:]:>12s}" for c in cands)))
        for e in events:
            ref = [per_byte(a.ref, t, r, e) for r in range(a.reps)]
            line, ratios = f"  {e:32s} {statistics.median(ref):22.6f} ", []
            for c in cands:
                cand = [per_byte(c, t, r, e) for r in range(a.reps)]
                ratios.append(statistics.median(x / y for x, y in zip(cand, ref) if y > 0) if any(ref) else float("nan"))
                line += f"{statistics.median(cand):22.6f} "
            print(line + "  " + " ".join(f"{x:12.4f}" if len(cands) > 1 else f"{x:.4f}" for x in ratios))
        for arm in arms:
            def med(e):
                return statistics.median(per_byte(arm, t, r, e) for r in range(a.reps))
            if a.groups:
                parts = []
                if "cycles" in events and "instructions" in events:
                    parts.append(f"IPC {med('instructions') / med('cycles'):.3f}")
                if "cycles" in events and "ref-cycles" in events:
                    parts.append(f"cycles over ref-cycles {med('cycles') / med('ref-cycles'):.3f}")
                w = workers.get(arm) or occ_workers(arm)
                if "task-clock" in events and w:
                    g = next(i for i, ev in enumerate(GROUPS) if "task-clock" in ev)
                    share = statistics.median(rows[(arm, t, r, g)]["counts"]["task-clock"][0] / 1000
                                              / (rows[(arm, t, r, g)]["wall"] * w) for r in range(a.reps))
                    parts.append(f"busy share {share:.3f} a worker (task-clock over wall x {w})")
                if parts:
                    print(f"  {arm}: {'; '.join(parts)}")
                continue
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
        if a.groups:
            derived(arms, t, events, a.reps, rows, record, count, per_byte)
    return 0


# the miss cost's derived figures (`just bench-miss-cost`): (label, kind, numerator, denominator);
# "ratio" a ratio of counts, "bw" GB/s at a byte a count (with the numerator's bytes a count)
OFFCORE = "offcore_requests_outstanding."
DERIVED = [
    ("IPC", "ratio", "instructions", "cycles"),
    ("stalls, share of cycles", "ratio", "cycle_activity.stalls_total", "cycles"),
    ("L2-miss stalls, share of cycles", "ratio", "cycle_activity.stalls_l2_miss", "cycles"),
    ("L3-miss stalls, share of cycles", "ratio", "cycle_activity.stalls_l3_miss", "cycles"),
    ("L3 hits, share of L2-miss loads", "hit", "mem_load_retired.l3_hit", "mem_load_retired.l3_miss"),
    ("page walks, share of cycles", "ratio", "dtlb_load_misses.walk_active", "cycles"),
    ("cycles a page walk", "ratio", "dtlb_load_misses.walk_pending", "dtlb_load_misses.walk_completed"),
    ("demand-read latency, cycles", "ratio", OFFCORE + "demand_data_rd", "offcore_requests.demand_data_rd"),
    ("L3-miss demand-read latency, cycles", "ratio", OFFCORE + "l3_miss_demand_data_rd",
     "offcore_requests.l3_miss_demand_data_rd"),
    ("L3-miss demand reads in flight, when any", "ratio", OFFCORE + "l3_miss_demand_data_rd",
     OFFCORE + "cycles_with_l3_miss_demand_data_rd"),
    ("data reads in flight, when any", "ratio", OFFCORE + "all_data_rd", OFFCORE + "cycles_with_data_rd"),
    ("cycles with a data read out, share", "ratio", OFFCORE + "cycles_with_data_rd", "cycles"),
    ("offcore data reads, GB/s", "bw", "offcore_requests.all_data_rd", 64),
    ("L3-miss demand reads, GB/s", "bw", "offcore_requests.l3_miss_demand_data_rd", 64),
    ("L3-miss retired loads, GB/s", "bw", "mem_load_retired.l3_miss", 64),
    ("DRAM reads (IMC), GB/s", "bw", "uncore_imc/data_reads/", 1),
    ("DRAM writes (IMC), GB/s", "bw", "uncore_imc/data_writes/", 1),
    ("DRAM reads (IMC CAS), GB/s", "bw", "unc_m_cas_count.rd", 64),
    ("DRAM writes (IMC CAS), GB/s", "bw", "unc_m_cas_count.wr", 64),
]


def derived(arms, t, events, reps, rows, record, count, per_byte):
    """The miss cost's table: each derived figure the groups allow, for each arm, the median over
    the reps (a ratio within one group read rep by rep, across groups as the ratio of medians)"""
    def figure(arm, kind, num, den):
        if num not in events or (kind != "bw" and den not in events):
            return None
        if kind == "bw":
            return statistics.median(count(record(arm, t, r, num), num) * den / record(arm, t, r, num)["wall"] / 1e9
                                     for r in range(reps))
        if kind == "hit":
            h = statistics.median(per_byte(arm, t, r, num) for r in range(reps))
            m = statistics.median(per_byte(arm, t, r, den) for r in range(reps))
            return h / (h + m) if h + m > 0 else None
        if record(arm, t, 0, num) is record(arm, t, 0, den):
            vals = [count(record(arm, t, r, num), num) / count(record(arm, t, r, num), den)
                    for r in range(reps) if count(record(arm, t, r, num), den) > 0]
            return statistics.median(vals) if vals else None
        n = statistics.median(per_byte(arm, t, r, num) for r in range(reps))
        d = statistics.median(per_byte(arm, t, r, den) for r in range(reps))
        return n / d if d > 0 else None

    table = []
    for label, kind, num, den in DERIVED:
        vals = [figure(arm, kind, num, den) for arm in arms]
        if any(v is not None for v in vals):
            table.append((label, vals))
    thp = [[rows[k].get("thp_faults") for k in rows if k[0] == arm and k[1] == t] for arm in arms]
    if any("+thp" in arm for arm in arms) and all(v is not None for vs in thp for v in vs):
        table.append(("THP faults a run (system-wide)", [statistics.median(vs) for vs in thp]))
    if not table:
        return
    print(f"\n  the miss cost at {t} threads: the median over {reps} reps "
          "(latency in core cycles; GB/s against the child's wall, an upper bound)")
    print(f"  {'figure':42s} " + " ".join(f"{arm[-20:]:>20s}" for arm in arms))
    for label, vals in table:
        print(f"  {label:42s} " + " ".join(f"{'-':>20s}" if v is None else f"{v:20.4f}" for v in vals))


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
    r.add_argument("--groups", default="", help="event groups, ';' between groups, in place of --events")
    p = sub.add_parser("report")
    p.add_argument("runs")
    p.add_argument("--ref", required=True)
    p.add_argument("--cand", required=True, action="append")
    p.add_argument("--threads", required=True)
    p.add_argument("--reps", type=int, default=10)
    p.add_argument("--events", choices=sorted(EVENT_SETS), default="default")
    p.add_argument("--groups", default="", help="event groups, ';' between groups, in place of --events")
    p.add_argument("--workers", action="append", default=[], metavar="ARM=N",
                   help="an arm's workers, for the busy share (an occupancy arm's are read from its name)")
    a = ap.parse_args(argv[1:])
    return run(a) if a.cmd == "run" else report(a)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
