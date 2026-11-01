"""The shipped decoder's one-to-four-worker resource attribution
(docs/journal/2026-11-01-decoder-attribution-protocol.md): the analysis of the occupancy
instrument's raw marks (C3), the counters and samples of the uninstrumented decode (C4),
and the registered rules (§5). Steps, as bench/attribution_run.sh calls them:

    attribution.py events  --exe EXE --file F --cpus C --out events.json
    attribution.py stat    --plan events.json --exe EXE --manifest M --cpus C --workers 1,4 --reps N --out stat.json
    attribution.py record  --plan events.json --exe EXE --manifest M --cpus C --workers 1,4 --reps N --dir D --out record.json
    attribution.py marks   --dir MARKS_DIR --w W --cpus N --out marks.json
    attribution.py summary --manifest M --occupancy-dir D --stat stat.json --record record.json --events events.json --out result.json

`perf` is run as `$PERF` (default `perf`; the runner sets `sudo -n perf` where it needs
root). Nothing here is a timing comparison: the figures are observations of one build.
"""
import argparse
import collections
import json
import math
import os
import re
import shlex
import statistics
import subprocess
import sys
import time
from pathlib import Path

PERF = shlex.split(os.environ.get("PERF", "perf"))

# --------------------------------------------------------------------------- event plan
# G0 is the fixed counters and the task clock; G1-G6 the programmable events (Skylake
# family: the i9-10900K's), at most three a group so that a group fits beside the NMI
# watchdog's counter; IMC the memory controller, machine-wide. `events` tries each group
# as a group ({...}: scheduled together or not at all), and where a group can't be counted
# whole, each event alone; an event countable in neither way is recorded unavailable. No
# event is ever replaced by another.
GROUPS = {
    "G0": ["cycles", "ref-cycles", "instructions", "task-clock"],
    "G1": ["cycle_activity.stalls_l3_miss", "cycle_activity.stalls_mem_any", "cycle_activity.stalls_total"],
    "G2": ["mem_load_retired.l3_miss", "mem_load_retired.l3_hit"],
    "G3": ["l1d_pend_miss.pending", "l1d_pend_miss.pending_cycles", "l1d_pend_miss.fb_full"],
    "G4": ["offcore_requests.all_data_rd", "offcore_requests_outstanding.all_data_rd"],
    "G5": ["idq_uops_not_delivered.core", "uops_issued.any", "uops_retired.retire_slots"],
    "G6": ["int_misc.recovery_cycles"],
}
IMC = ["uncore_imc/data_reads/", "uncore_imc/data_writes/"]
SAMPLED = ["cycles", "cycle_activity.stalls_l3_miss"]
# sampling periods: about 2,000 samples a second a core at 4-5 GHz for cycles, and as many
# again for the rarer stall cycles
PERIOD = {"cycles": "2000003", "cycle_activity.stalls_l3_miss": "200003"}
# the stage a sampled symbol belongs to (examples/attribution_decode's out-of-line
# functions, demangled or in Rust's v0 mangling, which perf may not demangle); anything
# inlined elsewhere is "worker_other". The C library's copies are a stage of their own
# ("memcpy": mostly the segments' assembly, part of the walk, and RLE1's output): R1
# reports build + walk without and with them, the attribution's two bounds.
STAGES = [
    ("build", re.compile(r"mibz2::bwt::build\b|3bwt5build")),
    ("walk", re.compile(r"room_segments|SegScratch.*assemble|10SegScratch8assemble|mibz2::bwt::walk\b|3bwt4walk|walk_segments")),
    ("mtf", re.compile(r"mibz2::mtf::|5mibz23mtf")),
    ("rle_crc", re.compile(r"mibz2::(rle|crc)::|5mibz23(rle|crc)")),
    ("memcpy", re.compile(r"__mem(move|cpy|set)|^mem(move|cpy|set)")),
]
STAGE_NAMES = ["build", "walk", "mtf", "rle_crc", "memcpy", "worker_other", "kernel", "scanner", "feeder", "reader"]


def sampled_event(perf_data):
    """the event a perf.data file holds (perf record falls back to cpu-clock or task-clock
    where the PMU can't count the event asked, and says so only on stderr)"""
    p = subprocess.run(PERF[-1:] + ["evlist", "-i", str(perf_data)], capture_output=True, text=True)
    return p.stdout.strip()


def role_of(name):
    if re.fullmatch(r"mibz2-\d+", name):
        return "worker"
    return {"mibz2-scan": "scanner", "mibz2-read": "feeder"}.get(name, "reader")


def stage_of(comm, sym):
    r = role_of(comm)
    if r != "worker":
        return r
    if sym.startswith("[k]") or "[kernel" in sym:
        return "kernel"
    for name, rx in STAGES:
        if rx.search(sym):
            return name
    return "worker_other"


def run(cmd, stdin=None, stdout=subprocess.DEVNULL):
    t = time.perf_counter()
    p = subprocess.run(cmd, stdin=stdin, stdout=stdout, stderr=subprocess.PIPE, text=True)
    return p, time.perf_counter() - t


def parse_stat(path):
    """perf stat -x, output: {event: (value or None, percent running)}"""
    out = {}
    for line in Path(path).read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        f = line.split(",")
        if len(f) < 5:
            continue
        val, ev, pct = f[0], f[2], f[4]
        try:
            v = float(val)
        except ValueError:
            v = None  # <not supported>, <not counted>
        try:
            p = float(pct.rstrip("%")) if pct else None
        except ValueError:
            p = None
        out[ev] = (v, p)
    return out


