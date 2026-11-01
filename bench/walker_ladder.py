#!/usr/bin/env python3
"""The minimal real-T-vector walker diagnostic's driver
(docs/journal/2026-11-01-walker-ladder-protocol.md; examples/walker_ladder.rs).

    python3 bench/walker_ladder.py audit FILE.bz2          the counter availability audit
    python3 bench/walker_ladder.py run FILE.bz2 [options]  the capture (not authorized yet)

audit, on the i9, before any capture: which of the protocol's events this CPU and kernel
offer (`perf list`), which count on a real ladder run, and whether each event group
counts without multiplexing. The ladder runs only to give the events something to count:
its counts and times are not kept. It writes walker-audit-<UTC time>.json and .md.

run: every rung (A, B, C, D, P) at 1, 8, 16 and 32 lanes, and S, each pinned to one CPU,
in an order shuffled by round; each config once a group of events (perf stat -x,) at REPS
walks and once at 0 (the setup alone), the difference divided by REPS x rows: counts a row.
Then the four-core check: A32, B32 and P32 as four copies at once, one a physical core.
--no-perf runs the same plan timed only (the VM's dry run: no counters there).
"""

import argparse
import datetime
import json
import os
import random
import shutil
import statistics
import subprocess
import sys

BIN = "target/release/examples/walker_ladder"
RUNGS = ["A", "B", "C", "D", "P"]
LANES = [1, 8, 16, 32]

# the events, in groups of at most four programmable counters (the i9 with SMT on), each
# with the fixed cycles and instructions; the audit confirms each group counts unmultiplexed
GROUPS = {
    "exec": ["uops_issued.any", "uops_retired.retire_slots", "br_misp_retired.all_branches",
             "idq_uops_not_delivered.core"],
    "mlp": ["l1d_pend_miss.pending", "l1d_pend_miss.pending_cycles", "l1d_pend_miss.fb_full",
            "cycle_activity.stalls_l1d_miss"],
    "stalls": ["cycle_activity.stalls_l2_miss", "cycle_activity.stalls_l3_miss",
               "cycle_activity.stalls_total", "resource_stalls.sb"],
    "pf_tlb": ["dtlb_load_misses.walk_active", "sw_prefetch_access.t0", "load_hit_pre.sw_pf",
               "mem_load_retired.l1_miss"],
    "loads": ["mem_load_retired.fb_hit", "mem_load_retired.l2_miss", "mem_load_retired.l3_miss",
              "resource_stalls.any"],
}
FIXED = ["cycles", "instructions"]
# how perf is invoked: WALKER_PERF="sudo perf" where counting needs root (the i9, as P87)
PERF = os.environ.get("WALKER_PERF", "perf").split()


