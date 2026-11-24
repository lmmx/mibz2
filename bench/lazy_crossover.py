#!/usr/bin/env python3
"""The lazy-crossover run's two reported readings (#145 QUEUED 6903395401 and 6903426694).

Neither is a gate. The rule's intervals are `ab summary`'s; this adds nothing to them.

1. **The memory dose.** For each forced workers arm, round by round: its CPU time at HIGH
   threads over its CPU time at 1 thread, on the same file and build. Workers share
   nothing but the machine, so the ratio is what a block costs among HIGH - 1 others over
   what it costs alone. Near 1, there is no memory penalty left to win.
2. **Peak memory.** Each arm's largest resident set at each thread count (the median over
   the rounds).

    lazy_crossover.py RUNS.jsonl DOSE.jsonl [HIGH]     (HIGH: default 10)
"""
import json
import math
import statistics
import sys

DOSE_ARMS = ["mibz2-workers-eager", "mibz2-workers-lazy"]


def read(path):
    rows, outputs, errors = [], set(), 0
    with open(path) as f:
        for line in f:
            if not line.strip():
                continue
            r = json.loads(line)
            if r.get("err"):
                errors += 1
                continue
            rows.append(r)
            outputs.add((r.get("out"), r.get("sha256") or r.get("hash")))
    return rows, outputs, errors


def main(argv):
    runs, dose = argv[1], argv[2]
    high = int(argv[3]) if len(argv) > 3 else 10
    geo = lambda xs: math.exp(sum(math.log(x) for x in xs) / len(xs))
    status = 0

    rows, outputs, errors = read(dose)
    print(f"dose readings {len(rows)}, with an error {errors}, distinct outputs {len(outputs)}")
    if errors or len(outputs) != 1:
        print("NOT READ: an arm failed, or the arms' outputs differ")
        return 1
    cpu = {(r["arm"], r["threads"], r["round"]): r["cpu"] for r in rows}
    rounds = sorted({k[2] for k in cpu})
    means = {}
    for arm in DOSE_ARMS:
        pairs = [
            (i, cpu[(arm, 1, i)], cpu[(arm, high, i)])
            for i in rounds
            if (arm, 1, i) in cpu and (arm, high, i) in cpu
        ]
        print(f"\n{arm}: CPU seconds at {high} threads over CPU seconds at 1, round by round")
        print(f"  round  at 1 thread  at {high} threads  ratio")
        for i, one, many in pairs:
            print(f"  {i:5d}  {one:11.4f}  {many:13.4f}  {many / one:.4f}")
        if not pairs:
            print(f"  no round has both readings")
            status = 1
            continue
        ratios = [many / one for _, one, many in pairs]
        means[arm] = geo(ratios)
        print(
            f"  the dose: {means[arm]:.3f} (geometric mean of {len(ratios)} rounds; "
            f"least {min(ratios):.3f}, most {max(ratios):.3f})"
        )
    if len(means) == 2:
        eager, lazy = (means[a] for a in DOSE_ARMS)
        print(
            f"\nthe penalty of {high} threads over 1: {100 * (eager - 1):.1f}% with a slot for "
            f"every letter, {100 * (lazy - 1):.1f}% with lazy letters"
        )

    rows, outputs, errors = read(runs)
    print(f"\nrun readings {len(rows)}, with an error {errors}, distinct outputs {len(outputs)}")
    if errors or len(outputs) != 1:
        print("NOT READ: an arm failed, or the arms' outputs differ")
        return 1
    rss = {}
    for r in rows:
        if "rss_kb" in r:
            rss.setdefault((r["arm"], r["threads"]), []).append(r["rss_kb"])
    arms = sorted({k[0] for k in rss})
    threads = sorted({k[1] for k in rss})
    if rss:
        print("\npeak resident memory, MiB (the median over the rounds)")
        print("  " + "threads".ljust(22) + "".join(f"{t:>8d}" for t in threads))
        for arm in arms:
            cells = [
                f"{statistics.median(rss[(arm, t)]) / 1024:8.0f}" if (arm, t) in rss else " " * 8
                for t in threads
            ]
            print("  " + arm.ljust(22) + "".join(cells))
    return status


if __name__ == "__main__":
    sys.exit(main(sys.argv))
