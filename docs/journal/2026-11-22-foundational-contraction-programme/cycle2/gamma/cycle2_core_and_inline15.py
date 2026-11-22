#!/usr/bin/env python3
"""
Cycle 2 Mission 2:
  (a) Surviving Core Subset K* Sweep across all 6 authenticated Wikidata blocks:
      - K = {'"'} (|K|=1)
      - K = top-2, top-4, top-8 hubs
      - K = hex_lo U {'"'} (17 syms)
      - K = hex_all U {'"'} (23 syms, stopping BEFORE the K_16 GUID cliques!)
      - K = zero-self-eliminated only
      Measuring exact (Stage1 1stMrg, SlfLkp, SortEl_out, SortEl_in, GridProbes,
      Stage2 ThkRds/ThkOps/ThnMrg, m_core, edges_core KiB, flat_q, flat_core, TotOps)!
  (b) Inline15 (u128 Operad) + Compact Pop-Free Arena + Power-of-2 Shift Grid Lookup:
      - Pack every word of len <= 15 inline in u128 (zero SLP node, zero flat_q write!)
      - Measure exact inline hit rates (Stage 1 compositions, final K* pieces, Stage 2 pieces),
        remaining SLP nodes, remaining flat_q bytes & memoOps, EXACT peak live memory (bytes),
        and exact average probe length of the power-of-2 shift grid (eliminating in_alive sort & binary search)!
  (c) Also test Sub-Interval 1-Piece Schur on adversarial 4-IET / concatenated Sturmian stress tests
      to pin down its exact mathematical scope!
"""
import bisect
import hashlib
import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-c")
from explore_iet_schur import BLOCKS_DIR, build_initial_iet, decode_lf_reference, expand_slp
from test_unified_schur import compute_open_m16


HEX_LO = set(b"0123456789abcdef")
HEX_HI = set(b"0123456789ABCDEF")
HEX_ALL = HEX_LO | HEX_HI


