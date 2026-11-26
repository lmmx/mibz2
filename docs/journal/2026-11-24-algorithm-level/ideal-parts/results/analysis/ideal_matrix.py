#!/usr/bin/env python3
"""The ideal-parts matrix's analysis (all six parts, SECMRD), written before the data, to the frozen
rule in ../../registration.md with its clarifications 1-8 (RULES C1a: diagnostic; it decides
nothing about production). Slice A's analysis (S x E) is ideal_parts.py beside it, unchanged.

    python3 ideal_matrix.py PLAN.json RUNS.jsonl --lscpu LSCPU.txt --out DIR [--blocks 12]

Its input is bench-ideal-parts' bundle: the plan (bench/ideal_parts.py) and runs.jsonl
(examples/character.rs `--one`, one record a decode).

**The arms** (clarification 8): the half-fraction I = +SECMRD (the 32 arms with an even number of
ideal parts), augmented with the 6 singles and the 6 five-part arms. At each count k (the pinned
cells), y = log wall of each arm in each block, the arms paired within a block.

1. **The effects:** Jain's sign table on the 32 fraction arms alone (resolution VI; the 12 added
   arms are never fitted). A part's sign is +1 ideal, -1 real; an interaction's is the product.
   Each effect is q_x = sum(sign_x(a) y_a) / 32 in each block, then the mean over the blocks with
   its 95% t-interval: the 6 main effects (each aliased with a five-factor interaction) and the 15
   two-factor interactions (each aliased with a four-factor one). For a 2^2 this is
   clarification 1's q_PQ = (y0 - y_P - y_Q + y_PQ)/4.
2. **The allocation of variation:** each effect's share of the total sum of squares of log wall
   over the 32 arms and the blocks (SS_x = 32 r q_x^2): the main effects, the two-factor
   interactions, the 10 aliased pairs of three-factor interactions together, and the residual.
3. **The ceiling table,** read directly from the measured arms:
   - a part's **room**, the wall with it alone ideal over all real (exp(y_P - y_none));
   - its room with Q ideal, exp(y_PQ - y_Q), for each other part Q;
   - its **share of the floor**, every other part ideal over all ideal (exp(y_{all but P} -
     y_all)).
4. **The floor:** all ideal over all real (exp(y_SECMRD - y_none)), reported.

**The labels** (registration.md):
- **A part's room at k:**
  - **ROOM** iff its interval's upper bound is at most 0.95;
  - **NO ROOM** iff the interval lies within [0.98, 1.02];
  - **MASKED** iff NO ROOM alone, and for some Q both q_PQ's interval is wholly below 0 and P is
    ROOM with Q ideal (the upper bound of exp(y_PQ - y_Q) at most 0.95);
  - **UNRESOLVED** otherwise.
- **An interaction at k** (q_PQ, the /4-equivalent coefficient; 4 q_PQ beside it):
  - **INTERACTS** iff its interval excludes 0;
  - **ADDITIVE** iff it lies within +-0.01;
  - **UNRESOLVED** otherwise.

**VOID** (the whole run; exit 1) if any of these:
- the machine is not the i9-10900K;
- any cell's timed decode (any block) or trace (the plan's traced blocks) is missing, duplicated
  or failed. A decode whose ideal parts were not wholly replayed fails in the harness;
- any decode's output is not the registered size (the bytes copied to the caller, or under D
  the bytes discarded in order), or its check pass is not byte-identical to the record pass's
  (size and SHA-256 of the blocks in order, every supplied block's bytes against the walk's);
- any decode has a major fault;
- a removed part shows more than 5% of its real busy time in its ideal arms' traces, at any
  count. Its work, the part's own activities alone (clarifications 3 and 5; each handover is its
  own entry): S the scanner's A2 and A3; E the workers' A7; C the workers' A6a, A6b and A6c; M
  the workers' A5; R the workers' A9; D the reader's copy to the caller (copy_out). The mean over
  the ideal arms' traces is compared with the mean over the real arms' traces;
- the pin is broken (a pinned trace's scanner on any CPU but the plan's) in more than 10% of the
  pinned traces.

**INCOMPLETE** if any traced decode fails an identity.

**Reported beside the labels, never part of them:**
- **each oracle's handover** (clarifications 5 and 6), its own ledger entry (hand_S to hand_R) in
  the traces of the part alone ideal: the busy time summed over the part's threads, and that
  over the threads (the scanner's one; the k workers), per decode;
- **the room net of handover,** (wall - handover) / all real, the handover taken as its per-thread
  figure (the part's work is spread over its threads), a geometric mean over the blocks;
- **the stores** (all six in every arm, clarification 7) and peak RSS (within the binary);
- **the clock split** for the single arms and all ideal: the clock ratio against all real
  (cycles over reference cycles) and the wall at a common clock;
- **the unpinned link** (all ideal over all real at 8 unpinned; pinned all real over unpinned).
"""