def perf_stat(events, cmd, stdin_path, out_path, machine_wide=False, group=True):
    spec = "{" + ",".join(events) + "}" if group and len(events) > 1 else ",".join(events)
    args = PERF + ["stat", "-x,", "-o", str(out_path), "-e", spec]
    if machine_wide:
        args.append("-a")
    with open(stdin_path, "rb") if stdin_path else open(os.devnull, "rb") as si:
        p, wall = run(args + ["--"] + cmd, stdin=si)
    vals = parse_stat(out_path) if Path(out_path).exists() else {}
    return p.returncode, wall, vals, p.stderr[-2000:]


def counted(vals, events):
    """every event counted, for all the time it was enabled (no multiplexing)"""
    return all(e in vals and vals[e][0] is not None and (vals[e][1] is None or vals[e][1] >= 99.99) for e in events)


# --------------------------------------------------------------------------- events
def events(a):
    """Which events this machine counts, and how they will be grouped: the plan the
    counters then follow exactly (recorded before any reading)"""
    tmp = Path(a.out).with_suffix(".probe")
    tmp.mkdir(exist_ok=True)
    cmd = ["taskset", "-c", a.cpus, a.exe, a.file, "1"]
    plan = dict(machine=sh("lscpu"), perf=sh(*PERF, "--version"), nmi_watchdog=rd("/proc/sys/kernel/nmi_watchdog"),
                paranoid=rd("/proc/sys/kernel/perf_event_paranoid"), groups={}, unavailable={}, singletons=[])
    for g, evs in GROUPS.items():
        rc, _, vals, err = perf_stat(evs, cmd, None, tmp / f"{g}.csv")
        if rc == 0 and counted(vals, evs):
            plan["groups"][g] = evs
            continue
        # the group whole failed: each event alone
        for e in evs:
            rc, _, v1, err1 = perf_stat([e], cmd, None, tmp / f"{g}-{e}.csv", group=False)
            if rc == 0 and counted(v1, [e]):
                plan["singletons"].append(dict(group=g, event=e))
            else:
                plan["unavailable"][e] = (v1.get(e, (None, None)), err1.strip()[-300:])
    rc, _, vals, err = perf_stat(IMC, ["sleep", "0.2"], None, tmp / "imc.csv", machine_wide=True)
    plan["imc"] = IMC if rc == 0 and counted(vals, IMC) else None
    if plan["imc"] is None:
        plan["unavailable"]["uncore_imc"] = (None, err.strip()[-300:])
    plan["sampled"] = []
    for e in SAMPLED:
        p, _ = run(PERF + ["record", "-q", "-e", e, "-c", PERIOD[e], "-o", str(tmp / "probe.data"), "--"] + cmd)
        if os.environ.get("CHOWN"):
            subprocess.run(shlex.split(os.environ["CHOWN"]) + [str(tmp / "probe.data")], check=False)
        got = sampled_event(tmp / "probe.data") if p.returncode == 0 else ""
        if p.returncode == 0 and got == e:
            plan["sampled"].append(e)
        else:
            plan.setdefault("unsampled", {})[e] = f"recorded {got!r}" if got else p.stderr.strip()[-300:]
    Path(a.out).write_text(json.dumps(plan, indent=1) + "\n")
    n_ok = sum(len(v) for v in plan["groups"].values()) + len(plan["singletons"])
    print(f"events: {n_ok} countable without multiplexing ({len(plan['groups'])} whole groups, "
          f"{len(plan['singletons'])} alone); unavailable: {sorted(plan['unavailable']) or 'none'}; "
          f"sampled: {plan['sampled'] or 'none'}")


def passes(plan):
    out = [(g, evs, True) for g, evs in plan["groups"].items()]
    out += [(f"{s['group']}:{s['event']}", [s["event"]], False) for s in plan["singletons"]]
    return out


def manifest(path):
    rows = []
    for l in Path(path).read_text().splitlines():
        name, role, file, decoded_sha, decoded_bytes = l.split("\t")
        rows.append(dict(name=name, role=role, file=file, sha=decoded_sha, bytes=int(decoded_bytes)))
    return rows


# --------------------------------------------------------------------------- stat
def stat(a):
    plan = json.loads(Path(a.plan).read_text())
    d = Path(a.out).with_suffix(".raw")
    d.mkdir(exist_ok=True)
    res = []
    workers = [int(x) for x in a.workers.split(",")]
    for rep in range(a.reps):
        for c in manifest(a.manifest):
            ws = workers[rep % len(workers):] + workers[:rep % len(workers)]
            for w in ws:
                cmd = ["taskset", "-c", a.cpus, a.exe, c["file"], str(w)]
                for name, evs, grouped in passes(plan):
                    f = d / f"{c['name']}-w{w}-{name.replace(':', '_')}-r{rep}.csv"
                    rc, wall, vals, err = perf_stat(evs, cmd, None, f, group=grouped)
                    res.append(dict(corpus=c["name"], w=w, rep=rep, group=name, rc=rc, wall_s=wall,
                                    complete=counted(vals, evs), values={e: vals.get(e, (None, None))[0] for e in evs},
                                    pct={e: vals.get(e, (None, None))[1] for e in evs}))
                if plan.get("imc"):
                    # the machine-wide traffic, less an idle reading of the same length just before
                    _, wall0, _, _ = perf_stat(["task-clock"], cmd, None, d / "warm.csv", group=False)
                    rc0, _, idle, _ = perf_stat(IMC, ["sleep", f"{wall0:.3f}"], None, d / f"{c['name']}-w{w}-imcidle-r{rep}.csv", machine_wide=True)
                    rc, wall, vals, _ = perf_stat(IMC, cmd, None, d / f"{c['name']}-w{w}-imc-r{rep}.csv", machine_wide=True)
                    res.append(dict(corpus=c["name"], w=w, rep=rep, group="IMC", rc=rc, wall_s=wall, idle_wall_s=wall0,
                                    complete=counted(vals, IMC) and counted(idle, IMC),
                                    values={e: vals.get(e, (None, None))[0] for e in IMC},
                                    idle={e: idle.get(e, (None, None))[0] for e in IMC}))
    Path(a.out).write_text(json.dumps(dict(plan=plan, cpus=a.cpus, reps=a.reps, runs=res), indent=1) + "\n")
    bad = [r for r in res if not r["complete"] or r["rc"] != 0]
    print(f"stat: {len(res)} passes, {len(bad)} incomplete (recorded, not interpreted)")


