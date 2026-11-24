#!/usr/bin/env python3
"""S8 (Principle 1 — Count-Targeted Landmark Induction & Orthogonal Bi-Directional Extension):

Part A (Exact NEEDS M-21b — MONODROMY record 114 §6):
  Tests MONODROMY's pre-registered count-targeted landmark selection rule on
  raw LF across all 6 admitted Wikidata blocks + 2 novel-hex benchmarks:
    - Ratio rho in {8, 16, 32}, levels k = 1, 2, 3, 4.
    - At level 1: choose byte c_1 whose count |I_{c_1}| is nearest n / rho
      (with |Y_1| >= 1), so Y_1 = I_{c_1} (contiguous, free from C[c]).
    - At level j >= 2: among symbols c_j appearing as the first emitted byte of
      return words on Y_{j-1} (extending the landmark string one byte leftward),
      choose c_j whose count is nearest |Y_{j-1}| / rho (with 1 <= |Y_j| < |Y_{j-1}|).
    - Report |Y_j|, rounds_j = max return steps at level j, total D = sum(rounds_j) + |Y_k|,
      D/n, Q = n + sum_{j=1}^k |Y_j|, and evaluate MONODROMY's pre-registered rule:
      "if D at the best (rho, k) is not under 0.5% of n on >= 4/6 blocks, withdraw."
    - Verify 8/8 byte-exact decoding from the final induced map R_k!

Part B (Orthogonal Count-Targeted Landmarks on Retained M_16 + Bi-Directional Final Walk):
  Combines row-axis Rauzy-Veech thick-strip contraction (L_min = 16) FIRST with
  count-targeted text-axis landmarks on R (m << n) and bi-directional meeting
  on Y_k (halving final walk |Y_k| -> ceil(|Y_k|/2)), verified 8/8 byte-exact!
"""

from __future__ import annotations

import json
import os
import sys
from collections import Counter
from typing import Any

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common