from __future__ import annotations

import argparse
import itertools
import json
import math
import os
import re
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ideal_parts import clock, gm, interaction_label, interval, room_label  # noqa: E402

FACTORS = ("S", "E", "C", "M", "R", "D")
ALL = "".join(FACTORS)
ACT = [
    "A0", "feed_wait", "in_wait", "in_maint", "A2", "A3", "slot_wait", "issue", "job_wait",
    "A5", "pool", "A6a", "A6b", "A6c", "A7", "A8", "A9", "publish", "A10", "read_wait",
    "lock_state", "lock_jobs", "hand_S", "hand_E", "hand_C", "hand_M", "hand_R", "copy_out",
    "resid",
]  # fmt: skip
# each part's own work in the ledgers: (role, activities); its handover's entry
WORK = {
    "S": ("Scanner", ("A2", "A3")),
    "E": ("Worker", ("A7",)),
    "C": ("Worker", ("A6a", "A6b", "A6c")),
    "M": ("Worker", ("A5",)),
    "R": ("Worker", ("A9",)),
    "D": ("Reader", ("copy_out",)),
}
HAND = {p: (WORK[p][0], f"hand_{p}") for p in "SECMR"}
REMOVED_MAX = 0.05
PIN_BROKEN_MAX = 0.10
OUT_BYTES = 4 * 209_700_196


def name(parts) -> str:
    return "".join(f for f in FACTORS if f in parts) or "none"


def arms_of(sizes):
    return [name(c) for r in sizes for c in itertools.combinations(FACTORS, r)]


FRACTION = arms_of(range(0, len(FACTORS) + 1, 2))
AUGMENT = arms_of((1, len(FACTORS) - 1))
ARMS = FRACTION + AUGMENT
MAIN = list(FACTORS)
PAIRS = ["".join(c) for c in itertools.combinations(FACTORS, 2)]
# the three-factor interactions, one of each aliased pair (I = SECMRD: XYZ = its complement)
TRIPLES = ["".join(c) for c in itertools.combinations(FACTORS, 3) if "S" in c]


def sign(effect, arm):
    """+1 or -1: the product over the effect's parts of +1 ideal, -1 real"""
    s = 1
    for f in effect:
        s *= 1 if f in arm else -1
    return s


def busy(x, role, acts):
    """A decode's exclusive time in the activities over a role's threads (ns)"""
    idx = [ACT.index(a) for a in acts]
    return sum(
        sum(p[i] for p in t["states"] for i in idx) for t in x["threads"] if t["role"] == role
    )