def reps_for(lanes, reps):
    """walks a run: a quarter at one lane, whose walk is about ten times slower"""
    return max(4, reps // 4) if lanes == 1 else reps


def stamp():
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def physical_cpus():
    """one CPU of each physical core this process may use"""
    seen, cpus = set(), []
    for c in sorted(os.sched_getaffinity(0)):
        p = f"/sys/devices/system/cpu/cpu{c}/topology/thread_siblings_list"
        sib = open(p).read().strip() if os.path.exists(p) else str(c)
        first = int(sib.replace("-", ",").split(",")[0])
        if first not in seen:
            seen.add(first)
            cpus.append(first)
    return cpus


def environment():
    """numbers only"""
    def read(p):
        try:
            return int(open(p).read().strip())
        except (OSError, ValueError):
            return None
    fam = model = stepping = None
    for line in open("/proc/cpuinfo"):
        k, _, v = line.partition(":")
        k = k.strip()
        if k == "cpu family" and fam is None:
            fam = int(v)
        elif k == "model" and model is None:
            model = int(v)
        elif k == "stepping" and stepping is None:
            stepping = int(v)
    logical = len(os.sched_getaffinity(0))
    phys = len(physical_cpus())
    return {"cpu_family": fam, "cpu_model": model, "cpu_stepping": stepping,
            "logical_cpus": logical, "physical_cores": phys,
            "threads_per_core": logical // max(phys, 1),
            "perf_event_paranoid": read("/proc/sys/kernel/perf_event_paranoid"),
            "nmi_watchdog": read("/proc/sys/kernel/nmi_watchdog")}


def ladder(file, rung, lanes, reps, cpu=None, events=None):
    """one ladder run (pinned), under perf stat when events are given: (its JSON, counts)"""
    cmd = [BIN, file, rung, str(lanes), str(reps)]
    if cpu is not None:
        cmd = ["taskset", "-c", str(cpu)] + cmd
    counts = {}
    if events:
        cmd = PERF + ["stat", "-x", ",", "-e", ",".join(events), "--"] + cmd
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        raise SystemExit(f"walker_ladder: {' '.join(cmd)} failed:\n{p.stderr[-2000:]}")
    out = json.loads(p.stdout.strip().splitlines()[-1])
    if events:
        for line in p.stderr.splitlines():
            f = line.split(",")
            if len(f) < 3:
                continue
            # unprivileged perf prints "cycles:u"; keep the event's own name
            value, event = f[0], f[2].split(":")[0]
            running = float(f[4]) if len(f) > 4 and f[4] not in ("", "<not supported>") else None
            counts[event] = {"value": None if value.startswith("<") else float(value),
                             "status": value if value.startswith("<") else "counted",
                             "running_pct": running}
    return out, counts


def audit(args):
    env = environment()
    if not shutil.which("perf"):
        raise SystemExit("walker_ladder: perf is not on PATH")
    listed = subprocess.run(PERF + ["list", "--no-desc"], capture_output=True, text=True).stdout
    listed_l = listed.lower()
    wanted = [e for g in GROUPS.values() for e in g]
    rows = []
    cpu = physical_cpus()[0]
    for e in FIXED + wanted:
        _, c = ladder(args.file, "A", 32, 3, cpu, [e])
        got = c.get(e, {"status": "absent from perf's output", "value": None, "running_pct": None})
        rows.append({"event": e, "listed": e.lower() in listed_l, "status": got["status"],
                     "counts": got["value"] is not None and got["value"] > 0})
    groups = []
    for name, evs in GROUPS.items():
        _, c = ladder(args.file, "A", 32, 3, cpu, FIXED + evs)
        pcts = [c.get(e, {}).get("running_pct") for e in FIXED + evs]
        groups.append({"group": name, "events": FIXED + evs,
                       "all_counted": all(c.get(e, {}).get("status") == "counted" for e in FIXED + evs),
                       "multiplexed": any(p is not None and p < 99.9 for p in pcts),
                       "running_pct": pcts})
    t = stamp()
    res = {"time": t, "environment": env, "events": rows, "groups": groups,
           "note": "availability only: the ladder runs' counts and times are not kept"}
    json.dump(res, open(f"walker-audit-{t}.json", "w"), indent=1)
    md = [f"# Walker counter audit ({t})", "", "Environment (numbers only): " +
          ", ".join(f"{k} {v}" for k, v in env.items()), "",
          "| event | in perf list | status | counts on a run |", "|---|---|---|---|"]
    md += [f"| `{r['event']}` | {r['listed']} | {r['status']} | {r['counts']} |" for r in rows]
    md += ["", "| group | all counted | multiplexed |", "|---|---|---|"]
    md += [f"| {g['group']} | {g['all_counted']} | {g['multiplexed']} |" for g in groups]
    open(f"walker-audit-{t}.md", "w").write("\n".join(md) + "\n")
    print("\n".join(md))
    print(f"\nwrote walker-audit-{t}.json and walker-audit-{t}.md")


def run(args):
    cpus = physical_cpus()
    cpu = cpus[0]
    t = stamp()
    jl = open(f"walker-ladder-{t}.jsonl", "w")
    configs = [(r, k) for r in RUNGS for k in LANES] + [("S", 32)]
    groups = [None] if args.no_perf else list(GROUPS)
    rng = random.Random(args.seed)
    for rnd in range(args.rounds):
        order = [(r, k, g) for (r, k) in configs for g in groups]
        rng.shuffle(order)
        for rung, lanes, g in order:
            evs = None if g is None else FIXED + GROUPS[g]
            reps = reps_for(lanes, args.reps)
            out, c = ladder(args.file, rung, lanes, reps, cpu, evs)
            base = {}
            if evs:
                _, base = ladder(args.file, rung, lanes, 0, cpu, evs)
            per_row = {}
            for e in (evs or []):
                v, b = c.get(e, {}).get("value"), base.get(e, {}).get("value")
                per_row[e] = None if v is None or b is None else (v - b) / (reps * out["rows"])
            rec = {"round": rnd, "rung": rung, "lanes": lanes, "group": g, "cpu": cpu,
                   "ns_a_row": out["ns_a_row"], "rows": out["rows"], "reps": reps,
                   "checked": out["checked"], "per_row": per_row,
                   "multiplexed": any((c.get(e, {}).get("running_pct") or 100) < 99.9
                                      for e in (evs or []))}
            jl.write(json.dumps(rec) + "\n")
            jl.flush()
            print(f"round {rnd} {rung}{lanes} {g or 'time'}: {out['ns_a_row']:.3f} ns a row")
    # four copies at once, one a physical core
    if len(cpus) >= 4:
        for rnd in range(args.rounds):
            for rung in ["A", "B", "P"]:
                for g in ([None] if args.no_perf else ["exec", "mlp"]):
                    evs = None if g is None else FIXED + GROUPS[g]
                    procs = []
                    for cc in cpus[:4]:
                        cmd = ["taskset", "-c", str(cc), BIN, args.file, rung, "32", str(args.reps)]
                        if evs:
                            cmd = PERF + ["stat", "-x", ",", "-e", ",".join(evs), "--"] + cmd
                        procs.append((cc, subprocess.Popen(cmd, stdout=subprocess.PIPE,
                                                           stderr=subprocess.PIPE, text=True)))
                    for cc, p in procs:
                        so, se = p.communicate()
                        if p.returncode != 0:
                            raise SystemExit(f"four-core {rung}: {se[-1000:]}")
                        o = json.loads(so.strip().splitlines()[-1])
                        vals = {}
                        for line in se.splitlines():
                            f = line.split(",")
                            if len(f) >= 3 and not f[0].startswith("<"):
                                try:
                                    vals[f[2].split(":")[0]] = float(f[0])
                                except ValueError:
                                    pass
                        rec = {"round": rnd, "rung": rung, "lanes": 32, "group": g, "cpu": cc,
                               "four_core": True, "ns_a_row": o["ns_a_row"], "rows": o["rows"],
                               "reps": args.reps, "checked": o["checked"],
                               "totals_with_setup": vals}
                        jl.write(json.dumps(rec) + "\n")
                    print(f"round {rnd} four-core {rung}32 {g or 'time'}")
    jl.close()
    print(f"wrote walker-ladder-{t}.jsonl")
    summarise(f"walker-ladder-{t}.jsonl")


def summarise(path):
    """The protocol's figures and decisions (§4 and §5 of the protocol)"""
    recs = [json.loads(l) for l in open(path)]
    one = [r for r in recs if not r.get("four_core")]
    four = [r for r in recs if r.get("four_core")]

    def med(xs):
        xs = [x for x in xs if x is not None]
        return statistics.median(xs) if xs else None

    ns = {(r, k): med([x["ns_a_row"] for x in one if (x["rung"], x["lanes"]) == (r, k)])
          for r in RUNGS + ["S"] for k in LANES}
    cyc = {(r, k): med([x["per_row"].get("cycles") for x in one
                        if (x["rung"], x["lanes"]) == (r, k) and x["per_row"]])
           for r in RUNGS for k in LANES}
    # G1: the shipped walk's time reproduces across the runs (max / min of its ns a row)
    p32 = [x["ns_a_row"] for x in one if (x["rung"], x["lanes"]) == ("P", 32)]
    spread = max(p32) / min(p32)
    g1 = spread <= 1.05
    print(f"G1: P32's ns a row, max / min over its {len(p32)} runs: {spread:.3f} "
          + ("(holds)" if g1 else "(fails: the run is void)"))
    if not all(x["checked"] for x in recs) or any(x.get("multiplexed") for x in one):
        print("G0 fails: a run unchecked or multiplexed: the run is void")
        g1 = False
    metric = cyc if any(v is not None for v in cyc.values()) else ns
    unit = "cycles a row" if metric is cyc else "ns a row (no counters: timing only)"
    print(f"\n{unit}, medians over rounds (one core)")
    print("| rung | " + " | ".join(f"{k} lanes" for k in LANES) + " |")
    print("|---|" + "--:|" * len(LANES))
    for r in RUNGS:
        print(f"| {r} | " + " | ".join("" if metric[(r, k)] is None else f"{metric[(r, k)]:.3f}"
                                       for k in LANES) + " |")
    bare = min(v for (r, k), v in metric.items() if r in ("A", "B") and v is not None)
    p32m = metric[("P", 32)]
    ratio = p32m / bare
    out = {"unit": unit, "bare": bare, "P32": p32m, "P32_over_bare": ratio, "G1_spread": spread,
           "valid": g1}
    if ratio <= 1.15:
        out["D1_D2"] = "D1: the shipped walk is within 15% of the bare chase: no walker rewrite"
    elif ratio >= 1.5:
        steps = {"B->C": metric[("C", 32)] - metric[("B", 32)],
                 "C->D": metric[("D", 32)] - metric[("C", 32)],
                 "D->P": p32m - metric[("D", 32)]}
        out["D1_D2"] = ("D2: the shipped walk is at least 1.5x the bare chase; the largest step: "
                        + max(steps, key=steps.get))
        out["steps"] = steps
    else:
        out["D1_D2"] = "between: report only, no rewrite on this evidence"
    if metric is cyc:
        uops = med([x["per_row"].get("uops_issued.any") for x in one
                    if (x["rung"], x["lanes"]) == ("P", 32) and x["per_row"]])
        if uops is not None:
            share = (uops / 4) / p32m
            out["D3"] = f"issue-width share {share:.2f}: " + (
                "execution-bound (>= 0.70)" if share >= 0.70 else "not execution-bound")

        def mlp(k):
            p = med([x["per_row"].get("l1d_pend_miss.pending") for x in one
                     if (x["rung"], x["lanes"]) == ("B", k) and x["per_row"]])
            c = med([x["per_row"].get("l1d_pend_miss.pending_cycles") for x in one
                     if (x["rung"], x["lanes"]) == ("B", k) and x["per_row"]])
            return None if not p or not c else p / c

        def fb(k):
            f = med([x["per_row"].get("l1d_pend_miss.fb_full") for x in one
                     if (x["rung"], x["lanes"]) == ("B", k) and x["per_row"]])
            c = cyc[("B", k)]
            return None if f is None or not c else f / c
        m16, m32, f16, f32 = mlp(16), mlp(32), fb(16), fb(32)
        if None not in (m16, m32, f16, f32):
            plateau = m32 / m16 < 1.10 and f32 >= 2 * max(f16, 1e-9)
            out["D4"] = (f"demand MLP B16 {m16:.2f}, B32 {m32:.2f}; fb_full share {f16:.3f} to "
                         f"{f32:.3f}: " + ("miss capacity bound" if plateau else "no plateau"))
    if four:
        def q(rung):
            return med([x["ns_a_row"] for x in four if x["rung"] == rung])
        n4 = ns[("P", 32)] and ns[("B", 32)] and (q("P") / q("B")) / (ns[("P", 32)] / ns[("B", 32)])
        out["D5"] = (f"four-core P/B over one-core P/B: {n4:.3f}: "
                     + ("transfers (within 15 points)" if abs(n4 - 1) <= 0.15
                        else "does not transfer: report and stop"))
    if not g1:
        out["read"] = "VOID: G0 or G1 failed, so none of the decisions above is read"
    print("\n" + json.dumps(out, indent=1))
    json.dump(out, open(path.replace(".jsonl", "-summary.json"), "w"), indent=1)


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("audit")
    a.add_argument("file")
    r = sub.add_parser("run")
    r.add_argument("file")
    r.add_argument("--rounds", type=int, default=3)
    r.add_argument("--reps", type=int, default=100)
    r.add_argument("--seed", type=int, default=20261101)
    r.add_argument("--no-perf", action="store_true")
    s = sub.add_parser("summarise")
    s.add_argument("jsonl")
    args = ap.parse_args()
    if not os.path.exists(BIN):
        sys.exit(f"walker_ladder: build it first (cargo build --release --example walker_ladder)")
    {"audit": audit, "run": run, "summarise": lambda a: summarise(a.jsonl)}[args.cmd](args)


if __name__ == "__main__":
    main()
