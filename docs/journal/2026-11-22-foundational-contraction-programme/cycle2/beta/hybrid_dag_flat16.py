#!/usr/bin/env python3
"""
Investigator Beta — Cycle 2, Objective 3:
Attacking IET-BSE's C_contraction Overhead via:
  1. Zero-Sort / Zero-Stale One-Pass Independent-Set + DAG-Batching of zero-self buckets
  2. Direct u128 Flat<=16 Inline Operad inside pieces (measuring exact fraction of compositions
     with wlen <= 16 requiring ZERO SLP allocation!)
  3. Early-Stop Hub Fusion at the single hub b='"' (comparing Full IET-BSE-AHT + Flat<=16 Memo-SLP
     vs Early-Stop at b='"' + Thick-AHT(th=2,4,8,16) + Core Walk of m_core rows!)
  4. Exact Cost Vector (W, D, B, Q, M) & Complete Economic Inequality against the ACTUAL i9-10900K bar:
     Open_construct = 0.72-0.90 ns/row (648-810 us/900k blk)
     Open_emit      = 4.8-5.8 ns/m_16 row (pooled 5.4-6.5 ns/m_16 row)

Verified SHA-256 + 100% byte-exact parity across ALL 6 authenticated Wikidata blocks!
"""

import bisect
import heapq
import json
import sys
import time
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-b")
import measure_iet_bse as mb
import escape_padded_ring as epr


# ---------- Independent Set of Zero-Self Buckets at t=0 ----------

def find_zero_self_indep_set(active_syms, dom_buf):
    """
    Greedy maximal independent set U subset active_syms at t=0 such that:
      - Every b in U has k_self(b) == 0 (zero self-loops)
      - No strip connects u -> v for u, v in U (so ALL b in U can be eliminated
        simultaneously in ONE pass with ZERO sorting, ZERO stale reads, ZERO self-RV!).
    Sort candidates by smallest degree (k_dom + k_im) or largest zero-self piece savings.
    """
    adj = {b: set() for b in active_syms}
    has_self = set()
    deg = {b: 0 for b in active_syms}
    for b in active_syms:
        for pc in dom_buf[b]:
            dst = pc[1]
            deg[b] += 1
            deg[dst] += 1
            if dst == b:
                has_self.add(b)
            else:
                adj[b].add(dst)
                adj[dst].add(b)

    cands = [b for b in active_syms if b not in has_self]
    # Greedy by fewest graph neighbors (maximizes independent set size)
    cands.sort(key=lambda b: (len(adj[b]), deg[b], b))
    indep = []
    blocked = set()
    for b in cands:
        if b not in blocked:
            indep.append(b)
            blocked.add(b)
            blocked.update(adj[b])
    return indep


# ---------- Full Instrumented u128 Flat<=16 + DAG-Batch + Hub-Fusion Engine ----------

