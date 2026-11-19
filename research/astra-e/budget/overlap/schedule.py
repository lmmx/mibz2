"""Finite-window early-release screens. Source rates are conditional; worker CPU is unchanged."""
import csv
import heapq
import json
import lzma
import math
import tarfile
from collections import defaultdict
from pathlib import Path
from statistics import median

HERE = Path(__file__).resolve().parent
BUDGET = HERE.parent
WORKSTATION = BUDGET.parent / "workstation"


def rows(path):
    open_text = lzma.open if path.suffix == ".xz" else open
    with open_text(path, "rt") as stream:
        return list(csv.DictReader(stream))


def observations():
    per = defaultdict(list)
    for r in rows(BUDGET / "scanner.csv.xz"):
        if int(r["block"]) < 431:
            per["scan", int(r["block"])].append(float(r["ns"]))
    for r in rows(BUDGET / "copy.csv.xz"):
        per["copy", int(r["block"])].append(float(r["ns"]))
    for r in rows(BUDGET / "stages.csv.xz"):
        if r["arm"] == "ahead":
            for key, field in [("work", "total_ns"), ("front", "front_ns")]:
                per[key, int(r["block"])].append(float(r[field]))
    assert len(per) == 431 * 4 and all(len(v) == 10 for v in per.values())
    return {key: [median(per[key, i]) for i in range(431)] * 4
            for key in ["work", "front", "copy", "scan"]}


def weighted_bound(delta, window):
    # A chain can collect this release edge only after its previous release has
    # crossed ordered take, a K-block slot edge, and re-entered the scanner.
    best = [0.0]
    for i, reward in enumerate(delta):
        best.append(max(best[-1], reward + best[max(0, i + 1 - window)]))
    return best[-1]


def replay(w, slots, scan, front, work, copy, early, assignment=None):
    free = [(0., k) for k in range(w)]
    ready_worker = [0.] * w
    roster, takes = [], []
    prior_scan = wait_slot = wait_worker = 0.
    exposed = 0
    for i, cpu in enumerate(work):
        s = max(prior_scan, takes[i - slots] if i >= slots else 0.)
        wait_slot += s - prior_scan
        e = s + scan[i]
        if assignment is None:
            avail, worker = heapq.heappop(free)
        else:
            worker = assignment[i]
            avail = ready_worker[worker]
        roster.append(worker)
        if avail < e:
            exposed += 1
            wait_worker += min(e - max(avail, s), front[i])
        begin = max(s if early else e, avail)
        rest = max(e, begin + front[i])
        done = rest + cpu - front[i]
        if assignment is None:
            heapq.heappush(free, (done, worker))
        ready_worker[worker] = done
        takes.append(max(done, takes[-1] + copy[i - 1] if takes else 0.))
        prior_scan = e
    return takes[-1] + copy[-1], roster, wait_slot, exposed, wait_worker


def load_rates():
    roles = rows(BUDGET / "i9-roles.csv")
    with tarfile.open(WORKSTATION / "evidence/edge-i9-683c22e.tar.gz") as tf:
        raw = [json.loads(line) for line in tf.extractfile(next(m for m in tf if m.name.endswith("/runs.jsonl")))]
    measured = {int(r["workers"]): r for r in rows(WORKSTATION / "edge-i9.csv") if r["reference"] == "lazy"}
    output = []
    for w in [7, 8, 9, 10]:
        old = [r for r in roles if int(r["workers"]) == w and r["arm"] == "opend"]
        old_lazy = [float(r["wall"]) for r in roles if int(r["workers"]) == w and r["arm"] == "lazy"]
        new = [r for r in raw if r["workers"] == w and r["arm"] == "candidate"]
        assert len(old) == len(old_lazy) == 12 and len(new) == 16
        edge_wall = median(r["wall_s"] for r in new)
        edge_cpu = median(r["cpu_s"] for r in new)
        lazy_wall = float(measured[w]["wall_ref_s"])
        scale = lazy_wall / median(old_lazy)
        # Per-thread ticks are coarse and from a different sweep. Transport the
        # scanner and main demand using the common Lazy wall calibration, and
        # transport worker share using total process CPU on the measured edge.
        s = scale * median(float(r["scanner"]) for r in old)
        reader = scale * median(float(r["main"]) for r in old)
        worker = edge_cpu * median(float(r["worker_sum"]) / float(r["cpu"]) for r in old)
        output.append((w, edge_wall, lazy_wall, 1 - .8 / float(measured[w]["wall"]),
                       s, reader, worker))
    return output


def scaled(v, total):
    factor = total / sum(v)
    return [x * factor for x in v]


def main():
    shape = observations()
    rate_rows, scenarios = [], []
    for w, t, lazy, need, scan_s, read_s, worker_s in load_rates():
        rate_rows.append(dict(workers=w,edge_wall=t,lazy_wall=lazy,need_fraction=need,need_s=t*need,
                              scan_s=scan_s,reader_s=read_s,worker_sum_s=worker_s,
                              scan_over_target=scan_s/(t*(1-need))))
        # Separate scan/worker stress tests of +/-20%, and the common old-role transport.
        for sf, wf, label in [(1,1,"transported"),(.8,1,"faster_scan"),(1.2,1,"slower_scan"),
                              (1,.8,"faster_workers"),(1,1.2,"slower_workers")]:
            scan = scaled(shape["scan"], scan_s * sf)
            copy = scaled(shape["copy"], read_s)
            work = scaled(shape["work"], worker_s * wf)
            front = [d * f / base for d, f, base in zip(work, shape["front"], shape["work"])]
            discounts = [min(s, f) for s, f in zip(scan, front)]
            atomic, roster, slotwait, exposed, avail = replay(w,4*w,scan,front,work,copy,False)
            fixed = replay(w,4*w,scan,front,work,copy,True,roster)[0]
            dynamic = replay(w,4*w,scan,front,work,copy,True)[0]
            unbounded, fixed_roster, *_ = replay(w,len(work),scan,front,work,copy,False)
            free_early = replay(w,len(work),scan,front,work,copy,True,fixed_roster)[0]
            packing = weighted_bound(discounts, 4*w)
            assert -1e-10 <= atomic-fixed <= packing+1e-10
            assert -1e-10 <= atomic-dynamic <= packing+1e-10
            assert -1e-10 <= unbounded-free_early <= max(discounts)+1e-10
            scenarios.append(dict(workers=w,scenario=label,atomic_s=atomic,atomic_over_edge=atomic/t,
                                  fixed_gain_s=atomic-fixed,rerouted_gain_s=atomic-dynamic,
                                  fixed_over_required=(atomic-fixed)/(t*need),
                                  rerouted_over_required=(atomic-dynamic)/(t*need),
                                  max_no_feedback_s=max(discounts),window_chain_bound_s=packing,
                                  bound_over_required=packing/(t*need),slot_wait_s=slotwait,
                                  ready_front_opportunities=exposed,unpropagated_front_s=avail,
                                  no_feedback_gain_s=unbounded-free_early))
    for file, data in [("rates.csv",rate_rows),("scenarios.csv",scenarios)]:
        with (HERE / file).open("w") as stream:
            writer = csv.DictWriter(stream,fieldnames=data[0],lineterminator="\n")
            writer.writeheader()
            writer.writerows(data)


if __name__ == "__main__":
    main()
