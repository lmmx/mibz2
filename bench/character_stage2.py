#!/usr/bin/env python3
"""bench-character-stage2's planner and driver (stage 2a; docs/journal/2026-11-25-character-stage2-registration.md).

    python3 bench/character_stage2.py plan STAGE1_RUNS.jsonl CALIBRATION.json LSCPU LSCPU_E OUT_PLAN.json
    python3 bench/character_stage2.py run PLAN.json --bin BIN --corpus X4.bz2 --out DIR [--blocks 6 ...]

**plan** applies the registration's rules to stage 1's records:
- **the anchors:** k* (the first count whose median scanner on-CPU share, schedstat run time over
  the wall, passes 0.9) and the step (the count above k* + 2 whose median scanner demand a block
  rises most from the count below, if by 5% or more; else 20), so k* - 2, k*, k* + 2 and the
  step, each within 1..20;
- **the coordinates** kept at each anchor (3% of their thread's time; A10 by the reader's waits
  over 2%; A6 as one unless each part passes 5% of the workers' time);
- **each dose:** delta x the activity's mean exclusive service a visit at that count, in core
  cycles at that count's achieved frequency, in iterations of the pacing loop (its cycles an
  iteration from the calibration).

**run** executes every cell of the plan in randomised blocks, one decode a cell a block, each in
its own process (examples/character.rs `--one`, the ledgers on), with the treatment written to a
file of a fixed-width name; a placement cell runs under `taskset`; the window-order cell decodes
the corpus with its 12 windows permuted. Each child's line goes to runs.jsonl with its cell.
"""

from __future__ import annotations

import argparse
import json
import os
import random
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import time

ACT = [
    "A0", "feed_wait", "in_wait", "in_maint", "A2", "A3", "slot_wait", "issue", "job_wait",
    "A5", "pool", "A6a", "A6b", "A6c", "A7", "A8", "A9", "publish", "A10", "read_wait",
    "lock_state", "lock_jobs", "resid",
]  # fmt: skip
CANDIDATES = {"A2": "Scanner", "A3": "Scanner", "A5": "Worker", "A6": "Worker", "A7": "Worker",
              "A8": "Worker", "A9": "Worker", "A10": "Reader"}  # fmt: skip