def induce_one_level(
    cur_next: list[int],
    cur_first_byte: list[int],
    cur_n: int,
    target_ratio: int,
    level_idx: int,
    forced_first_byte_list: list[int] | None = None,
) -> tuple[int, list[int], list[int], list[int], list[list[int]], int] | None:
    """Perform one level of count-targeted landmark induction on a single-cycle
    permutation `cur_next` of size `cur_n`.

    Each node u in 0..cur_n-1 has a first-emitted byte `cur_first_byte[u]`
    (at level 1 on raw LF, we select by F[u] = byte of F-bucket I_c so Y_1 = I_c is
    contiguous and free! At level j >= 2, we select by the next leftward byte
    emitted when stepping from u, i.e., cur_first_byte[u]!).

    Returns (chosen_sym, Y_nodes, next_Y, next_first_byte, return_chains, max_rounds)
    or None if no nontrivial strict subset exists.
    """
    label_arr = forced_first_byte_list if forced_first_byte_list is not None else cur_first_byte
    cnt = Counter(label_arr)
    target = cur_n / target_ratio

    # Choose symbol with 1 <= count < cur_n minimizing |count - target|
    candidates = [(abs(v - target), v, sym) for sym, v in cnt.items() if 1 <= v < cur_n]
    if not candidates:
        return None
    candidates.sort()
    _, chosen_cnt, chosen_sym = candidates[0]

    # Collect Y_nodes in ascending index order
    Y_nodes = [u for u in range(cur_n) if label_arr[u] == chosen_sym]
    assert len(Y_nodes) == chosen_cnt
    rank_in_Y = [-1] * cur_n
    for idx, u in enumerate(Y_nodes):
        rank_in_Y[u] = idx

    # Induce first-return map R_Y on Y_nodes (in parallel across all |Y| lanes!)
    m_y = len(Y_nodes)
    next_Y = [0] * m_y
    next_first_byte = [0] * m_y
    return_chains: list[list[int]] = [[] for _ in range(m_y)]
    max_rounds = 0
    total_steps = 0

    for idx, start_u in enumerate(Y_nodes):
        chain: list[int] = [start_u]
        u = cur_next[start_u]
        steps = 1
        while rank_in_Y[u] == -1:
            chain.append(u)
            u = cur_next[u]
            steps += 1
        next_Y[idx] = rank_in_Y[u]
        # Next leftward byte beyond the landmark string:
        # At level 1: start_u in I_{c_1}, its first emitted LF byte is L[start_u] = cur_first_byte[start_u]!
        # At level j >= 2: start_u already emitted c_j at step 0, so the next leftward extension
        # of the landmark string is the byte emitted AFTER c_j, i.e., cur_first_byte[cur_next[start_u]]!
        # Wait: let's check!
        #   Level 1: Y_1 = {u : F[u] = c_1}. Stepping LF once from u emits L[u] and lands at LF(u)
        #            whose suffix starts with L[u] + c_1! So the leftward extension of c_1 is L[u] = cur_first_byte[u]!
        #   Level 2: Y_2 = {u in Y_1 : L[u] = c_2}. Its suffix starts with c_1 and its preceding byte is c_2,
        #            so LF(u) has suffix starting with c_2 c_1! The byte preceding c_2 c_1 is L[LF(u)]!
        #   Wait: in the induced map R_1 on Y_1, stepping R_1 once from u traverses u -> LF(u) -> ... -> R_1(u).
        #   Is L[LF(u)] ALWAYS inside the return chain of u, or could LF(u) ALREADY be in Y_1?
        #   Either way, LF(u) is step 1 of the original LF chain! Wait: on level j >= 2, we only need
        #   a deterministic locally-known byte label on each node of Y_{j-1} that partitions Y_{j-1}
        #   into subsets! Using the byte at position `level_idx` of u's original LF orbit (i.e. L[LF^{j-1}(u)])
        #   IS the exact leftward extension c_j c_{j-1} ... c_1 — AND it is acquired during level j-1's
        #   wave (or in 1 step from the return word of u, wrapping into the next return word if len==1)!
        #   Even simpler & 100% self-contained on the induced return word W_{j-1}[u]:
        #   In MONODROMY 114 §3 & channel #23 ("return word begins with c'"), the label at level j >= 2
        #   is literally `L[LF^{j-1}(u)]` (the j-th byte backward from u, which extends the landmark
        #   string c_{j-1}...c_1 one byte leftward to c_j c_{j-1}...c_1!).
        #   We pass `next_left_byte` cleanly from the caller so level j ALWAYS uses exact L[LF^{j-1}(u)]!
        return_chains[idx] = chain
        if steps > max_rounds:
            max_rounds = steps
        total_steps += steps

    assert total_steps == cur_n, f"Not a single cycle: {total_steps} != {cur_n}"
    return chosen_sym, Y_nodes, next_Y, next_first_byte, return_chains, max_rounds


