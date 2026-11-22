#!/usr/bin/env python3
"""
Cycle 4 Task 2: Independent Adversarial Audit of Beta's Theorem B4 (Zero-Tombstone Causal Routing).
1. Exhaustively test ALL arbitrary binary columns (n <= 8, all p) and ternary columns (n <= 6, all p)
   including unary, periodic, multi-cycle, self-loops, closed cycles under random static rank orders!
   Also test Beta's pareto_superbucket.run_superbucket_zero_tombstone directly for bugs/crashes!
2. Test ALL 4 adversarial killers (3-phase, AltRing, Low-r Padded-Ring, SymMidRun) + Fib_20+$ under Theorem B4!
3. Independently verify Beta's claimed numbers on block-0.L & block-210.L:
   - 0.00% stale tombstones
   - ~2.13 r_0 total appends/scans (vs ~4.20 r_0 in non-causal 2-buffer)
   - peak_live_in_bufs <= q_0 vs TRUE mid-step peak_total_live = max(live_in_bufs + active_workspace_pieces)!
"""
import hashlib
import itertools
import json
import os
import random
import sys
import time
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-b")
import pareto_superbucket as ps
import escape_padded_ring as epr
import measure_iet_bse as mb


def ref_lf_cycle(L, p):
    """Exact reference LF decode (1 full cycle of length clen, tiled to n if periodic BWT)."""
    n = len(L)
    mx = max(L) + 1
    cnt = [0] * mx
    for x in L: cnt[x] += 1
    C = [0] * mx
    acc = 0
    for c in range(mx): C[c] = acc; acc += cnt[c]
    occ = list(C)
    LF = [0] * n
    for i, x in enumerate(L): LF[i] = occ[x]; occ[x] += 1
    out = []
    cur = p
    while True:
        out.append(L[cur])
        cur = LF[cur]
        if cur == p:
            break
    out.reverse()
    return out


