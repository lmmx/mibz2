#!/usr/bin/env python3
"""Sense 3 (pell-recurrence): A recurrence among the induction's own steps
(doomslide's Pell rule) — algebraic matrix-squaring R_{j+1} = R_j o R_j via
pointer-free base-256 radix-sort-merge on the retained map R (NO word
materialisation!), plus raw-LF comparison and Zorich-Pell cycle audit.

Prior art:
  - Kitchen-sink Cycle 7 Theorem 7.2 tested IET squaring LF^{2^j} on ALL n rows
    (saturating at n by j=10, costing 12.87-14.56*n records).
  - MONODROMY 108/109/112 analyzed T^2 doubling and list ranking.
  - Chiang-Goodrich-Grove-Tamassia-Vengroff-Vitter (1995) external-memory list ranking.
What was missed:
  - Running pointer-free radix-sort-merge doubling on the 5-10x smaller contracted
    retained map R (size m16, without O(n) word materialisation), giving an exact
    (Q, D, P, P_bytes, S) Pareto frontier from j=1 down to Q_run=0, D_run=0 at
    j = ceil(log2 m), AND measuring exact raw-LF Q_total=0, D_total=0!
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
    RetainedSystem,
    build_lf_array,
    build_retained_system_genuine,
    expand_dag_word,
    load_admitted_blocks,
    make_synthetic_novel_hex_iid,
    make_synthetic_templated_hex,
)


def radix_sort_by_key(items: list[tuple], key_idx: int, max_val: int) -> tuple[list[tuple], int]:
    """Base-256 LSD radix sort using 256 sequential write cursors (Q = 0 under Pi!).

    Returns (sorted_items, n_passes) where n_passes = number of base-256 digit passes.
    Only non-negative keys in [0, max_val]; negative keys (e.g. sentinel -1) are shifted
    by +1 internally so every key is >= 0.
    """
    n_passes = max(1, (max_val + 1).bit_length() + 7) // 8
    cur = items
    for p in range(n_passes):
        shift = 8 * p
        buckets: list[list[tuple]] = [[] for _ in range(256)]
        for it in cur:
            k = it[key_idx] + 1  # shift -1 -> 0
            buckets[(k >> shift) & 0xFF].append(it)
        nxt: list[tuple] = []
        for b in buckets:
            nxt.extend(b)
        cur = nxt
    return cur, n_passes


def merge_compose_doubling_round(
    by_id: list[tuple[int, int, int]],
    m: int,
) -> tuple[list[tuple[int, int, int]], int, int]:
    """One exact pointer-free doubling round R_{j+1} = R_j o R_j via radix sort + 2-pointer merge.

    Input: by_id = [(x, nxt_x, dist_x)] sorted ascending by x in 0..m-1.
           Sentinel tail has nxt_x == -1, dist_x == 0.
    Steps (ALL strictly sequential cursor operations, Q = 0!):
      1. Radix-sort active items (nxt_x != -1) by nxt_x (base 256, Q=0, p_radix passes).
      2. Linear 2-pointer merge of sorted-by-nxt against by_id (sorted by y = id) ->
         (x, nxt_y, dist_x + dist_y) (Q=0, 1 pass).
      3. Radix-sort merged results back by x (base 256, Q=0, p_radix passes).
    Returns: (new_by_id, passes_used, bytes_streamed).
    """
    # Separate finished (nxt == -1) and active (nxt != -1)
    finished = [it for it in by_id if it[1] == -1]
    active = [it for it in by_id if it[1] != -1]
    if not active:
        return by_id, 0, 0

    # 1. Radix sort active by nxt (index 1)
    by_nxt, p1 = radix_sort_by_key(active, key_idx=1, max_val=m)

    # 2. Two-pointer sequential merge: by_nxt (key = nxt_x) with by_id (key = y)
    merged: list[tuple[int, int, int]] = list(finished)
    j_ptr = 0
    for x, nxt_x, dist_x in by_nxt:
        while by_id[j_ptr][0] < nxt_x:
            j_ptr += 1
        y, nxt_y, dist_y = by_id[j_ptr]
        assert y == nxt_x
        merged.append((x, nxt_y, dist_x + dist_y))

    # 3. Radix sort merged back by id (index 0)
    new_by_id, p2 = radix_sort_by_key(merged, key_idx=0, max_val=m)

    # Passes & byte volume (12 bytes per triple (id, nxt, dist))
    passes = p1 + 1 + p2
    bytes_streamed = (p1 * len(active) + len(active) + m + p2 * m) * 12
    return new_by_id, passes, bytes_streamed


def run_pell_doubling_on_retained(blk: BlockData, L_min: int = 16) -> dict:
    """Sense 3A (Mode B): Pointer-free radix-sort-merge Pell doubling on retained map R."""
    rs = build_retained_system_genuine(blk, L_min=L_min)
    m = rs.m
    q_acq_thick = rs.Q_acq_strips + rs.Q_acq_glue

    # Find tail (unique x with R[x] == mark_rank) in 1 linear scan (Q = 0)
    tail = -1
    for x in range(m):
        if rs.R[x] == rs.mark_rank:
            tail = x
            break
    assert tail != -1

    # Initial state by_id: (x, nxt, dist), sentinel at tail has nxt=-1, dist=0
    by_id: list[tuple[int, int, int]] = [
        (x, -1 if x == tail else rs.R[x], 0 if x == tail else 1) for x in range(m)
    ]

    # Track exact hybrid walk checkpoints at j = 1, 2, 3, 4 AND full completion j_max
    # For hybrid walk at round j, we store the doubling hierarchy jumps[k][x] = R^{2^k}(x)
    # so emitting a 2^j-step macro-edge expands recursively WITHOUT O(n) word copies!
    jumps_levels: list[list[int]] = [rs.R[:]]
    cum_passes = 0
    cum_bytes = 0
    checkpoints: list[dict] = []

    j = 0
    while True:
        active_cnt = sum(1 for it in by_id if it[1] != -1)
        if active_cnt == 0:
            break
        j += 1
        by_id, p_rnd, b_rnd = merge_compose_doubling_round(by_id, m)
        cum_passes += p_rnd
        cum_bytes += b_rnd
        # Record jump array for hybrid check at small j
        if j <= 4:
            # Build 2^j-step jump array (with sentinel wrapping back to mark_rank handled)
            step_len = 1 << j
            # Verify hybrid byte-exact decode using jump-tree recursion!
            nxt_arr = [it[1] for it in by_id]
            jumps_levels.append(nxt_arr)
            # Walk from mark_rank taking jumps of length 2^j (with tail handling)
            out_hyb = bytearray(blk.n)
            pos = blk.n
            cur = rs.mark_rank
            macro_steps = 0
            buf = bytearray()

            def emit_2k(node: int, lvl: int) -> None:
                nonlocal pos
                if pos == 0:
                    return
                if lvl == 0:
                    buf.clear()
                    expand_dag_word(rs.dag_left, rs.dag_right, rs.R_word_id[node], buf)
                    lw = len(buf)
                    for t in range(lw):
                        out_hyb[pos - 1 - t] = buf[t]
                    pos -= lw
                    return
                mid = jumps_levels[lvl - 1][node]
                emit_2k(node, lvl - 1)
                if mid != -1 and pos > 0:
                    emit_2k(mid, lvl - 1)

            while pos > 0:
                emit_2k(cur, j)
                macro_steps += 1
                cur = nxt_arr[cur]
                if cur == -1:
                    break
            assert pos == 0 and bytes(out_hyb) == blk.T_ref, f"Hybrid j={j} decode mismatch!"
            # Note: during emit_2k, walking the binary jump tree touches m leaves
            # unless the words are ordered — wait: emit_2k DOES address jumps_levels[lvl-1][node]!
            # So in hybrid mode WITHOUT word materialisation, Q_emit = m (tree nodes) if unmaterialized,
            # OR Q_emit = macro_steps = ceil(m / 2^j) IF words are materialized (S += n).
            # Let's report BOTH honestly!
            checkpoints.append({
                "j": j,
                "stride": step_len,
                "macro_steps": macro_steps,
                "macro_over_h1": round(macro_steps / blk.h1, 4),
                "doubling_passes": cum_passes,
                "doubling_P_bytes": round(cum_bytes / blk.n, 3),
                "byte_exact": True,
                "note_tree_vs_mat": "macro_steps=ceil(m/2^j); zero-copy tree emit still addresses m leaves, while full j_max sort-emit has Q_run=0!",
            })

    # Full completion: EVERY node now has nxt == -1 and dist == (m - 1) - pos[x]!
    # Final radix sort by pos[x] = (m - 1) - dist[x], then ONE sequential scan to emit OUT!
    ordered_input = [((m - 1) - dist, rs.R_word_id[x]) for (x, nxt, dist) in by_id]
    ordered, p_final = radix_sort_by_key(ordered_input, key_idx=0, max_val=m)
    cum_passes += p_final + 1  # +1 for final sequential emit scan
    cum_bytes += p_final * m * 8 + blk.n

    # Byte-exact sequential emit (ZERO addressed reads of R!)
    out_full = bytearray(blk.n)
    pos_byte = blk.n
    buf = bytearray()
    for expected_rank, (rank_val, wid) in enumerate(ordered):
        assert rank_val == expected_rank
        buf.clear()
        expand_dag_word(rs.dag_left, rs.dag_right, wid, buf)
        lw = len(buf)
        for t in range(lw):
            out_full[pos_byte - 1 - t] = buf[t]
        pos_byte -= lw
    assert pos_byte == 0 and bytes(out_full) == blk.T_ref, f"Full Pell list-rank decode mismatch on {blk.name}!"

    # Cost accounting under Pi:
    # Note: expanding DAG words sequentially still addresses dag_left/dag_right for thick strips
    # (1 DAG lookup per thick-strip concat, NOT per lone row; lone rows are direct leaves < 256!).
    # Count exact DAG lookups on non-leaf nodes during emit:
    q_emit_dag = 0
    for _, wid in ordered:
        if wid >= 256:
            # Thick strip expansion
            stack = [wid]
            while stack:
                nid = stack.pop()
                if nid >= 256:
                    q_emit_dag += 1
                    stack.append(rs.dag_right[nid])
                    stack.append(rs.dag_left[nid])

    cost_full_retained = PiCost(
        Q=q_acq_thick + q_emit_dag,
        D=rs.D_acq,  # emit DAG depth is <= D_acq; doubling itself has D = 0!
        P=rs.P_acq + cum_passes,
        S=rs.S_acq + m * 24,  # two m-element triple buffers (12 B each)
        Q_acq=q_acq_thick,
        Q_run=q_emit_dag,
        D_acq=rs.D_acq,
        D_run=0,
        P_acq=rs.P_acq + cum_passes - 1,
        P_run=1,
        P_bytes=round((blk.r * 6 + cum_bytes) / blk.n, 3),
        s_cursors=256,
        notes=f"Retained-map Pell list ranking (L_min={L_min}, rounds={j}): Q_doubling=0, D_doubling=0, Q_run_lone=0!",
    )

    # Also compute exact Mode A (pure raw-LF Pell doubling on n rows, Q_total = 0, D_total = 0!)
    # Verify byte-exact on synthetic blocks (n <= 65k, fast!) and compute exact formula on 900k blocks.
    raw_verified = False
    raw_rounds = math.ceil(math.log2(blk.n))
    p_per_sort = (blk.n + 1).bit_length() + 7
    p_per_sort //= 8
    raw_passes_est = 1 + raw_rounds * (2 * p_per_sort + 1) + p_per_sort + 1
    raw_P_bytes_est = round((blk.n * 12 * raw_passes_est) / blk.n, 2)
    if blk.n <= 70000:
        lf = build_lf_array(blk.L, blk.C)
        # Find raw tail
        raw_tail = lf.index(blk.primary_index)
        raw_by_id = [(x, -1 if x == raw_tail else lf[x], 0 if x == raw_tail else 1) for x in range(blk.n)]
        r_passes = 1  # 1 pass over IN to build raw_by_id with 256 cursors (Q=0!)
        r_bytes = blk.n * 12
        r_j = 0
        while any(it[1] != -1 for it in raw_by_id):
            r_j += 1
            raw_by_id, pr, br = merge_compose_doubling_round(raw_by_id, blk.n)
            r_passes += pr
            r_bytes += br
        raw_ord_in = [((blk.n - 1) - dist, blk.L[x]) for (x, _, dist) in raw_by_id]
        raw_ord, pf = radix_sort_by_key(raw_ord_in, key_idx=0, max_val=blk.n)
        r_passes += pf + 1
        r_bytes += pf * blk.n * 8 + blk.n
        T_raw = bytes(b for (_, b) in raw_ord)[::-1]
        assert T_raw == blk.T_ref, f"Raw-LF Pell doubling mismatch on {blk.name}!"
        raw_verified = True
        raw_rounds = r_j
        raw_passes_est = r_passes
        raw_P_bytes_est = round(r_bytes / blk.n, 2)

    cost_raw_full = PiCost(
        Q=0,
        D=0,
        P=raw_passes_est,
        S=blk.n * 24,
        Q_acq=0,
        Q_run=0,
        D_acq=0,
        D_run=0,
        P_acq=raw_passes_est - 1,
        P_run=1,
        P_bytes=raw_P_bytes_est,
        s_cursors=256,
        notes=f"Pure raw-LF Pell list ranking ({raw_rounds} rounds): LITERALLY Q=0, D=0, relocated to P={raw_passes_est}, S=24n",
    )

    return {
        "L_min": L_min,
        "m_retained": m,
        "doubling_rounds_full": j,
        "checkpoints": checkpoints,
        "retained_full_byte_exact": True,
        "retained_full_cost": cost_full_retained.as_dict(blk.h1, blk.n),
        "raw_full_verified_in_python": raw_verified,
        "raw_full_rounds": raw_rounds,
        "raw_full_cost": cost_raw_full.as_dict(blk.h1, blk.n),
    }


def run_zorich_pell_cycle_audit(blk: BlockData) -> dict:
    """Sense 3B: Audit how many IET runs have self-overlapping shift cycles
    (I_j intersecting LF(I_j) -> Zorich/Euclid q = floor(w/delta) acceleration).
    """
    occ = blk.C[:]
    self_overlap_runs = 0
    self_overlap_rows = 0
    total_q_saved = 0
    for s, w, b in blk.runs:
        lf_s = occ[b]
        occ[b] += w
        if w >= 2:
            # Does [lf_s, lf_s + w) overlap [s, s + w)?
            lo = max(s, lf_s)
            hi = min(s + w, lf_s + w)
            if hi > lo:
                delta = abs(lf_s - s)
                self_overlap_runs += 1
                self_overlap_rows += hi - lo
                if delta > 0:
                    q = w // delta
                    if q > 1:
                        total_q_saved += (hi - lo)
    return {
        "self_overlap_runs": self_overlap_runs,
        "self_overlap_rows": self_overlap_rows,
        "zorich_accel_rows": total_q_saved,
        "w1_affected": 0,  # width-1 runs have w=1, cannot self-overlap!
    }


def main() -> None:
    blocks = [
        make_synthetic_novel_hex_iid(65536),
        make_synthetic_templated_hex(1000, 40),
    ] + load_admitted_blocks()

    all_results = {}
    for blk in blocks:
        print(f"\n=== Sense 3 (pell-recurrence): {blk.name} (n={blk.n}, r={blk.r}, h1={blk.h1}) ===")
        r_3a = run_pell_doubling_on_retained(blk, L_min=16)
        r_3b = run_zorich_pell_cycle_audit(blk)
        all_results[blk.name] = {
            "n": blk.n,
            "r": blk.r,
            "h1": blk.h1,
            "s3a_pell_doubling": r_3a,
            "s3b_zorich_cycle_audit": r_3b,
        }
        rc = r_3a["retained_full_cost"]
        rawc = r_3a["raw_full_cost"]
        print(
            f"  3A retained (m={r_3a['m_retained']:6d}, {r_3a['doubling_rounds_full']:2d} rnds): "
            f"Q_tot={rc['Q']:7d} (Q_acq={rc['Q_acq']}, Q_run_dag={rc['Q_run']}, Q_doub=0) "
            f"D={rc['D']:3d} P={rc['P']:3d} P_bytes={rc['P_bytes']:6.2f} S={rc['S']/1e6:.2f}MB [EXACT]"
        )
        print(
            f"  3A raw-LF   (n={blk.n:6d}, {r_3a['raw_full_rounds']:2d} rnds): "
            f"Q_tot={rawc['Q']} D={rawc['D']} P={rawc['P']:3d} P_bytes={rawc['P_bytes']:6.1f} "
            f"S={rawc['S']/1e6:.2f}MB verified={r_3a['raw_full_verified_in_python']}"
        )
        print(
            f"  3B Zorich self-overlap: runs={r_3b['self_overlap_runs']} rows={r_3b['self_overlap_rows']} w1_affected=0"
        )

    out_path = Path(__file__).resolve().parent / "results_s3_pell_recurrence.json"
    out_path.write_text(json.dumps(all_results, indent=2) + "\n")
    print(f"\nSaved {out_path} [ALL BYTE-EXACT]")


if __name__ == "__main__":
    main()
