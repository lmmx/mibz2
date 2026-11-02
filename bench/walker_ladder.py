#!/usr/bin/env python3
"""The minimal real-T-vector walker diagnostic's driver
(docs/journal/2026-11-01-walker-ladder-protocol.md; examples/walker_ladder.rs).

    python3 bench/walker_ladder.py probe FILE.bz2           the revised availability probe
    python3 bench/walker_ladder.py run FILE.bz2 [options]   the capture (not authorized yet)
    python3 bench/walker_ladder.py summarise RUN.jsonl      a capture's gates and decisions

**The measurement boundary.** perf starts with its counters disabled (`-D -1`); the ladder
enables them through perf's control FIFO (`--control fifo:CTL,ACK`) just before its timed
walks and disables them just after, so the counters and the ladder's clock cover the same
walks. Validation (every A-C walk's result, D's, P's and S's output hash) and setup are
outside both. A run at 0 walks counts the enable/disable pair alone: the control overhead.

**Groups.** Each group is passed as one perf event group, `{cycles,instructions,...}`, so its
members are scheduled together or not at all; a ratio is formed only from members of one
group, one run.

**The handoff.** The ladder's control FIFOs and its barrier are given on its own command
line after perf's `--` (`--ctl CTL --ack ACK`, `--barrier`), never through the environment,
which `sudo` filters (report 140); a run asked for control that cannot open it fails.

**Four cores.** Four copies, one a physical core, each with its own table, set up and walk
once, print "ready", and wait; the driver releases them together with a common deadline
(--four-ms after the release), and each walks until it, recording its walks and its
window's wall-clock bounds. So all four walk throughout each other's windows; the windows
must overlap (G0).

**Validity (G0), for every record that enters the analysis** (one-core, four-core and
control-overhead runs alike): checked; pinned; every event of its group counted (a zero
count is a count); every event running 99.9% of its enabled time or more (not
multiplexed); and the four-core windows' overlap at least 0.95 of each window.
"""

import argparse
import datetime
import json
import math
import os
import random
import shutil
import statistics
import subprocess
import sys
import tempfile

BIN = "target/release/examples/walker_ladder"
RUNGS = ["A", "B", "C", "D", "M", "P"]
LANES = [1, 8, 16, 32]
FIXED = ["cycles", "instructions"]

# The capture's groups (each with cycles and instructions, as one perf group), frozen
# after the revised probe. Three events at most: with the i9's NMI watchdog holding the
# fixed cycles counter, cycles takes one of the four programmable counters (SMT on).
GROUPS = {
    "exec": ["uops_issued.any", "uops_retired.retire_slots", "idq_uops_not_delivered.core"],
    "branch_sb": ["br_misp_retired.all_branches", "resource_stalls.sb", "resource_stalls.any"],
    "mlp": ["l1d_pend_miss.pending", "l1d_pend_miss.pending_cycles", "l1d_pend_miss.fb_full"],
    "stalls_l1": ["cycle_activity.stalls_l1d_miss", "cycle_activity.stalls_total"],
    "stalls_l2": ["cycle_activity.stalls_l2_miss", "cycle_activity.stalls_l3_miss",
                  "dtlb_load_misses.walk_active"],
    "pf": ["sw_prefetch_access.t0", "load_hit_pre.sw_pf", "mem_load_retired.l1_miss"],
    "loads": ["mem_load_retired.fb_hit", "mem_load_retired.l2_miss", "mem_load_retired.l3_miss"],
}
# smaller fallbacks the probe also tries, for the pairs the decisions need together
FALLBACKS = {
    "exec_pair": ["uops_issued.any", "uops_retired.retire_slots"],
    "mlp_pair": ["l1d_pend_miss.pending", "l1d_pend_miss.pending_cycles"],
    "fb_full": ["l1d_pend_miss.fb_full"],
}
# which decisions need which group's members counted together
NEEDS = {"D3": "exec", "D4": "mlp"}
# how perf is invoked: WALKER_PERF="sudo perf" where counting needs root (the i9)
PERF = os.environ.get("WALKER_PERF", "perf").split()
OVERLAP_MIN = 0.95
RUNNING_MIN = 99.9
OVERHEAD_MAX = 0.01


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


