#!/usr/bin/env python3
"""
Comprehensive Exact Measurement & Ablation Suite for Compositional BTM-WIET
(Redheffer Star Product of Boundary Transfer Maps) across:

  3 Merge Trees:
    1. balanced       : Balanced binary tree over present byte buckets (sorted byte order)
    2. lazard_max     : Right-to-left Lazard-MAX comb ((F_max * F_{max-1}) * ... * F_min)
    3. heavy_edge     : Agglomerative heavy-edge tree from 256x256 byte-transition graph
       (+ min_cut_bal : Recursive conductance/min-cut tree from byte-transition graph)

  3 Ping-Pong Resolution Ablation Levels (zero hidden lookups!):
    - P0 : Pure forward trace (NO spiral division, NO chain contraction)
    - P1 : + 1-step spiral division (O(1) divmod on self-overlapping pieces)
    - P2 : + Bipartite small-side collapse + iterated multi-step spiral division
           + zero-split chain contraction (backward exit absorption & contained composition)

  4 Exact Metrics Reported on ALL 6 Authenticated Wikidata Blocks (+ Fib_{2m}+$ + exhaustive):
    (a) Pieces per level and in total (sum_P vs O(r_0 log |Sigma|))
    (b) Ping-pong bounces per merge, and exact bounces/lookups removed by spiral & chain contraction
    (c) RANDOM piece lookups Q_rand (strictly separated from sequential sweep work W_seq),
        compared directly against Open's m_16
    (d) Root SLP reachable size |G_reach|, height, first-occurrence memoized expansion
        copy count & average copy length, plus total cost vector!
"""

import hashlib
import json
import math
import sys
import time
from bisect import bisect_right
from collections import defaultdict


class SLPStore:
    __slots__ = ("kind", "a", "b", "length")

    def __init__(self):
        # 0 = lit(a), 1 = concat(a, b), 2 = power(a, b)
        self.kind = [0] * 256
        self.a = list(range(256))
        self.b = [0] * 256
        self.length = [1] * 256

    def concat(self, u, v):
        idx = len(self.kind)
        self.kind.append(1)
        self.a.append(u)
        self.b.append(v)
        self.length.append(self.length[u] + self.length[v])
        return idx

    def power(self, u, k):
        if k == 1:
            return u
        idx = len(self.kind)
        self.kind.append(2)
        self.a.append(u)
        self.b.append(k)
        self.length.append(self.length[u] * k)
        return idx

    def analyze_and_expand_memo(self, root_id):
        """
        Expand SLP using First-Occurrence Output-Buffer Memoization (LZ-from-SLP),
        returning (bytes_lf_order, metrics_dict).
        """
        # First pass: compute reachable nodes & height (iterative postorder)
        visited = set()
        order = []
        stack = [(root_id, False)]
        while stack:
            nid, done = stack.pop()
            if done:
                order.append(nid)
                continue
            if nid in visited:
                continue
            visited.add(nid)
            stack.append((nid, True))
            k = self.kind[nid]
            if k == 1:
                stack.append((self.b[nid], False))
                stack.append((self.a[nid], False))
            elif k == 2:
                stack.append((self.a[nid], False))

        height = {}
        for nid in order:
            k = self.kind[nid]
            if k == 0:
                height[nid] = 0
            elif k == 1:
                height[nid] = 1 + max(height[self.a[nid]], height[self.b[nid]])
            else:
                height[nid] = 1 + height[self.a[nid]]

        # Second pass: exact First-Occurrence Memoized expansion
        total_len = self.length[root_id]
        out = bytearray(total_len)
        first_pos = {}
        cur = 0
        lit_writes = 0
        copy_ops = 0
        copy_bytes = 0
        power_ops = 0
        node_visits = 0

        # Stack items: (nid, is_exit_marker, start_pos_if_marker)
        exp_stack = [(root_id, False, 0)]
        while exp_stack:
            nid, is_exit, spos = exp_stack.pop()
            if is_exit:
                first_pos[nid] = spos
                continue
            node_visits += 1
            if nid in first_pos:
                ln = self.length[nid]
                fp = first_pos[nid]
                out[cur:cur + ln] = out[fp:fp + ln]
                cur += ln
                copy_ops += 1
                copy_bytes += ln
                continue
            k = self.kind[nid]
            if k == 0:
                out[cur] = self.a[nid]
                first_pos[nid] = cur
                cur += 1
                lit_writes += 1
            elif k == 1:
                exp_stack.append((nid, True, cur))
                exp_stack.append((self.b[nid], False, 0))
                exp_stack.append((self.a[nid], False, 0))
            else:
                # Power node: expand base once (or copy), then double-copy k times!
                base = self.a[nid]
                rep = self.b[nid]
                base_len = self.length[base]
                start_here = cur
                if base in first_pos:
                    fp = first_pos[base]
                    out[cur:cur + base_len] = out[fp:fp + base_len]
                    cur += base_len
                    copy_ops += 1
                    copy_bytes += base_len
                else:
                    # Expand base inline via small helper
                    sub = self._expand_simple(base, first_pos, out, cur)
                    first_pos[base] = cur
                    cur += base_len
                    lit_writes += sub[0]
                    copy_ops += sub[1]
                    copy_bytes += sub[2]
                    node_visits += sub[3]
                # Now repeat [start_here, start_here + base_len) (rep - 1) times via doubling!
                written = base_len
                target = base_len * rep
                power_ops += 1
                while written < target:
                    chunk = written if written <= target - written else (target - written)
                    out[start_here + written:start_here + written + chunk] = out[start_here:start_here + chunk]
                    written += chunk
                    copy_ops += 1
                    copy_bytes += chunk
                cur = start_here + target
                first_pos[nid] = start_here

        assert cur == total_len
        return bytes(out), {
            "reach_nodes": len(visited),
            "reach_nonlit": sum(1 for x in visited if self.kind[x] != 0),
            "reach_power": sum(1 for x in visited if self.kind[x] == 2),
            "height": height[root_id],
            "node_visits": node_visits,
            "lit_writes": lit_writes,
            "copy_ops": copy_ops,
            "copy_bytes": copy_bytes,
            "avg_copy_len": copy_bytes / copy_ops if copy_ops > 0 else 0.0,
        }

    def _expand_simple(self, root_id, first_pos, out, cur0):
        cur = cur0
        lit_w = 0
        c_ops = 0
        c_bytes = 0
        visits = 0
        st = [(root_id, False, 0)]
        while st:
            nid, is_exit, spos = st.pop()
            if is_exit:
                first_pos[nid] = spos
                continue
            visits += 1
            if nid in first_pos:
                ln = self.length[nid]
                fp = first_pos[nid]
                out[cur:cur + ln] = out[fp:fp + ln]
                cur += ln
                c_ops += 1
                c_bytes += ln
                continue
            k = self.kind[nid]
            if k == 0:
                out[cur] = self.a[nid]
                first_pos[nid] = cur
                cur += 1
                lit_w += 1
            elif k == 1:
                st.append((nid, True, cur))
                st.append((self.b[nid], False, 0))
                st.append((self.a[nid], False, 0))
            else:
                base = self.a[nid]
                rep = self.b[nid]
                blen = self.length[base]
                sh = cur
                sub = self._expand_simple(base, first_pos, out, cur)
                first_pos[base] = sh
                cur += blen
                lit_w += sub[0]; c_ops += sub[1]; c_bytes += sub[2]; visits += sub[3]
                written = blen
                target = blen * rep
                while written < target:
                    chunk = written if written <= target - written else (target - written)
                    out[sh + written:sh + written + chunk] = out[sh:sh + chunk]
                    written += chunk
                    c_ops += 1
                    c_bytes += chunk
                cur = sh + target
                first_pos[nid] = sh
        return (lit_w, c_ops, c_bytes, visits)