def independent_b4_audit(runs, p, rank_order, aht_thresh=2):
    """
    100% independent first-principles implementation of Theorem B4 Zero-Tombstone Causal Routing
    + AHT-Spiral, strictly verifying:
      - ZERO stale pieces ever popped from self_buf/exit_buf/entry_buf
      - Every piece routed ONCE to min(src_rank, dst_rank)
      - Exact tracking of BOTH peak_live_bufs AND mid-step peak_total_live (bufs + active workspace)!
      - Exact primary-row tracking!
    """
    import bisect, heapq
    n = sum(w for _, w in runs)
    mx = max(c for c, _ in runs)
    counts = [0] * (mx + 1)
    for c, w in runs: counts[c] += w
    C = [0] * (mx + 2)
    for c in range(mx + 1): C[c + 1] = C[c] + counts[c]

    active_syms = [c for c in range(mx + 1) if counts[c] > 0]
    rank = {c: i for i, c in enumerate(rank_order)}
    K = len(rank_order)

    # Initial strips: (src_sym, dst_sym, dlo, ilo, ln) where piece maps [dlo, dlo+ln) in F_{src} -> [ilo, ilo+ln) in F_{dst}
    # Using FL direction (F_c -> L-position) matching Beta's pareto_superbucket!
    f_cur = list(C[:mx + 1])
    init_strips = []
    pos = 0
    im_b = 0
    while im_b <= mx and C[im_b + 1] == 0: im_b += 1
    for c, w in runs:
        rem = w; cur_im = pos; cur_dom = f_cur[c]
        while rem > 0:
            while C[im_b + 1] <= cur_im: im_b += 1
            tk = min(rem, C[im_b + 1] - cur_im)
            init_strips.append((c, im_b, cur_dom, cur_im, tk))
            cur_im += tk; cur_dom += tk; rem -= tk
        f_cur[c] += w; pos += w
    q0 = len(init_strips)

    self_buf = [[] for _ in range(K)]
    exit_buf = [[] for _ in range(K)]
    entry_buf = [[] for _ in range(K)]

    rules = []
    def cat_w(w1, l1, w2, l2):
        if l1 + l2 <= 16:
            return w1 + w2, l1 + l2
        id1 = w1 if isinstance(w1, int) else (rules.append((0, w1)) or len(rules) - 1)
        id2 = w2 if isinstance(w2, int) else (rules.append((0, w2)) or len(rules) - 1)
        rules.append((1, id1, id2))
        return len(rules) - 1, l1 + l2

    def pow_w(w1, l1, k):
        if k == 1: return w1, l1
        if l1 * k <= 16: return w1 * k, l1 * k
        id1 = w1 if isinstance(w1, int) else (rules.append((0, w1)) or len(rules) - 1)
        rules.append((2, id1, k))
        return len(rules) - 1, l1 * k

    def expand_w(wd):
        if not isinstance(wd, int):
            return list(wd)
        out = []
        st = [(wd, 1)]
        while st:
            u, m = st.pop()
            r = rules[u]
            if r[0] == 0:
                out.extend(list(r[1]) * m)
            elif r[0] == 1:
                if m == 1:
                    st.append((r[2], 1)); st.append((r[1], 1))
                else:
                    sub = expand_w(u)
                    out.extend(sub * m)
            else:
                st.append((r[1], r[2] * m))
        return out

    writes = 0; reads = 0; stale = 0
    live_buf = 0; peak_buf = 0; peak_total_midstep = 0
    cur_e = -1

    def route(sr, dr, dlo, ilo, ln, wd, wl):
        nonlocal writes, live_buf, peak_buf
        assert sr > cur_e and dr > cur_e, f"Causal violation: routed ({sr}->{dr}) at cur_e={cur_e}"
        writes += 1
        live_buf += 1
        if live_buf > peak_buf: peak_buf = live_buf
        if sr == dr:
            self_buf[sr].append([dlo, ilo, ln, wd, wl])
        elif sr < dr:
            exit_buf[sr].append([dlo, dr, ilo, ln, wd, wl])
        else:
            entry_buf[dr].append([sr, dlo, ilo, ln, wd, wl])

    for c, im_b, dlo, ilo, ln in init_strips:
        route(rank[c], rank[im_b], dlo, ilo, ln, (c,), 1)

    p_sym = 0
    while C[p_sym + 1] <= p: p_sym += 1
    p_state = [rank[p_sym], p, 0, None]  # [p_r, p_pos, p_shift, p_cycle]

    for e in range(K):
        cur_e = e
        s_list = self_buf[e]; self_buf[e] = []
        x_list = exit_buf[e]; exit_buf[e] = []
        e_list = entry_buf[e]; entry_buf[e] = []

        # Verify ZERO stale pieces!
        for pc in x_list:
            if pc[1] <= e: stale += 1
        for pc in e_list:
            if pc[0] <= e: stale += 1
        popped = len(s_list) + len(x_list) + len(e_list)
        reads += popped
        live_buf -= popped

        self_pcs = {i: pc for i, pc in enumerate(s_list)}
        nid_c = len(s_list)
        entry_pcs = {}
        for pc in e_list:
            entry_pcs[nid_c] = pc; nid_c += 1
        exit_pcs = x_list

        def upd_mid():
            nonlocal peak_total_midstep
            tot_now = live_buf + len(self_pcs) + len(entry_pcs) + len(exit_pcs)
            if tot_now > peak_total_midstep:
                peak_total_midstep = tot_now

        upd_mid()

        # Phase 1: AHT-Spiral
        if self_pcs and any(v[2] >= aht_thresh for v in self_pcs.values()):
            im_p = [(v[1], pid, True) for pid, v in self_pcs.items()]
            im_p.extend((v[2], pid, False) for pid, v in entry_pcs.items())
            im_p.sort(key=lambda x: x[0])
            im_st = [x[0] for x in im_p]
            im_mt = [(x[1], x[2]) for x in im_p]

            def irem(v):
                ix = bisect.bisect_left(im_st, v); im_st.pop(ix); im_mt.pop(ix)
            def iins(v, pid, iss):
                ix = bisect.bisect_left(im_st, v); im_st.insert(ix, v); im_mt.insert(ix, (pid, iss))

            hp = [(-v[2], pid) for pid, v in self_pcs.items()]
            heapq.heapify(hp)
            while hp:
                neg_L, uid = heapq.heappop(hp)
                u = self_pcs.get(uid)
                if u is None or u[2] != -neg_L: continue
                L = u[2]
                if L < aht_thresh: break
                dlo, ilo, L, wd, wl = u
                delta = ilo - dlo
                if abs(delta) < L:
                    del self_pcs[uid]; irem(ilo)
                    if delta == 0:
                        if p_state[0] == e and p_state[3] is None and dlo <= p_state[1] < dlo + L:
                            p_state[3] = (wd, p_state[2], wl); p_state[0] = -1
                        upd_mid(); continue
                    if delta > 0:
                        q, rem = divmod(L, delta)
                        if p_state[0] == e and p_state[3] is None and dlo + delta <= p_state[1] < dlo + L:
                            m = (p_state[1] - dlo) // delta
                            p_state[1] -= m * delta; p_state[2] += m * wl
                        if rem > 0:
                            nid = nid_c; nid_c += 1
                            w1, wl1 = pow_w(wd, wl, q + 1)
                            i1 = dlo + (q + 1) * delta
                            self_pcs[nid] = [dlo, i1, rem, w1, wl1]
                            iins(i1, nid, True); heapq.heappush(hp, (-rem, nid))
                        ln2 = delta - rem
                        if ln2 > 0:
                            nid = nid_c; nid_c += 1
                            w2, wl2 = pow_w(wd, wl, q)
                            self_pcs[nid] = [dlo + rem, dlo + L, ln2, w2, wl2]
                            iins(dlo + L, nid, True); heapq.heappush(hp, (-ln2, nid))
                    else:
                        d = -delta; q, rem = divmod(L, d)
                        if p_state[0] == e and p_state[3] is None and dlo <= p_state[1] < dlo + L - d:
                            m = (dlo + L - 1 - p_state[1]) // d
                            p_state[1] += m * d; p_state[2] += m * wl
                        ln1 = d - rem
                        if ln1 > 0:
                            nid = nid_c; nid_c += 1
                            w1, wl1 = pow_w(wd, wl, q)
                            i1 = dlo - d + rem
                            self_pcs[nid] = [dlo + L - d, i1, ln1, w1, wl1]
                            iins(i1, nid, True); heapq.heappush(hp, (-ln1, nid))
                        if rem > 0:
                            nid = nid_c; nid_c += 1
                            w2, wl2 = pow_w(wd, wl, q + 1)
                            self_pcs[nid] = [dlo + L - rem, dlo - d, rem, w2, wl2]
                            iins(dlo - d, nid, True); heapq.heappush(hp, (-rem, nid))
                else:
                    del self_pcs[uid]; irem(ilo)
                    dhi = dlo + L
                    ix = max(0, bisect.bisect_right(im_st, dlo) - 1)
                    ovs = []
                    while ix < len(im_st) and im_st[ix] < dhi:
                        vi = im_st[ix]; vid, iss = im_mt[ix]
                        vln = self_pcs[vid][2] if iss else entry_pcs[vid][3]
                        if vi + vln > dlo: ovs.append((vi, vid, iss))
                        ix += 1
                    for vi, vid, iss in ovs:
                        irem(vi)
                        if iss:
                            vdlo, vilo, vln, vwd, vwl = self_pcs.pop(vid)
                            olo = max(dlo, vilo); ohi = min(dhi, vilo + vln)
                            ll, ml, rl = olo - vilo, ohi - olo, vilo + vln - ohi
                            if ll > 0:
                                lid = nid_c; nid_c += 1
                                self_pcs[lid] = [vdlo, vilo, ll, vwd, vwl]
                                iins(vilo, lid, True); heapq.heappush(hp, (-ll, lid))
                            mdlo = vdlo + ll; milo = ilo + (olo - dlo)
                            if p_state[0] == e and p_state[3] is None and olo <= p_state[1] < ohi:
                                p_state[1] = mdlo + (p_state[1] - olo); p_state[2] += vwl
                            mwd, mwl = cat_w(vwd, vwl, wd, wl)
                            mid = nid_c; nid_c += 1
                            self_pcs[mid] = [mdlo, milo, ml, mwd, mwl]
                            iins(milo, mid, True); heapq.heappush(hp, (-ml, mid))
                            if rl > 0:
                                rid = nid_c; nid_c += 1
                                self_pcs[rid] = [vdlo + ll + ml, ohi, rl, vwd, vwl]
                                iins(ohi, rid, True); heapq.heappush(hp, (-rl, rid))
                        else:
                            eb, elo, vilo, vln, vwd, vwl = entry_pcs.pop(vid)
                            olo = max(dlo, vilo); ohi = min(dhi, vilo + vln)
                            ll, ml, rl = olo - vilo, ohi - olo, vilo + vln - ohi
                            if ll > 0:
                                lid = nid_c; nid_c += 1
                                entry_pcs[lid] = [eb, elo, vilo, ll, vwd, vwl]
                                iins(vilo, lid, False)
                            melo = elo + ll; milo = ilo + (olo - dlo)
                            if p_state[0] == e and p_state[3] is None and olo <= p_state[1] < ohi:
                                p_state[0] = eb; p_state[1] = melo + (p_state[1] - olo); p_state[2] += vwl
                            mwd, mwl = cat_w(vwd, vwl, wd, wl)
                            mid = nid_c; nid_c += 1
                            entry_pcs[mid] = [eb, melo, milo, ml, mwd, mwl]
                            iins(milo, mid, False)
                            if rl > 0:
                                rid = nid_c; nid_c += 1
                                entry_pcs[rid] = [eb, elo + ll + ml, ohi, rl, vwd, vwl]
                                iins(ohi, rid, False)
                upd_mid()

        # Phase 2: Exact right-endpoint RV / 1D merge
        emitted = []
        if not self_pcs:
            if exit_pcs:
                exit_pcs.sort(key=lambda x: x[0])
                ents = sorted(entry_pcs.values(), key=lambda x: x[2])
                ei = xi = eo = xo = 0
                while xi < len(exit_pcs) and ei < len(ents):
                    ex = exit_pcs[xi]; en = ents[ei]
                    tk = min(ex[3] - xo, en[3] - eo)
                    if p_state[0] == e and p_state[3] is None and ex[0] + xo <= p_state[1] < ex[0] + xo + tk:
                        p_state[0] = en[0]; p_state[1] = en[1] + eo + (p_state[1] - (ex[0] + xo)); p_state[2] += en[5]
                    cw, cwl = cat_w(en[4], en[5], ex[4], ex[5])
                    emitted.append((en[0], ex[1], en[1] + eo, ex[2] + xo, tk, cw, cwl))
                    xo += tk; eo += tk
                    if xo == ex[3]: xi += 1; xo = 0
                    if eo == en[3]: ei += 1; eo = 0
        else:
            # Thin self-pieces (width < aht_thresh, e.g. width 1): resolve via direct point/interval map!
            # Use exact right-endpoint RV (identical to Beta's Phase 2)
            # Build sorted dom and im lists
            dom_arr = [(v[0], 0, i, v) for i, v in enumerate(self_pcs.values())]
            n_s = len(dom_arr)
            im_arr = [(v[1], 0, i, v) for i, v in enumerate(self_pcs.values())]
            for j, v in enumerate(exit_pcs):
                dom_arr.append((v[0], 1, n_s + j, v))
            base_en = n_s + len(exit_pcs)
            for j, v in enumerate(entry_pcs.values()):
                im_arr.append((v[2], 2, base_en + j, v))
            dom_arr.sort(key=lambda x: x[0])
            im_arr.sort(key=lambda x: x[0])
            # Build exact node arrays
            tot_n = base_en + len(entry_pcs)
            nk = [0] * tot_n; nl = [0] * tot_n; nw = [None] * tot_n; nwl = [0] * tot_n
            neb = [-1] * tot_n; nelo = [-1] * tot_n
            for _, knd, uid, v in dom_arr:
                nk[uid] = knd
                if knd == 0: nl[uid], nw[uid], nwl[uid] = v[2], v[3], v[4]
                else: nl[uid], nw[uid], nwl[uid], neb[uid], nelo[uid] = v[3], v[4], v[5], v[1], v[2]
            for _, knd, uid, v in im_arr:
                if knd == 2:
                    nk[uid] = 2; nl[uid], nw[uid], nwl[uid], neb[uid], nelo[uid] = v[3], v[4], v[5], v[0], v[1]
            dp = [-1] * tot_n; dn = [-1] * tot_n; ip = [-1] * tot_n; inn = [-1] * tot_n
            for ix in range(len(dom_arr)):
                u = dom_arr[ix][2]
                if ix > 0: dp[u] = dom_arr[ix - 1][2]
                if ix + 1 < len(dom_arr): dn[u] = dom_arr[ix + 1][2]
            d_tail = dom_arr[-1][2]
            for ix in range(len(im_arr)):
                u = im_arr[ix][2]
                if ix > 0: ip[u] = im_arr[ix - 1][2]
                if ix + 1 < len(im_arr): inn[u] = im_arr[ix + 1][2]
            i_tail = im_arr[-1][2]

            p_in = (p_state[0] == e and p_state[3] is None)
            p_uid = -1; p_off = 0
            if p_in:
                for pos0, _, uid, _ in dom_arr:
                    if pos0 <= p_state[1] < pos0 + nl[uid]:
                        p_uid = uid; p_off = p_state[1] - pos0; break
                assert p_uid != -1

            def dun(u):
                nonlocal d_tail
                pr, nx = dp[u], dn[u]
                if pr != -1: dn[pr] = nx
                if nx != -1: dp[nx] = pr
                else: d_tail = pr
                dp[u] = dn[u] = -1
            def iun(u):
                nonlocal i_tail
                pr, nx = ip[u], inn[u]
                if pr != -1: inn[pr] = nx
                if nx != -1: ip[nx] = pr
                else: i_tail = pr
                ip[u] = inn[u] = -1
            def dins(anc, u):
                nonlocal d_tail
                nx = dn[anc]; dn[anc] = u; dp[u] = anc; dn[u] = nx
                if nx != -1: dp[nx] = u
                else: d_tail = u
            def iins_a(anc, u):
                nonlocal i_tail
                nx = inn[anc]; inn[anc] = u; ip[u] = anc; inn[u] = nx
                if nx != -1: ip[nx] = u
                else: i_tail = u

            while d_tail != -1:
                d = d_tail; i = i_tail
                if d == i:
                    if p_in and p_uid == d:
                        p_state[3] = (nw[d], p_state[2], nwl[d]); p_state[0] = -1; p_in = False
                    dun(d); iun(d); continue
                kd, ki = nk[d], nk[i]; ld, li = nl[d], nl[i]
                if kd == 1 and ki == 2:
                    tk = min(ld, li)
                    cw, cwl = cat_w(nw[i], nwl[i], nw[d], nwl[d])
                    emitted.append((neb[i], neb[d], nelo[i] + (li - tk), nelo[d] + (ld - tk), tk, cw, cwl))
                    if p_in and p_uid == d and p_off >= ld - tk:
                        p_state[0] = neb[i]; p_state[1] = nelo[i] + (li - tk) + (p_off - (ld - tk)); p_state[2] += nwl[i]; p_in = False
                    if ld == tk: dun(d)
                    else: nl[d] = ld - tk
                    if li == tk: iun(i)
                    else: nl[i] = li - tk
                elif ld > li:
                    if kd == 0:
                        if p_in and p_uid == d and p_off >= ld - li:
                            if ki == 2:
                                p_state[0] = neb[i]; p_state[1] = nelo[i] + (p_off - (ld - li)); p_state[2] += nwl[i]; p_in = False
                            else:
                                p_uid = i; p_off -= (ld - li); p_state[2] += nwl[i]
                        nw[i], nwl[i] = cat_w(nw[i], nwl[i], nw[d], nwl[d])
                        nl[d] -= li
                        if inn[d] != i: iun(i); iins_a(d, i)
                    else:
                        if p_in and p_uid == d and p_off >= ld - li:
                            p_uid = i; p_off -= (ld - li); p_state[2] += nwl[i]
                        nw[i], nwl[i] = cat_w(nw[i], nwl[i], nw[d], nwl[d])
                        nl[d] -= li; nk[i] = 1; neb[i] = neb[d]; nelo[i] = nelo[d] + (ld - li)
                        iun(i)
                elif li > ld:
                    if ki == 0:
                        if p_in and p_uid == d: p_state[2] += nwl[i]
                        nw[d], nwl[d] = cat_w(nw[i], nwl[i], nw[d], nwl[d])
                        nl[i] -= ld
                        if dn[i] != d: dun(d); dins(i, d)
                    else:
                        if p_in and p_uid == d:
                            p_state[0] = neb[i]; p_state[1] = nelo[i] + (li - ld) + p_off; p_state[2] += nwl[i]; p_in = False
                        nw[d], nwl[d] = cat_w(nw[i], nwl[i], nw[d], nwl[d])
                        nl[i] -= ld; nk[d] = 2; neb[d] = neb[i]; nelo[d] = nelo[i] + (li - ld)
                        dun(d)
                else:
                    if kd == 0 and ki == 0:
                        if p_in and p_uid == d: p_uid = i; p_state[2] += nwl[i]
                        nw[i], nwl[i] = cat_w(nw[i], nwl[i], nw[d], nwl[d])
                        dun(d); iun(i); iins_a(d, i); iun(d)
                    elif kd == 0 and ki == 2:
                        if p_in and p_uid == d:
                            p_state[0] = neb[i]; p_state[1] = nelo[i] + p_off; p_state[2] += nwl[i]; p_in = False
                        nw[i], nwl[i] = cat_w(nw[i], nwl[i], nw[d], nwl[d])
                        dun(d); iun(i); iins_a(d, i); iun(d)
                    elif kd == 1 and ki == 0:
                        if p_in and p_uid == d: p_uid = i; p_state[2] += nwl[i]
                        nw[i], nwl[i] = cat_w(nw[i], nwl[i], nw[d], nwl[d])
                        nk[i] = 1; neb[i] = neb[d]; nelo[i] = nelo[d]
                        dun(d); iun(i)
                    else:
                        if p_in and p_uid == d:
                            p_state[0] = neb[i]; p_state[1] = nelo[i] + p_off; p_state[2] += nwl[i]; p_in = False
                        nw[d], nwl[d] = cat_w(nw[i], nwl[i], nw[d], nwl[d])
                        nk[d] = 2; neb[d] = neb[i]; nelo[d] = nelo[i]
                        dun(d); iun(i)

        for sr, dr, dlo, ilo, ln, wd, wl in emitted:
            route(sr, dr, dlo, ilo, ln, wd, wl)

    assert p_state[3] is not None, "Primary cycle never captured!"
    cyc_wd, p_sh, cyc_len = p_state[3]
    cyc = expand_w(cyc_wd)
    assert len(cyc) == cyc_len
    sh = p_sh % cyc_len
    rot = cyc[sh:] + cyc[:sh] if sh else cyc
    return rot, {
        "q0": q0, "writes": writes, "reads": reads, "stale": stale,
        "peak_buf": peak_buf, "peak_mid": peak_total_midstep,
    }


