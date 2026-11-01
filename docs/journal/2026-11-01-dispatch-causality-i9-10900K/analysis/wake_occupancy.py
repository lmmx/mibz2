"""Descriptive check of the researcher's figures: each worker sched_waking matched to that
worker's next switch-in; split by whether an allocated CPU was idle at the wake."""
import bisect, collections, json, statistics, sys
from pathlib import Path
repo, cap = Path(sys.argv[1]), Path(sys.argv[2])
sys.path.insert(0, str(repo / "bench"))
import dispatch as d
CPUS = [1, 2, 3, 4]
grp = collections.defaultdict(list)
for f in sorted(cap.glob("w4-r??.tsv.gz")):
    b = f.name[:-7]
    meta, names, tids, marks = d.load_marks(f)
    perf, _ = d.load_perf(cap / f"{b}.perf.txt.gz")
    workers = {tids[th] for th, nm in names.items() if d.role_of(nm) == "worker"}
    _, occupant, _ = d.sched_reconstruct(perf, set(tids.values()), CPUS)
    occ = {c: d.Step(list(v)) for c, v in occupant.items()}
    ins = collections.defaultdict(list); wakes = collections.defaultdict(list)
    for t, cpu, tid, ev, rest in perf:
        kv = dict(d.KV.findall(rest))
        if ev == "sched:sched_switch":
            ins[int(kv["next_pid"])].append(t)
        elif ev == "sched:sched_waking":
            wakes[int(kv["pid"])].append(t)
    for w in workers:
        it = ins[w]
        last = None
        for t in wakes[w]:
            i = bisect.bisect_right(it, t)
            if i >= len(it):
                continue
            t1 = it[i]
            if last is not None and t1 == last:
                continue  # a second waking before the same switch-in
            last = t1
            idle = any(occ[c].at(t) == 0 for c in CPUS)
            grp["an allocated CPU idle" if idle else "all four busy"].append(t1 - t)
out = {}
for k, v in grp.items():
    v.sort()
    out[k] = dict(n=len(v), median_us=round(statistics.median(v) / 1e3, 1), over_100us=round(sum(x > 100_000 for x in v) / len(v), 3),
                  p90_us=round(v[int(0.9 * (len(v) - 1))] / 1e3, 1))
out["total"] = sum(len(v) for v in grp.values())
print(json.dumps(out, indent=1))
