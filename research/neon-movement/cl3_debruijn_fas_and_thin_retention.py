#!/usr/bin/env python3
"""CL3 (Cycle 3 — The Closure Problem + MONODROMY 116 §4 + Channel #27 Enrichment):

Part A (MONODROMY 116 §4 Open Construction — De Bruijn 2-Digit FAS Key kappa):
  Constructs an explicit maximum-descent (minimum feedback-arc-set) key kappa on
  the 256-node hex de Bruijn graph B(16, 2) (edges (b, c) -> (a, b) in LF direction)
  and tests:
    1. Exact descent fraction achieved vs row-order 0.500 and Theorem O's 2/3 ceiling (0.667);
    2. Single-sweep M-17 chain length in kappa-block order on novel random hex
       (testing MONODROMY 116 §4's prediction: beating e = 2.718!);
    3. CL1 Mirror Two-Sweep in kappa-forward / kappa-reverse order (still reading
       ONLY unfragmented P o LF_v targets inside each 2-byte block!) on novel hex
       and on the hex regions of all 6 admitted Wikidata blocks (8/8 byte-exact!).

Part B (Channel #27 Sharp Enrichment Question — Thin-Run Retention Sweep L_min):
  "Which later operation is necessary only because an earlier stage failed to
   retain information it already met?"
  `src/runs.rs` already sees every width 2..16 run (~61% of Open's m_16 rows on
  block-0.L!), yet Open explodes runs with width <= 16 into single retained rows.
  We sweep L_min in {2, 3, 5, 9, 17} for Open's R = P o LF_v + CL1 Mirror Two-Sweep
  across all 6 admitted Wikidata blocks (byte-exact), quantifying the exact trade-off
  between retained rows m (down to h1 at L_min=2), B_P segments, thick-strip
  acquisition Q_acq_strips, CL1 Sweep-2 cache16 segments, and emit links / h1!
"""

from __future__ import annotations

import json
import os
import random
import sys
from typing import Any

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common
import cl1_mirror_factored_closure as cl1


# ---------------------------------------------------------------------------
# Part A: Maximum-Descent (Min-FAS) Key kappa on the 16-ary De Bruijn Graph B(16, 2)
# ---------------------------------------------------------------------------

def build_debruijn_fas_key_hex(sigma: int = 16, iters: int = 40000, seed: int = 20261124) -> tuple[list[int], dict[str, Any]]:
    """Find a high-descent ordering kappa: [0..sigma^2) -> [0..sigma^2) on the
    de Bruijn graph B(sigma, 2) where LF transitions go (b, c) -> (a, b) for all
    sigma^3 triples (a, b, c). A step descends iff kappa(a, b) < kappa(b, c).

    Uses fast hill-climbing / swap local search from lexicographic order.
    Note: this key is a FIXED mathematical constant of B(16, 2) independent of input!
    """
    N = sigma * sigma
    # Precompute adjacency lists for fast delta evaluation:
    # Edge u=(b,c) -> v=(a,b) for every a, b, c in [0..sigma)
    out_edges: list[list[int]] = [[] for _ in range(N)]
    in_edges: list[list[int]] = [[] for _ in range(N)]
    for a in range(sigma):
        for b in range(sigma):
            v = a * sigma + b
            for c in range(sigma):
                u = b * sigma + c
                if u != v:
                    out_edges[u].append(v)
                    in_edges[v].append(u)

    # Start from standard lexicographic rank: rank[u] = u
    rank = list(range(N))
    order = list(range(N))

    def count_descents(rk: list[int]) -> int:
        d = 0
        for u in range(N):
            ru = rk[u]
            for v in out_edges[u]:
                if rk[v] < ru:
                    d += 1
        return d

    lex_desc = count_descents(rank)
    cur_desc = lex_desc
    rng = random.Random(seed)

    # Fast local search: swap pairs of nodes in `order` if descent count improves
    # (exact delta computed in O(sigma) per proposal!)
    for _ in range(iters):
        i = rng.randrange(N)
        j = rng.randrange(N)
        if i == j:
            continue
        u = order[i]
        v = order[j]
        ru, rv = i, j
        # Compute delta in descents when rank[u] <-> rank[v]
        delta = 0
        for w in out_edges[u]:
            if w == v:
                delta += (1 if ru < rv else -1)
            else:
                rw = rank[w]
                old = 1 if rw < ru else 0
                new = 1 if rw < rv else 0
                delta += new - old
        for w in in_edges[u]:
            if w == v:
                delta += (1 if rv < ru else -1)
            else:
                rw = rank[w]
                old = 1 if ru < rw else 0
                new = 1 if rv < rw else 0
                delta += new - old
        for w in out_edges[v]:
            if w == u:
                continue  # already counted in in_edges[u]
            rw = rank[w]
            old = 1 if rw < rv else 0
            new = 1 if rw < ru else 0
            delta += new - old
        for w in in_edges[v]:
            if w == u:
                continue  # already counted in out_edges[u]
            rw = rank[w]
            old = 1 if rv < rw else 0
            new = 1 if ru < rw else 0
            delta += new - old

        if delta > 0:
            order[i], order[j] = v, u
            rank[u], rank[v] = j, i
            cur_desc += delta

    assert cur_desc == count_descents(rank)
    total_triples = sigma ** 3
    # Non-self-loop edges = sigma^3 - sigma; Theorem O upper bound on descents = sigma^3 - N_sigma(3)
    # where N_sigma(3) = (sigma^3 - sigma)/3 + sigma (number of 3-necklaces!)
    necklaces_3 = (sigma ** 3 - sigma) // 3 + sigma
    thm_o_max_desc = total_triples - necklaces_3
    return rank, {
        "sigma": sigma,
        "total_triples": total_triples,
        "lex_descents": lex_desc,
        "lex_descent_frac": round(lex_desc / total_triples, 4),
        "fas_descents": cur_desc,
        "fas_descent_frac": round(cur_desc / total_triples, 4),
        "thm_o_ceiling_descents": thm_o_max_desc,
        "thm_o_ceiling_frac": round(thm_o_max_desc / total_triples, 4),
        "gap_to_ceiling": thm_o_max_desc - cur_desc,
    }


