#!/usr/bin/env python3
"""
Deep mathematical verification and full cost-vector measurement for:
BWT-Direct Monotone Recompression
  - Variant A: Contiguity-Preserving Pair Contraction + Exact Self-Run Contraction
  - Variant B: Full Bucket-Split Max-Cut Pair Contraction + Exact Self-Run Contraction

Verifies:
  1. Exhaustive small cases: binary n<=8, ternary n<=5, all p (5,227 cases x 2 variants)
  2. Adversarial torture suite (long runs, periodic, Fibonacci, Thue-Morse, random, straddled ramp)
  3. All 6 authenticated Wikidata blocks in /tmp/mibz2-blocks/ with exact byte-for-byte check
     and complete (W, D, B, Q, M) + SLP expansion analysis.
"""
import hashlib
import json
import os
import random
import time

BLOCKS_DIR = "/tmp/mibz2-blocks"


def runs_from_seq(L):
    runs = []
    n = len(L)
    if n == 0:
        return runs
    cur_c = L[0]
    cur_l = 1
    for i in range(1, n):
        b = L[i]
        if b == cur_c:
            cur_l += 1
        else:
            runs.append((cur_c, cur_l))
            cur_c = b
            cur_l = 1
    runs.append((cur_c, cur_l))
    return runs


def bwt_forward(T):
    """Exact forward BWT (for generating adversarial test columns)."""
    n = len(T)
    # Use suffix-doubling on T+T for speed on n=100k!
    s = list(T)
    # Coordinate compress
    uniq = sorted(set(s))
    mp = {c: i for i, c in enumerate(uniq)}
    rk = [mp[c] for c in s]
    order = list(range(n))
    k = 1
    while k < n:
        order.sort(key=lambda i: (rk[i], rk[(i + k) % n]))
        nrk = [0] * n
        cur = 0
        nrk[order[0]] = 0
        for idx in range(1, n):
            i, prev = order[idx], order[idx - 1]
            if (rk[i], rk[(i + k) % n]) != (rk[prev], rk[(prev + k) % n]):
                cur += 1
            nrk[i] = cur
        rk = nrk
        if cur == n - 1:
            break
        k <<= 1
    L = [T[(i - 1) % n] for i in order]
    p = order.index(0)
    return L, p


def ref_inverse_bwt(L, p):
    n = len(L)
    sigma = max(L) + 1 if L else 0
    counts = [0] * sigma
    for c in L:
        counts[c] += 1
    C = [0] * sigma
    tot = 0
    for c in range(sigma):
        C[c] = tot
        tot += counts[c]
    F = [0] * n
    for c in range(sigma):
        for i in range(C[c], C[c] + counts[c]):
            F[i] = c
    pos = C[:]
    FL = [0] * n
    for i, c in enumerate(L):
        FL[pos[c]] = i
        pos[c] += 1
    out = [0] * n
    cur = p
    for t in range(n):
        out[t] = F[cur]
        cur = FL[cur]
    return out