def parse_perf(stderr):
    """perf stat -x, lines: {event: {value, status, run_time, running_pct}}; other lines
    (`Events disabled`, `Events enabled`, warnings) ignored. The fields are perf's: value,
    unit, event, counter run time (ns), percentage running."""
    counts = {}
    for line in stderr.splitlines():
        f = line.split(",")
        if len(f) < 5 or not f[2]:
            continue
        value, event = f[0], f[2].split(":")[0]
        try:
            running = float(f[4]) if f[4] else None
        except ValueError:
            running = None
        try:
            run_time = int(f[3]) if f[3] else None
        except ValueError:
            run_time = None
        if value.startswith("<"):
            counts[event] = {"value": None, "status": value, "run_time": run_time,
                             "running_pct": running}
        else:
            try:
                counts[event] = {"value": float(value), "status": "counted",
                                 "run_time": run_time, "running_pct": running}
            except ValueError:
                pass
    return counts


def event_state(c):
    """an event's measurement: "valid" (counted, its counter ran, not multiplexed),
    "multiplexed" (counted, ran, under RUNNING_MIN), or why it is no measurement at all
    (missing, <not counted>, <not supported>, counted with zero run time). perf prints
    100.00% for a counter that never ran, so the percentage is read only for counters
    that ran; a zero count from a counter that ran is a valid count."""
    if c is None:
        return "missing"
    if c.get("status") != "counted":
        return c.get("status") or "missing"
    if not c.get("run_time"):
        return "never ran (zero run time)"
    if c.get("running_pct") is None or c["running_pct"] < RUNNING_MIN:
        return "multiplexed"
    return "valid"


def command(file, rung, lanes, reps, cpu, events, fifo_dir, barrier=False, binary=None):
    """the ladder under perf (counters enabled only around its walks), or alone. Its
    control FIFOs and barrier are passed on its own command line, after perf's `--`, so
    they reach it through `sudo perf` (which filters the environment): report 140."""
    cmd = [binary or BIN, file, rung, str(lanes), str(reps)]
    if barrier:
        cmd += ["--barrier"]
    if events:
        ctl, ack = os.path.join(fifo_dir, "ctl"), os.path.join(fifo_dir, "ack")
        for p in (ctl, ack):
            if not os.path.exists(p):
                os.mkfifo(p)
        cmd += ["--ctl", ctl, "--ack", ack]
    if cpu is not None:
        cmd = ["taskset", "-c", str(cpu)] + cmd
    if events:
        cmd = PERF + ["stat", "-x", ",", "--control", f"fifo:{ctl},{ack}", "-D", "-1",
                      "-e", "{" + ",".join(events) + "}", "--"] + cmd
    return cmd, dict(os.environ)


def ladder(file, rung, lanes, reps, cpu=None, events=None, allow_fail=False):
    """one run: its JSON line, its counts, and perf's raw output (with allow_fail, a
    failed run returns None for its JSON, and its raw output)"""
    with tempfile.TemporaryDirectory() as d:
        cmd, env = command(file, rung, lanes, reps, cpu, events, d)
        p = subprocess.run(cmd, capture_output=True, text=True, env=env)
    if p.returncode != 0:
        if allow_fail:
            return None, parse_perf(p.stderr), p.stderr + p.stdout
        raise SystemExit(f"walker_ladder: {' '.join(cmd)} failed:\n{p.stderr[-2000:]}")
    out = json.loads(p.stdout.strip().splitlines()[-1])
    return out, parse_perf(p.stderr) if events else {}, p.stderr if events else ""