def run_raw_m21b_for_block(blk: common.BlockData) -> dict[str, Any]:
    """Run exact NEEDS M-21b (raw LF, rho in {8, 16, 32}, k = 1..4) and verify
    byte-exact decode at the best (rho, k) configuration!
    """
    n = blk.n
    L = blk.L
    C = blk.C
    counts = blk.counts
    lf = common.build_lf_array(L, C)

    # Precompute F[x] for all x (free from C[c] in 1 streaming pass)
    F_arr = [0] * n
    for c in range(256):
        if counts[c]:
            lo = C[c]
            hi = lo + counts[c]
            for x in range(lo, hi):
                F_arr[x] = c

    # Precompute L[LF^t(x)] for t = 0, 1, 2 (needed as leftward extensions for k = 2, 3, 4;
    # note: in the actual landmark wave, level j's lanes ALREADY step through LF^t(x) as they
    # build return words, so L[LF^{j-1}(x)] is read right off the return word at zero extra cost!)
    L_t0 = list(L)
    L_t1 = [L[lf[x]] for x in range(n)]
    L_t2 = [L[lf[lf[x]]] for x in range(n)]
    ext_by_level = [F_arr, L_t0, L_t1, L_t2]  # level 1 -> F[x], level 2 -> L[x], level 3 -> L[LF(x)], level 4 -> L[LF^2(x)]

    grid: list[dict[str, Any]] = []
    best_cfg: dict[str, Any] | None = None
    best_chains_stack: list[list[list[int]]] | None = None
    best_Y_stack: list[list[int]] | None = None
    best_final_next: list[int] | None = None

    for rho in (8, 16, 32):
        cur_next = lf
        cur_n = n
        # Track mapping from current induced node idx -> original row x in 0..n-1
        orig_row = list(range(n))
        levels_info: list[dict[str, Any]] = []
        chains_stack: list[list[list[int]]] = []
        Y_stack: list[list[int]] = []

        cum_rounds = 0
        cum_Q_build = 0

        for k in range(1, 5):
            # Label for each current node u is exact leftward extension byte ext_by_level[k-1][orig_row[u]]
            ext_src = ext_by_level[k - 1]
            labels = [ext_src[orig_row[u]] for u in range(cur_n)]
            res = induce_one_level(cur_next, labels, cur_n, rho, k, forced_first_byte_list=labels)
            if res is None:
                break
            sym, Y_nodes, next_Y, _, ret_chains, max_rnd = res
            cum_Q_build += cur_n
            cum_rounds += max_rnd
            m_y = len(Y_nodes)
            D_tot = cum_rounds + m_y
            Q_tot = cum_Q_build + m_y

            chains_stack.append(ret_chains)
            Y_stack.append(Y_nodes)

            # Update orig_row for next level
            orig_row = [orig_row[u] for u in Y_nodes]
            cur_next = next_Y
            cur_n = m_y

            lvl_rec = {
                "rho": rho,
                "k": k,
                "chosen_byte": sym,
                "Y_size": m_y,
                "Y_over_n": round(m_y / n, 6),
                "rounds_level": max_rnd,
                "cum_rounds": cum_rounds,
                "D_total": D_tot,
                "D_over_n": round(D_tot / n, 6),
                "D_pct_of_n": round(100.0 * D_tot / n, 4),
                "Q_total": Q_tot,
                "Q_over_n": round(Q_tot / n, 4),
                "Q_over_h1": round(Q_tot / blk.h1, 4) if blk.h1 else 0.0,
            }
            levels_info.append(lvl_rec)
            grid.append(lvl_rec)

            if best_cfg is None or D_tot < best_cfg["D_total"]:
                best_cfg = lvl_rec
                best_chains_stack = [c[:] for c in chains_stack]
                best_Y_stack = [y[:] for y in Y_stack]
                best_final_next = cur_next[:]

    assert best_cfg is not None and best_chains_stack is not None and best_Y_stack is not None and best_final_next is not None

    # Byte-exact decode verification at best (rho, k) from the hierarchical induced return map!
    # Unroll final cycle on Y_k -> expand level k chains -> ... -> level 1 chains -> raw rows -> bytes L[x]!
    k_best = best_cfg["k"]
    m_final = len(best_final_next)
    # Walk the single cycle of best_final_next starting from 0
    order = [0] * m_final
    u = 0
    for i in range(m_final):
        order[i] = u
        u = best_final_next[u]
    assert u == 0, "Final induced map not a single cycle!"

    # Expand downward from level k_best to raw rows 0..n-1
    seq = order
    for lvl in range(k_best - 1, -1, -1):
        chains = best_chains_stack[lvl]
        expanded: list[int] = []
        for idx in seq:
            expanded.extend(chains[idx])
        seq = expanded
    assert len(seq) == n

    # Rotate cyclic sequence so it starts at primary_index and read L[x] in reverse -> T!
    p_pos = seq.index(blk.primary_index)
    out = bytearray(n)
    for t in range(n):
        row = seq[(p_pos + t) % n]
        out[n - 1 - t] = L[row]
    assert bytes(out) == blk.T_ref, f"Raw M-21b byte-exact decode FAILED on {blk.name}"

    return {
        "best": best_cfg,
        "under_0_5_pct": best_cfg["D_pct_of_n"] < 0.5,
        "byte_exact": True,
        "grid": grid,
    }


