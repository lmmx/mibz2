#!/usr/bin/env python3
"""The zero-copy floor's analysis, written before the data to ../../registration-floor.md (RULES C1a:
diagnostic; it decides nothing, and nothing is adopted).

    python3 zero_copy_floor.py PLAN.json RUNS.jsonl --lscpu LSCPU.txt --out DIR [--blocks 8]

Its input is bench-zero-copy-floor's bundle: the plan (bench/ideal_parts.py plan --design floor) and
runs.jsonl (examples/character.rs `--one`, one record a decode). The arms: `none`; `SECMRD`, every
part ideal with zero-copy handovers (emission's word spelling run); `SFCMRD`, emission in E_full
(the spelling removed too); `SFCMR`, that with delivery real; and SFCMRD with each other part real
in turn (`FCMRD`, `SCMRD`, `SFMRD`, `SFCRD`; all but R is `SECMD`, R's room read against SECMRD,
since E_full removes the spelling only where R replays).

At each count k, the arms paired within a block, each figure with its 95% t-interval:
1. the floor, SECMRD over none (geometric mean of the blocks' ratios);
2. Residual A, the SECMRD wall (ms);
3. Residual B, the plumbing residual, the SFCMRD wall (ms);
4. E_consume on the wall, SECMRD - SFCMRD (ms, paired), and its ledger entry in SECMRD's traces;
5. delivery's share with everything else free, SFCMR - SFCMRD (ms, paired);
6. the handover, each oracle's ledger entry in SECMRD's traces, summed and per thread; the workers'
   per-thread sum and its share of Residual A;
7. each part's room against the zero-copy floor, SFCMRD over the arm with it alone real, labelled
   as registration.md labels a room (ROOM, NO ROOM, UNRESOLVED).

**VOID** (exit 1), registration.md's rules: not the i9-10900K; a decode missing, duplicated or failed
(a timeout, or ideal parts not wholly replayed, included); an output not the registered size or a
check pass not byte-identical; a major fault; a removed part over 5% of its real busy time (the
`none` arm's) in its ideal arms' traces; the pin broken in over 10% of the pinned traces.
**INCOMPLETE** if a traced decode fails an identity.

**The prediction scored:** Antigravity's Residual B at 1 worker, 14-19 ms: a hit iff the measured
interval overlaps [14, 19].
"""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ideal_parts import clock, gm, interval  # noqa: E402

A, B, BD = "SECMRD", "SFCMRD", "SFCMR"
# the all-but-X arms: B with each part real in turn (all-but-D is BD)
ALL_BUT = {"S": "FCMRD", "E": "SCMRD", "C": "SFMRD", "M": "SFCRD", "R": "SECMD", "D": BD}
# each room's floor: SFCMRD, but R's is SECMRD (E_full removes the spelling only where R replays,
# so with R real emission spells: SECMD against SECMRD keeps the spelling on both sides)
FLOOR_OF = {p: (A if p == "R" else B) for p in ALL_BUT}
ARMS = ("none", A, B, BD, *(ALL_BUT[p] for p in "SECMR"))
# registration.md's room labels, here on each part's room against the zero-copy floor
ROOM_UPPER, NO_ROOM = 0.95, (0.98, 1.02)
# the zero-copy oracles' ledger entries for emission's two halves (Antigravity's names; the
# recipe's guard checks the binary has them)
HAND_E_DELIVER = "hand_E_deliver"
E_CONSUME = "E_consume"
# each part's own work: (role, activities); E's spelling is removed in the F arms alone
WORK = {
    "S": ("Scanner", ("A2", "A3")),
    "E": ("Worker", ("A7",)),
    "C": ("Worker", ("A6a", "A6b", "A6c")),
    "M": ("Worker", ("A5",)),
    "R": ("Worker", ("A9",)),
    "D": ("Reader", ("copy_out",)),
}
HAND = {
    "S": ("Scanner", "hand_S"),
    "E": ("Worker", HAND_E_DELIVER),
    "C": ("Worker", "hand_C"),
    "M": ("Worker", "hand_M"),
    "R": ("Worker", "hand_R"),
}
REMOVED_MAX = 0.05
PIN_BROKEN_MAX = 0.10
OUT_BYTES = 4 * 209_700_196
# the prediction (IDEAL-PARTS-2 section 3.1): Residual B at 1 worker, ms
PREDICT_B1 = (14.0, 19.0)


def ideal_in(part, arm):
    """whether `part` is ideal in `arm` (F is emission ideal, in E_full)"""
    return part in arm or (part == "E" and "F" in arm)


def busy(x, role, acts):
    """A trace's exclusive time in the named activities over a role's threads (ns); an activity
    the binary lacks counts 0"""
    names = x.get("activities") or []
    idx = [names.index(a) for a in acts if a in names]
    return sum(
        sum(p[i] for p in t["states"] for i in idx) for t in x["threads"] if t["role"] == role
    )


