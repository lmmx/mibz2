"""Descriptive: for long runnable intervals of workers in the receiver's phases, was the
wakee placed on the waker's own CPU (the CPU that emitted sched_waking), and who woke it."""
import bisect, collections, json, sys
from pathlib import Path
repo, cap = Path(sys.argv[1]), Path(sys.argv[2])
sys.path.insert(0, str(repo / "bench"))
import dispatch as d
CPUS = [1, 2, 3, 4]
res = collections.Counter(); who = collections.Counter()
for f in sorted(cap.glob("w4-r??.tsv.gz")):
    b = f.name[:-7]
    meta, names, tids, marks = d.load_marks(f)
    perf, _ = d.load_perf(cap / f"{b}.perf.txt.gz")
    al = d.align(marks, names, tids, meta["epoch_monotonic_ns"], perf); corr = al["correction_ns"]; epoch = meta["epoch_monotonic_ns"]
    T = lambda ns: epoch + ns + corr
    role = {th: d.role_of(nm) for th, nm in names.items()}; tid_of = {th: tids[th] for th in names}; tidrole = {tid_of[th]: role[th] for th in names}
    sched, _, _ = d.sched_reconstruct(perf, set(tid_of.values()), CPUS)
    waking = collections.defaultdict(list); wakeup = collections.defaultdict(list)
    for t, cpu, tid, ev, rest in perf:
        kv = dict(d.KV.findall(rest))
        if ev == "sched:sched_waking":
            waking[int(kv["pid"])].append((t, cpu, tid))
        elif ev == "sched:sched_wakeup":
            wakeup[int(kv["pid"])].append((t, int(kv["target_cpu"])))
    for th in names:
        if role[th] != "worker": continue
        ph = d.Step([(T(m[0]), d.PHASE_OF[m[1]]) for m in marks[th] if m[1] in d.PHASE_OF], "start")
        tid = tid_of[th]; seq = sched.get(tid, [])
        wk = waking[tid]; wkt = [x[0] for x in wk]; wu = wakeup[tid]; wut = [x[0] for x in wu]
        for (t0, s0), (t1, s1) in zip(seq, seq[1:]):
            if s0 != "Q" or s1 != "R" or t1 - t0 <= 500_000 or ph.at(t0) not in ("mutex_wait", "recv"): continue
            i = bisect.bisect_left(wkt, t0)
            j = bisect.bisect_left(wut, t0)
            if i >= len(wk) or wk[i][0] != t0 or j >= len(wu) or wu[j][0] > t1:
                res["preempted (no wake): runnable after being switched out"] += t1 - t0
                continue
            _, wcpu, wtid = wk[i]; tgt = wu[j][1]
            res["placed on the waker's CPU" if tgt == wcpu else "placed elsewhere"] += t1 - t0
            who[(ph.at(t0), tidrole.get(wtid, "other"))] += t1 - t0
print(json.dumps(dict(long_runnable_ms={k: round(v / 1e6, 1) for k, v in res.most_common()},
                      by_phase_and_waker_ms={f"{p} / woken by {w}": round(v / 1e6, 1) for (p, w), v in who.most_common()}), indent=1))
