#!/usr/bin/env python3
"""
Investigator Beta — Cycle 2, Objective 1:
Escaping Referee #2's Low-r Padded-Ring Kill (r=39, 0.584n) AND MultiTag_M_runs(K) (r=17)
via Generalized Multi-Piece / Periodic Spiral Acceleration (AHT Widest-Self-Piece Schur Reduction)
inside IET-BSE!

Tests BOTH:
  - Pure AHT-Spiral inside T_self^* (Case 1 Euclidean Self-Overlap Spiral + Case 2 Widest-Piece 1-Schur)
  - Hybrid AHT-Spiral (AHT while max_self_width >= thresh, then endpoint RV when all self-pieces are thin)

Verifies 100% byte-exact parity on:
  1. Exhaustive binary n<=8 & ternary n<=5 BWTs + multi-cycle columns
  2. Referee #2's Low-r Padded-Ring (SymDB-Ring k=4 padded K=4,16,64,256,1024)
  3. MultiTag_M_runs(K) & MultiTag_MX(K) (K=4,16,64,256,1024)
  4. Fib_{2m}+$ (up to 2m=80, n=2.34e16) & Tribonacci+$ (up to n ~ 10^5)
  5. All 6 SHA-256 authenticated Wikidata blocks!
"""

import bisect
import json
import sys
import time
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-b")
import measure_iet_bse as mb


# ---------- Exact BWT encoder (O(n log n) suffix doubling) ----------

def bwt_encode_fast(T):
    """Exact forward BWT for byte/int sequence T (returns list L, int p)."""
    n = len(T)
    if n == 0:
        return [], 0
    s = list(T)
    sa = list(range(n))
    rk = list(s)
    k = 1
    while k < n:
        sa.sort(key=lambda i: (rk[i], rk[(i + k) % n]))
        tmp = [0] * n
        for j in range(1, n):
            a, b = sa[j - 1], sa[j]
            tmp[b] = tmp[a] + (1 if (rk[a], rk[(a + k) % n]) != (rk[b], rk[(b + k) % n]) else 0)
        rk = tmp
        if rk[sa[-1]] == n - 1:
            break
        k <<= 1
    L = [s[(i - 1) % n] for i in sa]
    p = sa.index(0)
    return L, p


# ---------- Adversarial Constant-r Families ----------

def de_bruijn_binary(k):
    """Standard binary de Bruijn sequence of order k (length 2^k)."""
    a = [0] * (2 * k)
    seq = []
    def db(t, p):
        if t > k:
            if k % p == 0:
                seq.extend(a[1:p + 1])
        else:
            a[t] = a[t - p]
            db(t + 1, p)
            for j in range(a[t - p] + 1, 2):
                a[t] = j
                db(t + 1, t)
    db(1, 1)
    return seq


def make_low_r_padded_ring(K, k=4):
    """
    Construct Referee #2's Low-r Padded-Ring family (|Sigma|=3):
    Symmetric de Bruijn ring of order k=4: DB(2, k) + complement/reverse + sentinel,
    with each symbol repeated K times (constant r ~ 33-39 independent of K!).
    Let's test BOTH standard padded DB(2,4)+sentinel AND SymDB-Ring (DB + flipped DB + 2-sentinel)!
    """
    db = de_bruijn_binary(k)
    # SymDB-Ring k=4: db + [1-x for x in reversed(db)] + [2] (or padded each char K times!)
    # Check r0 for a few canonical variants:
    # Variant A: db (len 16) + [2], each bit repeated K times -> n = 16*K + 1 (or +K)
    # Variant B: SymDB: db + [1-x for x in db] (len 32) + [2] -> each bit repeated K times!
    ring = []
    for bit in db:
        ring.extend([bit] * K)
    for bit in reversed(db):
        ring.extend([1 - bit] * K)
    ring.extend([2] * K)
    return ring


def make_padded_db_ring_v2(K, k=4):
    """Another Low-r Padded-Ring variant: DB(2,4) padded K times with alternating 2 delimiters -> |Sigma|=3."""
    db = de_bruijn_binary(k)
    ring = []
    for i, bit in enumerate(db):
        ring.extend([bit] * K)
        if i % 4 == 3:
            ring.extend([2] * K)
    return ring


def make_tribonacci(m):
    """Tribonacci word T_m + sentinel 3: 0 -> 01, 1 -> 02, 2 -> 0."""
    w = [0]
    for _ in range(m):
        nw = []
        for x in w:
            if x == 0: nw.extend([0, 1])
            elif x == 1: nw.extend([0, 2])
            else: nw.append(0)
        w = nw
    w.append(3)
    return w


# ---------- Generalized AHT-Spiral Solver for Open IET inside Bucket F_b ----------

