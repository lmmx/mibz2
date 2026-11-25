#!/usr/bin/env python3
"""Does the clock explain the character study's noise at 8 and 10 workers? (The Coordinator's
question after stage 2b, evidence a4f0dcef; the Pathfinder's frequency hypothesis.)

    python3 bench/character_freq.py PLAN.json RUNS.jsonl [--label 2b]

Every timed decode (the ledgers off) carries the process's PMU group 0, cycles and reference
cycles over the stopwatch's region: their ratio times the nominal clock is the achieved
frequency while the decode's threads ran (G). It also carries each CPU's mean scaling_cur_freq,
sampled during the decode (coarse: about one read in 50 ms).

- **Q1:** each anchor's base, sorted by wall, with G, the scanner's CPU's sampled frequency,
  the user CPU time and the cycles. Which mode does the slow wall go with: a lower G (the
  clock) or more cycles at the same G (the work, or its IPC)?
- **Q2:** the paired log ratios log(T_arm / T_base) over blocks, each screen arm's mean
  removed (its effect), pooled over the arms at an anchor:
  - the raw SD;
  - the SD of the frequency-adjusted ratios (each wall times its G: the wall the decode would
    have taken at a common clock, if the wall scales as 1/G);
  - R^2 of the clock alone (the OLS slope on the paired log(G) differences, through the
    centred data) and of the clock with the cycles.
  The share of the paired variance the clock explains is the R^2; the SD after adjusting is
  what a frequency-adjusted analysis would size with.
"""

from __future__ import annotations

import argparse
import json
import math
import re
import statistics as st
import sys

NOMINAL_GHZ = 3.7  # the i9-10900K's reference clock (lscpu's "@ 3.70GHz")


def ghz(r) -> float:
    pm = r.get("pmu") or {}
    c, rc = pm.get("cycles"), pm.get("ref-cycles")
    if not c or not rc or not rc[0]:
        return float("nan")
    return c[0] / rc[0] * NOMINAL_GHZ


def cycles(r) -> float:
    pm = r.get("pmu") or {}
    return float(pm["cycles"][0]) if pm.get("cycles") else float("nan")


def load(plan, runs):
    p = json.load(open(plan))
    t = {}
    for line in open(runs):
        r = json.loads(line)
        if r.get("kind") == "decode" and r.get("on_arm", r.get("on")) == 0:
            t[(r["cell"], r["block"])] = r
    return p, t


def ols(y, xs):
    """least squares of y on the columns xs (no intercept: the data are centred): R^2 and the
    coefficients"""
    k = len(xs)
    a = [[sum(xs[i][n] * xs[j][n] for n in range(len(y))) for j in range(k)] for i in range(k)]
    b = [sum(xs[i][n] * y[n] for n in range(len(y))) for i in range(k)]
    # (k is 1 or 2: solve directly)
    if k == 1:
        beta = [b[0] / a[0][0]] if a[0][0] else [0.0]
    else:
        det = a[0][0] * a[1][1] - a[0][1] * a[1][0]
        beta = (
            [(b[0] * a[1][1] - b[1] * a[0][1]) / det, (a[0][0] * b[1] - a[1][0] * b[0]) / det]
            if det
            else [0.0, 0.0]
        )
    fit = [sum(beta[i] * xs[i][n] for i in range(k)) for n in range(len(y))]
    sst = sum(v * v for v in y)
    ssr = sum((y[n] - fit[n]) ** 2 for n in range(len(y)))
    return (1 - ssr / sst if sst else float("nan")), beta