# --------------------------------------------------------------------------- record
REPORT = re.compile(r"^\s*([\d.]+)%\s+(\S+)\s+(.*?)\s*$")


def record(a):
    plan = json.loads(Path(a.plan).read_text())
    d = Path(a.dir)
    d.mkdir(parents=True, exist_ok=True)
    res = []
    for c in manifest(a.manifest):
        for w in [int(x) for x in a.workers.split(",")]:
            for e in plan.get("sampled", []):
                for rep in range(a.reps):
                    f = d / f"{c['name']}-w{w}-{e.replace('.', '_')}-r{rep}.data"
                    cmd = ["taskset", "-c", a.cpus, a.exe, c["file"], str(w)]
                    p, wall = run(PERF + ["record", "-q", "-e", e, "-c", PERIOD[e], "-o", str(f), "--"] + cmd)
                    if os.environ.get("CHOWN"):
                        subprocess.run(shlex.split(os.environ["CHOWN"]) + [str(f)], check=False)
                    got = sampled_event(f)
                    rep_txt = subprocess.run(PERF[-1:] + ["report", "-i", str(f), "--stdio", "--no-children", "-q",
                                                          "--sort", "comm,sym", "--percent-limit", "0"],
                                             capture_output=True, text=True).stdout
                    by = collections.Counter()
                    for line in rep_txt.splitlines():
                        m = REPORT.match(line)
                        if m:
                            by[stage_of(m.group(2), m.group(3))] += float(m.group(1)) / 100
                    ok = p.returncode == 0 and got == e
                    res.append(dict(corpus=c["name"], w=w, event=e, rep=rep, rc=p.returncode if ok else f"event recorded {got!r}",
                                    wall_s=wall, shares=dict(by)))
    Path(a.out).write_text(json.dumps(dict(runs=res), indent=1) + "\n")
    print(f"record: {len(res)} sampled runs")


# --------------------------------------------------------------------------- marks
def load_marks(p):
    names, rows = {}, []
    val = lambda x: None if x == "-" else int(x)  # noqa: E731
    with open(p) as f:
        next(f)
        for l in f:
            th, name, ns, kind, place, cpu, runq, wait, slices, ln, cap = l.rstrip("\n").split("\t")
            th = int(th)
            names[th] = name
            rows.append((int(ns), th, kind, val(place), val(cpu), val(wait), val(ln)))
    return names, rows


