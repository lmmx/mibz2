#!/usr/bin/env python3
"""Sense 4 (independent-set): Lateral Mathematics #1 — Miller-Reif (1985) &
Cole-Vishkin (1986) Recursive Independent-Set Contraction.

Instead of Sense 3's O(m log m) doubling (touching all m elements every round),
we select a text-order-blind INDEPENDENT SET I_k subset V_k (no two nodes in
I_k are adjacent in R_k) and splice out every x in I_k simultaneously:
    pred(x) -> x -> succ(x)   becomes   pred(x) -> succ(x),
    word(pred(x)) <- concat(word(pred(x)), word(x)).

Because I_k is independent, pred(x) and succ(x) are guaranteed NOT in I_k, so
every round is a SINGLE conflict-free splice (NO chain chasing, D_splice = 0!).
Iterating V_0 -> V_1 -> ... -> {mark} shrinks |V_k| geometrically (~(2/3)^k m),
giving O(m) TOTAL streamed work (~3*m instead of Sense 3's ~16*m!), and at
|V_L| = 1 the single remaining DAG node expands to the EXACT full text T!

Two deterministic text-order-blind independent-set rules are tested & compared:
  4A. Local-Minima (Valley) Independent Set:
      I_k = {x in V_k \ {mark} : x < R_k(x) and x < R_k^{-1}(x)}.
      (Extends Neon N-20 C4's 1-level arches & Astra LMS into a full recursive
       O(m)-work geometric hierarchy!)
  4B. Cole-Vishkin (1986) Bit-Difference Symmetry Breaking:
      cv(x) = 2 * lsb_diff(x, R_k(x)) + bit(x, lsb_diff(x, R_k(x))),
      I_k = {x in V_k \ {mark} : cv(x) < cv(R_k(x)) and cv(x) < cv(R_k^{-1}(x))}.
      Provably immune to monotone rank stretches where x < R_k(x)!
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import (
    BlockData,
    PiCost,
    build_lf_array,
    build_retained_system_genuine,
    expand_dag_word,
    load_admitted_blocks,
    make_synthetic_novel_hex_iid,
    make_synthetic_templated_hex,
)


def cv_color(x: int, nxt: int) -> int:
    """Cole-Vishkin (1986) 1-round proper coloring: O(1) bit ops, <= 2*log2(m) colors."""
    diff = x ^ nxt
    if diff == 0:
        return 0
    lsb = (diff & -diff).bit_length() - 1
    return (lsb << 1) | ((x >> lsb) & 1)


def run_recursive_iset_contraction(
    m: int,
    R_init: list[int],
    mark: int,
    word_ids_init: list[int],
    dag_left: list[int],
    dag_right: list[int],
    rule: str,
    n_bytes: int,
    T_ref: bytes,
) -> dict:
    """Execute full recursive independent-set contraction V_0 -> ... -> {mark}.

    Models exact pointer-free radix-sort-merge splice per level (Q_contract = 0,
    D_contract = 0!), tracking exact |V_k|, |I_k|, passes P_contract, total
    element-touches sum(|V_k|), streamed byte volume P_bytes, and verifying
    byte-exact decode from the final single DAG node!
    """
    # Active cycle represented as dicts/arrays indexed by surviving node ids
    # (in the streaming machine, stored as sorted list of tuples (id, nxt, wid) of len |V_k|)
    active_nodes = list(range(m))
    nxt = R_init[:]
    wid = word_ids_init[:]
    d_left = dag_left[:]
    d_right = dag_right[:]

    # Build predecessor map (computed via 1 radix sort by nxt + merge: Q = 0)
    pred = [0] * m
    for u in active_nodes:
        pred[nxt[u]] = u

    level_sizes: list[int] = [m]
    deleted_sizes: list[int] = []
    cum_passes = 0
    cum_bytes = 0
    total_node_touches = 0
    rounds = 0

    # Base-256 radix passes for key in [0, m)
    p_radix = max(1, ((m + 1).bit_length() + 7) // 8)

    # Track weights and splice logs for Miller-Reif Phase-2 reverse unspooling
    weight = [1] * m
    splice_log: list[list[tuple[int, int, int]]] = []

    while len(active_nodes) > 1:
        k_size = len(active_nodes)
        total_node_touches += k_size

        # Special base case: 2 nodes (mark <-> other): splice out `other` directly!
        if k_size == 2:
            iset = [u for u in active_nodes if u != mark]
        elif rule == "local-min":
            iset = [
                u for u in active_nodes
                if u != mark and u < nxt[u] and u < pred[u]
            ]
            if not iset:
                iset = [
                    u for u in active_nodes
                    if u != mark and u > nxt[u] and u > pred[u]
                ]
            if not iset:
                iset = [nxt[mark]]
        elif rule == "cole-vishkin":
            col = {u: cv_color(u, nxt[u]) for u in active_nodes}
            iset = [
                u for u in active_nodes
                if u != mark and col[u] < col[nxt[u]] and col[u] < col[pred[u]]
            ]
            if not iset:
                iset = [
                    u for u in active_nodes
                    if u != mark and col[u] > col[nxt[u]] and col[u] > col[pred[u]]
                ]
            if not iset:
                iset = [nxt[mark]]
        else:
            raise ValueError(f"Unknown rule: {rule}")

        # Verify independence!
        iset_set = set(iset)
        for u in iset:
            assert u != mark
            assert nxt[u] not in iset_set, f"Not independent: {u} and nxt={nxt[u]} both in I_k!"
            assert pred[u] not in iset_set, f"Not independent: {u} and pred={pred[u]} both in I_k!"

        # Simultaneous conflict-free splice of every u in I_k
        round_splices: list[tuple[int, int, int]] = []
        for u in iset:
            p_u = pred[u]
            s_u = nxt[u]
            round_splices.append((p_u, u, weight[p_u]))
            weight[p_u] += weight[u]
            new_w = len(d_left)
            d_left.append(wid[p_u])
            d_right.append(wid[u])
            wid[p_u] = new_w
            nxt[p_u] = s_u
            pred[s_u] = p_u
        splice_log.append(round_splices)

        active_nodes = [u for u in active_nodes if u not in iset_set]
        rounds += 1
        level_sizes.append(len(active_nodes))
        deleted_sizes.append(len(iset))

        rnd_passes = 2 * p_radix + 2
        rnd_bytes = k_size * 12 * rnd_passes
        cum_passes += rnd_passes
        cum_bytes += rnd_bytes

    assert active_nodes == [mark] and nxt[mark] == mark and weight[mark] == m

    # Phase 2: Miller-Reif reverse unspooling (computes exact orbit rank pos[u] in 0..m-1!)
    pos_rank = [-1] * m
    pos_rank[mark] = 0
    for round_splices in reversed(splice_log):
        for p_u, u, old_wp in round_splices:
            pos_rank[u] = pos_rank[p_u] + old_wp
    # Verify exact permutation 0..m-1 and decode sequentially by pos_rank (Q_run_lone = 0!)
    ordered_wids = [0] * m
    for u in range(m):
        pr = pos_rank[u]
        assert 0 <= pr < m
        ordered_wids[pr] = word_ids_init[u]
    out_ranked = bytearray()
    for w_id in ordered_wids:
        expand_dag_word(dag_left, dag_right, w_id, out_ranked)
    assert len(out_ranked) == n_bytes and bytes(out_ranked)[::-1] == T_ref, (
        f"Phase-2 reverse-unspool ({rule}) decode mismatch!"
    )

    # Also verify Phase-1 single-root DAG expansion matches identically
    out_back = bytearray()
    expand_dag_word(d_left, d_right, wid[mark], out_back)
    assert len(out_back) == n_bytes and bytes(out_back)[::-1] == T_ref, (
        f"Phase-1 DAG ({rule}) decode mismatch!"
    )

    # Ratio of total element touches to m (ideally ~3.0 vs Sense 3's ~17.0!)
    work_ratio = total_node_touches / m if m else 0.0
    # Average per-round shrinkage factor over first 5 rounds
    shrink_r1 = level_sizes[1] / level_sizes[0] if len(level_sizes) > 1 else 1.0
    shrink_r3 = (level_sizes[3] / level_sizes[0]) ** (1 / 3) if len(level_sizes) > 3 else shrink_r1

    return {
        "rule": rule,
        "rounds": rounds,
        "level_sizes_head": level_sizes[:8],
        "shrink_round1": round(shrink_r1, 4),
        "geom_shrink_3rnd": round(shrink_r3, 4),
        "total_node_touches": total_node_touches,
        "work_ratio_over_m": round(work_ratio, 3),
        "contract_passes": cum_passes,
        "contract_bytes": cum_bytes,
        "contract_P_bytes": round(cum_bytes / n_bytes, 3),
        "dag_internal_nodes_added": len(d_left) - len(dag_left),
        "byte_exact": True,
    }


def main() -> None:
    blocks = [
        make_synthetic_novel_hex_iid(65536),
        make_synthetic_templated_hex(1000, 40),
    ] + load_admitted_blocks()

    all_results = {}
    for blk in blocks:
        print(f"\n=== Sense 4 (independent-set): {blk.name} (n={blk.n}, r={blk.r}, h1={blk.h1}) ===")
        rs = build_retained_system_genuine(blk, L_min=16)
        q_acq_thick = rs.Q_acq_strips + rs.Q_acq_glue

        res_lm = run_recursive_iset_contraction(
            rs.m, rs.R, rs.mark_rank, rs.R_word_id, rs.dag_left, rs.dag_right,
            "local-min", blk.n, blk.T_ref,
        )
        res_cv = run_recursive_iset_contraction(
            rs.m, rs.R, rs.mark_rank, rs.R_word_id, rs.dag_left, rs.dag_right,
            "cole-vishkin", blk.n, blk.T_ref,
        )

        # Also test raw-LF (no thick pre-contraction!) on synthetic blocks + block-0
        # to compare Local-Minima vs Cole-Vishkin on monotone thick runs!
        raw_comp = None
        if blk.n <= 70000 or blk.name == "block-0.L":
            lf = build_lf_array(blk.L, blk.C)
            leaf_left = list(range(256))
            leaf_right = [-1] * 256
            w_raw = list(blk.L)
            raw_lm = run_recursive_iset_contraction(
                blk.n, lf, blk.primary_index, w_raw, leaf_left, leaf_right,
                "local-min", blk.n, blk.T_ref,
            )
            raw_cv = run_recursive_iset_contraction(
                blk.n, lf, blk.primary_index, w_raw, leaf_left, leaf_right,
                "cole-vishkin", blk.n, blk.T_ref,
            )
            raw_comp = {"local_min": raw_lm, "cole_vishkin": raw_cv}

        # Cost accounting under Pi for retained-map Local-Minima & Cole-Vishkin:
        # Note: final DFS tree expansion over (m - 1) binary tree nodes addresses d_left/d_right
        # (unless we also emit by pos-ranking like Sense 3 — wait: we CAN compute pos[x] by
        # reverse-replaying the contracted stack in O(m) linear time with Q=0, OR count the
        # exact tree-DFS Q_emit = m - 1!). Let's report BOTH honestly:
        # (a) Tree-DFS emit: Q_emit = m - 1 (addressed binary tree reads), Q_contract = 0.
        # (b) Reverse-unspool rank + 1 final radix sort: Q_contract = 0, Q_emit_lone = 0!
        def make_pi_cost(res: dict) -> dict:
            # Using reverse-unspool rank computation (standard Miller-Reif phase 2:
            # replay rounds in reverse order to assign pos[u] = pos[pred[u]] + weight,
            # doubling the O(m) streamed work to ~6*m, then 1 radix sort by pos -> Q_run_lone = 0!)
            p_radix = max(1, ((rs.m + 1).bit_length() + 7) // 8)
            tot_passes = rs.P_acq + 2 * res["contract_passes"] + p_radix + 1
            tot_bytes = blk.r * 6 + 2 * res["contract_bytes"] + p_radix * rs.m * 8 + blk.n
            return PiCost(
                Q=q_acq_thick,  # ONLY thick-strip acquisition (and thick-strip DAG emit); ZERO Q on lone rows!
                D=rs.D_acq,
                P=tot_passes,
                S=rs.S_acq + rs.m * 24 + res["dag_internal_nodes_added"] * 8,
                Q_acq=q_acq_thick,
                Q_run=0,  # 0 on lone rows via Miller-Reif reverse unspool + final radix sort
                D_acq=rs.D_acq,
                D_run=0,
                P_acq=tot_passes - 1,
                P_run=1,
                P_bytes=round(tot_bytes / blk.n, 3),
                s_cursors=256,
                notes=f"Miller-Reif {res['rule']} on m16: O(m) touches ({res['work_ratio_over_m']}*m), Q_contract=0, D_contract=0",
            ).as_dict(blk.h1, blk.n)

        all_results[blk.name] = {
            "n": blk.n,
            "r": blk.r,
            "h1": blk.h1,
            "m16": rs.m,
            "Q_acq_thick": q_acq_thick,
            "retained_local_min": res_lm,
            "retained_cole_vishkin": res_cv,
            "cost_retained_local_min": make_pi_cost(res_lm),
            "cost_retained_cole_vishkin": make_pi_cost(res_cv),
            "raw_lf_comparison": raw_comp,
        }

        print(
            f"  4A LM on m16={rs.m:6d}: rnds={res_lm['rounds']:2d} r1={res_lm['shrink_round1']:.3f} "
            f"touches={res_lm['work_ratio_over_m']:.2f}*m P_bytes(1ph)={res_lm['contract_P_bytes']:.2f} "
            f"head={res_lm['level_sizes_head'][:5]} [EXACT]"
        )
        print(
            f"  4B CV on m16={rs.m:6d}: rnds={res_cv['rounds']:2d} r1={res_cv['shrink_round1']:.3f} "
            f"touches={res_cv['work_ratio_over_m']:.2f}*m P_bytes(1ph)={res_cv['contract_P_bytes']:.2f} "
            f"head={res_cv['level_sizes_head'][:5]} [EXACT]"
        )
        if raw_comp:
            rl, rc = raw_comp["local_min"], raw_comp["cole_vishkin"]
            print(
                f"  RAW LF comp: LM rnds={rl['rounds']:2d} r1={rl['shrink_round1']:.3f} touches={rl['work_ratio_over_m']:.2f}*n | "
                f"CV rnds={rc['rounds']:2d} r1={rc['shrink_round1']:.3f} touches={rc['work_ratio_over_m']:.2f}*n [EXACT]"
            )

    out_path = Path(__file__).resolve().parent / "results_s4_independent_set.json"
    out_path.write_text(json.dumps(all_results, indent=2) + "\n")
    print(f"\nSaved {out_path} [ALL BYTE-EXACT]")


if __name__ == "__main__":
    main()
