#!/usr/bin/env python3
"""bench-pooled-scan's planner and driver: the pooled scan's kill test
(docs/journal/2026-11-24-algorithm-level/pooled-scan/registration.md §4, with §5's reading of
the floor run and §7's clarifications, frozen before the data). A diagnostic (RULES C1a): it
stops or continues the thread, and adopts nothing.

    python3 bench/pooled_scan.py plan LSCPU_E CORES.json OUT_PLAN.json
    python3 bench/pooled_scan.py run PLAN.json --bin BIN --corpus X4.bz2 --out DIR [--blocks 12]

**plan:** six arms (§4's table), each a pair of the scan (the shipped scanner, pooled `P`, or
the scan oracle `S`) and construction (real, or the zero-copy oracle `C`): `none`, `P`, `C`,
`PC`, `SC` and `S`, at 6, 7, 8, 9 and 10 workers; and `none` and `P` at 1, the shipped path's
check. Every cell is unpinned (no scanner thread exists in the pooled arms to pin), with the
standard pre-spin. The plan also names the CPU that K4's finder pass is pinned to: the first
CPU of the core whose pinned calibration ran fastest, as the floor run's scanner was.

**run:** bench/ideal_parts.py's driver, unchanged: every cell once a block, in randomised
order, each decode its own process (examples/character.rs `--one`); traces (the ledgers on)
in the first TRACE_BLOCKS blocks, every wall a decode with the ledgers off.
"""

from __future__ import annotations

import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ideal_parts as ip  # noqa: E402

ARMS = ("none", "P", "C", "PC", "SC", "S")
COUNTS = (6, 7, 8, 9, 10)
# the shipped path's check: at 1 worker the pooled scan is the shipped scanner
ONE = (1, ("none", "P"))
TRACE_BLOCKS = 3


def plan(a) -> dict:
    byc = ip.topology(a.lscpu_e)
    fcpu, ns = ip.fastest(byc, a.cores)
    cells = []
    k, arms = ONE
    for arm in arms:
        cells.append({"section": "one", "k": k, "arm": arm,
                      "treat": [f"oracle {arm}", f"prespin {ip.PRESPIN_MS}"]})  # fmt: skip
    for k in COUNTS:
        for arm in ARMS:
            cells.append({"section": "kill", "k": k, "arm": arm,
                          "treat": [f"oracle {arm}", f"prespin {ip.PRESPIN_MS}"]})  # fmt: skip
    out = {"design": "pooled-scan: the kill test", "arms": list(ARMS), "counts": list(COUNTS),
           "one": {"k": ONE[0], "arms": list(ONE[1])}, "judged": [8, 9], "finder_cpu": fcpu,
           "core_calibration_ns": ns, "trace_blocks": TRACE_BLOCKS, "prespin_ms": ip.PRESPIN_MS,
           "cells": cells}  # fmt: skip
    json.dump(out, open(a.out, "w"), indent=1)
    print(f"bench-pooled-scan: {len(cells)} cells ({len(ARMS)} arms x {len(COUNTS)} counts, and "
          f"{len(ONE[1])} at 1), unpinned; K4's finder pinned to CPU {fcpu}")  # fmt: skip
    return out


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
    pr.add_argument("--seed", type=int, default=20261126)
    pr.add_argument("--timeout", type=float, default=120.0, help="each decode's limit, s")
    a = ap.parse_args()
    if a.cmd == "plan":
        plan(a)
    else:
        ip.run(a)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