def run_orthogonal_bidir_m21b(blk: common.BlockData, L_min: int = 16) -> dict[str, Any]:
    """Part B: Row-axis Rauzy-Veech thick contraction (L_min) FIRST, then
    count-targeted landmark induction (rho in {8, 16, 32}, k = 1..4) on the
    m-row retained map R, PLUS bi-directional meeting on Y_k (halving final walk
    |Y_k| -> ceil(|Y_k|/2))! Verified 100% byte-exact against blk.T_ref!
    """
    n = blk.n
    L = blk.L
    ret = common.build_retained_system_genuine(blk, L_min=L_min)
    m = ret.m
    R = ret.R

    # For each retained node j in 0..m-1, precompute its retained-orbit bytes at offsets 0, 1, 2, 3:
    # Level 1 label: F[ret.rows[j]] (or L[ret.rows[j]] — both free in 1 scan of M!)
    # Let's use L[ret.rows[j]] at level 1, L[ret.rows[R[j]]] at level 2, L[ret.rows[R^2[j]]] at level 3,
    # L[ret.rows[R^3[j]]] at level 4 — all acquired along the retained-map waves!
    b0 = [L[ret.rows[j]] for j in range(m)]
    b1 = [b0[R[j]] for j in range(m)]
    b2 = [b1[R[j]] for j in range(m)]
    b3 = [b2[R[j]] for j in range(m)]
    ext_by_level = [b0, b1, b2, b3]

    best_cfg: dict[str, Any] | None = None
    best_chains_stack: list[list[list[int]]] | None = None
    best_final_next: list[int] | None = None

    for rho in (8, 16, 32):
        cur_next = R
        cur_n = m
        orig_j = list(range(m))
        chains_stack: list[list[list[int]]] = []
        cum_rounds = 0
        cum_Q_build = 0

        for k in range(1, 5):
            ext_src = ext_by_level[k - 1]
            labels = [ext_src[orig_j[u]] for u in range(cur_n)]
            res = induce_one_level(cur_next, labels, cur_n, rho, k, forced_first_byte_list=labels)
            if res is None:
                break
            sym, Y_nodes, next_Y, _, ret_chains, max_rnd = res
            cum_Q_build += cur_n
            cum_rounds += max_rnd
            m_y = len(Y_nodes)

            # Bi-directional final walk on Y_k halves final-walk depth: ceil(m_y / 2)
            D_unidir = ret.D_acq + cum_rounds + m_y
            D_bidir = ret.D_acq + cum_rounds + (m_y + 1) // 2
            Q_tot = ret.Q_acq_strips + ret.Q_acq_glue + cum_Q_build + m_y

            chains_stack.append(ret_chains)
            orig_j = [orig_j[u] for u in Y_nodes]
            cur_next = next_Y
            cur_n = m_y

            rec = {
                "rho": rho,
                "k": k,
                "chosen_byte": sym,
                "Y_size": m_y,
                "cum_rounds": cum_rounds,
                "D_acq_strips": ret.D_acq,
                "D_unidir": D_unidir,
                "D_unidir_pct_of_n": round(100.0 * D_unidir / n, 4),
                "D_bidir": D_bidir,
                "D_bidir_pct_of_n": round(100.0 * D_bidir / n, 4),
                "depth_collapse_vs_n": round(n / D_bidir, 1),
                "Q_total": Q_tot,
                "Q_over_h1": round(Q_tot / blk.h1, 4) if blk.h1 else 0.0,
                "Q_over_n": round(Q_tot / n, 4),
                "P": 2 + k,
            }
            if best_cfg is None or D_bidir < best_cfg["D_bidir"]:
                best_cfg = rec
                best_chains_stack = [c[:] for c in chains_stack]
                best_final_next = cur_next[:]

    assert best_cfg is not None and best_chains_stack is not None and best_final_next is not None

    # Byte-exact decode verification from orthogonal bi-directional final map!
    m_final = len(best_final_next)
    # Build inverse of best_final_next for explicit bi-directional meet-in-the-middle walk
    final_prev = [0] * m_final
    for u, v in enumerate(best_final_next):
        final_prev[v] = u

    # Meet-in-the-middle on Y_k: walk forward (best_final_next) and backward (final_prev) from 0
    fwd: list[int] = [0]
    bwd: list[int] = []
    uf = 0
    ub = 0
    steps_bidi = 0
    while len(fwd) + len(bwd) < m_final:
        uf = best_final_next[uf]
        steps_bidi += 1
        fwd.append(uf)
        if len(fwd) + len(bwd) >= m_final:
            break
        ub = final_prev[ub]
        bwd.append(ub)
    assert steps_bidi == (m_final + 1) // 2 or steps_bidi == m_final // 2
    order = fwd + list(reversed(bwd))
    assert len(order) == m_final and len(set(order)) == m_final

    # Expand through k levels back to retained ranks 0..m-1
    seq = order
    for lvl in range(best_cfg["k"] - 1, -1, -1):
        chains = best_chains_stack[lvl]
        expanded: list[int] = []
        for idx in seq:
            expanded.extend(chains[idx])
        seq = expanded
    assert len(seq) == m

    # Rotate to ret.mark_rank and expand word DAG nodes -> T!
    mp = seq.index(ret.mark_rank)
    out_rev = bytearray()
    for t in range(m):
        rk = seq[(mp + t) % m]
        common.expand_dag_word(ret.dag_left, ret.dag_right, ret.R_word_id[rk], out_rev)
    decoded = bytes(reversed(out_rev))
    assert decoded == blk.T_ref, f"Orthogonal M-21b byte-exact decode FAILED on {blk.name}"

    return {
        "L_min": L_min,
        "m_retained": m,
        "best": best_cfg,
        "byte_exact": True,
    }


