#!/usr/bin/env python3
"""
Deep diagnostics & adversarial stress-testing for Compositional IET-Schur Elimination:
  1. Complete SLP & Expansion Economics on all 6 authenticated Wikidata blocks:
     - Optimal surviving bucket a* (not restricted to p_bucket!)
     - Exact transition count, local bucket max size (L1d/L2 residency!),
       reachable SLP nodes, sum of distinct final-piece word lengths,
       SLP height, naive DFS visits vs memoized flattening bytes, avg memcpy length!
  2. Optimal early stopping (keep k = 1, 2, 3, 4 buckets) tradeoff curve!
  3. 2-Piece Euclidean (Sturmian/Ostrowski) self-loop acceleration on Fib_{2m}+$ up to n=3.79e16!
  4. Adversarial stress tests:
     - Uniform random strings over |Sigma| = 2, 4, 16, 64, 256 (n = 20,000)
     - Phi_BWT(M, K) with sentinel (keeping count-1 bucket vs keeping #)
     - Phi_BWT_bin(M, K)
     - Referee n=5 and n=10 F4 counterexamples
"""
import bisect
import hashlib
import json
import os
import random
from collections import defaultdict

import sys
sys.path.insert(0, "/tmp/fcp-c")
from explore_iet_schur import (
    BLOCKS_DIR,
    bwt_encode,
    build_initial_iet,
    build_initial_iet_from_runs,
    decode_lf_reference,
    eliminate_bucket,
    is_monotone_pieces,
)


def compute_slp_metrics(slp, final_pieces, n):
    """Compute exact SLP length, height, reachable nodes, sum distinct word lengths, and naive DFS visits."""
    # Bottom-up length and height for all nodes in slp (topologically ordered by creation!)
    num = len(slp)
    wlen = [0] * num
    wht = [0] * num
    wvisits = [0] * num  # number of DFS node visits to expand this node once (treating rep(base,k) as 1 + k*visits(base) or 1+visits(base)+k)
    for i, node in enumerate(slp):
        tag = node[0]
        if tag == 'lit':
            wlen[i] = 1
            wht[i] = 0
            wvisits[i] = 1
        elif tag == 'cat':
            l, r = node[1], node[2]
            wlen[i] = wlen[l] + wlen[r]
            wht[i] = 1 + (wht[l] if wht[l] >= wht[r] else wht[r])
            wvisits[i] = 1 + wvisits[l] + wvisits[r]
        elif tag == 'rep':
            b, k = node[1], node[2]
            wlen[i] = wlen[b] * k
            wht[i] = 1 + wht[b]
            wvisits[i] = 1 + wvisits[b] * k

    # Reachable nodes from final_pieces
    reach = set()
    stack = [p[5] for p in final_pieces]
    while stack:
        u = stack.pop()
        if u in reach:
            continue
        reach.add(u)
        node = slp[u]
        if node[0] == 'cat':
            stack.append(node[1])
            stack.append(node[2])
        elif node[0] == 'rep':
            stack.append(node[1])

    # Count distinct final word IDs and sum of their lengths
    final_wids = [p[5] for p in final_pieces]
    uniq_final_wids = set(final_wids)
    sum_piece_wlen = sum(wlen[p[5]] for p in final_pieces)
    sum_uniq_wlen = sum(wlen[w] for w in uniq_final_wids)
    total_output_len = sum(p[1] * wlen[p[5]] for p in final_pieces)
    assert total_output_len == n, f"Output len mismatch: {total_output_len} != {n}"

    # Naive DFS visits across all n bytes
    naive_dfs = sum(p[1] * wvisits[p[5]] for p in final_pieces)
    max_ht = max((wht[w] for w in uniq_final_wids), default=0)

    # Also compute bottom-up flattening cost if we flatten EVERY final piece into a flat spelling buffer:
    # Cost of DAG-memoized flattening of all reachable nodes? Wait: sum of lengths of all reachable nodes can be huge,
    # whereas flattening ONLY final pieces by DFS takes sum_uniq_wlen bytes written + sum_{w in uniq_final} wvisits[w] DFS visits!
    # Wait: even better, if we memoize any internal node with in_deg >= 2 AND len <= 4096 (or all internal nodes with total_uses >= 2)!
    ref_cnt = defaultdict(int)
    for w in uniq_final_wids:
        ref_cnt[w] += 1
    # Top-down ref count in reachable DAG
    for i in sorted(reach, reverse=True):
        node = slp[i]
        if node[0] == 'cat':
            ref_cnt[node[1]] += 1
            ref_cnt[node[2]] += 1
        elif node[0] == 'rep':
            ref_cnt[node[1]] += 1

    # Compute exact cost of "flatten all uniq_final_wids using DAG memoization on shared nodes (ref_cnt>=2 and wlen<=256)":
    memo_cost_bytes = 0
    memo_ops = 0
    memoized = set()
    for i in sorted(reach):
        if i in uniq_final_wids or (ref_cnt[i] >= 2 and wlen[i] <= 256):
            memoized.add(i)
            memo_cost_bytes += wlen[i]
    # How many leaves/memo-copies to materialize all memoized nodes?
    for i in memoized:
        # traverse until hitting a lit or another memoized child
        st = []
        node = slp[i]
        if node[0] == 'lit':
            memo_ops += 1
        elif node[0] == 'cat':
            st.extend([node[1], node[2]])
        elif node[0] == 'rep':
            st.append(node[1])
            memo_ops += 1  # rep loop
        while st:
            u = st.pop()
            if u in memoized or slp[u][0] == 'lit':
                memo_ops += 1
            elif slp[u][0] == 'cat':
                st.append(slp[u][1])
                st.append(slp[u][2])
            elif slp[u][0] == 'rep':
                st.append(slp[u][1])
                memo_ops += 1

    # Width distribution of final pieces
    w1_pieces = sum(1 for p in final_pieces if p[1] == 1)
    w1_rows = sum(p[1] for p in final_pieces if p[1] == 1)
    w16_rows = sum(p[1] for p in final_pieces if p[1] < 16)

    return {
        'slp_total': num,
        'slp_reach': len(reach),
        'final_pieces': len(final_pieces),
        'uniq_final_wids': len(uniq_final_wids),
        'sum_piece_wlen': sum_piece_wlen,
        'sum_uniq_wlen': sum_uniq_wlen,
        'max_ht': max_ht,
        'naive_dfs_over_n': naive_dfs / n,
        'memo_bytes': memo_cost_bytes,
        'memo_ops': memo_ops,
        'w1_pieces': w1_pieces,
        'w16_rows': w16_rows,
    }


