#!/usr/bin/env python3
"""R1 (Channel #25 / LEADS-dynamics.md — Lead 1: Feigenbaum Renormalisation Audit):

Tests directly whether a Feigenbaum-like renormalisation operator
    R_Y(LF) = alpha_Y^{-1} o LF^{tau_Y} o alpha_Y   (first-return + affine/order rescaling)
or  R_sq(LF) = LF o LF                              (squaring)
maps (L, LF) back into a problem of the SAME kind (a small-alphabet BWT LF map),
or exhibits self-similarity / fixed-point structure on templated JSON vs novel hex.

Exact BWT-characterization invariant (Mantaci-Restivo / rising-sequence theorem):
  A permutation sigma on m elements is the LF map of some BWT over an alphabet
  of size q IF AND ONLY IF sigma^{-1} has at most q rising sequences (equivalently,
  sigma has at most q - 1 "L-descents" where sigma(x+1) < sigma(x), PLUS the
  number of maximal slope-+1 runs r(sigma) measures IET complexity!).
  Even stronger: for LF, sigma(x+1) == sigma(x) + 1 on every interior of a run,
  so the minimal BWT alphabet size realizing sigma is:
      q_min(sigma) = 1 + |{x in [0..m-2] : sigma(x+1) < sigma(x)}|   (wait: for Psi=sigma^{-1}!)
  Let's check carefully:
    For LF: LF is strictly increasing on each symbol class L^{-1}(c) (a union of
    q increasing subsequences!). By Dilworth's theorem / Schensted, the minimum
    number of increasing subsequences covering sigma equals the length of the
    LONGEST DECREASING SUBSEQUENCE of sigma — and for Psi = LF^{-1}, Psi is
    piecewise increasing on the q contiguous F-buckets I_c, so the exact number
    of descents of Psi plus 1 equals <= q (exact equality when every adjacent
    F-bucket boundary has a descent)!
    Therefore:
      q_min_exact(sigma) = 1 + |{y in [0..m-2] : sigma^{-1}(y+1) < sigma^{-1}(y)}|
    is the EXACT minimum alphabet size of ANY BWT whose LF map equals sigma!

We measure q_min_exact(R_Y), r(R_Y)/|Y|, and r(LF^2)/r(LF) across all 8 blocks
(pure novel hex, templated 40-hex, and all 6 admitted Wikidata blocks), verifying
exactness and quantifying precisely why renormalisation DOES contract the scaffold
yet SHATTERS immediately on novel random hex (q_min explodes from 16 to ~|Y|/2!).
"""

from __future__ import annotations

import json
import os
import sys
from typing import Any

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common


def bwt_invariants(perm: list[int]) -> dict[str, Any]:
    """Compute exact BWT-class invariants of a permutation perm on [0..m):
      - r_runs: number of maximal slope-+1 intervals (perm[x+1] == perm[x] + 1)
      - q_min_bwt: 1 + descents(perm^{-1}) = EXACT minimum alphabet size of any
        BWT whose LF map is perm! (For a true BWT over alphabet Sigma, q_min <= |Sigma|.)
      - descents_perm: descents of perm itself (for comparison)
    """
    m = len(perm)
    if m <= 1:
        return {"m": m, "r_runs": m, "r_over_m": 1.0, "q_min_bwt": m, "q_over_m": 1.0}
    inv = [0] * m
    for x, y in enumerate(perm):
        inv[y] = x

    r_runs = 1
    desc_p = 0
    for x in range(m - 1):
        if perm[x + 1] != perm[x] + 1:
            r_runs += 1
        if perm[x + 1] < perm[x]:
            desc_p += 1

    desc_inv = 0
    for y in range(m - 1):
        if inv[y + 1] < inv[y]:
            desc_inv += 1
    q_min = 1 + desc_inv
    return {
        "m": m,
        "r_runs": r_runs,
        "r_over_m": round(r_runs / m, 4),
        "q_min_bwt": q_min,
        "q_over_m": round(q_min / m, 4),
        "descents_perm_over_m": round(desc_p / m, 4),
    }


def first_return_perm(perm: list[int], Y_sorted: list[int]) -> list[int]:
    """Compute rescaled first-return map alpha_Y^{-1} o perm^{tau_Y} o alpha_Y on [0..|Y|)."""
    m = len(perm)
    rank = [-1] * m
    for i, u in enumerate(Y_sorted):
        rank[u] = i
    my = len(Y_sorted)
    res = [0] * my
    steps_tot = 0
    for i, u0 in enumerate(Y_sorted):
        u = perm[u0]
        steps_tot += 1
        while rank[u] == -1:
            u = perm[u]
            steps_tot += 1
        res[i] = rank[u]
    assert steps_tot == m
    return res