def analyse_marks(path, w, cpus_alloc):
    """One run's raw marks: the windows, the workers' idle time by cause, the scanner's
    ceiling and, where the clocks were read (level 2), CPU by role and phase and the
    run-queue wait. Definitions: protocol §5 and the comments below."""
    names, rows = load_marks(path)
    roles = {t: role_of(n) for t, n in names.items()}
    rows.sort(key=lambda r: (r[0], r[1]))
    by_t = collections.defaultdict(list)
    for r in rows:
        by_t[r[1]].append(r)
    ds_times = sorted(r[0] for r in rows if r[2] == "DecodeStart")
    last_issued = max((r[0] for r in rows if r[2] == "Issued"), default=0)
    span0 = min(r[0] for r in rows)
    span1 = max(r[0] for r in rows if r[2] in ("Taken", "DecodeEnd"))
    t_s = ds_times[min(w, len(ds_times)) - 1]  # the w-th block's start: all workers have had work
    t_d = max(last_issued, t_s)  # the scanner's last block read: the drain from here
    out_bytes = sum(r[6] for r in rows if r[2] == "DecodeEnd" and r[6] is not None)

    # the sweep (each interval credited to the state after the marks at its start)
    decoding = set()
    base, inp, done = "reading", False, False
    issued = enq_unclaimed = taken = 0
    started, ready = set(), set()
    last_enq = max((r[0] for r in rows if r[2] == "Enqueued"), default=0)
    idle = collections.defaultdict(float)
    # the instrument's own split, for the validity check only: idle by the scanner's state,
    # whether or not a job sat in the queue (the instrument reports the handoff apart)
    idle_by_scan = collections.defaultdict(float)
    reading_s = slot_s = input_s = hol_later_s = 0.0
    prev = span0
    i = 0
    while i < len(rows) and rows[i][0] <= span1:
        t = rows[i][0]
        dt = (t - prev) / 1e9
        if dt > 0 and prev >= span0:
            n_idle = w - len(decoding)
            scan = "done" if done else ("input" if inp else base)
            if scan == "reading":
                reading_s += dt
            elif scan == "slot":
                slot_s += dt
            elif scan == "input":
                input_s += dt
            if n_idle > 0:
                idle_by_scan[scan] += n_idle * dt
                if enq_unclaimed > 0:
                    cause = "handoff"
                elif scan == "done":
                    cause = "scanner_done"
                elif scan == "input":
                    cause = "input_wait"
                elif scan == "reading":
                    cause = "scanner_reading"
                else:
                    e = taken  # the earliest untaken block (taken in order)
                    if e in ready:
                        cause = "slot_reader_binds"
                    elif e in started:
                        cause = "slot_head_of_line"
                        if any(p > e for p in ready):
                            hol_later_s += n_idle * dt
                    else:
                        cause = "slot_other"
                win = "startup" if prev < t_s else ("drain" if prev >= t_d else "steady")
                idle[(win, cause)] += n_idle * dt
        while i < len(rows) and rows[i][0] == t:
            _, th, kind, place, *_ = rows[i]
            r = roles[th]
            if r == "worker" and kind == "DecodeStart":
                decoding.add(th)
                started.add(place)
            elif r == "worker" and kind == "DecodeEnd":
                decoding.discard(th)
            elif kind == "JobGot":
                enq_unclaimed -= 1
            elif kind == "Ready":
                ready.add(place)
            elif kind == "Taken":
                ready.discard(place)
                taken = place + 1
            elif r == "scanner":
                if kind == "SlotWait":
                    base = "slot"
                elif kind == "SlotGot":
                    base = "reading"
                elif kind == "InputWait":
                    inp = True
                elif kind in ("InputGot", "InputEnd"):
                    inp = False
                elif kind == "Issued":
                    issued += 1
                elif kind == "Enqueued":
                    enq_unclaimed += 1
                    if t >= last_enq:
                        done = True
            i += 1
        prev = t
    span = (span1 - span0) / 1e9
    worker_time = w * span
    idle_total = sum(idle.values())
    res = dict(file=Path(path).name, w=w, span_s=span, out_bytes=out_bytes, worker_time_s=worker_time,
               idle_s=idle_total, idle={f"{k[0]}.{k[1]}": v for k, v in sorted(idle.items())},
               idle_by_scanner_state=dict(idle_by_scan),
               startup_s=(t_s - span0) / 1e9, drain_s=(span1 - t_d) / 1e9,
               scanner=dict(reading_s=reading_s, slot_s=slot_s, input_s=input_s,
                            cap_MBps=out_bytes / reading_s / 1e6 if reading_s > 0 else None),
               hol_s=sum(v for k, v in idle.items() if k[1] == "slot_head_of_line"), hol_later_ready_s=hol_later_s)

    # level 2: the clocks
    cpu_rows = [r for r in rows if r[4] is not None]
    if cpu_rows:
        role_cpu = collections.defaultdict(float)
        for th, rs in by_t.items():
            st = [r for r in rs if r[2] == "ThreadStart" and r[4] is not None]
            en = [r for r in rs if r[2] == "ThreadEnd" and r[4] is not None]
            if not en:
                continue
            # decoder threads: the absolute thread CPU (a lazily started trace still
            # counts the work before its first mark); the reader: from its first mark
            c = en[-1][4] - (st[0][4] if roles[th] == "reader" and st else 0)
            role_cpu[roles[th]] += c / 1e9
        phase = collections.defaultdict(float)
        rq = dec_el = 0.0
        for th, rs in by_t.items():
            if roles[th] != "worker":
                continue
            cur = {}
            for ns, _, kind, place, cpu, wait, _ in rs:
                if kind in ("DecodeStart", "InvertStart", "WalkStart", "InvertEnd", "DecodeEnd"):
                    cur[kind] = (ns, cpu, wait)
                if kind == "DecodeEnd" and "DecodeStart" in cur:
                    a0, e = cur["DecodeStart"], cur["DecodeEnd"]
                    dec_el += (e[0] - a0[0]) / 1e9
                    if e[2] is not None and a0[2] is not None:
                        rq += (e[2] - a0[2]) / 1e9
                    seq = ["DecodeStart", "InvertStart", "WalkStart", "InvertEnd", "DecodeEnd"]
                    lab = ["mtf", "build", "walk", "rle_crc"]
                    if all(k in cur for k in seq):
                        for x, y, n in zip(seq, seq[1:], lab):
                            phase[n] += (cur[y][1] - cur[x][1]) / 1e9
                    else:
                        phase["decode_unsplit"] += (e[1] - a0[1]) / 1e9
                    cur = {}
        dec_cpu = sum(phase.values())
        phase["worker_other"] = role_cpu["worker"] - dec_cpu
        total = sum(role_cpu.values())
        res.update(cpu_by_role_s=dict(role_cpu), cpu_by_phase_s=dict(phase), cpu_total_s=total,
                   cores_busy=total / span if span else None, unused_core_s=cpus_alloc * span - total,
                   runq_wait_decode_s=rq, decode_elapsed_s=dec_el)
    return res


def marks(a):
    out = [analyse_marks(p, a.w, a.cpus) for p in sorted(Path(a.dir).glob(f"*-t{a.w}-*.tsv"))]
    Path(a.out).write_text(json.dumps(out, indent=1) + "\n")
    print(f"marks: {len(out)} runs at w = {a.w}")


# --------------------------------------------------------------------------- summary
def mean(xs):
    xs = [x for x in xs if x is not None]
    return statistics.fmean(xs) if xs else None