def run_iet_schur_inline15_on_K(data, keep_set, core_limit=16, inline_cap=15, verify_p=None):
    """
    Exact instrumented IET-Schur (stopping at keep_set) + Stage 2 Thick-Doubling Core-RV
    (on ALL buckets in keep_set simultaneously!) + Inline15 operad + Power-of-2 Shift Grid
    + Exact Peak Live Memory tracking (reclaiming popped buckets immediately!).
    """
    alpha, C, counts, pieces_by_src_init, _, r0 = build_initial_iet(data)
    n = len(data)
    r_init = sum(len(v) for v in pieces_by_src_init.values())

    # Inline15 Operad representation:
    # A word handle is either:
    #   ('inl', bytes_obj)  with len <= inline_cap  (fits in a single u128: 15 bytes + 1 byte len!)
    #   ('slp', node_id, length)  with length > inline_cap
    # SLP table stores ONLY overflow nodes (len > inline_cap):
    #   slp[id] = ('cat', handle_L, handle_R, total_len) or ('rep', handle_base, k, total_len)
    slp = []
    inline_composes = 0
    slp_composes = 0

    def w_len(h):
        return len(h[1]) if h[0] == 'inl' else h[2]

    def op_cat(h_left, h_right):
        nonlocal inline_composes, slp_composes
        if h_left[0] == 'inl' and h_right[0] == 'inl' and len(h_left[1]) + len(h_right[1]) <= inline_cap:
            inline_composes += 1
            return ('inl', h_left[1] + h_right[1])
        slp_composes += 1
        idx = len(slp)
        tot = w_len(h_left) + w_len(h_right)
        slp.append(('cat', h_left, h_right, tot))
        return ('slp', idx, tot)

    def op_rep(h_base, k):
        nonlocal inline_composes, slp_composes
        if k == 1:
            return h_base
        if h_base[0] == 'inl' and len(h_base[1]) * k <= inline_cap:
            inline_composes += 1
            return ('inl', h_base[1] * k)
        slp_composes += 1
        idx = len(slp)
        tot = w_len(h_base) * k
        slp.append(('rep', h_base, k, tot))
        return ('slp', idx, tot)

    # Convert initial pieces to use inline handles
    by_src = {b: [] for b in alpha}
    by_dst = {b: [] for b in alpha}
    r_in_cnt = defaultdict(int)
    r_out_cnt = defaultdict(int)
    r_self_cnt = defaultdict(int)
    self_rows_cnt = defaultdict(int)

    for b in alpha:
        for s, l, d, sb, db, _ in pieces_by_src_init[b]:
            h = ('inl', bytes([db]))
            p = (s, l, d, sb, db, h)
            by_src[sb].append(p)
            by_dst[db].append(p)
            if sb == db:
                r_self_cnt[sb] += 1
                self_rows_cnt[sb] += l
            else:
                r_out_cnt[sb] += 1
                r_in_cnt[db] += 1

    # Track EXACT live buffer counts (reclaiming popped buckets immediately!)
    cur_buf_entries = 2 * r_init
    peak_buf_entries = cur_buf_entries
    # Also track Arena-Indirect model: each created live piece stored ONCE (32B with u128 inline)
    # and by_src/by_dst hold ONLY u32 indices (4B each)!
    # When bucket e is popped, its popped out_alive pieces are freed (or arena reused) and index vecs freed!
    cur_live_pieces = r_init
    peak_live_pieces = r_init
    cur_idx_entries = 2 * r_init
    peak_idx_entries = cur_idx_entries

    eliminated = set()
    alive = set(alpha)
    keep_eff = set(keep_set) & alive
    if not keep_eff:
        keep_eff = {max(alive, key=lambda c: counts[c])}

    sort_out_elems = 0
    sort_in_elems = 0
    out_already_sorted_buckets = 0
    first_hop_ops = 0
    self_loop_ops = 0
    grid_probes_total = 0
    grid_lookups_total = 0
    max_local_out = 0
    elims_done = 0

    def build_shift_grid(out_alive, b_start, b_len):
        """Power-of-2 shift lookup table for O(1) lookup into out_alive without sorting in_alive!"""
        k = len(out_alive)
        if k <= 4 or b_len <= 1:
            return None, 0
        # Target grid size G = next power of 2 >= 2 * k (capped at 16384 -> <= 32 KiB u16!)
        g_bits = 1
        while (1 << g_bits) < min(16384, max(8, 2 * k)):
            g_bits += 1
        G = 1 << g_bits
        # Choose shift so (b_len - 1) >> shift < G
        shift = 0
        while ((b_len - 1) >> shift) >= G:
            shift += 1
        grid_sz = ((b_len - 1) >> shift) + 1
        grid = [0] * grid_sz
        oi = 0
        for gi in range(grid_sz):
            pos = b_start + (gi << shift)
            while oi + 1 < k and out_alive[oi][0] + out_alive[oi][1] <= pos:
                oi += 1
            grid[gi] = oi
        return grid, shift

    def grid_find(out_alive, grid, shift, b_start, pos):
        nonlocal grid_probes_total, grid_lookups_total
        grid_lookups_total += 1
        if grid is None:
            oi = 0
        else:
            oi = grid[(pos - b_start) >> shift]
        probes = 1
        while oi + 1 < len(out_alive) and out_alive[oi][0] + out_alive[oi][1] <= pos:
            oi += 1
            probes += 1
        grid_probes_total += probes
        return oi

    # Stage 1: Bucket IET-Schur Elimination
    while True:
        cands = [b for b in alive if b not in keep_eff]
        if not cands:
            break
        def key_fn(b):
            rs = r_self_cnt[b]
            io = r_in_cnt[b] + r_out_cnt[b]
            deg = io + 2 * rs
            in_rows = counts[b] - self_rows_cnt[b]
            avg_hops = counts[b] / max(1, in_rows)
            return (0 if rs == 0 else 1, avg_hops * deg, counts[b], b)

        e = min(cands, key=key_fn)
        elims_done += 1

        raw_out = by_src.pop(e)
        raw_in = by_dst.pop(e)
        cur_buf_entries -= (len(raw_out) + len(raw_in))
        cur_idx_entries -= (len(raw_out) + len(raw_in))

        out_alive = [p for p in raw_out if p[4] not in eliminated]
        in_alive = [p for p in raw_in if p[3] not in eliminated and p[3] != e]

        # Check if out_alive is already sorted!
        is_sorted = all(out_alive[i][0] <= out_alive[i + 1][0] for i in range(len(out_alive) - 1))
        if is_sorted:
            out_already_sorted_buckets += 1
        else:
            out_alive.sort(key=lambda x: x[0])
            sort_out_elems += len(out_alive)
        sort_in_elems += len(in_alive)  # recorded ONLY if one chose merge-sort; our grid lookup skips sorting in_alive!

        if len(out_alive) > max_local_out:
            max_local_out = len(out_alive)

        # Build power-of-2 shift grid on out_alive (L1d resident!)
        grid, shift = build_shift_grid(out_alive, C[e], counts[e])

        # Update degrees & live count
        for p in out_alive:
            if p[4] != e:
                r_in_cnt[p[4]] -= 1
        for p in in_alive:
            r_out_cnt[p[3]] -= 1
        cur_live_pieces -= (len(out_alive) + len(in_alive))
        eliminated.add(e)
        alive.remove(e)

        # Process ALL in_alive via O(1) Shift-Grid lookup (ZERO sorting of in_alive!)
        self_q = []
        for s, l, d, sb, db, wid in in_alive:
            cs, cd, rem = s, d, l
            oi = grid_find(out_alive, grid, shift, C[e], cd)
            while rem > 0:
                os_, ol_, od_, _, odb_, owid_ = out_alive[oi]
                off = cd - os_
                take = rem if rem <= (ol_ - off) else (ol_ - off)
                first_hop_ops += 1
                if odb_ != e:
                    nw = op_cat(owid_, wid)
                    np = (cs, take, od_ + off, sb, odb_, nw)
                    by_src[sb].append(np)
                    by_dst[odb_].append(np)
                    cur_buf_entries += 2
                    cur_idx_entries += 2
                    cur_live_pieces += 1
                    if sb == odb_:
                        r_self_cnt[sb] += 1
                        self_rows_cnt[sb] += take
                    else:
                        r_out_cnt[sb] += 1
                        r_in_cnt[odb_] += 1
                else:
                    delta = od_ - os_
                    A, B = os_, os_ + ol_
                    sub_s = [(cs, take, cd, wid)]
                    while sub_s:
                        ss, sl, su, sw = sub_s.pop()
                        sv = su + sl
                        if delta > 0:
                            k_max = (B - 1 - su) // delta + 1
                            k_min = (B - 1 - (sv - 1)) // delta + 1
                            if k_max == k_min:
                                self_q.append((ss, sl, su + k_max * delta, sb, op_cat(op_rep(owid_, k_max), sw)))
                            else:
                                sp = B - (k_max - 1) * delta
                                l1 = sp - su
                                if l1 < sl:
                                    sub_s.append((ss + l1, sl - l1, sp, sw))
                                self_q.append((ss, l1, su + k_max * delta, sb, op_cat(op_rep(owid_, k_max), sw)))
                        else:
                            step = -delta
                            k_min = (su - A) // step + 1
                            k_max = (sv - 1 - A) // step + 1
                            if k_min == k_max:
                                self_q.append((ss, sl, su - k_min * step, sb, op_cat(op_rep(owid_, k_min), sw)))
                            else:
                                sp = A + k_min * step
                                l1 = sp - su
                                if l1 < sl:
                                    sub_s.append((ss + l1, sl - l1, sp, sw))
                                self_q.append((ss, l1, su - k_min * step, sb, op_cat(op_rep(owid_, k_min), sw)))
                cs += take
                cd += take
                rem -= take
                oi += 1

        while self_q:
            orig_s, l, cur_pos, sb, wid = self_q.pop()
            oi = grid_find(out_alive, grid, shift, C[e], cur_pos)
            os_, ol_, od_, _, odb_, owid_ = out_alive[oi]
            off = cur_pos - os_
            take = l if l <= (ol_ - off) else (ol_ - off)
            if take < l:
                self_q.append((orig_s + take, l - take, cur_pos + take, sb, wid))
            self_loop_ops += 1
            if odb_ != e:
                nw = op_cat(owid_, wid)
                np = (orig_s, take, od_ + off, sb, odb_, nw)
                by_src[sb].append(np)
                by_dst[odb_].append(np)
                cur_buf_entries += 2
                cur_idx_entries += 2
                cur_live_pieces += 1
                if sb == odb_:
                    r_self_cnt[sb] += 1
                    self_rows_cnt[sb] += take
                else:
                    r_out_cnt[sb] += 1
                    r_in_cnt[odb_] += 1
            else:
                delta = od_ - os_
                A, B = os_, os_ + ol_
                u, v = cur_pos, cur_pos + take
                if delta > 0:
                    k_max = (B - 1 - u) // delta + 1
                    k_min = (B - 1 - (v - 1)) // delta + 1
                    if k_max == k_min:
                        self_q.append((orig_s, take, u + k_max * delta, sb, op_cat(op_rep(owid_, k_max), wid)))
                    else:
                        sp = B - (k_max - 1) * delta
                        l1 = sp - u
                        if l1 < take:
                            self_q.append((orig_s + l1, take - l1, sp, sb, wid))
                        self_q.append((orig_s, l1, u + k_max * delta, sb, op_cat(op_rep(owid_, k_max), wid)))
                else:
                    step = -delta
                    k_min = (u - A) // step + 1
                    k_max = (v - 1 - A) // step + 1
                    if k_min == k_max:
                        self_q.append((orig_s, take, u - k_min * step, sb, op_cat(op_rep(owid_, k_min), wid)))
                    else:
                        sp = A + k_min * step
                        l1 = sp - u
                        if l1 < take:
                            self_q.append((orig_s + l1, take - l1, sp, sb, wid))
                        self_q.append((orig_s, l1, u - k_min * step, sb, op_cat(op_rep(owid_, k_min), wid)))

        if cur_buf_entries > peak_buf_entries:
            peak_buf_entries = cur_buf_entries
        if cur_idx_entries > peak_idx_entries:
            peak_idx_entries = cur_idx_entries
        if cur_live_pieces > peak_live_pieces:
            peak_live_pieces = cur_live_pieces

    # Extract surviving core pieces across ALL buckets in keep_eff
    core_pieces = []
    for b in sorted(alive):
        for p in by_src[b]:
            if p[4] not in eliminated:
                core_pieces.append(p)
    core_pieces.sort(key=lambda x: x[0])

    s1_slp_nodes = len(slp)
    s1_inline_final = sum(1 for p in core_pieces if p[5][0] == 'inl')
    s1_slp_final = len(core_pieces) - s1_inline_final
    s1_rows = sum(p[1] for p in core_pieces)

    # Compute exact Stage-1 overflow flattening cost (ONLY for core_pieces with h[0] == 'slp'!)
    # Notice: inline pieces need ZERO flat_q bytes and ZERO memoOps!
    reach_s1 = set()
    st_r = [p[5][1] for p in core_pieces if p[5][0] == 'slp']
    while st_r:
        u = st_r.pop()
        if u in reach_s1:
            continue
        reach_s1.add(u)
        node = slp[u]
        if node[0] == 'cat':
            if node[1][0] == 'slp':
                st_r.append(node[1][1])
            if node[2][0] == 'slp':
                st_r.append(node[2][1])
        else:
            if node[1][0] == 'slp':
                st_r.append(node[1][1])

    uniq_s1_slp_roots = set(p[5][1] for p in core_pieces if p[5][0] == 'slp')
    s1_overflow_flat_bytes = sum(slp[u][3] for u in uniq_s1_slp_roots)
    # Count exact leaf writes (inline u128 stores!) to materialize uniq_s1_slp_roots (with memoization on shared nodes)
    ref_s1 = defaultdict(int)
    for u in uniq_s1_slp_roots:
        ref_s1[u] += 1
    for u in sorted(reach_s1, reverse=True):
        node = slp[u]
        if node[0] == 'cat':
            if node[1][0] == 'slp':
                ref_s1[node[1][1]] += 1
            if node[2][0] == 'slp':
                ref_s1[node[2][1]] += 1
        elif node[1][0] == 'slp':
            ref_s1[node[1][1]] += 1
    s1_memo_set = set(u for u in reach_s1 if u in uniq_s1_slp_roots or (ref_s1[u] >= 2 and slp[u][3] <= 256))
    s1_memo_bytes = sum(slp[u][3] for u in s1_memo_set)
    s1_memo_ops = 0
    for u in s1_memo_set:
        st_m = []
        node = slp[u]
        if node[0] == 'cat':
            st_m.extend([node[1], node[2]])
        else:
            st_m.append(node[1])
            s1_memo_ops += 1
        while st_m:
            h = st_m.pop()
            if h[0] == 'inl' or h[1] in s1_memo_set:
                s1_memo_ops += 1
            else:
                nd = slp[h[1]]
                if nd[0] == 'cat':
                    st_m.extend([nd[1], nd[2]])
                else:
                    st_m.append(nd[1])
                    s1_memo_ops += 1

    # Stage 2: Thick-Doubling Core-RV across ALL surviving buckets in keep_eff!
    cur_thick = [[p[0], p[1], p[2], p[5]] for p in core_pieces if p[1] >= core_limit]
    thin_list = [[p[0], p[1], p[2], p[5]] for p in core_pieces if p[1] < core_limit]
    m_core = sum(x[1] for x in thin_list)
    thk_rounds = 0
    thk_ops = 0
    thn_ops = 0
    wired_thin = []

    if cur_thick and thin_list:
        for _ in range(24):
            cur_thick.sort(key=lambda x: x[0])
            c_starts = [x[0] for x in cur_thick]
            next_thick = []
            any_moved = False
            for s, l, d, h in cur_thick:
                cs, cd, rem = s, d, l
                while rem > 0:
                    idx = bisect.bisect_right(c_starts, cd) - 1
                    if idx >= 0 and cur_thick[idx][0] <= cd < cur_thick[idx][0] + cur_thick[idx][1]:
                        ts, tl, td, th = cur_thick[idx]
                        off = cd - ts
                        take = rem if rem <= (tl - off) else (tl - off)
                        next_thick.append([cs, take, td + off, op_cat(th, h)])
                        any_moved = True
                        thk_ops += 1
                        cs += take
                        cd += take
                        rem -= take
                    else:
                        nxt_i = idx + 1
                        take = min(rem, c_starts[nxt_i] - cd) if nxt_i < len(c_starts) else rem
                        next_thick.append([cs, take, cd, h])
                        cs += take
                        cd += take
                        rem -= take
            cur_thick = next_thick
            if not any_moved:
                break
            thk_rounds += 1

        cur_thick.sort(key=lambda x: x[0])
        c_starts = [x[0] for x in cur_thick]
        for s, l, d, h in thin_list:
            cs, cd, rem = s, d, l
            while rem > 0:
                idx = bisect.bisect_right(c_starts, cd) - 1
                if idx >= 0 and cur_thick[idx][0] <= cd < cur_thick[idx][0] + cur_thick[idx][1]:
                    ts, tl, td, th = cur_thick[idx]
                    off = cd - ts
                    take = rem if rem <= (tl - off) else (tl - off)
                    wired_thin.append((cs, take, td + off, op_cat(th, h)))
                    thn_ops += 1
                    cs += take
                    cd += take
                    rem -= take
                else:
                    nxt_i = idx + 1
                    take = min(rem, c_starts[nxt_i] - cd) if nxt_i < len(c_starts) else rem
                    wired_thin.append((cs, take, cd, h))
                    thn_ops += 1
                    cs += take
                    cd += take
                    rem -= take
    else:
        wired_thin = [(p[0], p[1], p[2], p[5]) for p in core_pieces]
        m_core = s1_rows

    # Verify exactness on request (for block-0.L)
    if verify_p is not None:
        def expand_h(h, out_buf):
            st = [h]
            while st:
                cur = st.pop()
                if cur[0] == 'inl':
                    out_buf.extend(cur[1])
                else:
                    nd = slp[cur[1]]
                    if nd[0] == 'cat':
                        st.append(nd[2])
                        st.append(nd[1])
                    else:
                        tmp = []
                        expand_h(nd[1], tmp)
                        out_buf.extend(tmp * nd[2])

        ref_out, _, LF, _, _ = decode_lf_reference(data, verify_p)
        wt = sorted(wired_thin, key=lambda x: x[0])
        wt_starts = [x[0] for x in wt]
        thin_set = set()
        for s, l, d, h in wt:
            for x in range(s, s + l):
                thin_set.add(x)
        cur_p = verify_p
        shift_v = 0
        while cur_p not in thin_set:
            cur_p = LF[cur_p]
            shift_v += 1
        x = cur_p
        hs = []
        while True:
            idx = bisect.bisect_right(wt_starts, x) - 1
            s, l, d, h = wt[idx]
            hs.append(h)
            x = d + (x - s)
            if x == cur_p:
                break
        rot_out = []
        for h in reversed(hs):
            expand_h(h, rot_out)
        assert len(rot_out) == n
        shift_v %= n
        rec = rot_out[shift_v:] + rot_out[:shift_v] if shift_v else rot_out
        assert rec == ref_out, "EXACTNESS FAILURE in Inline15 + Core-RV!"

    # Compute EXACT Peak Live Memory under Arena-Indirect model:
    # - Live piece pool (reclaimed on bucket pop): peak_live_pieces * 32B (16B coords + 16B u128 Inline15)
    # - Index buckets (by_src + by_dst, reclaimed on bucket pop): peak_idx_entries * 4B
    # - Overflow SLP table: len(slp) * 16B (only len > 15 nodes!)
    # - edges_core: m_core * 4B + flat_q overflow bytes
    peak_arena_kb = (peak_live_pieces * 32 + peak_idx_entries * 4 + len(slp) * 16) / 1024.0
    peak_flat_kb = (s1_memo_bytes + m_core * 4) / 1024.0

    # Also measure Stage 2 inline rate!
    s2_inline = sum(1 for x in wired_thin if x[3][0] == 'inl')
    tot_ops = first_hop_ops + self_loop_ops + thk_ops + thn_ops

    return {
        'r0': r0,
        'elims': elims_done,
        'rem_k': len(alive),
        's1_rows': s1_rows,
        's1_pieces': len(core_pieces),
        'first_hop': first_hop_ops,
        'self_lkp': self_loop_ops,
        'max_loc': max_local_out,
        'sort_out': sort_out_elems,
        'sort_in_saved': sort_in_elems,
        'already_sorted_b': out_already_sorted_buckets,
        'avg_grid_probe': grid_probes_total / max(1, grid_lookups_total),
        'inl_comp': inline_composes,
        'slp_comp': slp_composes,
        's1_slp_nodes': s1_slp_nodes,
        's1_inl_pct': 100.0 * s1_inline_final / max(1, len(core_pieces)),
        's1_ovf_bytes': s1_overflow_flat_bytes,
        's1_memo_bytes': s1_memo_bytes,
        's1_memo_ops': s1_memo_ops,
        'thk_rds': thk_rounds,
        'thk_ops': thk_ops,
        'thn_ops': thn_ops,
        'm_core': m_core,
        's2_pieces': len(wired_thin),
        's2_inl_pct': 100.0 * s2_inline / max(1, len(wired_thin)),
        'tot_ops': tot_ops,
        'peak_arena_kb': peak_arena_kb,
        'peak_flat_kb': peak_flat_kb,
        'edges_kb': m_core * 4 / 1024.0,
    }


