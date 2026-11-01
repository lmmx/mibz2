"""The dispatch-causality diagnostic's analysis (docs/journal/2026-11-01-dispatch-causality-protocol.md).
It joins one run's trace marks (examples/occupancy.rs's raw dump and its sidecar) with the
kernel's scheduler, futex and idle events (`perf script --ns` of a `perf record -k
CLOCK_MONOTONIC -C <allocation>` capture), and classifies the workers' time.

    dispatch.py run     --marks M.tsv --perf P.txt --stats S.txt --workers W --cpus 1,2,3,4 --out run.json
    dispatch.py quality --runs DIR --pairs pairs.tsv
    dispatch.py summary --runs DIR --pairs pairs.tsv --out result.json
    dispatch.py probe   --perf P.txt --stats S.txt --cpus 1,2,3,4

P.txt is `perf script --show-lost-events`; S.txt is `perf report --stats` of the same data.
Amendment 1 (validation only, 2026-11-01-dispatch-causality/amendment-1.md): the kernel's
own trace loss, lines the parser cannot read, an incomplete scheduling stream (switches into
idle without switches out, a thread switched out while not known to be running) and the
scheduler checks of every run are enforced in the summary, not only before the expansion.

Nothing here is a timing comparison. Every class and threshold is the protocol's (§§5-6).
"""
import argparse
import bisect
import collections
import gzip
import json
import re
import statistics
import sys
from pathlib import Path

ALIGN_LIMIT_NS = 20_000          # §6: the clocks' alignment uncertainty must be within this
MATERIAL = 0.05                  # §5 D1: receiver obstruction, a share of aggregate worker time
REPEATABLE = 0.8                 # §5 D1: the share of runs that must each reach it
COVERAGE = 0.90                  # §6: the classified share of queued-work nondecoding time
PERTURBATION = 0.03              # §6: throughput and CPU-s/GB, instrumented against not
SCHED_COVERAGE = 0.99            # §6: scheduler state known for this share of queued-work nondecoding time
SCHED_EVENTS = ("sched:sched_switch", "sched:sched_wakeup", "sched:sched_wakeup_new", "sched:sched_waking")
ANCHOR_EVENTS = ("syscalls:sys_enter_sched_yield",)
FUTEX_EVENTS = ("syscalls:sys_enter_futex", "syscalls:sys_exit_futex")
PHASE_OF = {"DecodeStart": "decoding", "InvertStart": "decoding", "WalkStart": "decoding", "InvertEnd": "decoding",
            "DecodeEnd": "pub_lock", "PubLockGot": "publish", "Ready": "notify", "Notified": "post",
            "JobLock": "mutex_wait", "JobRecv": "recv", "RecvDone": "release", "JobGot": "buf_lock",
            "BufLockGot": "buffer", "ThreadStart": "start", "ThreadEnd": "end"}
PERF_LINE = re.compile(r"^\s*(?P<comm>.*?)\s+(?P<tid>-?\d+)\s+\[(?P<cpu>\d+)\]\s+(?P<s>\d+)\.(?P<ns>\d{9}):\s+(?P<ev>[\w:]+):\s*(?P<rest>.*)$")
KV = re.compile(r"(\w+)=(\S+)")
LOST_LINE = re.compile(r"PERF_RECORD_LOST\S*\s+lost:?\s*(\d+)")
STAT_LINE = re.compile(r"^\s*(\w+) events:\s+(\d+)")


def opener(path):
    path = str(path)
    return gzip.open(path, "rt") if path.endswith(".gz") else open(path)


def role_of(name):
    if re.fullmatch(r"mibz2-\d+", name):
        return "worker"
    return {"mibz2-scan": "scanner", "mibz2-read": "feeder"}.get(name, "reader")


def load_marks(path):
    meta = json.loads(Path(str(path).removesuffix(".gz") + ".meta.json").read_text())
    names = {t["thread"]: t["name"] for t in meta["threads"]}
    tids = {t["thread"]: t["tid"] for t in meta["threads"]}
    marks = collections.defaultdict(list)  # thread -> [(ns, kind, place, len)]
    val = lambda x: None if x == "-" else int(x)  # noqa: E731
    with opener(path) as f:
        next(f)
        for l in f:
            th, name, ns, kind, place, cpu, runq, wait, slices, ln, cap = l.rstrip("\n").split("\t")
            marks[int(th)].append((int(ns), kind, val(place), val(ln)))
    return meta, names, tids, marks