def analyze_block(blk: common.BlockData) -> dict[str, Any]:
    n = blk.n
    L = blk.L
    C = blk.C
    counts = blk.counts
    lf = common.build_lf_array(L, C)
    m_alpha = sum(1 for c in counts if c > 0)

    # 0. Raw LF baseline (verify q_min_bwt <= |Sigma_active|!)
    inv_lf = bwt_invariants(lf)
    assert inv_lf["q_min_bwt"] <= m_alpha, (
        f"BWT invariant violation: q_min={inv_lf['q_min_bwt']} > m_alpha={m_alpha}"
    )

    # 1. Squaring operator R_sq(LF) = LF o LF
    lf2 = [lf[lf[x]] for x in range(n)]
    inv_lf2 = bwt_invariants(lf2)

    # 2. First-return renormalisation R_Y(LF) on:
    #    (a) A hex-digit F-bucket Y = I_'0' (novel random hex section!)
    #    (b) A count-targeted F-bucket Y = I_c closest to n / 16 (typical section)
    #    (c) Scaffold delimiter Y = I_'"' (if present, else most frequent byte)
    sections: dict[str, Any] = {}
    picks: list[tuple[str, int]] = []
    if counts[ord("0")] > 0:
        picks.append(("hex_0", ord("0")))
    # Nearest to n / 16
    c16 = min((c for c in range(256) if counts[c] > 0), key=lambda c: abs(counts[c] - n / 16))
    picks.append(("count_n16", c16))
    if counts[ord('"')] > 0:
        picks.append(("scaffold_quote", ord('"')))

    for label, sym in picks:
        lo = C[sym]
        hi = lo + counts[sym]
        Y = list(range(lo, hi))
        R_Y = first_return_perm(lf, Y)
        inv_Y = bwt_invariants(R_Y)
        inv_Y["sym"] = sym
        inv_Y["sym_char"] = chr(sym) if 32 <= sym < 127 else f"0x{sym:02x}"
        # Also apply a 2nd renormalisation step R_{Y2}(R_Y) on Y2 = {i in Y : L[Y[i]] == c2}
        # choosing c2 nearest |Y|/16!
        by_c2: dict[int, list[int]] = {}
        for i, orig_x in enumerate(Y):
            b = L[orig_x]
            by_c2.setdefault(b, []).append(i)
        c2_cands = [(abs(len(v) - len(Y) / 16), b, v) for b, v in by_c2.items() if 2 <= len(v) < len(Y)]
        if c2_cands:
            c2_cands.sort()
            _, c2, Y2_idx = c2_cands[0]
            R_Y2 = first_return_perm(R_Y, Y2_idx)
            inv_Y2 = bwt_invariants(R_Y2)
            inv_Y2["sym2"] = c2
            inv_Y["level2"] = inv_Y2
        sections[label] = inv_Y

    # 3. Also measure Open's retained map R (L_min=16 on Wikidata/tmpl, L_min=2 on pure hex):
    #    Rauzy-Veech thick-strip contraction IS a renormalisation onto the thin section M!
    lmin = 2 if blk.name.startswith("synth-hex-iid") else 16
    ret = common.build_retained_system_genuine(blk, L_min=lmin)
    inv_ret = bwt_invariants(ret.R)

    return {
        "name": blk.name,
        "n": n,
        "r": blk.r,
        "h1": blk.h1,
        "alphabet_active": m_alpha,
        "raw_LF": inv_lf,
        "squared_LF2": inv_lf2,
        "first_return_sections": sections,
        "retained_M_renorm": inv_ret,
    }


def main() -> None:
    blocks = [
        common.make_synthetic_novel_hex_iid(65536),
        common.make_synthetic_templated_hex(1000, 40),
        *common.load_admitted_blocks(),
    ]
    results = []
    for blk in blocks:
        res = analyze_block(blk)
        results.append(res)
        lf = res["raw_LF"]
        lf2 = res["squared_LF2"]
        s0 = res["first_return_sections"].get("hex_0") or res["first_return_sections"]["count_n16"]
        ret = res["retained_M_renorm"]
        print(
            f"{blk.name:24s} | LF: q_min={lf['q_min_bwt']:2d} r/n={lf['r_over_m']:.3f} | "
            f"LF^2: q_min={lf2['q_min_bwt']:4d} r/n={lf2['r_over_m']:.3f} | "
            f"R(I_'0'): |Y|={s0['m']:5d} q_min={s0['q_min_bwt']:5d} ({s0['q_over_m']:.3f}|Y|) r/|Y|={s0['r_over_m']:.3f} | "
            f"R(M16): m={ret['m']:6d} q_min={ret['q_min_bwt']:5d} ({ret['q_over_m']:.3f}m)"
        )

    out_path = os.path.join(
        os.path.dirname(os.path.abspath(__file__)),
        "results_r1_renormalisation_leads.json",
    )
    with open(out_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
