#!/usr/bin/env python3
"""bench-clock-mechanism's planner, driver and report (a C1a diagnostic; it decides nothing).

    python3 bench/clock_mechanism.py plan LSCPU_E CORES.json OUT_PLAN.json
    python3 bench/clock_mechanism.py run PLAN.json --bin BIN --corpus X4.bz2 --out DIR [--blocks 48]
    python3 bench/clock_mechanism.py report PLAN.json RUNS.jsonl --lscpu LSCPU [--blocks 48]

Stage 2b's timed decodes at 10 workers ran at one of two clocks, 4.91 or 4.09 GHz, in about a
quarter of them the low one, at random (LOG 7152489390). Is the low clock set by how long the
machine idled before the decode, or by a cold start that a short spin would warm? Journal
2026-10-28 section 4 (bench-regimes, 6 threads) found an idle gap of 0.15 s before a run gave slow
runs (5 of 12) and none without it, and under perf a clock about 7% lower after the gap; its
mechanism was left open, "a lower clock after idle and more stall cycles aren't separated". Each
decode here carries its clock, so the two are separated.

The decode is 2b's pinned base at 10: the scanner pinned (`scanner_cpu`) to the fastest core's
first CPU, every other thread masked off that core, the ledgers off. Four arms, one decode each
in each of the randomised blocks:
- **S0, S0.2, S1, S3:** the settle (the idle before the decode) 0 (back to back with the decode
  before it), 0.2, 1 and 3 s. S1 is 2b's, the reference.
- **SPIN:** the settle 1 s, then the pacing loop on every CPU at once for about 100 ms (the
  binary's `--calibrate`, one process a CPU, each pinned), then the decode at once.

Each decode's clock G is its PMU group 0, cycles over reference cycles times 3.7 GHz. A decode
is at the LOW clock iff G < CUT_GHZ, fixed before the data at the midpoint of 2b's two levels at
10 (4.09 and 4.91 GHz).
"""

from __future__ import annotations

import argparse
import json
import math
import os
import random
import re
import statistics as st
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import character_freq as cf  # noqa: E402
import character_stage2 as s2  # noqa: E402

K = 10
CUT_GHZ = 4.50
ARMS = [("S0", 0.0, 0), ("S0.2", 0.2, 0), ("S1", 1.0, 0), ("S3", 3.0, 0), ("SPIN", 1.0, 1)]
REF = "S1"
SPIN_S = 0.10
Z = 1.959964


def plan(a) -> dict:
    byc = s2.topology(a.lscpu_e)
    scpu, ns = s2.fastest(byc, a.cores)
    score = next(cs for cs in byc if scpu in cs)
    mask = [x for cs in byc for x in cs if x not in score]
    cal = json.load(open(a.cores))
    # the spin's iterations a CPU: SPIN_S at the scanner's core's calibrated rate
    r = cal.get(str(scpu)) or next(iter(cal.values()), None)
    it_per_ns = (
        (r["iterations"] / med_ns)
        if r and (med_ns := st.median(x[2] for x in r["repeats"]))
        else 2.0
    )
    spin_iters = int(SPIN_S * 1e9 * it_per_ns)
    out = {"k": K, "cut_ghz": CUT_GHZ, "scanner_cpu": scpu, "scanner_core": score, "mask": mask,
           "all_cpus": sorted(x for cs in byc for x in cs), "core_calibration_ns": ns,
           "spin_iterations": spin_iters, "arms": [{"arm": n, "settle": s, "spin": p} for n, s, p in ARMS]}  # fmt: skip
    json.dump(out, open(a.out, "w"), indent=1)
    print(f"bench-clock-mechanism: k {K}, the scanner pinned to CPU {scpu} (core {score}); arms "
          f"{[n for n, _, _ in ARMS]}; the spin {spin_iters} iterations a CPU (about {SPIN_S * 1e3:.0f} ms)")  # fmt: skip
    return out