def load_perf(path):
    """the events, and what the parse saw: the kernel's loss records (`--show-lost-events`),
    events whose sample tid perf could not resolve (-1: kept, their payload still names the
    tasks), and lines it could not read at all (never silently dropped: counted)"""
    ev = []
    rep = dict(lines=0, events=0, lost_records=0, lost_events=0, unparsed=0, unparsed_examples=[],
               unattributed=collections.Counter())
    with opener(path) as f:
        for l in f:
            rep["lines"] += 1
            m = PERF_LINE.match(l)
            if not m:
                lm = LOST_LINE.search(l)
                if lm:
                    rep["lost_records"] += 1
                    rep["lost_events"] += int(lm[1])
                elif l.strip():
                    rep["unparsed"] += 1
                    if len(rep["unparsed_examples"]) < 3:
                        rep["unparsed_examples"].append(l.rstrip()[:200])
                continue
            t = int(m["s"]) * 1_000_000_000 + int(m["ns"])
            tid = int(m["tid"])
            if tid < 0:
                rep["unattributed"][m["ev"]] += 1
            ev.append((t, int(m["cpu"]), tid, m["ev"], m["rest"]))
    ev.sort(key=lambda e: e[0])
    rep["events"] = len(ev)
    rep["unattributed"] = dict(rep["unattributed"])
    return ev, rep


def load_stats(path):
    """`perf report --stats`: the loss records in all, and the samples lost and kept by event
    (perf attributes each loss record to its event, so the two must agree with the script)"""
    out = dict(available=False, lost_records=0, per_event={})
    try:
        with opener(path) as f:
            text = f.read()
    except (OSError, TypeError):
        return out
    section = None
    for l in text.splitlines():
        h = re.match(r"^(\S.*?) stats:\s*$", l)
        if h:
            section = h[1]
            out["available"] = True
            continue
        m = STAT_LINE.match(l)
        if not m or section is None:
            continue
        name, n = m[1], int(m[2])
        if section == "Aggregated":
            if name == "LOST":
                out["lost_records"] = n
        else:
            e = out["per_event"].setdefault(section, dict(samples=0, lost=0))
            if name == "SAMPLE":
                e["samples"] = n
            elif name == "LOST_SAMPLES":
                e["lost"] = n
    return out


def sched_reconstruct(perf, ours, cpus):
    """per-thread scheduler states [(t, R|Q|S|None)] and the allocated CPUs' occupants, with
    the stream's own consistency. A state the stream contradicts becomes unknown (None), so
    it is unresolved, never assigned: a switch in from sleep with no wake recorded (the
    sleep's end unknown); one of our threads switched out while not known to be running (a
    switch in missed: its whole interval since the last transition unknown); switched in
    while already running (a switch out missed: likewise). Per allocated CPU, switches into
    idle and out of it must pair (within one, for the capture's edges)."""
    sched = collections.defaultdict(list)
    occupant = {c: [] for c in cpus}
    idle = {c: dict(to=0, out=0) for c in cpus}
    last = {}
    missing_wakeups = missed_in = missed_out = 0

    def unknown(pid):
        if sched[pid]:
            sched[pid][-1] = (sched[pid][-1][0], None)

    for t, cpu, tid, ev, rest in perf:
        if ev == "sched:sched_switch":
            kv = dict(KV.findall(rest))
            prev, nxt, ps = int(kv["prev_pid"]), int(kv["next_pid"]), kv.get("prev_state", "")
            if cpu in idle:
                idle[cpu]["to"] += nxt == 0
                idle[cpu]["out"] += prev == 0
            if prev in ours and last.get(prev, "R") != "R":
                missed_in += 1
                unknown(prev)
            st = "Q" if ps.startswith("R") else "S"
            sched[prev].append((t, st))
            last[prev] = st
            if last.get(nxt) == "S":
                missing_wakeups += 1
                for k in range(len(sched[nxt]) - 1, -1, -1):
                    if sched[nxt][k][1] == "S":
                        sched[nxt][k] = (sched[nxt][k][0], None)
                        break
            elif nxt in ours and last.get(nxt) == "R":
                missed_out += 1
                unknown(nxt)
            sched[nxt].append((t, "R"))
            last[nxt] = "R"
            if cpu in occupant:
                occupant[cpu].append((t, nxt))
        elif ev in ("sched:sched_wakeup", "sched:sched_wakeup_new", "sched:sched_waking"):
            pid = int(dict(KV.findall(rest)).get("pid", -1))
            # runnable from the first of sched_waking and sched_wakeup
            if last.get(pid) not in ("R", "Q"):
                sched[pid].append((t, "Q"))
                last[pid] = "Q"
    pairing = {c: v | dict(ok=abs(v["to"] - v["out"]) <= 1) for c, v in idle.items()}
    stream = dict(missing_wakeups=missing_wakeups, missed_switch_in=missed_in, missed_switch_out=missed_out,
                  idle_pairing=pairing, idle_pairing_ok=all(v["ok"] for v in pairing.values()))
    return sched, occupant, stream


