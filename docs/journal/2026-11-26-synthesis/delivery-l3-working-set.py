#!/usr/bin/env python3
"""Price delivery-l3-working-set's gates from the archived traces (no new runs).

Usage: python3 -I delivery-l3-working-set.py POOLED_RUNS_JSONL ZERO_COPY_RUNS_JSONL

POOLED_RUNS_JSONL is runs.jsonl from origin/evidence c71c4a56 (pooled-scan/run-20261126T133034Z,
tarball SHA-256 97e397ad...8867), unpinned; ZERO_COPY_RUNS_JSONL is runs.jsonl from cc8ba137
(zero-copy-floor/run-20261126T112751Z, 10df0d89...5e34), the scanner pinned to CPU 6 alone.
Only traced decodes (those with "activities") are used: n = 3 a cell (c71c), 4 (cc8).
ready_avg is the ready_bytes occupancy integral over the whole decode (end_ns).
"""

import json
import statistics as st
import sys

WORK = ("A5", "pool", "A6a", "A6b", "A6c", "A7", "A8", "A9", "publish", "A10")


def load(path, arms):
    """The traced decodes of the given arms at k >= 6."""
    rs = [json.loads(line) for line in open(path)]
    return [r for r in rs if "activities" in r and r["oracle"] in arms and r["k"] >= 6]


def ms(t, acts, a):
    """A thread's time in an activity, ms."""
    return t["states"][1][acts.index(a)] / 1e6


def station(r):
    """Wall, ready occupancy and each station's time for one decode."""
    acts = r["activities"]
    sc = next(t for t in r["threads"] if t["role"] == "Scanner")
    rd = next(t for t in r["threads"] if t["role"] == "Reader")
    w = [t for t in r["threads"] if t["role"] == "Worker"]
    ob = r["occupancy"]["ready_bytes"]
    return dict(
        wall=r["wall_s"] * 1e3,
        avg=ob["integral"] / r["end_ns"] / 2**20,
        hwm=ob["hwm"] / 2**20,
        scan=ms(sc, acts, "A2") + ms(sc, acts, "A3"),
        slot_wait=ms(sc, acts, "slot_wait"),
        copy_out=ms(rd, acts, "copy_out"),
        job_wait=st.mean(ms(t, acts, "job_wait") for t in w),
    )


def per_block(r):
    """Each worker's busy time per block (A7 visits), µs, and the SMT siblings it shared."""
    acts = r["activities"]
    i7 = acts.index("A7")
    ids = [acts.index(a) for a in WORK]
    main = {}
    for t in r["threads"]:
        for c, ns in t["cpus"].items():
            if ns > 0.2 * t["life_ns"]:
                main.setdefault(int(c) % 10, []).append(t["name"])
    out = []
    for t in r["threads"]:
        if t["role"] != "Worker":
            continue
        busy = sum(t["states"][1][i] for i in ids) / max(t["visits"][i7], 1) / 1e3
        cpu = int(max(t["cpus"], key=t["cpus"].get))
        out.append((busy, len(main.get(cpu % 10, [])) > 1))
    return out


def main(pooled, zero):
    """Print both tables."""
    print("## 1. Stations, per traced decode (ms; ready MiB)")
    for path, label in ((pooled, "c71c unpinned"), (zero, "cc8 scanner pinned")):
        for r in sorted(load(path, ("none", "S")), key=lambda r: (r["oracle"], r["k"])):
            if r["oracle"] == "S" and label.startswith("cc8"):
                continue
            s = station(r)
            print(
                f"{label:18} {r['oracle']:4} k={r['k']:2} wall {s['wall']:6.1f} ready avg {s['avg']:.2f}"
                f" hwm {s['hwm']:5.2f} | scan A2+A3 {s['scan']:6.1f} slot_wait {s['slot_wait']:5.1f}"
                f" | worker job_wait {s['job_wait']:5.1f} | copy_out {s['copy_out']:5.1f}"
            )
    print("\n## 2. Worker busy per block (µs): uniform part vs the slowest workers (c71c)")
    rows = load(pooled, ("none", "S"))
    base = {}
    for arm in ("none", "S"):
        b = [per_block(r) for r in rows if r["oracle"] == arm and r["k"] == 6]
        base[arm] = st.median(st.mean(sorted(x for x, _ in w)[: len(w) // 2]) for w in b)
    for r in sorted(rows, key=lambda r: (r["oracle"], r["k"])):
        w = per_block(r)
        xs = sorted(x for x, _ in w)
        lower = st.mean(xs[: len(xs) // 2])
        mean = st.mean(xs)
        sib = sorted(round(x) for x, s in w if s)
        print(
            f"{r['oracle']:4} k={r['k']:2} ready avg {station(r)['avg']:.2f} | lower half {lower:5.0f}"
            f" ({100 * (lower / base[r['oracle']] - 1):+5.1f}% vs k=6) | mean {mean:5.0f}"
            f" ({100 * (mean / base[r['oracle']] - 1):+5.1f}%) | slowest {round(xs[-1])}, {round(xs[-2])}"
            f" | on a shared physical core: {sib}"
        )


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
