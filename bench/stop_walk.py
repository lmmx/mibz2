#!/usr/bin/env python3
"""The stop-and-walk run (`just bench-stop-walk`; MONODROMY's HANDOFF 6916839849, Gate 2): the
rule's verdict, read from the run's records, and its two recorded readings. Standard library only.

    stop_walk.py RUNS.jsonl DOSE.jsonl [--judge 8,9,10,12,20] [--primary mibz2-workers-stop15]

**The arms.** `mibz2-workers-lazy`, the shipped kernel at 8 threads and more (lazy letters), is
the reference. `mibz2-workers-stop14`, `-stop15` and `-stop16` are the same workers, plan and
window with the lazy induction stopped after ⌈β r⌉ cells, β = 1.4, 1.5, 1.6, and the rest of the
block walked (`bwt::StopLazyCells15`). An arm differs from the reference in the kernel alone.

**The rule** (posted on #145 before the data). It decides one thing: whether stop-and-walk
becomes *the* lazy kernel, inside lazy's existing range (`LAZY_FROM_THREADS` = 8 and up, the
workers past the cores included). It adds no thread threshold.
- At each thread count, an arm's step is the geometric mean of the rounds' paired wall ratios,
  arm over reference, with its 95% interval (Student's t), as `ab summary` gives it. FASTER if
  the interval lies below 1, SLOWER if above, else NO DIFFERENCE (PERFORMANCE-POLICY's "beyond
  noise").
- The judged counts are 8, 9, 10, 12 and 20 threads: the whole of lazy's range on the i9. The
  primary arm is β = 1.5.
- **Adopts:** the primary arm is FASTER at one judged count or more, SLOWER at none, and its
  peak resident memory (the median over the rounds) is at most 10% above the reference's at
  every judged count (PERFORMANCE-POLICY rule 4).
- **Held on memory:** the wall conditions hold but the memory one does not: the Coordinator
  weighs it under rule 4.
- **A finding, not an adoption:** FASTER at some judged counts and SLOWER at others. The arm
  gets no range of its own; any change would be to the lazy threshold itself, a separate
  decision.
- **Not adopted** otherwise. β = 1.4 and 1.6 are read the same way, but only to say which way β
  would move: they decide nothing.
- **Void** if any run failed, or the arms' outputs differ on a file.

**Recorded, not judged** (`DOSE.jsonl`: one copy, at 1 thread and 10):
1. *1 thread:* each arm's wall ratio against the reference. The arms run more of the time in a
   latency-bound walk than the shipped writer; a 1-thread loss is expected.
2. *The memory dose:* each arm's CPU at 10 threads over its CPU at 1, round by round, as
   `bench/lazy_crossover.py` has it, and that dose over the reference's. Below 1: a block costs
   less among nine others than the shipped kernel's does, which is what the stop is for.
"""
import argparse
import collections
import json
import math
import statistics
import sys

REF = "mibz2-workers-lazy"
ARMS = ["mibz2-workers-stop14", "mibz2-workers-stop15", "mibz2-workers-stop16"]
# Student's t, two-sided 95%, by degrees of freedom (1..30); 1.96 beyond
T95 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
       2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060,
       2.056, 2.052, 2.048, 2.045, 2.042]


def interval(ratios):
    """(geometric mean, low, high) of paired ratios, with the 95% interval"""
    logs = [math.log(r) for r in ratios]
    n = len(logs)
    mu = statistics.mean(logs)
    if n < 2:
        return math.exp(mu), float("nan"), float("nan")
    t = T95[n - 2] if n - 1 <= len(T95) else 1.96
    h = t * statistics.stdev(logs) / math.sqrt(n)
    return math.exp(mu), math.exp(mu - h), math.exp(mu + h)


def read(path):
    """The records by (arm, threads, round), the runs that failed, the outputs by file"""
    rows, errors, outputs = {}, 0, collections.defaultdict(set)
    with open(path) as f:
        for line in f:
            if not line.strip():
                continue
            r = json.loads(line)
            if r.get("err") or r.get("termination", "success") != "success":
                errors += 1
                continue
            rows[(r["arm"], r["threads"], r["round"])] = r
            outputs[r.get("corpus", "")].add((r.get("out"), r.get("sha256") or r.get("hash")))
    return rows, errors, outputs


def step(rows, arm, t, key="wall", ref=REF):
    ratios = [r[key] / rows[(ref, t, i)][key] for (a, tt, i), r in rows.items()
              if a == arm and tt == t and (ref, t, i) in rows]
    return interval(ratios) + (len(ratios),) if ratios else None