def trace_report(parse, stats, stream):
    """whether the kernel's trace can reconstruct the scheduler (ok), and what it lost.
    Scheduler and anchor events lost, lines unread, loss counts that disagree between the
    script and perf's own statistics, missing statistics, or unpaired idle switches make
    the scheduler's reconstruction unsound: not ok. Futex loss affects only the futex
    mapping (futex_ok); power:cpu_idle is recorded but feeds nothing."""
    per = stats.get("per_event", {})
    lost = {e: v["lost"] for e, v in per.items() if v.get("lost")}
    sched_lost = sum(lost.get(e, 0) for e in SCHED_EVENTS + ANCHOR_EVENTS)
    consistent = stats.get("available", False) and parse["lost_records"] == stats.get("lost_records", 0) \
        and parse["lost_events"] == sum(v.get("lost", 0) for v in per.values())
    problems = []
    if not stats.get("available"):
        problems.append("perf's statistics unavailable: loss cannot be ruled out")
    if parse["unparsed"]:
        problems.append(f"{parse['unparsed']} perf script lines unread")
    if sched_lost:
        problems.append(f"{sched_lost} scheduler or anchor events lost")
    if stats.get("available") and not consistent:
        problems.append("loss records disagree between perf script and perf's statistics")
    if not stream["idle_pairing_ok"]:
        bad = [c for c, v in stream["idle_pairing"].items() if not v["ok"]]
        problems.append(f"switches into and out of idle unpaired on CPUs {bad}: the kernel's stream is incomplete")
    futex_lost = sum(lost.get(e, 0) for e in FUTEX_EVENTS)
    return dict(ok=not problems, problems=problems, futex_ok=futex_lost == 0 and consistent, lost_by_event=lost,
                lost_records=parse["lost_records"], lost_events=parse["lost_events"], unparsed=parse["unparsed"],
                unparsed_examples=parse["unparsed_examples"], unattributed=parse["unattributed"], stream=stream)


def align(marks, names, tids, epoch, perf):
    """the anchors: each pair's second ClockAnchor against the kernel's sched_yield entry
    that follows it on the same thread; offset = kernel - (epoch + mark ns)"""
    main = [th for th, n in names.items() if role_of(n) == "reader"]
    res = dict(ok=False)
    if not main:
        return res
    th = main[0]
    anchors = [m for m in marks[th] if m[1] == "ClockAnchor"]
    yields = [e[0] for e in perf if e[3] == "syscalls:sys_enter_sched_yield" and e[2] == tids[th]]
    if len(anchors) != 4 or len(yields) < 4:
        res.update(problem=f"{len(anchors)} anchors, {len(yields)} sched_yield events on the reader's thread")
        return res
    use = [1, 3]
    offs = [yields[i] - (epoch + anchors[i][0]) for i in use]
    gaps = [yields[i] - anchors[i][3] for i in use]
    corr = sum(offs) / 2
    u = abs(offs[0] - offs[1]) / 2 + max(0, max(gaps))
    res.update(ok=u <= ALIGN_LIMIT_NS and min(gaps) >= 0, offsets_ns=offs, read_to_syscall_ns=gaps, correction_ns=corr, uncertainty_ns=u)
    return res


class Step:
    """a step function of time: values after each change point (pointer-advanced queries)"""

    def __init__(self, changes, initial=None):
        changes.sort(key=lambda c: c[0])
        self.t = [c[0] for c in changes]
        self.v = [c[1] for c in changes]
        self.initial = initial

    def at(self, t):
        i = bisect.bisect_right(self.t, t) - 1
        return self.initial if i < 0 else self.v[i]