def main():
    rng = random.Random(2026)

    print("=== 2.1 EXHAUSTIVE ADVERSARIAL AUDIT OF THEOREM B4 ON ARBITRARY BINARY (n<=8) & TERNARY (n<=6) ===", flush=True)
    bin_ok = 0; tern_ok = 0; mid_exceeds_q0 = None
    beta_bugs = []
    # Binary n=1..8 all L, all p, both orders
    for n in range(1, 9):
        for mask in range(1 << n):
            L = [(mask >> i) & 1 for i in range(n)]
            runs = mb.rle_from_bytes(L)
            syms = sorted(set(L))
            for p in range(n):
                ref = ref_lf_cycle(L, p)
                for order in ([syms] if len(syms) == 1 else [syms, list(reversed(syms))]):
                    got, st = independent_b4_audit(runs, p, order, aht_thresh=2)
                    assert got == ref and st["stale"] == 0 and st["writes"] == st["reads"] and st["peak_buf"] <= st["q0"]
                    if st["peak_mid"] > st["q0"] and mid_exceeds_q0 is None:
                        mid_exceeds_q0 = ("bin", L, p, order, st)
                    bin_ok += 1
    # Ternary n=1..6 all L, random p + random static rank order (plus all p for n<=5!)
    for n in range(1, 7):
        for code in range(3 ** n):
            L = []; v = code
            for _ in range(n): L.append(v % 3); v //= 3
            runs = mb.rle_from_bytes(L)
            syms = sorted(set(L))
            p_list = range(n) if n <= 5 else [0, n // 2, n - 1]
            for p in p_list:
                ref = ref_lf_cycle(L, p)
                order = list(syms)
                rng.shuffle(order)
                got, st = independent_b4_audit(runs, p, order, aht_thresh=2)
                assert got == ref and st["stale"] == 0 and st["writes"] == st["reads"] and st["peak_buf"] <= st["q0"]
                if st["peak_mid"] > st["q0"] and mid_exceeds_q0 is None:
                    mid_exceeds_q0 = ("tern", L, p, order, st)
                tern_ok += 1
    # Also test Beta's pareto_superbucket on unary, periodic, and multi-cycle ternary L=[1,0,2,2]!
    for lbl, L, p in [
        ("unary a^4", [0, 0, 0, 0], 0),
        ("periodic (ab)^2", [1, 1, 0, 0], 0),
        ("multi-cycle [1,0,2,2] p=0", [1, 0, 2, 2], 0),
        ("multi-cycle [1,0,2,2] p=2", [1, 0, 2, 2], 2),
    ]:
        runs = mb.rle_from_bytes(L)
        ref = ref_lf_cycle(L, p)
        try:
            st_b = ps.run_superbucket_zero_tombstone(runs, p, K=16, strategy="freq_tiered_hub_last", aht_thresh=2, verify_seq=None)
            print(f"  Beta pareto_superbucket on {lbl:26s}: OK (buf_reads={st_b['buf_reads']}, peak_buf={st_b['peak_live_buf']})", flush=True)
        except Exception as ex:
            beta_bugs.append((lbl, f"{type(ex).__name__}: {ex}"))
            print(f"  Beta pareto_superbucket on {lbl:26s}: CRASH {type(ex).__name__}: {ex}", flush=True)

    print(f"  PASS! Binary={bin_ok}, Ternary={tern_ok} (ALL 0% stale, writes==reads, peak_buf<=q0, 100% bit-exact!)", flush=True)
    print(f"  Mid-step peak_mid > q0 witness (during AHT splits): {mid_exceeds_q0}", flush=True)

    print("\n=== 2.2 ADVERSARIAL CONSTANT-r KILLERS + Fib_20+$ UNDER THEOREM B4 ===", flush=True)
    db2 = epr.de_bruijn_binary(4)
    # Build Fib_20 + '$' (2)
    f0, f1 = [0], [0, 1]
    while len(f1) < 6765: f0, f1 = f1, f1 + f0
    fib20 = f1 + [2]
    killers = [
        ("3-phase K=128", [0, 1]*128 + [2] + [0, 0, 1]*128 + [2] + [0, 1, 1]*128 + [2]),
        ("AltRing k=128", [0, 1]*128 + [1, 2]*128 + [2, 0]*128),
        ("Low-r Padded-Ring K=64", [0 if x==0 else 1 for x in db2 for _ in range(64)] + [1 if x==0 else 2 for x in db2 for _ in range(64)] + [2 if x==0 else 0 for x in db2 for _ in range(64)]),
        ("SymMidRun K=128", [1]*128 + [0] + [1]*128 + [2]),
        ("Fib_20+$", fib20),
    ]
    for name, T in killers:
        L, p = epr.bwt_encode_fast(T)
        runs = mb.rle_from_bytes(L)
        r0 = len(runs)
        syms = sorted(set(L))
        # Test ALL 6 static rank orders!
        max_mid = 0; max_wr = 0
        for perm in itertools.permutations(syms):
            got, st = independent_b4_audit(runs, p, list(perm), aht_thresh=2)
            assert got == T and st["stale"] == 0 and st["peak_buf"] <= st["q0"]
            if st["peak_mid"] > max_mid: max_mid = st["peak_mid"]
            if st["writes"] > max_wr: max_wr = st["writes"]
        print(f"  [{name:22s}] n={len(T):5d} r0={r0:2d} q0={st['q0']:2d} | ALL {len(list(itertools.permutations(syms)))} orders EXACT, 0% stale | max_wr={max_wr:3d}, peak_buf<={st['q0']}, max_peak_mid={max_mid:3d} ({max_mid/st['q0']:.2f}q0)", flush=True)

    print("\n=== 2.3 INDEPENDENT VERIFICATION ON block-0.L & block-210.L ===", flush=True)
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    for blk in manifest["blocks"]:
        if blk["file"] not in ("block-0.L", "block-210.L"):
            continue
        raw = open(os.path.join("/tmp/mibz2-blocks", blk["file"]), "rb").read()
        assert hashlib.sha256(raw).hexdigest() == blk["sha256"]
        n, p, r0 = blk["n"], blk["primary_index"], blk["r"]
        runs = mb.rle_from_bytes(raw)
        # Static order: rarest_first_rows (matching Beta's freq_tiered_hub_last K=|Sigma|)
        cnt = defaultdict(int)
        for c, w in runs: cnt[c] += w
        order = sorted(cnt.keys(), key=lambda c: (cnt[c], c))
        t0 = time.time()
        got, st = independent_b4_audit(runs, p, order, aht_thresh=2)
        dt = time.time() - t0
        ref = mb.decode_lf_reference_bytes(raw, p)
        assert bytes(got) == ref, f"EXACTNESS FAIL on {blk['file']}"
        print(
            f"  {blk['file']:11s} (r0={r0:6d}, q0={st['q0']:6d}, {dt:.2f}s) | EXACT! | "
            f"writes=reads={st['writes']:6d} ({st['writes']/r0:.3f} r0) | stale={st['stale']} (0.00%) | "
            f"peak_buf={st['peak_buf']:6d} ({st['peak_buf']/st['q0']:.4f} q0) | "
            f"peak_mid={st['peak_mid']:6d} ({st['peak_mid']/st['q0']:.4f} q0)",
            flush=True,
        )


if __name__ == "__main__":
    main()