def main() -> None:
    blocks = [
        (common.make_synthetic_novel_hex_iid(65536), 2),
        (common.make_synthetic_templated_hex(1000, 40), 16),
        *[(b, 16) for b in common.load_admitted_blocks()],
    ]
    results = []
    n_admitted_under_05 = 0

    for blk, lmin in blocks:
        raw_res = run_raw_m21b_for_block(blk)
        orth_res = run_orthogonal_bidir_m21b(blk, L_min=lmin)
        rb = raw_res["best"]
        ob = orth_res["best"]
        if blk.name.startswith("block-") and raw_res["under_0_5_pct"]:
            n_admitted_under_05 += 1
        results.append(
            {
                "name": blk.name,
                "n": blk.n,
                "r": blk.r,
                "h1": blk.h1,
                "raw_m21b": raw_res,
                "orthogonal_bidir_m21b": orth_res,
            }
        )
        print(
            f"{blk.name:24s} | RAW M-21b best (rho={rb['rho']:2d},k={rb['k']}): "
            f"D={rb['D_total']:5d} ({rb['D_pct_of_n']:.3f}% n, |Yk|={rb['Y_size']:4d}, Q={rb['Q_over_n']:.3f}n) "
            f"{'PASS<0.5%' if raw_res['under_0_5_pct'] else 'FAIL>=0.5%'} | "
            f"ORTH-BIDIR (rho={ob['rho']:2d},k={ob['k']}): D={ob['D_bidir']:4d} ({ob['D_bidir_pct_of_n']:.3f}% n, {ob['depth_collapse_vs_n']:.0f}x) | EXACT"
        )

    print(
        f"\nMONODROMY NEEDS M-21b pre-registered gate (<0.5% of n on >=4/6 admitted blocks): "
        f"{n_admitted_under_05}/6 -> {'STANDS' if n_admitted_under_05 >= 4 else 'WITHDRAWN'}"
    )
    out_path = os.path.join(
        os.path.dirname(os.path.abspath(__file__)),
        "results_s8_count_targeted_landmarks.json",
    )
    with open(out_path, "w") as f:
        json.dump(
            {
                "admitted_blocks_under_0_5_pct": n_admitted_under_05,
                "m21b_verdict": "STANDS" if n_admitted_under_05 >= 4 else "WITHDRAWN",
                "blocks": results,
            },
            f,
            indent=2,
        )
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