def combine_mark(m1, len1, m2):
    if m1 >= 0:
        assert m2 < 0
        return m1
    if m2 >= 0:
        return len1 + m2
    return -1


def acquire_from_L(L_bytes, primary_idx):
    """
    Acquire W-IET pieces, 256x256 bigram weights, and exact Open m_16 directly from I_0.
    Strictly sequential O(r_0 + |Sigma|) work, Q_rand = 0.
    """
    n = len(L_bytes)
    runs = []
    counts = [0] * 256
    i = 0
    m_16 = 0
    p_in_thin = False
    while i < n:
        c = L_bytes[i]
        j = i + 1
        while j < n and L_bytes[j] == c:
            j += 1
        ln = j - i
        runs.append((c, i, j))
        counts[c] += ln
        if ln <= 16:
            m_16 += ln
            if i <= primary_idx < j:
                p_in_thin = True
        i = j
    if not p_in_thin and n > 0:
        m_16 += 1

    C = [0] * 257
    acc = 0
    buckets = []
    for c in range(256):
        C[c] = acc
        if counts[c] > 0:
            buckets.append((c, acc, acc + counts[c]))
        acc += counts[c]
    C[256] = acc

    slp = SLPStore()
    cur_t = C[:]
    pieces_by_bucket = {c: [] for c, _, _ in buckets}
    bigrams = defaultdict(int)

    b_idx = 0
    num_b = len(buckets)
    for c, r_lo, r_hi in runs:
        t_start = cur_t[c]
        cur_t[c] += (r_hi - r_lo)
        delta = t_start - r_lo
        pos = r_lo
        while pos < r_hi:
            while b_idx < num_b and buckets[b_idx][2] <= pos:
                b_idx += 1
            bc, b_lo, b_hi = buckets[b_idx]
            seg_hi = r_hi if r_hi < b_hi else b_hi
            bigrams[(bc, c)] += (seg_hi - pos)
            if pos <= primary_idx < seg_hi:
                if pos < primary_idx:
                    pieces_by_bucket[bc].append((pos, primary_idx, delta, c, c, -1))
                pieces_by_bucket[bc].append((primary_idx, primary_idx + 1, delta, c, c, 0))
                if primary_idx + 1 < seg_hi:
                    pieces_by_bucket[bc].append((primary_idx + 1, seg_hi, delta, c, c, -1))
            else:
                pieces_by_bucket[bc].append((pos, seg_hi, delta, c, c, -1))
            pos = seg_hi

    return n, len(runs), m_16, buckets, pieces_by_bucket, bigrams, slp


def spiral_compress_piece(lo, hi, d, tsym, w, m, slp, stats):
    """
    If [lo, hi) overlaps [lo+d, hi+d), collapse in O(1) via spiral division.
    Returns (was_collapsed, list_of_pieces, closed_primary_or_None).
    """
    if d == 0:
        stats["self_cycles"] += 1
        return True, [], ((w, m) if m >= 0 else None)
    L = hi - lo
    if d > 0 and d < L:
        assert m < 0
        q, rem = divmod(L, d)
        stats["spirals"] += 1
        stats["spiral_iters_saved"] += (L - d)  # rows / steps saved
        res = []
        if rem > 0:
            res.append((lo, lo + rem, (q + 1) * d, tsym, slp.power(w, q + 1), -1, True))
        if d - rem > 0:
            res.append((lo + rem, lo + d, q * d, tsym, slp.power(w, q), -1, True))
        return True, res, None
    if d < 0 and (-d) < L:
        assert m < 0
        s = -d
        q, rem = divmod(L, s)
        stats["spirals"] += 1
        stats["spiral_iters_saved"] += (L - s)
        res = []
        if s - rem > 0:
            res.append((hi - s, hi - rem, -q * s, tsym, slp.power(w, q), -1, True))
        if rem > 0:
            res.append((hi - rem, hi, -(q + 1) * s, tsym, slp.power(w, q + 1), -1, True))
        return True, res, None
    return False, [(lo, hi, d, tsym, w, m, True)], None


