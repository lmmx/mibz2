#!/usr/bin/env python3
"""Sense 2 (relabelling): Moving together up to a consistent relabelling.

Three mathematically distinct realizations (avoiding already-killed post-BWT
alphabet quotienting of Carbon C-3 / kitchen-sink C7-C8):
  2A. Period-p Strided / Arithmetic-Progression Rauzy-Veech (p = 2, 3, 4):
      A periodic stretch L[s + j] = L[s + (j mod p)] consists entirely of
      width-1 lone rows (for minimal period p >= 2), YET each residue class
      C_r = {s + r + t*p} obeys an EXACT affine transport
          LF(s + r + t*p) = LF(s + r) + t * mu_r
      (when mu_r = 1, mapping a scattered lone-row progression onto a
      CONTIGUOUS interval!). Byte-exact decode verified on all 8 blocks.
  2B. Monotone Ramp Relabelling (L[s + j] = c0 + j * delta, delta in {+1,-1}):
      Measures sequential counter ramps (e.g. Q-IDs / timestamps) inside h1.
  2C. Baker Parameterized S_Sigma-Equivariance (W_k(y) = phi(W_k(x)) for a
      bijection phi in S_Sigma): measures exact Bell-number collapse
      (|exact k-mers| -> |param k-mers| <= B_k) on novel random hex and all 6
      blocks, AND quantifies the Equivariance Obstruction (target rank
      discrepancy across phi-related pairs).
"""

from __future__ import annotations

import json
import math
import sys
from collections import Counter, defaultdict
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