def self_run_contraction_step(runs, sigma, n, p, p_offset, sym_len, sym_to_rule, rules, C, bucket_runs):
    """
    Exact 1-Round Monotone Self-Run Contraction on ALL symbols c with self-edges (L[i] == F[i] == c).
    Works for EVERY arbitrary column L (BWT or non-BWT, multi-cycle, periodic)!
    Uses run-level interval arithmetic with spiral division for O(1) collapse of long runs a^k!
    Returns (new_runs, new_p, new_p_offset, deleted_cnt, new_sym_to_rule, new_sigma, fixed_point_sym_or_None).
    """
    # For each symbol c that has self-edges inside B_c:
    # We compute for each surviving row in S_c = B_c \ D_c its exact run length k >= 1,
    # organized as contiguous pieces (start_in_Bc, end_in_Bc, k) covering S_c,
    # AND detect if p is a fixed point (1-cycle) or inside D_c (in which case move p to its landing row in S_c)!
    #
    # First check total self-edges:
    self_cnt = [0] * sigma
    for c in range(sigma):
        for a, w in bucket_runs[c]:
            if a == c:
                self_cnt[c] += w
    total_self = sum(self_cnt)
    if total_self == 0:
        return None

    # Compute u_c = C[c] + #{i < C[c] : L[i] == c} for each c with self_cnt[c] > 0.
    before_cnt = [0] * sigma
    seen = [0] * sigma
    pos = 0
    f_sym = 0
    while f_sym < sigma and C[f_sym + 1] == 0:
        f_sym += 1
    # Actually we can get before_cnt directly from bucket_runs!
    # Sum occurrences of `a` in bucket_runs[0..a-1]:
    for b in range(sigma):
        for a, w in bucket_runs[b]:
            if b < a:
                before_cnt[a] += w

    # For each c with self_cnt[c] > 0, solve the 1D monotone map g_c: D_c -> [u_c, v_c) inside B_c (0-based offsets in [0, len_c))!
    # In 0-based coordinates inside B_c (length Lc = C[c+1] - C[c]):
    # - D_c is a union of intervals (runs where a == c), with total length m = self_cnt[c].
    # - Target interval is [u, v) where u = before_cnt[c], v = u + m.
    # - Each D_c interval [l_i, r_i) maps by pure translation x -> x + delta_i onto [t_i, t_i + (r_i - l_i)) inside [u, v).
    # - Fixed points occur iff delta_i == 0 on some D_c interval [l_i, r_i)!
    #   (Because g_c(x) - x is piecewise constant on each D_c run, and monotonically non-increasing!)
    # - For every non-fixed D_c interval, delta_i != 0!
    # - Let's compute the exact inverse-orbit partition of E_c = [0, u) U [v, Lc) (entries into B_c from outside B_c)!
    #   Wait: EVERY surviving row y in S_c = [0, Lc) \ D_c is reached from a UNIQUE entry x in E_c by k-1 steps of g_c!
    #   Wait: even simpler! Trace intervals FORWARD under g_c starting from E_c!
    #   Wait: every interval of E_c \ D_c stops immediately (k = 1).
    #   Every interval of E_c cap D_c moves strictly right (if < u) or strictly left (if >= v),
    #   splitting only at boundaries of D_c intervals, and using spiral division whenever an interval
    #   maps back into the SAME D_c run [l_i, r_i)!
    #   Total sub-intervals created is O(#runs in B_c)!

    # Store for each c with self_cnt[c] > 0:
    #   s_pieces[c]: list of (s_lo, s_hi, k) in 0-based S_c-rank coordinates?
    #   Even better: store in 0-based B_c coordinates: list of (b_lo, b_hi, k) covering ALL of S_c in sorted order!
    #   AND track p if p is in B_c!

    s_pieces_by_c = {}
    fixed_points_by_c = {}  # c -> list of (b_lo, b_hi) of fixed points (deleted from S, unless p is inside!)
    total_deleted = 0
    new_p_old = p
    new_p_offset = p_offset

    for c in range(sigma):
        if self_cnt[c] == 0:
            continue
        Lc = C[c + 1] - C[c]
        u = before_cnt[c]
        m = self_cnt[c]
        v = u + m

        # Build sorted list of D_c intervals (l, r, delta) and S_c intervals (l, r) in [0, Lc)
        d_runs = []  # (l, r, delta)
        s_runs = []  # (l, r)
        off = 0
        t_cur = u
        for a, w in bucket_runs[c]:
            if a == c:
                delta = t_cur - off
                d_runs.append((off, off + w, delta))
                t_cur += w
            else:
                if s_runs and s_runs[-1][1] == off:
                    s_runs[-1] = (s_runs[-1][0], off + w)
                else:
                    s_runs.append((off, off + w))
            off += w
        assert off == Lc and t_cur == v

        # Check if p is inside B_c
        p_in_this = (C[c] <= p < C[c + 1])
        p_rel = p - C[c] if p_in_this else -1

        # Fixed points: D_c intervals with delta == 0
        fps = [(l, r) for (l, r, d) in d_runs if d == 0]
        fixed_points_by_c[c] = fps
        if p_in_this:
            for (l, r) in fps:
                if l <= p_rel < r:
                    # p IS a fixed point (1-cycle of symbol c)!
                    return (None, c, p_offset, 0, None, -1, c)

        # Now compute exact k(y) for every y in S_c by forward interval propagation from E_c = [0, u) U [v, Lc)!
        # Active queue of intervals (lo, hi, steps_so_far) currently in [0, Lc), where steps_so_far >= 1.
        # Notice:
        # - For any interval in S_c: it terminates with k = steps_so_far!
        # - For any interval in D_c run i = (l, r, delta) with delta != 0:
        #   - If delta > 0: an interval [lo, hi) inside [l, r) stays inside [l, r) for q = floor((r - 1 - x) / delta) + 1 steps!
        #     Specifically, if hi - lo <= delta, we can step it in O(1); if hi - lo > delta, it wraps as a spiral:
        #     Wait: the piece [lo, hi) enters [l, r) from OUTSIDE [l, r) (or from another D_c run), so can hi - lo > delta?
        #     YES if it entered from a wide interval! Split [lo, hi) by residues mod |delta|?
        #     NO, EVEN SIMPLER! Since every element x in [lo, hi) has q(x) = (r - 1 - x) // delta + 1 steps to exit [l, r):
        #     Split [lo, hi) into:
        #       1) The rightmost sub-interval [max(lo, r - delta), hi) -> exits in 1 step!
        #       2) Actually, for x in [lo, hi), q(x) is CONSTANT on intervals of width |delta|?
        #          No: if [lo, hi) has width <= delta, it splits into AT MOST TWO pieces (with q and q+1)!
        #          What if hi - lo > delta?! Then wait: since [lo, hi) is an image of E_c, and g_c is INJECTIVE,
        #          [lo, hi) and g_c([lo, hi)) = [lo + delta, hi + delta) MUST BE DISJOINT if [lo, hi) came from E_c!
        #          WAIT A SECOND!! PROVE THAT!!
        #          If x in E_c = [0, u) U [v, Lc), then x is NOT in [u, v) = Im(g_c)!
        #          Therefore g_c^0(E_c), g_c^1(E_c), g_c^2(E_c), ... are PAIRWISE DISJOINT!!
        #          THEREFORE, for ANY interval [lo, hi) in g_c^t(E_c) inside a D_c run (l, r, delta):
        #          [lo, hi) and [lo + delta, hi + delta) MUST BE DISJOINT!!
        #          HENCE `hi - lo <= |delta|` ALWAYS!! (100% PROVED!!)
        #          SO SPIRAL DIVISION SPLITS [lo, hi) INTO AT MOST 2 PIECES WITH EXACT CLOSED-FORM `q`!!

        # Let's build a sorted list of ALL intervals in [0, Lc): (l, r, type, delta) where type='S' or 'D'
        all_ivs = []
        for l, r in s_runs:
            all_ivs.append((l, r, 'S', 0))
        for l, r, d in d_runs:
            all_ivs.append((l, r, 'D', d))
        all_ivs.sort()
        iv_ends = [iv[1] for iv in all_ivs]

        from bisect import bisect_right

        # Initial active intervals: E_c = [0, u) and [v, Lc) with k = 1
        stack = []
        if u > 0:
            stack.append((0, u, 1))
        if v < Lc:
            stack.append((v, Lc, 1))

        # Track p if p_in_this!
        # Remember: if p_rel is in D_c, we trace p_rel forward under g_c until hitting S_c!
        # Wait: if p_rel is in D_c, how many steps `skip` does it take from p_rel to S_c?
        # Each step of LF moves BACKWARD 1 copy of symbol c, so p_offset += skip * len(c)!
        if p_in_this:
            cur_x = p_rel
            skip = 0
            while True:
                idx = bisect_right(iv_ends, cur_x)
                l, r, tp, d = all_ivs[idx]
                if tp == 'S':
                    new_p_old = C[c] + cur_x
                    new_p_offset += skip * sym_len[sym_to_rule[c]]
                    break
                else:
                    assert d != 0
                    if d > 0:
                        q = (r - 1 - cur_x) // d + 1
                        cur_x += q * d
                        skip += q
                    else:
                        ad = -d
                        q = (cur_x - l) // ad + 1
                        cur_x -= q * ad
                        skip += q

        landed = []  # (lo, hi, k) in S_c
        while stack:
            lo, hi, k = stack.pop()
            # Intersect [lo, hi) with all_ivs
            idx = bisect_right(iv_ends, lo)
            cur = lo
            while cur < hi:
                l, r, tp, d = all_ivs[idx]
                seg_hi = hi if hi < r else r
                if tp == 'S':
                    landed.append((cur, seg_hi, k))
                else:
                    assert d != 0, "E_c trajectory cannot hit a fixed point!"
                    # Because g_c^t(E_c) are pairwise disjoint, seg_hi - cur <= |d| ALWAYS!
                    if d > 0:
                        assert seg_hi - cur <= d
                        # For x in [cur, seg_hi): q(x) = (r - 1 - x) // d + 1
                        # Since seg_hi - cur <= d, q(x) takes at most 2 values!
                        q_lo = (r - 1 - cur) // d + 1
                        q_hi = (r - 1 - (seg_hi - 1)) // d + 1
                        if q_lo == q_hi:
                            stack.append((cur + q_lo * d, seg_hi + q_lo * d, k + q_lo))
                        else:
                            # Split point where (r - 1 - x) // d changes from q_lo - 1 to q_hi - 1 (= q_lo - 2):
                            # x >= r - (q_lo - 1) * d has q = q_hi
                            split = r - (q_lo - 1) * d
                            stack.append((cur + q_lo * d, split + q_lo * d, k + q_lo))
                            stack.append((split + q_hi * d, seg_hi + q_hi * d, k + q_hi))
                    else:
                        ad = -d
                        assert seg_hi - cur <= ad
                        # q(x) = (x - l) // ad + 1
                        q_lo = (cur - l) // ad + 1
                        q_hi = (seg_hi - 1 - l) // ad + 1
                        if q_lo == q_hi:
                            stack.append((cur - q_lo * ad, seg_hi - q_lo * ad, k + q_lo))
                        else:
                            split = l + q_lo * ad
                            stack.append((cur - q_lo * ad, split - q_lo * ad, k + q_lo))
                            stack.append((split - q_hi * ad, seg_hi - q_hi * ad, k + q_hi))
                cur = seg_hi
                idx += 1

        landed.sort()
        # Merge adjacent landed pieces with equal k (even across deleted D_c gaps! Wait: in S_c, keep B_c coords or rank coords!)
        # Keep exact B_c coords for S_c! And verify total landed length == Lc - m!
        tot_landed = sum(hi - lo for lo, hi, _ in landed)
        assert tot_landed == Lc - m, f"tot_landed={tot_landed} != Lc - m = {Lc - m}"

        # Assign each maximal contiguous block of equal k in S_c-order a piece_id!
        # Wait: in S_c-order (skipping D_c gaps), two adjacent landed intervals with the SAME k form ONE contiguous bucket in S_c!
        # Group landed intervals into bucket blocks: (k, sub_id, total_s_len_in_block)!
        # WAIT: When `i` in S has `L[i] = c` (so `i` is OUTSIDE B_c!), `LF(i)` lands in E_c = [0, u) U [v, Lc),
        # and then follows g_c^{k-1} into S_c!
        # WAIT A SECOND!! STOP AND CHECK WHICH BUCKET `LF_S(i)` LANDS IN!!
        # For `i` with `L[i] = c`: `LF(i) = x in E_c`, and `LF_S(i) = y = g_c^{k(x)-1}(x) in S_c`!!
        # SO WE NEED TO KNOW WHICH `S_c` BUCKET `x in E_c` LANDS IN!!
        # Let's track BOTH `x` (entry coord in E_c) AND `y` (landing coord in S_c)!!
        # Re-run with (src_lo, cur_lo, width, k) — trivial, 3 extra words per interval!!
        s_pieces_by_c[c] = (Lc, u, v, all_ivs, iv_ends, landed)
        total_deleted += m

    # Now re-propagate with (src_lo, cur_lo, width, k) to build:
    # 1) For each c with self_cnt[c] > 0:
    #    - S_c bucket layout: list of (s_lo, s_hi, block_id, k) in sorted y-order (merging adjacent equal-k in S_c)
    #    - Entry map on E_c: list of (e_lo, e_hi, block_id, k) in sorted x-order!
    #      Notice: because g_c preserves order on each fixed k, if two entries x1 < x2 land in the SAME block_id (same k),
    #      then y1 < y2 ALWAYS! So LF_S is STRICTLY MONOTONIC on every (c, block_id) fiber!!
    from bisect import bisect_right

    entry_map_by_c = {}
    block_k_by_c = {}
    for c, (Lc, u, v, all_ivs, iv_ends, landed) in s_pieces_by_c.items():
        # Assign block_id to landed intervals in sorted y-order (merging consecutive equal k!)
        y_blocks = []  # (y_lo, y_hi, block_id, k)
        blk_k = []
        for lo, hi, k in landed:
            if y_blocks and y_blocks[-1][3] == k:
                bid = y_blocks[-1][2]
            else:
                bid = len(blk_k)
                blk_k.append(k)
            y_blocks.append((lo, hi, bid, k))
        yb_ends = [b[1] for b in y_blocks]
        block_k_by_c[c] = blk_k

        # Propagate (src_lo, cur_lo, width, k) from E_c to map each entry sub-interval to its exact block_id!
        stack = []
        if u > 0:
            stack.append((0, 0, u, 1))
        if v < Lc:
            stack.append((v, v, Lc - v, 1))

        entries = []  # (src_lo, src_hi, bid)
        while stack:
            src_lo, cur_lo, width, k = stack.pop()
            cur_hi = cur_lo + width
            idx = bisect_right(iv_ends, cur_lo)
            cur = cur_lo
            s_cur = src_lo
            while cur < cur_hi:
                l, r, tp, d = all_ivs[idx]
                seg_hi = cur_hi if cur_hi < r else r
                seg_w = seg_hi - cur
                if tp == 'S':
                    # Look up block_id covering [cur, seg_hi) in y_blocks
                    y_idx = bisect_right(yb_ends, cur)
                    y_cur = cur
                    ys_cur = s_cur
                    while y_cur < seg_hi:
                        ylo, yhi, bid, yk = y_blocks[y_idx]
                        assert yk == k
                        take = min(seg_hi, yhi) - y_cur
                        entries.append((ys_cur, ys_cur + take, bid))
                        y_cur += take
                        ys_cur += take
                        y_idx += 1
                else:
                    if d > 0:
                        q_lo = (r - 1 - cur) // d + 1
                        q_hi = (r - 1 - (seg_hi - 1)) // d + 1
                        if q_lo == q_hi:
                            stack.append((s_cur, cur + q_lo * d, seg_w, k + q_lo))
                        else:
                            split = r - (q_lo - 1) * d
                            w1 = split - cur
                            stack.append((s_cur, cur + q_lo * d, w1, k + q_lo))
                            stack.append((s_cur + w1, split + q_hi * d, seg_w - w1, k + q_hi))
                    else:
                        ad = -d
                        q_lo = (cur - l) // ad + 1
                        q_hi = (seg_hi - 1 - l) // ad + 1
                        if q_lo == q_hi:
                            stack.append((s_cur, cur - q_lo * ad, seg_w, k + q_lo))
                        else:
                            split = l + q_lo * ad
                            w1 = split - cur
                            stack.append((s_cur, cur - q_lo * ad, w1, k + q_lo))
                            stack.append((s_cur + w1, split - q_hi * ad, seg_w - w1, k + q_hi))
                cur = seg_hi
                s_cur += seg_w
                idx += 1

        entries.sort()
        # Coalesce adjacent entries with same bid
        merged_entries = []
        for slo, shi, bid in entries:
            if merged_entries and merged_entries[-1][2] == bid and merged_entries[-1][1] == slo:
                merged_entries[-1] = (merged_entries[-1][0], shi, bid)
            else:
                merged_entries.append((slo, shi, bid))
        # Notice: merged_entries covers [0, u) and [v, Lc) in sorted order!
        # Wait: when sweeping L^{-1}(c) \ D_c, `rank` sweeps [0, u) then [v, Lc) — exact same order!
        entry_map_by_c[c] = merged_entries

    # 3. Assign sorted new symbol IDs!
    #    For each c in 0..sigma-1:
    #      if self_cnt[c] == 0: 1 symbol -> (c, 0) -> sym_to_rule[c]
    #      if self_cnt[c] > 0:  len(block_k_by_c[c]) symbols -> (c, bid) -> run(sym_to_rule[c], k)
    new_sym_to_rule = []
    base_id_for_c = [0] * sigma
    for c in range(sigma):
        base_id_for_c[c] = len(new_sym_to_rule)
        if self_cnt[c] == 0:
            if C[c + 1] > C[c]:
                new_sym_to_rule.append(sym_to_rule[c])
        else:
            rc = sym_to_rule[c]
            for k in block_k_by_c[c]:
                if k == 1:
                    new_sym_to_rule.append(rc)
                else:
                    rid = len(rules)
                    rules.append(('run', rc, k))
                    sym_len.append(sym_len[rc] * k)
                    new_sym_to_rule.append(rid)

    # 4. Sweep `runs` in L-order, deleting D_self (where c == f_sym) and emitting new symbol IDs!
    #    For c with self_cnt[c] > 0, maintain entry cursor (e_idx[c], e_off[c]) stepping through entry_map_by_c[c]!
    e_idx = {c: 0 for c in entry_map_by_c}
    e_off = {c: 0 for c in entry_map_by_c}

    out_runs = []
    out_pos = 0
    new_p = -1
    pos = 0
    f_sym = 0
    while f_sym < sigma and C[f_sym + 1] == 0:
        f_sym += 1

    for c, w in runs:
        rem = w
        cur_pos = pos
        while rem > 0:
            while C[f_sym + 1] <= cur_pos:
                f_sym += 1
            avail = C[f_sym + 1] - cur_pos
            take = rem if rem <= avail else avail
            if c != f_sym:
                # Surviving sub-run!
                if cur_pos <= new_p_old < cur_pos + take:
                    new_p = out_pos + (new_p_old - cur_pos)
                if self_cnt[c] == 0:
                    nid = base_id_for_c[c]
                    if out_runs and out_runs[-1][0] == nid:
                        out_runs[-1] = (nid, out_runs[-1][1] + take)
                    else:
                        out_runs.append((nid, take))
                    out_pos += take
                else:
                    # Step through entry_map_by_c[c] for `take` items!
                    emap = entry_map_by_c[c]
                    ei = e_idx[c]
                    eo = e_off[c]
                    t_rem = take
                    base_nid = base_id_for_c[c]
                    while t_rem > 0:
                        slo, shi, bid = emap[ei]
                        e_avail = (shi - slo) - eo
                        step = t_rem if t_rem <= e_avail else e_avail
                        nid = base_nid + bid
                        if out_runs and out_runs[-1][0] == nid:
                            out_runs[-1] = (nid, out_runs[-1][1] + step)
                        else:
                            out_runs.append((nid, step))
                        out_pos += step
                        t_rem -= step
                        eo += step
                        if eo == (shi - slo):
                            ei += 1
                            eo = 0
                    e_idx[c] = ei
                    e_off[c] = eo
            cur_pos += take
            rem -= take
        pos += w

    assert new_p != -1, f"new_p not found after self_run_contraction! new_p_old={new_p_old}"
    assert out_pos == n - total_deleted, f"out_pos={out_pos} != {n - total_deleted}"
    return (out_runs, new_p, new_p_offset, total_deleted, new_sym_to_rule, len(new_sym_to_rule), None)