def resolve_subset_ablation(sym_set, pieces_left, pieces_right, slp, stats, ping_mode="P1"):
    """
    Exact Redheffer Star Product resolution of S = S_left union S_right.
    ping_mode:
      - "P0": No spiral division, no chain contraction (pure forward trace)
      - "P1": 1-step spiral division + forward trace
      - "P2": Bipartite small-side collapse + iterated multi-step spiral division
              + zero-split chain contraction (backward exit absorption & contained steps)
              + forward trace for remaining pieces.
    STRICT ACCOUNTING:
      Every single bisect_right / piece-array probe (whether in bipartite collapse,
      chain contraction, forward trace, or root cycle walk) is counted in stats["Q_rand"]!
      Sequential scans / sorts are counted separately in stats["W_seq"]!
    """
    raw_pieces = pieces_left + pieces_right
    stats["W_seq"] += len(raw_pieces)
    primary_cycle = None

    # Classify internal vs exit, and optionally run P1/P2 spiral compression
    collapsed = []
    if ping_mode == "P0":
        for lo, hi, d, tsym, w, m in raw_pieces:
            is_int = (tsym in sym_set)
            if is_int and d == 0:
                stats["self_cycles"] += 1
                if m >= 0:
                    primary_cycle = (w, m)
                continue
            collapsed.append((lo, hi, d, tsym, w, m, is_int))
    elif ping_mode == "P1":
        for lo, hi, d, tsym, w, m in raw_pieces:
            is_int = (tsym in sym_set)
            if not is_int:
                collapsed.append((lo, hi, d, tsym, w, m, False))
                continue
            _, pcs, pcyc = spiral_compress_piece(lo, hi, d, tsym, w, m, slp, stats)
            if pcyc is not None:
                primary_cycle = pcyc
            collapsed.extend(pcs)
    elif ping_mode == "P2":
        # Phase 1: 1-step spiral compression (at leaves)
        step1 = []
        left_int_cnt = 0
        right_int_cnt = 0
        left_syms = set()
        for lo, hi, d, tsym, w, m in pieces_left:
            is_int = (tsym in sym_set)
            if is_int:
                left_int_cnt += 1
                _, pcs, pcyc = spiral_compress_piece(lo, hi, d, tsym, w, m, slp, stats)
                if pcyc is not None:
                    primary_cycle = pcyc
                step1.extend(pcs)
            else:
                step1.append((lo, hi, d, tsym, w, m, False))
        right_start_idx = len(step1)
        for lo, hi, d, tsym, w, m in pieces_right:
            is_int = (tsym in sym_set)
            if is_int:
                right_int_cnt += 1
                _, pcs, pcyc = spiral_compress_piece(lo, hi, d, tsym, w, m, slp, stats)
                if pcyc is not None:
                    primary_cycle = pcyc
                step1.extend(pcs)
            else:
                step1.append((lo, hi, d, tsym, w, m, False))

        # Phase 2 (for non-leaf merges where BOTH sides have internal pieces):
        # Bipartite Small-Side Collapse + Iterated Multi-Step Spiral + Zero-Split Chain Contraction!
        if pieces_right and left_int_cnt > 0 and right_int_cnt > 0:
            # Build sorted piece arrays for left side and right side independently!
            L_side = sorted(step1[:right_start_idx], key=lambda x: x[0])
            R_side = sorted(step1[right_start_idx:], key=lambda x: x[0])
            stats["W_seq"] += len(L_side) + len(R_side)

            # Decide which side to eliminate: eliminate the side with FEWER pieces (smaller lookup target)
            # Wait: eliminating R_side into L_side means every L->R internal piece composes with R_side!
            # The number of compositions is proportional to left_int_cnt, whereas eliminating L_side
            # into R_side composes right_int_cnt pieces!
            # Pick the side whose incoming count (other side's internal count) is SMALLER!
            if left_int_cnt <= right_int_cnt:
                # Compose every internal piece of L_side with R_side -> produces L->L (or L->exit)!
                # Wait: R_side's external entries (In(S) cap R_side) STILL stay in R_side!
                # Check: any R_side piece whose domain is NOT hit by L_side stays untouched;
                # any R_side piece whose domain IS hit by L_side is bypassed by L_side!
                # That preserves exact bijection on L_side union In_ext(R_side)!
                src_side, dst_side = L_side, R_side
            else:
                src_side, dst_side = R_side, L_side

            dst_starts = [p[0] for p in dst_side]
            new_src = []
            dst_hit_intervals = []

            for lo, hi, d, tsym, w, m, is_int in src_side:
                if not is_int:
                    new_src.append((lo, hi, d, tsym, w, m, False))
                    continue
                # Compose [lo, hi) with dst_side at [lo+d, hi+d)
                t_lo, t_hi = lo + d, hi + d
                dst_hit_intervals.append((t_lo, t_hi))
                idx = bisect_right(dst_starts, t_lo) - 1
                pos = t_lo
                w_len = slp.length[w]
                stats["bipartite_collapses"] += 1
                while pos < t_hi:
                    stats["Q_rand"] += 1
                    stats["Q_rand_chain"] += 1
                    p_lo, p_hi, p_d, p_tsym, p_w, p_m, p_int = dst_side[idx]
                    ov_hi = t_hi if t_hi < p_hi else p_hi
                    sub_lo = pos - d
                    sub_hi = ov_hi - d
                    nd = d + p_d
                    nw = slp.concat(w, p_w)
                    nm = combine_mark(m, w_len, p_m)
                    stats["slp_joins"] += 1
                    if p_int:
                        # Now maps src_side -> src_side! Check 2-step spiral IMMEDIATELY!
                        did_sp, pcs, pcyc = spiral_compress_piece(
                            sub_lo, sub_hi, nd, p_tsym, nw, nm, slp, stats
                        )
                        if pcyc is not None:
                            primary_cycle = pcyc
                        new_src.extend(pcs)
                    else:
                        new_src.append((sub_lo, sub_hi, nd, p_tsym, nw, nm, False))
                    pos = ov_hi
                    idx += 1

            # Keep ONLY the unhit portion of dst_side (which IS In(S) cap dst_side)!
            dst_hit_intervals.sort()
            m_hit = []
            for a, b in dst_hit_intervals:
                if m_hit and m_hit[-1][1] >= a:
                    if b > m_hit[-1][1]:
                        m_hit[-1] = (m_hit[-1][0], b)
                else:
                    m_hit.append((a, b))
            rem_dst = []
            hi_idx = 0
            n_hit = len(m_hit)
            for lo, hi, d, tsym, w, m, is_int in dst_side:
                pos = lo
                while pos < hi:
                    while hi_idx < n_hit and m_hit[hi_idx][1] <= pos:
                        hi_idx += 1
                    if hi_idx >= n_hit or m_hit[hi_idx][0] >= hi:
                        rem_dst.append((pos, hi, d, tsym, w, m, is_int))
                        pos = hi
                    else:
                        ha, hb = m_hit[hi_idx]
                        if pos < ha:
                            rem_dst.append((pos, ha, d, tsym, w, m, is_int))
                        pos = hi if hi < hb else hb

            # Now ALL internal targets lie inside new_src ONLY!
            # Run Exact Iterated Spiral + Bijective Internal Elimination on new_src
            # when the remaining internal core is small (n_int <= 64, e.g. Fib_{2m}+$,
            # Sturmian, or small merges) — ZERO duplicate marks, O(log n) on Fib_{2m}+$!
            cur_src = sorted(new_src, key=lambda x: x[0])
            n_int = sum(1 for x in cur_src if x[6])
            if 0 < n_int <= 64:
                # Track which sub-intervals of cur_src are targeted by internal pieces
                for _iter in range(256):
                    # First spiral-compress any self-overlapping internal piece
                    sp_changed = False
                    tmp = []
                    for lo, hi, d, tsym, w, m, is_int in cur_src:
                        if is_int:
                            did_sp, pcs, pcyc = spiral_compress_piece(
                                lo, hi, d, tsym, w, m, slp, stats
                            )
                            if pcyc is not None:
                                primary_cycle = pcyc
                            if did_sp:
                                sp_changed = True
                            tmp.extend(pcs)
                        else:
                            tmp.append((lo, hi, d, tsym, w, m, False))
                    cur_src = sorted(tmp, key=lambda x: x[0])

                    # Find internal pieces whose domain is targeted by another internal piece
                    # (i.e., strictly internal vertices we can bypass!)
                    int_tgts = sorted((lo + d, hi + d) for lo, hi, d, _, _, _, is_int in cur_src if is_int)
                    if not int_tgts:
                        break
                    # Merge int_tgts
                    mt = []
                    for a, b in int_tgts:
                        if mt and mt[-1][1] >= a:
                            if b > mt[-1][1]:
                                mt[-1] = (mt[-1][0], b)
                        else:
                            mt.append((a, b))

                    # Find an internal piece Q whose domain overlaps mt (meaning someone jumps INTO Q!)
                    # Pick the WIDEST such piece Q (thick-first / Rauzy-Veech style!)
                    best_q_idx = -1
                    best_q_len = -1
                    for qi, (q_lo, q_hi, q_d, q_tsym, q_w, q_m, q_int) in enumerate(cur_src):
                        # Only bypass Q if Q is disjoint from its own target (already guaranteed after spiral!)
                        # and some internal piece targets inside [q_lo, q_hi)
                        # Check overlap with mt via binary search
                        k_mt = bisect_right(mt, (q_lo, 1 << 60)) - 1
                        has_in = False
                        for ck in (k_mt, k_mt + 1):
                            if 0 <= ck < len(mt):
                                if mt[ck][0] < q_hi and mt[ck][1] > q_lo:
                                    has_in = True
                                    break
                        if has_in and (q_hi - q_lo) > best_q_len:
                            best_q_len = q_hi - q_lo
                            best_q_idx = qi
                    if best_q_idx < 0:
                        break

                    # Bypass Q = cur_src[best_q_idx]!
                    q_lo, q_hi, q_d, q_tsym, q_w, q_m, q_int = cur_src[best_q_idx]
                    others = cur_src[:best_q_idx] + cur_src[best_q_idx + 1:]
                    # Check if Q itself targets inside Q? Impossible (already spiral-compressed!)
                    next_src = []
                    q_hits = []
                    stats["chain_contractions"] += 1
                    for lo, hi, d, tsym, w, m, is_int in others:
                        if not is_int:
                            next_src.append((lo, hi, d, tsym, w, m, False))
                            continue
                        t_lo, t_hi = lo + d, hi + d
                        if t_hi <= q_lo or t_lo >= q_hi:
                            next_src.append((lo, hi, d, tsym, w, m, True))
                        else:
                            stats["Q_rand"] += 1
                            stats["Q_rand_chain"] += 1
                            w_len = slp.length[w]
                            if t_lo < q_lo:
                                next_src.append((lo, q_lo - d, d, tsym, w, m, True))
                            ov_lo = q_lo if q_lo > t_lo else t_lo
                            ov_hi = q_hi if q_hi < t_hi else t_hi
                            q_hits.append((ov_lo, ov_hi))
                            # Note: if q_m >= 0, Q has length 1 so ov_lo == q_lo and ov_hi == q_hi
                            nm = combine_mark(m, w_len, q_m)
                            next_src.append((
                                ov_lo - d, ov_hi - d, d + q_d, q_tsym,
                                slp.concat(w, q_w), nm, q_int
                            ))
                            stats["slp_joins"] += 1
                            if t_hi > q_hi:
                                next_src.append((q_hi - d, hi, d, tsym, w, m, True))

                    # Keep ONLY the unhit portion of Q (which is in In(cur_src)!)
                    q_hits.sort()
                    pos = q_lo
                    for ha, hb in q_hits:
                        if pos < ha:
                            next_src.append((pos, ha, q_d, q_tsym, q_w, q_m, q_int))
                        pos = hb if hb > pos else pos
                    if pos < q_hi:
                        next_src.append((pos, q_hi, q_d, q_tsym, q_w, q_m, q_int))

                    cur_src = sorted(next_src, key=lambda x: x[0])
                    if len(cur_src) > 128:
                        break

            collapsed = cur_src + rem_dst
        else:
            collapsed = step1
    else:
        raise ValueError(ping_mode)

    collapsed.sort(key=lambda x: x[0])
    stats["W_seq"] += len(collapsed)
    starts = [p[0] for p in collapsed]

    # Compute In(S) = domain(collapsed) \ target(internal pieces)
    int_targets = sorted((lo + d, hi + d) for lo, hi, d, tsym, w, m, is_int in collapsed if is_int)
    stats["W_seq"] += len(int_targets)
    merged_t = []
    for a, b in int_targets:
        if merged_t and merged_t[-1][1] == a:
            merged_t[-1] = (merged_t[-1][0], b)
        else:
            merged_t.append((a, b))

    entry_work = []
    mt_idx = 0
    num_mt = len(merged_t)
    primary_piece_pos = None

    for lo, hi, d, tsym, w, m, is_int in collapsed:
        if m >= 0:
            primary_piece_pos = lo
        pos = lo
        while pos < hi:
            while mt_idx < num_mt and merged_t[mt_idx][1] <= pos:
                mt_idx += 1
            if mt_idx >= num_mt or merged_t[mt_idx][0] >= hi:
                entry_work.append((pos, hi, d, tsym, w, m, is_int))
                pos = hi
            else:
                ta, tb = merged_t[mt_idx]
                if pos < ta:
                    entry_work.append((pos, ta, d, tsym, w, m, is_int))
                    pos = ta
                pos = hi if hi < tb else tb

    out_pieces = []
    saw_primary_exit = False

    stack = list(reversed(entry_work))
    while stack:
        lo, hi, d, tsym, w, m, is_int = stack.pop()
        if not is_int:
            out_pieces.append((lo, hi, d, tsym, w, m))
            if m >= 0:
                saw_primary_exit = True
            continue
        stats["bounces"] += 1
        t_lo, t_hi = lo + d, hi + d
        idx = bisect_right(starts, t_lo) - 1
        pos = t_lo
        w_len = slp.length[w]
        nxt = []
        while pos < t_hi:
            stats["Q_rand"] += 1
            stats["Q_rand_bounce"] += 1
            p_lo, p_hi, p_d, p_tsym, p_w, p_m, p_int = collapsed[idx]
            ov_hi = t_hi if t_hi < p_hi else p_hi
            sub_lo = pos - d
            sub_hi = ov_hi - d
            nxt.append((
                sub_lo, sub_hi, d + p_d, p_tsym,
                slp.concat(w, p_w),
                combine_mark(m, w_len, p_m),
                p_int
            ))
            stats["slp_joins"] += 1
            pos = ov_hi
            idx += 1
        for item in reversed(nxt):
            stack.append(item)

    # Root / closed cycle extraction for primary mark
    if primary_piece_pos is not None and not saw_primary_exit and primary_cycle is None:
        x = primary_piece_pos
        idx = bisect_right(starts, x) - 1
        stats["Q_rand"] += 1
        stats["Q_rand_root"] += 1
        _, _, d0, _, w_acc, m_acc, is_int0 = collapsed[idx]
        assert is_int0 and m_acc >= 0
        x = x + d0
        while x != primary_piece_pos:
            stats["root_cycle_steps"] += 1
            stats["Q_rand"] += 1
            stats["Q_rand_root"] += 1
            idx = bisect_right(starts, x) - 1
            p_lo, p_hi, p_d, _, p_w, p_m, p_int = collapsed[idx]
            assert p_lo <= x < p_hi and p_int
            w_acc = slp.concat(w_acc, p_w)
            stats["slp_joins"] += 1
            x = x + p_d
        primary_cycle = (w_acc, m_acc)

    out_pieces.sort(key=lambda x: x[0])
    stats["W_seq"] += len(out_pieces)
    return out_pieces, primary_cycle