def four(file, rung, ms, cpus, events, cap=4096):
    """four copies, released together after their setup, each walking until a common
    deadline `ms` after the release: [(cpu, JSON, counts, raw)]"""
    with tempfile.TemporaryDirectory() as d:
        procs = []
        for i, c in enumerate(cpus[:4]):
            sub = os.path.join(d, str(i))
            os.mkdir(sub)
            # REPS caps the walks (the result vector is reserved for it); the deadline ends them
            cmd, env = command(file, rung, 32, cap, c, events, sub, barrier=True)
            procs.append((c, subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                              stderr=subprocess.PIPE, text=True, env=env)))
        for _, p in procs:
            line = p.stdout.readline()
            if line.strip() != "ready":
                raise SystemExit(f"four-core {rung}: a copy did not get ready: {line!r}")
        import time
        deadline = time.time_ns() + int(ms * 1e6)
        for _, p in procs:
            p.stdin.write(f"go {deadline}\n")
            p.stdin.flush()
        res = []
        for c, p in procs:
            so, se = p.communicate()
            if p.returncode != 0:
                raise SystemExit(f"four-core {rung}: {se[-1000:]}")
            res.append((c, json.loads(so.strip().splitlines()[-1]),
                        parse_perf(se) if events else {}, se if events else ""))
        return res


def record(kind, rnd, rung, lanes, group, cpu, out, counts, raw, reps, events=None):
    if events is None:
        events = FIXED + GROUPS[group] if group else []
    out = out or {"rows": 0, "ns_a_row": None, "checked": False, "controlled": False}
    return {"kind": kind, "round": rnd, "rung": rung, "lanes": lanes, "group": group,
            "cpu": cpu, "reps": reps, "rows": out["rows"], "ns_a_row": out["ns_a_row"],
            "checked": out["checked"], "controlled": out.get("controlled", False),
            "capacity_unchanged": out.get("result_capacity_unchanged"),
            "window": [out.get("window_start_ns"), out.get("window_end_ns")],
            "events": events, "counts": counts, "perf_raw": raw,
            "per_row": {e: (counts[e]["value"] / (reps * out["rows"])
                            if reps and out["rows"] and event_state(counts.get(e)) == "valid"
                            else None)
                        for e in events}}


def validity(recs):
    """G0, over every record that enters the analysis: the list of failures (empty: valid)"""
    bad = []
    for r in recs:
        tag = f"{r['kind']} round {r['round']} {r['rung']}{r['lanes']} {r['group']} cpu {r['cpu']}"
        if not r.get("checked"):
            bad.append(f"{tag}: unchecked")
        if r.get("capacity_unchanged") is False:
            bad.append(f"{tag}: the result vector grew inside the window")
        if r.get("events"):
            if not r.get("controlled"):
                bad.append(f"{tag}: counters not bounded by the control FIFO")
            for e in r["events"]:
                st = event_state(r["counts"].get(e))
                if st == "multiplexed":
                    bad.append(f"{tag}: {e} running {r['counts'][e].get('running_pct')}% "
                               "(multiplexed)")
                elif st != "valid":
                    bad.append(f"{tag}: {e} {st}")
    # the four-core windows of one release overlap
    sets = {}
    for r in recs:
        if r["kind"] == "four":
            sets.setdefault((r["round"], r["rung"], r["group"]), []).append(r)
    for k, rs in sets.items():
        starts = [r["window"][0] for r in rs]
        ends = [r["window"][1] for r in rs]
        if None in starts or None in ends or len(rs) != 4:
            bad.append(f"four {k}: windows missing")
            continue
        common = min(ends) - max(starts)
        worst = min(common / max(e - s, 1) for s, e in zip(starts, ends))
        if worst < OVERLAP_MIN:
            bad.append(f"four {k}: windows overlap {worst:.3f} of a window (< {OVERLAP_MIN})")
    # the control overhead is small against the walks it brackets
    for r in recs:
        if r["kind"] == "overhead" and r.get("events"):
            peers = [x for x in recs if x["kind"] == "one" and x["group"] == r["group"]
                     and x["round"] == r["round"] and (x["rung"], x["lanes"]) == ("A", 32)]
            oc = (r["counts"].get("cycles") or {}).get("value")
            pc = [(x["counts"].get("cycles") or {}).get("value") for x in peers]
            pc = [v for v in pc if v]
            if oc is not None and pc and oc > OVERHEAD_MAX * min(pc):
                bad.append(f"overhead round {r['round']} {r['group']}: {oc:.0f} cycles, more "
                           f"than {OVERHEAD_MAX:.0%} of A32's {min(pc):.0f}")
    return bad