def run_hybrid_iet_bse(runs, p, inline_cap=16, stop_at_hub=False, hub_aht_thresh=16, verify_bytes=None):
    """
    Runs IET-BSE with:
      - Stage 0: Build initial sorted bi-monotone strips (every bucket's dom_buf & im_buf is
        ALREADY sorted at t=0!).
      - Stage 1: One-pass zero-sort, zero-stale elimination of maximal independent set U_0
        of zero-self buckets! (Pure 1D sequential two-pointer merges!)
      - Stage 2: Dynamic min-degree (rarest_first_deg) IET-BSE-AHT(th=2) on remaining buckets
        until 1 hub bucket remains (b_hub = '"' on Wikidata!).
        Tracks u128 inline operad: whenever wlen_a + wlen_b <= inline_cap, NO SLP node is created!
      - Stage 3:
        * If stop_at_hub == False: solve final hub with AHT-Spiral(th=2) -> Flat<=16 Memo-SLP expansion.
        * If stop_at_hub == True:  apply AHT-Spiral(th=hub_aht_thresh) on thick self-pieces of b_hub,
          leaving m_core thin rows (all remaining self-pieces have len < hub_aht_thresh), then walk
          the m_core cycle directly (expanding each step's inline word / memoized SLP word)!
    """
    r0 = len(runs)
    n = sum(w for _, w in runs)
    counts = [0] * 256
    for c, w in runs:
        counts[c] += w
    C = [0] * 257
    for c in range(256):
        C[c + 1] = C[c] + counts[c]
    active_syms = [c for c in range(256) if counts[c] > 0]

    # Representation of a word:
    #   If wlen <= inline_cap: stored inline as Python bytes object (represents u128 register!), NO SLP node!
    #   If wlen > inline_cap:  stored as integer rule_id >= 0 indexing `rules`!
    #   Rule entry in `rules`:
    #     (0, bytes_obj)           -> materialized flat leaf (only created when an inline word joins into a >16B cat!)
    #     (1, left_id, right_id)   -> binary cat (> inline_cap)
    #     (2, base_id, k)          -> power
    rules = []
    inline_to_id = {}  # deduplicate small materialized flat leaves? Track BOTH raw & dedup!
    u128_inline_merges = 0
    slp_cat_allocs = 0
    slp_pow_allocs = 0
    flat_leaf_materializations = 0

    def ensure_rule_id(wd):
        nonlocal flat_leaf_materializations
        if isinstance(wd, int):
            return wd
        # Materialize inline bytes into flat-leaf pool (no dedup to reflect zero-hash-table fast path!)
        nid = len(rules)
        rules.append((0, wd))
        flat_leaf_materializations += 1
        return nid

    def compose_word(w1, l1, w2, l2):
        nonlocal u128_inline_merges, slp_cat_allocs
        tot_l = l1 + l2
        if tot_l <= inline_cap:
            u128_inline_merges += 1
            return w1 + w2, tot_l  # pure u128 shift+OR, ZERO SLP node!
        slp_cat_allocs += 1
        id1 = ensure_rule_id(w1)
        id2 = ensure_rule_id(w2)
        nid = len(rules)
        rules.append((1, id1, id2))
        return nid, tot_l

    def pow_word(w1, l1, k):
        nonlocal u128_inline_merges, slp_pow_allocs
        if k == 1:
            return w1, l1
        tot_l = l1 * k
        if tot_l <= inline_cap:
            u128_inline_merges += 1
            return w1 * k, tot_l
        slp_pow_allocs += 1
        id1 = ensure_rule_id(w1)
        nid = len(rules)
        rules.append((2, id1, k))
        return nid, tot_l

    # Build initial sorted bi-monotone strips
    f_cur = list(C[:256])
    dom_buf = {b: [] for b in active_syms}
    im_buf = {b: [] for b in active_syms}
    pos = 0
    im_b = 0
    while im_b < 256 and C[im_b + 1] == 0:
        im_b += 1
    q_init = 0
    for c, w in runs:
        rem = w; cur_im = pos; cur_dom = f_cur[c]
        b_lit = bytes([c])
        while rem > 0:
            while C[im_b + 1] <= cur_im:
                im_b += 1
            take = min(rem, C[im_b + 1] - cur_im)
            pc = (c, im_b, cur_dom, cur_im, take, b_lit, 1)
            dom_buf[c].append(pc)
            im_buf[im_b].append(pc)
            q_init += 1
            cur_im += take; cur_dom += take; rem -= take
        f_cur[c] += w
        pos += w

    live_dom = {b: len(dom_buf[b]) for b in active_syms}
    live_im = {b: len(im_buf[b]) for b in active_syms}
    eliminated = [False] * 256
    alive = set(active_syms)

    # Track marker p
    p_b = 0
    while C[p_b + 1] <= p:
        p_b += 1
    p_state = [p_b, p, 0, None]  # [p_b, p_pos, p_shift, p_cycle]

    # Metrics
    s1_buckets = 0
    s1_rows = 0
    s1_merges = 0
    s1_reads = 0  # 100% live, 0% stale, 0 sort!

    s2_buf_reads_live = 0
    s2_buf_reads_stale = 0
    s2_sort_items = 0
    s2_emitted = 0
    s2_c1 = 0; s2_c2 = 0; s2_c2_ov = 0; s2_p2_mrg = 0; s2_p2_slf = 0

    # ---- STAGE 1: Zero-Sort, Zero-Stale, Zero-Self Independent Set Batch ----
    indep_U0 = find_zero_self_indep_set(active_syms, dom_buf)
    s1_buckets = len(indep_U0)
    for b in indep_U0:
        # ALREADY 100% sorted, 100% live, 0% self! Pure 1D sequential two-pointer merge!
        d_list = dom_buf[b]
        i_list = im_buf[b]
        s1_rows += counts[b]
        s1_reads += len(d_list) + len(i_list)
        xi = ei = x_off = e_off = 0
        nx, ne = len(d_list), len(i_list)
        emitted = []
        while xi < nx and ei < ne:
            ex = d_list[xi]  # (b, dst_b, dlo, ilo, ln, wd, wl)
            en = i_list[ei]  # (src_b, b, dlo, ilo, ln, wd, wl)
            rem_x = ex[4] - x_off
            rem_e = en[4] - e_off
            take = rem_x if rem_x < rem_e else rem_e
            s1_merges += 1
            src_b = en[0]; src_lo = en[2] + e_off
            dst_b = ex[1]; dst_lo = ex[3] + x_off
            cw, cwl = compose_word(en[5], en[6], ex[5], ex[6])
            emitted.append((src_b, dst_b, src_lo, dst_lo, take, cw, cwl))
            if p_state[0] == b and p_state[3] is None:
                x_pos = ex[2] + x_off
                if x_pos <= p_state[1] < x_pos + take:
                    p_state[0] = src_b
                    p_state[1] = src_lo + (p_state[1] - x_pos)
                    p_state[2] += en[6]
            x_off += take; e_off += take
            if x_off == ex[4]: xi += 1; x_off = 0
            if e_off == en[4]: ei += 1; e_off = 0
        assert xi == nx and ei == ne

        eliminated[b] = True
        alive.remove(b)
        for pc in i_list: live_dom[pc[0]] -= 1
        for pc in d_list: live_im[pc[1]] -= 1
        for ep in emitted:
            dom_buf[ep[0]].append(ep); im_buf[ep[1]].append(ep)
            live_dom[ep[0]] += 1; live_im[ep[1]] += 1
        del dom_buf[b]; del im_buf[b]

    # Helper: AHT-Spiral solver using compose_word/pow_word (u128 inline operad!)
    def solve_bucket_inline(b, d_list, i_list, aht_thresh=2, stop_after_aht=False):
        self_pcs = {}
        exit_pcs = []
        entry_pcs = {}
        next_id = 0
        for pc in d_list:
            db, ib, dlo, ilo, ln, wd, wl = pc
            if db == b and ib == b:
                self_pcs[next_id] = [dlo, ilo, ln, wd, wl]; next_id += 1
            else:
                exit_pcs.append([dlo, ib, ilo, ln, wd, wl])
        for pc in i_list:
            db, ib, dlo, ilo, ln, wd, wl = pc
            if db != b:
                entry_pcs[next_id] = [db, dlo, ilo, ln, wd, wl]; next_id += 1

        c1 = c2 = c2_ov = p2_mrg = p2_slf = 0

        if self_pcs and any(v[2] >= aht_thresh for v in self_pcs.values()):
            im_pairs = [(v[1], pid, True) for pid, v in self_pcs.items()]
            im_pairs.extend((v[2], pid, False) for pid, v in entry_pcs.items())
            im_pairs.sort(key=lambda x: x[0])
            im_starts = [x[0] for x in im_pairs]
            im_meta = [(x[1], x[2]) for x in im_pairs]

            def im_rem(v):
                idx = bisect.bisect_left(im_starts, v)
                im_starts.pop(idx); im_meta.pop(idx)

            def im_ins(v, pid, is_s):
                idx = bisect.bisect_left(im_starts, v)
                im_starts.insert(idx, v); im_meta.insert(idx, (pid, is_s))

            heap = [(-v[2], pid) for pid, v in self_pcs.items()]
            heapq.heapify(heap)

            while heap:
                neg_L, uid = heapq.heappop(heap)
                u = self_pcs.get(uid)
                if u is None or u[2] != -neg_L: continue
                L = u[2]
                if L < aht_thresh: break
                dlo, ilo, L, wd, wl = u
                delta = ilo - dlo
                if abs(delta) < L:
                    c1 += 1
                    del self_pcs[uid]; im_rem(ilo)
                    if delta == 0:
                        if p_state[0] == b and p_state[3] is None and dlo <= p_state[1] < dlo + L:
                            p_state[3] = (wd, p_state[2], wl); p_state[0] = -1
                        continue
                    if delta > 0:
                        q, rem = divmod(L, delta)
                        if p_state[0] == b and p_state[3] is None and dlo + delta <= p_state[1] < dlo + L:
                            m = (p_state[1] - dlo) // delta
                            p_state[1] -= m * delta; p_state[2] += m * wl
                        if rem > 0:
                            nid = next_id; next_id += 1
                            w1, wl1 = pow_word(wd, wl, q + 1)
                            i1 = dlo + (q + 1) * delta
                            self_pcs[nid] = [dlo, i1, rem, w1, wl1]
                            im_ins(i1, nid, True); heapq.heappush(heap, (-rem, nid))
                        ln2 = delta - rem
                        if ln2 > 0:
                            nid = next_id; next_id += 1
                            w2, wl2 = pow_word(wd, wl, q)
                            self_pcs[nid] = [dlo + rem, dlo + L, ln2, w2, wl2]
                            im_ins(dlo + L, nid, True); heapq.heappush(heap, (-ln2, nid))
                    else:
                        d = -delta; q, rem = divmod(L, d)
                        if p_state[0] == b and p_state[3] is None and dlo <= p_state[1] < dlo + L - d:
                            m = (dlo + L - 1 - p_state[1]) // d
                            p_state[1] += m * d; p_state[2] += m * wl
                        ln1 = d - rem
                        if ln1 > 0:
                            nid = next_id; next_id += 1
                            w1, wl1 = pow_word(wd, wl, q)
                            i1 = dlo - d + rem
                            self_pcs[nid] = [dlo + L - d, i1, ln1, w1, wl1]
                            im_ins(i1, nid, True); heapq.heappush(heap, (-ln1, nid))
                        if rem > 0:
                            nid = next_id; next_id += 1
                            w2, wl2 = pow_word(wd, wl, q + 1)
                            self_pcs[nid] = [dlo + L - rem, dlo - d, rem, w2, wl2]
                            im_ins(dlo - d, nid, True); heapq.heappush(heap, (-rem, nid))
                else:
                    c2 += 1
                    del self_pcs[uid]; im_rem(ilo)
                    dhi = dlo + L
                    idx = max(0, bisect.bisect_right(im_starts, dlo) - 1)
                    ovs = []
                    while idx < len(im_starts) and im_starts[idx] < dhi:
                        vi = im_starts[idx]; vid, is_s = im_meta[idx]
                        vln = self_pcs[vid][2] if is_s else entry_pcs[vid][3]
                        if vi + vln > dlo: ovs.append((vi, vid, is_s))
                        idx += 1
                    c2_ov += len(ovs)
                    for vi, vid, is_s in ovs:
                        im_rem(vi)
                        if is_s:
                            vdlo, vilo, vln, vwd, vwl = self_pcs.pop(vid)
                            olo = max(dlo, vilo); ohi = min(dhi, vilo + vln)
                            ll, ml, rl = olo - vilo, ohi - olo, vilo + vln - ohi
                            if ll > 0:
                                lid = next_id; next_id += 1
                                self_pcs[lid] = [vdlo, vilo, ll, vwd, vwl]
                                im_ins(vilo, lid, True); heapq.heappush(heap, (-ll, lid))
                            mdlo = vdlo + ll; milo = ilo + (olo - dlo)
                            if p_state[0] == b and p_state[3] is None and olo <= p_state[1] < ohi:
                                p_state[1] = mdlo + (p_state[1] - olo); p_state[2] += vwl
                            mwd, mwl = compose_word(vwd, vwl, wd, wl)
                            mid = next_id; next_id += 1
                            self_pcs[mid] = [mdlo, milo, ml, mwd, mwl]
                            im_ins(milo, mid, True); heapq.heappush(heap, (-ml, mid))
                            if rl > 0:
                                rid = next_id; next_id += 1
                                self_pcs[rid] = [vdlo + ll + ml, ohi, rl, vwd, vwl]
                                im_ins(ohi, rid, True); heapq.heappush(heap, (-rl, rid))
                        else:
                            eb, elo, vilo, vln, vwd, vwl = entry_pcs.pop(vid)
                            olo = max(dlo, vilo); ohi = min(dhi, vilo + vln)
                            ll, ml, rl = olo - vilo, ohi - olo, vilo + vln - ohi
                            if ll > 0:
                                lid = next_id; next_id += 1
                                entry_pcs[lid] = [eb, elo, vilo, ll, vwd, vwl]
                                im_ins(vilo, lid, False)
                            melo = elo + ll; milo = ilo + (olo - dlo)
                            if p_state[0] == b and p_state[3] is None and olo <= p_state[1] < ohi:
                                p_state[0] = eb; p_state[1] = melo + (p_state[1] - olo); p_state[2] += vwl
                            mwd, mwl = compose_word(vwd, vwl, wd, wl)
                            mid = next_id; next_id += 1
                            entry_pcs[mid] = [eb, melo, milo, ml, mwd, mwl]
                            im_ins(milo, mid, False)
                            if rl > 0:
                                rid = next_id; next_id += 1
                                entry_pcs[rid] = [eb, elo + ll + ml, ohi, rl, vwd, vwl]
                                im_ins(ohi, rid, False)

        if stop_after_aht:
            # Return remaining self_pcs directly as the core permutation!
            return self_pcs, (c1, c2, c2_ov, 0, 0)

        emitted = []
        if not self_pcs:
            if exit_pcs:
                exit_pcs.sort(key=lambda x: x[0])
                ent_l = sorted(entry_pcs.values(), key=lambda x: x[2])
                xi = ei = xo = eo = 0
                nx, ne = len(exit_pcs), len(ent_l)
                while xi < nx and ei < ne:
                    ex = exit_pcs[xi]; en = ent_l[ei]
                    take = min(ex[3] - xo, en[3] - eo)
                    p2_mrg += 1
                    sb = en[0]; slo = en[1] + eo
                    db = ex[1]; dlo_t = ex[2] + xo
                    cw, cwl = compose_word(en[4], en[5], ex[4], ex[5])
                    emitted.append((sb, db, slo, dlo_t, take, cw, cwl))
                    if p_state[0] == b and p_state[3] is None:
                        xp = ex[0] + xo
                        if xp <= p_state[1] < xp + take:
                            p_state[0] = sb; p_state[1] = slo + (p_state[1] - xp); p_state[2] += en[5]
                    xo += take; eo += take
                    if xo == ex[3]: xi += 1; xo = 0
                    if eo == en[3]: ei += 1; eo = 0
        else:
            # Endpoint RV finish on thin self pieces
            nk, nl, nw, nwl, neb, nel, ndl = [], [], [], [], [], [], []
            dp_p, im_p = [], []
            for pid, (dlo, ilo, ln, wd, wl) in self_pcs.items():
                nid = len(nk)
                nk.append(0); nl.append(ln); nw.append(wd); nwl.append(wl); neb.append(b); nel.append(-1); ndl.append(dlo)
                dp_p.append((dlo, nid)); im_p.append((ilo, nid))
            for dlo, eb, elo, ln, wd, wl in exit_pcs:
                nid = len(nk)
                nk.append(1); nl.append(ln); nw.append(wd); nwl.append(wl); neb.append(eb); nel.append(elo); ndl.append(dlo)
                dp_p.append((dlo, nid))
            for pid, (eb, elo, ilo, ln, wd, wl) in entry_pcs.items():
                nid = len(nk)
                nk.append(2); nl.append(ln); nw.append(wd); nwl.append(wl); neb.append(eb); nel.append(elo); ndl.append(-1)
                im_p.append((ilo, nid))
            dp_p.sort(key=lambda x: x[0]); im_p.sort(key=lambda x: x[0])
            d_ids = [x[1] for x in dp_p]; i_ids = [x[1] for x in im_p]
            nn = len(nk)
            dp = [-1]*nn; dn = [-1]*nn; ip = [-1]*nn; inn = [-1]*nn
            for idx, u in enumerate(d_ids):
                dp[u] = d_ids[idx-1] if idx > 0 else -1
                dn[u] = d_ids[idx+1] if idx+1 < len(d_ids) else -1
            dt = d_ids[-1] if d_ids else -1
            for idx, u in enumerate(i_ids):
                ip[u] = i_ids[idx-1] if idx > 0 else -1
                inn[u] = i_ids[idx+1] if idx+1 < len(i_ids) else -1
            it = i_ids[-1] if i_ids else -1

            def du(u):
                nonlocal dt
                pr, nx = dp[u], dn[u]
                if pr != -1: dn[pr] = nx
                if nx != -1: dp[nx] = pr
                else: dt = pr
                dp[u] = dn[u] = -1
            def iu(u):
                nonlocal it
                pr, nx = ip[u], inn[u]
                if pr != -1: inn[pr] = nx
                if nx != -1: ip[nx] = pr
                else: it = pr
                ip[u] = inn[u] = -1
            def da(a, u):
                nonlocal dt
                nx = dn[a]; dp[u] = a; dn[u] = nx; dn[a] = u
                if nx != -1: dp[nx] = u
                else: dt = u
            def ia(a, u):
                nonlocal it
                nx = inn[a]; ip[u] = a; inn[u] = nx; inn[a] = u
                if nx != -1: ip[nx] = u
                else: it = u

            pin = (p_state[0] == b and p_state[3] is None)
            pnid = -1; poff = 0
            if pin:
                for dlo_u, u in dp_p:
                    if dlo_u <= p_state[1] < dlo_u + nl[u]:
                        pnid = u; poff = p_state[1] - dlo_u; break
            while dt != -1:
                d, i = dt, it
                if d == i:
                    p2_slf += 1
                    if pin and pnid == d:
                        p_state[3] = (nw[d], p_state[2], nwl[d]); p_state[0] = -1; pin = False
                    du(d); iu(d); continue
                kd, ki, ld, li = nk[d], nk[i], nl[d], nl[i]
                if kd == 1 and ki == 2:
                    p2_mrg += 1
                    take = min(ld, li)
                    sb, db = neb[i], neb[d]
                    slo, dlo_t = nel[i] + (li - take), nel[d] + (ld - take)
                    cw, cwl = compose_word(nw[i], nwl[i], nw[d], nwl[d])
                    emitted.append((sb, db, slo, dlo_t, take, cw, cwl))
                    if pin and pnid == d and poff >= ld - take:
                        p_state[0] = sb; p_state[1] = slo + (poff - (ld - take)); p_state[2] += nwl[i]; pin = False
                    if ld == take: du(d)
                    else: nl[d] -= take
                    if li == take: iu(i)
                    else: nl[i] -= take
                elif ld > li:
                    p2_slf += 1
                    if kd == 0:
                        if pin and pnid == d and poff >= ld - li:
                            if ki == 2:
                                p_state[0] = neb[i]; p_state[1] = nel[i] + (poff - (ld - li)); p_state[2] += nwl[i]; pin = False
                            else:
                                pnid = i; poff -= (ld - li); p_state[2] += nwl[i]
                        nw[i], nwl[i] = compose_word(nw[i], nwl[i], nw[d], nwl[d])
                        nl[d] -= li
                        if inn[d] != i: iu(i); ia(d, i)
                    else:
                        if pin and pnid == d and poff >= ld - li:
                            pnid = i; poff -= (ld - li); p_state[2] += nwl[i]
                        nw[i], nwl[i] = compose_word(nw[i], nwl[i], nw[d], nwl[d])
                        nk[i] = 1; neb[i] = neb[d]; nel[i] = nel[d] + (ld - li); nl[d] -= li; iu(i)
                elif li > ld:
                    p2_slf += 1
                    if ki == 0:
                        if pin:
                            if pnid == d: p_state[2] += nwl[i]
                            elif pnid == i and poff >= li - ld: pnid = d; poff -= (li - ld)
                        nw[d], nwl[d] = compose_word(nw[i], nwl[i], nw[d], nwl[d])
                        nl[i] -= ld
                        if dn[i] != d: du(d); da(i, d)
                    else:
                        slo = nel[i] + (li - ld)
                        if pin and pnid == d:
                            p_state[0] = neb[i]; p_state[1] = slo + poff; p_state[2] += nwl[i]; pin = False
                        nw[d], nwl[d] = compose_word(nw[i], nwl[i], nw[d], nwl[d])
                        nk[d] = 2; neb[d] = neb[i]; nel[d] = slo; nl[i] -= ld; du(d)
                else:
                    p2_slf += 1
                    if kd == 0 and ki == 0:
                        if pin and pnid == d: pnid = i; p_state[2] += nwl[i]
                        nw[i], nwl[i] = compose_word(nw[i], nwl[i], nw[d], nwl[d])
                        if inn[d] == i: iu(d)
                        else: iu(i); ia(d, i); iu(d)
                        du(d)
                    elif kd == 1 and ki == 0:
                        if pin and pnid == d: pnid = i; p_state[2] += nwl[i]
                        nw[i], nwl[i] = compose_word(nw[i], nwl[i], nw[d], nwl[d])
                        nk[i] = 1; neb[i] = neb[d]; nel[i] = nel[d]; iu(i); du(d)
                    else:
                        if pin and pnid == d:
                            p_state[0] = neb[i]; p_state[1] = nel[i] + poff; p_state[2] += nwl[i]; pin = False
                        nw[d], nwl[d] = compose_word(nw[i], nwl[i], nw[d], nwl[d])
                        nk[d] = 2; neb[d] = neb[i]; nel[d] = nel[i]; du(d); iu(i)
        return emitted, (c1, c2, c2_ov, p2_mrg, p2_slf)

    # ---- STAGE 2: Min-Degree IET-BSE-AHT until 1 hub remains ----
    while len(alive) > 1:
        b = min(alive, key=lambda x: (live_dom[x] + live_im[x], counts[x], x))
        raw_d = dom_buf[b]; raw_i = im_buf[b]
        d_list = [pc for pc in raw_d if not eliminated[pc[1]]]
        i_list = [pc for pc in raw_i if not eliminated[pc[0]]]
        s2_buf_reads_live += len(d_list) + len(i_list)
        s2_buf_reads_stale += (len(raw_d) - len(d_list)) + (len(raw_i) - len(i_list))
        # Note: if k_self==0 and no fill-in, sort is cheap; track exact live items needing sort
        s2_sort_items += len(d_list) + len(i_list)

        emitted, (c1, c2, c2_ov, p2_mrg, p2_slf) = solve_bucket_inline(b, d_list, i_list, aht_thresh=2, stop_after_aht=False)
        s2_c1 += c1; s2_c2 += c2; s2_c2_ov += c2_ov; s2_p2_mrg += p2_mrg; s2_p2_slf += p2_slf
        s2_emitted += len(emitted)

        eliminated[b] = True; alive.remove(b)
        for pc in i_list:
            if pc[0] != b: live_dom[pc[0]] -= 1
        for pc in d_list:
            if pc[1] != b: live_im[pc[1]] -= 1
        for ep in emitted:
            dom_buf[ep[0]].append(ep); im_buf[ep[1]].append(ep)
            live_dom[ep[0]] += 1; live_im[ep[1]] += 1
        del dom_buf[b]; del im_buf[b]

    # ---- STAGE 3: Final Hub Bucket ----
    hub_b = next(iter(alive))
    raw_d = dom_buf[hub_b]; raw_i = im_buf[hub_b]
    d_hub = [pc for pc in raw_d if not eliminated[pc[1]]]
    i_hub = [pc for pc in raw_i if not eliminated[pc[0]]]
    s2_buf_reads_live += len(d_hub) + len(i_hub)
    s2_buf_reads_stale += (len(raw_d) - len(d_hub)) + (len(raw_i) - len(i_hub))
    s2_sort_items += len(d_hub) + len(i_hub)

    if not stop_at_hub:
        _, (hc1, hc2, hc2_ov, hp2_m, hp2_s) = solve_bucket_inline(hub_b, d_hub, i_hub, aht_thresh=2, stop_after_aht=False)
        assert p_state[3] is not None
        root_wd, sh, wl = p_state[3]
        assert wl == n

        # Expand & measure output-backed memoized Flat<=16 expansion!
        memo_pos = {}
        out = bytearray()
        node_exp = 0; memcpys = 0; memcpy_bytes = 0; flat_writes = 0; flat_bytes = 0
        stack = [root_wd]
        while stack:
            u = stack.pop()
            if isinstance(u, bytes):
                flat_writes += 1; flat_bytes += len(u); out.extend(u); continue
            if isinstance(u, tuple):
                # ('fin', rid, start_pos)
                memo_pos[u[1]] = (u[2], len(out) - u[2])
                continue
            if u in memo_pos:
                sp, ln = memo_pos[u]
                memcpys += 1; memcpy_bytes += ln
                out.extend(out[sp:sp + ln])
                continue
            r = rules[u]
            if r[0] == 0:
                flat_writes += 1; flat_bytes += len(r[1]); out.extend(r[1])
            elif r[0] == 1:
                node_exp += 1
                stack.append(('fin', u, len(out)))
                stack.append(r[2]); stack.append(r[1])
            else:
                node_exp += 1
                stack.append(('fin', u, len(out)))
                for _ in range(r[2]): stack.append(r[1])
        assert len(out) == n
        if verify_bytes is not None:
            s = sh % n
            decoded = bytes(out[s:]) + bytes(out[:s])
            assert decoded == verify_bytes, "MISMATCH in full Flat<=16 IET-BSE-AHT!"

        return {
            "mode": "full_aht_flat16",
            "hub_b": hub_b, "hub_q": len(d_hub), "hub_n": counts[hub_b],
            "s1_buckets": s1_buckets, "s1_rows": s1_rows, "s1_reads": s1_reads, "s1_merges": s1_merges,
            "s2_live": s2_buf_reads_live, "s2_stale": s2_buf_reads_stale, "s2_sort": s2_sort_items,
            "s2_emit": s2_emitted,
            "aht_c1": s2_c1 + hc1, "aht_c2": s2_c2 + hc2, "aht_c2_ov": s2_c2_ov + hc2_ov,
            "p2_mrg": s2_p2_mrg + hp2_m, "p2_slf": s2_p2_slf + hp2_s,
            "u128_inline_merges": u128_inline_merges,
            "slp_cat_allocs": slp_cat_allocs,
            "slp_pow_allocs": slp_pow_allocs,
            "flat_leaf_mats": flat_leaf_materializations,
            "exp_node_exp": node_exp, "exp_flat_writes": flat_writes, "exp_memcpys": memcpys,
            "exp_leaves": flat_writes + memcpys, "exp_total_ops": node_exp + flat_writes + memcpys,
        }
    else:
        # Early-Stop Hub Fusion: run Thick-AHT(th=hub_aht_thresh) inside b_hub, then build & walk core permutation!
        core_pcs, (hc1, hc2, hc2_ov, _, _) = solve_bucket_inline(hub_b, d_hub, i_hub, aht_thresh=hub_aht_thresh, stop_after_aht=True)
        # Compact remaining surviving coordinates in b_hub into 0..m_core-1!
        dom_sorted = sorted(core_pcs.values(), key=lambda x: x[0])
        m_core = sum(pc[2] for pc in dom_sorted)
        q_core = len(dom_sorted)
        # Build coordinate map via bisect on dom_sorted starts -> compact index!
        d_starts = [pc[0] for pc in dom_sorted]
        d_pref = [0] * (q_core + 1)
        for idx, pc in enumerate(dom_sorted):
            d_pref[idx + 1] = d_pref[idx] + pc[2]

        # Build explicit core permutation edges[0..m_core) and word[0..m_core)
        edges = [0] * m_core
        words_core = [None] * m_core
        for idx, (dlo, ilo, ln, wd, wl) in enumerate(dom_sorted):
            base_src = d_pref[idx]
            # Locate ilo in dom_sorted! (Note: [ilo, ilo+ln) can span multiple dom_sorted intervals!)
            cur_i = ilo
            rem = ln
            off = 0
            j = bisect.bisect_right(d_starts, cur_i) - 1
            while rem > 0:
                t_dlo, _, t_ln, _, _ = dom_sorted[j]
                t_off = cur_i - t_dlo
                take = min(rem, t_ln - t_off)
                base_dst = d_pref[j] + t_off
                for k in range(take):
                    edges[base_src + off + k] = base_dst + k
                    words_core[base_src + off + k] = wd
                cur_i += take; rem -= take; off += take; j += 1

        # Locate p in dom_sorted
        assert p_state[0] == hub_b and p_state[3] is None
        j_p = bisect.bisect_right(d_starts, p_state[1]) - 1
        start_row = d_pref[j_p] + (p_state[1] - dom_sorted[j_p][0])

        # Walk the m_core cycle with output-backed memoized Flat<=16 emission!
        memo_pos = {}
        out = bytearray()
        walk_steps = 0
        walk_inline = 0
        walk_memcpys = 0
        walk_node_exp = 0
        walk_flat_in_slp = 0

        cur = start_row
        for _ in range(m_core):
            walk_steps += 1
            wd = words_core[cur]
            if isinstance(wd, bytes):
                walk_inline += 1
                out.extend(wd)
            elif wd in memo_pos:
                sp, ln = memo_pos[wd]
                walk_memcpys += 1
                out.extend(out[sp:sp + ln])
            else:
                # Expand SLP node wd with memoization
                stack = [wd]
                while stack:
                    u = stack.pop()
                    if isinstance(u, tuple):
                        memo_pos[u[1]] = (u[2], len(out) - u[2]); continue
                    if u in memo_pos:
                        sp, ln = memo_pos[u]
                        walk_memcpys += 1; out.extend(out[sp:sp + ln]); continue
                    r = rules[u]
                    if r[0] == 0:
                        walk_flat_in_slp += 1; out.extend(r[1])
                    elif r[0] == 1:
                        walk_node_exp += 1
                        stack.append(('fin', u, len(out)))
                        stack.append(r[2]); stack.append(r[1])
                    else:
                        walk_node_exp += 1
                        stack.append(('fin', u, len(out)))
                        for _ in range(r[2]): stack.append(r[1])
            cur = edges[cur]
        assert cur == start_row, "Core walk did not close single cycle!"
        assert len(out) == n
        if verify_bytes is not None:
            s = p_state[2] % n
            decoded = bytes(out[s:]) + bytes(out[:s])
            assert decoded == verify_bytes, "MISMATCH in Early-Stop Hub Fusion!"

        return {
            "mode": f"hub_fusion_th{hub_aht_thresh}",
            "hub_b": hub_b, "hub_q": len(d_hub), "hub_n": counts[hub_b],
            "s1_buckets": s1_buckets, "s1_rows": s1_rows, "s1_reads": s1_reads, "s1_merges": s1_merges,
            "s2_live": s2_buf_reads_live, "s2_stale": s2_buf_reads_stale, "s2_sort": s2_sort_items,
            "s2_emit": s2_emitted,
            "aht_c1": s2_c1 + hc1, "aht_c2": s2_c2 + hc2, "aht_c2_ov": s2_c2_ov + hc2_ov,
            "p2_mrg": s2_p2_mrg, "p2_slf": s2_p2_slf,
            "u128_inline_merges": u128_inline_merges,
            "slp_cat_allocs": slp_cat_allocs,
            "m_core": m_core, "q_core": q_core,
            "walk_steps": walk_steps, "walk_inline": walk_inline,
            "walk_memcpys": walk_memcpys, "walk_node_exp": walk_node_exp, "walk_flat_in_slp": walk_flat_in_slp,
            "tot_emit_ops": walk_steps + walk_node_exp + walk_flat_in_slp + walk_memcpys,
        }