def run(a):
    meta, names, tids, marks = load_marks(a.marks)
    perf, parse = load_perf(a.perf)
    stats = load_stats(a.stats)
    cpus = [int(c) for c in a.cpus.split(",")]
    epoch = meta["epoch_monotonic_ns"]
    al = align(marks, names, tids, epoch, perf)
    corr = al.get("correction_ns", 0.0)
    u = al.get("uncertainty_ns", 0.0)
    T = lambda th, ns: epoch + ns + corr  # noqa: E731
    role = {th: role_of(n) for th, n in names.items()}
    workers = sorted(th for th in names if role[th] == "worker")
    tid_role = {tids[th]: role[th] for th in names}
    tid_of = {th: tids[th] for th in names}

    # the trace's own completeness, and the TIDs the kernel saw
    seen_tids = {e[2] for e in perf} | {int(x) for e in perf if e[3] == "sched:sched_switch" for x in re.findall(r"(?:prev|next)_pid=(\d+)", e[4])}
    complete = all(marks[th][0][1] == "ThreadStart" and marks[th][-1][1] == "ThreadEnd" for th in names) and all(t["dropped"] == 0 for t in meta["threads"])

    # scheduler states per thread: R running, Q runnable (woken, or preempted), S sleeping,
    # None unknown (the stream contradicts itself there)
    sched, occupant, stream = sched_reconstruct(perf, set(tid_of.values()), cpus)
    missing_wakeups = stream["missing_wakeups"]
    trace = trace_report(parse, stats, stream)
    st_of = {th: Step(list(sched.get(tid_of[th], []))) for th in names}
    occ = {c: Step(list(v)) for c, v in occupant.items()}

    def occupant_role(pid):
        if pid is None:
            return "unknown"
        if pid == 0:
            return "idle"
        return tid_role.get(pid, "foreign")

    # the workers' phases (from their marks), the receiver's owner, and job availability
    phase = {}
    owner_iv = []  # (start, end, worker)
    recvdone_before_got = {}
    for th in workers:
        ms = marks[th]
        phase[th] = Step([(T(th, m[0]), PHASE_OF[m[1]]) for m in ms if m[1] in PHASE_OF], "start")
        last_recv = None
        for m in ms:
            if m[1] == "JobRecv":
                last_recv = T(th, m[0])
            elif m[1] == "RecvDone":
                rd = T(th, m[0])
            elif m[1] == "JobGot" and last_recv is not None:
                owner_iv.append((last_recv, T(th, m[0]), th))
                recvdone_before_got[m[2]] = rd
                last_recv = None
    owner_iv.sort()
    scan = [th for th in names if role[th] == "scanner"][0]
    issued = {m[2]: T(scan, m[0]) for m in marks[scan] if m[1] == "Issued"}
    enq = {m[2]: T(scan, m[0]) for m in marks[scan] if m[1] == "Enqueued"}
    avail_d, bracket_d = [], []
    for p, e in enq.items():
        r = recvdone_before_got.get(p)
        if r is None:
            continue
        if r > e:
            avail_d += [(e, 1), (r, -1)]
        bracket_d += [(issued.get(p, e), 1), (min(e, r), -1)]

    def counter(d):
        d.sort()
        out, c = [], 0
        for t, x in d:
            c += x
            out.append((t, c))
        return Step(out, 0)
    avail = counter(avail_d)
    bracket = counter(bracket_d)

    # the span (as the attribution diagnostic's): first mark to the last Taken or DecodeEnd
    allm = [T(th, m[0]) for th in names for m in marks[th]]
    ends = [T(th, m[0]) for th in names for m in marks[th] if m[1] in ("Taken", "DecodeEnd")]
    span0, span1 = min(allm), max(ends)
    W = len(workers) * (span1 - span0)

    # breakpoints: every change of any input, plus each worker mark's +-u guard
    bp = {span0, span1}
    for th in workers:
        bp.update(phase[th].t)
        bp.update(st_of[th].t)
        for m in marks[th]:
            if m[1] in PHASE_OF:
                x = T(th, m[0])
                bp.update((x - u, x + u))
    for s in occ.values():
        bp.update(s.t)
    bp.update(avail.t)
    bp.update(bracket.t)
    bp = sorted(x for x in bp if span0 <= x <= span1)
    mark_times = {th: sorted(T(th, m[0]) for m in marks[th] if m[1] in PHASE_OF) for th in workers}
    avail_times = sorted(avail.t + bracket.t)

    cls = collections.Counter()
    occ_runnable = collections.Counter()
    backlog_job_ns = 0.0
    no_job = collections.Counter()
    for a_, b_ in zip(bp, bp[1:]):
        dt = b_ - a_
        if dt <= 0:
            continue
        m = (a_ + b_) / 2
        av = avail.at(m)
        idle_cpus = sum(1 for c in cpus if occupant_role(occ[c].at(m)) == "idle")
        nondec = [th for th in workers if phase[th].at(m) != "decoding"]
        if av > 0 and not nondec:
            backlog_job_ns += av * dt
        for th in nondec:
            ph = phase[th].at(m)
            st = st_of[th].at(m)
            if av <= 0:
                key = "unresolved.send_bracket" if bracket.at(m) > 0 else f"no_job.{st or 'unknown'}"
                (cls if key.startswith("unresolved") else no_job)[key] += dt
                continue
            # within the clocks' uncertainty of a boundary the class is not decided
            mt = mark_times[th]
            i = bisect.bisect_left(mt, m)
            near = min([abs(m - mt[j]) for j in (i - 1, i) if 0 <= j < len(mt)] + [float("inf")])
            j = bisect.bisect_left(avail_times, m)
            near_av = min([abs(m - avail_times[k]) for k in (j - 1, j) if 0 <= k < len(avail_times)] + [float("inf")])
            if near < u or near_av < u:
                cls["unresolved.clock"] += dt
                continue
            if st is None:
                cls["unresolved.sched"] += dt
                continue
            cpu = "cpu_idle" if idle_cpus > 0 else "cpus_busy"
            if st == "Q":
                cls[f"runnable.{ph}.{cpu}"] += dt
                for c in cpus:
                    occ_runnable[occupant_role(occ[c].at(m))] += dt
                continue
            owner = next((w for s, e, w in owner_iv if s <= m < e and w != th), None) if ph == "mutex_wait" else None
            if ph == "mutex_wait":
                key = ("rx.blocked_by_owner" if st == "S" else "rx.running_owner_held") if owner is not None else ("rx.handoff_asleep" if st == "S" else "rx.acquiring")
                cls[f"{key}.{cpu}"] += dt
            elif ph == "recv":
                cls["recv.asleep" if st == "S" else "recv.running"] += dt
            elif ph in ("pub_lock", "buf_lock"):
                cls["state_lock.blocked" if st == "S" else "state_lock.running"] += dt
            else:
                cls[f"{ph}.{'blocked' if st == 'S' else 'running'}"] += dt
    Tq = sum(cls.values())
    unresolved = sum(v for k, v in cls.items() if k.startswith("unresolved"))
    R = sum(v for k, v in cls.items() if k.startswith(("rx.blocked_by_owner", "rx.handoff_asleep", "rx.running_owner_held")))
    R_idle = sum(v for k, v in cls.items() if k.startswith(("rx.blocked_by_owner", "rx.handoff_asleep", "rx.running_owner_held")) and k.endswith("cpu_idle"))
    sched_cover = 1 - cls.get("unresolved.sched", 0) / Tq if Tq else None

    # the reader: from a block's publication to its taking, when the reader waited for it
    rd = [th for th in names if role[th] == "reader"][0]
    rmarks = marks[rd]
    waiting = Step([(T(rd, m[0]), m[1] == "CvWait") for m in rmarks if m[1] in ("CvWait", "CvWoke")], False)
    woke_t = sorted(T(rd, m[0]) for m in rmarks if m[1] == "CvWoke")
    taken = {m[2]: T(rd, m[0]) for m in rmarks if m[1] == "Taken"}
    taken_t = sorted(taken.values())
    ready = {}
    notified = {}
    for th in workers:
        for m in marks[th]:
            if m[1] == "Ready":
                ready[m[2]] = T(th, m[0])
            elif m[1] == "Notified":
                notified[m[2]] = T(th, m[0])
    scan_waiting = Step([(T(scan, m[0]), m[1] == "CvWait") for m in marks[scan] if m[1] in ("CvWait", "CvWoke")], False)
    rtid = tid_of[rd]
    rwake = sorted(t for t, s in sched.get(rtid, []) if s == "Q")
    rrun = sorted(t for t, s in sched.get(rtid, []) if s == "R")
    segs = collections.defaultdict(list)
    for p, t0 in ready.items():
        # only the block the reader is waiting for: the next in order, with it waiting
        if p not in taken or not waiting.at(t0) or p != bisect.bisect_left(taken_t, t0):
            continue
        t5 = taken[p]
        def nxt(lst, t):
            i = bisect.bisect_left(lst, t)
            return lst[i] if i < len(lst) else None
        t2 = nxt(rwake, t0)
        t3 = nxt(rrun, t2) if t2 is not None else None
        t4 = nxt(woke_t, t3) if t3 is not None else None
        if None in (t2, t3, t4) or not (t0 <= t2 <= t3 <= t4 <= t5):
            segs["unresolved"].append(t5 - t0)
            continue
        segs["publish_to_wakeup"].append(t2 - t0)
        segs["wakeup_to_running"].append(t3 - t2)
        segs["running_to_relocked"].append(t4 - t3)
        segs["relocked_to_taken"].append(t5 - t4)
        segs["total"].append(t5 - t0)
    reader = {k: dict(n=len(v), sum_ms=sum(v) / 1e6, median_us=statistics.median(v) / 1e3) for k, v in segs.items() if v}

    # futex addresses by the phase they occur in (waits only: FUTEX_WAIT, FUTEX_WAIT_BITSET)
    fx = collections.defaultdict(collections.Counter)
    for t, cpu, tid, ev, rest in perf:
        if ev != "syscalls:sys_enter_futex" or tid not in tid_role:
            continue
        mm = re.search(r"uaddr: (0x[0-9a-f]+), op: (0x[0-9a-f]+)", rest)
        if not mm or int(mm[2], 16) & 0x7F not in (0, 9):
            continue
        th = next(k for k, v in tid_of.items() if v == tid)
        if role[th] == "worker":
            ph = phase[th].at(t)
            where = {"mutex_wait": "receiver_mutex", "recv": "channel", "pub_lock": "state_lock", "buf_lock": "state_lock"}.get(ph, ph)
        else:
            w_ = waiting if role[th] == "reader" else scan_waiting if role[th] == "scanner" else None
            where = f"{role[th]}.{'condvar' if (w_ is not None and w_.at(t)) else 'other'}"
        fx[where][mm[1]] += 1
    futex = {k: dict(waits=sum(c.values()), addresses=len(c), dominant=c.most_common(1)[0][0], dominant_share=c.most_common(1)[0][1] / sum(c.values())) for k, c in fx.items()}

    out = dict(marks=str(a.marks), workers=len(workers), cpus=cpus, alignment=al, complete=complete,
               threads_in_sched=all(tid_of[th] in seen_tids for th in names), missing_wakeups=missing_wakeups,
               span_s=(span1 - span0) / 1e9, worker_time_s=W / 1e9, queued_nondecoding_s=Tq / 1e9,
               classes_s={k: v / 1e9 for k, v in sorted(cls.items())}, no_job_s={k: v / 1e9 for k, v in sorted(no_job.items())},
               backlog_all_decoding_job_s=backlog_job_ns / 1e9,
               runnable_cpu_occupants_s={k: v / 1e9 for k, v in occ_runnable.items()},
               coverage=(1 - unresolved / Tq) if Tq else None, sched_coverage=sched_cover,
               receiver_obstruction_share=R / W if W else None, receiver_obstruction_cpu_idle_share=R_idle / W if W else None,
               reader=reader, futex=futex, trace=trace)
    Path(a.out).write_text(json.dumps(out, indent=1) + "\n")
    print(json.dumps({k: out[k] for k in ("complete", "coverage", "sched_coverage", "receiver_obstruction_share", "queued_nondecoding_s", "worker_time_s")}
                     | dict(alignment_us=al.get("uncertainty_ns", 0) / 1e3, trace_ok=trace["ok"], trace_problems=trace["problems"])))