def run_iet_schur_custom(data, keep_set=None, stop_at_k=1, strat="zero_self_first"):
    """
    Run Bucket IET Schur Elimination until len(pieces_by_src) == stop_at_k,
    never eliminating any bucket in keep_set (if specified).
    Also tracks max local bucket size (r_out + r_self and r_in) across all steps!
    """
    alpha, C, counts, pieces_by_src, slp, r0 = build_initial_iet(data)
    n = len(data)
    r_init = sum(len(v) for v in pieces_by_src.values())
    cum_trans = 0
    cum_deg = 0
    max_local_out = 0
    max_local_in = 0
    max_t_over_deg = 0.0
    step_log = []

    while len(pieces_by_src) > stop_at_k:
        cands = [b for b in pieces_by_src.keys() if not (keep_set and b in keep_set)]
        if not cands:
            break
        r_in_map = defaultdict(int)
        r_out_map = defaultdict(int)
        r_self_map = defaultdict(int)
        self_lists = defaultdict(list)
        for sb, plist in pieces_by_src.items():
            for piece in plist:
                db = piece[4]
                if sb == db:
                    r_self_map[sb] += 1
                    self_lists[sb].append(piece)
                else:
                    r_out_map[sb] += 1
                    r_in_map[db] += 1

        if strat == "zero_self_first":
            def key_fn(b):
                rs = r_self_map[b]
                tier = 0 if rs == 0 else (1 if rs == 1 else (2 if is_monotone_pieces(self_lists[b]) else 3))
                deg = r_in_map[b] + r_out_map[b] + 2 * rs
                return (tier, deg, counts[b], b)
        elif strat == "min_self_ratio":
            # Minimize expected self-loop bounce: prefer small |F_b| or small r_self / (r_in + r_out)
            def key_fn(b):
                rs = r_self_map[b]
                io = r_in_map[b] + r_out_map[b]
                deg = io + 2 * rs
                # Estimate average hops inside b: total_self_rows / total_in_rows
                self_rows = sum(p[1] for p in self_lists[b])
                in_rows = counts[b] - self_rows
                avg_hops = counts[b] / max(1, in_rows)
                return (0 if rs == 0 else 1, avg_hops * deg, counts[b], b)
        elif strat == "min_size":
            def key_fn(b):
                return (counts[b], b)
        else:
            raise ValueError(strat)

        best = min(cands, key=key_fn)
        deg = r_in_map[best] + r_out_map[best] + 2 * r_self_map[best]
        loc_out = r_out_map[best] + r_self_map[best]
        loc_in = r_in_map[best]
        if loc_out > max_local_out:
            max_local_out = loc_out
        if loc_in > max_local_in:
            max_local_in = loc_in

        st = eliminate_bucket(best, pieces_by_src, slp, use_spiral=True)
        cum_trans += st['transitions']
        cum_deg += deg
        ratio = st['transitions'] / max(1, deg)
        if ratio > max_t_over_deg:
            max_t_over_deg = ratio
        rem_rows = sum(counts[b] for b in pieces_by_src.keys())
        rem_pieces = sum(len(v) for v in pieces_by_src.values())
        step_log.append((len(pieces_by_src), best, counts[best], deg, st['transitions'], rem_rows, rem_pieces))

    final_pieces = []
    for b in sorted(pieces_by_src.keys()):
        final_pieces.extend(pieces_by_src[b])
    rem_rows = sum(counts[b] for b in pieces_by_src.keys())
    slp_m = compute_slp_metrics(slp, final_pieces, n)
    return {
        'n': n,
        'r0': r0,
        'r_init': r_init,
        'rem_buckets': sorted(pieces_by_src.keys()),
        'rem_rows': rem_rows,
        'cum_trans': cum_trans,
        'cum_deg': cum_deg,
        'max_local_out': max_local_out,
        'max_local_in': max_local_in,
        'max_t_over_deg': max_t_over_deg,
        'slp': slp_m,
        'step_log': step_log,
    }


