#!/usr/bin/env python3
"""The memory dose of each arm of `just bench-lazy-place` (#145 QUEUED 6906630511): its CPU
seconds at HIGH threads over its own CPU seconds at 1 thread, on the same file and build,
round by round, for each HIGH the run has above 1. Reported, not a gate: the rule reads the
wall intervals of `ab summary`.

At 1 thread `mibz2` is the sequential reader (`BzReader::new`) and every other arm one lazy
worker beside its scanner, so the other arms' denominators are alike and their doses differ
by their CPU at HIGH alone; `mibz2`'s is on another footing, as the crossover run's is not.

    lazy_place.py DOSE.jsonl
"""
import collections
import json
import math
import sys


def main(argv):
    rows, outputs, errors = [], set(), 0
    with open(argv[1]) as f:
        for line in f:
            if not line.strip():
                continue
            r = json.loads(line)
            if r.get("err"):
                errors += 1
                continue
            rows.append(r)
            outputs.add((r.get("out"), r.get("sha256") or r.get("hash")))
    print(f"dose readings {len(rows)}, with an error {errors}, distinct outputs {len(outputs)}")
    if errors or len(outputs) != 1 or not rows:
        print("NOT READ: an arm failed, or the arms' outputs differ")
        return 1
    cpu = {}
    for r in rows:
        cpu[(r["arm"], r["threads"], r["round"])] = r["cpu"]
    arms = sorted({r["arm"] for r in rows})
    highs = sorted({r["threads"] for r in rows if r["threads"] > 1})
    print("\nCPU seconds at HIGH threads over the same arm's at 1, round by round")
    print(f"  {'arm':32s}{'threads':>8s}{'dose':>8s}{'least':>8s}{'most':>8s}{'rounds':>8s}")
    for arm in arms:
        for high in highs:
            ratios = [
                cpu[(arm, high, k)] / cpu[(arm, 1, k)]
                for (a, t, k) in cpu
                if a == arm and t == high and (arm, 1, k) in cpu and cpu[(arm, 1, k)] > 0
            ]
            if not ratios:
                continue
            g = math.exp(sum(math.log(x) for x in ratios) / len(ratios))
            print(f"  {arm:32s}{high:8d}{g:8.3f}{min(ratios):8.3f}{max(ratios):8.3f}{len(ratios):8d}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
