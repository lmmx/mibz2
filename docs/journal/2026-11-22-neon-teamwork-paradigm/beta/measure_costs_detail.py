#!/usr/bin/env python3
"""
Detailed cost-vector (W, D, B, Q, M) and O(1)-depth parallel 2-coloring analysis
for BWT-Direct Monotone Recompression (Variant B-Parallel: O(1)-depth hash/threshold
full-split pair contraction + exact self-run contraction) on all 6 authenticated Wikidata blocks.
"""
import hashlib
import json
import os
import time

# Reuse exact primitives from deep_analysis.py
import sys
sys.path.insert(0, "/tmp/teamwork-b")
from deep_analysis import (
    BLOCKS_DIR,
    runs_from_seq,
    ref_inverse_bwt,
    self_run_contraction_step,
    full_split_pair_step,
)


def parallel_o1_depth_coloring(bucket_runs, sigma, diff_asc, diff_desc, round_idx):
    """
    100% O(1)-depth data-parallel 2-coloring (ZERO sequential greedy loop, ZERO adjacency list!).
    Compares:
      - Optimal ascending/descending threshold cut (zero bucket splits!)
      - 2 deterministic O(1)-depth hash 2-colorings (both polarities) + parity (both polarities)
    Returns (in_L, del_cnt, kind).
    """
    # Best threshold
    best_th_cnt = 0
    best_dir = 'asc'
    best_th = -1
    acc = 0
    for th in range(1, sigma):
        acc += diff_asc[th]
        if acc > best_th_cnt:
            best_th_cnt = acc
            best_dir = 'asc'
            best_th = th
    acc = 0
    for th in range(1, sigma):
        acc += diff_desc[th]
        if acc > best_th_cnt:
            best_th_cnt = acc
            best_dir = 'desc'
            best_th = th

    # O(1)-depth parallel hash colorings
    cands = []
    if sigma >= 4:
        m1 = 2654435761
        m2 = 2246822519
        s1 = (round_idx % 15) + 8
        s2 = ((round_idx * 3 + 5) % 15) + 8
        col1 = [(((s + 1 + round_idx * 17) * m1) >> s1) & 1 for s in range(sigma)]
        col2 = [(((s + 7 + round_idx * 31) * m2) >> s2) & 1 for s in range(sigma)]
        colp = [s & 1 for s in range(sigma)]
        for cname, col in [("hash1", col1), ("hash2", col2), ("parity", colp)]:
            c01 = 0
            c10 = 0
            for b in range(sigma):
                cb = col[b]
                for a, w in bucket_runs[b]:
                    if a != b and col[a] != cb:
                        if cb == 1:
                            c01 += w
                        else:
                            c10 += w
            if c01 >= c10:
                cands.append((c01, cname + "_01", [c == 0 for c in col]))
            else:
                cands.append((c10, cname + "_10", [c == 1 for c in col]))

    best_gen_cnt, best_gen_name, best_gen_inL = (0, "", None)
    for cnt, nm, inL in cands:
        if cnt > best_gen_cnt:
            best_gen_cnt, best_gen_name, best_gen_inL = cnt, nm, inL

    # Prefer zero-split threshold if >= 85% of best hash cut (minimizes bucket splits!)
    if best_th_cnt >= 0.85 * best_gen_cnt and best_th_cnt > 0:
        in_L = [s < best_th for s in range(sigma)] if best_dir == 'asc' else [s >= best_th for s in range(sigma)]
        return in_L, best_th_cnt, 'thresh'
    elif best_gen_cnt > 0:
        return best_gen_inL, best_gen_cnt, best_gen_name
    else:
        return None, 0, 'done'