def full_split_pair_step(runs, sigma, n, p, p_offset, sym_len, sym_to_rule, rules, C, bucket_runs, in_L):
    """
    Exact Full Bucket-Split Pair Contraction for ANY 2-coloring in_L[0..sigma)!
    Zero contiguity restriction: EVERY pair (a, b) with a in Sigma_L, b in Sigma_R contracts!
    Uncontracted occurrences of a in Sigma_L split ONLY where Sigma_R followers interleave inside B_a.
    Guaranteed O(r_t + sigma) work and worst-case >= 25% deletion when in_L is a greedy Max-Cut!
    """
    # 1. For each a in Sigma_L, scan all occurrences of a in bucket_runs[0..sigma-1] in INCREASING order of b!
    #    WAIT: bucket_runs[b] is ALREADY sorted by b = 0..sigma-1!
    #    For each a in Sigma_L, as b goes 0..sigma-1:
    #      - Whenever we see a in bucket_runs[b] with b in Sigma_L: belongs to current uncontracted block of a!
    #      - Whenever we see a in bucket_runs[b] with b in Sigma_R: contracted pair (a, b), which CLOSES any open uncontracted block!
    #    So in ONE pass over bucket_runs (O(r_t + sigma) work!), we build for every a in Sigma_L:
    #      - events[a]: ordered list of ('U', total_uncon_len) and ('C', b, pair_len)
    #    Let's do it cleanly!
    # Track for each a in Sigma_L: list of events in increasing b-order!
    a_events = [[] for _ in range(sigma)]  # only populated for in_L[a]
    del_cnt = 0
    for b in range(sigma):
        b_in_L = in_L[b]
        for a, w in bucket_runs[b]:
            if in_L[a]:
                ev = a_events[a]
                if b_in_L:
                    if ev and ev[-1][0] == 0:  # 0 = uncon
                        ev[-1][1] += w
                    else:
                        ev.append([0, w])
                else:
                    # contracted (a, b)
                    del_cnt += w
                    if ev and ev[-1][0] == 1 and ev[-1][1] == b:
                        ev[-1][2] += w
                    else:
                        ev.append([1, b, w])

    if del_cnt == 0:
        return None

    # 2. Assign exact sorted new symbol IDs in O(sigma + #events) time (ZERO comparison sort!!)!
    #    For each c in 0..sigma-1:
    #      - If not in_L[c]: surviving uncontracted c (only if bucket B_c has at least one row with L[j] not in Sigma_L)
    #      - If in_L[c]: walk a_events[c] in order! Each ('U', w) gets a new ID pointing to sym_to_rule[c];
    #                    each ('C', b, w) gets a new ID with rule ('pair', sym_to_rule[c], sym_to_rule[b])!
    #    Also build:
    #      - uncon_pieces[a]: list of (new_id, piece_len) for uncontracted a in Sigma_L!
    #      - pair_id_map: for each (a, b) contracted, its new_id (actually we can index directly by b-order!)
    new_sym_to_rule = []
    uncon_pieces = [None] * sigma
    # For c in Sigma_R: track uncon_R_id[c] and also map contracted (a, c) -> new_id
    # Notice: as we iterate a = 0..sigma-1, we visit every ('C', b, w) once; store pair_new_id[(a, b)]?
    # Even cleaner: for c in Sigma_R, how many rows of B_c survive?
    r_surv_cnt = [0] * sigma
    for b in range(sigma):
        if not in_L[b]:
            s = 0
            for a, w in bucket_runs[b]:
                if not in_L[a]:
                    s += w
            r_surv_cnt[b] = s

    uncon_R_id = [-1] * sigma
    pair_to_id = {}  # In C/Rust, pre-assigned directly onto f_runs via 1 counting-sort pass!

    for c in range(sigma):
        if not in_L[c]:
            if r_surv_cnt[c] > 0:
                uncon_R_id[c] = len(new_sym_to_rule)
                new_sym_to_rule.append(sym_to_rule[c])
        else:
            up = []
            rc = sym_to_rule[c]
            for item in a_events[c]:
                if item[0] == 0:
                    nid = len(new_sym_to_rule)
                    new_sym_to_rule.append(rc)
                    up.append((nid, item[1]))
                else:
                    b = item[1]
                    nid = len(new_sym_to_rule)
                    rb = sym_to_rule[b]
                    rid = len(rules)
                    rules.append(('pair', rc, rb))
                    sym_len.append(sym_len[rc] + sym_len[rb])
                    new_sym_to_rule.append(rid)
                    pair_to_id[(c, b)] = nid
            uncon_pieces[c] = up

    # 3. Locate p and update p if p in D!
    pos = 0
    f_sym = 0
    while f_sym < sigma and C[f_sym + 1] == 0:
        f_sym += 1
    p_L = -1
    p_F = -1
    p_rank = -1
    seen = [0] * sigma
    for c, w in runs:
        if pos <= p < pos + w:
            p_L = c
            p_rank = seen[c] + (p - pos)
        seen[c] += w
        if pos <= p < pos + w:
            # Find p_F
            cur_f = f_sym
            while C[cur_f + 1] <= p:
                cur_f += 1
            p_F = cur_f
        pos += w

    if in_L[p_L] and (not in_L[p_F]):
        new_p_old = C[p_L] + p_rank
        p_offset += sym_len[sym_to_rule[p_L]]
    else:
        new_p_old = p

    # 4. Sweep `runs` in L-order to emit contracted runs!
    b_idx = [0] * sigma
    b_off = [0] * sigma
    u_idx = [0] * sigma
    u_off = [0] * sigma

    out_runs = []
    out_pos = 0
    new_p = -1
    pos = 0
    f_sym = 0
    while f_sym < sigma and C[f_sym + 1] == 0:
        f_sym += 1

    for c, w in runs:
        end = pos + w
        if in_L[c]:
            # Surviving portions are where in_L[f_sym] is True!
            # Wait: as surviving occurrences of c sweep L^{-1}(c) cap S,
            # they sweep uncon_pieces[c] sequentially!
            up = uncon_pieces[c]
            ui = u_idx[c]
            uo = u_off[c]
            rem = w
            cur_pos = pos
            while rem > 0:
                while C[f_sym + 1] <= cur_pos:
                    f_sym += 1
                avail = C[f_sym + 1] - cur_pos
                take = rem if rem <= avail else avail
                if in_L[f_sym]:
                    if cur_pos <= new_p_old < cur_pos + take:
                        new_p = out_pos + (new_p_old - cur_pos)
                    t_rem = take
                    while t_rem > 0:
                        nid, plen = up[ui]
                        p_avail = plen - uo
                        step = t_rem if t_rem <= p_avail else p_avail
                        if out_runs and out_runs[-1][0] == nid:
                            out_runs[-1] = (nid, out_runs[-1][1] + step)
                        else:
                            out_runs.append((nid, step))
                        out_pos += step
                        t_rem -= step
                        uo += step
                        if uo == plen:
                            ui += 1
                            uo = 0
                cur_pos += take
                rem -= take
            u_idx[c] = ui
            u_off[c] = uo
        else:
            # c in Sigma_R: NEVER deleted!
            if pos <= new_p_old < end:
                new_p = out_pos + (new_p_old - pos)
            br = bucket_runs[c]
            bi = b_idx[c]
            bo = b_off[c]
            rem = w
            ur_id = uncon_R_id[c]
            while rem > 0:
                a, blen = br[bi]
                avail = blen - bo
                take = rem if rem <= avail else avail
                if in_L[a]:
                    nid = pair_to_id[(a, c)]
                else:
                    nid = ur_id
                if out_runs and out_runs[-1][0] == nid:
                    out_runs[-1] = (nid, out_runs[-1][1] + take)
                else:
                    out_runs.append((nid, take))
                out_pos += take
                rem -= take
                bo += take
                if bo == blen:
                    bi += 1
                    bo = 0
            b_idx[c] = bi
            b_off[c] = bo
        pos = end

    assert new_p != -1
    assert out_pos == n - del_cnt
    return out_runs, new_p, p_offset, del_cnt, new_sym_to_rule, len(new_sym_to_rule)