def solve_bucket_aht_spiral(b, d_list, i_list, rules, p_state, aht_thresh=2, allow_split=True):
    """
    Resolve open IET on F_b with Generalized Multi-Piece / Periodic Spiral Acceleration (AHT):
      Phase 1 (AHT Widest-Self-Piece Loop, while max_self_len >= aht_thresh):
        Pick widest self-piece u = (dlo, ilo, L, wd, wl):
          - Case 1 (Self-overlap |ilo - dlo| < L):
              Closed-form O(1) Euclidean spiral contraction! (Zero other pieces touched!)
              Deletes L - |delta| rows in 1 step, replaces u by <= 2 pieces of total width |delta|.
          - Case 2 (Disjoint |ilo - dlo| >= L):
              1-Piece Schur Elimination of u: push all incoming pieces (self or entry) overlapping
              D_u = [dlo, dlo + L) forward by +(ilo - dlo) and delete u!
      Phase 2 (Thin-Self Finish, when all remaining self-pieces have len < aht_thresh, or k_self == 0):
        Run exact right-endpoint inner-RV on remaining pieces in F_b (pure 1D merge if k_self == 0!).

    Maintains sorted image intervals via bisect for O(log k + overlap_count) Case 2 lookup!
    p_state = [p_b, p_pos, p_shift, p_cycle] (mutated in-place).
    Returns (emitted_list, stats_dict).
    """
    # Separate pieces into:
    #   self_pcs : dict id -> [dlo, ilo, ln, wd, wl]
    #   exit_pcs : list of [dlo, ext_b, ext_lo, ln, wd, wl]
    #   entry_pcs: dict id -> [ext_b, ext_lo, ilo, ln, wd, wl]
    self_pcs = {}
    exit_pcs = []
    entry_pcs = {}
    next_id = 0

    for pc in d_list:
        db, ib, dlo, ilo, ln, wd, wl = pc
        if db == b and ib == b:
            self_pcs[next_id] = [dlo, ilo, ln, wd, wl]
            next_id += 1
        else:
            exit_pcs.append([dlo, ib, ilo, ln, wd, wl])

    for pc in i_list:
        db, ib, dlo, ilo, ln, wd, wl = pc
        if db != b:
            entry_pcs[next_id] = [db, dlo, ilo, ln, wd, wl]
            next_id += 1

    def make_cat(w1, w2):
        nid = len(rules)
        rules.append((1, w1, w2))
        return nid

    def make_pow(w1, k):
        if k == 1:
            return w1
        nid = len(rules)
        rules.append((2, w1, k))
        return nid

    aht_case1_steps = 0
    aht_case1_rows = 0
    aht_case2_steps = 0
    aht_case2_overlaps = 0
    aht_case2_splits = 0
    max_live_self = len(self_pcs)

    # Maintain sorted list of (ilo, id, is_self) for fast overlap queries in Case 2!
    # Wait: only build the sorted image index if we actually enter AHT!
    need_aht = any(v[2] >= aht_thresh for v in self_pcs.values()) if self_pcs else False

    if need_aht:
        # Sorted list of image starts: im_starts[idx] = ilo, im_meta[idx] = (pid, is_self)
        # Since all image intervals inside F_b are disjoint, sorting by ilo uniquely orders them!
        im_pairs = []
        for pid, v in self_pcs.items():
            im_pairs.append((v[1], pid, True))
        for pid, v in entry_pcs.items():
            im_pairs.append((v[2], pid, False))
        im_pairs.sort(key=lambda x: x[0])
        im_starts = [x[0] for x in im_pairs]
        im_meta = [(x[1], x[2]) for x in im_pairs]

        def im_remove(ilo_val):
            idx = bisect.bisect_left(im_starts, ilo_val)
            assert idx < len(im_starts) and im_starts[idx] == ilo_val
            im_starts.pop(idx)
            im_meta.pop(idx)

        def im_insert(ilo_val, pid, is_self):
            idx = bisect.bisect_left(im_starts, ilo_val)
            im_starts.insert(idx, ilo_val)
            im_meta.insert(idx, (pid, is_self))

        # Use a max-heap over self-pieces (lazy deletion) so picking widest is O(log k)!
        import heapq
        heap = [(-v[2], pid) for pid, v in self_pcs.items()]
        heapq.heapify(heap)

        while heap:
            neg_L, uid = heapq.heappop(heap)
            u = self_pcs.get(uid)
            if u is None or u[2] != -neg_L:
                continue
            L = u[2]
            if L < aht_thresh:
                break
            if len(self_pcs) > max_live_self:
                max_live_self = len(self_pcs)

            dlo, ilo, L, wd, wl = u
            delta = ilo - dlo

            if abs(delta) < L:
                # ===== CASE 1: CLOSED-FORM EUCLIDEAN SPIRAL CONTRACTION =====
                aht_case1_steps += 1
                del self_pcs[uid]
                im_remove(ilo)

                if delta == 0:
                    # Closed cycle component [dlo, dlo + L)
                    aht_case1_rows += L
                    if p_state[0] == b and p_state[3] is None and dlo <= p_state[1] < dlo + L:
                        p_state[3] = (wd, p_state[2], wl)
                        p_state[0] = -1
                    continue

                if delta > 0:
                    # Rightward spiral: delete [dlo + delta, dlo + L)
                    q = L // delta
                    rem = L % delta
                    aht_case1_rows += L - delta
                    if p_state[0] == b and p_state[3] is None and dlo + delta <= p_state[1] < dlo + L:
                        m = (p_state[1] - dlo) // delta
                        p_state[1] -= m * delta
                        p_state[2] += m * wl
                    # Piece 1 (rem > 0): [dlo, dlo+rem) -> [dlo+(q+1)*delta, dlo+L+delta), pow q+1
                    if rem > 0:
                        nid1 = next_id; next_id += 1
                        w1 = make_pow(wd, q + 1)
                        wl1 = (q + 1) * wl
                        ilo1 = dlo + (q + 1) * delta
                        self_pcs[nid1] = [dlo, ilo1, rem, w1, wl1]
                        im_insert(ilo1, nid1, True)
                        heapq.heappush(heap, (-rem, nid1))
                    # Piece 2: [dlo+rem, dlo+delta) -> [dlo+L, dlo+(q+1)*delta), pow q
                    ln2 = delta - rem
                    if ln2 > 0:
                        nid2 = next_id; next_id += 1
                        w2 = make_pow(wd, q)
                        wl2 = q * wl
                        dlo2 = dlo + rem
                        ilo2 = dlo + L
                        self_pcs[nid2] = [dlo2, ilo2, ln2, w2, wl2]
                        im_insert(ilo2, nid2, True)
                        heapq.heappush(heap, (-ln2, nid2))
                else:
                    # Leftward spiral: d = -delta > 0, delete [dlo, dlo + L - d)
                    d = -delta
                    q = L // d
                    rem = L % d
                    aht_case1_rows += L - d
                    if p_state[0] == b and p_state[3] is None and dlo <= p_state[1] < dlo + L - d:
                        m = (dlo + L - 1 - p_state[1]) // d
                        p_state[1] += m * d
                        p_state[2] += m * wl
                    # Piece 1: [dlo+L-d, dlo+L-rem) -> [dlo-d+rem, dlo), pow q
                    ln1 = d - rem
                    if ln1 > 0:
                        nid1 = next_id; next_id += 1
                        w1 = make_pow(wd, q)
                        wl1 = q * wl
                        dlo1 = dlo + L - d
                        ilo1 = dlo - d + rem
                        self_pcs[nid1] = [dlo1, ilo1, ln1, w1, wl1]
                        im_insert(ilo1, nid1, True)
                        heapq.heappush(heap, (-ln1, nid1))
                    # Piece 2 (rem > 0): [dlo+L-rem, dlo+L) -> [dlo-d, dlo-d+rem), pow q+1
                    if rem > 0:
                        nid2 = next_id; next_id += 1
                        w2 = make_pow(wd, q + 1)
                        wl2 = (q + 1) * wl
                        dlo2 = dlo + L - rem
                        ilo2 = dlo - d
                        self_pcs[nid2] = [dlo2, ilo2, rem, w2, wl2]
                        im_insert(ilo2, nid2, True)
                        heapq.heappush(heap, (-rem, nid2))

            else:
                # ===== CASE 2: 1-PIECE SCHUR ELIMINATION OF DISJOINT WIDEST SELF-PIECE =====
                if not allow_split:
                    break
                aht_case2_steps += 1
                del self_pcs[uid]
                im_remove(ilo)

                # Find all image intervals overlapping [dlo, dlo + L)
                dhi = dlo + L
                idx = bisect.bisect_right(im_starts, dlo) - 1
                if idx < 0:
                    idx = 0

                overlaps = []
                while idx < len(im_starts) and im_starts[idx] < dhi:
                    v_ilo = im_starts[idx]
                    vid, is_self = im_meta[idx]
                    v_ln = self_pcs[vid][2] if is_self else entry_pcs[vid][3]
                    if v_ilo + v_ln > dlo:
                        overlaps.append((v_ilo, vid, is_self))
                    idx += 1

                aht_case2_overlaps += len(overlaps)

                for v_ilo, vid, is_self in overlaps:
                    im_remove(v_ilo)
                    if is_self:
                        vdlo, vilo, vln, vwd, vwl = self_pcs.pop(vid)
                        ov_lo = dlo if dlo > vilo else vilo
                        ov_hi = dhi if dhi < vilo + vln else vilo + vln
                        left_ln = ov_lo - vilo
                        mid_ln = ov_hi - ov_lo
                        right_ln = (vilo + vln) - ov_hi

                        if left_ln > 0:
                            aht_case2_splits += 1
                            lid = next_id; next_id += 1
                            self_pcs[lid] = [vdlo, vilo, left_ln, vwd, vwl]
                            im_insert(vilo, lid, True)
                            heapq.heappush(heap, (-left_ln, lid))

                        # Middle (overlapping D_u): compose v_mid -> u!
                        mid_dlo = vdlo + left_ln
                        mid_ilo = ilo + (ov_lo - dlo)
                        if p_state[0] == b and p_state[3] is None and ov_lo <= p_state[1] < ov_hi:
                            p_state[1] = mid_dlo + (p_state[1] - ov_lo)
                            p_state[2] += vwl
                        mid_wd = make_cat(vwd, wd)
                        mid_wl = vwl + wl
                        mid_id = next_id; next_id += 1
                        self_pcs[mid_id] = [mid_dlo, mid_ilo, mid_ln, mid_wd, mid_wl]
                        im_insert(mid_ilo, mid_id, True)
                        heapq.heappush(heap, (-mid_ln, mid_id))

                        if right_ln > 0:
                            aht_case2_splits += 1
                            rid = next_id; next_id += 1
                            r_dlo = vdlo + left_ln + mid_ln
                            r_ilo = ov_hi
                            self_pcs[rid] = [r_dlo, r_ilo, right_ln, vwd, vwl]
                            im_insert(r_ilo, rid, True)
                            heapq.heappush(heap, (-right_ln, rid))
                    else:
                        # Entry piece overlapping D_u
                        eb, elo, vilo, vln, vwd, vwl = entry_pcs.pop(vid)
                        ov_lo = dlo if dlo > vilo else vilo
                        ov_hi = dhi if dhi < vilo + vln else vilo + vln
                        left_ln = ov_lo - vilo
                        mid_ln = ov_hi - ov_lo
                        right_ln = (vilo + vln) - ov_hi

                        if left_ln > 0:
                            aht_case2_splits += 1
                            lid = next_id; next_id += 1
                            entry_pcs[lid] = [eb, elo, vilo, left_ln, vwd, vwl]
                            im_insert(vilo, lid, False)

                        mid_elo = elo + left_ln
                        mid_ilo = ilo + (ov_lo - dlo)
                        if p_state[0] == b and p_state[3] is None and ov_lo <= p_state[1] < ov_hi:
                            # Marker exits F_b to eb!
                            p_state[0] = eb
                            p_state[1] = mid_elo + (p_state[1] - ov_lo)
                            p_state[2] += vwl
                        mid_wd = make_cat(vwd, wd)
                        mid_wl = vwl + wl
                        mid_id = next_id; next_id += 1
                        entry_pcs[mid_id] = [eb, mid_elo, mid_ilo, mid_ln, mid_wd, mid_wl]
                        im_insert(mid_ilo, mid_id, False)

                        if right_ln > 0:
                            aht_case2_splits += 1
                            rid = next_id; next_id += 1
                            entry_pcs[rid] = [eb, elo + left_ln + mid_ln, ov_hi, right_ln, vwd, vwl]
                            im_insert(ov_hi, rid, False)

    # ===== PHASE 2: FAST FINISH (1D merge if k_self==0, or thin endpoint RV if k_self>0) =====
    emitted = []
    phase2_merge_steps = 0
    phase2_self_steps = 0

    if not self_pcs:
        # Pure 1D sequential two-pointer merge of sorted exits & sorted entries!
        if exit_pcs:
            exit_pcs.sort(key=lambda x: x[0])
            ent_list = sorted(entry_pcs.values(), key=lambda x: x[2])
            ei = 0
            xi = 0
            e_off = 0
            x_off = 0
            ne = len(ent_list)
            nx = len(exit_pcs)
            while xi < nx and ei < ne:
                ex = exit_pcs[xi]
                en = ent_list[ei]
                # Coordinates match! (Both tile the exact same surviving subset S_b!)
                rem_x = ex[3] - x_off
                rem_e = en[3] - e_off
                take = rem_x if rem_x < rem_e else rem_e
                phase2_merge_steps += 1
                src_b = en[0]
                src_lo = en[1] + e_off
                dst_b = ex[1]
                dst_lo = ex[2] + x_off
                cw = make_cat(en[4], ex[4])
                cwl = en[5] + ex[5]
                emitted.append((src_b, dst_b, src_lo, dst_lo, take, cw, cwl))
                if p_state[0] == b and p_state[3] is None:
                    x_pos = ex[0] + x_off
                    if x_pos <= p_state[1] < x_pos + take:
                        p_state[0] = src_b
                        p_state[1] = src_lo + (p_state[1] - x_pos)
                        p_state[2] += en[5]
                x_off += take
                e_off += take
                if x_off == ex[3]:
                    xi += 1; x_off = 0
                if e_off == en[3]:
                    ei += 1; e_off = 0
            assert xi == nx and ei == ne
    else:
        # Thin self-pieces remain (all len < aht_thresh): run exact right-endpoint inner-RV!
        # Build unified dom_list and im_list sorted by coordinate (note S_b may have holes deleted by AHT,
        # which is 100% valid because dom and im tile the EXACT SAME surviving set S_b!)
        nodes_kind = []
        nodes_len = []
        nodes_word = []
        nodes_wlen = []
        nodes_ext_b = []
        nodes_ext_lo = []
        nodes_dlo = []

        dom_pairs = []
        im_pairs = []

        for pid, (dlo, ilo, ln, wd, wl) in self_pcs.items():
            nid = len(nodes_kind)
            nodes_kind.append(0); nodes_len.append(ln); nodes_word.append(wd); nodes_wlen.append(wl)
            nodes_ext_b.append(b); nodes_ext_lo.append(-1); nodes_dlo.append(dlo)
            dom_pairs.append((dlo, nid))
            im_pairs.append((ilo, nid))
        for dlo, eb, elo, ln, wd, wl in exit_pcs:
            nid = len(nodes_kind)
            nodes_kind.append(1); nodes_len.append(ln); nodes_word.append(wd); nodes_wlen.append(wl)
            nodes_ext_b.append(eb); nodes_ext_lo.append(elo); nodes_dlo.append(dlo)
            dom_pairs.append((dlo, nid))
        for pid, (eb, elo, ilo, ln, wd, wl) in entry_pcs.items():
            nid = len(nodes_kind)
            nodes_kind.append(2); nodes_len.append(ln); nodes_word.append(wd); nodes_wlen.append(wl)
            nodes_ext_b.append(eb); nodes_ext_lo.append(elo); nodes_dlo.append(-1)
            im_pairs.append((ilo, nid))

        dom_pairs.sort(key=lambda x: x[0])
        im_pairs.sort(key=lambda x: x[0])
        dom_ids = [x[1] for x in dom_pairs]
        im_ids = [x[1] for x in im_pairs]

        num_nodes = len(nodes_kind)
        dp = [-1] * num_nodes; dn = [-1] * num_nodes
        ip = [-1] * num_nodes; inn = [-1] * num_nodes
        for idx, u in enumerate(dom_ids):
            dp[u] = dom_ids[idx - 1] if idx > 0 else -1
            dn[u] = dom_ids[idx + 1] if idx + 1 < len(dom_ids) else -1
        d_tail = dom_ids[-1] if dom_ids else -1
        for idx, u in enumerate(im_ids):
            ip[u] = im_ids[idx - 1] if idx > 0 else -1
            inn[u] = im_ids[idx + 1] if idx + 1 < len(im_ids) else -1
        i_tail = im_ids[-1] if im_ids else -1

        def d_unlink(u):
            nonlocal d_tail
            pr, nx = dp[u], dn[u]
            if pr != -1: dn[pr] = nx
            if nx != -1: dp[nx] = pr
            else: d_tail = pr
            dp[u] = dn[u] = -1

        def i_unlink(u):
            nonlocal i_tail
            pr, nx = ip[u], inn[u]
            if pr != -1: inn[pr] = nx
            if nx != -1: ip[nx] = pr
            else: i_tail = pr
            ip[u] = inn[u] = -1

        def d_ins_after(anc, u):
            nonlocal d_tail
            nx = dn[anc]
            dp[u] = anc; dn[u] = nx; dn[anc] = u
            if nx != -1: dp[nx] = u
            else: d_tail = u

        def i_ins_after(anc, u):
            nonlocal i_tail
            nx = inn[anc]
            ip[u] = anc; inn[u] = nx; inn[anc] = u
            if nx != -1: ip[nx] = u
            else: i_tail = u

        p_in_b = (p_state[0] == b and p_state[3] is None)
        p_nid = -1
        p_off = 0
        if p_in_b:
            for dlo_u, u in dom_pairs:
                if dlo_u <= p_state[1] < dlo_u + nodes_len[u]:
                    p_nid = u
                    p_off = p_state[1] - dlo_u
                    break
            assert p_nid != -1

        while d_tail != -1:
            d = d_tail
            i = i_tail
            if d == i:
                phase2_self_steps += 1
                if p_in_b and p_nid == d:
                    p_state[3] = (nodes_word[d], p_state[2], nodes_wlen[d])
                    p_state[0] = -1
                    p_in_b = False; p_nid = -1
                d_unlink(d); i_unlink(d)
                continue
            kd, ki = nodes_kind[d], nodes_kind[i]
            ld, li = nodes_len[d], nodes_len[i]
            if kd == 1 and ki == 2:
                phase2_merge_steps += 1
                take = ld if ld < li else li
                src_b = nodes_ext_b[i]; dst_b = nodes_ext_b[d]
                src_lo = nodes_ext_lo[i] + (li - take)
                dst_lo = nodes_ext_lo[d] + (ld - take)
                cw = make_cat(nodes_word[i], nodes_word[d])
                cwl = nodes_wlen[i] + nodes_wlen[d]
                emitted.append((src_b, dst_b, src_lo, dst_lo, take, cw, cwl))
                if p_in_b and p_nid == d and p_off >= ld - take:
                    p_state[0] = src_b
                    p_state[1] = src_lo + (p_off - (ld - take))
                    p_state[2] += nodes_wlen[i]
                    p_in_b = False; p_nid = -1
                if ld == take: d_unlink(d)
                else: nodes_len[d] = ld - take
                if li == take: i_unlink(i)
                else: nodes_len[i] = li - take
            elif ld > li:
                phase2_self_steps += 1
                if kd == 0:
                    if p_in_b and p_nid == d and p_off >= ld - li:
                        if ki == 2:
                            p_state[0] = nodes_ext_b[i]
                            p_state[1] = nodes_ext_lo[i] + (p_off - (ld - li))
                            p_state[2] += nodes_wlen[i]
                            p_in_b = False; p_nid = -1
                        else:
                            p_nid = i; p_off -= (ld - li); p_state[2] += nodes_wlen[i]
                    nodes_word[i] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[i] += nodes_wlen[d]
                    nodes_len[d] -= li
                    if inn[d] != i:
                        i_unlink(i); i_ins_after(d, i)
                else:
                    if p_in_b and p_nid == d and p_off >= ld - li:
                        p_nid = i; p_off -= (ld - li); p_state[2] += nodes_wlen[i]
                    nodes_word[i] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[i] += nodes_wlen[d]
                    nodes_kind[i] = 1
                    nodes_ext_b[i] = nodes_ext_b[d]
                    nodes_ext_lo[i] = nodes_ext_lo[d] + (ld - li)
                    nodes_len[d] -= li
                    i_unlink(i)
            elif li > ld:
                phase2_self_steps += 1
                if ki == 0:
                    if p_in_b:
                        if p_nid == d: p_state[2] += nodes_wlen[i]
                        elif p_nid == i and p_off >= li - ld: p_nid = d; p_off -= (li - ld)
                    nodes_word[d] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[d] += nodes_wlen[i]
                    nodes_len[i] -= ld
                    if dn[i] != d:
                        d_unlink(d); d_ins_after(i, d)
                else:
                    src_lo = nodes_ext_lo[i] + (li - ld)
                    if p_in_b and p_nid == d:
                        p_state[0] = nodes_ext_b[i]
                        p_state[1] = src_lo + p_off
                        p_state[2] += nodes_wlen[i]
                        p_in_b = False; p_nid = -1
                    nodes_word[d] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[d] += nodes_wlen[i]
                    nodes_kind[d] = 2
                    nodes_ext_b[d] = nodes_ext_b[i]
                    nodes_ext_lo[d] = src_lo
                    nodes_len[i] -= ld
                    d_unlink(d)
            else:
                phase2_self_steps += 1
                if kd == 0 and ki == 0:
                    if p_in_b and p_nid == d:
                        p_nid = i; p_state[2] += nodes_wlen[i]
                    nodes_word[i] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[i] += nodes_wlen[d]
                    if inn[d] == i: i_unlink(d)
                    else: i_unlink(i); i_ins_after(d, i); i_unlink(d)
                    d_unlink(d)
                elif kd == 1 and ki == 0:
                    if p_in_b and p_nid == d:
                        p_nid = i; p_state[2] += nodes_wlen[i]
                    nodes_word[i] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[i] += nodes_wlen[d]
                    nodes_kind[i] = 1
                    nodes_ext_b[i] = nodes_ext_b[d]
                    nodes_ext_lo[i] = nodes_ext_lo[d]
                    i_unlink(i); d_unlink(d)
                elif kd == 0 and ki == 2:
                    if p_in_b and p_nid == d:
                        p_state[0] = nodes_ext_b[i]
                        p_state[1] = nodes_ext_lo[i] + p_off
                        p_state[2] += nodes_wlen[i]
                        p_in_b = False; p_nid = -1
                    nodes_word[d] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[d] += nodes_wlen[i]
                    nodes_kind[d] = 2
                    nodes_ext_b[d] = nodes_ext_b[i]
                    nodes_ext_lo[d] = nodes_ext_lo[i]
                    d_unlink(d); i_unlink(i)

    return emitted, {
        "c1_steps": aht_case1_steps,
        "c1_rows": aht_case1_rows,
        "c2_steps": aht_case2_steps,
        "c2_overlaps": aht_case2_overlaps,
        "c2_splits": aht_case2_splits,
        "p2_merge": phase2_merge_steps,
        "p2_self": phase2_self_steps,
        "max_live_self": max_live_self,
    }