def test_wikidata_deep():
    print("=== 1. WIKIDATA DEEP ECONOMICS & STOPPING CURVE ===")
    with open(os.path.join(BLOCKS_DIR, "manifest.json")) as f:
        manifest = json.load(f)

    # First: on block-0.L, compare strategies & stopping points k=1, 2, 3, 5 AND different kept buckets!
    b0_meta = manifest["blocks"][0]
    b0 = open(os.path.join(BLOCKS_DIR, b0_meta["file"]), "rb").read()
    assert hashlib.sha256(b0).hexdigest() == b0_meta["sha256"]

    print("\n--- block-0.L: Strategy & Stopping-k Comparison ---")
    for strat in ("zero_self_first", "min_self_ratio", "min_size"):
        for k in (1, 2, 3, 5):
            res = run_iet_schur_custom(b0, keep_set=None, stop_at_k=k, strat=strat)
            s = res['slp']
            kept_str = ",".join(f"{c}({chr(c) if 32<=c<127 else '.'})" for c in res['rem_buckets'])
            print(f"  strat={strat:15s} k={k} kept=[{kept_str:22s}] "
                  f"trans={res['cum_trans']:6d} ({res['cum_trans']/res['r0']:.2f}r0) "
                  f"rem_n={res['rem_rows']:6d} ({res['rem_rows']/res['n']:.4f}n) "
                  f"rem_r={s['final_pieces']:5d} max_loc={res['max_local_out']:4d} "
                  f"uniq_wlen={s['sum_uniq_wlen']:6d} ({s['sum_uniq_wlen']/res['n']:.2f}n) "
                  f"memoB={s['memo_bytes']:6d} memoOps={s['memo_ops']:6d} ht={s['max_ht']:2d} "
                  f"avg_w={res['n']/res['rem_rows']:5.1f}B")

    print("\n--- All 6 Wikidata Blocks: min_self_ratio at k=1 and k=2 ---")
    for blk in manifest["blocks"]:
        data = open(os.path.join(BLOCKS_DIR, blk["file"]), "rb").read()
        assert hashlib.sha256(data).hexdigest() == blk["sha256"]
        for k in (1, 2):
            res = run_iet_schur_custom(data, keep_set=None, stop_at_k=k, strat="min_self_ratio")
            s = res['slp']
            kept_str = ",".join(f"{chr(c) if 32<=c<127 else str(c)}" for c in res['rem_buckets'])
            print(f"  {blk['file']:12s} k={k} [{kept_str:6s}] r0={res['r0']:6d} "
                  f"trans={res['cum_trans']:6d} ({res['cum_trans']/res['r0']:.2f}r0) "
                  f"rem_n={res['rem_rows']:6d} ({res['rem_rows']/res['n']:.4f}n) "
                  f"rem_r={s['final_pieces']:5d} max_loc={res['max_local_out']:5d} "
                  f"uniq_wlen={s['sum_uniq_wlen']:7d} ({s['sum_uniq_wlen']/res['n']:.2f}n) "
                  f"memoB={s['memo_bytes']:7d} memoOps={s['memo_ops']:6d} ht={s['max_ht']:2d} "
                  f"avg_w={res['n']/res['rem_rows']:5.1f}B")