def analyse(p, t, label="", out=sys.stdout) -> dict:
    cells = p["cells"]
    blocks = sorted({b for (_, b) in t})
    scpu = p.get("scanner_cpu")
    res = {}
    pr = lambda *a: print(*a, file=out)  # noqa: E731
    for k in sorted({c["k"] for c in cells if c["section"] == "screen"}):
        b0 = next(
            i
            for i, c in enumerate(cells)
            if c["section"] == "screen" and c["k"] == k and c["arm"] == "base"
        )
        pinned = bool(cells[b0].get("pinned"))
        pr(
            f"\n## {label} k {k} ({'the scanner pinned to CPU %s' % scpu if pinned else 'unpinned'})\n"
        )
        pr("Q1: the base's decodes, by wall")
        pr("  wall     G (GHz)  scanner CPU (GHz)  user (s)  cycles (1e9)")
        base = sorted((t[(b0, b)] for b in blocks if (b0, b) in t), key=lambda r: r["wall_s"])
        for r in base:
            sf = (
                r["freq_khz"][scpu] / 1e6
                if scpu is not None and r.get("freq_khz")
                else float("nan")
            )
            pr(
                f"  {r['wall_s']:.3f}   {ghz(r):.2f}     {sf:.2f}              {r['user_s']:.3f}     {cycles(r) / 1e9:.2f}"
            )
        g = [ghz(r) for r in base]
        if g and all(math.isfinite(x) for x in g):
            cut = (max(g) + min(g)) / 2 if max(g) - min(g) > 0.1 else None
            if cut:
                hi = [r["wall_s"] for r in base if ghz(r) >= cut]
                lo = [r["wall_s"] for r in base if ghz(r) < cut]
                med_all = st.median(r["wall_s"] for r in base)
                slow = [r for r in base if r["wall_s"] > med_all * 1.02]
                slow_lo = sum(1 for r in slow if ghz(r) < cut)
                pr(f"  the clock's two levels: {min(g):.2f} and {max(g):.2f} GHz (cut {cut:.2f}); "
                   f"median wall at the high level {st.median(hi):.3f} s ({len(hi)}), at the low {st.median(lo) if lo else float('nan'):.3f} s ({len(lo)})")  # fmt: skip
                pr(
                    f"  the slow decodes (wall > 1.02 x the median): {len(slow)}, of which {slow_lo} at the low clock"
                )
                res[(k, "q1")] = {
                    "levels": (min(g), max(g)),
                    "slow": len(slow),
                    "slow_low_clock": slow_lo,
                }
        # Q2: the paired log ratios, each arm's mean removed
        y, xg, xc, ya = [], [], [], []
        for i, c in enumerate(cells):
            if c["section"] != "screen" or c["k"] != k or c["arm"] == "base":
                continue
            rows = [(t[(i, b)], t[(b0, b)]) for b in blocks if (i, b) in t and (b0, b) in t]
            rows = [(a, b) for a, b in rows if math.isfinite(ghz(a)) and math.isfinite(ghz(b))]
            if len(rows) < 3:
                continue
            yy = [math.log(a["wall_s"] / b["wall_s"]) for a, b in rows]
            gg = [math.log(ghz(a) / ghz(b)) for a, b in rows]
            cc = [math.log(cycles(a) / cycles(b)) for a, b in rows]
            aa = [yy[n] + gg[n] for n in range(len(rows))]  # (the wall at a common clock)
            for v, dst in ((yy, y), (gg, xg), (cc, xc), (aa, ya)):
                m = st.fmean(v)
                dst += [x - m for x in v]
        if len(y) < 4:
            continue
        df = len(y) - 1
        sd_raw = math.sqrt(sum(v * v for v in y) / df)
        sd_adj = math.sqrt(sum(v * v for v in ya) / df)
        r2_g, bg = ols(y, [xg])
        r2_gc, bgc = ols(y, [xg, xc])
        pr(f"\nQ2: {len(y)} paired ratios over the screen's arms (each arm's mean removed)")
        pr(f"  the paired SD of log(T/T0): {sd_raw:.4f}")
        pr(f"  the clock alone: R^2 {r2_g:.3f} (slope {bg[0]:+.2f}; -1 if the wall scales as 1/G)")
        pr(f"  the clock and the cycles: R^2 {r2_gc:.3f} (slopes {bgc[0]:+.2f}, {bgc[1]:+.2f})")
        pr(
            f"  the SD of the frequency-adjusted ratios (each wall times its G): {sd_adj:.4f} ({sd_adj / sd_raw:.0%} of the raw)"
        )
        res[(k, "q2")] = {"n": len(y), "sd_raw": sd_raw, "sd_adj": sd_adj, "r2_clock": r2_g, "slope": bg[0],
                          "r2_clock_cycles": r2_gc}  # fmt: skip
    return res


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("plan")
    ap.add_argument("runs")
    ap.add_argument("--label", default="")
    a = ap.parse_args()
    p, t = load(a.plan, a.runs)
    m = re.search(r"run-\d+T\d+Z", a.runs)
    print(f"# The clock and the noise: {a.label or (m.group(0) if m else a.runs)}")
    analyse(p, t, a.label)
    return 0


if __name__ == "__main__":
    sys.exit(main())