def greedy_max_cut_coloring(bucket_runs, sigma):
    """
    Deterministic O(r_t + sigma) greedy 2-coloring maximizing directed cut (Sigma_L -> Sigma_R)
    on non-self edges (guaranteed >= 1/4 of all non-self edges!).
    Assigns colors sequentially for s = 0..sigma-1 based on already-colored neighbors!
    """
    # Build undirected adjacency weights for s -> t (only to lower-indexed vertices to make greedy O(r_t)!)
    # Even better: store directed weights to already-colored neighbors!
    # Let color[s] in {0, 1}. Count edges going 0->1 vs 1->0!
    # Classic 1/2-approx greedy undirected max-cut + pick better direction (0->1 or 1->0) => >= 1/4!
    adj_lower = [[] for _ in range(sigma)]
    for b in range(sigma):
        for a, w in bucket_runs[b]:
            if a == b:
                continue
            if a < b:
                adj_lower[b].append((a, w))
            else:
                adj_lower[a].append((b, w))

    col = [0] * sigma
    for s in range(sigma):
        w0 = 0
        w1 = 0
        for nb, w in adj_lower[s]:
            if col[nb] == 0:
                w0 += w
            else:
                w1 += w
        col[s] = 1 if w0 >= w1 else 0

    # Check both directions: 0->1 vs 1->0
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
        return [c == 0 for c in col], c01
    else:
        return [c == 1 for c in col], c10