def run_periodic_strided_rv(blk: BlockData, max_p: int = 4) -> dict:
    """Sense 2A: Period-p Strided Rauzy-Veech on lone rows.

    Scans L in ONE streaming pass (Q_acq = 0, s = 256 cursors) to find all
    non-overlapping maximal periodic runs of minimal period p in {2..max_p}
    with length >= 2*p (so >= 2 full periods, every residue class has >= 2 rows).
    Verifies exact affine law LF(s + r + t*p) == LF(s + r) + t * mu_r for
    EVERY detected segment, then executes a byte-exact decode using the
    strided-segment representation!
    """
    n = blk.n
    L = blk.L
    C = blk.C

    # Mark width-1 rows (lone rows)
    is_w1 = bytearray(n)
    for s, w, _ in blk.runs:
        if w == 1:
            is_w1[s] = 1

    # Single streaming pass to build exact LF via 256 sequential cursors (Q = 0)
    # AND detect maximal non-overlapping periodic segments (minimal period 2..max_p, len >= 2*p)
    # Note: we check ONLY runs that are not constant (L[i] == L[i+1] is already RV p=1).
    occ = C[:]
    # Detect greedy left-to-right non-overlapping periodic segments of period p in 2..max_p
    # Only include rows where local pattern has minimal period p >= 2 (not constant!).
    seg_of_row = [-1] * n
    segments: list[tuple[int, int, int, list[int], list[int], list[int]]] = []
    # Each segment: (start, length, p, pattern_bytes, base_lf_per_res, mu_per_res)

    # First pass: compute prefix rank at segment starts during the same linear scan
    # To keep code transparent, track running occ[b] as i advances 0..n-1.
    i = 0
    while i < n:
        # Try periods p = 2..max_p (prefer smallest minimal period with longest span >= 2*p)
        best_p = 0
        best_end = i
        for p in range(2, max_p + 1):
            if i + 2 * p > n:
                continue
            # Must not be all-constant (that's p=1 Rauzy-Veech!)
            pat = L[i : i + p]
            if len(set(pat)) == 1:
                continue
            # Must not have a smaller divisor period
            if p == 4 and pat[0] == pat[2] and pat[1] == pat[3]:
                continue
            # Check at least 2 full periods match
            if L[i + p : i + 2 * p] != pat:
                continue
            end = i + 2 * p
            while end < n and L[end] == pat[(end - i) % p]:
                end += 1
            if end > best_end:
                best_end = end
                best_p = p
        if best_p > 0:
            s = i
            length = best_end - s
            p = best_p
            pat = list(L[s : s + p])
            # Multiplicity of each residue's byte inside one period
            pat_cnt = Counter(pat)
            mu = [pat_cnt[b] for b in pat]
            # Base LF for first occurrence of each residue r in 0..p-1
            base_lf = [0] * p
            for r in range(p):
                b = pat[r]
                base_lf[r] = occ[b]
                occ[b] += 1
            # Advance occ for remaining rows of segment and VERIFY exact affine formula!
            seg_id = len(segments)
            for j in range(p):
                seg_of_row[s + j] = seg_id
            for j in range(p, length):
                b = L[s + j]
                actual_lf = occ[b]
                occ[b] += 1
                r = j % p
                t = j // p
                pred_lf = base_lf[r] + t * mu[r]
                assert actual_lf == pred_lf, (
                    f"Affine transport violated at s={s}, j={j}, p={p}: "
                    f"actual={actual_lf} != pred={pred_lf}"
                )
                seg_of_row[s + j] = seg_id
            segments.append((s, length, p, pat, base_lf, mu))
            i = best_end
        else:
            occ[L[i]] += 1
            i += 1

    # Also build standard LF for non-periodic rows (already computed in same pass if stored)
    lf_full = build_lf_array(L, C)

    # Measure contraction on h1 (width-1 lone rows)
    by_p_segs: Counter[int] = Counter()
    by_p_rows: Counter[int] = Counter()
    by_p_w1: Counter[int] = Counter()
    w1_in_periodic = 0
    w1_saved_by_strided = 0  # each residue class of a segment represents q rows with 1 rule -> saves (q-1)
    total_strided_rules = 0

    for s, length, p, pat, base_lf, mu in segments:
        by_p_segs[p] += 1
        by_p_rows[p] += length
        w1_here = sum(is_w1[s : s + length])
        by_p_w1[p] += w1_here
        w1_in_periodic += w1_here
        # Number of affine rules for this segment is p (one per residue class)
        total_strided_rules += p
        if w1_here > p:
            w1_saved_by_strided += w1_here - p

    # Byte-exact decode using ONLY the affine transport rule inside periodic segments
    # (never reading lf_full for rows inside periodic segments!)
    out = bytearray(n)
    x = blk.primary_index
    used_affine_steps = 0
    for k in range(n - 1, -1, -1):
        sid = seg_of_row[x]
        if sid != -1:
            s, length, p, pat, base_lf, mu = segments[sid]
            j = x - s
            r = j % p
            t = j // p
            out[k] = pat[r]
            x = base_lf[r] + t * mu[r]
            used_affine_steps += 1
        else:
            out[k] = L[x]
            x = lf_full[x]
    assert x == blk.primary_index and bytes(out) == blk.T_ref, "Periodic-strided decode mismatch!"

    # Also check: how many strided segments have ALL residue classes landing in
    # contiguous intervals (mu_r == 1 for all r), and could compose with a second step?
    pure_mu1_segs = sum(1 for seg in segments if all(m == 1 for m in seg[5]))
    # Reduced run count if periodic segments are counted as p rules instead of `length` rows:
    r_eff = blk.r - sum(length for (_, length, _, _, _, _) in segments if True) + total_strided_rules
    # More precisely, how many maximal runs were covered by segments:
    runs_covered = 0
    for s, w, _ in blk.runs:
        if seg_of_row[s] != -1 and seg_of_row[s + w - 1] == seg_of_row[s]:
            runs_covered += 1

    # Cost accounting under Pi:
    # Build: 1 streaming pass over L/IN (Q_acq = 0, P_acq = 1, s = 256 cursors)
    # Run: walking still touches each row unless we compose the affine map with subsequent
    # thick/strided strips; here each step on a lone row NOT contracted by composition is 1 Q.
    # Even if every periodic segment composed away its internal periodicity, savings on h1 is
    # bounded by w1_saved_by_strided!
    cost_walk_with_strided = PiCost(
        Q=n,
        D=n,
        P=2,
        S=len(segments) * 48 + n * 4,
        Q_acq=0,
        Q_run=n,
        D_acq=0,
        D_run=n,
        P_acq=1,
        P_run=1,
        P_bytes=round((n + len(segments) * 48) / n, 3),
        s_cursors=256,
        notes="1-step strided representation proves exact affine law; contraction on h1 bounded by w1_saved",
    )

    return {
        "n_segments": len(segments),
        "pure_mu1_segments": pure_mu1_segs,
        "by_p_segments": dict(sorted(by_p_segs.items())),
        "by_p_rows": dict(sorted(by_p_rows.items())),
        "by_p_w1": dict(sorted(by_p_w1.items())),
        "w1_in_periodic": w1_in_periodic,
        "w1_in_periodic_frac": round(w1_in_periodic / blk.h1, 5) if blk.h1 else 0.0,
        "w1_saved_upper_bound": w1_saved_by_strided,
        "w1_saved_frac": round(w1_saved_by_strided / blk.h1, 5) if blk.h1 else 0.0,
        "used_affine_steps": used_affine_steps,
        "byte_exact": True,
        "cost": cost_walk_with_strided.as_dict(blk.h1, n),
    }