def summary(a):
    """The registered rules (§5) on the primary; the controls the same, reported beside it.
    Each hypothesis gets a status and the share of its question it accounts for; the
    supported ones are ranked by that share, and more than one can be supported."""
    corpora = manifest(a.manifest)
    stat_j = json.loads(Path(a.stat).read_text()) if a.stat and Path(a.stat).exists() else dict(runs=[])
    rec_j = json.loads(Path(a.record).read_text()) if a.record and Path(a.record).exists() else dict(runs=[])
    ev = json.loads(Path(a.events).read_text()) if a.events and Path(a.events).exists() else {}
    out = dict(protocol="docs/journal/2026-11-01-decoder-attribution-protocol.md", corpora={},
               unavailable_events=sorted(ev.get("unavailable", {})), sampled_events=ev.get("sampled", []))
    for c in corpora:
        name, nbytes = c["name"], c["bytes"]
        occ_dir = Path(a.occupancy_dir) / name
        occ = json.loads((occ_dir / "occupancy.json").read_text())
        per_w = {}
        for w in (1, 4):
            runs = [r for r in occ["runs"] if r["threads"] == w]
            kv = lambda r: dict(p.split("=", 1) for p in r["line"].split() if "=" in p)  # noqa: E731
            off = [kv(r) for r in runs if r["mode"] == "off"]
            res = next((x for x in occ["results"] if x["threads"] == w), {})
            pert = {m: res.get("perturbation", {}).get(m, {}).get("verdict", "missing") for m in ("timeline", "full")}
            ok = {m: v.startswith("no perturbation") for m, v in pert.items()}
            tl = [analyse_marks(p, w, a.cpus) for p in sorted(occ_dir.glob(f"marks/*-t{w}-timeline.tsv"))]
            fu = [analyse_marks(p, w, a.cpus) for p in sorted(occ_dir.glob(f"marks/*-t{w}-full.tsv"))]
            st = [r for r in stat_j["runs"] if r["corpus"] == name and r["w"] == w and r["complete"] and r["rc"] == 0]
            gv = lambda e: mean([r["values"].get(e) for r in st if e in r["values"]])  # noqa: E731
            task_s = mean([r["values"].get("task-clock") / 1e3 for r in st if r["values"].get("task-clock")])  # msec
            cyc, ins, ref = gv("cycles"), gv("instructions"), gv("ref-cycles")
            gb = nbytes / 1e9
            row = dict(
                perturbation=pert,
                off=dict(throughput_MBps=mean([float(k["out"]) / float(k["wall"]) / 1e6 for k in off]),
                         cpu_s_per_GB=mean([float(k["cpu"]) / (float(k["out"]) / 1e9) for k in off])),
                counters=dict(cpu_s_per_GB=task_s / gb if task_s else None,
                              cycles_per_byte=cyc / nbytes if cyc else None,
                              instructions_per_byte=ins / nbytes if ins else None,
                              effective_GHz=cyc / task_s / 1e9 if cyc and task_s else None,
                              cycles_per_ref_cycle=cyc / ref if cyc and ref else None,
                              per_byte={e: (gv(e) / nbytes if gv(e) is not None else None)
                                        for g in list(GROUPS.values())[1:] for e in g}),
                samples={e: {s: mean([r["shares"].get(s, 0.0) for r in rec_j["runs"]
                                      if r["corpus"] == name and r["w"] == w and r["event"] == e and r["rc"] == 0])
                             for s in STAGE_NAMES}
                         for e in ev.get("sampled", [])},
                timeline=dict(runs=len(tl), interpretable=ok["timeline"],
                              **{k: mean([r[k] for r in tl]) for k in ("span_s", "worker_time_s", "idle_s", "startup_s", "drain_s", "hol_s", "hol_later_ready_s")},
                              scanner_cap_MBps=mean([r["scanner"]["cap_MBps"] for r in tl]),
                              idle={k: mean([r["idle"].get(k, 0.0) for r in tl]) for k in sorted({k for r in tl for k in r["idle"]})}),
                full=dict(runs=len(fu), interpretable=ok["full"],
                          **{k: mean([r.get(k) for r in fu]) for k in ("cpu_total_s", "cores_busy", "unused_core_s", "runq_wait_decode_s", "decode_elapsed_s", "span_s")},
                          cpu_by_role_s={k: mean([r["cpu_by_role_s"].get(k, 0.0) for r in fu]) for k in ("worker", "scanner", "feeder", "reader")} if fu and "cpu_by_role_s" in fu[0] else {},
                          cpu_by_phase_s={k: mean([r["cpu_by_phase_s"].get(k, 0.0) for r in fu]) for k in ("mtf", "build", "walk", "rle_crc", "worker_other", "decode_unsplit")} if fu and "cpu_by_phase_s" in fu[0] else {}),
            )
            per_w[w] = row
        out["corpora"][name] = dict(role=c["role"], by_w=per_w, rules=rules(per_w, nbytes),
                                    validity=validity(occ, occ_dir, a.cpus, stat_j, rec_j, name))
    if a.host_before and a.host_after and a.allocation:
        out["host"] = host(a.host_before, a.host_after, json.loads(Path(a.allocation).read_text()))
    Path(a.out).write_text(json.dumps(out, indent=1) + "\n")
    prim = next(c["name"] for c in corpora if c["role"] == "primary")
    print(json.dumps(dict(primary=prim, rules=out["corpora"][prim]["rules"]), indent=1))