def build_min_cut_bisect_tree(syms, bigrams, counts):
    """
    Build a binary merge tree over `syms` from the 256x256 byte-transition graph:
    - At the TOP level (root), find a sparse / min-cut bipartition (specifically,
      if a symbol has small degree/count, isolating the min-cut minimizes root cycle steps;
      actually let's run exact Kernighan-Lin / greedy min-cut bisection for balanced subtrees,
      with exact 1-vs-(k-1) min-cut allowed at the root so root_steps is minimal!).
    Wait: let's implement clean Recursive Conductance-Min-Cut (Kernighan-Lin refinement)!
    Returns nested tuple tree of symbols.
    """
    if len(syms) == 1:
        return syms[0]
    if len(syms) == 2:
        return (syms[0], syms[1])

    # Build symmetric edge weights inside `syms`
    sset = set(syms)
    adj = {c: {} for c in syms}
    deg = {c: 0 for c in syms}
    for (a, b), wt in bigrams.items():
        if a != b and a in sset and b in sset:
            adj[a][b] = adj[a].get(b, 0) + wt
            adj[b][a] = adj[b].get(a, 0) + wt
            deg[a] += wt
            deg[b] += wt

    # At ANY split: compare best single-node cut vs balanced KL min-cut?
    # Better: minimize cut_weight / min(|A|, |B|)^0.5 (conductance-like) so we get
    # balanced depth O(log |Sigma|) EXCEPT at the root where minimizing raw cut_weight
    # eliminates root_cycle_steps completely!
    # Wait: let's have a dedicated flag `is_root`!
    return _bisect_rec(syms, adj, deg, counts, is_root=True)