def run_checks(r):
    """one run's data-quality checks (§6, with amendment 1's trace completeness): the same
    for the expansion and for the final decision, never on the findings"""
    return dict(complete=bool(r["complete"]), alignment=bool(r["alignment"].get("ok", False)),
                threads_in_sched=bool(r["threads_in_sched"]), sched_coverage=(r["sched_coverage"] or 0) >= SCHED_COVERAGE,
                trace_complete=bool(r.get("trace", {}).get("ok", False)), coverage=(r["coverage"] or 0) >= COVERAGE)


def quality(r, pert):
    """§6's expansion conditions, on data quality alone (never on the findings)"""
    c = run_checks(r)
    return dict(complete=c["complete"], alignment=c["alignment"], threads_in_sched=c["threads_in_sched"],
                sched_coverage=c["sched_coverage"], trace_complete=c["trace_complete"], perturbation=pert)


def run_files(d, w):
    return sorted(Path(d).glob(f"w{w}-r[0-9][0-9].json"))


def receiver_associated(r):
    """exploratory, never a pass criterion: the registered receiver obstruction plus the
    time runnable in the receiver mutex's wait phase (woken from it, or preempted in it,
    not yet running), as a share of aggregate worker time"""
    w = r["worker_time_s"]
    extra = sum(v for k, v in r["classes_s"].items() if k.startswith("runnable.mutex_wait."))
    return (r["receiver_obstruction_share"] or 0) + extra / w if w else None