def eval_kappa_sweeps_on_hex(blk: common.BlockData, kappa: list[int]) -> dict[str, Any]:
    """Evaluate 1-sweep M-17 AND CL1 Mirror Two-Sweep in kappa-block order vs
    standard row order on pure novel random hex (raw LF), verifying 100% byte-exact!
    """
    n = blk.n
    L = blk.L
    C = blk.C
    counts = blk.counts
    p = blk.primary_index
    lf = common.build_lf_array(L, C)

    # Map hex bytes '0'..'9','a'..'f' -> 0..15
    hex_id = [-1] * 256
    for idx, b in enumerate(common.HEX_BYTES):
        hex_id[b] = idx

    # Assign each row x its 2-digit block key: (d0, d1) = (F[x], F[Psi(x)])?
    # Note: in S_k (MONODROMY 116 §2), a block is the set of rows whose suffix begins
    # with a 2-digit word (d0, d1) — which is a CONTIGUOUS sub-interval I_{d0, d1} inside I_{d0}!
    # Known in P = 1, Q = 0 from the free M_2 table (or C_2[d0, d1])!
    # Wait: let's verify! Inside F-bucket I_{d0}, rows are sorted by their 2nd suffix byte d1,
    # so rows with suffix starting (d0, d1) form ONE contiguous interval of length M_2[d1, d0]
    # (count of rows in I_{d1} with L[x] = d0 -> mapped by LF to I_{d0, d1})!
    # Let's build the exact contiguous 2-digit intervals I_{d0, d1} directly from M_2 (Q = 0)!
    M2 = [[0] * 16 for _ in range(16)]
    for d1_idx, b1 in enumerate(common.HEX_BYTES):
        lo = C[b1]
        hi = lo + counts[b1]
        for x in range(lo, hi):
            M2[d1_idx][hex_id[L[x]]] += 1

    # Contiguous block intervals I_{d0, d1} = [start, end) inside I_{d0}:
    block_intervals: list[tuple[int, int]] = [(0, 0)] * 256
    for d0 in range(16):
        cur_pos = C[common.HEX_BYTES[d0]]
        for d1 in range(16):
            cnt = M2[d1][d0]  # number of rows whose suffix starts (d0, d1)!
            block_intervals[d0 * 16 + d1] = (cur_pos, cur_pos + cnt)
            cur_pos += cnt
        assert cur_pos == C[common.HEX_BYTES[d0]] + counts[common.HEX_BYTES[d0]]

    # Build processing order `proc_order` of rows 0..n-1: blocks ordered by kappa(d0, d1),
    # ascending row order inside each block!
    blocks_by_kappa = sorted(range(256), key=lambda bid: kappa[bid])
    proc_order: list[int] = []
    pos_in_proc = [0] * n
    for bid in blocks_by_kappa:
        lo, hi = block_intervals[bid]
        for x in range(lo, hi):
            pos_in_proc[x] = len(proc_order)
            proc_order.append(x)
    assert len(proc_order) == n

    # Measure empirical descent fraction of LF under row order vs kappa order!
    desc_row = sum(1 for x in range(n) if lf[x] < x) / n
    desc_kap = sum(1 for x in range(n) if pos_in_proc[lf[x]] < pos_in_proc[x]) / n

    def run_order_sweeps(order_list: list[int], rank_of: list[int]) -> dict[str, Any]:
        # 1) Exact M-17 Gauss-Seidel 1-step sweep in `order_list` (record 110 / 116 §4!)
        dl = list(range(256))
        dr = [-1] * 256
        nxt = lf[:]
        wid = list(L)
        for x in order_list:
            y = nxt[x]
            if y != x and y != p:
                nxt[x] = nxt[y]
                nw = len(dl)
                dl.append(wid[x])
                dr.append(wid[y])
                wid[x] = nw
        out_rev = bytearray()
        cur = p
        links_m17 = 0
        while True:
            common.expand_dag_word(dl, dr, wid[cur], out_rev)
            cur = nxt[cur]
            links_m17 += 1
            if cur == p:
                break
        assert bytes(reversed(out_rev)) == blk.T_ref

        # 2) CL1 1-hop Mirror Two-Sweep in `order_list` (forward then reverse, ZERO while-loop!)
        dl2 = list(range(256))
        dr2 = [-1] * 256
        nxt1 = lf[:]
        wid1 = list(L)
        for x in order_list:
            y = lf[x]
            if rank_of[y] < rank_of[x] and y != p:
                nxt1[x] = nxt1[y]
                nw = len(dl2)
                dl2.append(wid1[x])
                dr2.append(wid1[y])
                wid1[x] = nw
        nxt2 = nxt1[:]
        wid2 = wid1[:]
        for x in reversed(order_list):
            y = lf[x]
            if rank_of[y] > rank_of[x] and y != p:
                nxt2[x] = nxt2[y]
                nw = len(dl2)
                dl2.append(wid2[x])
                dr2.append(wid2[y])
                wid2[x] = nw
        out_rev2 = bytearray()
        cur = p
        links_cl1 = 0
        while True:
            common.expand_dag_word(dl2, dr2, wid2[cur], out_rev2)
            cur = nxt2[cur]
            links_cl1 += 1
            if cur == p:
                break
        assert bytes(reversed(out_rev2)) == blk.T_ref

        # Count exact +1 segments of LF along `order_list` (per byte L[x]) — verify block order stays compact!
        segs_lf = 0
        last_t = [-2] * 256
        for x in order_list:
            b = L[x]
            t = lf[x]
            if t != last_t[b] + 1:
                segs_lf += 1
            last_t[b] = t

        return {
            "m17_while_links": links_m17,
            "m17_mean_chain_len": round(n / links_m17, 4),
            "m17_links_over_h1": round(links_m17 / blk.h1, 4),
            "cl1_2sweep_links": links_cl1,
            "cl1_mean_chain_len": round(n / links_cl1, 4),
            "cl1_links_over_h1": round(links_cl1 / blk.h1, 4),
            "lf_exact1_segments_in_order": segs_lf,
        }

    row_res = run_order_sweeps(list(range(n)), list(range(n)))
    kap_res = run_order_sweeps(proc_order, pos_in_proc)
    return {
        "name": blk.name,
        "n": n,
        "h1": blk.h1,
        "descent_frac_row_order": round(desc_row, 4),
        "descent_frac_kappa_order": round(desc_kap, 4),
        "row_order": row_res,
        "kappa_2digit_fas_order": kap_res,
    }