def delivered(x):
    """the decode's output: the bytes copied to the caller, or under D the bytes discarded"""
    return x.get("bytes", 0) + (x.get("replays") or {}).get("D_discarded_bytes", 0)


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
        if delivered(x) != out_bytes:
            void.append(f"{where}: {delivered(x)} bytes delivered, not {out_bytes}")
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
    planned = {c["arm"] for c in cells if c["section"] == "pinned"}
    if set(ARMS) - planned:
        void.append(f"the plan lacks arms {sorted(set(ARMS) - planned)}")

    def traces(k, arm):
        ci = find.get(("pinned", k, arm))
        return [dec[(ci, b, 1)] for b in range(tb) if (ci, b, 1) in dec]

    # the removed parts' busy time, and the pin
    removed = []
    for k in counts:
        for part, (role, acts) in WORK.items():
            real = [busy(x, role, acts) for a in ARMS if part not in a for x in traces(k, a)]
            ideal = [busy(x, role, acts) for a in ARMS if part in a for x in traces(k, a)]
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

    pr("# The ideal-parts matrix (SECMRD; I = +SECMRD augmented): a diagnostic, deciding nothing\n")
    pr(
        f"counts {counts}; {len(ARMS)} arms ({len(FRACTION)} in the fraction, {len(AUGMENT)} added); "
        f"{blocks} blocks; traces in the first {tb}; the scanner pinned to CPU {scpu}"
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
    for k in counts:
        pr(f"  k {k:<3} " + "  ".join(f"{part}: {s:.2%}" for kk, part, s in removed if kk == k))

    rows = []

    def row(k, measure, v, label=""):
        rows.append(
            {"k": k, "measure": measure, "value": v[0], "lo": v[1], "hi": v[2], "label": label}
        )

    def wall(k, arm, b, section="pinned"):
        return dec[(find[(section, k, arm)], b, 0)]["wall_s"]

    # ---- the effects ----
    pr(
        "\n## The effects (Jain's sign table on the 32 fraction arms, log wall; mean over blocks, 95% t-interval)\n"
    )
    table = {}
    for k in counts:
        try:
            y = {a: [math.log(wall(k, a, b)) for b in range(blocks)] for a in ARMS}
        except KeyError as e:
            pr(f"k {k}: FAILED (no decode {e})")
            continue
        n = len(FRACTION)
        q = {e: interval([sum(sign(e, a) * y[a][b] for a in FRACTION) / n for b in range(blocks)])
             for e in MAIN + PAIRS + TRIPLES}  # fmt: skip
        allv = [v for a in FRACTION for v in y[a]]
        mu = statistics.fmean(allv)
        sst = sum((v - mu) ** 2 for v in allv)
        ss = {e: n * blocks * q[e][0] ** 2 for e in q}
        sse = max(0.0, sst - sum(ss.values()))
        share = lambda v: v / sst if sst else float("nan")  # noqa: E731
        pr(f"k {k}: main effects")
        for e in MAIN:
            m, lo, hi = q[e]
            pr(f"  q_{e:<3} {m:+.4f} [{lo:+.4f}, {hi:+.4f}]   variation {share(ss[e]):.1%}")
            row(k, f"q {e}", q[e])
        pr(f"k {k}: two-factor interactions (q; 4q, the difference of differences, beside it)")
        labels = {}
        for e in PAIRS:
            m, lo, hi = q[e]
            lab = interaction_label(lo, hi)
            labels[e] = lab
            pr(f"  q_{e:<3} {m:+.4f} [{lo:+.4f}, {hi:+.4f}]   4q {4 * m:+.4f} [{4 * lo:+.4f}, {4 * hi:+.4f}]"
               f"   {lab:<11}  variation {share(ss[e]):.1%}")  # fmt: skip
            row(k, f"interaction {e[0]}.{e[1]}", q[e], lab)
        pr(
            f"k {k}: variation: main {share(sum(ss[e] for e in MAIN)):.1%}, two-factor "
            f"{share(sum(ss[e] for e in PAIRS)):.1%}, three-factor (aliased pairs) "
            f"{share(sum(ss[e] for e in TRIPLES)):.1%}, residual {share(sse):.1%}"
        )
        table[k] = (y, q)

    # ---- the ceiling table ----
    pr(
        "\n## The ceiling table (measured arms; ratios of wall, geometric means over blocks, 95% intervals)\n"
    )
    pr(
        "k   part  room: alone over all real       label        share of the floor (all but it over all ideal)   room with Q ideal"
    )
    for k, (y, q) in table.items():
        for part in FACTORS:
            alone = gm([y[part][b] - y["none"][b] for b in range(blocks)])
            withq = {}
            for other in FACTORS:
                if other == part:
                    continue
                both = name(part + other)
                withq[other] = gm([y[both][b] - y[other][b] for b in range(blocks)])
            rest = name(set(FACTORS) - {part})
            floor_share = gm([y[rest][b] - y[ALL][b] for b in range(blocks)])
            # MASKED: NO ROOM alone, and some Q with q_PQ wholly below 0 and P ROOM behind Q
            # (slice A's rule, one Q at a time; the first Q that masks it is named)
            lab, behind = room_label(alone, (0, 0, math.inf), 0, 1), ""
            for other, wq in withq.items():
                e = name(part + other)
                if lab == "NO ROOM" and room_label(alone, wq, q[e][1], q[e][2]) == "MASKED":
                    lab, behind = "MASKED", other
                    break
            lab_s = f"MASKED ({behind})" if behind else lab
            pr(
                f"{k:<3} {part:<5} {alone[0]:.4f} [{alone[1]:.4f}, {alone[2]:.4f}]   {lab_s:<12} "
                f"{floor_share[0]:.4f} [{floor_share[1]:.4f}, {floor_share[2]:.4f}]          "
                + " ".join(f"{o}:{w[0]:.3f}" for o, w in withq.items())
            )
            row(k, f"room {part}", alone, lab)
            row(k, f"floor share {part}", floor_share)
            for o, w in withq.items():
                row(k, f"room {part} with {o} ideal", w)
        fl = gm([y[ALL][b] - y["none"][b] for b in range(blocks)])
        pr(
            f"{k:<3} floor {fl[0]:.4f} [{fl[1]:.4f}, {fl[2]:.4f}]   (all ideal over all real: the decoder's residual)"
        )
        row(k, "floor", fl)

    # ---- reported beside the labels ----
    pr("\n## Reported beside the labels (never part of them)\n")
    pr(
        "each oracle's handover (its own ledger entry, in the traces of the part alone ideal) and the room net of it:"
    )
    pr(
        "k   part  handover: summed ms  per thread ms   room (measured)   net of handover (wall - handover per thread) / all real"
    )
    for k, (y, q) in table.items():
        for part in FACTORS:
            alone = gm([y[part][b] - y["none"][b] for b in range(blocks)])
            if part not in HAND:
                pr(f"{k:<3} {part:<5} (no handover: delivery discarded)   room {alone[0]:.4f}")
                continue
            role, act = HAND[part]
            tr = traces(k, part)
            if not tr:
                pr(f"{k:<3} {part:<5} no traces")
                continue
            summed = statistics.fmean(busy(x, role, (act,)) for x in tr) / 1e9
            threads = 1 if role == "Scanner" else k
            per = summed / threads
            try:
                net = gm(
                    [
                        math.log(max(1e-9, wall(k, part, b) - per) / wall(k, "none", b))
                        for b in range(blocks)
                    ]
                )
            except (KeyError, ValueError):
                net = (float("nan"),) * 3
            pr(f"{k:<3} {part:<5} {summed * 1e3:10.1f}  {per * 1e3:14.1f}   {alone[0]:.4f}            "
               f"{net[0]:.4f} [{net[1]:.4f}, {net[2]:.4f}]")  # fmt: skip
            row(k, f"handover {part} (s per thread)", (per, per, per))
            row(k, f"room {part} net of handover", net)
    pr("\nthe clock split against all real: the clock ratio; the wall at a common clock")
    for k in table:
        parts = []
        for a in list(FACTORS) + [ALL]:
            try:
                cl = [math.log(clock(dec[(find[("pinned", k, a)], b, 0)])
                               / clock(dec[(find[("pinned", k, "none")], b, 0)])) for b in range(blocks)]  # fmt: skip
                wl = [math.log(wall(k, a, b) / wall(k, "none", b)) + cl[b] for b in range(blocks)]
                c, w = gm(cl), gm(wl)
                parts.append(f"{a} {c[0]:.3f}/{w[0]:.3f}")
            except (KeyError, ValueError, ZeroDivisionError):
                parts.append(f"{a} n/a")
        pr(f"  k {k:<3} " + "; ".join(parts))
    pr("\nthe stores (every arm holds all six) and peak RSS (medians over the timed decodes):")
    for k in table:
        st, rss = None, {}
        for a in ARMS:
            xs = [
                dec[(find[("pinned", k, a)], b, 0)]
                for b in range(blocks)
                if (find[("pinned", k, a)], b, 0) in dec
            ]
            if xs:
                st = st or xs[0].get("oracle_store")
                rss[a] = statistics.median(x.get("peak_kb", 0) for x in xs) / 1024
        if st:
            mib = lambda v: v / 2**20  # noqa: E731
            pr(f"  k {k:<3} stores: S {mib(st.get('symbol_bytes', 0)):.0f} MiB, E {mib(st.get('code_bytes', 0)):.0f}, "
               f"C {mib((st.get('C') or {}).get('bytes', 0)):.0f}, M {mib((st.get('M') or {}).get('bytes', 0)):.0f}, "
               f"R {mib((st.get('R') or {}).get('bytes', 0)):.0f}; total {mib(st.get('total_bytes', 0)):.0f} MiB")  # fmt: skip
        if rss:
            v = sorted(rss.values())
            pr(f"  k {k:<3} peak RSS: all real {rss.get('none', float('nan')):.0f} MiB, all ideal "
               f"{rss.get(ALL, float('nan')):.0f}, over the arms {v[0]:.0f} to {v[-1]:.0f}")  # fmt: skip
    link = p.get("link") or {}
    lk = link.get("k")
    try:
        u = gm(
            [
                math.log(wall(lk, ALL, b, "link") / wall(lk, "none", b, "link"))
                for b in range(blocks)
            ]
        )
        pin = gm(
            [math.log(wall(lk, "none", b) / wall(lk, "none", b, "link")) for b in range(blocks)]
        )
        pr(f"\nthe unpinned link at {lk}: all ideal over all real {u[0]:.4f} [{u[1]:.4f}, {u[2]:.4f}]; "
           f"pinned all real over unpinned {pin[0]:.4f} [{pin[1]:.4f}, {pin[2]:.4f}]")  # fmt: skip
    except (KeyError, TypeError):
        pr("\nthe unpinned link: not recorded")
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
    with open(os.path.join(a.out, "ideal-matrix.tsv"), "w") as f:
        f.write("k\tmeasure\tvalue\tlo\thi\tlabel\n")
        for r in rows:
            f.write(
                f"{r['k']}\t{r['measure']}\t{r['value']:.6f}\t{r['lo']:.6f}\t{r['hi']:.6f}\t{r['label']}\n"
            )
    return code


if __name__ == "__main__":
    sys.exit(main())