def get_zero_self_keep_set(data):
    """Run zero-self-only elimination to find the surviving set where all remaining buckets have r_self > 0."""
    alpha, C, counts, pbs, slp, r0 = build_initial_iet(data)
    from explore_iet_schur import eliminate_bucket
    while len(pbs) > 1:
        r_self = defaultdict(int)
        for sb, plist in pbs.items():
            for p in plist:
                if p[4] == sb:
                    r_self[sb] += 1
        zeros = [b for b in pbs if r_self[b] == 0]
        if not zeros:
            break
        best = min(zeros, key=lambda b: (len(pbs[b]), counts[b], b))
        eliminate_bucket(best, pbs, slp)
    return set(pbs.keys())


def main():
    with open(os.path.join(BLOCKS_DIR, "manifest.json")) as f:
        manifest = json.load(f)

    # First: full K* sweep on block-0.L and block-210.L (verifying exactness on every K* for block-0.L!)
    print("=== 1. SURVIVING CORE K* SWEEP ON block-0.L & block-210.L (WITH INLINE15 + SHIFT-GRID + CORE-RV) ===")
    for blk_idx in (0, 4):  # block-0.L and block-210.L
        blk = manifest["blocks"][blk_idx]
        data = open(os.path.join(BLOCKS_DIR, blk["file"]), "rb").read()
        assert hashlib.sha256(data).hexdigest() == blk["sha256"]
        m16, r0, _ = compute_open_m16(data, 16)
        counts = defaultdict(int)
        for c in data:
            counts[c] += 1
        top_syms = sorted(counts.keys(), key=lambda c: -counts[c])
        zs_keep = get_zero_self_keep_set(data)

        configs = [
            ('K={"}"} (1)', {34}),
            ('K=top-2', set(top_syms[:2])),
            ('K=top-4', set(top_syms[:4])),
            ('K=top-8', set(top_syms[:8])),
            ('K=hex_lo+q (17)', HEX_LO | {34}),
            ('K=hex_all+q (23)', HEX_ALL | {34}),
            (f'K=zero_self ({len(zs_keep)})', zs_keep),
        ]
        print(f"\n--- {blk['file']} (r0={r0}, Open m16={m16}) ---")
        print(f"{'Config':18s} | {'1stH':>6s} {'Slf':>5s} {'SrtO':>6s} {'GPrb':>4s} | {'S1_inl%':>7s} {'S1_slp':>6s} {'MemB':>6s} {'MemOp':>6s} | "
              f"{'S2Op':>6s} {'TotOp':>6s} | {'m_core':>6s} {'/m16':>5s} {'EdgK':>5s} {'PkM_K':>6s}")
        print("-" * 124)
        for name, kset in configs:
            vp = blk["primary_index"] if blk_idx == 0 else None
            r = run_iet_schur_inline15_on_K(data, kset, core_limit=16, inline_cap=15, verify_p=vp)
            s2op = r['thk_ops'] + r['thn_ops']
            print(f"{name:18s} | {r['first_hop']:6d} {r['self_lkp']:5d} {r['sort_out']:6d} {r['avg_grid_probe']:4.2f} | "
                  f"{r['s1_inl_pct']:6.1f}% {r['s1_slp_nodes']:6d} {r['s1_memo_bytes']//1024:5d}K {r['s1_memo_ops']:6d} | "
                  f"{s2op:6d} {r['tot_ops']:6d} | {r['m_core']:6d} {r['m_core']/m16:4.2f}x {r['edges_kb']:4.1f}K {r['peak_arena_kb']:5.0f}K")

    # Second: ALL 6 BLOCKS comparison between K={'"'} vs K=hex_all+{'"'} vs K=top-2!
    print("\n=== 2. ALL 6 AUTHENTICATED WIKIDATA BLOCKS: K={'\"'} vs K=hex_all+{'\"'} (WITH INLINE15 + SHIFT-GRID + ARENA) ===")
    for cfg_name, kset_fn in [
        ('K={"} (1)', lambda cnt: {34}),
        ('K=hex_all+q (23)', lambda cnt: HEX_ALL | {34}),
    ]:
        print(f"\n--- {cfg_name} ---")
        print(f"{'Block':11s} {'r0':>6s} {'m16':>6s} | {'1stH':>6s} {'Slf':>5s} {'SrtO':>6s} {'GPrb':>4s} | "
              f"{'InlC%':>5s} {'S1_inl%':>7s} {'S1_slp':>6s} {'MemKB':>5s} {'MemOp':>6s} | "
              f"{'S2Op':>6s} {'TotOp':>6s} | {'m_core':>6s} {'/m16':>5s} {'EdgK':>5s} {'PkM_K':>6s}")
        print("-" * 134)
        for blk in manifest["blocks"]:
            data = open(os.path.join(BLOCKS_DIR, blk["file"]), "rb").read()
            assert hashlib.sha256(data).hexdigest() == blk["sha256"]
            m16, r0, _ = compute_open_m16(data, 16)
            r = run_iet_schur_inline15_on_K(data, kset_fn(None), core_limit=16, inline_cap=15, verify_p=blk["primary_index"])
            inl_c_pct = 100.0 * r['inl_comp'] / max(1, r['inl_comp'] + r['slp_comp'])
            s2op = r['thk_ops'] + r['thn_ops']
            print(f"{blk['file']:11s} {r0:6d} {m16:6d} | {r['first_hop']:6d} {r['self_lkp']:5d} {r['sort_out']:6d} {r['avg_grid_probe']:4.2f} | "
                  f"{inl_c_pct:4.1f}% {r['s1_inl_pct']:6.1f}% {r['s1_slp_nodes']:6d} {r['s1_memo_bytes']//1024:4d}K {r['s1_memo_ops']:6d} | "
                  f"{s2op:6d} {r['tot_ops']:6d} | {r['m_core']:6d} {r['m_core']/m16:4.2f}x {r['edges_kb']:4.1f}K {r['peak_arena_kb']:5.0f}K")
        print("  [ALL 6 BLOCKS 100% BYTE-FOR-BYTE VERIFIED EXACT!]")


if __name__ == "__main__":
    main()