def run_iet_bse_aht(runs, p, order_mode="rarest_first_deg", aht_thresh=2, verify_seq=None):
    """Full IET-BSE with Generalized AHT-Spiral inside EVERY bucket (including the final bucket!)."""
    r0 = len(runs)
    n = sum(w for _, w in runs)
    max_sym = max(c for c, _ in runs)
    counts = [0] * (max_sym + 1)
    for c, w in runs:
        counts[c] += w
    C = [0] * (max_sym + 2)
    for c in range(max_sym + 1):
        C[c + 1] = C[c] + counts[c]
    active_syms = [c for c in range(max_sym + 1) if counts[c] > 0]

    rules = [(0, c) for c in range(max_sym + 1)]
    f_cur = list(C[:max_sym + 1])
    dom_buf = {b: [] for b in active_syms}
    im_buf = {b: [] for b in active_syms}

    pos = 0
    im_b = 0
    while im_b <= max_sym and C[im_b + 1] == 0:
        im_b += 1
    q_init = 0
    for c, w in runs:
        rem = w; cur_im = pos; cur_dom = f_cur[c]
        while rem > 0:
            while C[im_b + 1] <= cur_im:
                im_b += 1
            take = min(rem, C[im_b + 1] - cur_im)
            pc = (c, im_b, cur_dom, cur_im, take, c, 1)
            dom_buf[c].append(pc)
            im_buf[im_b].append(pc)
            q_init += 1
            cur_im += take; cur_dom += take; rem -= take
        f_cur[c] += w
        pos += w

    live_dom = {b: len(dom_buf[b]) for b in active_syms}
    live_im = {b: len(im_buf[b]) for b in active_syms}

    static_order = None
    if order_mode == "most_freq_first":
        static_order = sorted(active_syms, key=lambda b: (-counts[b], b))
    elif order_mode == "rarest_first_rows":
        static_order = sorted(active_syms, key=lambda b: (counts[b], b))
    elif order_mode == "max_sym":
        static_order = sorted(active_syms, reverse=True)

    eliminated = [False] * (max_sym + 1)
    alive = set(active_syms)

    p_b = 0
    while C[p_b + 1] <= p:
        p_b += 1
    p_state = [p_b, p, 0, None]  # [p_b, p_pos, p_shift, p_cycle]

    tot = defaultdict(int)
    max_live_self_all = 0
    q_curr = q_init
    q_violations = 0
    order_idx = 0

    while alive:
        if len(alive) == 1:
            b = next(iter(alive))
        elif static_order is not None:
            b = static_order[order_idx]; order_idx += 1
        else:
            b = min(alive, key=lambda x: (live_dom[x] + live_im[x], counts[x], x))

        d_list = [pc for pc in dom_buf[b] if not eliminated[pc[1]]]
        i_list = [pc for pc in im_buf[b] if not eliminated[pc[0]]]
        emitted, st = solve_bucket_aht_spiral(b, d_list, i_list, rules, p_state, aht_thresh=aht_thresh)

        for k, v in st.items():
            if k != "max_live_self":
                tot[k] += v
        if st["max_live_self"] > max_live_self_all:
            max_live_self_all = st["max_live_self"]

        eliminated[b] = True
        alive.remove(b)
        for pc in i_list:
            if pc[0] != b: live_dom[pc[0]] -= 1
        for pc in d_list:
            if pc[1] != b: live_im[pc[1]] -= 1
        for ep in emitted:
            sb, db = ep[0], ep[1]
            dom_buf[sb].append(ep); im_buf[db].append(ep)
            live_dom[sb] += 1; live_im[db] += 1
        del dom_buf[b]; del im_buf[b]

        if alive:
            k_self = sum(1 for pc in d_list if pc[1] == b)
            old_q = q_curr
            q_curr = q_curr - (len(d_list) + len(i_list) - k_self) + len(emitted)
            if q_curr > old_q - 1:
                q_violations += 1

    assert p_state[3] is not None
    root_wd, sh, wl = p_state[3]

    if verify_seq is not None:
        # Expand SLP
        out = []
        stack = [root_wd]
        while stack:
            u = stack.pop()
            r = rules[u]
            if r[0] == 0: out.append(r[1])
            elif r[0] == 1: stack.append(r[2]); stack.append(r[1])
            else:
                for _ in range(r[2]): stack.append(r[1])
        assert len(out) == wl
        s = sh % wl
        rot = out[s:] + out[:s]
        if len(verify_seq) % len(rot) == 0:
            full = rot * (len(verify_seq) // len(rot))
        else:
            full = rot
        assert full == list(verify_seq), f"MISMATCH! len(got)={len(full)}, len(want)={len(verify_seq)}"

    total_ops = tot["c1_steps"] + tot["c2_steps"] + tot["p2_merge"] + tot["p2_self"]
    return {
        "n": n, "r0": r0, "q_viol": q_violations,
        "c1_steps": tot["c1_steps"], "c1_rows": tot["c1_rows"],
        "c2_steps": tot["c2_steps"], "c2_overlaps": tot["c2_overlaps"], "c2_splits": tot["c2_splits"],
        "p2_merge": tot["p2_merge"], "p2_self": tot["p2_self"],
        "total_steps": total_ops,
        "max_live_self": max_live_self_all,
        "slp_nodes": len(rules) - (max_sym + 1),
    }


def main():
    print("=== 1. EXHAUSTIVE SMALL VERIFICATION (Binary n<=8, Ternary n<=5, Multi-cycle) ===")
    cnt = 0
    for n in range(1, 9):
        for mask in range(1 << n):
            T = [(mask >> i) & 1 for i in range(n)]
            L, p = bwt_encode_fast(T)
            runs = mb.rle_from_bytes(L)
            for th in (1, 2):
                st = run_iet_bse_aht(runs, p, order_mode="rarest_first_deg", aht_thresh=th, verify_seq=T)
                assert st["q_viol"] == 0
            cnt += 1
    for n in range(1, 6):
        for idx in range(3 ** n):
            T = []; v = idx
            for _ in range(n): T.append(v % 3); v //= 3
            L, p = bwt_encode_fast(T)
            runs = mb.rle_from_bytes(L)
            for th in (1, 2):
                st = run_iet_bse_aht(runs, p, order_mode="most_freq_first", aht_thresh=th, verify_seq=T)
                assert st["q_viol"] == 0
            cnt += 1
    print(f"  PASS: {cnt} exhaustive BWTs x 2 thresholds (0 failures, 0 q_viol)!")

    print("\n=== 2. CONFRONTING ADVERSARIAL CONSTANT-r FAMILIES (Cycle 1 IET-BSE vs Cycle 2 IET-BSE-AHT) ===")
    for name, gen in [
        ("Low-r Padded-Ring (SymDB k=4)", lambda K: make_low_r_padded_ring(K, k=4)),
        ("Padded-DB-Ring v2 (DB k=4)", lambda K: make_padded_db_ring_v2(K, k=4)),
        ("MultiTag_M_runs (8 tags)", lambda K: list(b"".join(bytes([65+i]) + b"M"*K + bytes([80+i]) for i in range(8)))),
        ("MultiTag_MX (8 tags)", lambda K: list(b"".join(bytes([65+i]) + b"MX"*K + bytes([80+i]) for i in range(8)))),
    ]:
        print(f"\n  [{name}]")
        for K in [4, 16, 64, 256]:
            T = gen(K)
            L, p = bwt_encode_fast(T)
            runs = mb.rle_from_bytes(L)
            r0 = len(runs)
            # Cycle 1 baseline (rarest_first_deg & most_freq_first)
            c1_rd = mb.run_iet_bse_instrumented(runs, p, order_mode="rarest_first_deg", verify_bytes=None)
            c1_mf = mb.run_iet_bse_instrumented(runs, p, order_mode="most_freq_first", verify_bytes=None)
            # Cycle 2 AHT-Spiral (thresh=2)
            aht_rd = run_iet_bse_aht(runs, p, order_mode="rarest_first_deg", aht_thresh=2, verify_seq=T)
            aht_mf = run_iet_bse_aht(runs, p, order_mode="most_freq_first", aht_thresh=2, verify_seq=T)
            aht_ms = run_iet_bse_aht(runs, p, order_mode="max_sym", aht_thresh=2, verify_seq=T)
            print(f"    K={K:3d} n={len(T):5d} r0={r0:2d} | "
                  f"C1(r_deg)={c1_rd['total_all_steps']:5d} ({c1_rd['total_all_steps']/len(T):.3f}n) "
                  f"C1(m_frq)={c1_mf['total_all_steps']:5d} | "
                  f"AHT(r_deg)={aht_rd['total_steps']:3d} (c1={aht_rd['c1_steps']},c2={aht_rd['c2_steps']},ov={aht_rd['c2_overlaps']}) "
                  f"AHT(m_frq)={aht_mf['total_steps']:3d} AHT(max_s)={aht_ms['total_steps']:3d} [VERIFIED]")

    print("\n=== 3. FIBONACCI (up to 2m=80, n=2.34e16) & TRIBONACCI+$ ===")
    fibs = [0, 1]
    for _ in range(85): fibs.append(fibs[-1] + fibs[-2])
    for two_m in [12, 20, 40, 80]:
        f2, f1 = fibs[two_m - 2], fibs[two_m - 1]
        runs = [(1, f2 - 1), (2, 1), (0, f1), (1, 1)]
        p = f2 - 1
        st = run_iet_bse_aht(runs, p, order_mode="rarest_first_deg", aht_thresh=2, verify_seq=None)
        print(f"  Fib_{two_m:2d}+$ (n={f2+f1:18d}, r0=4): AHT steps={st['total_steps']:3d} (c1={st['c1_steps']}, c2={st['c2_steps']}, p2={st['p2_merge']+st['p2_self']}, max_self={st['max_live_self']})")

    for m in [8, 12, 16]:
        T = make_tribonacci(m)
        L, p = bwt_encode_fast(T)
        runs = mb.rle_from_bytes(L)
        st = run_iet_bse_aht(runs, p, order_mode="rarest_first_deg", aht_thresh=2, verify_seq=T)
        c1 = mb.run_iet_bse_instrumented(runs, p, order_mode="rarest_first_deg", verify_bytes=None)
        print(f"  Trib_{m:2d}+$ (n={len(T):6d}, r0={len(runs):2d}): C1 steps={c1['total_all_steps']:4d} | AHT steps={st['total_steps']:3d} (c1={st['c1_steps']}, c2={st['c2_steps']}, ov={st['c2_overlaps']}) [VERIFIED]")

    print("\n=== 4. ALL 6 SHA-256 AUTHENTICATED WIKIDATA BLOCKS (AHT thresh=2 vs thresh=16 vs Cycle 1) ===")
    blocks = mb.load_authenticated_blocks()
    for binfo, raw in blocks:
        fname = binfo["file"]
        runs = mb.rle_from_bytes(raw)
        r0 = len(runs)
        _, m16, _ = mb.compute_open_m16(runs, binfo["primary_index"], limit=16)
        ref = mb.decode_lf_reference_bytes(raw, binfo["primary_index"])
        for th in (2, 16):
            t0 = time.perf_counter()
            st = run_iet_bse_aht(runs, binfo["primary_index"], order_mode="rarest_first_deg", aht_thresh=th, verify_seq=ref)
            dt = time.perf_counter() - t0
            print(f"  {fname:12s} [AHT th={th:2d}] VERIFIED ({dt:.2f}s): tot_steps={st['total_steps']:6d} ({st['total_steps']/r0:.2f}r, {st['total_steps']/m16:.2f}*m16) "
                  f"| c1={st['c1_steps']:4d} (rows={st['c1_rows']:5d}) c2={st['c2_steps']:5d} (ov={st['c2_overlaps']:5d}, spl={st['c2_splits']:5d}) "
                  f"p2_mrg={st['p2_merge']:6d} p2_slf={st['p2_self']:5d} | |G|={st['slp_nodes']:6d}")


if __name__ == "__main__":
    main()
