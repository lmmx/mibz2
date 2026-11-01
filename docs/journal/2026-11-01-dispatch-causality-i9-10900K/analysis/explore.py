"""Descriptive (not registered) analysis of the i9 dispatch-causality capture: wake-to-run
latency by how the CPU was found (idle and its C-state, or busy), and what the scanner and
the admission window were doing while workers had no job. Reads the frozen analysis's own
loaders and reconstruction (bench/dispatch.py at the pin).

    python3 -I explore.py REPO CAPTURE_DIR
"""
import bisect
import collections
import json
import re
import statistics
import sys
from pathlib import Path

repo, cap = Path(sys.argv[1]), Path(sys.argv[2])
sys.path.insert(0, str(repo / "bench"))
import dispatch as d  # noqa: E402

CPUS = [1, 2, 3, 4]
lat = collections.defaultdict(list)          # (phase, how) -> [ns]
cstate_of_exit = collections.Counter()       # C-state index the CPU left, for wakes onto an idle CPU
lat_by_c = collections.defaultdict(list)
nojob_scanner = collections.Counter()        # what the scanner was doing while a worker had no job
nojob_total = 0
window_full_ns = span_ns_all = 0
scanner_state = collections.Counter()
reader_lat = []

for f in sorted(cap.glob("w4-r??.tsv.gz")):
    b = f.name[: -len(".tsv.gz")]
    meta, names, tids, marks = d.load_marks(f)
    perf, parse = d.load_perf(cap / f"{b}.perf.txt.gz")
    epoch = meta["epoch_monotonic_ns"]
    al = d.align(marks, names, tids, epoch, perf)
    corr = al["correction_ns"]
    T = lambda ns: epoch + ns + corr  # noqa: E731
    role = {th: d.role_of(n) for th, n in names.items()}
    tid_of = {th: tids[th] for th in names}
    ours = set(tid_of.values())
    sched, occupant, stream = d.sched_reconstruct(perf, ours, CPUS)
    occ = {c: d.Step(list(v)) for c, v in occupant.items()}
    workers = sorted(th for th in names if role[th] == "worker")
    scan = next(th for th in names if role[th] == "scanner")
    rd = next(th for th in names if role[th] == "reader")

    # each CPU's idle state: power:cpu_idle state=N on entry, 4294967295 on exit
    cst = {c: [] for c in CPUS}
    for t, cpu, tid, ev, rest in perf:
        if ev == "power:cpu_idle" and cpu in cst:
            m = re.search(r"state=(\d+)", rest)
            if m:
                cst[cpu].append((t, int(m[1])))
    cstep = {c: d.Step(v) for c, v in cst.items()}

    # every wake -> run interval of a worker or the reader: the CPU it ran on, and how it was found
    switch_in = collections.defaultdict(list)  # tid -> [(t, cpu, prev_pid)]
    for t, cpu, tid, ev, rest in perf:
        if ev == "sched:sched_switch":
            kv = dict(d.KV.findall(rest))
            switch_in[int(kv["next_pid"])].append((t, cpu, int(kv["prev_pid"])))
    for th in workers + [rd]:
        tid = tid_of[th]
        ph = d.Step([(T(m[0]), d.PHASE_OF[m[1]]) for m in marks[th] if m[1] in d.PHASE_OF], "start") if th != rd else None
        ins = switch_in[tid]
        ins_t = [x[0] for x in ins]
        seq = sched.get(tid, [])
        for (t0, s0), (t1, s1) in zip(seq, seq[1:]):
            if s0 != "Q" or s1 != "R":
                continue
            i = bisect.bisect_left(ins_t, t1)
            if i >= len(ins) or ins[i][0] != t1:
                continue
            _, cpu, prev = ins[i]
            how = "from idle" if prev == 0 else ("preempted another worker" if prev in {tid_of[w] for w in workers} else "behind another thread")
            phase = ph.at(t0) if ph else "reader"
            lat[(phase, how)].append(t1 - t0)
            if prev == 0 and cpu in cstep:
                # the state entered last before the switch in (the exit marker is 4294967295)
                last = [s for tt, s in cst[cpu] if tt < t1 and s != 4294967295]
                cs = last[-1] if last else None
                cstate_of_exit[cs] += 1
                lat_by_c[cs].append(t1 - t0)

    # the scanner and the window, over the run's span
    sm = marks[scan]
    issued = sorted(T(m[0]) for m in sm if m[1] == "Issued")
    taken = sorted(T(m[0]) for m in marks[rd] if m[1] == "Taken")
    slotwait = []
    open_ = None
    for m in sm:
        if m[1] == "SlotWait":
            open_ = T(m[0])
        elif m[1] == "SlotGot" and open_ is not None:
            slotwait.append((open_, T(m[0])))
            open_ = None
    sw = d.Step(sorted([(a, 1) for a, _ in slotwait] + [(b, 0) for _, b in slotwait]), 0)
    phase_scan = d.Step(sorted([(T(m[0]), {"SlotWait": "slot wait", "SlotGot": "scanning", "Issued": "sending", "Enqueued": "scanning"}[m[1]])
                                for m in sm if m[1] in ("SlotWait", "SlotGot", "Issued", "Enqueued")]), "start")
    s_sched = d.Step(list(sched.get(tid_of[scan], [])))
    allm = [T(m[0]) for th in names for m in marks[th]]
    t_start, t_end = min(allm), max(T(m[0]) for m in marks[rd] if m[1] == "Taken")
    # sample every 2 µs: worker nondecoding with no job available; the scanner's phase and state
    enq = sorted((T(m[0]), m[2]) for m in sm if m[1] == "Enqueued")
    got = {}
    for th in workers:
        for m in marks[th]:
            if m[1] == "RecvDone":
                last_rd = T(m[0])
            if m[1] == "JobGot":
                got[m[2]] = last_rd
    deltas = sorted([(t, 1) for t, p in enq if p in got and got[p] > t] + [(got[p], -1) for t, p in enq if p in got and got[p] > t])
    acc, ch = 0, []
    for t, x in deltas:
        acc += x
        ch.append((t, acc))
    avail = d.Step(ch, 0)
    phases = {th: d.Step([(T(m[0]), d.PHASE_OF[m[1]]) for m in marks[th] if m[1] in d.PHASE_OF], "start") for th in workers}
    step = 2_000
    t = t_start
    while t < t_end:
        span_ns_all += step
        k = bisect.bisect_right(issued, t) - bisect.bisect_right(taken, t)
        if k >= 8:
            window_full_ns += step
        sp = phase_scan.at(t)
        ss = s_sched.at(t)
        scanner_state[(sp, ss)] += step
        if avail.at(t) <= 0:
            for th in workers:
                if phases[th].at(t) not in ("decoding", "start", "end"):
                    nojob_total += step
                    nojob_scanner[(sp, ss, "window full" if k >= 8 else "window open")] += step
        t += step


def q(v):
    v = sorted(v)
    return dict(n=len(v), median_us=round(statistics.median(v) / 1e3, 1), p90_us=round(v[int(0.9 * (len(v) - 1))] / 1e3, 1),
                sum_ms=round(sum(v) / 1e6, 2)) if v else {}


out = dict(
    wake_to_run={f"{p} / {h}": q(v) for (p, h), v in sorted(lat.items(), key=lambda x: -sum(x[1]))},
    idle_exit_cstate_counts={str(k): v for k, v in cstate_of_exit.most_common()},
    wake_to_run_by_cstate={str(k): q(v) for k, v in lat_by_c.items()},
    window_full_share=window_full_ns / span_ns_all,
    scanner_time_share={f"{p} / {s}": round(v / span_ns_all, 4) for (p, s), v in sorted(scanner_state.items(), key=lambda x: -x[1])},
    nojob_worker_time_by_scanner={f"{p} / {s} / {w}": round(v / nojob_total, 4) for (p, s, w), v in sorted(nojob_scanner.items(), key=lambda x: -x[1])},
)
print(json.dumps(out, indent=1))
