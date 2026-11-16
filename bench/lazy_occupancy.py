#!/usr/bin/env python3
"""Who binds the decoder: each thread's CPU time over the run's wall time (`just
bench-lazy-occupancy`, #145).

For each corpus, arm and thread count, over the rounds: the scanner's share, the workers'
(mean and most), the consuming thread's and the feeder's. A share is a thread's CPU seconds
(`ab`'s per-thread sample, 10 ms ticks) over the run's wall seconds, so it reads well only
on a run long enough for many ticks: the recipe runs the file and several copies of it end
to end (a corpus named `...-xN` beside the one it copies), and from the two walls this also
splits a run's fixed cost from a copy's time.

    lazy_occupancy.py RUNS.jsonl
"""
import collections
import json
import statistics
import sys

SCAN, READ, MAIN = "mibz2-scan", "mibz2-read", "main"


def main(argv):
    rows, outputs, errors = [], collections.defaultdict(set), 0
    with open(argv[1]) as f:
        for line in f:
            if not line.strip():
                continue
            r = json.loads(line)
            if r.get("err"):
                errors += 1
                continue
            rows.append(r)
            outputs[r.get("corpus", "")].add((r.get("out"), r.get("sha256") or r.get("hash")))
    differ = sorted(c for c, o in outputs.items() if len(o) != 1)
    print(f"readings {len(rows)}, with an error {errors}, corpora {len(outputs)}, with differing outputs {len(differ)}")
    if errors or differ or not rows:
        print(f"NOT READ: an arm failed, or the arms' outputs differ on a corpus {differ}")
        return 1
    groups = collections.defaultdict(list)
    for r in rows:
        cpu = collections.defaultdict(list)
        for item in r.get("thread_cpu", "").split():
            name, _, value = item.rpartition("=")
            cpu[name].append(float(value))
        workers = [v for name, vs in cpu.items() if name not in (SCAN, READ, MAIN, "ab", "sampler") for v in vs]
        wall = r["wall"]
        groups[(r.get("corpus", ""), r["arm"], r["threads"])].append(
            (
                wall,
                sum(cpu.get(SCAN, [0.0])) / wall,
                (sum(workers) / len(workers) / wall) if workers else 0.0,
                (max(workers) / wall) if workers else 0.0,
                len(workers),
                sum(cpu.get(MAIN, [0.0])) / wall,
                sum(cpu.get(READ, [0.0])) / wall,
                sum(cpu.get(SCAN, [0.0])),
            )
        )
    print("\na thread's CPU over the wall, the mean of the rounds")
    print(f"  {'corpus':14s}{'arm':26s}{'threads':>8s}{'wall s':>9s}{'scanner':>9s}{'workers':>9s}{'busiest':>9s}{'  n':>4s}{'consumer':>10s}{'feeder':>8s}{'scanner s':>11s}")
    walls = {}
    for (corpus, arm, threads), v in sorted(groups.items()):
        m = lambda i: statistics.mean(x[i] for x in v)
        walls[(corpus, arm, threads)] = statistics.median(x[0] for x in v)
        print(
            f"  {corpus:14s}{arm:26s}{threads:8d}{walls[(corpus, arm, threads)]:9.4f}{m(1):9.0%}{m(2):9.0%}{m(3):9.0%}"
            f"{m(4):4.0f}{m(5):10.0%}{m(6):8.0%}{m(7):11.3f}"
        )
    # The fixed cost: with a corpus of one copy and one of N copies of it (named ...-xN), a
    # run of one copy takes `fixed + a copy's time`, a run of N takes `fixed + N` of them.
    copies = {}
    for corpus in {k[0] for k in walls}:
        base, _, n = corpus.rpartition("-x")
        if base and n.isdigit() and any(k[0] == base for k in walls):
            copies[corpus] = (base, int(n))
    if copies:
        print("\nfixed cost of a run, and a copy's time once it is taken out (median walls)")
        print(f"  {'arm':26s}{'threads':>8s}{'one copy':>10s}{'N copies':>10s}{'N':>4s}{'a copy':>9s}{'fixed':>9s}{'fixed share of one copy':>26s}")
        for (corpus, arm, threads), many in sorted(walls.items()):
            if corpus not in copies:
                continue
            base, n = copies[corpus]
            one = walls.get((base, arm, threads))
            if one is None or n < 2:
                continue
            each = (many - one) / (n - 1)
            fixed = one - each
            print(f"  {arm:26s}{threads:8d}{one:10.4f}{many:10.4f}{n:4d}{each:9.4f}{fixed:9.4f}{fixed / one:26.0%}")
    print(
        "\nreading: a stage binds where its share is near 100%. The scanner is one thread and "
        "cannot be given more; the workers can; the consumer is the caller's. The scanner's seconds "
        "are its CPU for the same work at every thread count: where they rise with the "
        "threads, it is sharing its core."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
