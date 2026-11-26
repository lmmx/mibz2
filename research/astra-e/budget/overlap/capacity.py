"""A necessary target-capacity envelope for ALL 10 x 10 archived block cost shapes."""
import csv
from collections import defaultdict
from statistics import median
from schedule import BUDGET, HERE, load_rates, rows, scaled, weighted_bound


def samples(path, field, arm=None):
    result = defaultdict(dict)
    for r in rows(path):
        if int(r["block"]) < 431 and (arm is None or r["arm"] == arm):
            group = result[int(r["round"])]
            block = int(r["block"])
            assert block not in group
            group[block] = int(r[field])
    assert set(result) == set(range(1, 11))
    return {k: [v[i] for i in range(431)] for k, v in result.items()}


def main():
    scanner = samples(BUDGET / "scanner.csv.xz", "ns")
    front = samples(BUDGET / "stages.csv.xz", "front_ns", "ahead")
    worker = samples(BUDGET / "stages.csv.xz", "total_ns", "ahead")
    result = []
    for w, t, _, need, *_ in load_rates():
        goal = t * (1 - need)
        for scan_round, s in scanner.items():
            scan_budget = scaled(s * 4, goal)
            scan_only = weighted_bound(scan_budget, 4*w)
            for work_round, f in front.items():
                factor = w * goal / (4 * sum(worker[work_round]))
                front_budget = [cost * factor for cost in f] * 4
                delta = list(map(min, scan_budget, front_budget))
                both = weighted_bound(delta, 4*w)
                assert both <= scan_only + 1e-12
                result.append(dict(workers=w, scan_round=scan_round, work_round=work_round,
                                   goal_s=goal, required_s=t*need, scan_only_s=scan_only,
                                   scan_only_over_required=scan_only/(t*need),
                                   complete_bound_s=both, complete_over_required=both/(t*need),
                                   no_window_bound_s=max(delta)))
    with (HERE / "capacity.csv").open("w") as stream:
        writer = csv.DictWriter(stream, fieldnames=result[0], lineterminator="\n")
        writer.writeheader()
        writer.writerows(result)
    for w in [7,8,9,10]:
        group = [r for r in result if r["workers"] == w]
        for key in ["scan_only_over_required", "complete_over_required"]:
            a = [r[key] for r in group]
            print(w, key, *(round(x,4) for x in (min(a), median(a), max(a))),
                  sum(x>=1 for x in a))
        print(w,"max_complete_ms",round(max(r["complete_bound_s"] for r in group)*1000,3))


if __name__ == "__main__":
    main()