def run_monotone_ramps(blk: BlockData, min_len: int = 3) -> dict:
    """Sense 2B: Monotone ramp relabelling L[s+j] = c0 + j*delta (delta in {+1,-1})."""
    n = blk.n
    L = blk.L
    is_w1 = bytearray(n)
    for s, w, _ in blk.runs:
        if w == 1:
            is_w1[s] = 1

    ramps = 0
    ramp_rows = 0
    ramp_w1 = 0
    max_ramp = 0
    i = 0
    while i + 1 < n:
        d = L[i + 1] - L[i]
        if d in (1, -1):
            j = i + 2
            while j < n and L[j] - L[j - 1] == d:
                j += 1
            length = j - i
            if length >= min_len:
                ramps += 1
                ramp_rows += length
                ramp_w1 += sum(is_w1[i:j])
                if length > max_ramp:
                    max_ramp = length
            i = j
        else:
            i += 1
    return {
        "min_len": min_len,
        "n_ramps": ramps,
        "ramp_rows": ramp_rows,
        "ramp_w1": ramp_w1,
        "ramp_w1_frac": round(ramp_w1 / blk.h1, 5) if blk.h1 else 0.0,
        "max_ramp_len": max_ramp,
    }


def run_parameterized_equivariance(blk: BlockData, ks: tuple[int, ...] = (4, 6, 8)) -> dict:
    """Sense 2C: Baker S_Sigma-Equivariant (Parameterized) k-step trajectories.

    For each width-1 lone row x in J1, compute its k-step backward word
        W_k(x) = (L[x], L[LF(x)], ..., L[LF^{k-1}(x)])
    and its canonical S_Sigma-relabelling shape pi_k(x) (first-appearance index).
    Compares:
      - number of distinct exact k-mers vs distinct parameterized shapes (<= Bell B_k)
      - fraction of J1 sharing a parameterized shape of size >= 2 vs exact k-mer size >= 2
      - Equivariance Obstruction: for pairs (x, y) sharing pi_k(x) = pi_k(y) with
        bijection phi, measures the exact 1-step target rank discrepancy
            |(LF(y) - C[phi(c)]) - (LF(x) - C[c])|
        and fraction of pairs with zero rank discrepancy (exact equivariant transport).
    """
    n = blk.n
    L = blk.L
    C = blk.C
    lf = build_lf_array(L, C)

    w1_rows = [s for (s, w, _) in blk.runs if w == 1]
    h1 = len(w1_rows)
    max_k = max(ks)

    # Precompute k-step words for w1 rows (sample up to 50k w1 rows if huge? No, h1 <= 97k is fast!)
    # Actually 97k * 8 = 776k steps -> < 0.3s in Python!
    words: list[tuple[int, ...]] = []
    for x in w1_rows:
        cur = x
        w = []
        for _ in range(max_k):
            w.append(L[cur])
            cur = lf[cur]
        words.append(tuple(w))

    bell = {4: 15, 6: 203, 8: 4140}
    out = {}
    for k in ks:
        exact_cnt: Counter[tuple[int, ...]] = Counter()
        param_groups: dict[tuple[int, ...], list[int]] = defaultdict(list)

        for idx, w_full in enumerate(words):
            wk = w_full[:k]
            exact_cnt[wk] += 1
            # Canonical parameterized shape (0, 1, ...)
            seen: dict[int, int] = {}
            nxt = 0
            sig = []
            for b in wk:
                v = seen.get(b)
                if v is None:
                    v = nxt
                    seen[b] = v
                    nxt += 1
                sig.append(v)
            param_groups[tuple(sig)].append(idx)

        exact_shared = sum(c for c in exact_cnt.values() if c >= 2)
        param_shared = sum(len(g) for g in param_groups.values() if len(g) >= 2)

        # Measure Equivariance Obstruction on up to 5,000 consecutive pairs in same param_group
        # with distinct leading bytes (genuine non-trivial relabelling phi(c1) = c2 != c1)
        discs: list[int] = []
        exact_zero_disc = 0
        sampled_pairs = 0
        for sig, idxs in param_groups.items():
            if len(idxs) < 2:
                continue
            for j in range(len(idxs) - 1):
                if sampled_pairs >= 5000:
                    break
                x = w1_rows[idxs[j]]
                y = w1_rows[idxs[j + 1]]
                cx = L[x]
                cy = L[y]
                if cx == cy:
                    continue
                # Relative rank within respective F-buckets:
                rx = lf[x] - C[cx]
                ry = lf[y] - C[cy]
                d = abs(ry - rx)
                discs.append(d)
                if d == 0:
                    exact_zero_disc += 1
                sampled_pairs += 1
            if sampled_pairs >= 5000:
                break

        discs.sort()
        med_disc = discs[len(discs) // 2] if discs else 0
        mean_disc = round(sum(discs) / len(discs), 1) if discs else 0.0

        # Also compute exact word-slot savings: average distinct symbols d_k vs k
        avg_distinct = sum(len(set(sig)) * len(g) for sig, g in param_groups.items()) / h1

        out[f"k={k}"] = {
            "bell_bound": bell.get(k, -1),
            "exact_classes": len(exact_cnt),
            "param_classes": len(param_groups),
            "collapse_ratio": round(len(exact_cnt) / max(1, len(param_groups)), 2),
            "exact_shared_w1_frac": round(exact_shared / h1, 4),
            "param_shared_w1_frac": round(param_shared / h1, 4),
            "avg_distinct_bytes_in_kmer": round(avg_distinct, 3),
            "sampled_nontrivial_phi_pairs": sampled_pairs,
            "median_rank_discrepancy": med_disc,
            "mean_rank_discrepancy": mean_disc,
            "zero_discrepancy_pairs": exact_zero_disc,
        }
    return out


def main() -> None:
    blocks = [
        make_synthetic_novel_hex_iid(65536),
        make_synthetic_templated_hex(1000, 40),
    ] + load_admitted_blocks()

    all_results = {}
    for blk in blocks:
        print(f"\n=== Sense 2 (relabelling): {blk.name} (n={blk.n}, r={blk.r}, h1={blk.h1}) ===")
        r_2a = run_periodic_strided_rv(blk, max_p=4)
        r_2b = run_monotone_ramps(blk, min_len=3)
        r_2c = run_parameterized_equivariance(blk)
        all_results[blk.name] = {
            "n": blk.n,
            "r": blk.r,
            "h1": blk.h1,
            "s2a_periodic_strided": r_2a,
            "s2b_monotone_ramps": r_2b,
            "s2c_parameterized_S_sigma": r_2c,
        }
        print(
            f"  2A periodic(p=2..4): segs={r_2a['n_segments']:5d} (mu1={r_2a['pure_mu1_segments']:5d}) "
            f"w1_in={r_2a['w1_in_periodic']:5d} ({r_2a['w1_in_periodic_frac']*100:.2f}%) "
            f"w1_saved_ub={r_2a['w1_saved_upper_bound']:5d} ({r_2a['w1_saved_frac']*100:.2f}%) [EXACT]"
        )
        print(
            f"  2B ramps(>=3):       ramps={r_2b['n_ramps']:4d} w1={r_2b['ramp_w1']:5d} "
            f"({r_2b['ramp_w1_frac']*100:.2f}%) max_len={r_2b['max_ramp_len']}"
        )
        p8 = r_2c["k=8"]
        print(
            f"  2C param(k=8):       exact_cls={p8['exact_classes']:6d} -> param_cls={p8['param_classes']:4d} "
            f"({p8['collapse_ratio']}x collapse) | exact_shared={p8['exact_shared_w1_frac']*100:.1f}% "
            f"param_shared={p8['param_shared_w1_frac']*100:.1f}% | med_rank_disc={p8['median_rank_discrepancy']}"
        )

    out_path = Path(__file__).resolve().parent / "results_s2_relabelling.json"
    out_path.write_text(json.dumps(all_results, indent=2) + "\n")
    print(f"\nSaved {out_path} [ALL BYTE-EXACT]")


if __name__ == "__main__":
    main()
