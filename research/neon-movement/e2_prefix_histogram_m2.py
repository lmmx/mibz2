#!/usr/bin/env python3
"""E2 (Principle 2 — In-Stage N^Sigma Semiring Enrichment):
Prefix-Histogram Snapshot Algebra M_2[a, b] (Free 2-Gram Interval Lifting)

In one streaming pass over L (or run records IN, P = 1, Q = 0, s = |Sigma| + 1),
snapshotting the running prefix-histogram vector occ[0..255] in N^Sigma at each
F-bucket boundary C[a] yields the exact 2-gram interval image table:
    I_{ba} = LF(I_a cap L^{-1}(b)) = [C_2[b, a], C_2[b, a] + M_2[a, b]).

MONODROMY record 115 (§4) noted M_2 is free in P=1, Q=0 and asked whether it has
any use beyond macro-edges. Here we formalize and measure FOUR exact uses:
  1. Hapax-bigram zero-lookup resolution (M_2[a, b] == 1): LF(x) = C_2[b, a]
     is determined directly from (F[x], L[x]) alone.
  2. Monochromatic-image lone-row bundling: when I_{ba} lies wholly inside a
     single maximal run R_d of L, ALL M_2[a, b] scattered rows in I_a with
     L[x] = b (including lone rows!) share the exact same 2nd byte d and enter
     the same contiguous sub-interval inside I_d — allowing O(1) 2-step bypass
     in P = 2, Q_acq = 0 without touching I_{ba} during composition.
  3. 2-gram piecewise-linear IFS target prediction: sharpening S6's 1-gram
     linear predictor using exact M_2[a, b] slopes inside each F-bucket I_a.
  4. Exact byte-for-byte decode verifying the M_2-bypass retained decoder on
     all 8/8 blocks (2 synthetic novel hex + 6 admitted Wikidata blocks), with
     honest Pi cost accounting (Q, D, P, S).
"""

from __future__ import annotations

import bisect
import json
import math
import os
import sys
from collections import Counter
from typing import Any

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common


def build_m2_table(blk: common.BlockData) -> tuple[list[list[int]], list[list[int]]]:
    """Single streaming pass over blk.runs (P=1, Q=0): build exact M_2[a][b] and C_2[b][a].

    M_2[a][b] = number of rows x in F-bucket I_a with L[x] == b.
    C_2[b][a] = C[b] + sum_{a' < a} M_2[a'][b] = start row of I_{ba} = LF(I_a cap L^{-1}(b)).
    """
    C = blk.C
    counts = blk.counts
    n = blk.n

    # Build F-bucket intervals [C[a], C[a] + counts[a])
    active_a = [a for a in range(256) if counts[a] > 0]
    M2 = [[0] * 256 for _ in range(256)]

    # Stream runs against F-buckets in a single linear merge (O(r + |Sigma|), Q=0)
    a_idx = 0
    for r_start, r_w, b in blk.runs:
        r_end = r_start + r_w
        pos = r_start
        while pos < r_end:
            a = active_a[a_idx]
            a_end = C[a] + counts[a]
            if pos >= a_end:
                a_idx += 1
                continue
            take = min(r_end, a_end) - pos
            M2[a][b] += take
            pos += take

    # Build C_2[b][a]
    C2 = [[0] * 256 for _ in range(256)]
    for b in range(256):
        acc = C[b]
        for a in range(256):
            C2[b][a] = acc
            acc += M2[a][b]
        assert acc == C[b] + counts[b]
    return M2, C2


def shannon_entropy_bits(counter: Counter[int]) -> float:
    tot = sum(counter.values())
    if tot == 0:
        return 0.0
    ent = 0.0
    for v in counter.values():
        p = v / tot
        ent -= p * math.log2(p)
    return ent