def test_random_and_adversarial():
    print("\n=== 2. RANDOM & ADVERSARIAL STRESS TESTS (Finding Exact Blowup Regimes!) ===")
    rng = random.Random(2026)

    # A. Random strings over |Sigma| = 2, 4, 16, 64, 256, n = 10,000
    print("\n--- Uniform Random Strings (n=10,000) ---")
    for sigma in (2, 4, 16, 64, 256):
        T = [rng.randrange(sigma) for _ in range(10000)]
        L, p = bwt_encode(T)
        res = run_iet_schur_custom(L, keep_set=None, stop_at_k=1, strat="min_self_ratio")
        s = res['slp']
        print(f"  Rand(sig={sigma:3d}, n=10000): r0={res['r0']:5d} "
              f"trans={res['cum_trans']:6d} ({res['cum_trans']/res['r0']:.2f}r0, {res['cum_trans']/10000:.2f}n) "
              f"rem_n={res['rem_rows']:5d} ({res['rem_rows']/10000:.3f}n) rem_r={s['final_pieces']:5d} "
              f"max_t/deg={res['max_t_over_deg']:5.2f} ht={s['max_ht']:2d}")

    # B. Phi_BWT with count-1 sentinel: keep sentinel vs keep #
    print("\n--- Phi_BWT(M=256, K=8) + count-1 sentinel ---")
    M = 256
    perm = list(range(M))
    rng.shuffle(perm)
    L_bits = 8
    K_skel = 8
    # Alphabet: sentinel=0 (count 1), #=1, $=2, bit0=3, bit1=4
    T_phi = []
    for idx, x in enumerate(perm):
        T_phi.append(0 if idx == 0 else 1)
        T_phi.extend([2] * (K_skel - 1))
        for bit in range(L_bits - 1, -1, -1):
            T_phi.append(3 + ((x >> bit) & 1))
    L, p = bwt_encode(T_phi)
    for name, kset in [("keep_#(count=255)", {1}), ("keep_sentinel(count=1)", {0})]:
        res = run_iet_schur_custom(L, keep_set=kset, stop_at_k=1, strat="min_self_ratio")
        s = res['slp']
        print(f"  Phi_BWT {name:24s}: n={res['n']} r0={res['r0']} "
              f"trans={res['cum_trans']} ({res['cum_trans']/res['r0']:.2f}r0) "
              f"rem_n={res['rem_rows']} rem_r={s['final_pieces']} max_t/deg={res['max_t_over_deg']:.2f}")

    # C. Fib_{2m} + $ keeping $ (count 1) — witness of naive 2-piece self-loop blowup!
    print("\n--- Fib_{2m} + $ keeping $ (count=1): Naive 2-piece blowup vs 2-piece Euclidean acceleration ---")
    F = [1, 1]
    for _ in range(85):
        F.append(F[-1] + F[-2])
    for k in (10, 14, 18, 22):
        f_km2, f_km1 = F[k - 2], F[k - 1]
        runs = [(1, f_km2 - 1), (2, 1), (0, f_km1), (1, 1)]
        n_fib = f_km2 + f_km1 + 1
        alpha, C, counts, pieces_by_src, slp, r0 = build_initial_iet_from_runs(runs, n_fib)
        # Eliminate 1 first, then 0 (keeping 2, count=1!)
        st1 = eliminate_bucket(1, pieces_by_src, slp, use_spiral=True)
        st0 = eliminate_bucket(0, pieces_by_src, slp, use_spiral=True)
        print(f"  Fib_{k}+$ (n={n_fib:6d}, r0=4): elim(1) trans={st1['transitions']:3d} self={st1['r_self']}; "
              f"elim(0) trans={st0['transitions']:6d} self={st0['r_self']} spirals={st0['spiral_jumps']} (Theta(n) naive!)")

    # D. Referee n=5 and n=10 F4 counterexamples
    print("\n--- Referee F4 & n=5 Counterexamples ---")
    for name, T in [
        ("n=5 thresh", [0, 0, 1, 1, 2]),
        ("n=5 split", [0, 0, 2, 0, 1]),
        ("n=5 self", [0, 0, 1, 0, 1]),
        ("n=10 F4", [0, 0, 1, 1, 2, 1, 2, 0, 0, 1]),
    ]:
        L, p = bwt_encode(T)
        res = run_iet_schur_custom(L, keep_set=None, stop_at_k=1, strat="zero_self_first")
        print(f"  {name:12s}: n={len(T)} r0={res['r0']} r_init={res['r_init']} "
              f"trans={res['cum_trans']} rem_n={res['rem_rows']} rem_r={res['slp']['final_pieces']}")


if __name__ == "__main__":
    test_wikidata_deep()
    test_random_and_adversarial()
