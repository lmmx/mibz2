"""Descriptive: for each worker's runnable interval in the receiver's phases, whether an
allocated CPU sat idle meanwhile, where the wakeup placed it, where it ran, and what it
waited behind."""
import bisect, collections, json, re, statistics, sys
from pathlib import Path
repo, cap = Path(sys.argv[1]), Path(sys.argv[2])
sys.path.insert(0, str(repo / "bench"))
import dispatch as d
CPUS = [1, 2, 3, 4]
tot = collections.Counter(); n = collections.Counter(); behind = collections.Counter(); same_cpu = collections.Counter()
long_idle = []
for f in sorted(cap.glob("w4-r??.tsv.gz")):
    b = f.name[:-7]
    meta, names, tids, marks = d.load_marks(f)
    perf, _ = d.load_perf(cap / f"{b}.perf.txt.gz")
    al = d.align(marks, names, tids, meta["epoch_monotonic_ns"], perf); corr = al["correction_ns"]; epoch = meta["epoch_monotonic_ns"]
    T = lambda ns: epoch + ns + corr
    role = {th: d.role_of(nm) for th, nm in names.items()}; tid_of = {th: tids[th] for th in names}
    tidrole = {tid_of[th]: role[th] for th in names}
    sched, occupant, _ = d.sched_reconstruct(perf, set(tid_of.values()), CPUS)
    occ = {c: d.Step(list(v)) for c, v in occupant.items()}
    wake_target = collections.defaultdict(list)
    ins = collections.defaultdict(list)
    for t, cpu, tid, ev, rest in perf:
        kv = dict(d.KV.findall(rest))
        if ev in ("sched:sched_wakeup", "sched:sched_wakeup_new"):
            wake_target[int(kv["pid"])].append((t, int(kv.get("target_cpu", "-1"))))
        elif ev == "sched:sched_switch":
            ins[int(kv["next_pid"])].append((t, cpu))
    for th in names:
        if role[th] != "worker": continue
        ph = d.Step([(T(m[0]), d.PHASE_OF[m[1]]) for m in marks[th] if m[1] in d.PHASE_OF], "start")
        tid = tid_of[th]; seq = sched.get(tid, [])
        wt = wake_target[tid]; wtt = [x[0] for x in wt]; it = ins[tid]; itt = [x[0] for x in it]
        for (t0, s0), (t1, s1) in zip(seq, seq[1:]):
            if s0 != "Q" or s1 != "R": continue
            p = ph.at(t0)
            if p not in ("mutex_wait", "recv"): continue
            j = bisect.bisect_right(itt, t1) - 1
            ran = it[j][1] if j >= 0 and it[j][0] == t1 else None
            k = bisect.bisect_right(wtt, t1) - 1
            tgt = wt[k][1] if k >= 0 and wt[k][0] >= t0 - 1 else None
            # time with at least one allocated CPU idle, sampled every 1 µs-equivalent via change points
            pts = sorted({t0, t1} | {x for c in CPUS for x in occ[c].t if t0 < x < t1})
            idle_t = sum(b_ - a_ for a_, b_ in zip(pts, pts[1:]) if any(occ[c].at((a_ + b_) / 2) == 0 for c in CPUS))
            dur = t1 - t0
            key = "long (>500 µs)" if dur > 500_000 else "short"
            tot[(key, "with an allocated CPU idle")] += idle_t; tot[(key, "all four busy")] += dur - idle_t; n[key] += 1
            if ran is not None:
                occ_role = tidrole.get(occ[ran].at(t0 + 1), "idle" if occ[ran].at(t0 + 1) == 0 else "foreign")
                behind[(key, occ_role)] += dur
                same_cpu[(key, "ran on its wakeup target" if tgt == ran else ("target unknown" if tgt is None else "migrated"))] += dur
            if dur > 500_000 and idle_t > 0.5 * dur:
                long_idle.append(dur)
out = dict(intervals=dict(n), runnable_ms={f"{a} / {b}": round(v / 1e6, 1) for (a, b), v in sorted(tot.items())},
           waited_behind_ms={f"{a} / {b}": round(v / 1e6, 1) for (a, b), v in sorted(behind.items(), key=lambda x: -x[1])},
           placement_ms={f"{a} / {b}": round(v / 1e6, 1) for (a, b), v in sorted(same_cpu.items(), key=lambda x: -x[1])},
           long_mostly_idle=dict(n=len(long_idle), median_us=round(statistics.median(long_idle) / 1e3) if long_idle else None))
print(json.dumps(out, indent=1))
