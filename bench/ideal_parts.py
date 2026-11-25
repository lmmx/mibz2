#!/usr/bin/env python3
"""bench-ideal-parts' planner and driver (slice A of the ideal-parts matrix:
docs/journal/2026-11-24-algorithm-level/ideal-parts/, its design.md and its frozen
registration.md). Characterisation: it decides nothing about production.

    python3 bench/ideal_parts.py plan LSCPU_E CORES.json OUT_PLAN.json
    python3 bench/ideal_parts.py run PLAN.json --bin BIN --corpus X4.bz2 --out DIR [--blocks 12]

**plan:** the cells of slice A, the 2^2 of S (the Huffman scan) and E (emission), each real or
ideal:
- **the pinned cells:** each arm (none, S, E, SE) at 1, 6, 8 and 10 workers, the scanner pinned
  to the first CPU of the core whose pinned calibration ran fastest (stage 2b's `cores`
  phase), every other thread masked off that core;
- **the link:** all real and all ideal at 8 workers unpinned, to carry the pinned results back
  to production's regime.

Every cell's treatment holds its arm (`oracle none|S|E|SE`) and the standard warm-up (`prespin
100`). Traces (the ledgers on) run in the first TRACE_BLOCKS blocks; every wall is a decode with
the ledgers off.

**run:** every cell once a block, in randomised order, each decode its own process
(examples/character.rs `--one`, the features character and scan-oracle). In every arm the process
runs the record pass, the pre-spin, the timed decode, then the check pass (byte identity). Each
child's record goes to runs.jsonl with its cell; a decode that fails is recorded as failed and
the run carries on.
"""

from __future__ import annotations

import argparse
import json
import os
import random
import shutil
import statistics
import subprocess
import tempfile
import time