def decode_recomp(L_init, p_init, variant="A", collect_metrics=False):
    """
    variant="A": Contiguity-preserving (zero-split) + Self-Run Contraction
    variant="B": Full Bucket-Split Greedy Max-Cut + Self-Run Contraction
    """
    n_orig = len(L_init)
    if n_orig == 0:
        return [], {}

    uniq = sorted(set(L_init))
    remap = {c: i for i, c in enumerate(uniq)}
    runs = runs_from_seq([remap[b] for b in L_init])
    sigma = len(uniq)
    rules = [('base', c) for c in uniq]
    sym_len = [1] * sigma
    sym_to_rule = list(range(sigma))

    n = n_orig
    p = p_init
    p_offset = 0
    r0 = len(runs)
    total_run_work = 0
    total_indep_reads = 0  # runs with c in Sigma_R (non-sequential f_runs stream reads)
    max_r = r0
    max_sigma = sigma
    rounds = 0
    self_rounds = 0
    round_trace = []
    t0 = time.time()

    while True:
        r_cur = len(runs)
        total_run_work += r_cur
        if r_cur > max_r:
            max_r = r_cur
        if sigma > max_sigma:
            max_sigma = sigma

        # Compute C and bucket_runs
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

        if self_total == n:
            # Every single row is a self-edge -> p is a fixed point!
            root = sym_to_rule[p_F]
            break

        # Trigger Self-Run Contraction if self-edges are significant (>= 10% of n, or when n is small and self_total > 0)
        # In Variant B, always contract self-edges if self_total >= 0.05 * n or (n - self_total) == 0;
        # even simpler: if self_total >= max(1, int(0.10 * n)), do 1 round of exact Self-Run Contraction!
        if self_total > 0 and (self_total >= 0.10 * n or (variant == "B" and self_total >= 0.05 * n)):
            res = self_run_contraction_step(runs, sigma, n, p, p_offset, sym_len, sym_to_rule, rules, C, bucket_runs)
            if res[6] is not None:
                root = sym_to_rule[res[6]]
                p_offset = res[2]
                break
            runs, p, p_offset, del_cnt, sym_to_rule, sigma, _ = res
            n -= del_cnt
            rounds += 1
            self_rounds += 1
            if collect_metrics:
                round_trace.append((rounds, 'self', n, len(runs), sigma, del_cnt, 0))
            continue

        # Pair contraction step!
        if variant == "B":
            # Compare best threshold (zero-split!) vs Greedy Max-Cut (full-split!)
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

            in_L_mc, mc_cnt = greedy_max_cut_coloring(bucket_runs, sigma)
            # Use threshold if it achieves at least 80% of Max-Cut (keeps zero splits!), else full-split Max-Cut!
            if best_th_cnt >= 0.80 * mc_cnt and best_th_cnt > 0:
                if best_dir == 'asc':
                    in_L = [s < best_th for s in range(sigma)]
                else:
                    in_L = [s >= best_th for s in range(sigma)]
                kind = 'thresh'
            else:
                in_L = in_L_mc
                kind = 'maxcut'

            indep_r = sum(1 for c, _ in runs if not in_L[c])
            total_indep_reads += indep_r
            res = full_split_pair_step(runs, sigma, n, p, p_offset, sym_len, sym_to_rule, rules, C, bucket_runs, in_L)
            runs, p, p_offset, del_cnt, sym_to_rule, sigma = res
            n -= del_cnt
            rounds += 1
            if collect_metrics:
                round_trace.append((rounds, kind, n, len(runs), sigma, del_cnt, indep_r))
        else:
            # Variant A: Contiguity-preserving (zero-split)
            # Pick best among asc/desc threshold, hash, parity
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
            best_cnt = best_th_cnt
            best_choice = ('thresh', best_dir, best_th)
            if sigma >= 4:
                mult = 2654435761
                shift = (rounds % 16) + 8
                hcol = [(((s + 1 + rounds) * mult) >> shift) & 1 for s in range(sigma)]
                for mname, col in [("hash", hcol), ("parity", [s & 1 for s in range(sigma)])]:
                    for flip in (0, 1):
                        in_L_c = [(col[s] ^ flip) == 0 for s in range(sigma)]
                        u = [sigma] * sigma
                        v = [-1] * sigma
                        for b in range(sigma):
                            if in_L_c[b]:
                                for a, _ in bucket_runs[b]:
                                    if in_L_c[a]:
                                        if b < u[a]: u[a] = b
                                        if b > v[a]: v[a] = b
                        cnt = 0
                        for b in range(sigma):
                            if not in_L_c[b]:
                                for a, w in bucket_runs[b]:
                                    if in_L_c[a] and (b < u[a] or b > v[a]):
                                        cnt += w
                        if cnt > best_cnt:
                            best_cnt = cnt
                            best_choice = ('gen', in_L_c, u, v, mname)

            if best_cnt == 0:
                # Fall back to self-run contraction if any self-edges exist
                if self_total > 0:
                    res = self_run_contraction_step(runs, sigma, n, p, p_offset, sym_len, sym_to_rule, rules, C, bucket_runs)
                    if res[6] is not None:
                        root = sym_to_rule[res[6]]
                        p_offset = res[2]
                        break
                    runs, p, p_offset, del_cnt, sym_to_rule, sigma, _ = res
                    n -= del_cnt
                    rounds += 1
                    self_rounds += 1
                    continue
                else:
                    root = sym_to_rule[p_F]
                    break

            if best_choice[0] == 'thresh':
                _, d, th = best_choice
                in_L = [s < th for s in range(sigma)] if d == 'asc' else [s >= th for s in range(sigma)]
                u = [0] * sigma if d == 'asc' else [th] * sigma
                v = [th - 1] * sigma if d == 'asc' else [sigma - 1] * sigma
                kind = 'thresh'
            else:
                _, in_L, u, v, kind = best_choice

            indep_r = sum(1 for c, _ in runs if not in_L[c])
            total_indep_reads += indep_r

            # Execute zero-split contraction
            # Locate p
            pos = 0
            f_sym = 0
            while f_sym < sigma and C[f_sym + 1] == 0:
                f_sym += 1
            p_L = -1
            p_rank = -1
            seen = [0] * sigma
            for c, w in runs:
                if pos <= p < pos + w:
                    p_L = c
                    p_rank = seen[c] + (p - pos)
                seen[c] += w
                pos += w
            if in_L[p_L] and (not in_L[p_F]) and (p_F < u[p_L] or p_F > v[p_L]):
                new_p_old = C[p_L] + p_rank
                p_offset += sym_len[sym_to_rule[p_L]]
            else:
                new_p_old = p

            b_idx = [0] * sigma
            b_off = [0] * sigma
            out_runs = []
            out_pos = 0
            new_p = -1
            pos = 0
            f_sym = 0
            while f_sym < sigma and C[f_sym + 1] == 0:
                f_sym += 1
            for c, w in runs:
                end = pos + w
                if in_L[c]:
                    uc, vc = u[c], v[c]
                    key = (c, uc)
                    rem = w
                    cur_pos = pos
                    while rem > 0:
                        while C[f_sym + 1] <= cur_pos:
                            f_sym += 1
                        avail = C[f_sym + 1] - cur_pos
                        take = rem if rem <= avail else avail
                        if not ((not in_L[f_sym]) and (f_sym < uc or f_sym > vc)):
                            if cur_pos <= new_p_old < cur_pos + take:
                                new_p = out_pos + (new_p_old - cur_pos)
                            if out_runs and out_runs[-1][0] == key:
                                out_runs[-1] = (key, out_runs[-1][1] + take)
                            else:
                                out_runs.append((key, take))
                            out_pos += take
                        cur_pos += take
                        rem -= take
                else:
                    if pos <= new_p_old < end:
                        new_p = out_pos + (new_p_old - pos)
                    br = bucket_runs[c]
                    bi, bo = b_idx[c], b_off[c]
                    rem = w
                    while rem > 0:
                        a, blen = br[bi]
                        avail = blen - bo
                        take = rem if rem <= avail else avail
                        key = (a, c) if (in_L[a] and (c < u[a] or c > v[a])) else (c, 0)
                        if out_runs and out_runs[-1][0] == key:
                            out_runs[-1] = (key, out_runs[-1][1] + take)
                        else:
                            out_runs.append((key, take))
                        out_pos += take
                        rem -= take
                        bo += take
                        if bo == blen:
                            bi += 1
                            bo = 0
                    b_idx[c], b_off[c] = bi, bo
                pos = end

            uniq_keys = sorted(set(k for k, _ in out_runs))
            key_to_id = {k: i for i, k in enumerate(uniq_keys)}
            new_sym_to_rule = []
            for k1, k2 in uniq_keys:
                if (not in_L[k1]) or (k2 == u[k1]):
                    new_sym_to_rule.append(sym_to_rule[k1])
                else:
                    ra, rb = sym_to_rule[k1], sym_to_rule[k2]
                    rid = len(rules)
                    rules.append(('pair', ra, rb))
                    sym_len.append(sym_len[ra] + sym_len[rb])
                    new_sym_to_rule.append(rid)
            runs = [(key_to_id[k], w) for k, w in out_runs]
            sigma = len(uniq_keys)
            sym_to_rule = new_sym_to_rule
            p = new_p
            n -= best_cnt
            rounds += 1
            if collect_metrics:
                round_trace.append((rounds, kind, n, len(runs), sigma, best_cnt, indep_r))

    # Expand grammar root
    stack = [root]
    out = []
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
    if d == n_orig:
        decoded = out[p_offset:] + out[:p_offset]
    else:
        decoded = [out[(p_offset + t) % d] for t in range(n_orig)]

    stats = {
        "rounds": rounds,
        "self_rounds": self_rounds,
        "total_run_work": total_run_work,
        "total_indep_reads": total_indep_reads,
        "max_r": max_r,
        "max_sigma": max_sigma,
        "total_rules": len(rules),
        "r0": r0,
        "elapsed": time.time() - t0,
    }

    if collect_metrics:
        # Compute SLP height, reachable rules, and truncated expansion metrics for L in {16, 32, 64}!
        # Compute height and reachability from root
        # Count rule usage (how many times each rule is expanded when stopping at len <= L)
        height = [0] * len(rules)
        for i, rl in enumerate(rules):
            if rl[0] == 'base':
                height[i] = 0
            elif rl[0] == 'pair':
                height[i] = 1 + max(height[rl[1]], height[rl[2]])
            else:
                height[i] = 1 + height[rl[1]]
        stats["slp_height"] = height[root]

        # Count reachable rules from root
        reach = set()
        stk = [root]
        while stk:
            s = stk.pop()
            if s in reach:
                continue
            reach.add(s)
            rl = rules[s]
            if rl[0] == 'pair':
                stk.append(rl[1])
                stk.append(rl[2])
            elif rl[0] == 'run':
                stk.append(rl[1])
        stats["reachable_rules"] = len(reach)

        # Truncated expansion metrics for cutoffs L in (16, 32, 64)
        exp_metrics = {}
        for L_cut in (16, 32, 64):
            # Walk from root stopping whenever sym_len[s] <= L_cut
            leaves = 0
            internal = 0
            used_small = set()
            stk = [root]
            while stk:
                s = stk.pop()
                if sym_len[s] <= L_cut:
                    leaves += 1
                    used_small.add(s)
                else:
                    internal += 1
                    rl = rules[s]
                    if rl[0] == 'pair':
                        stk.append(rl[2])
                        stk.append(rl[1])
                    elif rl[0] == 'run':
                        _, a, k = rl
                        if sym_len[a] <= L_cut:
                            leaves += k
                            used_small.add(a)
                        else:
                            for _ in range(k):
                                stk.append(a)
            small_bytes = sum(sym_len[s] for s in used_small)
            exp_metrics[L_cut] = {
                "leaves": leaves,
                "internal": internal,
                "total_nodes": leaves + internal,
                "unique_small_rules": len(used_small),
                "small_buf_bytes": small_bytes,
            }
        stats["expansion"] = exp_metrics
        stats["round_trace"] = round_trace

    return decoded, stats