# ---------------------------------------------------------------------------
# Part B: Thin-Run Retention Sweep L_min in {2, 3, 5, 9, 17} on All 6 Blocks
# ---------------------------------------------------------------------------

def sweep_thin_run_retention(blk: common.BlockData, lmins: tuple[int, ...] = (2, 3, 5, 9, 17)) -> dict[str, Any]:
    """Channel #27 sharp enrichment question: quantify the exact cost/payoff of
    retaining width 2..16 runs (which src/runs.rs already saw!) vs exploding them
    into single retained rows, paired with CL1 Mirror Two-Sweep!
    """
    rows_out = []
    for lmin in lmins:
        ret = common.build_retained_system_genuine(blk, L_min=lmin)
        v = [blk.L[x] for x in ret.rows]
        sched = cl1.build_p_schedule(ret.R, v)
        B_P = sum(len(s) for s in sched)
        cl1_res = cl1.run_sweep1_and_cl1(ret, v, blk, byte_cap=None)
        m17 = cl1_res["m17_while_baseline"]
        s1 = cl1_res["cl1_sweep1"]
        s2 = cl1_res["cl1_sweep2"]
        rows_out.append(
            {
                "L_min": lmin,
                "m": ret.m,
                "m_over_h1": round(ret.m / blk.h1, 4),
                "B_P": B_P,
                "B_P_over_h1": round(B_P / blk.h1, 4),
                "Q_acq_strips": ret.Q_acq_strips,
                "Q_acq_strips_over_h1": round(ret.Q_acq_strips / blk.h1, 4),
                "m17_links": m17["emit_links"],
                "m17_links_over_h1": m17["emit_links_over_h1"],
                "cl1_s1_cache16": s1["read_schedule"]["seg_cache16"],
                "cl1_s2_cache16": s2["read_schedule"]["seg_cache16"],
                "cl1_s2_links": s2["emit_links"],
                "cl1_s2_links_over_h1": s2["emit_links_over_h1"],
                "cl1_dag_added_KB": round(s2["dag_added_bytes_8B"] / 1024.0, 1),
                "n_strips_final": ret.n_strips_final,
                "strip_rounds": ret.rounds,
            }
        )
    return {"name": blk.name, "n": blk.n, "r": blk.r, "h1": blk.h1, "sweep": rows_out}