def validity(occ, occ_dir, cpus, stat_j, rec_j, name):
    """§6: correctness (the instrument's parity), this analysis against the instrument's own
    idle shares on every timeline run (within 0.02 of worker time), and the samples'
    overhead (their wall time against the counting passes')"""
    kv = lambda r: dict(p.split("=", 1) for p in r["line"].split() if "=" in p)  # noqa: E731
    diffs = []
    for r in occ["runs"]:
        if r["mode"] != "timeline":
            continue
        f = occ_dir / "marks" / f"r{r['round']:02d}-t{r['threads']}-timeline.tsv"
        if not f.exists():
            continue
        m, k = analyse_marks(f, r["threads"], cpus), kv(r)
        if k.get("t.complete") != "yes":
            diffs.append(dict(run=f.name, problem=k.get("t.problem")))
            continue
        mine = lambda c: m["idle_by_scanner_state"].get(c, 0.0) / m["worker_time_s"]  # noqa: E731
        for c, key in (("reading", "t.w.idle.reading"), ("input", "t.w.idle.input")):
            diffs.append(dict(run=f.name, cause=c, mine=mine(c), instrument=float(k[key]), diff=abs(mine(c) - float(k[key]))))
    agree = all(d.get("diff", 1) <= 0.02 for d in diffs) and bool(diffs)
    walls = lambda xs: mean([x["wall_s"] for x in xs])  # noqa: E731
    g0 = [r for r in stat_j["runs"] if r["corpus"] == name and r["group"] in ("G0", "G0:task-clock")]
    over = {}
    for w in (1, 4):
        a_ = walls([r for r in rec_j["runs"] if r["corpus"] == name and r["w"] == w and r["event"] == "cycles"])
        b_ = walls([r for r in g0 if r["w"] == w])
        over[w] = a_ / b_ if a_ and b_ else None
    return dict(parity=occ.get("parity"), analysis_agrees_with_instrument=agree,
                worst_disagreement=max((d.get("diff", 1) for d in diffs), default=None),
                sampling_wall_over_counting=over, sampling_overhead_over_3pct=[w for w, v in over.items() if v and v > 1.03])


def host(before, after, alloc):
    """the allocated CPUs' and their idle siblings' busy share over the measurements"""
    def parse(p):
        o = {}
        for l in Path(p).read_text().splitlines():
            f = l.split()
            if f and f[0].startswith("cpu") and f[0] != "cpu":
                o[int(f[0][3:])] = [int(x) for x in f[1:]]
        return o
    b, e = parse(before), parse(after)
    res = {}
    for c in alloc["alloc"] + alloc["idle_siblings"]:
        d = [y - x for x, y in zip(b[c], e[c])]
        tot = sum(d[:8]) or 1
        res[str(c)] = dict(role="allocated" if c in alloc["alloc"] else "idle sibling", busy=(tot - d[3] - d[4]) / tot)
    sib = [v["busy"] for v in res.values() if v["role"] == "idle sibling"]
    return dict(cpus=res, siblings_idle=all(x < 0.02 for x in sib))


