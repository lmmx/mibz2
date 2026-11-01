"""The dispatch-causality diagnostic's analysis (docs/journal/2026-11-01-dispatch-causality-protocol.md).
It joins one run's trace marks (examples/occupancy.rs's raw dump and its sidecar) with the
kernel's scheduler, futex and idle events (`perf script --ns` of a `perf record -k
CLOCK_MONOTONIC -C <allocation>` capture), and classifies the workers' time.

    dispatch.py run     --marks M.tsv --perf P.txt --workers W --cpus 1,2,3,4 --out run.json
    dispatch.py summary --runs DIR --pairs pairs.tsv --out result.json

Nothing here is a timing comparison. Every class and threshold is the protocol's (§§5-6).
"""
import argparse
import bisect
import collections
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
PHASE_OF = {"DecodeStart": "decoding", "InvertStart": "decoding", "WalkStart": "decoding", "InvertEnd": "decoding",
            "DecodeEnd": "pub_lock", "PubLockGot": "publish", "Ready": "notify", "Notified": "post",
            "JobLock": "mutex_wait", "JobRecv": "recv", "RecvDone": "release", "JobGot": "buf_lock",
            "BufLockGot": "buffer", "ThreadStart": "start", "ThreadEnd": "end"}
PERF_LINE = re.compile(r"^\s*(?P<comm>.*?)\s+(?P<tid>\d+)\s+\[(?P<cpu>\d+)\]\s+(?P<s>\d+)\.(?P<ns>\d{9}):\s+(?P<ev>[\w:]+):\s*(?P<rest>.*)$")
KV = re.compile(r"(\w+)=(\S+)")


def role_of(name):
    if re.fullmatch(r"mibz2-\d+", name):
        return "worker"
    return {"mibz2-scan": "scanner", "mibz2-read": "feeder"}.get(name, "reader")


def load_marks(path):
    meta = json.loads(Path(str(path) + ".meta.json").read_text())
    names = {t["thread"]: t["name"] for t in meta["threads"]}
    tids = {t["thread"]: t["tid"] for t in meta["threads"]}
    marks = collections.defaultdict(list)  # thread -> [(ns, kind, place, len)]
    val = lambda x: None if x == "-" else int(x)  # noqa: E731
    with open(path) as f:
        next(f)
        for l in f:
            th, name, ns, kind, place, cpu, runq, wait, slices, ln, cap = l.rstrip("\n").split("\t")
            marks[int(th)].append((int(ns), kind, val(place), val(ln)))
    return meta, names, tids, marks


def load_perf(path):
    ev = []
    with open(path) as f:
        for l in f:
            m = PERF_LINE.match(l)
            if not m:
                continue
            t = int(m["s"]) * 1_000_000_000 + int(m["ns"])
            ev.append((t, int(m["cpu"]), int(m["tid"]), m["ev"], m["rest"]))
    ev.sort(key=lambda e: e[0])
    return ev


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
    perf = load_perf(a.perf)
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

    # scheduler states per thread: R running, Q runnable (woken, or preempted), S sleeping
    sched = collections.defaultdict(list)
    occupant = {c: [] for c in cpus}
    missing_wakeups = 0
    last_state = {}
    for t, cpu, tid, ev, rest in perf:
        if ev == "sched:sched_switch":
            kv = dict(KV.findall(rest))
            prev, nxt, ps = int(kv["prev_pid"]), int(kv["next_pid"]), kv.get("prev_state", "")
            st = "Q" if ps.startswith("R") else "S"
            sched[prev].append((t, st))
            last_state[prev] = st
            if last_state.get(nxt) == "S":
                # switched in from sleep with no wake recorded: when it became runnable is
                # unknown, so that whole sleep is unresolved rather than counted as asleep
                missing_wakeups += 1
                for k in range(len(sched[nxt]) - 1, -1, -1):
                    if sched[nxt][k][1] == "S":
                        sched[nxt][k] = (sched[nxt][k][0], None)
                        break
            sched[nxt].append((t, "R"))
            last_state[nxt] = "R"
            if cpu in occupant:
                occupant[cpu].append((t, nxt))
        elif ev in ("sched:sched_wakeup", "sched:sched_wakeup_new", "sched:sched_waking"):
            pid = int(dict(KV.findall(rest)).get("pid", -1))
            # runnable from the first of sched_waking and sched_wakeup
            if last_state.get(pid) not in ("R", "Q"):
                sched[pid].append((t, "Q"))
                last_state[pid] = "Q"
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
               reader=reader, futex=futex)
    Path(a.out).write_text(json.dumps(out, indent=1) + "\n")
    print(json.dumps({k: out[k] for k in ("complete", "coverage", "sched_coverage", "receiver_obstruction_share", "queued_nondecoding_s", "worker_time_s")} | dict(alignment_us=al.get("uncertainty_ns", 0) / 1e3)))