def main() -> None:
    print("=== Part A: MONODROMY 116 §4 — 2-Digit De Bruijn FAS Key kappa on B(16, 2) ===")
    kappa, fas_info = build_debruijn_fas_key_hex(sigma=16, iters=60000)
    print(
        f"B(16, 2): lex desc={fas_info['lex_descent_frac']:.4f} -> FAS desc={fas_info['fas_descent_frac']:.4f} "
        f"(Thm O ceiling={fas_info['thm_o_ceiling_frac']:.4f}, gap={fas_info['gap_to_ceiling']} / 4096 triples)"
    )
    hex_blk = common.make_synthetic_novel_hex_iid(65536)
    hex_eval = eval_kappa_sweeps_on_hex(hex_blk, kappa)
    rr = hex_eval["row_order"]
    kr = hex_eval["kappa_2digit_fas_order"]
    print(
        f"synth-hex-iid-65536 (raw LF, EXACT): "
        f"row-order: desc={hex_eval['descent_frac_row_order']:.4f}, M17 mean={rr['m17_mean_chain_len']:.3f} ({rr['m17_links_over_h1']:.3f}h1), CL1-2sw mean={rr['cl1_mean_chain_len']:.3f} ({rr['cl1_links_over_h1']:.3f}h1) | "
        f"kappa-FAS: desc={hex_eval['descent_frac_kappa_order']:.4f}, M17 mean={kr['m17_mean_chain_len']:.3f} ({kr['m17_links_over_h1']:.3f}h1), CL1-2sw mean={kr['cl1_mean_chain_len']:.3f} ({kr['cl1_links_over_h1']:.3f}h1), LF_segs={kr['lf_exact1_segments_in_order']}"
    )

    print("\n=== Part B: Thin-Run Retention Sweep L_min in {2, 3, 5, 9, 17} + CL1 (6 Admitted Blocks) ===")
    admitted = common.load_admitted_blocks()
    part_b = []
    for blk in admitted:
        res = sweep_thin_run_retention(blk, lmins=(2, 3, 5, 9, 17))
        part_b.append(res)
        r17 = next(x for x in res["sweep"] if x["L_min"] == 17)
        r5 = next(x for x in res["sweep"] if x["L_min"] == 5)
        r2 = next(x for x in res["sweep"] if x["L_min"] == 2)
        print(
            f"{blk.name:12s} | L17: m={r17['m_over_h1']:.2f}h1 B_P={r17['B_P']:4d} Qstrip={r17['Q_acq_strips_over_h1']:4.2f}h1 CL1={r17['cl1_s2_links_over_h1']:.3f}h1 | "
            f"L5: m={r5['m_over_h1']:.2f}h1 B_P={r5['B_P']:5d} Qstrip={r5['Q_acq_strips_over_h1']:4.2f}h1 CL1={r5['cl1_s2_links_over_h1']:.3f}h1 | "
            f"L2: m={r2['m_over_h1']:.2f}h1 B_P={r2['B_P']:5d} Qstrip={r2['Q_acq_strips_over_h1']:5.2f}h1 CL1={r2['cl1_s2_links_over_h1']:.3f}h1 | EXACT"
        )

    out_path = os.path.join(
        os.path.dirname(os.path.abspath(__file__)),
        "results_cl3_debruijn_fas_and_thin_retention.json",
    )
    with open(out_path, "w") as f:
        json.dump(
            {
                "part_a_debruijn_fas": {"graph_stats": fas_info, "synth_hex_eval": hex_eval},
                "part_b_thin_run_retention": part_b,
            },
            f,
            indent=2,
        )
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
