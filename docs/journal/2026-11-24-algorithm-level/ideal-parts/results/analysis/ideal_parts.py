#!/usr/bin/env python3
"""The ideal-parts matrix's analysis (slice A: S x E), written before the data, to the frozen rule
in ../../registration.md (RULES C1a: diagnostic; it decides nothing about production).

    python3 ideal_parts.py PLAN.json RUNS.jsonl --lscpu LSCPU.txt --out DIR [--blocks 12]

Its input is bench-ideal-parts' bundle: the plan (bench/ideal_parts.py) and runs.jsonl
(examples/character.rs `--one`, one record a decode).

**At each count k** (the pinned cells), y = log wall of each arm (none, S, E, SE) in each block,
the arms paired within a block:

1. **The effects** (Jain's 2^2 sign table on log wall, one value a block, then the mean over the
   blocks with its 95% t-interval):
   - q_S = (-y_none + y_S - y_E + y_SE) / 4, and likewise q_E;
   - the interaction q_SE = (y_none - y_S - y_E + y_SE) / 4. It is negative where a part's
     effect is larger with the other ideal.
2. **The allocation of variation:** each effect's share of the total sum of squares of log wall
   about its mean over the arms and blocks (SS_x = 4 r q_x^2), and the residual's share.
3. **The ceiling table:**
   - a part's **room**, the wall with it alone ideal over all real (exp(y_P - y_none)), with its
     95% interval;
   - its room with the other part ideal (exp(y_PQ - y_Q));
   - its share of the floor, the wall with every other part ideal over all ideal (exp(y_Q - y_PQ)).
4. **The floor:** all ideal over all real (exp(y_SE - y_none)), reported.

**The labels** (registration.md):
- **A part's room at k:**
  - **ROOM** iff its interval's upper bound is at most 0.95;
  - **NO ROOM** iff the interval lies within [0.98, 1.02];
  - **MASKED** iff NO ROOM alone, the interaction's interval wholly below 0, and ROOM with the
    other part ideal;
  - **UNRESOLVED** otherwise.
- **The interaction at k** (on Jain's q_SE, the /4 coefficient; its 4 q_SE, the difference of
  differences, is shown beside it):
  - **INTERACTS** iff its interval (of q_SE) excludes 0;
  - **ADDITIVE** iff it lies within +-0.01;
  - **UNRESOLVED** otherwise.

**VOID** (the whole run; exit 1) if any of these:
- the machine is not the i9-10900K;
- any cell's timed decode (any block) or trace (the plan's traced blocks) is missing, duplicated
  or failed;
- any decode's output is not the registered size, or its check pass is not byte-identical to the
  record pass's (size and CRC-32, every supplied block's bytes against the walk's);
- any decode has a major fault;
- a removed part shows more than 5% of its real busy time in its ideal arm's traces, at any
  count. The scan's work is the scanner's A2 and A3; emission's is the workers' A7. The mean over
  the ideal arm's traces is compared with the mean over the real arms' traces;
- the pin is broken (a pinned trace's scanner on any CPU but the plan's) in more than 10% of the
  pinned traces.

**INCOMPLETE** if any traced decode fails an identity.

**Reported beside the labels, never part of them:**
- the oracle's memory (the stores' bytes) and its handover (delivery and consumption ns, blocks
  replayed);
- the clock split, each arm's clock ratio against all real (cycles over reference cycles), and
  its wall at a common clock;
- the unpinned link (all ideal over all real at 8 unpinned; pinned all real over unpinned);
- each arm's peak RSS.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import statistics
import sys

ARMS = ("none", "S", "E", "SE")
ACT = [
    "A0", "feed_wait", "in_wait", "in_maint", "A2", "A3", "slot_wait", "issue", "job_wait",
    "A5", "pool", "A6a", "A6b", "A6c", "A7", "A8", "A9", "publish", "A10", "read_wait",
    "lock_state", "lock_jobs", "resid",
]  # fmt: skip
# each part's work in the ledgers: (role, activities)
WORK = {"S": ("Scanner", ("A2", "A3")), "E": ("Worker", ("A7",))}
ROOM_UPPER, NO_ROOM = 0.95, (0.98, 1.02)
ADDITIVE = 0.01
REMOVED_MAX = 0.05
PIN_BROKEN_MAX = 0.10
OUT_BYTES = 4 * 209_700_196
TCRIT = {1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365, 8: 2.306,
         9: 2.262, 10: 2.228, 11: 2.201, 12: 2.179, 13: 2.160, 14: 2.145, 15: 2.131, 16: 2.120,
         17: 2.110, 18: 2.101, 19: 2.093, 20: 2.086, 23: 2.069, 31: 2.040}  # fmt: skip


def tcrit(df):
    if df in TCRIT:
        return TCRIT[df]
    return 1.96 if df > 60 else TCRIT[min(TCRIT, key=lambda d: abs(d - df))]


def interval(vals):
    """(mean, lower, upper): the 95% t-interval of the mean; NaNs for fewer than 2 or any
    non-finite value
    """
    if len(vals) < 2 or not all(math.isfinite(v) for v in vals):
        nan = float("nan")
        return (
            (statistics.fmean(vals) if vals and all(math.isfinite(v) for v in vals) else nan),
            nan,
            nan,
        )
    m = statistics.fmean(vals)
    hw = tcrit(len(vals) - 1) * statistics.stdev(vals) / math.sqrt(len(vals))
    return m, m - hw, m + hw


def gm(vals):
    """A ratio from paired logs: (geometric mean, lower, upper)"""
    m, lo, hi = interval(vals)
    return math.exp(m), math.exp(lo), math.exp(hi)


def busy(x, role, acts):
    """A decode's exclusive time in the activities over a role's threads (ns)"""
    idx = [ACT.index(a) for a in acts]
    return sum(
        sum(p[i] for p in t["states"] for i in idx) for t in x["threads"] if t["role"] == role
    )


def clock(x):
    pm = x.get("pmu") or {}
    c, r = pm.get("cycles"), pm.get("ref-cycles")
    return c[0] / r[0] if c and r and r[0] else float("nan")


def room_label(r, with_other, q_lo, q_hi):
    g, lo, hi = r
    if hi <= ROOM_UPPER:
        return "ROOM"
    no_room = NO_ROOM[0] <= lo and hi <= NO_ROOM[1]
    if no_room and q_hi < 0 and with_other[2] <= ROOM_UPPER:
        return "MASKED"
    if no_room:
        return "NO ROOM"
    return "UNRESOLVED"


def interaction_label(q_lo, q_hi):
    if q_lo > 0 or q_hi < 0:
        return "INTERACTS"
    if -ADDITIVE <= q_lo and q_hi <= ADDITIVE:
        return "ADDITIVE"
    return "UNRESOLVED"


def analyse(p, recs, lscpu_text, blocks, out_bytes=OUT_BYTES, out=sys.stdout):
    pr = lambda *s: print(*s, file=out)  # noqa: E731
    cells = p["cells"]
    tb = p.get("trace_blocks", 2)
    void, incomplete = [], []
    if not any("i9-10900K" in m for m in re.findall(r"Model name:\s*(.+)", lscpu_text)):
        void.append("not the i9-10900K")
    got = {}
    for x in recs:
        if x.get("kind") == "failed":
            void.append(
                f"a decode failed: {x.get('section')} {x.get('arm')} k {x.get('k')} block {x.get('block')}"
            )
            continue
        if x.get("kind") != "decode":
            continue
        where = f"{x.get('section')} {x.get('arm')} k {x.get('k')} block {x.get('block')}"
        if x.get("bytes") != out_bytes:
            void.append(f"{where}: {x.get('bytes')} bytes, not {out_bytes}")
        cp = x.get("check_pass") or {}
        if cp.get("identical") is not True:
            void.append(f"{where}: the check pass is not byte-identical ({cp or 'missing'})")
        if x.get("majflt", 0) > 0:
            void.append(f"{where}: major faults")
        got.setdefault((x["cell"], x["block"], x.get("on_arm", x.get("on"))), []).append(x)
        if x.get("on") == 1 and not x.get("identities_ok"):
            incomplete.append(where)
    for ci, c in enumerate(cells):
        for b in range(blocks):
            for on in (0, 1) if b < tb else (0,):
                n = len(got.get((ci, b, on), []))
                if n != 1:
                    void.append(
                        f"{c['section']} {c['arm']} k {c['k']} block {b} {'trace' if on else 'timed'}: {n} decodes"
                    )
    dec = {key: v[0] for key, v in got.items() if len(v) == 1}
    find = {(c["section"], c["k"], c["arm"]): ci for ci, c in enumerate(cells)}
    counts = sorted({c["k"] for c in cells if c["section"] == "pinned"})

    # the removed parts' busy time, and the pin
    removed = []
    for k in counts:
        for part, (role, acts) in WORK.items():
            real = [busy(dec[(find[("pinned", k, a)], b, 1)], role, acts) for a in ARMS if part not in a
                    for b in range(tb) if (find[("pinned", k, a)], b, 1) in dec]  # fmt: skip
            ideal = [busy(dec[(find[("pinned", k, a)], b, 1)], role, acts) for a in ARMS if part in a
                     for b in range(tb) if (find[("pinned", k, a)], b, 1) in dec]  # fmt: skip
            if real and ideal and statistics.fmean(real) > 0:
                share = statistics.fmean(ideal) / statistics.fmean(real)
                removed.append((k, part, share))
                if share > REMOVED_MAX:
                    void.append(
                        f"k {k}: {part}'s work ({'+'.join(acts)}) is {share:.1%} of its real busy time in its ideal arms"
                    )
    scpu = p.get("scanner_cpu")
    pinned_traces = [x for (ci, b, on), x in dec.items() if on == 1 and cells[ci].get("pinned")]
    broken = [x for x in pinned_traces
              if set(next((t.get("cpus", {}) for t in x["threads"] if t["role"] == "Scanner"), {})) - {str(scpu)}]  # fmt: skip
    if pinned_traces and len(broken) > PIN_BROKEN_MAX * len(pinned_traces):
        void.append(
            f"the pin broken in {len(broken)} of {len(pinned_traces)} pinned traces (over 10%)"
        )

    pr("# The ideal-parts matrix, slice A (S x E): a diagnostic, deciding nothing\n")
    pr(
        f"counts {counts}; {blocks} blocks; traces in the first {tb}; the scanner pinned to CPU {scpu}"
    )
    pr(
        f"the pin: the scanner on CPU {scpu} alone in {len(pinned_traces) - len(broken)} of {len(pinned_traces)} pinned traces"
    )
    if void:
        pr(f"\nVOID ({len(void)} problems): the figures below are for diagnosis only")
        for v in void[:30]:
            pr(f"  {v}")
        if len(void) > 30:
            pr(f"  ... and {len(void) - 30} more")
    if incomplete:
        pr(f"\nINCOMPLETE: {len(incomplete)} traced decodes fail an identity: {incomplete[:10]}")
    pr(
        "\nthe removed parts' busy time in their ideal arms' traces, over the real arms' (VOID over 5%):"
    )
    for k, part, share in removed:
        pr(f"  k {k:<3} {part}: {share:.2%}")

    rows = []

    def wall(k, arm, b, section="pinned"):
        return dec[(find[(section, k, arm)], b, 0)]["wall_s"]

    pr("\n## The effects (Jain's 2^2 sign table on log wall; mean over blocks, 95% t-interval)\n")
    pr(
        "k   q_S                         q_E                         q_SE (interaction)          4 q_SE (the difference of differences)  label         variation: S / E / SE / residual"
    )
    table = {}
    for k in counts:
        try:
            y = {a: [math.log(wall(k, a, b)) for b in range(blocks)] for a in ARMS}
        except KeyError as e:
            pr(f"{k:<3} FAILED (no decode {e})")
            continue
        qS = [(-y["none"][b] + y["S"][b] - y["E"][b] + y["SE"][b]) / 4 for b in range(blocks)]
        qE = [(-y["none"][b] - y["S"][b] + y["E"][b] + y["SE"][b]) / 4 for b in range(blocks)]
        qI = [(y["none"][b] - y["S"][b] - y["E"][b] + y["SE"][b]) / 4 for b in range(blocks)]
        iS, iE, iI = interval(qS), interval(qE), interval(qI)
        allv = [v for a in ARMS for v in y[a]]
        mu = statistics.fmean(allv)
        sst = sum((v - mu) ** 2 for v in allv)
        ss = {n: 4 * blocks * q[0] ** 2 for n, q in (("S", iS), ("E", iE), ("SE", iI))}
        sse = max(0.0, sst - sum(ss.values()))
        share = {n: v / sst if sst else float("nan") for n, v in {**ss, "residual": sse}.items()}
        lab = interaction_label(iI[1], iI[2])
        pr(
            f"{k:<3} "
            + "  ".join(f"{m:+.4f} [{lo:+.4f}, {hi:+.4f}]" for m, lo, hi in (iS, iE, iI))
            + f"  {4 * iI[0]:+.4f} [{4 * iI[1]:+.4f}, {4 * iI[2]:+.4f}]{'':<15}"
            + f"  {lab:<13} "
            + " / ".join(f"{share[n]:.1%}" for n in ("S", "E", "SE", "residual"))
        )
        rows.append(
            {
                "k": k,
                "measure": "interaction S.E",
                "value": iI[0],
                "lo": iI[1],
                "hi": iI[2],
                "label": lab,
            }
        )
        table[k] = (y, iI)

    pr("\n## The ceiling table (ratios of wall, geometric means over blocks, 95% intervals)\n")
    pr(
        "k   part  room: alone over all real       label        with the other ideal            share of the floor (all but it ideal over all ideal)"
    )
    for k, (y, iI) in table.items():
        for part, other in (("S", "E"), ("E", "S")):
            alone = gm([y[part][b] - y["none"][b] for b in range(blocks)])
            both = "SE"
            withq = gm([y[both][b] - y[other][b] for b in range(blocks)])
            floor_share = gm([y[other][b] - y[both][b] for b in range(blocks)])
            lab = room_label(alone, withq, iI[1], iI[2])
            pr(
                f"{k:<3} {part:<5} {alone[0]:.4f} [{alone[1]:.4f}, {alone[2]:.4f}]   {lab:<12} "
                f"{withq[0]:.4f} [{withq[1]:.4f}, {withq[2]:.4f}]   {floor_share[0]:.4f} [{floor_share[1]:.4f}, {floor_share[2]:.4f}]"
            )
            rows.append(
                {
                    "k": k,
                    "measure": f"room {part}",
                    "value": alone[0],
                    "lo": alone[1],
                    "hi": alone[2],
                    "label": lab,
                }
            )
            rows.append(
                {
                    "k": k,
                    "measure": f"room {part} with {other} ideal",
                    "value": withq[0],
                    "lo": withq[1],
                    "hi": withq[2],
                    "label": "",
                }
            )
        fl = gm([y["SE"][b] - y["none"][b] for b in range(blocks)])
        pr(
            f"{k:<3} floor {fl[0]:.4f} [{fl[1]:.4f}, {fl[2]:.4f}]   (all ideal over all real: the decoder's residual)"
        )
        rows.append(
            {"k": k, "measure": "floor", "value": fl[0], "lo": fl[1], "hi": fl[2], "label": ""}
        )

    pr("\n## Reported beside the labels (never part of them)\n")
    pr("the clock split, each arm against all real: the clock ratio; the wall at a common clock")
    for k in table:
        parts = []
        for a in ("S", "E", "SE"):
            try:
                cl = [
                    math.log(
                        clock(dec[(find[("pinned", k, a)], b, 0)])
                        / clock(dec[(find[("pinned", k, "none")], b, 0)])
                    )
                    for b in range(blocks)
                ]
                wl = [math.log(wall(k, a, b) / wall(k, "none", b)) + cl[b] for b in range(blocks)]
                c, w = gm(cl), gm(wl)
                parts.append(
                    f"{a}: clock {c[0]:.4f} [{c[1]:.4f}, {c[2]:.4f}], common-clock wall {w[0]:.4f}"
                )
            except (KeyError, ValueError, ZeroDivisionError):
                parts.append(f"{a}: not recorded")
        pr(f"  k {k:<3} " + "; ".join(parts))
    pr("the oracle (medians over the timed decodes): its stores, the handover, peak RSS")
    for k in table:
        for a in ARMS:
            xs = [
                dec[(find[("pinned", k, a)], b, 0)]
                for b in range(blocks)
                if (find[("pinned", k, a)], b, 0) in dec
            ]
            if not xs:
                continue
            st = xs[0].get("oracle_store") or {}
            mib = (st.get("symbol_bytes", 0) + st.get("code_bytes", 0)) / 2**20
            pr(
                f"  k {k:<3} {a:<4} store {mib:.1f} MiB, replayed {statistics.median(x.get('replayed', 0) for x in xs):.0f} blocks, "
                f"supplied {statistics.median(x.get('supplied_blocks', 0) for x in xs):.0f}, "
                f"handover {statistics.median((x.get('deliver_ns', 0) + x.get('consume_ns', 0)) / 1e6 for x in xs):.1f} ms, "
                f"peak RSS {statistics.median(x.get('peak_kb', 0) for x in xs) / 1024:.0f} MiB"
            )
    link = p.get("link") or {}
    lk = link.get("k")
    try:
        u = gm(
            [
                math.log(wall(lk, "SE", b, "link") / wall(lk, "none", b, "link"))
                for b in range(blocks)
            ]
        )
        pin = gm(
            [math.log(wall(lk, "none", b) / wall(lk, "none", b, "link")) for b in range(blocks)]
        )
        pr(f"the unpinned link at {lk}: all ideal over all real {u[0]:.4f} [{u[1]:.4f}, {u[2]:.4f}]; "
           f"pinned all real over unpinned {pin[0]:.4f} [{pin[1]:.4f}, {pin[2]:.4f}]")  # fmt: skip
    except (KeyError, TypeError):
        pr("the unpinned link: not recorded")
    if void:
        pr(f"\nVOID ({len(void)} problems: {'; '.join(void[:3])})")
    return (1 if void else 0), rows


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("plan")
    ap.add_argument("runs")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--blocks", type=int, default=12)
    ap.add_argument("--out-bytes", type=int, default=OUT_BYTES)
    a = ap.parse_args(argv)
    p = json.load(open(a.plan))
    recs = [json.loads(line) for line in open(a.runs) if line.strip()]
    code, rows = analyse(p, recs, open(a.lscpu).read(), a.blocks, a.out_bytes)
    with open(os.path.join(a.out, "ideal-parts.tsv"), "w") as f:
        f.write("k\tmeasure\tvalue\tlo\thi\tlabel\n")
        for r in rows:
            f.write(
                f"{r['k']}\t{r['measure']}\t{r['value']:.6f}\t{r['lo']:.6f}\t{r['hi']:.6f}\t{r['label']}\n"
            )
    return code


if __name__ == "__main__":
    sys.exit(main())