def rules(per_w, nbytes):
    r1, r4 = per_w.get(1), per_w.get(4)
    res, status = {}, {}
    c1, c4 = r1["counters"]["cpu_s_per_GB"], r4["counters"]["cpu_s_per_GB"]
    k1, k4 = r1["counters"]["cycles_per_byte"], r4["counters"]["cycles_per_byte"]
    f1, f4 = r1["counters"]["effective_GHz"], r4["counters"]["effective_GHz"]
    i1, i4 = r1["counters"]["instructions_per_byte"], r4["counters"]["instructions_per_byte"]
    have = lambda *x: all(v is not None for v in x)  # noqa: E731

    # R2 (H0, the clock), with the effective frequency of each workload measured: C = K / f,
    # so ln(C4/C1) = ln(K4/K1) - ln(f4/f1). The cycles part is "more execution cycles": not,
    # by itself, memory (R3 decides that).
    if have(c1, c4, k1, k4, f1, f4) and c4 != c1:
        lc = math.log(c4 / c1)
        res["R2"] = dict(cpu_ratio=c4 / c1, cycles_ratio=k4 / k1, effective_GHz=(f1, f4),
                         share_from_frequency=-math.log(f4 / f1) / lc, share_from_cycles=math.log(k4 / k1) / lc,
                         instructions_ratio=i4 / i1 if have(i1, i4) else None,
                         clock_explains_two_thirds=(k4 / k1 - 1) <= (c4 / c1 - 1) / 3)
        status["H0"] = ("supported" if res["R2"]["clock_explains_two_thirds"] else "not supported",
                        res["R2"]["share_from_frequency"], "Q1")
    else:
        status["H0"] = ("unresolved: cycles, the task clock or the clock unavailable", None, "Q1")

    # R1 (H1): the build+walk share of the added CPU-s/GB, from the cycle samples (M3), the
    # timeline's per-phase CPU (M1 full) as the cross-check
    sb = lambda r: (r["samples"].get("cycles") or {})  # noqa: E731
    s1, s4 = sb(r1), sb(r4)
    bw = lambda s: (s.get("build") or 0) + (s.get("walk") or 0) if s else None  # noqa: E731
    if have(c1, c4) and s1 and s4 and c4 != c1:
        frac = (bw(s4) * c4 - bw(s1) * c1) / (c4 - c1)
        mc = lambda x: x.get("memcpy") or 0  # noqa: E731
        upper = ((bw(s4) + mc(s4)) * c4 - (bw(s1) + mc(s1)) * c1) / (c4 - c1)
        owners = {k: ((s4.get(k) or 0) * c4 - (s1.get(k) or 0) * c1) / (c4 - c1) for k in set(s4) | set(s1)}
        # the C library's copies can't be split between the walk's assembly and RLE1's
        # output from the samples: build + walk without them (lower) and with them (upper);
        # H1 is falsified only if the upper bound is below 0.5
        res["R1"] = dict(share=frac, share_with_memcpy=upper, owners=owners, falsified=upper < 0.5,
                         straddles=frac < 0.5 <= upper)
    tr = None
    if r1["full"]["interpretable"] and r4["full"]["interpretable"] and r1["full"]["cpu_by_phase_s"] and r4["full"]["cpu_by_phase_s"]:
        gb = nbytes / 1e9
        p1, p4 = r1["full"]["cpu_by_phase_s"], r4["full"]["cpu_by_phase_s"]
        t1, t4 = r1["full"]["cpu_total_s"] / gb, r4["full"]["cpu_total_s"] / gb
        if t4 != t1:
            tr = ((p4["build"] + p4["walk"]) / gb - (p1["build"] + p1["walk"]) / gb) / (t4 - t1)
    if "R1" in res:
        res["R1"]["cross_check_trace"] = tr
        res["R1"]["resolved"] = (not res["R1"]["straddles"] and tr is not None
                                 and (tr < 0.5) == (res["R1"]["share_with_memcpy"] < 0.5))
        if not res["R1"]["resolved"]:
            status["H1"] = ("unresolved: the copies straddle 0.5, the samples and the trace disagree about it, or the trace is not interpretable", res["R1"]["share"], "Q1")
        elif res["R1"]["falsified"]:
            status["H1"] = ("falsified", res["R1"]["share"], "Q1")
    else:
        status["H1"] = ("unresolved: no cycle samples", tr, "Q1")
    # R3 (H1's signature)
    if "H1" not in status:
        st = lambda r: (r["samples"].get("cycle_activity.stalls_l3_miss") or {})  # noqa: E731
        l3 = lambda r: r["counters"]["per_byte"].get("cycle_activity.stalls_l3_miss")  # noqa: E731
        if st(r1) and st(r4) and have(l3(r1), l3(r4), k1, k4, i1, i4):
            d_stall = bw(st(r4)) * l3(r4) - bw(st(r1)) * l3(r1)
            d_cyc = bw(s4) * k4 - bw(s1) * k1
            ins_ok = abs(i4 / i1 - 1) <= 0.05
            res["R3"] = dict(bw_stall_cycles_per_byte_added=d_stall, bw_cycles_per_byte_added=d_cyc,
                             stall_share_of_added=d_stall / d_cyc if d_cyc else None, instructions_within_5pct=ins_ok,
                             in_flight=(r1["counters"]["per_byte"].get("l1d_pend_miss.pending"), r4["counters"]["per_byte"].get("l1d_pend_miss.pending")),
                             fb_full=(r1["counters"]["per_byte"].get("l1d_pend_miss.fb_full"), r4["counters"]["per_byte"].get("l1d_pend_miss.fb_full")))
            sup = d_cyc > 0 and d_stall > 0.5 * d_cyc and ins_ok
            status["H1"] = ("supported" if sup else "attribution holds (R1), memory signature not shown (R3)", res["R1"]["share"], "Q1")
        else:
            status["H1"] = ("attribution holds (R1); R3 unresolved: stall samples or counters unavailable", res["R1"]["share"], "Q1")

    # Q2 at w = 4: worker idle by window and cause, and the run-queue wait
    tl, fu = r4["timeline"], r4["full"]
    if tl["interpretable"] and tl["idle_s"]:
        idle = tl["idle"]
        tot = tl["idle_s"]
        win = lambda w_: sum(v for k, v in idle.items() if k.startswith(w_ + "."))  # noqa: E731
        cause = lambda c_: sum(v for k, v in idle.items() if k.endswith("." + c_) and k.startswith("steady."))  # noqa: E731
        X4 = r4["off"]["throughput_MBps"]
        cap = tl["scanner_cap_MBps"]
        # the steady state's only: idle while the scanner reads its first blocks is startup (H4)
        reading_idle = idle.get("steady.scanner_reading", 0.0) / tl["worker_time_s"]
        res["R4"] = dict(scanner_cap_MBps=cap, throughput_MBps=X4, cap_over_target=cap / (1.5 * X4) if cap and X4 else None,
                         worker_idle_while_scanner_reads=reading_idle,
                         binding_for_1_5x=bool(cap and X4 and cap < 1.5 * X4),
                         demoted=bool(cap and X4 and cap >= 2 * X4 and reading_idle < 0.10))
        status["H2"] = ("supported" if res["R4"]["binding_for_1_5x"] else ("not supported (demoted)" if res["R4"]["demoted"] else "bounded, not binding"),
                        reading_idle * tl["worker_time_s"] / tot, "Q2")
        res["R5"] = dict(hol_share_of_idle=tl["hol_s"] / tot, hol_with_a_later_block_ready_share=tl["hol_later_ready_s"] / tot,
                         promoted=tl["hol_s"] / tot >= 0.25)
        status["H3"] = ("supported" if res["R5"]["promoted"] else "not supported", tl["hol_s"] / tot, "Q2")
        sd = (win("startup") + win("drain")) / tot
        res["R6"] = dict(startup_drain_share_of_idle=sd, startup_s=tl["startup_s"], drain_s=tl["drain_s"], span_s=tl["span_s"], rejects_sustained=sd >= 0.5)
        status["H4"] = ("supported" if res["R6"]["rejects_sustained"] else "not supported", sd, "Q2")
        acct = {f"{w_}": win(w_) for w_ in ("startup", "drain")}
        for c_ in ("scanner_reading", "slot_head_of_line", "slot_reader_binds", "slot_other", "input_wait", "handoff", "scanner_done"):
            acct[f"steady.{c_}"] = cause(c_)
    else:
        for h in ("H2", "H3", "H4"):
            status[h] = ("unresolved: the timeline perturbs or is missing", None, "Q2")
        acct, tot = {}, None
    if fu["interpretable"] and fu.get("runq_wait_decode_s") is not None and fu.get("decode_elapsed_s"):
        rqd = fu["runq_wait_decode_s"] / fu["decode_elapsed_s"]
        rqu = fu["runq_wait_decode_s"] / fu["unused_core_s"] if fu.get("unused_core_s") else None
        res["R7"] = dict(runq_share_of_decoding=rqd, runq_share_of_unused=rqu, cores_busy=fu["cores_busy"],
                         unused_core_s=fu["unused_core_s"], promoted=rqd >= 0.10 or (rqu is not None and rqu >= 0.25))
        status["H5"] = ("supported" if res["R7"]["promoted"] else "not supported", rqu, "Q2")
        if acct:
            acct["runq_wait"] = fu["runq_wait_decode_s"] * (tl["span_s"] / fu["span_s"]) if fu.get("span_s") else fu["runq_wait_decode_s"]
    else:
        status["H5"] = ("unresolved: the full trace perturbs or is missing", None, "Q2")
    if acct:
        s = sum(acct.values())
        shares = {k: v / s for k, v in acct.items()} if s else {}
        top = max(shares.items(), key=lambda kv: kv[1]) if shares else (None, 0)
        res["Q2_account"] = dict(shares=shares, largest=top[0] if top[1] >= 0.25 else "diffuse (no owner reaches 25%)")

    # every hypothesis with its status and share; the supported ranked within each question
    ranking = {}
    for q in ("Q1", "Q2"):
        sup = [(h, s[1]) for h, s in status.items() if s[2] == q and s[0].startswith("supported")]
        ranking[q] = [h for h, _ in sorted(sup, key=lambda x: -(x[1] or 0))]
    res["hypotheses"] = {h: dict(status=s[0], share=s[1], question=s[2]) for h, s in sorted(status.items())}
    res["ranking_supported"] = ranking
    return res