def probe_pair(rec20, rec0):
    """one candidate group's probe: the 20-walk window and its 0-walk control, each through
    validity(). Multiplexing is assessed only from counters that ran."""
    bad = validity([rec20]) + validity([rec0])
    states = {e: event_state(rec20["counts"].get(e)) for e in rec20["events"]}
    ran = [s for s in states.values() if s in ("valid", "multiplexed")]
    return {"valid": not bad, "failures": bad, "states": states,
            "controlled": bool(rec20.get("controlled")) and bool(rec0.get("controlled")),
            "multiplexed": ("unassessed" if len(ran) < len(states)
                            else any(s == "multiplexed" for s in states.values()))}


def probe(args):
    """the revised availability probe: first the control path itself, then every candidate
    group as one perf group, bounded by the control FIFO, each with its 0-walk control;
    perf's raw output kept for every run"""
    env = environment()
    if not shutil.which(PERF[-1]):
        raise SystemExit(f"walker_ladder: {PERF[-1]} is not on PATH")
    ver = subprocess.run(PERF + ["--version"], capture_output=True, text=True).stdout.strip()
    cpu = physical_cpus()[0]
    t = stamp()
    base = os.path.join(args.out, f"walker-probe-{t}")
    res = {"time": t, "environment": env, "perf_version": ver,
           "note": "availability only: counts kept as perf printed them, not analysed"}

    def finish(md):
        json.dump(res, open(base + ".json", "w"), indent=1)
        open(base + ".md", "w").write("\n".join(md) + "\n")
        print("\n".join(md))
        print(f"\nwrote {base}.json and {base}.md (perf's raw output in the .json)")

    md = [f"# Walker counter probe ({t})", "", "Environment (numbers only): " +
          ", ".join(f"{k} {v}" for k, v in env.items()) + f"; {ver}", ""]
    # the control path first: one run of cycles and instructions, 3 walks, and its control
    ev = list(FIXED)
    o3, c3, raw3 = ladder(args.file, "A", 32, 3, cpu, ev, allow_fail=True)
    o0, c0, raw0 = ladder(args.file, "A", 32, 0, cpu, ev, allow_fail=True)
    ctl = probe_pair(record("probe", 0, "A", 32, None, cpu, o3, c3, raw3, 3, ev),
                     record("probe", 0, "A", 32, None, cpu, o0, c0, raw0, 0, ev))
    ctl.update({"perf_raw": raw3, "perf_raw_control_only": raw0})
    res["control_path"] = ctl
    if not ctl["valid"]:
        md += ["**The control path failed: no group was probed.** The counters were not "
               "enabled and counted around the walks, so no availability can be read.", "",
               "Failures:", ""] + [f"- {f}" for f in ctl["failures"]]
        finish(md)
        raise SystemExit(1)
    md += ["The control path holds: cycles and instructions counted, bounded by the FIFO, "
           "in the 3-walk window and its 0-walk control.", "",
           "| group | valid (window and control) | multiplexed | events' states | "
           "cycles, 20 walks | cycles, control only |", "|---|---|---|---|--:|--:|"]
    rows = []
    for name, evs in list(GROUPS.items()) + list(FALLBACKS.items()) + \
            [(f"single:{e}", [e]) for g in GROUPS.values() for e in g]:
        ev = FIXED + evs
        o20, c20, r20 = ladder(args.file, "A", 32, 20, cpu, ev, allow_fail=True)
        o0, c0, r0 = ladder(args.file, "A", 32, 0, cpu, ev, allow_fail=True)
        pr = probe_pair(record("probe", 0, "A", 32, None, cpu, o20, c20, r20, 20, ev),
                        record("probe", 0, "A", 32, None, cpu, o0, c0, r0, 0, ev))
        pr.update({"group": name, "events": ev,
                   "running_pct": {e: (c20.get(e) or {}).get("running_pct") for e in ev},
                   "run_time": {e: (c20.get(e) or {}).get("run_time") for e in ev},
                   "cycles_20_walks": (c20.get("cycles") or {}).get("value"),
                   "cycles_control_only": (c0.get("cycles") or {}).get("value"),
                   "perf_raw": r20, "perf_raw_control_only": r0})
        rows.append(pr)
        bad_states = sorted({v for v in pr["states"].values() if v != "valid"})
        md.append(f"| {name} | {pr['valid']} | {pr['multiplexed']} | "
                  f"{', '.join(bad_states) or 'all valid'} | {pr['cycles_20_walks']} | "
                  f"{pr['cycles_control_only']} |")
    res["groups"] = rows
    finish(md)