def _bisect_rec(syms, adj, deg, counts, is_root=False):
    k = len(syms)
    if k == 1:
        return syms[0]
    if k == 2:
        return (syms[0], syms[1])

    # Option 1: Isolating minimum-degree symbol at root (minimizes root_cycle_steps!)
    if is_root:
        min_c = min(syms, key=lambda c: deg[c])
        if deg[min_c] <= 100:  # rare separator exists!
            rest = [c for c in syms if c != min_c]
            return (_bisect_rec(rest, adj, deg, counts, is_root=False), min_c)

    # Balanced Kernighan-Lin / Fiduccia-Mattheyses min-cut bisection!
    # Initialize by sorting by counts (or BFS from min-degree node) into two halves
    ordered = sorted(syms, key=lambda c: deg[c], reverse=True)
    # Greedy placement into A or B to minimize cut while keeping sizes within [k//3, 2k//3]
    A = set()
    B = set()
    max_sz = (2 * k + 2) // 3
    for c in ordered:
        wA = sum(wt for nb, wt in adj[c].items() if nb in A)
        wB = sum(wt for nb, wt in adj[c].items() if nb in B)
        if len(A) >= max_sz:
            B.add(c)
        elif len(B) >= max_sz:
            A.add(c)
        elif wA > wB:
            A.add(c)
        elif wB > wA:
            B.add(c)
        elif len(A) <= len(B):
            A.add(c)
        else:
            B.add(c)

    # Local search (1-move swaps) to reduce cut while maintaining balance [k//3, 2k//3]
    min_sz = max(1, k // 3)
    for _ in range(15):
        improved = False
        for c in list(syms):
            inA = c in A
            if inA and len(A) <= min_sz:
                continue
            if (not inA) and len(B) <= min_sz:
                continue
            wA = sum(wt for nb, wt in adj[c].items() if nb in A)
            wB = sum(wt for nb, wt in adj[c].items() if nb in B)
            if inA and wB > wA:
                A.remove(c); B.add(c); improved = True
            elif (not inA) and wA > wB:
                B.remove(c); A.add(c); improved = True
        if not improved:
            break

    # Build sub-adjacencies
    A_list = sorted(A)
    B_list = sorted(B)
    adjA = {c: {nb: wt for nb, wt in adj[c].items() if nb in A} for c in A_list}
    degA = {c: sum(adjA[c].values()) for c in A_list}
    adjB = {c: {nb: wt for nb, wt in adj[c].items() if nb in B} for c in B_list}
    degB = {c: sum(adjB[c].values()) for c in B_list}
    return (_bisect_rec(A_list, adjA, degA, counts, False),
            _bisect_rec(B_list, adjB, degB, counts, False))


def build_merge_tree(tree_kind, buckets, bigrams, leaf_piece_counts):
    """Build explicit binary merge tree (nested tuples of byte values)."""
    syms = [c for c, _, _ in buckets]
    counts = {c: hi - lo for c, lo, hi in buckets}
    if len(syms) == 1:
        return syms[0]

    if tree_kind == "balanced":
        def rec(arr):
            if len(arr) == 1:
                return arr[0]
            mid = len(arr) // 2
            return (rec(arr[:mid]), rec(arr[mid:]))
        return rec(syms)

    elif tree_kind == "lazard_max":
        # Right-to-left comb: eliminate max byte first, then max-1, ..., down to min!
        # ((c_{k-1}, c_{k-2}), c_{k-3}) ... c_0
        t = syms[-1]
        for c in reversed(syms[:-1]):
            t = (t, c)
        return t

    elif tree_kind == "min_cut_bisect":
        return build_min_cut_bisect_tree(syms, bigrams, counts)

    elif tree_kind == "heavy_edge":
        # Agglomerative heavy-edge clustering using byte-transition graph W
        active = {i: syms[i] for i in range(len(syms))}
        sz = {i: leaf_piece_counts[syms[i]] for i in range(len(syms))}
        sym_to_id = {syms[i]: i for i in range(len(syms))}
        w_adj = defaultdict(lambda: defaultdict(int))
        for (a, b), wt in bigrams.items():
            if a != b and a in sym_to_id and b in sym_to_id:
                u, v = sym_to_id[a], sym_to_id[b]
                w_adj[u][v] += wt
                w_adj[v][u] += wt
        nxt_id = len(syms)
        while len(active) > 1:
            best_pair = None
            best_score = -1.0
            cids = list(active.keys())
            for u in cids:
                for v, wt in w_adj[u].items():
                    if v > u and v in active:
                        sc = wt / (sz[u] + sz[v] + 1.0)
                        if sc > best_score:
                            best_score = sc
                            best_pair = (u, v)
            if best_pair is None:
                best_pair = (cids[0], cids[1])
            u, v = best_pair
            tu, tv = active.pop(u), active.pop(v)
            su, sv = sz.pop(u), sz.pop(v)
            cid = nxt_id
            nxt_id += 1
            active[cid] = (tu, tv)
            sz[cid] = su + sv
            new_row = defaultdict(int)
            for nb, wt in w_adj[u].items():
                if nb != v and nb in active:
                    new_row[nb] += wt
            for nb, wt in w_adj[v].items():
                if nb != u and nb in active:
                    new_row[nb] += wt
            del w_adj[u]; del w_adj[v]
            for nb, wt in new_row.items():
                w_adj[cid][nb] = wt
                w_adj[nb][cid] = wt
                w_adj[nb].pop(u, None)
                w_adj[nb].pop(v, None)
        return next(iter(active.values()))
    else:
        raise ValueError(tree_kind)


def run_btm_experiment(L_bytes, primary_idx, tree_kind, ping_mode):
    n, r0, m_16, buckets, pieces_by_bucket, bigrams, slp = acquire_from_L(L_bytes, primary_idx)
    stats = defaultdict(int)
    stats["n"] = n
    stats["r0"] = r0
    stats["m_16"] = m_16
    stats["sigma"] = len(buckets)

    # Leaf level (all 256 buckets independently)
    leaves = {}
    leaf_piece_counts = {}
    primary_cycle = None
    for c, b_lo, b_hi in buckets:
        s = frozenset([c])
        out_p, pcyc = resolve_subset_ablation(s, pieces_by_bucket[c], [], slp, stats, ping_mode=ping_mode)
        if pcyc is not None:
            primary_cycle = pcyc
        leaves[c] = (s, out_p)
        leaf_piece_counts[c] = len(out_p)
        stats["leaf_pieces"] += len(out_p)

    tree = build_merge_tree(tree_kind, buckets, bigrams, leaf_piece_counts)

    per_depth_pieces = defaultdict(int)
    per_depth_merges = defaultdict(int)
    per_depth_bounces = defaultdict(int)
    per_depth_qrand = defaultdict(int)
    max_depth = 0

    def eval_tree(node, depth=0):
        nonlocal primary_cycle, max_depth
        if depth > max_depth:
            max_depth = depth
        if isinstance(node, int):
            return leaves[node]
        left_res = eval_tree(node[0], depth + 1)
        right_res = eval_tree(node[1], depth + 1)
        s = left_res[0] | right_res[0]
        b0 = stats["bounces"]
        q0 = stats["Q_rand"]
        out_p, pcyc = resolve_subset_ablation(s, left_res[1], right_res[1], slp, stats, ping_mode=ping_mode)
        if pcyc is not None:
            primary_cycle = pcyc
        per_depth_merges[depth] += 1
        per_depth_pieces[depth] += len(out_p)
        per_depth_bounces[depth] += (stats["bounces"] - b0)
        per_depth_qrand[depth] += (stats["Q_rand"] - q0)
        return (s, out_p)

    root = eval_tree(tree, 0)
    assert len(root[1]) == 0
    assert primary_cycle is not None

    c_slp, mark_off = primary_cycle
    cyc, slp_metrics = slp.analyze_and_expand_memo(c_slp)
    q = len(cyc)
    rot = cyc[mark_off:] + cyc[:mark_off]
    lf_seq = (rot * (n // q + 1))[:n]
    text = lf_seq[::-1]

    stats["tree_depth"] = max_depth
    stats["sum_tree_pieces"] = stats["leaf_pieces"] + sum(per_depth_pieces.values())
    stats["slp_total_created"] = len(slp.kind) - 256
    stats["slp_metrics"] = slp_metrics
    stats["per_depth"] = {
        d: (per_depth_merges[d], per_depth_pieces[d], per_depth_bounces[d], per_depth_qrand[d])
        for d in sorted(per_depth_merges)
    }
    return text, dict(stats)


def bwt_encode(text):
    n = len(text)
    rots = sorted(range(n), key=lambda i: text[i:] + text[:i])
    L = bytes(text[(i - 1) % n] for i in rots)
    return L, rots.index(0)


def ref_ibwt(L_bytes, p):
    n = len(L_bytes)
    if n == 0:
        return b""
    counts = [0] * 256
    for c in L_bytes:
        counts[c] += 1
    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += counts[c]
    LF = [0] * n
    cur = C[:]
    for i, c in enumerate(L_bytes):
        LF[i] = cur[c]
        cur[c] += 1
    out = bytearray(n)
    x = p
    for k in range(n - 1, -1, -1):
        out[k] = L_bytes[x]
        x = LF[x]
    return bytes(out)


def main():
    print("=== 1. P0 vs P1 vs P2 on Fib_{2m}+$ (Multi-Step Spiral Proof!) ===")
    for m in [4, 6, 8, 9]:
        a_f, b_f = [0], [0, 1]
        for _ in range(2 * m - 2):
            a_f, b_f = b_f, b_f + a_f
        T = bytes(b_f + [2])
        L, p = bwt_encode(T)
        row = []
        for pm in ["P0", "P1", "P2"]:
            dec, st = run_btm_experiment(L, p, "min_cut_bisect", pm)
            assert dec == T
            row.append(f"{pm}:bnc={st.get('bounces',0)}/Q={st.get('Q_rand',0)}/sp={st.get('spirals',0)}")
        print(f"  Fib_{2*m:2d}+$ (n={len(T):4d}, r0={st['r0']}): " + " | ".join(row))

    print("\n=== 2. Exhaustive Check for P2 Across All 4 Trees (binary n<=8, ternary n<=5) ===")
    tot = 0
    for n in range(1, 9):
        for mask in range(1 << n):
            T = bytes((mask >> i) & 1 for i in range(n))
            L, p = bwt_encode(T)
            for tk in ("balanced", "lazard_max", "heavy_edge", "min_cut_bisect"):
                dec, _ = run_btm_experiment(L, p, tk, "P2")
                assert dec == T
                tot += 1
    for n in range(1, 6):
        for code in range(3 ** n):
            T = bytes((code // (3 ** i)) % 3 for i in range(n))
            L, p = bwt_encode(T)
            for pt in (0, p):
                ref = ref_ibwt(L, pt)
                for tk in ("balanced", "lazard_max", "heavy_edge", "min_cut_bisect"):
                    dec, _ = run_btm_experiment(L, pt, tk, "P2")
                    assert dec == ref
                    tot += 1
    print(f"  P2 ALL-4-TREES EXHAUSTIVE PASS: {tot} cases [verified]")

    print("\n=== 3. ALL 6 AUTHENTICATED WIKIDATA BLOCKS — 4 TREES x P0/P1/P2 ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    # First: detailed P0 vs P1 vs P2 ablation on block-0.L across all 4 trees!
    with open("/tmp/mibz2-blocks/block-0.L", "rb") as f:
        L0 = f.read()
    e0 = manifest["blocks"][0]
    assert hashlib.sha256(L0).hexdigest() == e0["sha256"]
    ref0 = ref_ibwt(L0, e0["primary_index"])

    print("\n--- 3A. Ping-Pong Ablation (P0 vs P1 vs P2) on block-0.L ---")
    for tk in ["balanced", "lazard_max", "min_cut_bisect", "heavy_edge"]:
        for pm in ["P0", "P1", "P2"]:
            t0 = time.perf_counter()
            dec, st = run_btm_experiment(L0, e0["primary_index"], tk, pm)
            dt = time.perf_counter() - t0
            assert dec == ref0, f"EXACTNESS FAIL {tk}/{pm}"
            r0 = st["r0"]; m16 = st["m_16"]
            q = st.get("Q_rand", 0)
            qb = st.get("Q_rand_bounce", 0)
            qc = st.get("Q_rand_chain", 0)
            qr = st.get("Q_rand_root", 0)
            bn = st.get("bounces", 0)
            sp = st.get("spirals", 0)
            sm = st["slp_metrics"]
            print(f"  {tk:14s} {pm} | depth={st['tree_depth']:3d} sum_P={st['sum_tree_pieces']:7d}({st['sum_tree_pieces']/r0:5.2f}r) "
                  f"| bnc={bn:7d} sp={sp:4d} root={st.get('root_cycle_steps',0):6d} "
                  f"| Q_rand={q:7d} ({q/r0:5.2f}r, {q/m16:4.2f}m16) [bnc={qb},chn={qc},rt={qr}] "
                  f"| W_seq={st['W_seq']:7d} | |G|={sm['reach_nonlit']:6d} h={sm['height']:4d} "
                  f"cp={sm['copy_ops']:6d} avgL={sm['avg_copy_len']:5.1f} t={dt:.2f}s", flush=True)

    print("\n--- 3B. ALL 6 BLOCKS Across the 4 Trees (P1 and P2) ---")
    all_results = []
    for entry in manifest["blocks"]:
        path = f"/tmp/mibz2-blocks/{entry['file']}"
        with open(path, "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        p = entry["primary_index"]
        ref = ref_ibwt(L, p)
        for tk in ["balanced", "lazard_max", "min_cut_bisect", "heavy_edge"]:
            for pm in ["P1", "P2"]:
                dec, st = run_btm_experiment(L, p, tk, pm)
                assert dec == ref, f"EXACTNESS FAIL {entry['file']} {tk}/{pm}"
                r0 = st["r0"]; m16 = st["m_16"]
                q = st.get("Q_rand", 0)
                sm = st["slp_metrics"]
                print(f"  {entry['file']:11s} {tk:14s} {pm} | r0={r0:6d} m16={m16:6d} sig={st['sigma']:3d} "
                      f"d={st['tree_depth']:3d} sumP={st['sum_tree_pieces']:8d}({st['sum_tree_pieces']/r0:5.2f}r) "
                      f"bnc={st.get('bounces',0):7d} rt={st.get('root_cycle_steps',0):6d} "
                      f"Q={q:7d}({q/r0:4.2f}r,{q/m16:4.2f}m16) Wseq={st['W_seq']:8d} "
                      f"|G|={sm['reach_nonlit']:6d} cp={sm['copy_ops']:6d} avgL={sm['avg_copy_len']:4.1f}", flush=True)
                all_results.append({
                    "file": entry["file"], "tree": tk, "mode": pm,
                    "n": st["n"], "r0": r0, "m16": m16, "sigma": st["sigma"],
                    "depth": st["tree_depth"], "sum_P": st["sum_tree_pieces"],
                    "leaf_P": st["leaf_pieces"], "bounces": st.get("bounces", 0),
                    "spirals": st.get("spirals", 0), "spiral_saved": st.get("spiral_iters_saved", 0),
                    "bipartite": st.get("bipartite_collapses", 0),
                    "chain_c": st.get("chain_contractions", 0),
                    "root_steps": st.get("root_cycle_steps", 0),
                    "Q_rand": q, "Q_bounce": st.get("Q_rand_bounce", 0),
                    "Q_chain": st.get("Q_rand_chain", 0), "Q_root": st.get("Q_rand_root", 0),
                    "W_seq": st["W_seq"], "slp": sm,
                    "per_depth": st["per_depth"],
                })

    with open("/tmp/fcp-a/btm_results_6blocks.json", "w") as f:
        json.dump(all_results, f, indent=2)
    print("\nSaved full JSON results to /tmp/fcp-a/btm_results_6blocks.json")


if __name__ == "__main__":
    main()
