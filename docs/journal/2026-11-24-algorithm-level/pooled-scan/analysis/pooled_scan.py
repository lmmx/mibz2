#!/usr/bin/env python3
"""The pooled scan's kill test, scored as ../registration.md §4 freezes it, with §5's reading of
the floor run and §7's clarifications, all written before the data (RULES C1a: a diagnostic; it
stops or continues the thread, and adopts nothing).

    python3 pooled_scan.py PLAN.json RUNS.jsonl --finder FINDER.json --exact EXACT.txt \\
        --lscpu LSCPU.txt --out DIR [--blocks 12]

Its input is bench-pooled-scan's bundle: the plan (bench/pooled_scan.py), runs.jsonl
(examples/character.rs `--one`, one record a decode), K4's finder pass (examples/pooled_scan.rs
`--finder`) and the exactness phase's result (tests/pooled_scan.rs on the machine).

**The verdict** at the judged counts, 8 and 9 workers (§5: the floor run's Residual A does not
rise from 8 to 10), each ratio the geometric mean of the blocks' paired ratios with its 95%
t-interval. KILL if any of these holds at 8 or at 9:
- **K1:** the upper bound of PC/C exceeds the square root of SC/C's point;
- **K2:** the lower bound of P/none exceeds 1.03;
- and at any count from 6 to 10, **K3:** the median peak RSS of P over none's exceeds 1.10;
- **K4:** the finder's median over its 11 passes exceeds 10 ms.
CONTINUE otherwise.

**VOID** (exit 1): the exactness phase not passed; not the i9-10900K; a decode missing,
duplicated or failed; an output not the registered size, or a check pass not byte-identical;
a pooled arm's timed decode at 2 or more workers not wholly pooled (any fallback, degrade or
dropped candidate on wikidata x4, where every magic is real), or at 1 worker not the shipped
path (anything dispatched), or an unpooled arm with anything dispatched; K4's pass missing,
not pinned to its CPU, or not finding every block; K1's or K2's bound undefined at a judged
count (fewer than 2 blocks: a NaN compares false, and would pass silently).

**INCOMPLETE:** a trace fails an identity that applies to it. In the pooled arms at 2 or more
workers the block-indexed identities are exempt (§7.1): the pooled path records no block's
points, so the flow, linear-order, forced-flow, conservation, admission, Little's-law and
utilisation checks have no blocks to count. The thread and timeline identities apply to every
trace. INCOMPLETE withholds the traces' reported figures; the verdict, from the timed walls,
stands.

**Reported, deciding nothing:** the capture behind C, log(C/PC) / log(C/SC), capped at 1 (§5:
the floor run's Residual B is under 10%), the raw figure beside it where over; P/none at every
count; P's wall at 8 and 9 beside Antigravity's 172-175 ms; CPU; the candidates dropped, the
fallbacks and the degrades per decode; the scan's CPU (the workers' A2 + A3 in P against the
scanner's in none) and each worker's idle share of the wall (its job_wait); the chain thread's
copy (pool_copy) beside its estimate of about 5 ms, its finder (pool_find) and its wait for the
workers' reads (pool_wait); the scanner-to-worker queue's symbol bytes, which P doesn't queue.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import statistics
import sys

sys.path.insert(
    0,
    os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "..", "ideal-parts", "results", "analysis"
    ),
)
from ideal_parts import gm, interval  # noqa: E402

ARMS = ("none", "P", "C", "PC", "SC", "S")
POOLED = ("P", "PC")
JUDGED = (8, 9)
K2_MAX = 1.03
K3_MAX = 1.10
K3_COUNTS = (6, 7, 8, 9, 10)
K4_MAX_NS = 10e6
OUT_BYTES = 4 * 209_700_196
# Antigravity's prediction for P at 8-9 workers (IDEAL-PARTS-2 §5), ms: reported, unscored
PREDICT_P = (172.0, 175.0)
# the chain's copy, estimated before the data (prototype.md, departure 2), ms
COPY_ESTIMATE_MS = 5.0
# the capture's cap (§5: Residual B under 10% of the shipped wall at 8)
CAPTURE_CAP = 1.0
# the identities that count blocks: exempt in a pooled arm's trace at 2+ workers (§7.1)
BLOCK_INDEXED = (
    "flow",
    "linear order",
    "forced flow",
    "conservation",
    "admission",
    "Little's law",
    "utilisation",
)


def identity_key(name):
    return name.split(":")[0].strip()


def busy(x, roles, acts):
    """A trace's exclusive time in the named activities over the threads of `roles` (ns); an
    activity the binary lacks counts 0"""
    names = x.get("activities") or []
    idx = [names.index(a) for a in acts if a in names]
    return sum(
        sum(p[i] for p in t["states"] for i in idx) for t in x["threads"] if t["role"] in roles
    )


def per_thread(x, role, acts):
    """each thread of `role`: (its activities' time, its life), ns"""
    names = x.get("activities") or []
    idx = [names.index(a) for a in acts if a in names]
    return [
        (sum(p[i] for p in t["states"] for i in idx), t["life_ns"])
        for t in x["threads"]
        if t["role"] == role
    ]


def is_pooled(arm, k):
    return arm in POOLED and k >= 2


def analyse(p, recs, finder, exact, lscpu_text, blocks, out_bytes=OUT_BYTES, out=sys.stdout):
    pr = lambda *s: print(*s, file=out)  # noqa: E731
    cells = p["cells"]
    tb = p.get("trace_blocks", 3)
    void, incomplete = [], []
    if not re.search(r"\bPASSED\b", exact or ""):
        void.append(f"the exactness phase did not pass ({(exact or 'missing').strip()[:200]})")
    if not any("i9-10900K" in m for m in re.findall(r"Model name:\s*(.+)", lscpu_text)):
        void.append("not the i9-10900K")
    if {c["arm"] for c in cells if c["section"] == "kill"} != set(ARMS):
        void.append(f"the plan's arms are not {ARMS}")
    got = {}
    for x in recs:
        if x.get("kind") == "failed":
            why = " (timeout)" if x.get("timeout") else ""
            void.append(
                f"a decode failed{why}: {x.get('arm')} k {x.get('k')} block {x.get('block')}"
            )
            continue
        if x.get("kind") != "decode":
            continue
        arm, k, on = x.get("arm"), x.get("k"), x.get("on_arm", x.get("on"))
        where = f"{arm} k {k} block {x.get('block')}{' trace' if on else ''}"
        if x.get("bytes") != out_bytes:
            void.append(f"{where}: {x.get('bytes')} bytes, not {out_bytes}")
        cp = x.get("check_pass") or {}
        if cp.get("identical") is not True:
            void.append(f"{where}: the check pass is not byte-identical")
        st = x.get("pooled") or {}
        if is_pooled(arm, k):
            if not st.get("accepted") or any(
                st.get(f) for f in ("fallbacks", "degraded", "rejected")
            ):
                void.append(f"{where}: not wholly pooled ({st})")
        elif st.get("dispatched"):
            void.append(f"{where}: candidates dispatched off the pooled path ({st})")
        got.setdefault((x["cell"], x["block"], on), []).append(x)
        if on == 1:
            bad = [
                i["name"]
                for i in x.get("identities", [])
                if not i["ok"]
                and not (is_pooled(arm, k) and identity_key(i["name"]) in BLOCK_INDEXED)
            ]
            if bad or "identities" not in x:
                incomplete.append(f"{where}: {[identity_key(n) for n in bad] or 'none recorded'}")
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

    def timed(k, arm):
        ci = find.get((k, arm))
        return [dec[(ci, b, 0)] for b in range(blocks) if (ci, b, 0) in dec]

    def traces(k, arm):
        ci = find.get((k, arm))
        return [dec[(ci, b, 1)] for b in range(tb) if (ci, b, 1) in dec]

    def ratio(k, a, b_):
        """a over b_, paired within the blocks: (gm, lo, hi), or None if a block lacks either"""
        ca, cb = find.get((k, a)), find.get((k, b_))
        if ca is None or cb is None:
            return None
        try:
            v = [
                math.log(dec[(ca, b, 0)]["wall_s"] / dec[(cb, b, 0)]["wall_s"])
                for b in range(blocks)
            ]
        except KeyError:
            return None
        return gm(v)

    # K4's pass
    n_blocks = next((x.get("blocks") for (ci, b, on), x in dec.items() if on == 1 and cells[ci]["arm"] == "none"), None)  # fmt: skip
    if not finder:
        void.append("K4's finder pass is missing")
    else:
        if finder.get("repeats") != 11 or len(finder.get("ns", [])) != 11:
            void.append(f"K4's pass has {finder.get('repeats')} repeats, not 11")
        if finder.get("cpu") != p.get("finder_cpu"):
            void.append(f"K4's pass ran on CPU {finder.get('cpu')}, not {p.get('finder_cpu')}")
        if n_blocks is not None and finder.get("blocks") != n_blocks:
            void.append(f"K4's pass found {finder.get('blocks')} block magics, not {n_blocks}")

    pr("# The pooled scan's kill test (registration.md §4, §5 and §7)\n")
    pr(
        f"counts {counts}; judged {list(JUDGED)}; arms {', '.join(ARMS)}; {blocks} blocks; traces in the first {tb}; unpinned"
    )
    rows = []

    def row(k, measure, v, label=""):
        rows.append(
            {"k": k, "measure": measure, "value": v[0], "lo": v[1], "hi": v[2], "label": label}
        )

    f = lambda v, d=4: f"{v[0]:.{d}f} [{v[1]:.{d}f}, {v[2]:.{d}f}]"  # noqa: E731
    kills = []
    pr("\n## K1 and K2 (95% intervals over the blocks)\n")
    pr(
        "k    PC/C                         SC/C                         sqrt(SC/C)   capture   P/none"
    )
    for k in counts:
        pn = ratio(k, "P", "none")
        if k == 1:
            if pn:
                pr(
                    f"{k:<4} {'':<28} {'':<28} {'':<12} {'':<9} {f(pn)}   (the shipped path: both arms the shipped scanner)"
                )
                row(k, "P/none", pn)
            continue
        pcc, scc = ratio(k, "PC", "C"), ratio(k, "SC", "C")
        if not (pn and pcc and scc):
            pr(f"{k:<4} FAILED (an arm's decode missing)")
            continue
        thr = math.sqrt(scc[0])
        cap = math.log(pcc[0]) / math.log(scc[0]) if scc[0] != 1 else float("nan")
        # (§5: the floor run's Residual B, 2.7% of the shipped wall at 8, is under 10%, so the
        # scanner queue's removal is not a second source: the capture is capped at 1)
        shown = f"{min(cap, CAPTURE_CAP):.3f}" + (f" ({cap:.3f})" if cap > CAPTURE_CAP else "")
        k1 = pcc[2] > thr
        k2 = pn[1] > K2_MAX
        # (a bound that is undefined, as with fewer than 2 blocks, compares false: it would pass
        # silently, so at a judged count it is a VOID instead)
        if k in JUDGED and not all(math.isfinite(v) for v in (pcc[2], pn[1], thr)):
            void.append(f"k {k}: K1's or K2's bound is undefined (fewer than 2 blocks?)")
        mark = ""
        if k in JUDGED:
            mark = (
                "  judged:"
                + (" K1" if k1 else "")
                + (" K2" if k2 else "")
                + ("" if k1 or k2 else " passes")
            )
            if k1:
                kills.append(f"K1 at {k}: PC/C's upper bound {pcc[2]:.4f} > sqrt(SC/C) {thr:.4f}")
            if k2:
                kills.append(f"K2 at {k}: P/none's lower bound {pn[1]:.4f} > {K2_MAX}")
        pr(f"{k:<4} {f(pcc):<28} {f(scc):<28} {thr:<12.4f} {shown:<9} {f(pn)}{mark}")
        row(k, "PC/C", pcc, "K1" if k1 and k in JUDGED else "")
        row(k, "SC/C", scc)
        row(k, "sqrt(SC/C)", (thr, thr, thr))
        row(k, "capture capped", (min(cap, CAPTURE_CAP),) * 3)
        row(k, "capture raw", (cap, cap, cap))
        row(k, "P/none", pn, "K2" if k2 and k in JUDGED else "")
        for a, b_ in (("C", "none"), ("S", "none"), ("PC", "none")):
            r = ratio(k, a, b_)
            if r:
                row(k, f"{a}/{b_}", r)

    pr("\n## K3: peak RSS, P over none (medians over the timed decodes)\n")
    for k in counts:
        a, b_ = timed(k, "P"), timed(k, "none")
        if not (a and b_):
            continue
        r = statistics.median(x["peak_kb"] for x in a) / statistics.median(x["peak_kb"] for x in b_)
        k3 = k in K3_COUNTS and r > K3_MAX
        if k3:
            kills.append(f"K3 at {k}: peak RSS P/none {r:.3f} > {K3_MAX}")
        pr(f"k {k:<3} {r:.3f}{'  K3' if k3 else ''}")
        row(k, "peak RSS P/none", (r, r, r), "K3" if k3 else "")

    pr("\n## K4: the finder alone, one thread, pinned (examples/pooled_scan.rs --finder)\n")
    if finder:
        med = finder.get("median_ns", float("nan"))
        lo, hi = finder.get("interval_ns", [float("nan")] * 2)
        k4 = not med <= K4_MAX_NS
        if k4:
            kills.append(f"K4: the finder's median {med / 1e6:.2f} ms > 10 ms")
        pr(
            f"median {med / 1e6:.2f} ms [{lo / 1e6:.2f}, {hi / 1e6:.2f}] "
            f"(order statistics {finder.get('interval_ranks')}, coverage {finder.get('interval_coverage')}), "
            f"{finder.get('gb_s')} GB/s, AVX2 {finder.get('avx2')}, CPU {finder.get('cpu')}; "
            f"{finder.get('blocks')} block and {finder.get('ends')} end magics{'  K4' if k4 else ''}"
        )
        pr(f"each pass, in order, ms: {[round(v / 1e6, 2) for v in finder.get('ns', [])]}")
        row(0, "finder ms", (med / 1e6, lo / 1e6, hi / 1e6), "K4" if k4 else "")

    pr("\n## Reported, deciding nothing\n")
    pr("P's wall at the judged counts, beside Antigravity's 172-175 ms (unscored):")
    for k in JUDGED:
        w = [x["wall_s"] * 1e3 for x in timed(k, "P")]
        if len(w) >= 2:
            v = interval(w)
            pr(f"  k {k}: {f(v, 1)} ms")
            row(k, "P wall ms", v)
    pr(
        "\nper timed decode (medians): wall ms; CPU (user + sys) s; peak RSS MiB; pooled: dropped, fallbacks, degraded"
    )
    for k in counts:
        bits = []
        for a in ARMS:
            xs = timed(k, a)
            if not xs:
                continue
            st = [x.get("pooled") or {} for x in xs]
            s = (
                f"{a} {statistics.median(x['wall_s'] for x in xs) * 1e3:.1f} ms, "
                f"{statistics.median(x['user_s'] + x['sys_s'] for x in xs):.3f} s, "
                f"{statistics.median(x['peak_kb'] for x in xs) / 1024:.0f} MiB"
            )
            if a in POOLED:
                s += (
                    f"; {statistics.median(v.get('rejected', 0) for v in st):.0f}, "
                    f"{statistics.median(v.get('fallbacks', 0) for v in st):.0f}, "
                    f"{statistics.median(v.get('degraded', 0) for v in st):.0f}"
                )
            bits.append(s)
        pr(f"  k {k:<3} " + "\n        ".join(bits))

    if incomplete:
        pr(
            f"\nINCOMPLETE: {len(incomplete)} traces fail an identity that applies (their figures withheld):"
        )
        for v in incomplete[:10]:
            pr(f"  {v}")
    else:
        pr(
            "\nfrom the traces (ms; 95% over the traces; ledgers on, so slower than the timed decodes):"
        )
        for k in counts:
            if k == 1:
                continue
            tn, tp = traces(k, "none"), traces(k, "P")
            if not (tn and tp):
                continue
            scan_n = interval([busy(x, ("Scanner",), ("A2", "A3")) / 1e6 for x in tn])
            scan_p = interval([busy(x, ("Worker",), ("A2", "A3")) / 1e6 for x in tp])
            idle_n = interval(
                [
                    statistics.fmean(w / l for w, l in per_thread(x, "Worker", ("job_wait",)))
                    for x in tn
                ]
            )
            idle_p = interval(
                [
                    statistics.fmean(w / l for w, l in per_thread(x, "Worker", ("job_wait",)))
                    for x in tp
                ]
            )
            copy = interval([busy(x, ("Scanner",), ("pool_copy",)) / 1e6 for x in tp])
            fnd = interval([busy(x, ("Scanner",), ("pool_find",)) / 1e6 for x in tp])
            wt = interval([busy(x, ("Scanner",), ("pool_wait",)) / 1e6 for x in tp])
            q = statistics.fmean(x.get("queue_syms_bytes", 0) for x in tn) / 2**20
            copied = statistics.fmean((x.get("pooled") or {}).get("copied", 0) for x in tp) / 2**20
            pr(
                f"  k {k:<3} the scan: the workers' A2+A3 in P {f(scan_p, 1)} against the scanner's in none {f(scan_n, 1)}"
            )
            pr(
                f"        each worker's idle share (job_wait / life): P {f(idle_p, 3)}, none {f(idle_n, 3)}"
            )
            pr(
                f"        the chain: pool_copy {f(copy, 2)} (estimated about {COPY_ESTIMATE_MS:.0f}), "
                f"pool_find {f(fnd, 2)}, pool_wait {f(wt, 1)}"
            )
            pr(
                f"        bytes: queued as symbols in none {q:.1f} MiB, not queued in P; copied for the workers in P {copied:.1f} MiB"
            )
            row(k, "scan A2+A3 ms P workers", scan_p)
            row(k, "scan A2+A3 ms none scanner", scan_n)
            row(k, "worker idle share P", idle_p)
            row(k, "worker idle share none", idle_n)
            row(k, "pool_copy ms", copy)
            row(k, "pool_find ms", fnd)
            row(k, "pool_wait ms", wt)
    if void:
        verdict = "VOID"
    elif kills:
        verdict = "KILL"
    else:
        verdict = "CONTINUE"
    pr(f"\n## The verdict: {verdict}\n")
    if void:
        pr(f"VOID ({len(void)} problems; the figures above are for diagnosis only):")
        for v in void[:30]:
            pr(f"  {v}")
        if len(void) > 30:
            pr(f"  ... and {len(void) - 30} more")
    for kl in kills:
        pr(f"  {kl}")
    if verdict == "CONTINUE":
        pr(
            "  none of K1-K4 holds at 8 or 9 (K3 at 6-10): the pooled scan goes on, with thread 4's construction work, to an adoption registration"
        )
    return {"VOID": 1, "KILL": 0, "CONTINUE": 0}[verdict], verdict, rows


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("plan")
    ap.add_argument("runs")
    ap.add_argument("--finder", required=True)
    ap.add_argument("--exact", required=True)
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--blocks", type=int, default=12)
    ap.add_argument("--out-bytes", type=int, default=OUT_BYTES)
    a = ap.parse_args(argv)
    p = json.load(open(a.plan))
    recs = [json.loads(line) for line in open(a.runs) if line.strip()]
    finder = None
    if os.path.exists(a.finder):
        lines = [ln for ln in open(a.finder) if ln.strip()]
        finder = json.loads(lines[-1]) if lines else None
    exact = open(a.exact).read() if os.path.exists(a.exact) else ""
    code, verdict, rows = analyse(
        p, recs, finder, exact, open(a.lscpu).read(), a.blocks, a.out_bytes
    )
    with open(os.path.join(a.out, "pooled-scan.tsv"), "w") as fo:
        fo.write("k\tmeasure\tvalue\tlo\thi\tlabel\n")
        for r in rows:
            fo.write(
                f"{r['k']}\t{r['measure']}\t{r['value']:.6f}\t{r['lo']:.6f}\t{r['hi']:.6f}\t{r['label']}\n"
            )
    with open(os.path.join(a.out, "verdict.txt"), "w") as fo:
        fo.write(verdict + "\n")
    return code


if __name__ == "__main__":
    sys.exit(main())