def reps_for(lanes, reps):
    """walks a run: a quarter at one lane, whose walk is about ten times slower"""
    return max(4, reps // 4) if lanes == 1 else reps


def preflight(file, cpu):
    """each group once, controlled, on a short window: stop before the capture if any event
    is uncounted or multiplexed"""
    bad = []
    for name, evs in GROUPS.items():
        out, c, _ = ladder(file, "A", 32, 3, cpu, FIXED + evs)
        r = record("one", -1, "A", 32, name, cpu, out, c, "", 3)
        bad += validity([r])
    if bad:
        raise SystemExit("walker_ladder: preflight failed, no capture made:\n  " + "\n  ".join(bad))
    print(f"preflight: all {len(GROUPS)} groups counted as perf groups, unmultiplexed, "
          "bounded by the control FIFO")


def run(args):
    cpus = physical_cpus()
    cpu = cpus[0]
    if not args.no_perf:
        preflight(args.file, cpu)
    t = stamp()
    path = os.path.join(args.out, f"walker-ladder-{t}.jsonl")
    jl = open(path, "w")
    configs = [(r, k) for r in RUNGS for k in LANES] + [("S", 32)]
    groups = [None] if args.no_perf else list(GROUPS)
    rng = random.Random(args.seed)

    def emit(rec):
        jl.write(json.dumps(rec) + "\n")
        jl.flush()
    for rnd in range(args.rounds):
        order = [(r, k, g) for (r, k) in configs for g in groups]
        rng.shuffle(order)
        for rung, lanes, g in order:
            reps = reps_for(lanes, args.reps)
            out, c, raw = ladder(args.file, rung, lanes, reps, cpu, FIXED + GROUPS[g] if g else None)
            emit(record("one", rnd, rung, lanes, g, cpu, out, c, raw, reps))
            print(f"round {rnd} {rung}{lanes} {g or 'time'}: {out['ns_a_row']:.3f} ns a row")
        for g in groups:
            if g:
                out, c, raw = ladder(args.file, "A", 32, 0, cpu, FIXED + GROUPS[g])
                emit(record("overhead", rnd, "A", 32, g, cpu, out, c, raw, 0))
    if len(cpus) >= 4:
        for rnd in range(args.rounds):
            for rung in ["A", "B", "P"]:
                for g in ([None] if args.no_perf else ["exec", "mlp"]):
                    for c, out, cnt, raw in four(args.file, rung, args.four_ms, cpus,
                                                 FIXED + GROUPS[g] if g else None,
                                                 args.four_cap):
                        emit(record("four", rnd, rung, 32, g, c, out, cnt, raw, out["reps"]))
                    print(f"round {rnd} four-core {rung}32 {g or 'time'}")
    jl.close()
    print(f"wrote {path}")
    summarise(path)


def med(xs):
    xs = [x for x in xs if x is not None]
    return statistics.median(xs) if xs else None


def summarise(path):
    """the protocol's gates and decisions (§6)"""
    recs = [json.loads(l) for l in open(path)]
    one = [r for r in recs if r["kind"] == "one"]
    fourc = [r for r in recs if r["kind"] == "four"]
    bad = validity(recs)
    p32 = [x["ns_a_row"] for x in one if (x["rung"], x["lanes"]) == ("P", 32)]
    spread = max(p32) / min(p32)
    g1 = spread <= 1.05
    print(f"G0: {'holds' if not bad else str(len(bad)) + ' failures, e.g. ' + bad[0]}")
    print(f"G1: P32's ns a row, max / min over its {len(p32)} runs: {spread:.3f} "
          + ("(holds)" if g1 else "(fails)"))
    valid = g1 and not bad

    def cfg(rung, k, group=None):
        return [x for x in one if (x["rung"], x["lanes"]) == (rung, k)
                and (group is None or x["group"] == group)]
    counted = any(x["group"] for x in one)
    if counted:
        # cycles a row from every group's runs of a config (each run's own cycles)
        metric = {(r, k): med([x["per_row"].get("cycles") for x in cfg(r, k)])
                  for r in RUNGS for k in LANES}
        unit = "cycles a row"
    else:
        metric = {(r, k): med([x["ns_a_row"] for x in cfg(r, k)]) for r in RUNGS for k in LANES}
        unit = "ns a row (no counters: timing only)"
    print(f"\n{unit}, medians over runs (one core)")
    print("| rung | " + " | ".join(f"{k} lanes" for k in LANES) + " |")
    print("|---|" + "--:|" * len(LANES))
    for r in RUNGS:
        print(f"| {r} | " + " | ".join("" if metric[(r, k)] is None else f"{metric[(r, k)]:.3f}"
                                       for k in LANES) + " |")
    out = {"unit": unit, "G0_failures": bad, "G1_spread": spread, "valid": valid}
    # D1/D2: the bare reference at 32 lanes, the same lane count as every contrast: the
    # cheapest of A, B and M (M is the shipped step without its output store)
    bare32 = min(metric[("A", 32)], metric[("B", 32)], metric[("M", 32)])
    pm = metric[("P", 32)]
    ratio = pm / bare32
    out.update({"bare32": bare32, "P32": pm, "P32_over_bare32": ratio,
                "best_bare_any_lanes": min(v for (r, k), v in metric.items()
                                           if r in ("A", "B", "M") and v is not None)})
    # net contrasts at 32 lanes (each changes more than one thing: the protocol's §2 table)
    steps = {"A->B": metric[("B", 32)] - metric[("A", 32)],
             "B->C": metric[("C", 32)] - metric[("B", 32)],
             "C->D": metric[("D", 32)] - metric[("C", 32)],
             "M->P": pm - metric[("M", 32)],
             "D->P": pm - metric[("D", 32)]}
    out["contrasts_32"] = steps
    if ratio <= 1.15:
        out["D1_D2"] = "D1: P32 within 15% of the bare chase at 32 lanes: no walker scheduling rewrite"
    elif ratio >= 1.5:
        out["D1_D2"] = ("D2: P32 at least 1.5x the bare chase at 32 lanes; the largest net "
                        "contrast: " + max(steps, key=steps.get))
    else:
        out["D1_D2"] = "between 1.15 and 1.5: reported, no rewrite on this evidence"
    if counted:
        # D3: issue and retirement pressure, from one group's simultaneous counts
        ex = cfg("P", 32, NEEDS["D3"])
        if ex:
            def share(e):
                return med([x["per_row"].get(e) / x["per_row"]["cycles"] / 4 for x in ex
                            if x["per_row"].get(e) is not None and x["per_row"].get("cycles")])
            iss, ret, fe = share("uops_issued.any"), share("uops_retired.retire_slots"), \
                share("idq_uops_not_delivered.core")
            out["D3"] = {"issued_share": iss, "retired_share": ret,
                         "front_end_undelivered_share": fe,
                         "reading": ("high issue pressure (issued >= 0.70 of 4 a cycle): added "
                                     "instructions are unlikely to be absorbed; not a "
                                     "bottleneck classification")
                         if iss is not None and iss >= 0.70 else
                         "issue pressure below 0.70 of 4 a cycle: no classification follows"}
        # D4: demand MLP and fill-buffer pressure, from the mlp group's simultaneous counts

        def mlp(k):
            xs = cfg("B", k, NEEDS["D4"])
            return med([x["per_row"]["l1d_pend_miss.pending"] / x["per_row"]["l1d_pend_miss.pending_cycles"]
                        for x in xs if x["per_row"].get("l1d_pend_miss.pending_cycles")])

        def fb(k):
            xs = cfg("B", k, NEEDS["D4"])
            return med([x["per_row"]["l1d_pend_miss.fb_full"] / x["per_row"]["cycles"]
                        for x in xs if x["per_row"].get("cycles")
                        and x["per_row"].get("l1d_pend_miss.fb_full") is not None])
        m16, m32, f16, f32 = mlp(16), mlp(32), fb(16), fb(32)
        if None not in (m16, m32, f16, f32):
            consistent = m32 / m16 < 1.10 and f32 >= 2 * f16 and f32 >= 0.01
            out["D4"] = {"demand_mlp_B16": m16, "demand_mlp_B32": m32,
                         "fb_full_events_a_cycle_B16": f16, "fb_full_events_a_cycle_B32": f32,
                         "reading": ("consistent with a miss-capacity constraint (not an "
                                     "exclusive diagnosis)") if consistent else
                         "no plateau with rising fill-buffer pressure"}
        else:
            out["D4"] = "not read: the mlp group's members were not all counted together"
    if fourc:
        def q(rung):
            return med([x["ns_a_row"] for x in fourc if x["rung"] == rung])
        n1 = med([x["ns_a_row"] for x in cfg("P", 32)]) / med([x["ns_a_row"] for x in cfg("B", 32)])
        r = (q("P") / q("B")) / n1
        out["D5"] = {"ratio_of_ratios": r,
                     "reading": ("transfers: |r - 1| <= 0.15" if abs(r - 1) <= 0.15
                                 else "does not transfer (|r - 1| > 0.15): report and stop")}
    if not valid:
        out["read"] = "VOID: G0 or G1 failed, so none of the decisions above is read"
    print("\n" + json.dumps(out, indent=1))
    json.dump(out, open(path.replace(".jsonl", "-summary.json"), "w"), indent=1)
    return out


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("probe")
    a.add_argument("file")
    a.add_argument("--out", default=".")
    r = sub.add_parser("run")
    r.add_argument("file")
    r.add_argument("--rounds", type=int, default=3)
    r.add_argument("--reps", type=int, default=100)
    r.add_argument("--seed", type=int, default=20261101)
    r.add_argument("--four-ms", type=float, default=300.0)
    r.add_argument("--four-cap", type=int, default=4096)
    r.add_argument("--out", default=".")
    r.add_argument("--no-perf", action="store_true")
    s = sub.add_parser("summarise")
    s.add_argument("jsonl")
    args = ap.parse_args()
    if getattr(args, "out", None):
        os.makedirs(args.out, exist_ok=True)
    if args.cmd != "summarise" and not os.path.exists(BIN):
        sys.exit("walker_ladder: build it first (cargo build --release --example walker_ladder)")
    {"probe": probe, "run": run, "summarise": lambda a: summarise(a.jsonl)}[args.cmd](args)


if __name__ == "__main__":
    main()
