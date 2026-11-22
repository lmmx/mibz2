#!/usr/bin/env python3
"""
Investigator Beta — Cycle 4:
  1. Formal Verification of Theorem B4 Exactness, 0% Stale, Peak Boundary Pieces <= q_0,
     and Peak Micro-Step Pieces (with hysteresis guard q_self > 2*k_self_init -> boundary-RV)
     across 873 exhaustive BWTs, 642 multi-cycle columns, 4 adversarial killers + Fib_80, and 6 Wikidata blocks.
  2. Sub-0.97 MiB Memory Diet Measurements on ALL 6 Authenticated Wikidata Blocks:
       - Exact bit-packed 12B (Inline3) vs 16B (Inline7) causal records
       - Exact slab16 / slab8 dead-word liveness (total appended vs peak LIVE ref-counted vs reachable vs zero-slab SLP)
       - Exact Vec doubling (3x256 separate Vecs) vs Fixed Chunk-Chain Pool (chunk=32, 64, 128)
       - Full PeakM breakdown (piece pool + word store + SLP + workspace) vs Open's 0.97 MiB/w!
  3. Optimal Static Causal Elimination Order under Theorem B4 across 5 static orders on all 6 blocks:
       rarest_first_deg, rarest_first_rows, most_freq_first, fvs_scc_last, min_fill_static.
"""

import bisect
import heapq
import json
import random
import sys
import time
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-b")
import measure_iet_bse as mb
import escape_padded_ring as epr
import deep_audit_iet_bse as da


# ---------- Static Causal Elimination Orders ----------

def build_static_causal_order(active_syms, counts, init_strips, mode="rarest_first_deg"):
    """Build static elimination permutation rank[sym] in 0..sigma-1 (last = final hub)."""
    inc_deg = defaultdict(int)
    adj_out = {b: defaultdict(int) for b in active_syms}
    adj_in = {b: defaultdict(int) for b in active_syms}
    for src, dst, _, _, _ in init_strips:
        inc_deg[src] += 1
        inc_deg[dst] += 1
        if src != dst:
            adj_out[src][dst] += 1
            adj_in[dst][src] += 1

    if mode == "rarest_first_deg":
        ordered = sorted(active_syms, key=lambda b: (inc_deg[b], counts[b], b))
    elif mode == "rarest_first_rows":
        ordered = sorted(active_syms, key=lambda b: (counts[b], inc_deg[b], b))
    elif mode == "most_freq_first":
        ordered = sorted(active_syms, key=lambda b: (-counts[b], -inc_deg[b], b))
    elif mode == "fvs_scc_last":
        # Bug-free greedy FVS / acyclic-complement order, keeping top-8 hub bytes + '"' last!
        dom_buf = defaultdict(list)
        for src, dst, dlo, ilo, ln in init_strips:
            dom_buf[src].append((src, dst, dlo, ilo, ln, src, 1))
        ordered, _, _ = da.build_true_fvs_acyclic_order(active_syms, dom_buf)
        # Ensure heaviest hub (e.g., '"') is strictly last!
        hub = max(active_syms, key=lambda b: counts[b])
        if ordered[-1] != hub:
            ordered.remove(hub)
            ordered.append(hub)
    elif mode == "min_fill_static":
        # Static greedy Markowitz (min in_deg * out_deg on evolving transition graph, tie-break in+out)!
        rem = set(active_syms)
        hub = max(active_syms, key=lambda b: counts[b])
        rem.remove(hub)
        # Track active directed adjacency sets
        out_s = {u: {v for v in adj_out[u] if v != u} for u in active_syms}
        in_s = {u: {v for v in adj_in[u] if v != u} for u in active_syms}
        deg_w = dict(inc_deg)
        ordered = []
        while rem:
            b = min(rem, key=lambda u: (len(in_s[u]) * len(out_s[u]), len(in_s[u]) + len(out_s[u]), deg_w[u], counts[u], u))
            ordered.append(b)
            rem.remove(b)
            # Symbolic fill-in on graph
            ins = [x for x in in_s[b] if x in rem or x == hub]
            outs = [y for y in out_s[b] if y in rem or y == hub]
            for x in ins:
                out_s[x].discard(b)
                for y in outs:
                    if x != y:
                        out_s[x].add(y)
                        in_s[y].add(x)
            for y in outs:
                in_s[y].discard(b)
        ordered.append(hub)
    else:
        raise ValueError(mode)

    return {b: i for i, b in enumerate(ordered)}, ordered


# ---------- Exact Ref-Counted Slab + Vec-vs-Chunk-Pool Simulator ----------

class BucketPoolSimulator:
    """
    Simultaneously tracks exact memory allocation of:
      1. Naive 3 * 256 separate Rust Vecs (doubling 0 -> 4 -> 8 -> 16 -> ..., freed on bucket pop!)
      2. Pooled fixed-size chunk chains (chunk_size in {32, 64, 128}), recycled immediately into a free-list
         as soon as bucket `e` is popped!
    """
    def __init__(self, num_buckets, chunk_sizes=(32, 64, 128)):
        self.num_buckets = num_buckets
        self.chunk_sizes = chunk_sizes
        # 3 buffers per bucket: 0=self, 1=exit, 2=entry
        self.lens = [[0, 0, 0] for _ in range(num_buckets)]
        self.vec_caps = [[0, 0, 0] for _ in range(num_buckets)]
        self.cur_vec_cap = 0
        self.peak_vec_cap = 0

        self.cur_chunks = {cs: 0 for cs in chunk_sizes}
        self.peak_chunks = {cs: 0 for cs in chunk_sizes}
        self.cur_live = 0
        self.peak_live = 0

    def push(self, b, kind):
        self.cur_live += 1
        if self.cur_live > self.peak_live:
            self.peak_live = self.cur_live
        old_len = self.lens[b][kind]
        new_len = old_len + 1
        self.lens[b][kind] = new_len

        # Rust Vec doubling (0 -> 4 -> 8 -> 16 ...)
        cap = self.vec_caps[b][kind]
        if new_len > cap:
            new_cap = 4 if cap == 0 else cap * 2
            self.vec_caps[b][kind] = new_cap
            self.cur_vec_cap += (new_cap - cap)
            if self.cur_vec_cap > self.peak_vec_cap:
                self.peak_vec_cap = self.cur_vec_cap

        # Fixed chunk-chain pool
        for cs in self.chunk_sizes:
            if old_len % cs == 0:
                self.cur_chunks[cs] += 1
                if self.cur_chunks[cs] > self.peak_chunks[cs]:
                    self.peak_chunks[cs] = self.cur_chunks[cs]

    def pop_bucket(self, b):
        for kind in (0, 1, 2):
            ln = self.lens[b][kind]
            self.cur_live -= ln
            self.cur_vec_cap -= self.vec_caps[b][kind]
            self.vec_caps[b][kind] = 0
            for cs in self.chunk_sizes:
                if ln > 0:
                    used_chunks = (ln + cs - 1) // cs
                    self.cur_chunks[cs] -= used_chunks
            self.lens[b][kind] = 0