SCANNER_SURFACE = ["A2", "A3", "issue"]
WORKER_SURFACE = ["A5", "pool", "A6a", "A6b", "A6c", "A7", "A8", "A9", "publish"]
ENV = {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C"}
DELTA = 0.20
# the memory dose a worker's Open block (A7's end): about one core's share of the i9's 20 MiB L3,
# streamed or chased at equal bytes (8 MiB chased swamped the block in the container's preflight)
DOSE_BYTES = 2 << 20
SURFACE = [0.0, 0.10, 0.20]
FALLBACK_CYCLES_PER_ITER = 0.5  # (8 iterations folded into one multiply-add, about 4 cycles)
STREAM = b"BZh9\x31\x41\x59\x26\x53\x59"
NO_TURBO = "/sys/devices/system/cpu/intel_pstate/no_turbo"


def med(xs):
    xs = [x for x in xs if x is not None]
    return statistics.median(xs) if xs else float("nan")


def load(path):
    return [json.loads(line) for line in open(path) if line.strip()]


def excl(x, role, act):
    """a decode's exclusive time in an activity over a role's threads (ns), and its visits"""
    i = ACT.index(act)
    t_ns = sum(sum(p[i] for p in t["states"]) for t in x["threads"] if t["role"] == role)
    v = sum(t["visits"][i] for t in x["threads"] if t["role"] == role) if i < len(ACT) - 1 else 0
    return t_ns, v


def plan(a) -> dict:
    recs = [x for x in load(a.stage1) if x.get("kind") == "decode" and x.get("section") == "grid"]
    by_k: dict[int, list] = {}
    for x in recs:
        by_k.setdefault(x["k"], []).append(x)
    nominal = float(re.search(r"@\s*([0-9.]+)\s*GHz", open(a.lscpu).read()).group(1))
    # the scanner's on-CPU share and its demand a block, at each count
    share, dscan, ghz = {}, {}, {}
    for k, xs in sorted(by_k.items()):
        s = []
        for x in xs:
            sc = next(t for t in x["threads"] if t["role"] == "Scanner")
            s.append((sc["sched"][3] - sc["sched"][0]) / (x["wall_s"] * 1e9))
        share[k] = med(s)
        dscan[k] = med(
            sum(excl(x, "Scanner", c)[0] for c in ["in_maint", "A2", "A3", "issue", "resid"])
            / x["blocks"]
            for x in xs
        )
        g = [x["pmu"]["cycles"][0] / x["pmu"]["ref-cycles"][0] * nominal for x in xs
             if (x.get("pmu") or {}).get("cycles") and x["pmu"].get("ref-cycles") and x["pmu"]["ref-cycles"][0]]  # fmt: skip
        ghz[k] = med(g) if g else nominal
    passing = [k for k in sorted(share) if share[k] >= 0.9]
    kstar = passing[0] if passing else max(share, key=share.get)
    rises = {
        k: dscan[k] / dscan[k - 1] - 1 for k in sorted(dscan) if k - 1 in dscan and k > kstar + 2
    }
    best = max(rises, key=rises.get) if rises else None
    step = best if best is not None and rises[best] >= 0.05 else 20
    anchors = sorted({min(max(k, 1), 20) for k in (kstar - 2, kstar, kstar + 2, step)})
    # the loop's cycles an iteration
    cal = json.load(open(a.calibration))
    cpi = [r[0] / cal["iterations"] for r in cal["repeats"] if r[0]]
    cycles_per_iter = med(cpi) if cpi else FALLBACK_CYCLES_PER_ITER

    def iters(ns, k, delta):
        return max(0, round(delta * ns * ghz[k] / cycles_per_iter))

    cells = []
    coords = {}
    for k in anchors:
        xs = by_k[k]
        life = {
            r: med(sum(t["life_ns"] for t in x["threads"] if t["role"] == r) for x in xs)
            for r in ("Scanner", "Worker", "Reader")
        }
        svc = {}  # an activity's mean exclusive service a visit, ns
        frac = {}
        for act in ACT[:-1]:
            role = (
                "Scanner"
                if act in ("in_maint", "A2", "A3", "issue", "slot_wait", "in_wait")
                else "Reader"
                if act in ("A10", "read_wait")
                else "Worker"
            )
            t_v = [excl(x, role, act) for x in xs]
            tot = med(t for t, _ in t_v)
            vis = med(v for _, v in t_v)
            svc[act] = tot / vis if vis else 0.0
            frac[act] = tot / life[role] if life[role] else 0.0
        split = all(frac[p] > 0.05 for p in ("A6a", "A6b", "A6c"))
        kept = []
        for c, role in CANDIDATES.items():
            if c == "A6":
                if split:
                    kept += [p for p in ("A6a", "A6b", "A6c") if frac[p] >= 0.03]
                elif frac["A6a"] + frac["A6b"] + frac["A6c"] >= 0.03:
                    kept.append("A6")
                continue
            if c == "A10":
                if frac["read_wait"] > 0.02:
                    kept.append(c)
                continue
            if frac[c] >= 0.03 and svc[c] > 0:
                kept.append(c)
        coords[k] = {"kept": kept, "frac": frac, "svc_ns": svc, "ghz": ghz[k], "split_A6": split}
        cells.append({"section": "screen", "k": k, "arm": "base", "treat": []})
        for c in kept:
            if c == "A6":
                ns = svc["A6a"] + svc["A6b"] + svc["A6c"]
                cells.append(
                    {
                        "section": "screen",
                        "k": k,
                        "arm": c,
                        "treat": [f"pace A6c {iters(ns, k, DELTA)}"],
                        "delta": DELTA,
                    }
                )
            else:
                cells.append(
                    {
                        "section": "screen",
                        "k": k,
                        "arm": c,
                        "treat": [f"pace {c} {iters(svc[c], k, DELTA)}"],
                        "delta": DELTA,
                    }
                )
    svc_k = coords[kstar]["svc_ns"] if kstar in coords else {}
    # the surface at k*: (scanner dose, all-worker dose)
    for s_d in SURFACE:
        for w_d in SURFACE:
            tr = [
                f"pace {c} {iters(svc_k[c], kstar, s_d)}"
                for c in SCANNER_SURFACE
                if s_d > 0 and svc_k.get(c)
            ]
            tr += [
                f"pace {c} {iters(svc_k[c], kstar, w_d)}"
                for c in WORKER_SURFACE
                if w_d > 0 and svc_k.get(c)
            ]
            cells.append(
                {
                    "section": "surface",
                    "k": kstar,
                    "arm": f"s{int(s_d * 100):02d}w{int(w_d * 100):02d}",
                    "treat": tr,
                }
            )
    # the gates at k*
    for w in (4, 8):
        for cap in (12, 64):
            cells.append(
                {
                    "section": "gates",
                    "k": kstar,
                    "arm": f"W{w}k_cap{cap}",
                    "treat": [f"window {w * kstar}", f"ready_cap {cap << 20}"],
                }
            )
    # the memory dose at A7's end, at k* and 20
    for k in sorted({kstar, 20}):
        for arm in ("none", "stream", "chase"):
            tr = [] if arm == "none" else [f"dose A7 {DOSE_BYTES} {arm}"]
            cells.append({"section": "memory", "k": k, "arm": arm, "treat": tr})
    # placement (masks from the topology)
    cores = {}
    for line in open(a.lscpu_e).read().splitlines()[1:]:
        v = line.split()
        if len(v) >= 4 and v[0].isdigit():
            head = open(a.lscpu_e).readline().split()
            cores.setdefault(int(v[head.index("CORE")]), []).append(int(v[head.index("CPU")]))
    byc = [sorted(cs) for _, cs in sorted(cores.items())]
    if len(byc) >= 7 and all(len(c) >= 2 for c in byc[:4]):
        distinct = [c[0] for c in byc[:7]]
        siblings = [x for c in byc[:4] for x in c[:2]]
        # (turbo crossed in where no_turbo is writable without a password; else the placement
        # factor alone, the achieved frequency a covariate: SMT and turbo then not separated)
        turbo = os.access(NO_TURBO, os.W_OK)
        for arm, mask in (("distinct", distinct), ("siblings", siblings)):
            cells.append({"section": "placement", "k": 4, "arm": arm, "treat": [], "mask": mask})
            if turbo:
                cells.append(
                    {
                        "section": "placement",
                        "k": 4,
                        "arm": arm + "_noturbo",
                        "treat": [],
                        "mask": mask,
                        "no_turbo": 1,
                    }
                )
    fav = favoured(byc)
    if fav is not None:
        sib = next((c for c in byc if fav in c), [fav])
        allc = [x for c in byc for x in c]
        others = [x for x in allc if x not in sib]
        co = next((c for c in byc if fav not in c and len(c) >= 2), None)
        for k in sorted({10, step}):
            cells.append({"section": "scanner", "k": k, "arm": "unpinned", "treat": []})
            cells.append(
                {
                    "section": "scanner",
                    "k": k,
                    "arm": "favoured",
                    "treat": [f"scanner_cpu {fav}"],
                    "mask": others,
                }
            )
            if co:
                cells.append(
                    {
                        "section": "scanner",
                        "k": k,
                        "arm": "copinned",
                        "treat": [f"scanner_cpu {co[1]}"],
                        "mask": [x for x in allc if x != co[1]],
                    }
                )
    # validation: the combined +10% (A3 with A7), and the window permutation
    for k in sorted({kstar, min(kstar + 2, 20)}):
        if k in coords:
            sv = coords[k]["svc_ns"]
            cells.append(
                {
                    "section": "validation",
                    "k": k,
                    "arm": "A3+A7_10",
                    "treat": [
                        f"pace A3 {iters(sv['A3'], k, 0.10)}",
                        f"pace A7 {iters(sv['A7'], k, 0.10)}",
                    ],
                    "delta": 0.10,
                }
            )
    for k in sorted({kstar, 20}):
        cells.append(
            {"section": "validation", "k": k, "arm": "permuted", "treat": [], "corpus": "permuted"}
        )
        cells.append({"section": "validation", "k": k, "arm": "base", "treat": []})
    out = {"kstar": kstar, "step": step, "anchors": anchors, "share": share, "dscan_ns": dscan, "ghz": ghz,
           "cycles_per_iter": cycles_per_iter, "calibration": cal, "coords": coords, "cells": cells,
           "permutation": random.Random(20261125).sample(range(12), 12)}  # fmt: skip
    json.dump(out, open(a.out, "w"), indent=1)
    print(
        f"k* {kstar}, step {step}, anchors {anchors}; {len(cells)} cells; {cycles_per_iter:.3f} cycles an iteration"
    )
    for k in anchors:
        print(f"  k {k}: kept {coords[k]['kept']}")
    return out


def favoured(byc):
    """the favoured core's first CPU: the highest acpi_cppc/highest_perf, else cpuinfo_max_freq"""
    best = None
    for key in ("acpi_cppc/highest_perf", "cpufreq/cpuinfo_max_freq"):
        vals = {}
        for c in [x for cs in byc for x in cs]:
            try:
                vals[c] = int(open(f"/sys/devices/system/cpu/cpu{c}/{key}").read())
            except OSError:
                pass
        if vals and len(set(vals.values())) > 1:
            best = max(vals, key=vals.get)
            break
    return best


def permute_corpus(src, dst, perm):
    data = open(src, "rb").read()
    starts = [m.start() for m in re.finditer(re.escape(STREAM), data)]
    streams = [data[s:e] for s, e in zip(starts, starts[1:] + [len(data)])]
    if len(streams) % 12:
        sys.exit(f"{src}: {len(streams)} streams, not a multiple of 12 windows")
    with open(dst, "wb") as f:
        for c in range(len(streams) // 12):
            for i in perm:
                f.write(streams[c * 12 + i])


def run(a):
    p = json.load(open(a.plan))
    runs = open(os.path.join(a.out, "runs.jsonl"), "a")
    tmp = tempfile.mkdtemp(prefix="stage2-")
    tb = os.path.join(tmp, "b.tsv")
    perm_corpus = os.path.join(tmp, "permuted.bz2")
    permute_corpus(a.corpus, perm_corpus, p["permutation"])
    cells = p["cells"]
    t0 = time.time()
    t_warm = time.time()
    while time.time() - t_warm < a.prewarm:
        subprocess.run(
            [a.bin, "--one", a.corpus, "10", "0", tb, "0", "none"], capture_output=True, env=ENV
        )
    position, prev = 0, 0
    for b in range(a.blocks):
        order = list(range(len(cells)))
        random.Random(a.seed * 1000 + b).shuffle(order)
        for o, ci in enumerate(order):
            c = cells[ci]
            tf = os.path.join(tmp, f"t{ci:04d}.txt")
            with open(tf, "w") as f:
                f.write("\n".join(c["treat"]) + "\n")
            corpus = perm_corpus if c.get("corpus") == "permuted" else a.corpus
            # (one layout: the count two digits wide, the same block file, the treatment's file
            # named t0000..)
            cmd = [a.bin, "--one", corpus, f"{c['k']:02d}", "1", tb, "0", tf]
            if c.get("mask"):
                cmd = ["taskset", "-c", ",".join(map(str, c["mask"]))] + cmd
            time.sleep(a.settle)
            turbo_was = None
            if c.get("no_turbo"):
                turbo_was = open(NO_TURBO).read().strip()
                with open(NO_TURBO, "w") as f:
                    f.write("1")
            try:
                r = subprocess.run(cmd, capture_output=True, text=True, env=ENV)
            finally:
                if turbo_was is not None:
                    with open(NO_TURBO, "w") as f:
                        f.write(turbo_was)
            extra = {
                "cell": ci,
                "block": b,
                "order": o,
                "position": position,
                "prev_k": prev,
                **{k: v for k, v in c.items() if k != "treat"},
            }
            position += 1
            prev = c["k"]
            if r.returncode != 0 or not r.stdout.strip():
                rec = {"kind": "failed", "stderr": r.stderr[-2000:], **extra}
            else:
                rec = json.loads(r.stdout.strip().splitlines()[-1])
                rec.update(extra)
            runs.write(json.dumps(rec, separators=(",", ":")) + "\n")
            runs.flush()
        print(f"block {b} done, {time.time() - t0:.0f} s", flush=True)
    shutil.rmtree(tmp, ignore_errors=True)


TCRIT = {1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365, 8: 2.306, 9: 2.262,
         10: 2.228, 11: 2.201, 15: 2.131}  # fmt: skip


def tcrit(df):
    return TCRIT.get(df, 1.96 if df > 30 else TCRIT[min(TCRIT, key=lambda d: abs(d - df))])


def paced_svc(x, arm):
    """the paced activity's mean exclusive service a visit in a decode (ns): A6 as one"""
    if arm in ("A6",):
        t = sum(excl(x, "Worker", p)[0] for p in ("A6a", "A6b", "A6c"))
        v = excl(x, "Worker", "A6c")[1]
        return t / v if v else float("nan")
    role = CANDIDATES.get(arm) or ("Worker" if arm in ("A6a", "A6b", "A6c") else None)
    t, v = excl(x, role, arm)
    return t / v if v else float("nan")


def busy(x, role, acts):
    return sum(excl(x, role, c)[0] for c in acts) / x["blocks"]


def interval(vals):
    m = statistics.fmean(vals)
    if len(vals) < 2:
        return m, float("nan"), float("nan")
    hw = tcrit(len(vals) - 1) * statistics.stdev(vals) / len(vals) ** 0.5
    return m, m - hw, m + hw


def report(a) -> int:
    import math

    p = json.load(open(a.plan))
    recs = [x for x in load(a.runs) if x.get("kind") in ("decode", "failed")]
    void = []
    if not any("i9-10900K" in m for m in re.findall(r"Model name:\s*(.+)", open(a.lscpu).read())):
        void.append("not the i9-10900K")
    for x in recs:
        if x["kind"] == "failed":
            void.append(f"a decode failed: {x.get('section')} {x.get('arm')} k {x.get('k')}")
        elif x.get("bytes") != a.out_bytes:
            void.append(f"{x['section']} {x['arm']} k {x['k']}: {x.get('bytes')} bytes")
        elif x.get("majflt", 0) > 0:
            void.append(f"{x['section']} {x['arm']} k {x['k']}: major faults")
    got = {}
    for x in recs:
        if x["kind"] == "decode":
            got.setdefault((x["cell"], x["block"]), []).append(x)
    for ci in range(len(p["cells"])):
        for b in range(a.blocks):
            if len(got.get((ci, b), [])) != 1:
                void.append(f"cell {ci} block {b}: {len(got.get((ci, b), []))} decodes")
    incomplete = [
        f"{x['section']} {x['arm']} k {x['k']} block {x['block']}"
        for x in recs
        if x["kind"] == "decode" and not x.get("identities_ok")
    ]
    if void:
        print(f"VOID ({len(void)} problems); no report")
        for v in void[:30]:
            print(f"  {v}")
        return 1
    dec = {k: v[0] for k, v in got.items()}
    cell = {i: c for i, c in enumerate(p["cells"])}

    def find(section, k, arm):
        return next(
            (
                i
                for i, c in cell.items()
                if c["section"] == section and c["k"] == k and c["arm"] == arm
            ),
            None,
        )

    print("# bench-character-stage2 (stage 2a; diagnostic: decides nothing)\n")
    print(
        f"k* {p['kstar']}, the step {p['step']}, anchors {p['anchors']}; {p['cycles_per_iter']:.3f} cycles a pacing iteration; {a.blocks} blocks"
    )
    if incomplete:
        print(f"INCOMPLETE: {len(incomplete)} decodes fail an identity: {incomplete[:10]}")
    rows = []
    misses = []
    print(
        "\n## The screen: each coordinate's coefficient (+20%, one at a time; C = mean log(T/T0) / log(1 + achieved dose))\n"
    )
    print("k   coordinate  dose target/achieved   C      95% interval        label")
    for k in p["anchors"]:
        base = find("screen", k, "base")
        for c in (
            p["coords"][str(k)]["kept"]
            if isinstance(next(iter(p["coords"])), str)
            else p["coords"][k]["kept"]
        ):
            ci = find("screen", k, c)
            achieved, coefs = [], []
            for b in range(a.blocks):
                x, x0 = dec[(ci, b)], dec[(base, b)]
                d = paced_svc(x, c) / paced_svc(x0, c) - 1
                achieved.append(d)
            d_ach = med(achieved)
            for b in range(a.blocks):
                x, x0 = dec[(ci, b)], dec[(base, b)]
                coefs.append(
                    math.log(x["wall_s"] / x0["wall_s"]) / math.log(1 + d_ach)
                    if d_ach > 0
                    else float("nan")
                )
            m, lo, hi = interval(coefs)
            label = (
                "CONTROLS"
                if lo > 0.10
                else "NEGLIGIBLE"
                if lo >= -0.10 and hi <= 0.10
                else "UNRESOLVED"
            )
            if abs(d_ach / DELTA - 1) > 0.25:
                misses.append(f"k {k} {c}: achieved {d_ach:+.3f} against {DELTA:+.2f}")
                label += " DOSE-MISSED"
            rows.append(
                {
                    "section": "screen",
                    "k": k,
                    "coord": c,
                    "target": DELTA,
                    "achieved": d_ach,
                    "C": m,
                    "lo": lo,
                    "hi": hi,
                    "label": label,
                }
            )
            print(
                f"{k:<3} {c:<11} {DELTA:+.2f}/{d_ach:+.3f}            {m:+.3f}  [{lo:+.3f}, {hi:+.3f}]   {label}"
            )
    print(
        "\nthe controllers, by anchor: "
        + "; ".join(
            f"{k}: {[r['coord'] for r in rows if r['k'] == k and r['label'] == 'CONTROLS']}"
            for k in p["anchors"]
        )
    )
    # the surface
    ks = p["kstar"]
    s0 = find("surface", ks, "s00w00")
    if s0 is not None:
        print("\n## The surface at k* (mean log slowdown against (0, 0), over blocks)\n")
        slow = {}
        for sd in (0, 10, 20):
            line = []
            for wd in (0, 10, 20):
                ci = find("surface", ks, f"s{sd:02d}w{wd:02d}")
                v = [
                    math.log(dec[(ci, b)]["wall_s"] / dec[(s0, b)]["wall_s"])
                    for b in range(a.blocks)
                ]
                slow[(sd, wd)] = v
                line.append(f"{statistics.fmean(v):+.4f}")
            print(f"scanner {sd:>2}%: " + "  ".join(line))
        rnd = random.Random(a.seed)

        def roundness(idx):
            s = lambda key: statistics.fmean(slow[key][i] for i in idx)  # noqa: E731
            den = s((20, 0)) + s((0, 20))
            return s((20, 20)) / den if den else float("nan")

        r0 = roundness(range(a.blocks))
        boot = sorted(
            roundness([rnd.randrange(a.blocks) for _ in range(a.blocks)]) for _ in range(2000)
        )
        print(
            f"the knee's roundness: {r0:.3f} [{boot[50]:.3f}, {boot[1949]:.3f}] (1 additive, about 0.5 a sharp max)"
        )
    # the contrasts
    print("\n## The contrasts (each arm's wall over its section's reference, paired by block)\n")
    refs = {"gates": "W4k_cap12", "memory": "none", "placement": "distinct", "scanner": "unpinned"}
    for section, ref in refs.items():
        for k in sorted({c["k"] for c in cell.values() if c["section"] == section}):
            r_ci = find(section, k, ref)
            if r_ci is None:
                continue
            for ci, c in cell.items():
                if c["section"] != section or c["k"] != k or ci == r_ci:
                    continue
                v = [
                    math.log(dec[(ci, b)]["wall_s"] / dec[(r_ci, b)]["wall_s"])
                    for b in range(a.blocks)
                ]
                m, lo, hi = interval(v)
                tag = "CHANGED" if lo > 0 or hi < 0 else ""
                extra = ""
                if section == "gates":
                    w = [
                        dec[(ci, b)].get("admission", {}).get("waits", [0, 0, 0])
                        for b in range(a.blocks)
                    ]
                    extra = f"  waits window/cap/both {[sum(x[i] for x in w) for i in range(3)]}"
                print(
                    f"{section:<9} k {k:<3} {c['arm']:<11} over {ref:<10} {math.exp(m):.4f} [{math.exp(lo):.4f}, {math.exp(hi):.4f}] {tag}{extra}"
                )
                rows.append(
                    {
                        "section": section,
                        "k": k,
                        "coord": c["arm"],
                        "ratio": math.exp(m),
                        "lo": math.exp(lo),
                        "hi": math.exp(hi),
                        "label": tag,
                    }
                )
    # the oracle arms (the four-arm binary's own A, B and C at the knee; §5's decision)
    if a.oracle and os.path.exists(a.oracle):
        print(
            "\n## The oracle arms (the four-arm binary at k*, its rounds paired; ratios against its own A)\n"
        )
        o = {}
        for line in open(a.oracle).read().splitlines()[1:]:
            f = line.split("\t")
            if f and f[0] == "run":
                o.setdefault((int(f[1]), f[3]), []).append(float(f[5]))
        for arm in ("B", "C", "D"):
            v = [
                math.log(o[(r, arm)][0] / o[(r, "A")][0])
                for (r, x) in o
                if x == arm and (r, "A") in o
            ]
            if v:
                m, lo, hi = interval(v)
                print(
                    f"{arm}/A {math.exp(m):.4f} [{math.exp(lo):.4f}, {math.exp(hi):.4f}] over {len(v)} rounds"
                )
        a_walls = [w[0] for (r, x), w in o.items() if x == "A"]
        base = find("screen", ks, "base")
        if a_walls and base is not None:
            ours = statistics.median(dec[(base, b)]["wall_s"] for b in range(a.blocks))
            gap = statistics.median(a_walls) / ours - 1
            print(
                f"the layout check: the four-arm binary's A against this binary's baseline at k*: {gap:+.2%}"
                + ("  (over 3%: the oracle ratios VOID)" if abs(gap) > 0.03 else "")
            )
    # validation (held out: reported, never fitted)
    print("\n## Validation (held out from every fit)\n")
    for ci, c in cell.items():
        if c["section"] == "validation" and c["arm"] != "base":
            ref = (
                find("screen", c["k"], "base")
                if c["arm"] != "permuted"
                else find("validation", c["k"], "base")
            )
            if ref is None:
                continue
            v = [
                math.log(dec[(ci, b)]["wall_s"] / dec[(ref, b)]["wall_s"]) for b in range(a.blocks)
            ]
            m, lo, hi = interval(v)
            print(
                f"k {c['k']:<3} {c['arm']:<10} {math.exp(m):.4f} [{math.exp(lo):.4f}, {math.exp(hi):.4f}]"
            )
    if misses:
        print(
            "\nDOSE-MISSED (achieved more than 25% from target; the coefficient uses the achieved dose): "
            + "; ".join(misses)
        )
    with open(os.path.join(a.out, "stage2.tsv"), "w") as f:
        cols = sorted({k for r in rows for k in r})
        f.write("\t".join(cols) + "\n")
        for r in rows:
            f.write("\t".join(str(r.get(c, "")) for c in cols) + "\n")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    pp = sub.add_parser("plan")
    pp.add_argument("stage1")
    pp.add_argument("calibration")
    pp.add_argument("lscpu")
    pp.add_argument("lscpu_e")
    pp.add_argument("out")
    pr = sub.add_parser("run")
    pr.add_argument("plan")
    pr.add_argument("--bin", required=True)
    pr.add_argument("--corpus", required=True)
    pr.add_argument("--out", required=True)
    pr.add_argument("--blocks", type=int, default=6)
    pr.add_argument("--settle", type=float, default=1.0)
    pr.add_argument("--prewarm", type=float, default=50.0)
    pr.add_argument("--seed", type=int, default=20261125)
    rp = sub.add_parser("report")
    rp.add_argument("plan")
    rp.add_argument("runs")
    rp.add_argument("--lscpu", required=True)
    rp.add_argument("--out", required=True)
    rp.add_argument("--blocks", type=int, default=6)
    rp.add_argument("--seed", type=int, default=20261125)
    rp.add_argument("--out-bytes", type=int, default=4 * 209_700_196)
    rp.add_argument("--oracle", help="the four-arm binary's runs.tsv at k*")
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