def analyze_block(blk: common.BlockData, limit: int = 16) -> dict[str, Any]:
    n = blk.n
    L = blk.L
    C = blk.C
    counts = blk.counts
    M2, C2 = build_m2_table(blk)

    # Index maximal runs by start for O(log r) check whether I_{ba} sits inside ONE run of L
    run_starts = [s for (s, _, _) in blk.runs]

    # Classify every non-empty bigram (a, b) with M2[a][b] > 0:
    #   hapax: M2[a][b] == 1
    #   mono_run: I_{ba} = [lo, hi) is wholly contained inside ONE maximal run (rs, rw, d) of L
    #   mono_thick: mono_run AND rw >= limit (lies inside a thick Rauzy-Veech run!)
    n_nonempty_bigrams = 0
    n_hapax_bigrams = 0
    n_mono_bigrams = 0
    n_mono_thick_bigrams = 0
    mono_info: dict[tuple[int, int], tuple[int, int, int]] = {}  # (a, b) -> (d, lf_base_of_Iba, run_w)

    # Precompute rank_before for each run in one streaming pass over runs (Q=0)
    run_rank = [0] * len(blk.runs)
    occ_r = [0] * 256
    for idx, (_, w, b) in enumerate(blk.runs):
        run_rank[idx] = occ_r[b]
        occ_r[b] += w

    for a in range(256):
        if counts[a] == 0:
            continue
        for b in range(256):
            cnt = M2[a][b]
            if cnt == 0:
                continue
            n_nonempty_bigrams += 1
            if cnt == 1:
                n_hapax_bigrams += 1
            lo = C2[b][a]
            hi = lo + cnt
            r_idx = bisect.bisect_right(run_starts, lo) - 1
            rs, rw, d = blk.runs[r_idx]
            if lo >= rs and hi <= rs + rw:
                n_mono_bigrams += 1
                if rw >= limit:
                    n_mono_thick_bigrams += 1
                # Exact LF image of lo inside I_d:
                lf_lo = C[d] + run_rank[r_idx] + (lo - rs)
                mono_info[(a, b)] = (d, lf_lo, rw)

    # Count how many total rows and how many LONE rows (h1) fall in hapax / mono_run / mono_thick
    # Also compute 1-gram vs 2-gram piecewise-linear IFS prediction residual entropy!
    active_a = [a for a in range(256) if counts[a] > 0]
    rows_hapax = 0
    rows_mono = 0
    rows_mono_thick = 0
    h1_hapax = 0
    h1_mono = 0
    h1_mono_thick = 0

    res1_hist: Counter[int] = Counter()
    res2_hist: Counter[int] = Counter()
    res1_abs: list[int] = []
    res2_abs: list[int] = []

    occ = [0] * 256
    a_idx = 0
    # Stream through runs to classify rows and measure 1-gram vs 2-gram IFS residuals
    for r_start, r_w, b in blk.runs:
        is_lone = (r_w == 1)
        for offset in range(r_w):
            x = r_start + offset
            while x >= C[active_a[a_idx]] + counts[active_a[a_idx]]:
                a_idx += 1
            a = active_a[a_idx]
            y_true = C[b] + occ[b]
            occ[b] += 1

            # 1-gram global linear predictor
            y_pred1 = C[b] + (x * counts[b]) // n
            e1 = y_true - y_pred1
            res1_hist[e1] += 1
            res1_abs.append(abs(e1))

            # 2-gram exact M2-bucket linear predictor (FREE from M2!)
            y_pred2 = C2[b][a] + ((x - C[a]) * M2[a][b]) // counts[a]
            e2 = y_true - y_pred2
            res2_hist[e2] += 1
            res2_abs.append(abs(e2))

            if M2[a][b] == 1:
                rows_hapax += 1
                if is_lone:
                    h1_hapax += 1
            if (a, b) in mono_info:
                rows_mono += 1
                if is_lone:
                    h1_mono += 1
                if mono_info[(a, b)][2] >= limit:
                    rows_mono_thick += 1
                    if is_lone:
                        h1_mono_thick += 1

    res1_abs.sort()
    res2_abs.sort()
    ent1 = shannon_entropy_bits(res1_hist)
    ent2 = shannon_entropy_bits(res2_hist)

    # ------------------------------------------------------------------
    # Byte-exact M2-enriched 2-step bypass decoder on RAW LF (Q_acq = 0, P = 2):
    # In Pass 2 (streaming x = 0..n-1 with 256 cursors), for every row x in I_a
    # with L[x] = b:
    #   - If (a, b) in mono_info (and I_{ba} does NOT contain primary_index as stop):
    #     we directly emit a 2-byte edge (b, d) -> lf_lo + (y - C2[b][a]) WITHOUT
    #     touching row y at all during build!
    #   - Otherwise, standard 1-byte edge (b) -> y.
    # Let's check exact decode AND how many emit steps Q_run are saved on raw LF
    # AND on the retained system (LIMIT=16)!
    # ------------------------------------------------------------------
    next_row = [0] * n
    edge_len = bytearray(n)
    edge_b0 = bytearray(n)
    edge_b1 = bytearray(n)

    occ = [0] * 256
    a_idx = 0
    for r_start, r_w, b in blk.runs:
        for offset in range(r_w):
            x = r_start + offset
            while x >= C[active_a[a_idx]] + counts[active_a[a_idx]]:
                a_idx += 1
            a = active_a[a_idx]
            y = C[b] + occ[b]
            occ[b] += 1
            info = mono_info.get((a, b))
            if info is not None and y != blk.primary_index:
                d, lf_lo, _ = info
                # 2-step exact bypass using ONLY M2 + run record of I_{ba}!
                z = lf_lo + (y - C2[b][a])
                next_row[x] = z
                edge_len[x] = 2
                edge_b0[x] = b
                edge_b1[x] = d
            else:
                next_row[x] = y
                edge_len[x] = 1
                edge_b0[x] = b

    # Walk from primary_index to decode T (byte-exact check!)
    out = bytearray(n)
    pos = n
    x = blk.primary_index
    q_run_raw_m2 = 0
    bypass_hits_raw = 0
    while pos > 0:
        q_run_raw_m2 += 1
        el = edge_len[x]
        if el == 2 and pos >= 2:
            out[pos - 1] = edge_b0[x]
            out[pos - 2] = edge_b1[x]
            pos -= 2
            bypass_hits_raw += 1
            x = next_row[x]
        else:
            out[pos - 1] = edge_b0[x]
            pos -= 1
            # Fallback single step if pos == 1
            if el == 2:
                # Recompute single step y from C2/mono? Wait: only happens if pos==1,
                # which can't happen because y != primary_index guarantees remaining >= 2!
                raise AssertionError("Unexpected pos==1 on 2-step edge")
            x = next_row[x]
    assert x == blk.primary_index
    assert bytes(out) == blk.T_ref, f"Byte-exact decode FAILED on {blk.name}"

    # Also measure how many RETAINED rows (at LIMIT=16) get bypassed via M2 mono-run
    # (specifically: a retained row x whose LF(x) is ALSO a retained row, i.e., lone->lone
    # or thin->thin, so the M2 bypass ACTUALLY eliminates a retained-row Q step rather than
    # merely duplicating a thick-strip entry!).
    ret = common.build_retained_system_genuine(blk, L_min=limit)
    ret_set = set(ret.rows)
    ret_mono_total = 0
    ret_mono_to_retained = 0  # genuinely saves a retained-row Q step!
    ret_mono_to_thick = 0     # enters a thick strip (already contracted by RV!)

    # Compute exact F-bucket for each retained row in one linear merge
    a_idx = 0
    lf_ref = common.build_lf_array(L, C)
    for x in ret.rows:
        while x >= C[active_a[a_idx]] + counts[active_a[a_idx]]:
            a_idx += 1
        a = active_a[a_idx]
        b = L[x]
        if (a, b) in mono_info:
            ret_mono_total += 1
            y = lf_ref[x]
            if y in ret_set:
                ret_mono_to_retained += 1
            else:
                ret_mono_to_thick += 1

    # Pi cost vector for the M2-enriched 2-step raw-LF decoder
    m_active = len(active_a)
    s_m2 = m_active * m_active * 4  # M2 table bytes
    pi_raw = common.PiCost(
        Q=q_run_raw_m2,
        D=q_run_raw_m2,
        P=2,  # Pass 1: build M2 + runs; Pass 2: build 2-step next_row with 256 cursors
        S=s_m2 + 6 * n,  # next_row(4n) + b0(n) + b1(n) + M2
        Q_acq=0,
        Q_run=q_run_raw_m2,
        D_acq=0,
        D_run=q_run_raw_m2,
        P_acq=2,
        P_run=0,
        P_bytes=(2 * n + 6 * n) / n,
        s_cursors=m_active + 2,
        notes="P=2 Q_acq=0 M2-enriched 2-step monochromatic-image bypass on raw LF",
    )

    return {
        "name": blk.name,
        "n": n,
        "r": blk.r,
        "h1": blk.h1,
        "alphabet_active": m_active,
        "byte_exact": True,
        "m2_table_bytes": s_m2,
        "bigrams": {
            "nonempty": n_nonempty_bigrams,
            "hapax": n_hapax_bigrams,
            "mono_run": n_mono_bigrams,
            "mono_thick": n_mono_thick_bigrams,
        },
        "coverage_all_rows": {
            "hapax_rows": rows_hapax,
            "hapax_frac": round(rows_hapax / n, 6),
            "mono_rows": rows_mono,
            "mono_frac": round(rows_mono / n, 4),
            "mono_thick_rows": rows_mono_thick,
            "mono_thick_frac": round(rows_mono_thick / n, 4),
        },
        "coverage_lone_rows_h1": {
            "h1_hapax": h1_hapax,
            "h1_hapax_frac": round(h1_hapax / blk.h1, 6) if blk.h1 else 0.0,
            "h1_mono": h1_mono,
            "h1_mono_frac": round(h1_mono / blk.h1, 4) if blk.h1 else 0.0,
            "h1_mono_thick": h1_mono_thick,
            "h1_mono_thick_frac": round(h1_mono_thick / blk.h1, 4) if blk.h1 else 0.0,
        },
        "raw_lf_m2_bypass": {
            "Q_run": q_run_raw_m2,
            "Q_run_over_n": round(q_run_raw_m2 / n, 4),
            "Q_run_over_h1": round(q_run_raw_m2 / blk.h1, 4) if blk.h1 else 0.0,
            "bypass_hits": bypass_hits_raw,
            "steps_saved_frac_of_n": round((n - q_run_raw_m2) / n, 4),
        },
        "retained_M16_m2_bypass": {
            "m_retained": ret.m,
            "ret_mono_total": ret_mono_total,
            "ret_mono_to_thick_redundant": ret_mono_to_thick,
            "ret_mono_to_retained_genuine": ret_mono_to_retained,
            "genuine_saved_frac_of_m": round(ret_mono_to_retained / ret.m, 4) if ret.m else 0.0,
            "genuine_saved_frac_of_h1": round(ret_mono_to_retained / blk.h1, 4) if blk.h1 else 0.0,
        },
        "ifs_1gram_vs_2gram": {
            "res1_median_abs": res1_abs[n // 2],
            "res2_median_abs": res2_abs[n // 2],
            "res1_entropy_bits": round(ent1, 3),
            "res2_entropy_bits": round(ent2, 3),
            "entropy_saved_bits": round(ent1 - ent2, 3),
        },
        "pi_cost_raw": pi_raw.as_dict(blk.h1, n),
    }


def main() -> None:
    blocks = [
        common.make_synthetic_novel_hex_iid(65536),
        common.make_synthetic_templated_hex(1000, 40),
        *common.load_admitted_blocks(),
    ]
    results = []
    for blk in blocks:
        res = analyze_block(blk, limit=16)
        results.append(res)
        c1 = res["coverage_lone_rows_h1"]
        rm = res["retained_M16_m2_bypass"]
        raw = res["raw_lf_m2_bypass"]
        ifs = res["ifs_1gram_vs_2gram"]
        print(
            f"{blk.name:24s} | raw_Q/n={raw['Q_run_over_n']:.3f} | "
            f"h1_mono={c1['h1_mono_frac']*100:5.2f}% | "
            f"M16_gen_save={rm['genuine_saved_frac_of_h1']*100:4.2f}% h1 ({rm['ret_mono_to_retained_genuine']:4d}) | "
            f"IFS_H: {ifs['res1_entropy_bits']:.2f}->{ifs['res2_entropy_bits']:.2f}b (med {ifs['res1_median_abs']}->{ifs['res2_median_abs']}) | EXACT"
        )

    out_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "results_e2_prefix_histogram_m2.json")
    with open(out_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
