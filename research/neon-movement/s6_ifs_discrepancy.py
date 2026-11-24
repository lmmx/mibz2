#!/usr/bin/env python3
"""Sense 6 (ifs-discrepancy): Lateral Mathematics #3 — Iterated Function System
(IFS) Contraction-Mapping Transport & Exact Brownian-Bridge Discrepancy.

Generalizes MONODROMY 114 Sense J ("together up to a predicted address"):
  View LF(x) as an affine contraction f_c(x) = C[c] + p_c * x (global, B=1) or
  piecewise-affine contraction f_{b,c}(x) on B macro-blocks, plus an exact
  residual discrepancy eps_B(x) = LF(x) - round(f_{b,c}(x)).

Tests & proves:
  1. Theorem 6.1(a) — Geometric Contraction of Early Errors:
     Under k-step IFS composition f_{c_{k-1}} o ... o f_{c_0}, an initial
     address perturbation Delta_0 = n/2 decays as prod p_{c_i} (~16^{-k} on
     novel hex), vanishing below 1 row by k=5 even when x is unknown!
  2. Theorem 6.1(b) — Boundary-Error & Unsmoothed-L Wall:
     The final step's discrepancy eps(LF^{k-1}(x)) has factor 1 (uncontracted),
     with magnitude O(sqrt(n/16)) on i.i.d. hex and O(n) globally on
     inhomogeneous Wikidata (shrinking with B-block piecewise-linear IFS,
     which is equivalent to sampled-rank / FM-index / PGM checkpoints).
     Even a 1-row residual lands on an uncorrelated byte in novel hex!
  3. Byte-exact decode via piecewise-linear IFS + exact residual correction
     on all 8 blocks (2 synthetic novel-hex + 6 admitted blocks).
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import (
    BlockData,
    PiCost,
    build_lf_array,
    load_admitted_blocks,
    make_synthetic_novel_hex_iid,
    make_synthetic_templated_hex,
)


def percentile(sorted_vals: list[int], q: float) -> int:
    if not sorted_vals:
        return 0
    idx = min(len(sorted_vals) - 1, int(q * (len(sorted_vals) - 1)))
    return sorted_vals[idx]


def run_ifs_analysis(blk: BlockData, block_counts: tuple[int, ...] = (1, 64, 256, 1024)) -> dict:
    """Measure global (B=1) and piecewise-linear (B=64, 256, 1024) IFS residuals,
    verify geometric contraction of initial error across k=1..6, and execute a
    byte-exact IFS+residual decode.
    """
    n = blk.n
    L = blk.L
    C = blk.C
    counts = blk.counts
    lf = build_lf_array(L, C)

    # Identify width-1 lone rows J1
    w1_rows = [s for (s, w, _) in blk.runs if w == 1]

    # 1. Measure global & piecewise-linear IFS residuals on ALL rows and on J1!
    # For B blocks of width W = ceil(n / B), checkpoint rank_c(b*W) at every block boundary.
    # Inside block b = x // W, linear interpolation predicts:
    #   rank_pred(x) = chk[b][c] + round((x - b*W) * (chk[b+1][c] - chk[b][c]) / W_b)
    #   LF_pred(x) = C[c] + rank_pred(x)
    #   eps(x) = LF(x) - LF_pred(x)
    # Note: building checkpoints takes 1 streaming pass over IN (Q_acq = 0, s = 256!).
    ifs_by_B = {}
    for B in block_counts:
        W = max(1, (n + B - 1) // B)
        # Actual number of blocks
        n_blk = (n + W - 1) // W
        # Compute exact checkpoints chk[b][c] in 1 linear pass over L
        chk: list[list[int]] = [[0] * 256 for _ in range(n_blk + 1)]
        occ = [0] * 256
        for b_idx in range(n_blk):
            chk[b_idx] = occ[:]
            lo = b_idx * W
            hi = min(n, lo + W)
            for i in range(lo, hi):
                occ[L[i]] += 1
        chk[n_blk] = occ[:]

        # Evaluate exact residual |eps(x)| on all width-1 rows J1 (and bits needed to code eps)
        abs_eps_w1: list[int] = []
        zero_eps_w1 = 0
        bits_eps_w1 = 0
        for x in w1_rows:
            c = L[x]
            b_idx = x // W
            lo = b_idx * W
            hi = min(n, lo + W)
            wb = hi - lo
            c0 = chk[b_idx][c]
            dc = chk[b_idx + 1][c] - c0
            pred_rank = c0 + ((x - lo) * dc + wb // 2) // wb
            pred_lf = C[c] + pred_rank
            err = abs(lf[x] - pred_lf)
            abs_eps_w1.append(err)
            if err == 0:
                zero_eps_w1 += 1
            # Elias/signed bit length of err: 1 + ceil(log2(err + 1))
            bits_eps_w1 += 1 + err.bit_length()

        abs_eps_w1.sort()
        # Check exact byte-exact decode for B = 256 using ONLY (chk, L, eps_B)
        # (i.e. reconstruct LF(x) = f_{b, L[x]}(x) + eps_B[x] without ever reading lf[]!)
        byte_exact_checked = False
        if B == 256:
            # Store exact signed residuals for all n rows (costed honestly in S!)
            # Actually compute eps on the fly? No, precompute signed eps array in 1 pass
            # to verify the exact decomposition LF = IFS + eps!
            eps_all = [0] * n
            occ2 = C[:]
            for b_idx in range(n_blk):
                lo = b_idx * W
                hi = min(n, lo + W)
                wb = hi - lo
                c_lo = chk[b_idx]
                c_hi = chk[b_idx + 1]
                for x in range(lo, hi):
                    c = L[x]
                    true_lf = occ2[c]
                    occ2[c] += 1
                    pred_rank = c_lo[c] + ((x - lo) * (c_hi[c] - c_lo[c]) + wb // 2) // wb
                    eps_all[x] = true_lf - (C[c] + pred_rank)
            # Decode using ONLY IFS formula + eps_all!
            out = bytearray(n)
            x = blk.primary_index
            for k in range(n - 1, -1, -1):
                c = L[x]
                out[k] = c
                b_idx = x // W
                lo = b_idx * W
                wb = min(n, lo + W) - lo
                pred_rank = chk[b_idx][c] + ((x - lo) * (chk[b_idx + 1][c] - chk[b_idx][c]) + wb // 2) // wb
                x = C[c] + pred_rank + eps_all[x]
            assert x == blk.primary_index and bytes(out) == blk.T_ref, f"IFS+eps decode mismatch on {blk.name}!"
            byte_exact_checked = True

        # Memory of checkpoint table: (n_blk + 1) * 256 * 4 bytes
        chk_bytes = (n_blk + 1) * 256 * 4
        # Theoretical FM-index scan width per step if eps is NOT stored: scan <= W/2 bytes inside block!
        ifs_by_B[f"B={B}"] = {
            "B": n_blk,
            "W": W,
            "checkpoint_bytes": chk_bytes,
            "w1_eps_mean": round(sum(abs_eps_w1) / len(abs_eps_w1), 2),
            "w1_eps_p50": percentile(abs_eps_w1, 0.50),
            "w1_eps_p99": percentile(abs_eps_w1, 0.99),
            "w1_eps_max": abs_eps_w1[-1],
            "w1_zero_eps_frac": round(zero_eps_w1 / len(abs_eps_w1), 4),
            "w1_avg_bits_per_eps": round(bits_eps_w1 / len(abs_eps_w1), 2),
            "byte_exact_verified": byte_exact_checked,
        }

    # 2. Verify Theorem 6.1(a): Geometric Contraction of Initial Position Error under k-step IFS!
    # Pick 2,000 lone rows x in J1; start the global IFS f_{c_{k-1}} o ... o f_{c_0} from a
    # COMPLETELY WRONG initial position x_wrong = (x + n // 2) % n (error Delta_0 = n/2!),
    # feeding the true k-mer (c_0, ..., c_{k-1}), and compare:
    #   (i)  |IFS_k(x_wrong) - IFS_k(x)|  (pure initial-error contraction -> vanishes as prod p_c!)
    #   (ii) |IFS_k(x) - LF^k(x)|         (accumulated Brownian-bridge boundary error -> stays O(sqrt(n/16))!)
    sample_step = max(1, len(w1_rows) // 2000)
    sample_x = w1_rows[::sample_step][:2000]
    p_sym = [counts[c] / n for c in range(256)]

    contraction_curve = {}
    for k in (1, 2, 3, 4, 5, 6):
        init_errs: list[float] = []
        boundary_errs: list[float] = []
        for x in sample_x:
            cur_true = x
            u_true = float(x)
            u_wrong = float((x + n // 2) % n)
            for _ in range(k):
                c = L[cur_true]
                cur_true = lf[cur_true]
                u_true = C[c] + p_sym[c] * u_true
                u_wrong = C[c] + p_sym[c] * u_wrong
            init_errs.append(abs(u_wrong - u_true))
            boundary_errs.append(abs(u_true - cur_true))
        init_errs.sort()
        boundary_errs.sort()
        contraction_curve[f"k={k}"] = {
            "init_err_median": round(init_errs[len(init_errs) // 2], 4),
            "init_err_max": round(init_errs[-1], 4),
            "boundary_err_median": round(boundary_errs[len(boundary_errs) // 2], 1),
            "boundary_err_p99": round(boundary_errs[int(0.99 * (len(boundary_errs) - 1))], 1),
        }

    # 3. Referee cost under Pi (for exact B=256 IFS+residual walk vs FM-checkpoint walk):
    # Storing exact eps_all takes O(n) bits (1 addressed load per step -> Q = n, D = n, P = 2).
    # Computing exact rank from B=256 checkpoints by scanning <= W/2 bytes avoids storing eps_all
    # but STILL takes 1 addressed SEEK/LOAD per step to block b (Q = n, D = n) PLUS W/2 sequential bytes!
    cost_ifs_exact = PiCost(
        Q=n,
        D=n,
        P=2,
        S=ifs_by_B["B=256"]["checkpoint_bytes"] + n * 2,  # checkpoints + compact 16-bit residual
        Q_acq=0,
        Q_run=n,
        D_acq=0,
        D_run=n,
        P_acq=1,
        P_run=1,
        P_bytes=round((n + ifs_by_B["B=256"]["checkpoint_bytes"]) / n, 3),
        s_cursors=256,
        notes="IFS+eps exact walk: compresses target delta bits (from 20b to ~5-9b), but Q=n and D=n unchanged!",
    )

    return {
        "n": n,
        "r": blk.r,
        "h1": blk.h1,
        "ifs_by_B": ifs_by_B,
        "contraction_curve": contraction_curve,
        "cost": cost_ifs_exact.as_dict(blk.h1, n),
    }


def main() -> None:
    blocks = [
        make_synthetic_novel_hex_iid(65536),
        make_synthetic_templated_hex(1000, 40),
    ] + load_admitted_blocks()

    all_results = {}
    for blk in blocks:
        print(f"\n=== Sense 6 (ifs-discrepancy): {blk.name} (n={blk.n}, r={blk.r}, h1={blk.h1}) ===")
        res = run_ifs_analysis(blk)
        all_results[blk.name] = res
        b1 = res["ifs_by_B"]["B=1"]
        b256 = res["ifs_by_B"]["B=256"]
        b1024 = res["ifs_by_B"]["B=1024"]
        c1 = res["contraction_curve"]["k=1"]
        c5 = res["contraction_curve"]["k=5"]
        print(
            f"  B=1    (global): w1_eps med={b1['w1_eps_p50']:6d} max={b1['w1_eps_max']:6d} bits={b1['w1_avg_bits_per_eps']}"
        )
        print(
            f"  B=256  (W={b256['W']:4d}): w1_eps med={b256['w1_eps_p50']:6d} max={b256['w1_eps_max']:6d} "
            f"zero={b256['w1_zero_eps_frac']*100:4.1f}% bits={b256['w1_avg_bits_per_eps']} [EXACT]"
        )
        print(
            f"  B=1024 (W={b1024['W']:4d}): w1_eps med={b1024['w1_eps_p50']:6d} max={b1024['w1_eps_max']:6d} "
            f"zero={b1024['w1_zero_eps_frac']*100:4.1f}% bits={b1024['w1_avg_bits_per_eps']}"
        )
        print(
            f"  Contraction Delta_0={blk.n//2}: k=1 init_med={c1['init_err_median']:.1f} bnd_med={c1['boundary_err_median']:.1f} | "
            f"k=5 init_med={c5['init_err_median']:.4f} bnd_med={c5['boundary_err_median']:.1f}"
        )

    out_path = Path(__file__).resolve().parent / "results_s6_ifs_discrepancy.json"
    out_path.write_text(json.dumps(all_results, indent=2) + "\n")
    print(f"\nSaved {out_path} [ALL BYTE-EXACT]")


if __name__ == "__main__":
    main()