class WordStoreTracker:
    """
    Exact ref-counted word-store tracker measuring:
      - inline_cap: words <= inline_cap live INSIDE piece record (0 slab bytes!)
      - flat_cap:   words in (inline_cap, flat_cap] live in Flat Slab!
      - > flat_cap: binary SLP cat/pow nodes (8 bytes each!)
    Tracks BOTH:
      - Append-only slab bytes (total_slab_bytes, total_slab_slots)
      - Ref-counted / dead-word-eliminated LIVE slab bytes (peak_live_slab_bytes, peak_live_slab_slots)!
      Note: when a flat word is referenced by an SLP node, its refcount is incremented permanently
      (until expansion); when referenced ONLY by live pieces, its refcount drops to 0 the instant
      the piece dies/merges, immediately freeing its slab slot!
    """
    def __init__(self, inline_cap=3, flat_cap=16, use_bytes=True):
        self.inline_cap = inline_cap
        self.flat_cap = flat_cap
        self.use_bytes = use_bytes

        # Word representation:
        #   ('inl', seq)          -> len <= inline_cap (0 slab!)
        #   ('slab', sid, seq)    -> inline_cap < len <= flat_cap
        #   ('slp', rid, wlen)    -> len > flat_cap
        self.slab_ref = []       # refcount per slab entry
        self.slab_len = []       # byte length per slab entry
        self.slab_seq = []       # actual bytes/tuple for exact verification

        self.cur_live_slab_slots = 0
        self.peak_live_slab_slots = 0
        self.cur_live_slab_bytes = 0
        self.peak_live_slab_bytes = 0
        self.tot_slab_slots = 0
        self.tot_slab_bytes = 0

        # SLP rules: (0, slab_or_inl_seq) | (1, id1, id2) | (2, id1, k)
        # Note: in actual Rust, an SLP cat node (8B = [u32; 2]) holds two tagged u32 handles
        # (Inline3 or SlabID or SlpID) DIRECTLY — NO extra wrapper rule (0, ...) needed!
        self.slp_nodes = []      # list of ('cat', h1, h2) or ('pow', h1, k)
        self.inline_ops = 0
        self.slab_ops = 0
        self.slp_ops = 0

    def make_lit(self, sym):
        seq = bytes([sym]) if self.use_bytes else (sym,)
        return ('inl', seq), 1

    def incref(self, h):
        if h[0] == 'slab':
            sid = h[1]
            self.slab_ref[sid] += 1

    def decref(self, h):
        if h[0] == 'slab':
            sid = h[1]
            self.slab_ref[sid] -= 1
            if self.slab_ref[sid] == 0:
                self.cur_live_slab_slots -= 1
                self.cur_live_slab_bytes -= self.slab_len[sid]

    def _alloc_slab(self, seq):
        sid = len(self.slab_ref)
        ln = len(seq)
        self.slab_ref.append(1)
        self.slab_len.append(ln)
        self.slab_seq.append(seq)
        self.tot_slab_slots += 1
        self.tot_slab_bytes += ln
        self.cur_live_slab_slots += 1
        self.cur_live_slab_bytes += ln
        if self.cur_live_slab_slots > self.peak_live_slab_slots:
            self.peak_live_slab_slots = self.cur_live_slab_slots
        if self.cur_live_slab_bytes > self.peak_live_slab_bytes:
            self.peak_live_slab_bytes = self.cur_live_slab_bytes
        return ('slab', sid, seq)

    def compose(self, h1, l1, h2, l2):
        tl = l1 + l2
        if tl <= self.inline_cap:
            self.inline_ops += 1
            seq = (h1[1] if h1[0] == 'inl' else h1[2]) + (h2[1] if h2[0] == 'inl' else h2[2])
            return ('inl', seq), tl
        if tl <= self.flat_cap:
            self.slab_ops += 1
            s1 = h1[1] if h1[0] == 'inl' else h1[2]
            s2 = h2[1] if h2[0] == 'inl' else h2[2]
            return self._alloc_slab(s1 + s2), tl
        # Allocate 8-byte SLP cat node (referencing h1 & h2 permanently!)
        self.slp_ops += 1
        self.incref(h1)
        self.incref(h2)
        rid = len(self.slp_nodes)
        self.slp_nodes.append(('cat', h1, h2))
        return ('slp', rid, tl), tl

    def power(self, h1, l1, k):
        if k == 1:
            self.incref(h1)
            return h1, l1
        tl = l1 * k
        if tl <= self.inline_cap:
            self.inline_ops += 1
            seq = (h1[1] if h1[0] == 'inl' else h1[2]) * k
            return ('inl', seq), tl
        if tl <= self.flat_cap:
            self.slab_ops += 1
            s1 = h1[1] if h1[0] == 'inl' else h1[2]
            return self._alloc_slab(s1 * k), tl
        self.slp_ops += 1
        self.incref(h1)
        rid = len(self.slp_nodes)
        self.slp_nodes.append(('pow', h1, k))
        return ('slp', rid, tl), tl

    def expand_and_measure(self, root_h):
        """Output-backed memoized expansion (memoizing SLP nodes; inline/slab emitted directly!)."""
        memo_pos = {}
        out = bytearray() if self.use_bytes else []
        node_exp = 0
        memcpys = 0
        flat_writes = 0
        used_slab_ids = set()

        stack = [root_h]
        while stack:
            u = stack.pop()
            tag = u[0]
            if tag == 'fin':
                memo_pos[u[1]] = (u[2], len(out) - u[2])
            elif tag == 'inl':
                flat_writes += 1
                out.extend(u[1])
            elif tag == 'slab':
                flat_writes += 1
                used_slab_ids.add(u[1])
                out.extend(u[2])
            else:
                rid = u[1]
                if rid in memo_pos:
                    sp, ln = memo_pos[rid]
                    memcpys += 1
                    out.extend(out[sp:sp + ln])
                    continue
                node_exp += 1
                node = self.slp_nodes[rid]
                stack.append(('fin', rid, len(out)))
                if node[0] == 'cat':
                    stack.append(node[2])
                    stack.append(node[1])
                else:
                    for _ in range(node[2]):
                        stack.append(node[1])
        reach_slab_bytes = sum(self.slab_len[sid] for sid in used_slab_ids)
        return out, {
            "node_exp": node_exp,
            "flat_writes": flat_writes,
            "memcpys": memcpys,
            "emit_leaves": flat_writes + memcpys,
            "total_exp_ops": node_exp + flat_writes + memcpys,
            "reach_slab_slots": len(used_slab_ids),
            "reach_slab_bytes": reach_slab_bytes,
        }


