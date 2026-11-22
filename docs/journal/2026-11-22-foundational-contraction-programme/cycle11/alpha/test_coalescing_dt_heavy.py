#!/usr/bin/env python3
"""
Cycle 11 — Part 2:
S2_dt_heavy + Free Gap-Closing Coalescence & Coalescence-Seeking Scheduling on:
  1. ALL 6 authenticated Wikidata blocks (/tmp/mibz2-blocks/block-*.L),
  2. Large repetitive z(T) << r_0 families:
     - Thue-Morse T_6..T_12 (n = 64 .. 4,096)
     - Fibonacci F_18+$ (n = 4,182)
     - Period-Doubling PD_6..PD_12 (n = 64 .. 4,096)
     - Random-Copy Repetitive Strings S * 8 + sparse edits (where r_0 / z ~ 5-10!)

Tests:
  - S2_dt_heavy (no coalescence, baseline = 1.587 r_0 on Wikidata)
  - S2_dt_heavy + Immediate Gap-Closing Coalescence (in compressed coordinates with hash-consed words!)
  - Coalescence-First + Heavy-Tiebreak (prioritize moves triggering immediate coalescence!)
Measures:
  - total_ops / r_0, exact Delta-P step histogram {-5,-4,-3,-2,-1,0,+1},
  - hash-consed unique SLP size G_dedup / r_0, and comparison vs exact LZ77 z(T)!
"""
import hashlib
import heapq
import json
import random
import sys
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import bwt_core as bc


class HashConsWordStore:
    """Exact hash-consed RLSLP so identical composite words receive the exact same integer ID!"""
    __slots__ = ("kind", "a", "b", "length", "concat_tbl", "pow_tbl", "inline")

    def __init__(self):
        self.kind = [0] * 256
        self.a = list(range(256))
        self.b = [0] * 256
        self.length = [1] * 256
        self.inline = [bytes([c]) for c in range(256)]
        self.concat_tbl = {}
        self.pow_tbl = {}

    def concat(self, u, v):
        key = (u, v)
        got = self.concat_tbl.get(key)
        if got is not None:
            return got
        lu, lv = self.length[u], self.length[v]
        idx = len(self.kind)
        if lu + lv <= 16 and self.kind[u] == 0 and self.kind[v] == 0:
            self.kind.append(0); self.a.append(0); self.b.append(0)
            self.inline.append(self.inline[u] + self.inline[v])
        else:
            self.kind.append(1); self.a.append(u); self.b.append(v)
            self.inline.append(b"")
        self.length.append(lu + lv)
        self.concat_tbl[key] = idx
        return idx

    def power(self, u, k):
        if k == 1:
            return u
        key = (u, k)
        got = self.pow_tbl.get(key)
        if got is not None:
            return got
        lu = self.length[u]
        idx = len(self.kind)
        if lu * k <= 16 and self.kind[u] == 0:
            self.kind.append(0); self.a.append(0); self.b.append(0)
            self.inline.append(self.inline[u] * k)
        else:
            self.kind.append(2); self.a.append(u); self.b.append(k)
            self.inline.append(b"")
        self.length.append(lu * k)
        self.pow_tbl[key] = idx
        return idx

    def expand(self, nid):
        if self.kind[nid] == 0:
            return self.inline[nid]
        out = bytearray()
        st = [(nid, 1)]
        while st:
            u, mult = st.pop()
            k = self.kind[u]
            if k == 0:
                b = self.inline[u]
                out.extend(b if mult == 1 else b * mult)
            elif k == 1:
                if mult == 1:
                    st.append((self.b[u], 1)); st.append((self.a[u], 1))
                else:
                    out.extend(self.expand(u) * mult)
            else:
                st.append((self.a[u], self.b[u] * mult))
        return bytes(out)


