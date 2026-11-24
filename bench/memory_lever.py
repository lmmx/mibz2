#!/usr/bin/env python3
"""The memory-lever run's sign reading (case 3 of its rule, #145 LOG 6902022809).

For each memory arm, round by round: its wall over `mibz2-workers-w`'s at 10 threads and at
4 threads, and in how many rounds the first is below the second. The intervals of the
rule's other cases are `ab summary`'s; this adds nothing to them.

    memory_lever.py RUNS.jsonl [HIGH [LOW]]     (thread counts; default 10 and 4)
"""
import json
import math
import sys

REF = "mibz2-workers-w"
ARMS = ["mibz2-workers-w-lazy", "mibz2-workers-w-nt"]


def main(argv):
    path = argv[1]
    high = int(argv[2]) if len(argv) > 2 else 10
    low = int(argv[3]) if len(argv) > 3 else 4
    wall, outputs, errors = {}, set(), 0
    with open(path) as f:
        for line in f:
            if not line.strip():
                continue
            r = json.loads(line)
            if r.get("err"):
                errors += 1
                continue
            wall[(r["arm"], r["threads"], r["round"])] = r["wall"]
            outputs.add((r.get("out"), r.get("sha256") or r.get("hash")))
    print(f"readings {len(wall)}, with an error {errors}, distinct outputs {len(outputs)}")
    if errors or len(outputs) != 1:
        print("NOT READ: an arm failed, or the arms' outputs differ")
        return 1
    rounds = sorted({k[2] for k in wall})
    status = 0
    for arm in ARMS:
        rows = []
        for i in rounds:
            keys = [(a, t, i) for a in (arm, REF) for t in (high, low)]
            if all(k in wall for k in keys):
                at_high = wall[(arm, high, i)] / wall[(REF, high, i)]
                at_low = wall[(arm, low, i)] / wall[(REF, low, i)]
                rows.append((i, at_high, at_low))
        print(f"\n{arm} over {REF}, wall, round by round")
        print(f"  round  at {high} threads  at {low} threads  {high} below {low}")
        for i, at_high, at_low in rows:
            print(f"  {i:5d}  {at_high:13.4f}  {at_low:12.4f}  {'yes' if at_high < at_low else 'no'}")
        if not rows:
            print("  no round has all four readings")
            status = 1
            continue
        below = sum(1 for _, h, l in rows if h < l)
        n = len(rows)
        geo = lambda xs: math.exp(sum(math.log(x) for x in xs) / len(xs))
        # two-sided sign test: the chance of a split at least this uneven under no difference
        k = max(below, n - below)
        p = min(1.0, 2 * sum(math.comb(n, j) for j in range(k, n + 1)) / 2**n)
        print(
            f"  geometric means: {geo([h for _, h, _ in rows]):.4f} at {high}, "
            f"{geo([l for _, _, l in rows]):.4f} at {low}"
        )
        print(f"  the ratio at {high} threads is below the ratio at {low} in {below} of {n} rounds (sign test p = {p:.4f})")
        met = n == 10 and below >= 9
        print(f"  case 3's count (at least 9 of 10 rounds): {'met' if met else 'not met'}")
    return status


if __name__ == "__main__":
    sys.exit(main(sys.argv))