ARMS = ("none", "S", "E", "SE")
COUNTS = (1, 6, 8, 10)
LINK = (8, ("none", "SE"))
TRACE_BLOCKS = 2
PRESPIN_MS = 100
ENV = {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C"}


def med(xs):
    xs = [x for x in xs if x is not None]
    return statistics.median(xs) if xs else float("nan")


def topology(lscpu_e):
    """lscpu -e's CPUs by core: [[cpu, sibling], ...] in core order"""
    lines = open(lscpu_e).read().splitlines()
    head = lines[0].split()
    cores = {}
    for line in lines[1:]:
        v = line.split()
        if len(v) >= len(head) - 1 and v[0].isdigit():
            cores.setdefault(int(v[head.index("CORE")]), []).append(int(v[head.index("CPU")]))
    return [sorted(cs) for _, cs in sorted(cores.items())]


def fastest(byc, cores_json):
    """the scanner's CPU: the first CPU of the core whose pinned calibration ran fastest (the
    median ns of its repeats; ties to the lowest core)"""
    cal = json.load(open(cores_json))
    ns = {}
    for cs in byc:
        r = cal.get(str(cs[0]))
        if r and r.get("repeats"):
            ns[cs[0]] = med(x[2] for x in r["repeats"])
    if not ns:
        return byc[0][0], {}
    return min(sorted(ns), key=lambda c: ns[c]), ns


def plan(a) -> dict:
    byc = topology(a.lscpu_e)
    scpu, ns = fastest(byc, a.cores)
    score = next(cs for cs in byc if scpu in cs)
    others = [x for cs in byc for x in cs if x not in score]
    cells = []
    for k in COUNTS:
        for arm in ARMS:
            cells.append({"section": "pinned", "k": k, "arm": arm, "pinned": 1, "mask": others,
                          "treat": [f"oracle {arm}", f"prespin {PRESPIN_MS}", f"scanner_cpu {scpu}"]})  # fmt: skip
    k, arms = LINK
    for arm in arms:
        cells.append({"section": "link", "k": k, "arm": arm,
                      "treat": [f"oracle {arm}", f"prespin {PRESPIN_MS}"]})  # fmt: skip
    out = {"design": "ideal-parts slice A", "arms": list(ARMS), "counts": list(COUNTS),
           "link": {"k": k, "arms": list(arms)}, "scanner_cpu": scpu, "scanner_core": score,
           "pinned_mask": others, "core_calibration_ns": ns, "trace_blocks": TRACE_BLOCKS,
           "prespin_ms": PRESPIN_MS, "cells": cells}  # fmt: skip
    json.dump(out, open(a.out, "w"), indent=1)
    print(f"bench-ideal-parts slice A: {len(cells)} cells ({len(ARMS)} arms x {len(COUNTS)} counts "
          f"pinned, the link at {k}); the scanner pinned to CPU {scpu} (core {score})")  # fmt: skip
    return out


def ons(p, b):
    """a cell's decodes in block b: a timed decode (the ledgers off), and a trace in the plan's
    first trace_blocks blocks"""
    return (0, 1) if b < p.get("trace_blocks", TRACE_BLOCKS) else (0,)


def run(a):
    p = json.load(open(a.plan))
    runs = open(os.path.join(a.out, "runs.jsonl"), "a")
    tmp = tempfile.mkdtemp(prefix="ideal-")
    tb = os.path.join(tmp, "b.tsv")
    cells = p["cells"]
    t_warm = time.time()
    while time.time() - t_warm < a.prewarm:
        subprocess.run(
            [a.bin, "--one", a.corpus, "10", "0", tb, "0", "none"], capture_output=True, env=ENV
        )
    t0 = time.time()
    position, prev = 0, 0
    for b in range(a.blocks):
        entries = [(ci, on) for ci in range(len(cells)) for on in ons(p, b)]
        random.Random(a.seed * 1000 + b).shuffle(entries)
        for o, (ci, on) in enumerate(entries):
            c = cells[ci]
            tf = os.path.join(tmp, f"t{ci:04d}.txt")
            with open(tf, "w") as f:
                f.write("\n".join(c["treat"]) + "\n")
            if os.path.exists(tb):
                os.remove(tb)
            cmd = [a.bin, "--one", a.corpus, f"{c['k']:02d}", str(on), tb, "0", tf]
            if c.get("mask"):
                cmd = ["taskset", "-c", ",".join(map(str, c["mask"]))] + cmd
            time.sleep(a.settle)
            extra = {"on_arm": on, "cell": ci, "block": b, "order": o, "position": position,
                     "prev_k": prev, **{kk: v for kk, v in c.items() if kk not in ("treat", "mask")}}  # fmt: skip
            position += 1
            prev = c["k"]
            try:
                r = subprocess.run(cmd, capture_output=True, text=True, env=ENV)
                if r.returncode != 0 or not r.stdout.strip():
                    rec = {
                        "kind": "failed",
                        "rc": r.returncode,
                        "stderr": r.stderr[-2000:],
                        **extra,
                    }
                else:
                    rec = json.loads(r.stdout.strip().splitlines()[-1])
                    rec.update(extra)
            except Exception as e:
                rec = {"kind": "failed", "stderr": f"{type(e).__name__}: {e}", **extra}
            if rec["kind"] == "failed":
                print(
                    f"run: {c['section']} {c['arm']} k {c['k']} block {b} failed: {rec['stderr'][-300:]}",
                    flush=True,
                )
            runs.write(json.dumps(rec, separators=(",", ":")) + "\n")
            runs.flush()
        print(f"block {b} done, {time.time() - t0:.0f} s", flush=True)
    shutil.rmtree(tmp, ignore_errors=True)


def main() -> int:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    pp = sub.add_parser("plan")
    pp.add_argument("lscpu_e")
    pp.add_argument("cores")
    pp.add_argument("out")
    pr = sub.add_parser("run")
    pr.add_argument("plan")
    pr.add_argument("--bin", required=True)
    pr.add_argument("--corpus", required=True)
    pr.add_argument("--out", required=True)
    pr.add_argument("--blocks", type=int, default=12)
    pr.add_argument("--settle", type=float, default=0.0)
    pr.add_argument("--prewarm", type=float, default=50.0)
    pr.add_argument("--seed", type=int, default=20261125)
    a = ap.parse_args()
    if a.cmd == "plan":
        plan(a)
    else:
        run(a)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