def test_exhaustive():
    for var in ("A", "B"):
        total = 0
        for base, max_n in [(2, 8), (3, 5)]:
            for n in range(1, max_n + 1):
                for z in range(base ** n):
                    L = []
                    tmp = z
                    for _ in range(n):
                        L.append(tmp % base)
                        tmp //= base
                    for p in range(n):
                        ref = ref_inverse_bwt(L, p)
                        got, _ = decode_recomp(L, p, variant=var)
                        assert got == ref, f"FAIL var={var} L={L} p={p}: got {got} != {ref}"
                        total += 1
        print(f"[EXHAUSTIVE VERIFIED Variant {var}] {total} / {total} cases (binary n<=8, ternary n<=5, all p) PASSED!")


def test_adversarial():
    print("\n=== ADVERSARIAL TORTURE SUITE ===")
    cases = []
    # 1. Long single-char run
    cases.append(("a^100000 + b", [0] * 100000 + [1]))
    # 2. Periodic 2-cycle repeat
    cases.append(("(ab)^50000 + c", [0, 1] * 50000 + [2]))
    # 3. Fibonacci word F_22 (len 28657, r=2!)
    a, b = [0], [0, 1]
    while len(b) < 28000:
        a, b = b, b + a
    cases.append((f"Fibonacci (n={len(b)})", b))
    # 4. Thue-Morse (n=16384)
    tm = [0]
    while len(tm) < 16384:
        tm = tm + [1 - x for x in tm]
    cases.append((f"Thue-Morse (n={len(tm)})", tm))
    # 5. Random incompressible bytes (n=50000, sigma=256)
    rng = random.Random(42)
    cases.append(("Random bytes (n=50000)", [rng.randrange(256) for _ in range(50000)]))
    # 6. Adversarial straddled ramp (sigma=128, n=25000): ramp 2..125 repeated + every sym followed by 0,1,126,127!
    adv = []
    for s in range(2, 126):
        adv.extend([s, 0, s, 1, s, 126, s, 127])
    ramp = list(range(2, 126))
    adv.extend(ramp * 200)
    cases.append((f"Adversarial straddled ramp (n={len(adv)})", adv))

    for name, T in cases:
        L, p = bwt_forward(T)
        r0 = len(runs_from_seq(L))
        for var in ("A", "B"):
            got, st = decode_recomp(L, p, variant=var)
            assert got == T, f"MISMATCH on {name} variant {var}!"
            print(
                f"  [{name:34s}] var={var}: r0={r0:5d}, rounds={st['rounds']:3d} (self={st['self_rounds']}), "
                f"run_work={st['total_run_work']:7d} ({st['total_run_work']/r0:5.2f}x r0), "
                f"max_r={st['max_r']:5d} ({st['max_r']/r0:.2f}x), rules={st['total_rules']:5d}"
            )