def fold_with_coalescence(L_bytes: bytes, primary_idx: int, mode="dt_heavy_coal"):
    """
    Word-carrying Dunfield-Thurston Heaviest-First Train-Track Folding WITH
    EXACT Dynamic Gap-Closing Coalescence (domain + codomain doubly-linked lists)!

    Because we maintain BOTH:
      - dom_prev[u], dom_next[u]: exact circular/linear domain neighbor links in compressed coords!
      - cod_prev[u], cod_next[u]: exact codomain (target) neighbor links in compressed coords!
    Two live strips (u, v) are ADJACENT IN BOTH COMPRESSED DOMAIN AND COMPRESSED CODOMAIN iff:
      dom_next[u] == v  AND  cod_next[u] == v  AND  word_id[u] == word_id[v]  AND  unmarked!
    WAIT A SECOND!! CHECK THAT!!
    In compressed coordinates (after closing ALL deleted gaps!):
      - `dom_next[u] == v` means I_u and I_v are immediately adjacent in compressed domain!
      - `cod_next[u] == v` means J_u and J_v are immediately adjacent in compressed codomain!
      - Therefore `dom_next[u] == v AND cod_next[u] == v` is 100% EQUIVALENT to
        "u and v have identical compressed delta AND are adjacent in compressed domain" —
        IN O(1) TIME WITH ZERO COORDINATE RE-INDEXING!!

    Check: When we coalesce (u, v) into u:
      - u absorbs v's width (w_u += w_v), inherits dom_next[v], cod_next[v],
      - out_adj[u] = merge(out_adj[u], out_adj[v]) (deduplicating boundary overlap!),
      - in_adj[u]  = merge(in_adj[u], in_adj[v])!
      - And we check for cascading coalescences cleanly!
    """
    n, r0, m16, st_tup, j_order, _ = bc.load_runs_and_open_baseline(L_bytes, primary_idx)
    ws = HashConsWordStore()
    q0 = len(st_tup)
    if q0 == 0:
        return b"", {}

    # Strip fields: [width, word_id, mark_off, alive]
    # Exact positions are tracked implicitly by ordered adjacency + widths!
    # Wait: to keep exact overlap lengths cleanly without coordinate drift, let's track
    # `dom_prev`, `dom_next`, `cod_prev`, `cod_next` AND exact overlap widths on edges?
    # Even simpler & 100% bug-free: track each live strip's `(lo, hi, delta)` in ORIGINAL coordinates
    # for overlap math, AND track `dom_prev/next` & `cod_prev/next` for O(1) gap-closed coalescence!
    # WAIT: if we coalesce u and v across a deleted gap, their original coordinates have a gap between them!
    # So instead, store explicit ordered `out_edges[u] = [(v, ov_width), ...]` and `in_edges[v] = [(u, ov_width), ...]`!
    # OH!! THAT IS SO MUCH CLEANER — PURE WEIGHTED TRAIN-TRACK REPRESENTATION (ZERO COORDINATES AT ALL)!!
    # Check:
    #   - Every strip u has: width[u], word[u], mark[u], alive[u]
    #   - `out_e[u]` = ordered list of [v, ov_w] summing to width[u]!
    #   - `in_e[v]`  = ordered list of [u, ov_w] summing to width[v]!
    #   - `dom_prev[u], dom_next[u]` (domain order of live strips!)
    #   - `cod_prev[u], cod_next[u]` (codomain order of live strips!)
    #   - Coalesce(u, v) whenever dom_next[u] == v and cod_next[u] == v and word[u] == word[v] and mark[u] < 0 and mark[v] < 0!
    #   - Self-overlap spiral on u: u appears in out_e[u] as [u, ov_w] ->
    #       if ov_w == width[u]: closed cycle (d == 0)!
    #       if ov_w < width[u]: wait — in weighted train track, if out_e[u] starts or ends with [u, ov_w],
    #       let d = width[u] - ov_w; exact Euclidean spiral q, rem = divmod(width[u], d)!!

    width = [hi - lo for lo, hi, _, _, _ in st_tup]
    word = [c for _, _, _, c, _ in st_tup]
    mark = [m for _, _, _, _, m in st_tup]
    alive = [True] * q0

    dom_prev = [i - 1 for i in range(q0)]
    dom_next = [i + 1 if i + 1 < q0 else -1 for i in range(q0)]
    cod_prev = [-1] * q0
    cod_next = [-1] * q0
    for idx, u in enumerate(j_order):
        if idx > 0: cod_prev[u] = j_order[idx - 1]
        if idx + 1 < q0: cod_next[u] = j_order[idx + 1]

    out_e = [[] for _ in range(q0)]
    in_e = [[] for _ in range(q0)]
    ii = 0; jj = 0
    while ii < q0 and jj < q0:
        v = ii; u = j_order[jj]
        v_lo, v_hi = st_tup[v][0], st_tup[v][1]
        u_tlo, u_thi = st_tup[u][0] + st_tup[u][2], st_tup[u][1] + st_tup[u][2]
        if u_thi <= v_lo: jj += 1
        elif v_hi <= u_tlo: ii += 1
        else:
            ov = min(u_thi, v_hi) - max(u_tlo, v_lo)
            out_e[u].append([v, ov])
            in_e[v].append([u, ov])
            if u_thi < v_hi: jj += 1
            elif v_hi < u_thi: ii += 1
            else: ii += 1; jj += 1

    def unlink_dom(u):
        p, nx = dom_prev[u], dom_next[u]
        if p >= 0: dom_next[p] = nx
        if nx >= 0: dom_prev[nx] = p
        dom_prev[u] = dom_next[u] = -1
        return p, nx

    def unlink_cod(u):
        p, nx = cod_prev[u], cod_next[u]
        if p >= 0: cod_next[p] = nx
        if nx >= 0: cod_prev[nx] = p
        cod_prev[u] = cod_next[u] = -1
        return p, nx

    def replace_dom(old_u, new_list):
        """Replace old_u in dom linked list with ordered new_list (non-empty)."""
        p, nx = dom_prev[old_u], dom_next[old_u]
        prev = p
        for x in new_list:
            dom_prev[x] = prev
            if prev >= 0: dom_next[prev] = x
            prev = x
        dom_next[prev] = nx
        if nx >= 0: dom_prev[nx] = prev
        if old_u not in new_list:
            dom_prev[old_u] = dom_next[old_u] = -1

    def replace_cod(old_u, new_list):
        p, nx = cod_prev[old_u], cod_next[old_u]
        prev = p
        for x in new_list:
            cod_prev[x] = prev
            if prev >= 0: cod_next[prev] = x
            prev = x
        cod_next[prev] = nx
        if nx >= 0: cod_prev[nx] = prev
        if old_u not in new_list:
            cod_prev[old_u] = cod_next[old_u] = -1

    def replace_edge_list(elist, old_id, new_pairs):
        """Replace occurrence of [old_id, _] in elist with new_pairs, merging adjacent same-id entries."""
        idx = next(i for i, e in enumerate(elist) if e[0] == old_id)
        elist[idx:idx + 1] = new_pairs
        # Merge adjacent identical IDs around idx
        lo = max(0, idx - 1)
        hi = min(len(elist), idx + len(new_pairs) + 1)
        i = lo
        while i + 1 < len(elist) and i < hi:
            if elist[i][0] == elist[i + 1][0]:
                elist[i][1] += elist[i + 1][1]
                elist.pop(i + 1)
                hi -= 1
            else:
                i += 1

    use_coal = (mode != "dt_heavy_no_coal")
    live_cnt = q0
    dp_hist = defaultdict(int)
    stats = defaultdict(int)
    stats["r0"] = r0
    closed_primary = None

    heap = []

    def is_closed_or_spiral(u):
        if not alive[u]: return False
        for v, ov in out_e[u]:
            if v == u:
                return True
        return False

    def is_r1(u):
        if not alive[u] or len(out_e[u]) != 1: return False
        v = out_e[u][0][0]
        return v != u and alive[v] and len(in_e[v]) == 1 and in_e[v][0][0] == u and not is_closed_or_spiral(v)

    def is_r2(v):
        if not alive[v] or not in_e[v] or is_closed_or_spiral(v): return False
        for u, _ in in_e[v]:
            if u == v or not alive[u] or len(out_e[u]) != 1: return False
        return True

    def is_r3(u):
        if not alive[u] or len(out_e[u]) != 1: return False
        v = out_e[u][0][0]
        if v == u or not alive[v] or is_closed_or_spiral(v): return False
        return (in_e[v][0][0] == u) or (in_e[v][-1][0] == u)

    def is_r3s(v):
        if not alive[v] or len(in_e[v]) != 1 or is_closed_or_spiral(v): return False
        u = in_e[v][0][0]
        if u == v or not alive[u] or is_closed_or_spiral(u): return False
        return (out_e[u][0][0] == v and width[v] < width[u]) or (out_e[u][-1][0] == v and width[v] < width[u])

    def estimate_coal_bonus(u):
        """Quick O(1) heuristic for Coalescence-First: does firing R1 on u immediately trigger >=1 coalescence?"""
        if not is_r1(u):
            return 0
        v = out_e[u][0][0]
        bonus = 0
        # Domain-gap coalescence across deleted v?
        pl, pr = dom_prev[v], dom_next[v]
        if pl >= 0 and pr >= 0 and pl != u and pr != u and word[pl] == word[pr] and cod_next[pl] == pr:
            bonus += 1
        # Codomain-gap coalescence across deleted v?
        ql, qr = cod_prev[v], cod_next[v]
        if ql >= 0 and qr >= 0 and ql != u and qr != u and word[ql] == word[qr] and dom_next[ql] == qr:
            bonus += 1
        # Site-A coalescence of u' = u o v with dom_prev[u] or dom_next[u]?
        cand_w = ws.concat_tbl.get((word[u], word[v]))
        if cand_w is not None:
            al, ar = dom_prev[u], dom_next[u]
            if al >= 0 and word[al] == cand_w and cod_next[al] == v:
                bonus += 1
            if ar >= 0 and word[ar] == cand_w and cod_prev[ar] == v:
                bonus += 1
        return bonus

    def push_c(u):
        if u < 0 or not alive[u]: return
        if is_closed_or_spiral(u):
            heapq.heappush(heap, (0, 0, u))
            return
        if not (is_r1(u) or is_r2(u) or is_r3(u) or is_r3s(u)):
            return
        if mode == "coal_first":
            bon = estimate_coal_bonus(u)
            if bon > 0:
                heapq.heappush(heap, (1, -bon * 1000000 - width[u], u))
            else:
                heapq.heappush(heap, (2, -width[u], u))
        else:
            heapq.heappush(heap, (1, -width[u], u))

    def try_coalesce_pair(u, v):
        """If live u, v satisfy dom_next[u]==v AND cod_next[u]==v AND word[u]==word[v] AND unmarked, merge v into u!"""
        nonlocal live_cnt
        if not use_coal or u < 0 or v < 0 or u == v or not alive[u] or not alive[v]:
            return False
        if dom_next[u] != v or cod_next[u] != v:
            return False
        if word[u] != word[v] or mark[u] >= 0 or mark[v] >= 0:
            return False
        # Merge v into u!
        width[u] += width[v]
        alive[v] = False
        live_cnt -= 1
        stats["coalesced"] += 1
        # Unlink v from dom and cod
        unlink_dom(v)
        unlink_cod(v)
        # Merge out_e[u] + out_e[v] (rewriting v -> u in every target's in_e, including if nxt == u or nxt == v!)
        v_outs = list(out_e[v])
        v_ins = list(in_e[v])
        out_e[v] = []; in_e[v] = []
        # Append v_outs to out_e[u] (replacing any reference to v by u!)
        for nxt, ov in v_outs:
            rnxt = u if nxt == v else nxt
            if out_e[u] and out_e[u][-1][0] == rnxt:
                out_e[u][-1][1] += ov
            else:
                out_e[u].append([rnxt, ov])
        # Replace any reference to v inside out_e[u] itself (if u had an edge to v!)
        i_o = 0
        while i_o < len(out_e[u]):
            if out_e[u][i_o][0] == v:
                out_e[u][i_o][0] = u
            if i_o > 0 and out_e[u][i_o - 1][0] == out_e[u][i_o][0]:
                out_e[u][i_o - 1][1] += out_e[u][i_o][1]
                out_e[u].pop(i_o)
            else:
                i_o += 1
        # Update in_e of all external targets of v
        for nxt, ov in v_outs:
            if nxt != u and nxt != v:
                replace_edge_list(in_e[nxt], v, [[u, ov]])
                push_c(nxt)
        # Append v_ins to in_e[u] (replacing any reference to v by u!)
        for prev, ov in v_ins:
            rprev = u if prev == v else prev
            if in_e[u] and in_e[u][-1][0] == rprev:
                in_e[u][-1][1] += ov
            else:
                in_e[u].append([rprev, ov])
        i_i = 0
        while i_i < len(in_e[u]):
            if in_e[u][i_i][0] == v:
                in_e[u][i_i][0] = u
            if i_i > 0 and in_e[u][i_i - 1][0] == in_e[u][i_i][0]:
                in_e[u][i_i - 1][1] += in_e[u][i_i][1]
                in_e[u].pop(i_i)
            else:
                i_i += 1
        # Update out_e of all external sources of v
        for prev, ov in v_ins:
            if prev != u and prev != v:
                replace_edge_list(out_e[prev], v, [[u, ov]])
                push_c(prev)
        push_c(u)
        return True

    def coalesce_around(cands):
        """Greedily fire all triggered coalescences around candidate nodes."""
        if not use_coal:
            return 0
        q_c = list(cands)
        cnt = 0
        while q_c:
            x = q_c.pop()
            if x < 0 or not alive[x]: continue
            # Check (dom_prev[x], x) and (x, dom_next[x])
            p = dom_prev[x]
            if p >= 0 and try_coalesce_pair(p, x):
                cnt += 1; q_c.append(p); continue
            nx = dom_next[x]
            if nx >= 0 and try_coalesce_pair(x, nx):
                cnt += 1; q_c.append(x); continue
        return cnt

    for u in range(q0):
        push_c(u)

    while heap:
        _, _, cand = heapq.heappop(heap)
        if not alive[cand]:
            continue
        p_before = live_cnt

        # 1. Closed cycle or Spiral
        if is_closed_or_spiral(cand):
            u = cand
            ov_self = next(ov for v, ov in out_e[u] if v == u)
            if ov_self == width[u]:
                if mark[u] >= 0:
                    closed_primary = (word[u], mark[u])
                alive[u] = False; live_cnt -= 1
                dp_l, dp_r = unlink_dom(u)
                cp_l, cp_r = unlink_cod(u)
                out_e[u] = []; in_e[u] = []
                stats["spirals"] += 1
                coalesce_around([dp_l, dp_r, cp_l, cp_r])
                dp_hist[live_cnt - p_before] += 1
                continue
            # Self-overlap spiral: width = L, overlap = ov_self, shift d = L - ov_self
            L = width[u]; d = L - ov_self
            # Check direction: is [u, ov_self] at the LEFT of out_e[u] (d > 0) or RIGHT (d < 0)?
            is_pos = (out_e[u][0][0] == u)
            # Wait: check exact order! If d > 0, J_u = [lo+d, hi+d), so I_u overlaps J_u on I_u's RIGHT [lo+d, hi)
            # and J_u's LEFT [lo+d, hi)! So out_e[u][0] == [u, ov_self] and in_e[u][-1] == [u, ov_self]!
            q_sp, rem = divmod(L, d)
            stats["spirals"] += 1
            other_out = [e for e in out_e[u] if e[0] != u]
            other_in = [e for e in in_e[u] if e[0] != u]
            new_dom = []
            new_cod = []
            if is_pos:
                # Entry [0, d) splits into p1 (width rem, w^{q+1}) then p2 (width d-rem, w^q) in dom order;
                # in cod order (exit window [0, d)): p2 (width d-rem) comes BEFORE p1 (width rem)!
                p1 = p2 = -1
                if rem > 0:
                    p1 = len(width); width.append(rem); word.append(ws.power(word[u], q_sp + 1)); mark.append(-1); alive.append(True)
                    dom_prev.append(-1); dom_next.append(-1); cod_prev.append(-1); cod_next.append(-1)
                    out_e.append([]); in_e.append([])
                if d - rem > 0:
                    p2 = len(width); width.append(d - rem); word.append(ws.power(word[u], q_sp)); mark.append(-1); alive.append(True)
                    dom_prev.append(-1); dom_next.append(-1); cod_prev.append(-1); cod_next.append(-1)
                    out_e.append([]); in_e.append([])
                new_dom = [x for x in (p1, p2) if x >= 0]
                new_cod = [x for x in (p2, p1) if x >= 0]
            else:
                # d < 0: entry is right end of I_u -> p2 (width d-rem, w^q) then p1 (width rem, w^{q+1}) in dom order;
                # in cod order: p1 then p2!
                p1 = p2 = -1
                if d - rem > 0:
                    p2 = len(width); width.append(d - rem); word.append(ws.power(word[u], q_sp)); mark.append(-1); alive.append(True)
                    dom_prev.append(-1); dom_next.append(-1); cod_prev.append(-1); cod_next.append(-1)
                    out_e.append([]); in_e.append([])
                if rem > 0:
                    p1 = len(width); width.append(rem); word.append(ws.power(word[u], q_sp + 1)); mark.append(-1); alive.append(True)
                    dom_prev.append(-1); dom_next.append(-1); cod_prev.append(-1); cod_next.append(-1)
                    out_e.append([]); in_e.append([])
                new_dom = [x for x in (p2, p1) if x >= 0]
                new_cod = [x for x in (p1, p2) if x >= 0]

            alive[u] = False
            live_cnt += len(new_dom) - 1
            replace_dom(u, new_dom)
            replace_cod(u, new_cod)

            # Rewire other_in (total width d) onto new_dom (total width d) via linear merge!
            i_in = 0; rem_in = other_in[0][1] if other_in else 0
            i_nd = 0; rem_nd = width[new_dom[0]]
            repl_in = defaultdict(list)
            while i_in < len(other_in) and i_nd < len(new_dom):
                src = other_in[i_in][0]; dst = new_dom[i_nd]
                tk = rem_in if rem_in < rem_nd else rem_nd
                in_e[dst].append([src, tk]); repl_in[src].append([dst, tk])
                rem_in -= tk; rem_nd -= tk
                if rem_in == 0:
                    i_in += 1
                    if i_in < len(other_in): rem_in = other_in[i_in][1]
                if rem_nd == 0:
                    i_nd += 1
                    if i_nd < len(new_dom): rem_nd = width[new_dom[i_nd]]
            for src, rlist in repl_in.items():
                replace_edge_list(out_e[src], u, rlist); push_c(src)

            # Rewire new_cod (total width d) onto other_out (total width d) via linear merge!
            i_nc = 0; rem_nc = width[new_cod[0]]
            i_ot = 0; rem_ot = other_out[0][1] if other_out else 0
            repl_ot = defaultdict(list)
            while i_nc < len(new_cod) and i_ot < len(other_out):
                src = new_cod[i_nc]; dst = other_out[i_ot][0]
                tk = rem_nc if rem_nc < rem_ot else rem_ot
                out_e[src].append([dst, tk]); repl_ot[dst].append([src, tk])
                rem_nc -= tk; rem_ot -= tk
                if rem_nc == 0:
                    i_nc += 1
                    if i_nc < len(new_cod): rem_nc = width[new_cod[i_nc]]
                if rem_ot == 0:
                    i_ot += 1
                    if i_ot < len(other_out): rem_ot = other_out[i_ot][1]
            for dst, rlist in repl_ot.items():
                replace_edge_list(in_e[dst], u, rlist); push_c(dst)

            out_e[u] = []; in_e[u] = []
            coalesce_around(new_dom + new_cod)
            for x in new_dom: push_c(x)
            dp_hist[live_cnt - p_before] += 1
            continue

        # 2. Smooth (1,1) fold R1
        if is_r1(cand):
            u = cand; v = out_e[u][0][0]
            wu_len = ws.length[word[u]]
            word[u] = ws.concat(word[u], word[v])
            mark[u] = bc.combine_mark(mark[u], wu_len, mark[v])
            alive[v] = False; live_cnt -= 1
            # u takes v's codomain position! v's old domain position is deleted; u's old codomain position (= v's dom!) is deleted!
            dp_l, dp_r = unlink_dom(v)
            cp_l, cp_r = unlink_cod(u)
            replace_cod(v, [u])
            v_outs = out_e[v]
            out_e[u] = [list(e) for e in v_outs]
            for nxt, ov in v_outs:
                replace_edge_list(in_e[nxt], v, [[u, ov]])
                push_c(nxt)
            out_e[v] = []; in_e[v] = []
            stats["smooth_11"] += 1; stats["total_ops"] += 1
            coalesce_around([u, dp_l, dp_r, cp_l, cp_r, dom_prev[u], dom_next[u]])
            push_c(u)
            for prev, _ in in_e[u]: push_c(prev)
            dp_hist[live_cnt - p_before] += 1
            continue

        # 3. Full-branch Stallings fold R2 (delete v, all u in in_e[v] have out_deg == 1)
        if is_r2(cand):
            v = cand; vw = word[v]; vm = mark[v]
            v_ins = [u for u, _ in in_e[v]]
            v_outs = [list(e) for e in out_e[v]]
            alive[v] = False; live_cnt -= 1
            dp_l, dp_r = unlink_dom(v)
            # In codomain order, the contiguous block v_ins (which tiled I_v) moves to v's codomain slot!
            cp_l = cod_prev[v_ins[0]]
            cp_r = cod_next[v_ins[-1]]
            if cp_l >= 0: cod_next[cp_l] = cp_r
            if cp_r >= 0: cod_prev[cp_r] = cp_l
            replace_cod(v, v_ins)

            for u in v_ins:
                wu_len = ws.length[word[u]]
                word[u] = ws.concat(word[u], vw)
                mark[u] = bc.combine_mark(mark[u], wu_len, vm)
                out_e[u] = []
                stats["full_fold"] += 1; stats["total_ops"] += 1

            # Linear merge v_ins onto v_outs
            i_u = 0; rem_u = width[v_ins[0]]
            i_w = 0; rem_w = v_outs[0][1]
            repl_w = defaultdict(list)
            while i_u < len(v_ins) and i_w < len(v_outs):
                u = v_ins[i_u]; nxt = v_outs[i_w][0]
                tk = rem_u if rem_u < rem_w else rem_w
                out_e[u].append([nxt, tk]); repl_w[nxt].append([u, tk])
                rem_u -= tk; rem_w -= tk
                if rem_u == 0:
                    i_u += 1
                    if i_u < len(v_ins): rem_u = width[v_ins[i_u]]
                if rem_w == 0:
                    i_w += 1
                    if i_w < len(v_outs): rem_w = v_outs[i_w][1]
            for nxt, rlist in repl_w.items():
                replace_edge_list(in_e[nxt], v, rlist); push_c(nxt)
            out_e[v] = []; in_e[v] = []
            coalesce_around(v_ins + [dp_l, dp_r, cp_l, cp_r])
            for u in v_ins:
                push_c(u)
                for prev, _ in in_e[u]: push_c(prev)
            dp_hist[live_cnt - p_before] += 1
            continue

        # 4. Partial-branch band split R3
        if is_r3(cand):
            u = cand; v = out_e[u][0][0]
            is_prefix = (in_e[v][0][0] == u)
            ln = width[u]
            wu_len = ws.length[word[u]]
            word[u] = ws.concat(word[u], word[v])
            mark[u] = bc.combine_mark(mark[u], wu_len, -1)
            width[v] -= ln
            if is_prefix: in_e[v].pop(0)
            else: in_e[v].pop()
            # Codomain update: u moves to sit immediately BEFORE v (if prefix) or AFTER v (if suffix)!
            cp_l, cp_r = unlink_cod(u)
            if is_prefix:
                replace_cod(v, [u, v])
            else:
                replace_cod(v, [v, u])
            out_e[u] = []
            rem = ln
            touched_nxts = []
            if is_prefix:
                while rem > 0:
                    nxt, ov = out_e[v][0]
                    touched_nxts.append(nxt)
                    if ov <= rem:
                        out_e[u].append([nxt, ov]); out_e[v].pop(0)
                        replace_edge_list(in_e[nxt], v, [[u, ov]]); rem -= ov
                    else:
                        out_e[u].append([nxt, rem]); out_e[v][0][1] -= rem
                        replace_edge_list(in_e[nxt], v, [[u, rem], [v, ov - rem]]); rem = 0
            else:
                rev_u = []
                while rem > 0:
                    nxt, ov = out_e[v][-1]
                    touched_nxts.append(nxt)
                    if ov <= rem:
                        rev_u.append([nxt, ov]); out_e[v].pop()
                        replace_edge_list(in_e[nxt], v, [[u, ov]]); rem -= ov
                    else:
                        rev_u.append([nxt, rem]); out_e[v][-1][1] -= rem
                        replace_edge_list(in_e[nxt], v, [[v, ov - rem], [u, rem]]); rem = 0
                out_e[u] = list(reversed(rev_u))
            for nxt in touched_nxts:
                push_c(nxt)
            stats["band_spl"] += 1; stats["total_ops"] += 1
            coalesce_around([u, v, cp_l, cp_r, dom_prev[u], dom_next[u], dom_prev[v], dom_next[v]])
            push_c(u); push_c(v)
            for prev, _ in in_e[u]: push_c(prev)
            if in_e[v]: push_c(in_e[v][0][0]); push_c(in_e[v][-1][0])
            dp_hist[live_cnt - p_before] += 1
            continue

        # 5. Dual band split R3*
        if is_r3s(cand):
            v = cand; u = in_e[v][0][0]
            is_prefix = (out_e[u][0][0] == v)
            ln = width[v]
            alive[v] = False
            u_abs = len(width)
            wu_len = ws.length[word[u]]
            width.append(ln); word.append(ws.concat(word[u], word[v]))
            mark.append(bc.combine_mark(-1, wu_len, mark[v])); alive.append(True)
            dom_prev.append(-1); dom_next.append(-1); cod_prev.append(-1); cod_next.append(-1)
            out_e.append([list(e) for e in out_e[v]]); in_e.append([])
            width[u] -= ln
            if is_prefix: out_e[u].pop(0)
            else: out_e[u].pop()
            # Domain update: delete v, split u -> [u_abs, u] (if prefix) or [u, u_abs] (if suffix)
            dp_l, dp_r = unlink_dom(v)
            if is_prefix: replace_dom(u, [u_abs, u])
            else: replace_dom(u, [u, u_abs])
            # Codomain update: u_abs takes v's codomain slot! (u stays in its shrunk codomain slot!)
            replace_cod(v, [u_abs])
            for nxt, ov in out_e[u_abs]:
                replace_edge_list(in_e[nxt], v, [[u_abs, ov]]); push_c(nxt)
            out_e[v] = []; in_e[v] = []
            # Split in_e[u] by ln from left or right
            rem = ln
            touched_prevs = []
            if is_prefix:
                while rem > 0:
                    prev, ov = in_e[u][0]
                    touched_prevs.append(prev)
                    if ov <= rem:
                        in_e[u_abs].append([prev, ov]); in_e[u].pop(0)
                        replace_edge_list(out_e[prev], u, [[u_abs, ov]]); rem -= ov
                    else:
                        in_e[u_abs].append([prev, rem]); in_e[u][0][1] -= rem
                        replace_edge_list(out_e[prev], u, [[u_abs, rem], [u, ov - rem]]); rem = 0
            else:
                rev_in = []
                while rem > 0:
                    prev, ov = in_e[u][-1]
                    touched_prevs.append(prev)
                    if ov <= rem:
                        rev_in.append([prev, ov]); in_e[u].pop()
                        replace_edge_list(out_e[prev], u, [[u_abs, ov]]); rem -= ov
                    else:
                        rev_in.append([prev, rem]); in_e[u][-1][1] -= rem
                        replace_edge_list(out_e[prev], u, [[u, ov - rem], [u_abs, rem]]); rem = 0
                in_e[u_abs] = list(reversed(rev_in))
            for prev in touched_prevs:
                push_c(prev)
            stats["band_spl"] += 1; stats["total_ops"] += 1
            coalesce_around([u_abs, u, dp_l, dp_r, dom_prev[u_abs], dom_next[u_abs]])
            push_c(u_abs); push_c(u)
            if out_e[u]: push_c(out_e[u][0][0]); push_c(out_e[u][-1][0])
            dp_hist[live_cnt - p_before] += 1
            continue

    assert closed_primary is not None and live_cnt == 0, f"Incomplete fold live={live_cnt}"
    w_c, m_c = closed_primary
    cyc = ws.expand(w_c)
    q = len(cyc)
    rot = cyc[m_c:] + cyc[:m_c]
    text = ((rot * (n // q + 1))[:n])[::-1]

    # Count distinct reachable hash-consed SLP rules
    vis = set(); st_v = [w_c]
    while st_v:
        x = st_v.pop()
        if x in vis: continue
        vis.add(x)
        if ws.kind[x] == 1: st_v.append(ws.a[x]); st_v.append(ws.b[x])
        elif ws.kind[x] == 2: st_v.append(ws.a[x])
    stats["G_dedup"] = sum(1 for x in vis if x >= 256)
    stats["dp_hist"] = dict(sorted(dp_hist.items()))
    return text, dict(stats)


def main():
    print("=== 1. EXHAUSTIVE EXACTNESS CHECK (binary n<=8, ternary n<=5) ===")
    cnt = 0
    for base, max_n in [(2, 8), (3, 5)]:
        for n in range(2, max_n + 1):
            for code in range(base ** n):
                T = bytes((code // (base ** i)) % base for i in range(n))
                L, p = bc.bwt_encode(T)
                for m in ("dt_heavy_coal", "coal_first"):
                    dec, _ = fold_with_coalescence(L, p, mode=m)
                    assert dec == T
                    cnt += 1
    print(f"  EXHAUSTIVE PASS: {cnt} coalescing train-track runs verified exact [verified]")

    print("\n=== 2. ALL 6 AUTHENTICATED WIKIDATA BLOCKS: S2_dt_heavy vs +Coalescence vs Coalescence-First ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    zs = {"block-0.L": 20083, "block-2.L": 21950, "block-85.L": 30001,
          "block-143.L": 40664, "block-210.L": 46451, "block-207.L": 33608}
    blocks = []
    for entry in manifest["blocks"]:
        with open(f"/tmp/mibz2-blocks/{entry['file']}", "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        ref = bc.ref_ibwt(L, entry["primary_index"])
        blocks.append((entry, L, ref))
    r0_sum = sum(e["r"] for e, _, _ in blocks)
    z_sum = sum(zs.values())

    for mode in ("dt_heavy_no_coal", "dt_heavy_coal", "coal_first"):
        s_ops = s_coal = s_g = 0
        agg_hist = defaultdict(int)
        for entry, L, ref in blocks:
            dec, st = fold_with_coalescence(L, entry["primary_index"], mode=mode)
            assert dec == ref
            s_ops += st.get("total_ops", 0)
            s_coal += st.get("coalesced", 0)
            s_g += st.get("G_dedup", 0)
            for k, v in st["dp_hist"].items():
                agg_hist[k] += v
        print(f"  {mode:18s} | ops={s_ops:7d} ({s_ops/r0_sum:5.3f}r0, {s_ops/895391:5.3f}m16) | "
              f"coal={s_coal:6d} | G_dedup={s_g:6d} ({s_g/r0_sum:5.3f}r0, {s_g/z_sum:4.2f}z) | "
              f"dP_hist={dict(sorted(agg_hist.items()))} [6/6 EXACT]", flush=True)

    print("\n=== 3. REPETITIVE z(T) << r_0 FAMILIES (Thue-Morse, Fibonacci, Period-Doubling, Copy+Edits) ===")
    fams = []
    tm = [0]
    for k in range(1, 13):
        tm = tm + [1 - x for x in tm]
        if k in (6, 8, 10, 12): fams.append((f"ThueMorse_{k}", bytes(tm)))
    a, b = [0], [0, 1]
    for _ in range(16): a, b = b, b + a
    fams.append(("Fib_18+$", bytes(b + [2])))
    pd = [0]
    for k in range(1, 13):
        nxt = []
        for x in pd: nxt.extend([0, 1] if x == 0 else [0, 0])
        pd = nxt
        if k in (8, 10, 12): fams.append((f"PeriodDbl_{k}", bytes(pd)))
    # Random-copy repetitive string: 128-byte random base copied 16 times with 16 single-byte edits (r0/z >> 1!)
    rng = random.Random(7)
    base = bytearray(rng.randrange(4) for _ in range(128))
    rep = bytearray()
    for copy_i in range(16):
        c = bytearray(base)
        c[(copy_i * 7) % 128] = (c[(copy_i * 7) % 128] + 1) % 4
        rep.extend(c)
    fams.append(("Copy16+16edits", bytes(rep)))

    for name, T in fams:
        L, p = bc.bwt_encode(T)
        dec0, st0 = fold_with_coalescence(L, p, mode="dt_heavy_no_coal")
        dec1, st1 = fold_with_coalescence(L, p, mode="dt_heavy_coal")
        dec2, st2 = fold_with_coalescence(L, p, mode="coal_first")
        assert dec0 == T and dec1 == T and dec2 == T
        r0 = st0["r0"]
        print(f"  {name:15s} (n={len(T):4d}, r0={r0:4d}) | NoCoal={st0.get('total_ops',0):4d}({st0.get('total_ops',0)/r0:.3f}r) | "
              f"DT+Coal={st1.get('total_ops',0):4d}({st1.get('total_ops',0)/r0:.3f}r, c={st1.get('coalesced',0)}) | "
              f"CoalFirst={st2.get('total_ops',0):4d}({st2.get('total_ops',0)/r0:.3f}r, c={st2.get('coalesced',0)}, G={st2['G_dedup']}) [EXACT]")


if __name__ == "__main__":
    main()