# ---------- Theorem B4 Engine with Hysteresis Guard, Micro-Peak & Ref-Counted Memory Audit ----------

def run_b4_instrumented(
    runs, p, order_mode="rarest_first_deg",
    inline_cap=3, flat_cap=16, aht_thresh=2, hysteresis_mult=2.0,
    verify_seq=None
):
    r0 = len(runs)
    n = sum(w for _, w in runs)
    max_sym = max(c for c, _ in runs)
    counts = [0] * (max_sym + 1)
    for c, w in runs: counts[c] += w
    C = [0] * (max_sym + 2)
    for c in range(max_sym + 1): C[c + 1] = C[c] + counts[c]
    active_syms = [c for c in range(max_sym + 1) if counts[c] > 0]

    # Build initial bi-monotone strips
    f_cur = list(C[:max_sym + 1])
    init_strips = []
    pos = 0; im_b = 0
    while im_b <= max_sym and C[im_b + 1] == 0: im_b += 1
    for c, w in runs:
        rem = w; cur_im = pos; cur_dom = f_cur[c]
        while rem > 0:
            while C[im_b + 1] <= cur_im: im_b += 1
            take = min(rem, C[im_b + 1] - cur_im)
            init_strips.append((c, im_b, cur_dom, cur_im, take))
            cur_im += take; cur_dom += take; rem -= take
        f_cur[c] += w; pos += w
    q_init = len(init_strips)

    rank_map, ordered_syms = build_static_causal_order(active_syms, counts, init_strips, mode=order_mode)
    num_b = len(ordered_syms)

    use_bytes = (max_sym < 256)
    ws = WordStoreTracker(inline_cap=inline_cap, flat_cap=flat_cap, use_bytes=use_bytes)
    pool = BucketPoolSimulator(num_b, chunk_sizes=(32, 64, 128))

    self_buf = [[] for _ in range(num_b)]
    exit_buf = [[] for _ in range(num_b)]
    entry_buf = [[] for _ in range(num_b)]

    tot_appends = 0
    q_live_global = 0
    peak_boundary_pieces = 0
    peak_micro_pieces = 0
    hysteresis_triggers = 0

    def route_piece(src_r, dst_r, dlo, ilo, ln, wd, wl):
        nonlocal tot_appends, q_live_global, peak_boundary_pieces, peak_micro_pieces
        tot_appends += 1
        q_live_global += 1
        if q_live_global > peak_micro_pieces:
            peak_micro_pieces = q_live_global
        if src_r == dst_r:
            self_buf[src_r].append([dlo, ilo, ln, wd, wl])
            pool.push(src_r, 0)
        elif src_r < dst_r:
            exit_buf[src_r].append([dlo, dst_r, ilo, ln, wd, wl])
            pool.push(src_r, 1)
        else:
            entry_buf[dst_r].append([src_r, dlo, ilo, ln, wd, wl])
            pool.push(dst_r, 2)

    for c, im_b, dlo, ilo, ln in init_strips:
        wd, wl = ws.make_lit(c)
        route_piece(rank_map[c], rank_map[im_b], dlo, ilo, ln, wd, wl)
    peak_boundary_pieces = q_live_global
    assert q_live_global == q_init

    # Primary index p
    p_sym = 0
    while C[p_sym + 1] <= p: p_sym += 1
    p_state = [rank_map[p_sym], p, 0, None]

    # Metrics
    max_deg_before_last = 0
    tot_self_ops = 0
    tot_merge_ops = 0
    tot_c1 = 0; tot_c2 = 0
    q_boundary_viol = 0
    max_live_ws_pcs = 0

    for b in range(num_b):
        s_list = self_buf[b]; x_list = exit_buf[b]; e_list = entry_buf[b]
        self_buf[b] = []; exit_buf[b] = []; entry_buf[b] = []
        pool.pop_bucket(b)

        n_pcs = len(s_list) + len(x_list) + len(e_list)
        if b < num_b - 1 and n_pcs > max_deg_before_last:
            max_deg_before_last = n_pcs
        if n_pcs > max_live_ws_pcs:
            max_live_ws_pcs = n_pcs
        if n_pcs == 0:
            continue

        q_before = q_live_global
        q_ext = q_live_global - n_pcs  # untouched pieces in buckets > b

        self_pcs = {i: pc for i, pc in enumerate(s_list)}
        next_id = len(s_list)
        entry_pcs = {}
        for pc in e_list:
            entry_pcs[next_id] = pc; next_id += 1
        exit_pcs = x_list
        k_self_init = len(self_pcs)

        def check_micro():
            nonlocal peak_micro_pieces, max_live_ws_pcs
            cur_loc = len(self_pcs) + len(entry_pcs) + len(exit_pcs)
            if cur_loc > max_live_ws_pcs:
                max_live_ws_pcs = cur_loc
            cur_g = q_ext + cur_loc
            if cur_g > peak_micro_pieces:
                peak_micro_pieces = cur_g

        # Phase 1: AHT Widest-Self-Piece with Hysteresis Fallback (len(self_pcs) > hysteresis_mult * k_self_init -> RV!)
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
            self_cap = max(k_self_init + 2, int(hysteresis_mult * k_self_init))

            while heap:
                if len(self_pcs) > self_cap:
                    hysteresis_triggers += 1
                    break
                neg_L, uid = heapq.heappop(heap)
                u = self_pcs.get(uid)
                if u is None or u[2] != -neg_L: continue
                L = u[2]
                if L < aht_thresh: break
                dlo, ilo, L, wd, wl = u
                delta = ilo - dlo

                if abs(delta) < L:
                    tot_c1 += 1; tot_self_ops += 1
                    del self_pcs[uid]; im_rem(ilo)
                    if delta == 0:
                        if p_state[0] == b and p_state[3] is None and dlo <= p_state[1] < dlo + L:
                            ws.incref(wd)
                            p_state[3] = (wd, p_state[2], wl); p_state[0] = -1
                        ws.decref(wd)
                        continue
                    if delta > 0:
                        q, rem = divmod(L, delta)
                        if p_state[0] == b and p_state[3] is None and dlo + delta <= p_state[1] < dlo + L:
                            m = (p_state[1] - dlo) // delta
                            p_state[1] -= m * delta; p_state[2] += m * wl
                        if rem > 0:
                            nid = next_id; next_id += 1
                            w1, wl1 = ws.power(wd, wl, q + 1)
                            i1 = dlo + (q + 1) * delta
                            self_pcs[nid] = [dlo, i1, rem, w1, wl1]
                            im_ins(i1, nid, True); heapq.heappush(heap, (-rem, nid))
                        ln2 = delta - rem
                        if ln2 > 0:
                            nid = next_id; next_id += 1
                            w2, wl2 = ws.power(wd, wl, q)
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
                            w1, wl1 = ws.power(wd, wl, q)
                            i1 = dlo - d + rem
                            self_pcs[nid] = [dlo + L - d, i1, ln1, w1, wl1]
                            im_ins(i1, nid, True); heapq.heappush(heap, (-ln1, nid))
                        if rem > 0:
                            nid = next_id; next_id += 1
                            w2, wl2 = ws.power(wd, wl, q + 1)
                            self_pcs[nid] = [dlo + L - rem, dlo - d, rem, w2, wl2]
                            im_ins(dlo - d, nid, True); heapq.heappush(heap, (-rem, nid))
                    ws.decref(wd)
                    check_micro()
                else:
                    tot_c2 += 1; tot_self_ops += 1
                    del self_pcs[uid]; im_rem(ilo)
                    dhi = dlo + L
                    idx = max(0, bisect.bisect_right(im_starts, dlo) - 1)
                    ovs = []
                    while idx < len(im_starts) and im_starts[idx] < dhi:
                        vi = im_starts[idx]; vid, is_s = im_meta[idx]
                        vln = self_pcs[vid][2] if is_s else entry_pcs[vid][3]
                        if vi + vln > dlo: ovs.append((vi, vid, is_s))
                        idx += 1
                    for vi, vid, is_s in ovs:
                        im_rem(vi)
                        if is_s:
                            vdlo, vilo, vln, vwd, vwl = self_pcs.pop(vid)
                            olo = max(dlo, vilo); ohi = min(dhi, vilo + vln)
                            ll, ml, rl = olo - vilo, ohi - olo, vilo + vln - ohi
                            if ll > 0:
                                ws.incref(vwd)
                                lid = next_id; next_id += 1
                                self_pcs[lid] = [vdlo, vilo, ll, vwd, vwl]
                                im_ins(vilo, lid, True); heapq.heappush(heap, (-ll, lid))
                            mdlo = vdlo + ll; milo = ilo + (olo - dlo)
                            if p_state[0] == b and p_state[3] is None and olo <= p_state[1] < ohi:
                                p_state[1] = mdlo + (p_state[1] - olo); p_state[2] += vwl
                            mwd, mwl = ws.compose(vwd, vwl, wd, wl)
                            mid = next_id; next_id += 1
                            self_pcs[mid] = [mdlo, milo, ml, mwd, mwl]
                            im_ins(milo, mid, True); heapq.heappush(heap, (-ml, mid))
                            if rl > 0:
                                ws.incref(vwd)
                                rid = next_id; next_id += 1
                                self_pcs[rid] = [vdlo + ll + ml, ohi, rl, vwd, vwl]
                                im_ins(ohi, rid, True); heapq.heappush(heap, (-rl, rid))
                            ws.decref(vwd)
                        else:
                            eb, elo, vilo, vln, vwd, vwl = entry_pcs.pop(vid)
                            olo = max(dlo, vilo); ohi = min(dhi, vilo + vln)
                            ll, ml, rl = olo - vilo, ohi - olo, vilo + vln - ohi
                            if ll > 0:
                                ws.incref(vwd)
                                lid = next_id; next_id += 1
                                entry_pcs[lid] = [eb, elo, vilo, ll, vwd, vwl]
                                im_ins(vilo, lid, False)
                            melo = elo + ll; milo = ilo + (olo - dlo)
                            if p_state[0] == b and p_state[3] is None and olo <= p_state[1] < ohi:
                                p_state[0] = eb; p_state[1] = melo + (p_state[1] - olo); p_state[2] += vwl
                            mwd, mwl = ws.compose(vwd, vwl, wd, wl)
                            mid = next_id; next_id += 1
                            entry_pcs[mid] = [eb, melo, milo, ml, mwd, mwl]
                            im_ins(milo, mid, False)
                            if rl > 0:
                                ws.incref(vwd)
                                rid = next_id; next_id += 1
                                entry_pcs[rid] = [eb, elo + ll + ml, ohi, rl, vwd, vwl]
                                im_ins(ohi, rid, False)
                            ws.decref(vwd)
                    ws.decref(wd)
                    check_micro()

        # Phase 2: Fast finish
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
                    tot_merge_ops += 1
                    src_r = en[0]; src_lo = en[1] + eo
                    dst_r = ex[1]; dst_lo = ex[2] + xo
                    cw, cwl = ws.compose(en[4], en[5], ex[4], ex[5])
                    emitted.append((src_r, dst_r, src_lo, dst_lo, take, cw, cwl))
                    if p_state[0] == b and p_state[3] is None:
                        xp = ex[0] + xo
                        if xp <= p_state[1] < xp + take:
                            p_state[0] = src_r; p_state[1] = src_lo + (p_state[1] - xp); p_state[2] += en[5]
                    xo += take; eo += take
                    if xo == ex[3]:
                        ws.decref(ex[4]); xi += 1; xo = 0
                    if eo == en[3]:
                        ws.decref(en[4]); ei += 1; eo = 0
        else:
            nk, nl, nw, nwl, neb, nel = [], [], [], [], [], []
            dp_p, im_p = [], []
            for pid, (dlo, ilo, ln, wd, wl) in self_pcs.items():
                nid = len(nk)
                nk.append(0); nl.append(ln); nw.append(wd); nwl.append(wl); neb.append(b); nel.append(-1)
                dp_p.append((dlo, nid)); im_p.append((ilo, nid))
            for dlo, eb, elo, ln, wd, wl in exit_pcs:
                nid = len(nk)
                nk.append(1); nl.append(ln); nw.append(wd); nwl.append(wl); neb.append(eb); nel.append(elo)
                dp_p.append((dlo, nid))
            for pid, (eb, elo, ilo, ln, wd, wl) in entry_pcs.items():
                nid = len(nk)
                nk.append(2); nl.append(ln); nw.append(wd); nwl.append(wl); neb.append(eb); nel.append(elo)
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
                    tot_self_ops += 1
                    if pin and pnid == d:
                        ws.incref(nw[d])
                        p_state[3] = (nw[d], p_state[2], nwl[d]); p_state[0] = -1; pin = False
                    ws.decref(nw[d]); du(d); iu(d); continue
                kd, ki, ld, li = nk[d], nk[i], nl[d], nl[i]
                if kd == 1 and ki == 2:
                    tot_merge_ops += 1
                    take = min(ld, li)
                    src_r, dst_r = neb[i], neb[d]
                    slo, dlo_t = nel[i] + (li - take), nel[d] + (ld - take)
                    cw, cwl = ws.compose(nw[i], nwl[i], nw[d], nwl[d])
                    emitted.append((src_r, dst_r, slo, dlo_t, take, cw, cwl))
                    if pin and pnid == d and poff >= ld - take:
                        p_state[0] = src_r; p_state[1] = slo + (poff - (ld - take)); p_state[2] += nwl[i]; pin = False
                    if ld == take: ws.decref(nw[d]); du(d)
                    else: nl[d] -= take
                    if li == take: ws.decref(nw[i]); iu(i)
                    else: nl[i] -= take
                elif ld > li:
                    tot_self_ops += 1
                    old_wi = nw[i]
                    if kd == 0:
                        if pin and pnid == d and poff >= ld - li:
                            if ki == 2:
                                p_state[0] = neb[i]; p_state[1] = nel[i] + (poff - (ld - li)); p_state[2] += nwl[i]; pin = False
                            else:
                                pnid = i; poff -= (ld - li); p_state[2] += nwl[i]
                        nw[i], nwl[i] = ws.compose(nw[i], nwl[i], nw[d], nwl[d])
                        ws.decref(old_wi)
                        nl[d] -= li
                        if inn[d] != i: iu(i); ia(d, i)
                    else:
                        if pin and pnid == d and poff >= ld - li:
                            pnid = i; poff -= (ld - li); p_state[2] += nwl[i]
                        nw[i], nwl[i] = ws.compose(nw[i], nwl[i], nw[d], nwl[d])
                        ws.decref(old_wi)
                        nk[i] = 1; neb[i] = neb[d]; nel[i] = nel[d] + (ld - li); nl[d] -= li; iu(i)
                elif li > ld:
                    tot_self_ops += 1
                    old_wd = nw[d]
                    if ki == 0:
                        if pin:
                            if pnid == d: p_state[2] += nwl[i]
                            elif pnid == i and poff >= li - ld: pnid = d; poff -= (li - ld)
                        nw[d], nwl[d] = ws.compose(nw[i], nwl[i], nw[d], nwl[d])
                        ws.decref(old_wd)
                        nl[i] -= ld
                        if dn[i] != d: du(d); da(i, d)
                    else:
                        slo = nel[i] + (li - ld)
                        if pin and pnid == d:
                            p_state[0] = neb[i]; p_state[1] = slo + poff; p_state[2] += nwl[i]; pin = False
                        nw[d], nwl[d] = ws.compose(nw[i], nwl[i], nw[d], nwl[d])
                        ws.decref(old_wd)
                        nk[d] = 2; neb[d] = neb[i]; nel[d] = slo; nl[i] -= ld; du(d)
                else:
                    tot_self_ops += 1
                    if kd == 0 and ki == 0:
                        if pin and pnid == d: pnid = i; p_state[2] += nwl[i]
                        old_wi = nw[i]
                        nw[i], nwl[i] = ws.compose(nw[i], nwl[i], nw[d], nwl[d])
                        ws.decref(old_wi); ws.decref(nw[d])
                        if inn[d] == i: iu(d)
                        else: iu(i); ia(d, i); iu(d)
                        du(d)
                    elif kd == 1 and ki == 0:
                        if pin and pnid == d: pnid = i; p_state[2] += nwl[i]
                        old_wi = nw[i]
                        nw[i], nwl[i] = ws.compose(nw[i], nwl[i], nw[d], nwl[d])
                        ws.decref(old_wi); ws.decref(nw[d])
                        nk[i] = 1; neb[i] = neb[d]; nel[i] = nel[d]; iu(i); du(d)
                    else:
                        if pin and pnid == d:
                            p_state[0] = neb[i]; p_state[1] = nel[i] + poff; p_state[2] += nwl[i]; pin = False
                        old_wd = nw[d]
                        nw[d], nwl[d] = ws.compose(nw[i], nwl[i], nw[d], nwl[d])
                        ws.decref(old_wd); ws.decref(nw[i])
                        nk[d] = 2; neb[d] = neb[i]; nel[d] = nel[i]; du(d); iu(i)

        q_live_global = q_ext
        for ep in emitted:
            route_piece(*ep)
        if b < num_b - 1:
            if q_live_global > q_before - 1:
                q_boundary_viol += 1
            if q_live_global > peak_boundary_pieces:
                peak_boundary_pieces = q_live_global

    assert q_live_global == 0
    assert p_state[3] is not None
    root_h, sh, wl = p_state[3]
    if n <= 2_000_000:
        out, exp_st = ws.expand_and_measure(root_h)
        ws.decref(root_h)
        assert len(out) == wl
    else:
        out = None
        exp_st = {"emit_leaves": 0, "total_exp_ops": 0, "reach_slab_slots": 0, "reach_slab_bytes": 0}
        ws.decref(root_h)

    if verify_seq is not None:
        s = sh % wl
        rot = (bytes(out[s:]) + bytes(out[:s])) if use_bytes else (out[s:] + out[:s])
        if len(verify_seq) % len(rot) == 0:
            full = rot * (len(verify_seq) // len(rot))
        else:
            full = rot
        want = bytes(verify_seq) if use_bytes else list(verify_seq)
        assert full == want, f"MISMATCH in B4 {order_mode}!"

    return {
        "n": n, "r0": r0, "q_init": q_init, "order": order_mode,
        "q_boundary_viol": q_boundary_viol,
        "peak_boundary_pieces": peak_boundary_pieces,
        "peak_micro_pieces": peak_micro_pieces,
        "hysteresis_triggers": hysteresis_triggers,
        "tot_appends": tot_appends,
        "tot_self_ops": tot_self_ops,
        "tot_merge_ops": tot_merge_ops,
        "tot_c1": tot_c1, "tot_c2": tot_c2,
        "max_deg_before_last": max_deg_before_last,
        "max_live_ws_pcs": max_live_ws_pcs,
        # Pool stats
        "pool_peak_live": pool.peak_live,
        "pool_peak_vec_cap": pool.peak_vec_cap,
        "pool_peak_chunks": {cs: pool.peak_chunks[cs] * cs for cs in pool.chunk_sizes},
        # Word store stats
        "tot_slab_slots": ws.tot_slab_slots,
        "tot_slab_bytes": ws.tot_slab_bytes,
        "peak_live_slab_slots": ws.peak_live_slab_slots,
        "peak_live_slab_bytes": ws.peak_live_slab_bytes,
        "reach_slab_slots": exp_st["reach_slab_slots"],
        "reach_slab_bytes": exp_st["reach_slab_bytes"],
        "slp_nodes": len(ws.slp_nodes),
        "emit_leaves": exp_st["emit_leaves"],
        "total_exp_ops": exp_st["total_exp_ops"],
    }


def main():
    print("=== 1. FORMAL VERIFICATION OF THEOREM B4 (Exhaustive, Multi-Cycle, 4 Adversarial Killers, Fib_80) ===")
    cnt = 0
    max_micro_over = 0
    for n in range(1, 9):
        for mask in range(1 << n):
            T = [(mask >> i) & 1 for i in range(n)]
            L, p = epr.bwt_encode_fast(T)
            runs = mb.rle_from_bytes(L)
            st = run_b4_instrumented(runs, p, order_mode="rarest_first_deg", verify_seq=T)
            assert st["q_boundary_viol"] == 0 and st["peak_boundary_pieces"] <= st["q_init"]
            max_micro_over = max(max_micro_over, st["peak_micro_pieces"] - st["q_init"])
            cnt += 1
    for n in range(1, 6):
        for idx in range(3 ** n):
            T = []; v = idx
            for _ in range(n): T.append(v % 3); v //= 3
            L, p = epr.bwt_encode_fast(T)
            runs = mb.rle_from_bytes(L)
            st = run_b4_instrumented(runs, p, order_mode="rarest_first_deg", verify_seq=T)
            assert st["q_boundary_viol"] == 0 and st["peak_boundary_pieces"] <= st["q_init"]
            max_micro_over = max(max_micro_over, st["peak_micro_pieces"] - st["q_init"])
            cnt += 1
    mc = 0
    for n in range(1, 7):
        for mask in range(1 << n):
            L = [(mask >> i) & 1 for i in range(n)]
            runs = mb.rle_from_bytes(L)
            counts = [L.count(0), L.count(1)]
            pos = [0, counts[0]]; LF = [0]*n
            for i, ch in enumerate(L): LF[i] = pos[ch]; pos[ch] += 1
            for p in range(n):
                cyc = []
                cur = p
                while True:
                    cyc.append(L[cur]); cur = LF[cur]
                    if cur == p: break
                st = run_b4_instrumented(runs, p, order_mode="rarest_first_deg", verify_seq=cyc[::-1])
                assert st["q_boundary_viol"] == 0 and st["peak_boundary_pieces"] <= st["q_init"]
                mc += 1
    print(f"  PASS: {cnt} exhaustive BWTs + {mc} multi-cycle/all-p cases (0 q_boundary_viol, max micro overshoot = +{max_micro_over})!")

    # Adversarial killers + Fib_80
    def make_sym_mid(K):
        rng = random.Random(123)
        low_tags = [bytes([65 + (i // 8), 65 + (i % 8)]) for i in range(64)]
        high_tags = [bytes([83 + (i // 8), 83 + (i % 8)]) for i in range(64)]
        rng.shuffle(low_tags); rng.shuffle(high_tags)
        return list(b"".join(low_tags[i] + bytes([77]*K) + high_tags[i] for i in range(64)))

    for name, T in [
        ("Low-r Padded-Ring K=256", epr.make_low_r_padded_ring(256, k=4)),
        ("AltRing K=256", [0]*256 + [1]*256 + [2]*256 + [1]*256 + [0]*256 + [2]*256 + [3]),
        ("3-Phase K=256", [0,1,2]*256 + [0,2,1]*256 + [1,0,2]*256 + [3]),
        ("SymMidRun K=64", make_sym_mid(64)),
    ]:
        L, p = epr.bwt_encode_fast(T)
        runs = mb.rle_from_bytes(L)
        st = run_b4_instrumented(runs, p, order_mode="rarest_first_deg", verify_seq=T)
        assert st["q_boundary_viol"] == 0 and st["peak_boundary_pieces"] <= st["q_init"]
        print(f"  {name:24s}: n={len(T):5d} q0={st['q_init']:3d} pk_bnd={st['peak_boundary_pieces']:3d} pk_mic={st['peak_micro_pieces']:3d} "
              f"app={st['tot_appends']:3d} self={st['tot_self_ops']:3d} mrg={st['tot_merge_ops']:3d} hyst={st['hysteresis_triggers']} [VERIFIED]")

    fibs = [0, 1]
    for _ in range(85): fibs.append(fibs[-1] + fibs[-2])
    f2, f1 = fibs[78], fibs[79]
    runs_f = [(1, f2 - 1), (2, 1), (0, f1), (1, 1)]
    st_f = run_b4_instrumented(runs_f, f2 - 1, order_mode="rarest_first_deg", verify_seq=None)
    print(f"  {'Fib_80+$':24s}: n={f2+f1} q0={st_f['q_init']} pk_bnd={st_f['peak_boundary_pieces']} pk_mic={st_f['peak_micro_pieces']} self={st_f['tot_self_ops']} mrg={st_f['tot_merge_ops']} [OK]")

    print("\n=== 2. SUB-0.97 MiB MEMORY DIET ON ALL 6 AUTHENTICATED WIKIDATA BLOCKS (rarest_first_deg) ===")
    blocks = mb.load_authenticated_blocks()
    block_cache = []
    for binfo, raw in blocks:
        runs = mb.rle_from_bytes(raw)
        _, m16, _ = mb.compute_open_m16(runs, binfo["primary_index"], limit=16)
        ref = mb.decode_lf_reference_bytes(raw, binfo["primary_index"])
        block_cache.append((binfo, runs, m16, ref))

    regimes = [
        ("A:12B_Inl3+Flat16", 12, 3, 16, 16),
        ("B:12B_Inl3+Flat8 ", 12, 3, 8,  8),
        ("C:12B_Inl3+0Slab ", 12, 3, 3,  0),
        ("D:16B_Inl7+Flat16", 16, 7, 16, 16),
        ("E:16B_Inl7+0Slab ", 16, 7, 7,  0),
    ]

    for rname, rec_B, inl_c, flt_c, slot_B in regimes:
        print(f"\n  --- Regime {rname} (piece={rec_B}B, inline<={inl_c}B, slab<={flt_c}B [{slot_B}B/slot], SLP=8B/node) ---")
        print(f"  {'Block':11s} | {'LivePcs':>7s} {'Chunk64':>7s} {'VecCap':>7s} | {'SlabApp':>7s} {'SlabLiv':>7s} {'SlabRch':>7s} | {'SLP_KiB':>7s} {'WS_KiB':>6s} | {'PeakM_App':>9s} {'PeakM_RC':>9s} | {'Leaves':>6s}({'xm16':>4s}) {'ExpOps':>6s}({'xm16':>4s})")
        max_pm_app = 0; max_pm_rc = 0
        for binfo, runs, m16, ref in block_cache:
            st = run_b4_instrumented(runs, binfo["primary_index"], order_mode="rarest_first_deg",
                                     inline_cap=inl_c, flat_cap=flt_c, verify_seq=ref)
            assert st["q_boundary_viol"] == 0 and st["peak_boundary_pieces"] <= st["q_init"]
            # Memory components in KiB:
            # 1. Chunk64 pool of piece records (rec_B each)
            pcs_live_kib = st["pool_peak_live"] * rec_B / 1024.0
            pcs_c64_kib = st["pool_peak_chunks"][64] * rec_B / 1024.0
            pcs_vec_kib = st["pool_peak_vec_cap"] * rec_B / 1024.0
            # 2. Slab (append-only packed bytes vs ref-counted fixed slot_B slots!)
            slab_app_kib = st["tot_slab_bytes"] / 1024.0
            slab_rc_kib = (st["peak_live_slab_slots"] * slot_B) / 1024.0 if slot_B > 0 else 0.0
            slab_rch_kib = st["reach_slab_bytes"] / 1024.0
            # 3. SLP table: 8 B per node (two tagged u32 handles)
            slp_kib = (st["slp_nodes"] * 8) / 1024.0
            # 4. Active bucket workspace: max_live_ws_pcs * 16 B (shift-bin-grid / RV links, reused per bucket)
            ws_kib = (st["max_live_ws_pcs"] * 16) / 1024.0

            # Exact PeakM with Chunk64 pool (Append-only packed slab vs Ref-counted slab)
            pm_app = pcs_c64_kib + slab_app_kib + slp_kib + ws_kib
            pm_rc = pcs_c64_kib + slab_rc_kib + slp_kib + ws_kib
            max_pm_app = max(max_pm_app, pm_app)
            max_pm_rc = max(max_pm_rc, pm_rc)

            print(f"  {binfo['file']:11s} | {pcs_live_kib:6.0f}K {pcs_c64_kib:6.0f}K {pcs_vec_kib:6.0f}K | "
                  f"{slab_app_kib:6.0f}K {slab_rc_kib:6.0f}K {slab_rch_kib:6.0f}K | "
                  f"{slp_kib:6.0f}K {ws_kib:5.0f}K | {pm_app/1024:7.2f}Mi {pm_rc/1024:7.2f}Mi | "
                  f"{st['emit_leaves']:6d}({st['emit_leaves']/m16:4.2f}) {st['total_exp_ops']:6d}({st['total_exp_ops']/m16:4.2f})")
        print(f"  -> Max across 6 blocks: PeakM(Append-Slab+Chunk64)={max_pm_app/1024:.2f} MiB | PeakM(RC-Slab+Chunk64)={max_pm_rc/1024:.2f} MiB")

    print("\n=== 3. POOL VS 768 VECS OVERHEAD (block-0.L .. block-210.L, 12B piece) ===")
    for binfo, runs, m16, _ in block_cache:
        st = run_b4_instrumented(runs, binfo["primary_index"], order_mode="rarest_first_deg", inline_cap=3, flat_cap=16, verify_seq=None)
        pl = st["pool_peak_live"]
        vc = st["pool_peak_vec_cap"]
        c32 = st["pool_peak_chunks"][32]
        c64 = st["pool_peak_chunks"][64]
        c128 = st["pool_peak_chunks"][128]
        print(f"  {binfo['file']:11s}: live={pl:6d} ({pl*12/1024:5.0f}K) | 768Vec={vc:6d} ({vc/pl:.2f}x, {vc*12/1024:5.0f}K) | "
              f"Chunk32={c32:6d} ({c32/pl:.2f}x, {c32*12/1024:5.0f}K) | Chunk64={c64:6d} ({c64/pl:.2f}x, {c64*12/1024:5.0f}K) | Chunk128={c128:6d} ({c128/pl:.2f}x, {c128*12/1024:5.0f}K) | "
              f"micro_peak={st['peak_micro_pieces']:6d} (overshoot=+{st['peak_micro_pieces']-st['q_init']}, hyst={st['hysteresis_triggers']})")

    print("\n=== 4. OPTIMAL STATIC CAUSAL ELIMINATION ORDER UNDER THEOREM B4 (5 ORDERS x 6 BLOCKS) ===")
    orders = ["rarest_first_deg", "rarest_first_rows", "most_freq_first", "fvs_scc_last", "min_fill_static"]
    tot_r0 = sum(b["r"] for b, _, _, _ in block_cache)
    tot_m16 = sum(m for _, _, m, _ in block_cache)
    print(f"  {'Order':18s} | {'App/r0':>6s} | {'Self/r0':>7s} | {'Mrg/r0':>6s} | {'maxDeg<L':>8s} | {'PkLiv/r0':>8s} | {'SlabApp':>7s} {'SlabRC':>7s} | {'SLP_nd':>7s} | {'Leaves/m16':>10s} | {'ExpOps/m16':>10s}")
    order_results = {}
    for om in orders:
        agg = defaultdict(int)
        max_deg = 0
        max_pk_ratio = 0.0
        max_slab_app = 0; max_slab_rc = 0
        for binfo, runs, m16, ref in block_cache:
            st = run_b4_instrumented(runs, binfo["primary_index"], order_mode=om, inline_cap=3, flat_cap=16, verify_seq=ref)
            assert st["q_boundary_viol"] == 0
            for k in ("tot_appends", "tot_self_ops", "tot_merge_ops", "tot_slab_bytes", "peak_live_slab_slots", "slp_nodes", "emit_leaves", "total_exp_ops"):
                agg[k] += st[k]
            max_deg = max(max_deg, st["max_deg_before_last"])
            max_pk_ratio = max(max_pk_ratio, st["peak_boundary_pieces"] / binfo["r"])
            max_slab_app = max(max_slab_app, st["tot_slab_bytes"])
            max_slab_rc = max(max_slab_rc, st["peak_live_slab_slots"] * 16)
        order_results[om] = dict(agg, max_deg=max_deg, max_pk_ratio=max_pk_ratio)
        print(f"  {om:18s} | {agg['tot_appends']/tot_r0:6.3f} | {agg['tot_self_ops']/tot_r0:7.3f} | {agg['tot_merge_ops']/tot_r0:6.3f} | "
              f"{max_deg:8d} | {max_pk_ratio:8.4f} | {max_slab_app/1024:6.0f}K {max_slab_rc/1024:6.0f}K | "
              f"{agg['slp_nodes']:7d} | {agg['emit_leaves']/tot_m16:10.4f} | {agg['total_exp_ops']/tot_m16:10.4f}")

    with open("/tmp/fcp-b/cycle4_results.json", "w") as f:
        json.dump(order_results, f, indent=2)
    print("\nSaved Cycle 4 order summary to /tmp/fcp-b/cycle4_results.json")


if __name__ == "__main__":
    main()