def allocation(a):
    """The walk trial's allocation rule: the first `--cores` physical cores, by lowest CPU,
    not cpu0's core where enough others exist; one CPU each (its lowest); their siblings
    left idle. On the i9-10900K: CPUs 1-4, siblings 11-14."""
    cores = {}
    for d in sorted(Path("/sys/devices/system/cpu").glob("cpu[0-9]*"), key=lambda p: int(p.name[3:])):
        t = d / "topology"
        if not (t / "core_id").exists():
            continue
        key = ((t / "physical_package_id").read_text().strip(), (t / "core_id").read_text().strip())
        cores.setdefault(key, []).append(int(d.name[3:]))
    allowed = set(os.sched_getaffinity(0))
    sets = sorted((sorted(c) for c in cores.values() if set(c) <= allowed), key=lambda c: c[0])
    others = [c for c in sets if 0 not in c]
    pick = others[:a.cores] if len(others) >= a.cores else sets[:a.cores]
    print(json.dumps(dict(alloc=[c[0] for c in pick], idle_siblings=sorted(x for c in pick for x in c[1:]),
                          cores=pick, physical_cores_allowed=len(sets), includes_cpu0_core=any(0 in c for c in pick))))


def sh(*a):
    try:
        return subprocess.check_output(a, text=True, stderr=subprocess.STDOUT).strip()
    except Exception as e:  # noqa: BLE001
        return f"unavailable: {e}"


def rd(p):
    try:
        return Path(p).read_text().strip()
    except Exception as e:  # noqa: BLE001
        return f"unavailable: {e}"


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="mode", required=True)
    e = sub.add_parser("events")
    for n in ("exe", "file", "cpus", "out"):
        e.add_argument("--" + n, required=True)
    for name in ("stat", "record"):
        s = sub.add_parser(name)
        for n in ("plan", "exe", "manifest", "cpus", "workers", "out"):
            s.add_argument("--" + n, required=True)
        s.add_argument("--reps", type=int, default=5 if name == "stat" else 3)
        if name == "record":
            s.add_argument("--dir", required=True)
    m = sub.add_parser("marks")
    m.add_argument("--dir", required=True)
    m.add_argument("--w", type=int, required=True)
    m.add_argument("--cpus", type=int, default=4)
    m.add_argument("--out", required=True)
    s = sub.add_parser("summary")
    for n in ("manifest", "occupancy-dir", "out"):
        s.add_argument("--" + n, required=True)
    for n in ("stat", "record", "events", "host-before", "host-after", "allocation"):
        s.add_argument("--" + n)
    s.add_argument("--cpus", type=int, default=4)
    c = sub.add_parser("allocation")
    c.add_argument("--cores", type=int, default=4)
    a = p.parse_args()
    dict(allocation=allocation, events=events, stat=stat, record=record, marks=marks, summary=summary)[a.mode](a)