def quality(r, pert):
    """§6's expansion conditions, on data quality alone (never on the findings)"""
    return dict(complete=r["complete"], alignment=r["alignment"].get("ok", False), threads_in_sched=r["threads_in_sched"],
                sched_coverage=(r["sched_coverage"] or 0) >= 0.99, perturbation=pert)


def summary(a):
    runs = [json.loads(p.read_text()) for p in sorted(Path(a.runs).glob("w4-r[0-9][0-9].json"))]
    ref = [json.loads(p.read_text()) for p in sorted(Path(a.runs).glob("w1-r[0-9][0-9].json"))]
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
    valid = dict(coverage_all=all((r["coverage"] or 0) >= COVERAGE for r in runs), perturbation=pert_ok,
                 alignment_all=all(r["alignment"].get("ok") for r in runs + ref), complete_all=all(r["complete"] for r in runs + ref))
    decision = ("promote an MPMC experiment" if mean_R is not None and mean_R >= MATERIAL and reps >= REPEATABLE
                else "do not proceed to MPMC: receiver obstruction below the materiality threshold, or not repeatable")
    out = dict(protocol="docs/journal/2026-11-01-dispatch-causality-protocol.md", runs_w4=len(runs), runs_w1=len(ref),
               perturbation=pert, validity=valid, receiver_obstruction=dict(mean_share=mean_R, per_run=shares, runs_at_or_above_5pct=reps,
               cpu_idle_share_mean=statistics.fmean([r["receiver_obstruction_cpu_idle_share"] for r in runs]) if runs else None),
               queued_nondecoding_ranked=ranked, classes_s=dict(classes),
               decision=decision if all(valid.values()) else f"not interpreted (validity: {[k for k, v in valid.items() if not v]}); provisional: {decision}",
               w1_reference=[dict(receiver_obstruction_share=r["receiver_obstruction_share"], coverage=r["coverage"]) for r in ref],
               reader=[r["reader"] for r in runs], futex=[r["futex"] for r in runs])
    Path(a.out).write_text(json.dumps(out, indent=1) + "\n")
    print(json.dumps({k: out[k] for k in ("decision", "validity", "perturbation", "receiver_obstruction", "queued_nondecoding_ranked")}, indent=1))


def quality_cmd(a):
    """the expansion decision after the first three repetitions (§6), printed for the runner"""
    runs = [json.loads(p.read_text()) for p in sorted(Path(a.runs).glob("w4-r[0-9][0-9].json"))]
    pairs = [l.split("\t") for l in Path(a.pairs).read_text().splitlines() if l and not l.startswith("#")]
    x = [p for p in pairs if p[0] == "4"]
    f = lambda m, g: statistics.fmean([g(p) for p in x if p[2] == m])  # noqa: E731
    thr = f("instr", lambda p: float(p[5]) / float(p[3])) / f("off", lambda p: float(p[5]) / float(p[3]))
    cpu = f("instr", lambda p: float(p[4]) / float(p[5])) / f("off", lambda p: float(p[4]) / float(p[5]))
    pert = abs(thr - 1) <= PERTURBATION and abs(cpu - 1) <= PERTURBATION
    q = [quality(r, pert) for r in runs]
    ok = all(all(v.values()) for v in q)
    print(json.dumps(dict(expand=ok, perturbation=dict(throughput=thr, cpu_per_byte=cpu, ok=pert), runs=q)))
    sys.exit(0 if ok else 3)


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    s = p.add_subparsers(dest="mode", required=True)
    r = s.add_parser("run")
    for n in ("marks", "perf", "cpus", "out"):
        r.add_argument("--" + n, required=True)
    r.add_argument("--workers", type=int, required=True)
    for n in ("summary", "quality"):
        q = s.add_parser(n)
        q.add_argument("--runs", required=True)
        q.add_argument("--pairs", required=True)
        if n == "summary":
            q.add_argument("--out", required=True)
    a = p.parse_args()
    dict(run=run, summary=summary, quality=quality_cmd)[a.mode](a)