def spin(binary, cpus, iters) -> float:
    """the pacing loop on every CPU at once, each process pinned; returns its wall"""
    t0 = time.perf_counter()
    ps = [
        subprocess.Popen(
            ["taskset", "-c", str(c), binary, "--calibrate", str(iters), "1"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            env=s2.ENV,
        )  # fmt: skip
        for c in cpus
    ]
    for p in ps:
        p.wait()
    return time.perf_counter() - t0


def run(a):
    p = json.load(open(a.plan))
    runs = open(os.path.join(a.out, "runs.jsonl"), "a")
    tmp = tempfile.mkdtemp(prefix="clock-")
    tb = os.path.join(tmp, "b.tsv")
    tf = os.path.join(tmp, "t0000.txt")
    with open(tf, "w") as f:
        f.write(f"scanner_cpu {p['scanner_cpu']}\n")
    mask = ",".join(map(str, p["mask"]))
    t_warm = time.time()
    while time.time() - t_warm < a.prewarm:
        subprocess.run(
            [a.bin, "--one", a.corpus, "10", "0", tb, "0", "none"], capture_output=True, env=s2.ENV
        )
    position = 0
    t0 = time.time()
    for b in range(a.blocks):
        arms = list(p["arms"])
        random.Random(a.seed * 1000 + b).shuffle(arms)
        for o, arm in enumerate(arms):
            extra = {"arm": arm["arm"], "block": b, "order": o, "position": position, "k": p["k"]}
            position += 1
            try:
                time.sleep(arm["settle"] * a.settle_scale)
                spun = spin(a.bin, p["all_cpus"], p["spin_iterations"]) if arm["spin"] else None
                if os.path.exists(tb):
                    os.remove(tb)
                r = subprocess.run(["taskset", "-c", mask, a.bin, "--one", a.corpus, f"{p['k']:02d}", "0", tb, "0", tf],
                                   capture_output=True, text=True, env=s2.ENV)  # fmt: skip
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
                    rec["spin_s"] = spun
            except Exception as e:
                rec = {"kind": "failed", "stderr": f"{type(e).__name__}: {e}", **extra}
            if rec["kind"] == "failed":
                print(f"run: {arm['arm']} block {b} failed: {rec['stderr'][-300:]}", flush=True)
            runs.write(json.dumps(rec, separators=(",", ":")) + "\n")
            runs.flush()
        if b % 8 == 7 or b == a.blocks - 1:
            print(f"block {b} done, {time.time() - t0:.0f} s", flush=True)


def wilson(x, n):
    if n == 0:
        return float("nan"), float("nan")
    ph = x / n
    d = 1 + Z * Z / n
    c = (ph + Z * Z / (2 * n)) / d
    h = Z * math.sqrt(ph * (1 - ph) / n + Z * Z / (4 * n * n)) / d
    return c - h, c + h


def newcombe(x1, n1, x2, n2):
    """the 95% interval of p1 - p2 (Newcombe's hybrid score interval, method 10)"""
    p1, p2 = x1 / n1, x2 / n2
    l1, u1 = wilson(x1, n1)
    l2, u2 = wilson(x2, n2)
    d = p1 - p2
    return d - math.sqrt((p1 - l1) ** 2 + (u2 - p2) ** 2), d + math.sqrt(
        (u1 - p1) ** 2 + (p2 - l2) ** 2
    )


def interval(v):
    m = st.fmean(v)
    if len(v) < 2:
        return m, float("nan"), float("nan")
    hw = s2.tcrit(len(v) - 1) * st.stdev(v) / math.sqrt(len(v))
    return m, m - hw, m + hw


def report(a) -> int:
    p = json.load(open(a.plan))
    recs = [json.loads(x) for x in open(a.runs) if x.strip()]
    void = []
    if not any("i9-10900K" in m for m in re.findall(r"Model name:\s*(.+)", open(a.lscpu).read())):
        void.append("not the i9-10900K")
    got = {}
    for r in recs:
        if r.get("kind") == "failed":
            void.append(f"a decode failed: {r.get('arm')} block {r.get('block')}")
        elif r.get("kind") == "decode":
            if r.get("bytes") != a.out_bytes:
                void.append(f"{r['arm']} block {r['block']}: {r.get('bytes')} bytes")
            if r.get("majflt", 0) > 0:
                void.append(f"{r['arm']} block {r['block']}: major faults")
            got.setdefault((r["arm"], r["block"]), []).append(r)
    arms = [x["arm"] for x in p["arms"]]
    for arm in arms:
        for b in range(a.blocks):
            if len(got.get((arm, b), [])) != 1:
                void.append(f"{arm} block {b}: {len(got.get((arm, b), []))} decodes")
    dec = {k: v[0] for k, v in got.items()}
    nog = sum(1 for r in dec.values() if not math.isfinite(cf.ghz(r)))
    if nog > 0.1 * max(1, len(dec)):
        void.append(f"{nog} of {len(dec)} decodes without the clock (PMU group 0)")
    print("# bench-clock-mechanism (a diagnostic: decides nothing)\n")
    print(
        f"k {p['k']}, the scanner pinned to CPU {p['scanner_cpu']}; LOW iff G < {p['cut_ghz']:.2f} GHz; {a.blocks} blocks"
    )
    if void:
        print(f"VOID ({len(void)} problems): the figures below are for diagnosis only")
        for v in void[:20]:
            print(f"  {v}")
    blocks = range(a.blocks)

    def low(arm):
        g = [cf.ghz(dec[(arm, b)]) for b in blocks if (arm, b) in dec]
        g = [x for x in g if math.isfinite(x)]
        return sum(1 for x in g if x < p["cut_ghz"]), len(g), g

    xr, nr, _ = low(REF)
    print(
        "\narm    n   low share [95%]        vs S1 [95%]              label    G median  wall median  SD log wall  paired SD vs S1  wall / S1 [95%]"
    )
    out = {}
    for arm in arms:
        x, n, g = low(arm)
        lo, hi = wilson(x, n) if n else (float("nan"),) * 2
        walls = [dec[(arm, b)]["wall_s"] for b in blocks if (arm, b) in dec]
        sd = st.stdev([math.log(w) for w in walls]) if len(walls) > 1 else float("nan")
        pair = [
            math.log(dec[(arm, b)]["wall_s"] / dec[(REF, b)]["wall_s"])
            for b in blocks
            if (arm, b) in dec and (REF, b) in dec
        ]
        if arm == REF or not n or not nr:
            dl, dh, label, psd, ratio = (
                float("nan"),
                float("nan"),
                "(ref)" if arm == REF else "",
                float("nan"),
                "-",
            )
        else:
            dl, dh = newcombe(x, n, xr, nr)
            label = (
                "CONTROL"
                if dh < 0
                else "WORSE"
                if dl > 0
                else "NO-EFFECT"
                if (dl >= -0.10 and dh <= 0.10)
                else "UNRESOLVED"
            )
            psd = st.stdev(pair) if len(pair) > 1 else float("nan")
            m, l2, h2 = interval(pair)
            ratio = f"{math.exp(m):.4f} [{math.exp(l2):.4f}, {math.exp(h2):.4f}]"
        d = x / n - xr / nr if n and nr else float("nan")
        print(f"{arm:<5} {n:>3}  {x / n if n else float('nan'):.2f} [{lo:.2f}, {hi:.2f}]   "
              f"{d:+.2f} [{dl:+.2f}, {dh:+.2f}]      {label:<10} {st.median(g) if g else float('nan'):.2f}      "
              f"{st.median(walls) if walls else float('nan'):.3f}        {sd:.4f}       {psd:.4f}           {ratio}")  # fmt: skip
        out[arm] = {"low": x, "n": n, "diff": (d, dl, dh), "label": label, "sd_log_wall": sd}
    print("\n(CONTROL: the arm's low-clock share's difference from S1 has its 95% interval wholly below 0;"
          " WORSE: wholly above; NO-EFFECT: wholly within +-0.10; else UNRESOLVED)")  # fmt: skip
    with open(os.path.join(a.out, "clock.json"), "w") as f:
        json.dump(out, f, indent=1)
    if void:
        print(f"\nVOID ({len(void)} problems: {'; '.join(void[:3])})")
        return 1
    return 0


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
    pr.add_argument("--blocks", type=int, default=48)
    pr.add_argument("--prewarm", type=float, default=50.0)
    pr.add_argument("--settle-scale", type=float, default=1.0, help="(DRY: 0 makes every settle 0)")
    pr.add_argument("--seed", type=int, default=20261127)
    rp = sub.add_parser("report")
    rp.add_argument("plan")
    rp.add_argument("runs")
    rp.add_argument("--lscpu", required=True)
    rp.add_argument("--out", required=True)
    rp.add_argument("--blocks", type=int, default=48)
    rp.add_argument("--out-bytes", type=int, default=4 * 209_700_196)
    a = ap.parse_args()
    if a.cmd == "plan":
        plan(a)
    elif a.cmd == "run":
        run(a)
    else:
        return report(a)
    return 0


if __name__ == "__main__":
    sys.exit(main())