def analyze_block_detailed(entry, print_rounds=False):
    path = os.path.join(BLOCKS_DIR, entry["file"])
    with open(path, "rb") as f:
        L = f.read()
    assert hashlib.sha256(L).hexdigest() == entry["sha256"]
    n_orig = len(L)
    p = entry["primary_index"]

    uniq = sorted(set(L))
    remap = {c: i for i, c in enumerate(uniq)}
    runs = runs_from_seq([remap[b] for b in L])
    sigma = len(uniq)
    rules = [('base', c) for c in uniq]
    sym_len = [1] * sigma
    sym_to_rule = list(range(sigma))

    n = n_orig
    p_offset = 0
    r0 = len(runs)
    rounds = 0

    total_r = 0
    total_f_runs = 0
    total_indep_R_reads = 0
    total_indep_L_split_reads = 0
    total_split_syms = 0
    peak_r = r0
    peak_sigma = sigma
    peak_mem_bytes = 0

    # Bucketed cache-level breakdown of independent reads (based on working set of f_runs + cursors)
    q_l1 = 0  # <= 32 KiB
    q_l2 = 0  # <= 512 KiB
    q_l3 = 0  # > 512 KiB

    while True:
        r_cur = len(runs)
        total_r += r_cur
        if r_cur > peak_r:
            peak_r = r_cur
        if sigma > peak_sigma:
            peak_sigma = sigma

        counts = [0] * sigma
        for c, w in runs:
            counts[c] += w
        C = [0] * (sigma + 1)
        tot = 0
        for c in range(sigma):
            C[c] = tot
            tot += counts[c]
        C[sigma] = tot

        bucket_runs = [[] for _ in range(sigma)]
        diff_asc = [0] * (sigma + 1)
        diff_desc = [0] * (sigma + 1)
        self_total = 0
        pos = 0
        f_sym = 0
        while f_sym < sigma and C[f_sym + 1] == 0:
            f_sym += 1
        p_F = -1
        f_runs_cnt = 0
        for c, w in runs:
            rem = w
            cur_pos = pos
            while rem > 0:
                while C[f_sym + 1] <= cur_pos:
                    f_sym += 1
                avail = C[f_sym + 1] - cur_pos
                take = rem if rem <= avail else avail
                if cur_pos <= p < cur_pos + take:
                    p_F = f_sym
                bucket_runs[f_sym].append((c, take))
                f_runs_cnt += 1
                if c == f_sym:
                    self_total += take
                elif c < f_sym:
                    diff_asc[c + 1] += take
                    diff_asc[f_sym + 1] -= take
                else:
                    diff_desc[f_sym + 1] += take
                    diff_desc[c + 1] -= take
                cur_pos += take
                rem -= take
            pos += w
        total_f_runs += f_runs_cnt

        if self_total == n:
            root = sym_to_rule[p_F]
            break

        if self_total > 0 and self_total >= 0.05 * n:
            res = self_run_contraction_step(runs, sigma, n, p, p_offset, sym_len, sym_to_rule, rules, C, bucket_runs)
            if res[6] is not None:
                root = sym_to_rule[res[6]]
                p_offset = res[2]
                break
            runs, p, p_offset, del_cnt, sym_to_rule, sigma, _ = res
            n -= del_cnt
            rounds += 1
            continue

        in_L, del_cnt, kind = parallel_o1_depth_coloring(bucket_runs, sigma, diff_asc, diff_desc, rounds)
        if del_cnt == 0:
            if self_total > 0:
                res = self_run_contraction_step(runs, sigma, n, p, p_offset, sym_len, sym_to_rule, rules, C, bucket_runs)
                if res[6] is not None:
                    root = sym_to_rule[res[6]]
                    p_offset = res[2]
                    break
                runs, p, p_offset, del_cnt, sym_to_rule, sigma, _ = res
                n -= del_cnt
                rounds += 1
                continue
            else:
                root = sym_to_rule[p_F]
                break

        # Measure how many Sigma_L symbols actually split (>1 uncontracted block)
        a_blocks = [0] * sigma
        a_prev_type = [-1] * sigma  # 0=uncon, 1=con
        for b in range(sigma):
            b_in_L = in_L[b]
            for a, _ in bucket_runs[b]:
                if in_L[a]:
                    t = 0 if b_in_L else 1
                    if t == 0 and a_prev_type[a] != 0:
                        a_blocks[a] += 1
                    a_prev_type[a] = t
        split_syms = sum(1 for a in range(sigma) if a_blocks[a] > 1)
        total_split_syms += split_syms

        r_reads = sum(1 for c, _ in runs if not in_L[c])
        l_split_reads = sum(1 for c, _ in runs if in_L[c] and a_blocks[c] > 1)
        total_indep_R_reads += r_reads
        total_indep_L_split_reads += l_split_reads

        # Working set in bytes for Step 3 of this round:
        # runs (8B * r_cur) + f_runs (8B * f_runs_cnt, only Sigma_R entries active + new_id 4B -> 12B * f_runs_cnt)
        # + cursors (8B * sigma) + out_runs (8B * r_cur)
        ws_lookup = 12 * f_runs_cnt + 8 * sigma  # random-read target working set!
        ws_total = 8 * r_cur + ws_lookup + 8 * r_cur + 12 * len(rules)
        if ws_total > peak_mem_bytes:
            peak_mem_bytes = ws_total

        q_this = r_reads + l_split_reads
        if ws_lookup <= 32 * 1024:
            q_l1 += q_this
        elif ws_lookup <= 512 * 1024:
            q_l2 += q_this
        else:
            q_l3 += q_this

        if print_rounds and (rounds < 12 or rounds % 5 == 0 or n <= 50):
            print(
                f"    r{rounds:02d} [{kind:9s}]: n={n:7d}, r={r_cur:6d}, f_runs={f_runs_cnt:6d}, "
                f"sigma={sigma:5d}, splits={split_syms:4d}, q_indep={q_this:6d}, ws_lookup={ws_lookup//1024:4d} KiB"
            )

        res = full_split_pair_step(runs, sigma, n, p, p_offset, sym_len, sym_to_rule, rules, C, bucket_runs, in_L)
        runs, p, p_offset, del_cnt, sym_to_rule, sigma = res
        n -= del_cnt
        rounds += 1

    # Verify exactness & measure SLP expansion detail
    stack = [root]
    out = bytearray()
    while stack:
        s = stack.pop()
        rl = rules[s]
        if rl[0] == 'base':
            out.append(rl[1])
        elif rl[0] == 'pair':
            stack.append(rl[2])
            stack.append(rl[1])
        else:
            _, a, k = rl
            for _ in range(k):
                stack.append(a)
    d = len(out)
    decoded = bytes(out[p_offset:] + out[:p_offset]) if d == n_orig else bytes(out[(p_offset + t) % d] for t in range(n_orig))

    # Reference verify
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 256
    tot = 0
    for c in range(256):
        C[c] = tot
        tot += counts[c]
    F = bytearray(n_orig)
    for c in range(256):
        F[C[c]:C[c] + counts[c]] = bytes([c]) * counts[c]
    pos = C[:]
    FL = [0] * n_orig
    for i, b in enumerate(L):
        FL[pos[b]] = i
        pos[b] += 1
    ref = bytearray(n_orig)
    cur = p_init = entry["primary_index"]
    for t in range(n_orig):
        ref[t] = F[cur]
        cur = FL[cur]
    assert decoded == bytes(ref), f"MISMATCH on {entry['file']}!"

    # SLP height & expansion stats at L=32 and L=64
    height = [0] * len(rules)
    for i, rl in enumerate(rules):
        if rl[0] == 'base':
            height[i] = 0
        elif rl[0] == 'pair':
            height[i] = 1 + max(height[rl[1]], height[rl[2]])
        else:
            height[i] = 1 + height[rl[1]]

    exp_summary = {}
    for L_cut in (32, 64):
        leaves = 0
        internal = 0
        used_leaves = set()
        used_internal = set()
        stk = [root]
        while stk:
            s = stk.pop()
            if sym_len[s] <= L_cut:
                leaves += 1
                used_leaves.add(s)
            else:
                internal += 1
                used_internal.add(s)
                rl = rules[s]
                if rl[0] == 'pair':
                    stk.append(rl[2])
                    stk.append(rl[1])
                elif rl[0] == 'run':
                    _, a, k = rl
                    for _ in range(k):
                        stk.append(a)
        leaf_buf = sum(sym_len[s] for s in used_leaves)
        internal_tbl = len(used_internal) * 8
        exp_summary[L_cut] = (internal, leaves, len(used_internal), internal_tbl // 1024, len(used_leaves), leaf_buf // 1024)

    print(
        f"  [{entry['file']}] EXACT | rounds={rounds:2d}, sum_r={total_r:6d} ({total_r/r0:.2f}x r0, {total_r/n_orig:.2f}x n), "
        f"sum_fruns={total_f_runs:6d}, max_r={peak_r:6d} ({peak_r/r0:.2f}x), max_sigma={peak_sigma:5d}"
    )
    print(
        f"    Q_dep=0 | Q_indep={total_indep_R_reads + total_indep_L_split_reads:6d} "
        f"(R={total_indep_R_reads:6d}, L_split={total_indep_L_split_reads:5d}) | "
        f"by ws_lookup: L1(<=32K)={q_l1:5d}, L2(<=512K)={q_l2:6d}, L3(>512K)={q_l3:6d} | peak_M={peak_mem_bytes//1024:4d} KiB"
    )
    i32, l32, ui32, it32, ul32, lb32 = exp_summary[32]
    i64, l64, ui64, it64, ul64, lb64 = exp_summary[64]
    print(
        f"    SLP: rules={len(rules):5d}, height={height[root]:2d} | "
        f"L<=32: int_visits={i32:5d} (uniq={ui32:5d}, {it32:3d} KiB), leaf_cpys={l32:5d} (uniq={ul32:5d}, {lb32:3d} KiB) | "
        f"L<=64: int_visits={i64:5d} (uniq={ui64:5d}, {it64:3d} KiB), leaf_cpys={l64:5d} (uniq={ul64:5d}, {lb64:3d} KiB)"
    )


def main():
    with open(os.path.join(BLOCKS_DIR, "manifest.json")) as f:
        manifest = json.load(f)
    print("=== VARIANT B-PARALLEL (O(1)-DEPTH 2-COLORING + FULL BUCKET-SPLIT) DETAILED COST AUDIT ===")
    for idx, entry in enumerate(manifest["blocks"]):
        analyze_block_detailed(entry, print_rounds=(idx in (0, 4)))


if __name__ == "__main__":
    main()