def verdict(g, lo, hi):
    return "FASTER" if hi < 1 else "SLOWER" if lo > 1 else "NO DIFFERENCE"


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("dose")
    ap.add_argument("--judge", default="8,9,10,12,20")
    ap.add_argument("--primary", default="mibz2-workers-stop15")
    a = ap.parse_args(argv[1:])
    judge = [int(t) for t in a.judge.split(",")]

    rows, errors, outputs = read(a.runs)
    drows, derrors, doutputs = read(a.dose)
    differ = [c for o in (outputs, doutputs) for c, v in o.items() if len(v) != 1]
    print(f"runs {len(rows)} and {len(drows)}; failed {errors + derrors}; files whose arms' outputs differ {differ}")
    if errors or derrors or differ:
        print("VOID: a run failed, or the arms' outputs differ")
        return 1

    print(f"\nwall, arm over {REF}: the geometric mean of the paired rounds' ratios [95% interval];"
          " peak resident memory, the median over the rounds, over the reference's")
    found, rss = {}, {}

    def peak(arm, t):
        v = [r["rss_kb"] for (a, tt, _), r in rows.items() if a == arm and tt == t and r.get("rss_kb")]
        return statistics.median(v) if v else None

    for t in sorted({k[1] for k in rows}):
        print(f"\n{t} threads{' (judged)' if t in judge else ''}")
        ref_rss = peak(REF, t)
        for arm in ARMS:
            s = step(rows, arm, t)
            if s is None:
                print(f"  {arm}: no paired rounds")
                continue
            g, lo, hi, n = s
            v = verdict(g, lo, hi)
            found[(arm, t)] = v
            a_rss = peak(arm, t)
            rss[(arm, t)] = a_rss / ref_rss if a_rss and ref_rss else None
            mem = f"; memory {rss[(arm, t)]:.3f}" if rss[(arm, t)] else "; memory not recorded"
            print(f"  {arm}: {g:.4f} [{lo:.4f}, {hi:.4f}] over {n} rounds: {v}{mem}")

    prim = [found.get((a.primary, t)) for t in judge]
    missing = [t for t, v in zip(judge, prim) if v is None]
    print()
    if missing:
        print(f"VOID: the primary arm has no reading at {missing} threads")
        return 1
    faster = [t for t, v in zip(judge, prim) if v == "FASTER"]
    slower = [t for t, v in zip(judge, prim) if v == "SLOWER"]
    over = [t for t in judge if rss.get((a.primary, t)) is None or rss[(a.primary, t)] > 1.10]
    if faster and not slower and not over:
        print(f"VERDICT: ADOPTS. {a.primary} is FASTER at {faster} threads, SLOWER at none, and its peak "
              f"memory is within 10% of the reference's at every judged count: it becomes the lazy kernel")
    elif faster and not slower:
        print(f"VERDICT: HELD ON MEMORY. {a.primary} is FASTER at {faster} threads and SLOWER at none, but its "
              f"peak memory is more than 10% above the reference's (or not recorded) at {over} threads: "
              "PERFORMANCE-POLICY rule 4, the Coordinator's to weigh")
    elif faster and slower:
        print(f"VERDICT: A FINDING, NOT ADOPTED. {a.primary} is FASTER at {faster} and SLOWER at {slower} "
              "threads. It gets no range of its own: any change would be to the lazy threshold itself, "
              "a separate decision")
    else:
        print(f"VERDICT: NOT ADOPTED. {a.primary} is FASTER at none of {judge} threads"
              + (f" and SLOWER at {slower}" if slower else ""))
    for arm in ARMS:
        if arm != a.primary:
            print(f"  ({arm}, read for direction only: "
                  + ", ".join(f"{t} {found.get((arm, t), 'none')}" for t in judge) + ")")

    print("\nRecorded, not judged (one copy):")
    for arm in ARMS:
        s = step(drows, arm, 1)
        if s:
            print(f"  1 thread, {arm}: wall {s[0]:.4f} [{s[1]:.4f}, {s[2]:.4f}] over {s[3]} rounds")
    high = max((k[1] for k in drows), default=1)
    doses = {}
    for arm in [REF] + ARMS:
        ratios = [drows[(arm, high, i)]["cpu"] / drows[(arm, 1, i)]["cpu"]
                  for (aa, tt, i) in drows if aa == arm and tt == 1 and (arm, high, i) in drows]
        if ratios:
            doses[arm] = ratios
            g, lo, hi = interval(ratios)
            print(f"  dose, {arm}: CPU at {high} over CPU at 1, {g:.4f} [{lo:.4f}, {hi:.4f}] over {len(ratios)} rounds")
    if REF in doses:
        for arm in ARMS:
            if arm in doses:
                pairs = list(zip(doses[arm], doses[REF]))
                g, lo, hi = interval([x / y for x, y in pairs])
                print(f"  dose over the reference's, {arm}: {g:.4f} [{lo:.4f}, {hi:.4f}]")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