def summary(a):
    files = run_files(a.runs, 4)
    runs = [json.loads(p.read_text()) for p in files]
    ref = [json.loads(p.read_text()) for p in run_files(a.runs, 1)]
    pairs = [l.split("\t") for l in Path(a.pairs).read_text().splitlines() if l and not l.startswith("#")]
    # pairs: w, rep, mode(off|instr), wall, cpu, out, rss_kb, tier
    def ratio(w, key):
        x = [p for p in pairs if p[0] == str(w)]
        off = [p for p in x if p[2] == "off"]
        ins = [p for p in x if p[2] == "instr"]
        f = {"throughput": lambda p: float(p[5]) / float(p[3]), "cpu_per_byte": lambda p: float(p[4]) / float(p[5]), "rss": lambda p: float(p[6])}[key]
        return statistics.fmean(map(f, ins)) / statistics.fmean(map(f, off)) if off and ins else None
    pert = {w: {k: ratio(w, k) for k in ("throughput", "cpu_per_byte", "rss")} for w in (1, 4)}
    pert_ok = all(v is not None and abs(v - 1) <= PERTURBATION for w in (1, 4) for k, v in pert[w].items() if k != "rss")
    shares = [r["receiver_obstruction_share"] for r in runs]
    mean_R = statistics.fmean(shares) if shares else None
    reps = sum(1 for s in shares if s >= MATERIAL) / len(shares) if shares else 0
    classes = collections.Counter()
    for r in runs:
        for k, v in r["classes_s"].items():
            classes[k] += v
    tq = sum(classes.values())
    groups = collections.Counter()
    for k, v in classes.items():
        g = k.split(".")[0] if not k.startswith("rx.") else ("receiver" if k.split(".")[1] in ("blocked_by_owner", "handoff_asleep", "running_owner_held") else "rx_other")
        groups[g] += v
    ranked = sorted(((g, v / tq) for g, v in groups.items()), key=lambda x: -x[1]) if tq else []
    # every w = 4 run in the decision, the expansion's included, meets every check (amendment 1)
    checks = {p.stem: run_checks(r) for p, r in zip(files, runs)}
    valid = dict(runs_present=bool(runs), coverage_all=all(c["coverage"] for c in checks.values()), perturbation=pert_ok,
                 alignment_all=all(r["alignment"].get("ok") for r in runs + ref), complete_all=all(r["complete"] for r in runs + ref),
                 threads_in_sched_all=all(c["threads_in_sched"] for c in checks.values()),
                 sched_coverage_all=all(c["sched_coverage"] for c in checks.values()),
                 trace_complete_all=all(c["trace_complete"] for c in checks.values()))
    interpreted = all(valid.values())
    assoc = [receiver_associated(r) for r in runs]
    decision = ("promote an MPMC experiment" if mean_R is not None and mean_R >= MATERIAL and reps >= REPEATABLE
                else "do not proceed to MPMC: receiver obstruction below the materiality threshold, or not repeatable")
    out = dict(protocol="docs/journal/2026-11-01-dispatch-causality-protocol.md", runs_w4=len(runs), runs_w1=len(ref),
               perturbation=pert, validity=valid, receiver_obstruction=dict(mean_share=mean_R, per_run=shares, runs_at_or_above_5pct=reps,
               cpu_idle_share_mean=statistics.fmean([r["receiver_obstruction_cpu_idle_share"] for r in runs]) if runs else None),
               queued_nondecoding_ranked=ranked, classes_s=dict(classes),
               decision=decision if interpreted else f"not interpreted (validity: {[k for k, v in valid.items() if not v]}); provisional: {decision}",
               interpreted=interpreted,
               outcome=("promote" if decision.startswith("promote") else "do not proceed") if interpreted else "invalid evidence",
               run_checks=checks, trace_problems={p.stem: r.get("trace", {}).get("problems", ["no trace report"]) for p, r in zip(files, runs)},
               receiver_associated_exploratory=dict(note="not a pass criterion: the registered obstruction plus runnable time in the receiver mutex's wait phase",
                                                    mean_share=statistics.fmean(assoc) if assoc and None not in assoc else None, per_run=assoc),
               w1_reference=[dict(receiver_obstruction_share=r["receiver_obstruction_share"], coverage=r["coverage"], checks=run_checks(r)) for r in ref],
               reader=[r["reader"] for r in runs], futex=[r["futex"] for r in runs])
    Path(a.out).write_text(json.dumps(out, indent=1) + "\n")
    print(json.dumps({k: out[k] for k in ("decision", "outcome", "validity", "perturbation", "receiver_obstruction", "receiver_associated_exploratory", "queued_nondecoding_ranked")}, indent=1))