def visits(x, role, act):
    """A trace's visits to the named activity over a role's threads (0 if the binary lacks it)"""
    names = x.get("activities") or []
    if act not in names:
        return 0
    i = names.index(act)
    return sum(
        t.get("visits", [])[i]
        for t in x["threads"]
        if t["role"] == role and i < len(t.get("visits", []))
    )


def delivered(x):
    return x.get("bytes", 0) + (x.get("replays") or {}).get("D_discarded_bytes", 0)


def ms(vals):
    """(mean, lower, upper) of values in s, as ms"""
    m, lo, hi = interval([v * 1e3 for v in vals])
    return m, lo, hi


def analyse(p, recs, lscpu_text, blocks, out_bytes=OUT_BYTES, out=sys.stdout):
    pr = lambda *s: print(*s, file=out)  # noqa: E731
    cells = p["cells"]
    tb = p.get("trace_blocks", 4)
    void, incomplete = [], []
    if not any("i9-10900K" in m for m in re.findall(r"Model name:\s*(.+)", lscpu_text)):
        void.append("not the i9-10900K")
    got = {}
    for x in recs:
        if x.get("kind") == "failed":
            why = " (timeout)" if x.get("timeout") else ""
            void.append(
                f"a decode failed{why}: {x.get('section')} {x.get('arm')} k {x.get('k')} block {x.get('block')}"
            )
            continue
        if x.get("kind") != "decode":
            continue
        where = f"{x.get('arm')} k {x.get('k')} block {x.get('block')}"
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
                        f"{c['arm']} k {c['k']} block {b} {'trace' if on else 'timed'}: {n} decodes"
                    )
    dec = {key: v[0] for key, v in got.items() if len(v) == 1}
    find = {(c["k"], c["arm"]): ci for ci, c in enumerate(cells)}
    counts = sorted({c["k"] for c in cells})
    if {c["arm"] for c in cells} != set(ARMS):
        void.append(f"the plan's arms are not {ARMS}")

    def traces(k, arm):
        ci = find.get((k, arm))
        return [dec[(ci, b, 1)] for b in range(tb) if (ci, b, 1) in dec]

    def wall(k, arm, b):
        return dec[(find[(k, arm)], b, 0)]["wall_s"]

    # the removed parts' own work, against the all-real arm's
    removed = []
    for k in counts:
        real = traces(k, "none")
        for part, (role, acts) in WORK.items():
            r = [busy(x, role, acts) for x in real]
            i = [
                busy(x, role, acts)
                for a in ARMS
                if a != "none" and ideal_in(part, a)
                for x in traces(k, a)
            ]
            if r and i and statistics.fmean(r) > 0:
                share = statistics.fmean(i) / statistics.fmean(r)
                removed.append((k, part, share))
                if share > REMOVED_MAX:
                    void.append(
                        f"k {k}: {part}'s work ({'+'.join(acts)}) is {share:.1%} of its real busy time in its ideal arms"
                    )
        # E_full removes the spelling: E_consume in the F arms against SECMRD's
        r = [busy(x, "Worker", (E_CONSUME,)) for x in traces(k, A)]
        i = [busy(x, "Worker", (E_CONSUME,)) for a in ARMS if "F" in a for x in traces(k, a)]
        if r and i and statistics.fmean(r) > 0:
            share = statistics.fmean(i) / statistics.fmean(r)
            removed.append((k, "E_consume", share))
            if share > REMOVED_MAX:
                void.append(f"k {k}: E_consume is {share:.1%} of SECMRD's in the E_full arms")
        # F is E_full only where R replays (else the oracle falls back to Supplied and spells):
        # every F arm's traces show no call to the spelling at all
        for a in ARMS:
            if "F" in a:
                n = sum(visits(x, "Worker", E_CONSUME) for x in traces(k, a))
                if n:
                    void.append(
                        f"k {k}: {a}'s traces call E_consume {n} times (E_full not in force)"
                    )
    pinned = [x for (ci, b, on), x in dec.items() if on == 1 and cells[ci].get("pinned")]
    scpu = p.get("scanner_cpu")
    broken = [x for x in pinned
              if set(next((t.get("cpus", {}) for t in x["threads"] if t["role"] == "Scanner"), {})) - {str(scpu)}]  # fmt: skip
    if pinned and len(broken) > PIN_BROKEN_MAX * len(pinned):
        void.append(f"the pin broken in {len(broken)} of {len(pinned)} pinned traces (over 10%)")

    pr("# The zero-copy floor: a diagnostic, deciding nothing\n")
    pr(
        f"counts {counts}; arms {', '.join(ARMS)}; {blocks} blocks; traces in the first {tb}; the scanner pinned to CPU {scpu}"
    )
    pr(
        f"the pin: the scanner on CPU {scpu} alone in {len(pinned) - len(broken)} of {len(pinned)} pinned traces"
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
        "\nthe removed parts' busy time in their ideal arms' traces, over the real arm's (VOID over 5%):"
    )
    for k in counts:
        pr(f"  k {k:<3} " + "  ".join(f"{part}: {s:.2%}" for kk, part, s in removed if kk == k))

    rows = []

    def row(k, measure, v, label=""):
        rows.append(
            {"k": k, "measure": measure, "value": v[0], "lo": v[1], "hi": v[2], "label": label}
        )

    pr("\n## The floor and its parts (95% intervals over the blocks; ms unless a ratio)\n")
    pr(
        "k    floor (SECMRD/none)        Residual A (SECMRD)      Residual B (SFCMRD)      E_consume (A - B)        D's share (SFCMR - SFCMRD)"
    )
    walls = {}
    for k in counts:
        try:
            w = {a: [wall(k, a, b) for b in range(blocks)] for a in ARMS}
        except KeyError as e:
            pr(f"{k:<4} FAILED (no decode {e})")
            continue
        walls[k] = w
        fl = gm([math.log(w[A][b] / w["none"][b]) for b in range(blocks)])
        ra, rb = ms(w[A]), ms(w[B])
        ec = ms([w[A][b] - w[B][b] for b in range(blocks)])
        ds = ms([w[BD][b] - w[B][b] for b in range(blocks)])
        f = lambda v, d=1: f"{v[0]:.{d}f} [{v[1]:.{d}f}, {v[2]:.{d}f}]"  # noqa: E731
        pr(f"{k:<4} {f(fl, 4):<26} {f(ra):<24} {f(rb):<24} {f(ec):<24} {f(ds)}")
        row(k, "floor", fl)
        row(k, "residual A ms", ra)
        row(k, "residual B ms", rb)
        row(k, "E_consume wall ms", ec)
        row(k, "D share ms", ds)

    pr(
        "\n## Each part's room against the zero-copy floor (SFCMRD over all but it ideal; geometric means, 95% intervals)\n"
    )
    pr(
        "registration.md's labels: ROOM iff the upper bound <= 0.95; NO ROOM iff within [0.98, 1.02]; else UNRESOLVED"
    )
    pr("(MASKED needs the pairs, which this run lacks)\n")
    pr("k    " + "".join(f"{p:<36}" for p in ALL_BUT))
    for k, w in walls.items():
        cells_ = []
        for part, arm in ALL_BUT.items():
            r = gm([math.log(w[FLOOR_OF[part]][b] / w[arm][b]) for b in range(blocks)])
            if r[2] <= ROOM_UPPER:
                lab = "ROOM"
            elif NO_ROOM[0] <= r[1] and r[2] <= NO_ROOM[1]:
                lab = "NO ROOM"
            else:
                lab = "UNRESOLVED"
            cells_.append(f"{r[0]:.4f} [{r[1]:.4f}, {r[2]:.4f}] {lab:<10}")
            row(k, f"room {part} against the floor", r, lab)
        pr(f"{k:<4} " + " ".join(cells_))
    pr(
        "(C's room is measured with a cold store: the zero-copy borrow leaves the walk to read C's"
        " store cold, where the copying oracle warmed it; see A7 per arm below)"
    )
    pr("\nthe walk (A7) per worker in each arm's traces, ms (C's cold-store effect shows here):")
    for k in counts:
        bits = []
        for a in ARMS:
            tr = traces(k, a)
            if tr:
                v = interval([busy(x, "Worker", ("A7",)) / 1e6 / k for x in tr])
                bits.append(f"{a} {v[0]:.1f}")
                row(k, f"A7 ms per worker {a}", v)
        pr(f"  k {k:<3} " + "; ".join(bits))
    # (reported only, the Coordinator's split of emission: its walk made free with the spelling
    # kept, SECMRD over SCMRD; the spelling is then SFCMRD over SECMRD)
    pr(
        "\nemission's walk against the floor, the spelling kept (SECMRD over SCMRD; reported only, no label):"
    )
    for k, w in walls.items():
        r = gm([math.log(w[A][b] / w[ALL_BUT["E"]][b]) for b in range(blocks)])
        pr(f"  k {k:<3} {r[0]:.4f} [{r[1]:.4f}, {r[2]:.4f}]")
        row(k, "E walk against SECMRD", r)

    pr(
        "\n## The handover and the spelling in SECMRD's traces (ms; summed over the part's threads, and per thread; 95% over the traces)\n"
    )
    for k in counts:
        tr = traces(k, A)
        if not tr:
            pr(f"k {k}: no traces")
            continue
        parts, per_worker = [], [0.0] * len(tr)
        for part, (role, act) in list(HAND.items()) + [("E_consume", ("Worker", E_CONSUME))]:
            threads = 1 if role == "Scanner" else k
            summed = [busy(x, role, (act,)) / 1e6 for x in tr]
            per = interval([v / threads for v in summed])
            parts.append(
                f"{part} {statistics.fmean(summed):.1f} / {per[0]:.2f} [{per[1]:.2f}, {per[2]:.2f}]"
            )
            row(
                k,
                f"handover {part} ms per thread"
                if part != "E_consume"
                else "E_consume ledger ms per thread",
                per,
            )
            if role == "Worker" and part != "E_consume":
                per_worker = [pw + v / threads for pw, v in zip(per_worker, summed)]
        hw = interval(per_worker)
        a = statistics.fmean(walls[k][A]) * 1e3 if k in walls else float("nan")
        pr(f"k {k:<3} " + "; ".join(parts))
        pr(
            f"      the workers' handovers per thread {hw[0]:.2f} [{hw[1]:.2f}, {hw[2]:.2f}] ms, {hw[0] / a:.1%} of Residual A"
        )
        row(k, "workers' handover ms per thread", hw)

    pr("\n## The prediction, scored\n")
    if 1 in walls:
        rb = ms(walls[1][B])
        lo, hi = PREDICT_B1
        label = "hit" if rb[1] <= hi and rb[2] >= lo else "miss"
        pr(
            f"Antigravity's Residual B at 1 worker, {lo:.0f}-{hi:.0f} ms: measured {rb[0]:.1f} [{rb[1]:.1f}, {rb[2]:.1f}] ms: {label.upper()}"
        )
        row(1, "prediction residual B", rb, label)
    else:
        pr("Residual B at 1 worker: not measured")
    pr(
        "(beside it, unscored: Antigravity's Residual A of about 273, 60-62, 51-55 and 50-54 ms at 1, 6, 8 and 10 workers)"
    )
    pr(
        "(and reported only, not comparable to the i9: Antigravity's 2-core Xeon VM had the SFCMRD wall"
        " at 44, 28 and 21 ms at 1, 4 and 8 workers; measured here "
        + ", ".join(f"{ms(walls[k][B])[0]:.1f} ms at {k}" for k in (1, 8) if k in walls)
        + ")"
    )

    pr("\n## Beside them (medians over the timed decodes)\n")
    for k in counts:
        bits = []
        for a in ARMS:
            xs = [dec[(find[(k, a)], b, 0)] for b in range(blocks) if (find[(k, a)], b, 0) in dec]
            if not xs:
                continue
            g = [clock(x) * 3.7 for x in xs]
            pm = [x.get("pmu") or {} for x in xs]
            llc = [(m.get("LLC-load-misses") or [None])[0] for m in pm]
            llc = [v for v in llc if v is not None]
            imc = [x.get("imc_mib") or {} for x in xs]
            traffic = [sum(v for kk, v in m.items() if kk != "why" and v is not None) for m in imc]
            gbs = [t * 2**20 / x["wall_s"] / 1e9 for t, x in zip(traffic, xs) if t]
            why = next((m["why"] for m in imc if m.get("why")), "")
            fin = [v for v in g if math.isfinite(v)]
            s = f"{a} {statistics.median(x['wall_s'] for x in xs) * 1e3:.1f} ms"
            s += f", {statistics.median(fin):.2f} GHz" if fin else ", clock n/a"
            s += f", LLC load misses {statistics.median(llc) / 1e6:.1f} M" if llc else ", LLC n/a"
            s += (
                f", memory {statistics.median(gbs):.1f} GB/s"
                if gbs
                else f", memory n/a ({why or 'not read'})"
            )
            s += f", peak RSS {statistics.median(x.get('peak_kb', 0) for x in xs) / 1024:.0f} MiB"
            bits.append(s)
        pr(f"k {k:<3} " + "\n      ".join(bits))
    if void:
        pr(f"\nVOID ({len(void)} problems: {'; '.join(void[:3])})")
    return (1 if void else 0), rows


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("plan")
    ap.add_argument("runs")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--blocks", type=int, default=8)
    ap.add_argument("--out-bytes", type=int, default=OUT_BYTES)
    a = ap.parse_args(argv)
    p = json.load(open(a.plan))
    recs = [json.loads(line) for line in open(a.runs) if line.strip()]
    code, rows = analyse(p, recs, open(a.lscpu).read(), a.blocks, a.out_bytes)
    with open(os.path.join(a.out, "zero-copy-floor.tsv"), "w") as f:
        f.write("k\tmeasure\tvalue\tlo\thi\tlabel\n")
        for r in rows:
            f.write(
                f"{r['k']}\t{r['measure']}\t{r['value']:.6f}\t{r['lo']:.6f}\t{r['hi']:.6f}\t{r['label']}\n"
            )
    return code


if __name__ == "__main__":
    sys.exit(main())