def main():
    blocks = mb.load_authenticated_blocks()
    print("=== CYCLE 2 OBJECTIVE 3: DAG-BATCH + u128 FLAT<=16 OPERAD + HUB FUSION (ALL 6 BLOCKS) ===\n")

    tot_m16 = 0; tot_r0 = 0; tot_n = 0
    agg_full = defaultdict(int)
    agg_hub8 = defaultdict(int)
    agg_hub16 = defaultdict(int)

    for binfo, raw in blocks:
        fname = binfo["file"]
        n, p = binfo["n"], binfo["primary_index"]
        runs = mb.rle_from_bytes(raw)
        r0 = len(runs)
        _, m16, _ = mb.compute_open_m16(runs, p, limit=16)
        ref = mb.decode_lf_reference_bytes(raw, p)
        tot_m16 += m16; tot_r0 += r0; tot_n += n

        st_full = run_hybrid_iet_bse(runs, p, inline_cap=16, stop_at_hub=False, verify_bytes=ref)
        st_h8 = run_hybrid_iet_bse(runs, p, inline_cap=16, stop_at_hub=True, hub_aht_thresh=8, verify_bytes=ref)
        st_h16 = run_hybrid_iet_bse(runs, p, inline_cap=16, stop_at_hub=True, hub_aht_thresh=16, verify_bytes=ref)

        for k, v in st_full.items():
            if isinstance(v, int): agg_full[k] += v
        for k, v in st_h8.items():
            if isinstance(v, int): agg_hub8[k] += v
        for k, v in st_h16.items():
            if isinstance(v, int): agg_hub16[k] += v

        tot_comp = st_full["u128_inline_merges"] + st_full["slp_cat_allocs"] + st_full["slp_pow_allocs"]
        inl_pct = 100.0 * st_full["u128_inline_merges"] / tot_comp

        # Calibrated i9 Open budget (us): 0.72-0.90 ns * n + 4.8-5.8 ns * m16
        open_lo_us = (0.72 * n + 4.8 * m16) / 1000.0
        open_hi_us = (0.90 * n + 5.8 * m16) / 1000.0

        print(f"[{fname}] n={n} r0={r0} m16={m16} | i9 Open bar = {open_lo_us:.0f}-{open_hi_us:.0f} us")
        print(f"  Stage 1 (0-sort, 0-stale IS): {st_full['s1_buckets']:2d}/{len(set(raw))} bkts, {st_full['s1_rows']:6d} rows ({100*st_full['s1_rows']/n:4.1f}%), {st_full['s1_merges']:5d} merges (0 sort, 0 stale)")
        print(f"  Stage 2+3 FULL AHT+Flat16   : s2_rd={st_full['s2_live']+st_full['s2_stale']:6d} (live={st_full['s2_live']:6d}, stale={st_full['s2_stale']:6d}) sort={st_full['s2_sort']:6d} ({st_full['s2_sort']/r0:.2f}r)")
        print(f"    u128 operad               : {st_full['u128_inline_merges']:6d} inline ({inl_pct:4.1f}% ZERO-SLP!) vs {st_full['slp_cat_allocs']:6d} SLP cats ({st_full['slp_cat_allocs']/r0:.2f}r)")
        print(f"    Full Exp (Flat<=16+Memo)  : leaves={st_full['exp_leaves']:5d} ({st_full['exp_leaves']/m16:.2f}*m16) [flat={st_full['exp_flat_writes']}, cpy={st_full['exp_memcpys']}] tot_ops={st_full['exp_total_ops']:6d} ({st_full['exp_total_ops']/m16:.2f}*m16)")
        print(f"  Hub-Fusion (b={repr(bytes([st_h8['hub_b']]))}, th= 8) : m_core={st_h8['m_core']:5d} ({st_h8['m_core']/m16:.3f}*m16, edges={4*st_h8['m_core']//1024:3d}KiB, q_core={st_h8['q_core']:4d}) | "
              f"hub_c2={st_h8['aht_c2']:4d} p2_slf={st_h8['p2_slf']:5d} | walk={st_h8['walk_steps']:5d} + slp_exp={st_h8['walk_node_exp']:5d} + flat={st_h8['walk_flat_in_slp']:5d} + cpy={st_h8['walk_memcpys']:5d}")
        print(f"  Hub-Fusion (b={repr(bytes([st_h16['hub_b']]))}, th=16) : m_core={st_h16['m_core']:5d} ({st_h16['m_core']/m16:.3f}*m16, edges={4*st_h16['m_core']//1024:3d}KiB, q_core={st_h16['q_core']:4d}) | "
              f"hub_c2={st_h16['aht_c2']:4d} p2_slf={st_h16['p2_slf']:5d} | walk={st_h16['walk_steps']:5d} + slp_exp={st_h16['walk_node_exp']:5d} + flat={st_h16['walk_flat_in_slp']:5d} + cpy={st_h16['walk_memcpys']:5d}\n")

    print("=" * 105)
    print(f"6-BLOCK AGGREGATES (n={tot_n}, r0={tot_r0}, m16={tot_m16}):")
    open_6_lo = (0.72 * tot_n + 4.8 * tot_m16) / 1000.0
    open_6_hi = (0.90 * tot_n + 5.8 * tot_m16) / 1000.0
    print(f"  i9 OpenCells 6-block bar    : {open_6_lo/1000:.2f}-{open_6_hi/1000:.2f} ms ({open_6_lo/6:.0f}-{open_6_hi/6:.0f} us/blk avg)")
    print(f"  Stage 1 (0-sort IS)         : {agg_full['s1_rows']} rows ({100*agg_full['s1_rows']/tot_n:.1f}%), {agg_full['s1_merges']} merges, 0 sort, 0 stale")
    tot_c = agg_full["u128_inline_merges"] + agg_full["slp_cat_allocs"] + agg_full["slp_pow_allocs"]
    print(f"  Full AHT+Flat16             : s2_sort={agg_full['s2_sort']} ({agg_full['s2_sort']/tot_r0:.2f}r vs 2.40r C1) | "
          f"stale={agg_full['s2_stale']} ({agg_full['s2_stale']/tot_r0:.2f}r vs 1.79r C1) | "
          f"u128_inline={agg_full['u128_inline_merges']} ({100*agg_full['u128_inline_merges']/tot_c:.1f}%), SLP_cats={agg_full['slp_cat_allocs']} ({agg_full['slp_cat_allocs']/tot_r0:.2f}r) | "
          f"exp_leaves={agg_full['exp_leaves']} ({agg_full['exp_leaves']/tot_m16:.2f}*m16), exp_ops={agg_full['exp_total_ops']} ({agg_full['exp_total_ops']/tot_m16:.2f}*m16)")
    print(f"  Hub-Fusion (th= 8)          : m_core={agg_hub8['m_core']} ({agg_hub8['m_core']/tot_m16:.3f}*m16) | "
          f"p2_slf={agg_hub8['p2_slf']} (vs {agg_full['p2_slf']} full!) | slp_cats={agg_hub8['slp_cat_allocs']} | walk+slp_ops={agg_hub8['tot_emit_ops']} ({agg_hub8['tot_emit_ops']/tot_m16:.2f}*m16)")
    print(f"  Hub-Fusion (th=16)          : m_core={agg_hub16['m_core']} ({agg_hub16['m_core']/tot_m16:.3f}*m16) | "
          f"p2_slf={agg_hub16['p2_slf']} (vs {agg_full['p2_slf']} full!) | slp_cats={agg_hub16['slp_cat_allocs']} | walk+slp_ops={agg_hub16['tot_emit_ops']} ({agg_hub16['tot_emit_ops']/tot_m16:.2f}*m16)")


if __name__ == "__main__":
    main()