def quality_cmd(a):
    """the expansion decision after the first three repetitions (§6), printed for the runner"""
    runs = [json.loads(p.read_text()) for p in run_files(a.runs, 4)]
    pairs = [l.split("\t") for l in Path(a.pairs).read_text().splitlines() if l and not l.startswith("#")]
    x = [p for p in pairs if p[0] == "4"]
    f = lambda m, g: statistics.fmean([g(p) for p in x if p[2] == m])  # noqa: E731
    thr = f("instr", lambda p: float(p[5]) / float(p[3])) / f("off", lambda p: float(p[5]) / float(p[3]))
    cpu = f("instr", lambda p: float(p[4]) / float(p[5])) / f("off", lambda p: float(p[4]) / float(p[5]))
    pert = abs(thr - 1) <= PERTURBATION and abs(cpu - 1) <= PERTURBATION
    q = [quality(r, pert) for r in runs]
    ok = bool(q) and all(all(v.values()) for v in q)
    print(json.dumps(dict(expand=ok, perturbation=dict(throughput=thr, cpu_per_byte=cpu, ok=pert), runs=q)))
    sys.exit(0 if ok else 3)


def probe(a):
    """before any capture: does this kernel's stream let the scheduler be reconstructed on
    the allocated CPUs (switches into and out of idle paired, nothing lost)? Exit 0, else 4."""
    perf, parse = load_perf(a.perf)
    cpus = [int(c) for c in a.cpus.split(",")]
    _, _, stream = sched_reconstruct(perf, set(), cpus)
    t = trace_report(parse, load_stats(a.stats), stream)
    print(json.dumps(dict(ok=t["ok"], problems=t["problems"], idle_pairing=stream["idle_pairing"], lost_by_event=t["lost_by_event"])))
    sys.exit(0 if t["ok"] else 4)


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    s = p.add_subparsers(dest="mode", required=True)
    r = s.add_parser("run")
    for n in ("marks", "perf", "stats", "cpus", "out"):
        r.add_argument("--" + n, required=True)
    r.add_argument("--workers", type=int, required=True)
    pr = s.add_parser("probe")
    for n in ("perf", "stats", "cpus"):
        pr.add_argument("--" + n, required=True)
    for n in ("summary", "quality"):
        q = s.add_parser(n)
        q.add_argument("--runs", required=True)
        q.add_argument("--pairs", required=True)
        if n == "summary":
            q.add_argument("--out", required=True)
    a = p.parse_args()
    dict(run=run, summary=summary, quality=quality_cmd, probe=probe)[a.mode](a)