def test_wikidata():
    print("\n=== ALL 6 AUTHENTICATED WIKIDATA BLOCKS (Variant A vs Variant B) ===")
    with open(os.path.join(BLOCKS_DIR, "manifest.json")) as f:
        manifest = json.load(f)
    for entry in manifest["blocks"]:
        path = os.path.join(BLOCKS_DIR, entry["file"])
        with open(path, "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        n_orig = len(L)
        p = entry["primary_index"]
        # Reference decode once
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
        cur = p
        for t in range(n_orig):
            ref[t] = F[cur]
            cur = FL[cur]
        ref_bytes = bytes(ref)

        print(f"\n--- {entry['file']} (n={n_orig}, r0={entry['r']}, SHA OK) ---")
        for var in ("A", "B"):
            got, st = decode_recomp(L, p, variant=var, collect_metrics=True)
            assert bytes(got) == ref_bytes, f"MISMATCH on {entry['file']} var={var}"
            e16 = st["expansion"][16]
            e32 = st["expansion"][32]
            e64 = st["expansion"][64]
            print(
                f"  Var {var} [EXACT]: rounds={st['rounds']:2d} (self={st['self_rounds']}), "
                f"run_work={st['total_run_work']:7d} ({st['total_run_work']/st['r0']:5.2f}x r0, {st['total_run_work']/n_orig:.2f}x n), "
                f"indep_reads={st['total_indep_reads']:6d} ({st['total_indep_reads']/n_orig:.2f}x n)"
            )
            print(
                f"         max_r={st['max_r']:6d} ({st['max_r']/st['r0']:.2f}x r0), "
                f"max_sigma={st['max_sigma']:5d}, rules={st['total_rules']:6d} (reach={st['reachable_rules']:6d}), "
                f"slp_height={st['slp_height']:2d}"
            )
            print(
                f"         SLP exp L<=16: nodes={e16['total_nodes']:6d} (leaves={e16['leaves']:6d}, small_buf={e16['small_buf_bytes']//1024:3d} KiB) | "
                f"L<=32: nodes={e32['total_nodes']:6d} (leaves={e32['leaves']:6d}, small_buf={e32['small_buf_bytes']//1024:3d} KiB) | "
                f"L<=64: nodes={e64['total_nodes']:6d} (leaves={e64['leaves']:6d}, small_buf={e64['small_buf_bytes']//1024:4d} KiB)"
            )


if __name__ == "__main__":
    test_exhaustive()
    test_adversarial()
    test_wikidata()
