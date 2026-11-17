#!/usr/bin/env python3
"""The workers' CPU and everything else's, read from a run's judged records, for each arm at each
count (`just bench-workers-sweep-low`, for MONODROMY's record 46, LOG 6943777329 on #145).
Reported, not judged: it decides nothing, and no verdict reads it. Standard library only.

    thread_split.py RUNS.jsonl [--workers ARM=N|T ...]

**What a record carries** (`ab run`, bench 3.x): `wall`, `cpu` (the child's total, the kernel's),
and `thread_cpu`, each busy thread's CPU seconds sampled from /proc every 10 ms
("mibz2-0=1.23 mibz2-1=1.20 main=0.31 mibz2-scan=0.12 ..."; a thread below 2% of the busiest's
CPU is left out). The decoder's workers are the threads named `mibz2-N` (`StreamDecoder`'s);
the scanner (`mibz2-scan`), the reader (`mibz2-read`) and the main thread (reading the decoder and
hashing its output) are not.

**What it prints**, for each corpus, count and arm, the median over the rounds of:
- *the workers' busy share:* the workers' CPU over (wall × W);
- *the non-workers' CPU over wall:* the total CPU less the workers', so the threads below 2% and
  the sampler's shortfall (the record's unattributed CPU) count as non-worker;
- *of which named:* the named non-worker threads' CPU over wall;
- the worker threads seen.

**W** is read from an occupancy arm's name (`mibz2-occ-N-S`, or the eager `mibz2-oce-N-S`: N),
or given: `--workers ARM=N`, or `--workers ARM=T` for the count asked (production's `mibz2`,
`Workers(T)` at 4, 6 and 8 threads; not at 10 on the i9 since #524, where it is 7). `--threads
N` reads the records at N threads asked alone, so production's W can be given count by count
(`just bench-eager-trim`: 7 at 10 threads). An arm of unknown W has no busy share printed. A record without `thread_cpu`, or with no worker
thread in it, is counted and left out, and said so.
"""
import argparse
import json
import re
import statistics
import sys

WORKER = re.compile(r"^mibz2-\d+$")


def split(thread_cpu):
    """(the workers' CPU, the named non-workers' CPU, the worker threads seen) from `thread_cpu`;
    None if it does not parse"""
    if not isinstance(thread_cpu, str) or not thread_cpu.strip():
        return None
    workers = others = 0.0
    seen = 0
    for part in thread_cpu.split():
        name, eq, secs = part.rpartition("=")
        try:
            v = float(secs)
        except ValueError:
            return None
        if not eq or not name:
            return None
        if WORKER.match(name):
            workers += v
            seen += 1
        else:
            others += v
    return workers, others, seen


def occ_workers(arm):
    m = re.match(r"^mibz2-oc[ce]-(\d+)-(\d+)$", arm)
    return int(m.group(1)) if m else None


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("--workers", action="append", default=[], metavar="ARM=N|T")
    ap.add_argument("--threads", type=int, default=None, help="the records at this count asked alone")
    a = ap.parse_args(argv[1:])
    given = {}
    for spec in a.workers:
        arm, eq, n = spec.rpartition("=")
        if not eq or not arm or not (n == "T" or n.isdigit() and int(n) > 0):
            print(f"VOID: --workers {spec!r} is not ARM=N or ARM=T")
            return 1
        given[arm] = n
    cells, skipped = {}, []
    for ln, line in enumerate(open(a.runs), 1):
        if not line.strip():
            continue
        try:
            r = json.loads(line)
            key = (str(r["corpus"]), int(r["threads"]), str(r["arm"]))
            wall, cpu = float(r["wall"]), float(r["cpu"])
        except (ValueError, KeyError, TypeError):
            skipped.append(f"line {ln}: not a record")
            continue
        if a.threads is not None and key[1] != a.threads:
            continue
        s = split(r.get("thread_cpu"))
        if s is None or s[2] == 0 or wall <= 0:
            skipped.append(f"line {ln}: {key} no worker threads in thread_cpu {r.get('thread_cpu')!r}")
            continue
        corpus, t, arm = key
        w = given.get(arm)
        w = t if w == "T" else int(w) if w else occ_workers(arm)
        cells.setdefault(key, []).append((s[0] / (wall * w) if w else None, (cpu - s[0]) / wall, s[1] / wall, s[2]))
    print("Workers' and non-workers' CPU, reported and not judged: the median over the rounds; busy share = "
          "the mibz2-N threads' CPU / (wall x W); non-worker = (total CPU - workers') / wall; named = the named "
          "non-worker threads' CPU / wall")
    for corpus in sorted({k[0] for k in cells}):
        print(f"\n{corpus}")
        for t in sorted({k[1] for k in cells if k[0] == corpus}, reverse=True):
            print(f"  {t} threads")
            for arm in sorted({k[2] for k in cells if k[:2] == (corpus, t)},
                              key=lambda x: (x != "mibz2", -(occ_workers(x) or 0), x)):
                rows = cells[(corpus, t, arm)]
                busy = [x[0] for x in rows if x[0] is not None]
                share = f"{statistics.median(busy):.3f}" if busy else "n/a (W unknown)"
                print(f"    {arm}: busy share a worker {share}; non-worker CPU / wall "
                      f"{statistics.median(x[1] for x in rows):.3f} (named {statistics.median(x[2] for x in rows):.3f}); "
                      f"worker threads seen {statistics.median(x[3] for x in rows):g}; rounds {len(rows)}")
    if skipped:
        print(f"\n{len(skipped)} records left out:")
        for s in skipped[:10]:
            print(f"  {s}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
