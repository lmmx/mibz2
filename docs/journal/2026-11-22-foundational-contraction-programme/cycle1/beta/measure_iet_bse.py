#!/usr/bin/env python3
"""
Investigator Beta — Foundational Contraction Programme (Cycle 1)
Rigorous Measurement & Verification of Bucket-Schur Elimination in IET Space (IET-BSE)
with NO BWT-validity constraint, across all 6 SHA-256 authenticated Wikidata blocks.

Answers measurements (a), (b), (c), (d):
  (a) Total piece touches (sequential vs L1/L2-local random vs global random) across 5 orders:
      1. most_freq_first   (largest |F_b| first, rarest last)
      2. rarest_first_rows (smallest |F_b| first, most frequent last)
      3. rarest_first_deg  (dynamic min-degree k_dom + k_im first)
      4. fvs_acyclic_order (transition-graph acyclic-complement topological order, FVS last)
      5. min_self_fill     (dynamic min current self-pieces, tie-break min 2-cycle fill-in, min deg)
      6. max_sym           (control = Cells / Lazard-MAX bucket order)
  (b) Every non-monotone self-IET T_self^* encountered: count, piece counts, rows |F_b|,
      inversions, exact resolution work, width-1 steps, hidden-Theta(rows) check.
  (c) Final bucket cycle length (rows & pieces), final RV steps, grammar size (created & reachable), height.
  (d) Output-backed memoized SLP expansion cost + complete economic inequality vs OpenCells.
"""

import hashlib
import json
import time
from collections import defaultdict


# ---------- Authentication & Reference Helpers ----------

def load_authenticated_blocks():
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    blocks = []
    for binfo in manifest["blocks"]:
        path = "/tmp/mibz2-blocks/" + binfo["file"]
        raw = open(path, "rb").read()
        h = hashlib.sha256(raw).hexdigest()
        assert h == binfo["sha256"], f"SHA-256 MISMATCH for {binfo['file']}: {h} != {binfo['sha256']}"
        assert len(raw) == binfo["n"]
        blocks.append((binfo, raw))
    return blocks


def rle_from_bytes(raw):
    runs = []
    cur = raw[0]
    cnt = 1
    for x in raw[1:]:
        if x == cur:
            cnt += 1
        else:
            runs.append((cur, cnt))
            cur = x
            cnt = 1
    runs.append((cur, cnt))
    return runs


def compute_open_m16(runs, p, limit=16):
    """Exact OpenCells retained row count m_16 (sum of runs <= limit, plus 1 if p in thick run? Wait:
    in Open, thick strips are L-runs > limit intersected with F-buckets or maximal runs > limit;
    let's report exact maximal-run thin rows AND exact strip-level thin rows)."""
    thin_max_runs = sum(w for _, w in runs if w <= limit)
    # Also compute exact bi-monotone strip thin rows (intersect L-runs with F-buckets)
    counts = [0] * 256
    for c, w in runs:
        counts[c] += w
    C = [0] * 257
    for c in range(256):
        C[c + 1] = C[c] + counts[c]
    thin_strips = 0
    thick_strips = 0
    pos = 0
    fb = 0
    while fb < 256 and C[fb + 1] == 0:
        fb += 1
    for c, w in runs:
        rem = w
        cur = pos
        while rem > 0:
            while C[fb + 1] <= cur:
                fb += 1
            take = min(rem, C[fb + 1] - cur)
            if take <= limit:
                thin_strips += take
            else:
                thick_strips += 1
            cur += take
            rem -= take
        pos += w
    return thin_max_runs, thin_strips, thick_strips


def decode_lf_reference_bytes(raw, p):
    n = len(raw)
    counts = [0] * 256
    for x in raw:
        counts[x] += 1
    C = [0] * 256
    acc = 0
    for i in range(256):
        C[i] = acc
        acc += counts[i]
    pos = list(C)
    LF = [0] * n
    for i, ch in enumerate(raw):
        LF[i] = pos[ch]
        pos[ch] += 1
    out = bytearray(n)
    cur = p
    for k in range(n - 1, -1, -1):
        out[k] = raw[cur]
        cur = LF[cur]
    return bytes(out)


def expand_slp_bytes(rules, root):
    out = bytearray()
    stack = [root]
    while stack:
        u = stack.pop()
        r = rules[u]
        tag = r[0]
        if tag == 0:  # lit
            out.append(r[1])
        elif tag == 1:  # cat
            stack.append(r[2])
            stack.append(r[1])
        else:  # pow
            base, k = r[1], r[2]
            for _ in range(k):
                stack.append(base)
    return bytes(out)


def measure_slp_expansion(rules, root, thresholds=(4, 8, 16)):
    """Compute reachable rules, height, and output-backed memoized expansion ops for each threshold."""
    wlen = {}
    height = {}
    stack = [(root, False)]
    while stack:
        u, done = stack.pop()
        if u in wlen:
            continue
        r = rules[u]
        tag = r[0]
        if tag == 0:
            wlen[u] = 1
            height[u] = 0
        elif tag == 1:
            if done:
                wlen[u] = wlen[r[1]] + wlen[r[2]]
                h1, h2 = height[r[1]], height[r[2]]
                height[u] = 1 + (h1 if h1 > h2 else h2)
            else:
                stack.append((u, True))
                stack.append((r[1], False))
                stack.append((r[2], False))
        else:
            if done:
                wlen[u] = wlen[r[1]] * r[2]
                height[u] = 1 + height[r[1]]
            else:
                stack.append((u, True))
                stack.append((r[1], False))

    res = {
        "reachable_rules": len(wlen),
        "slp_height": height[root],
        "root_len": wlen[root],
        "by_tau": {},
    }
    for tau in thresholds:
        memo = set()
        memcpys = 0
        memcpy_bytes = 0
        lit_writes = 0
        node_expansions = 0
        stack = [root]
        # Post-order marking via negative id
        while stack:
            u = stack.pop()
            if u < 0:
                memo.add(-u - 1)
                continue
            wl = wlen[u]
            if wl >= tau and u in memo:
                memcpys += 1
                memcpy_bytes += wl
                continue
            r = rules[u]
            tag = r[0]
            if tag == 0:
                lit_writes += 1
            elif tag == 1:
                node_expansions += 1
                if wl >= tau:
                    stack.append(-u - 1)
                stack.append(r[2])
                stack.append(r[1])
            else:
                node_expansions += 1
                base, k = r[1], r[2]
                if wl >= tau:
                    stack.append(-u - 1)
                for _ in range(k):
                    stack.append(base)
        res["by_tau"][tau] = {
            "memo_nodes": len(memo),
            "node_expansions": node_expansions,
            "memcpys": memcpys,
            "memcpy_bytes": memcpy_bytes,
            "avg_copy_len": (memcpy_bytes / memcpys) if memcpys else 0.0,
            "lit_writes": lit_writes,
            "total_ops": node_expansions + memcpys + lit_writes,
        }
    return res


# ---------- Graph Acyclic / FVS Order Builder ----------

def build_fvs_acyclic_order(active_syms, dom_buf):
    """
    Construct a static transition-graph acyclic-complement order:
    1. Build directed graph G on active_syms with edge weights = #strips a -> b (a != b).
    2. Iteratively peel vertices with in-degree 0 or out-degree 0 (strictly acyclic!).
    3. When a non-empty core remains (every remaining vertex has in >= 1 and out >= 1),
       pick the highest-weighted-degree (or highest 2-cycle + total degree) vertex as an
       FVS member and remove it from the core to expose more acyclic vertices!
    4. Elimination order: ALL acyclic-complement vertices FIRST (in topological order,
       so eliminating them causes minimal cycle fill-in), followed by FVS vertices in
       reverse removal order (smallest FVS first, heaviest hub LAST)!
    Returns (elim_order_list, fvs_size, acyclic_size).
    """
    adj_out = {u: defaultdict(int) for u in active_syms}
    adj_in = {u: defaultdict(int) for u in active_syms}
    for u in active_syms:
        for pc in dom_buf[u]:
            v = pc[1]
            if v != u:
                adj_out[u][v] += 1
                adj_in[v][u] += 1

    rem = set(active_syms)
    left_topo = []
    right_topo = []
    fvs = []

    while rem:
        progressed = True
        while progressed:
            progressed = False
            sources = [u for u in rem if not any(w in rem for w in adj_in[u])]
            if sources:
                sources.sort(key=lambda u: (sum(adj_out[u][w] for w in rem), u))
                for u in sources:
                    rem.remove(u)
                    left_topo.append(u)
                progressed = True
                continue
            sinks = [u for u in rem if not any(w in rem for w in adj_out[u])]
            if sinks:
                sinks.sort(key=lambda u: (sum(adj_in[u][w] for w in rem), u))
                for u in sinks:
                    rem.remove(u)
                    right_topo.append(u)
                progressed = True
        if rem:
            # Pick highest-feedback vertex to place in FVS (kept for later)
            def hub_score(u):
                out_w = sum(adj_out[u][w] for w in rem)
                in_w = sum(adj_in[u][w] for w in rem)
                out_d = sum(1 for w in adj_out[u] if w in rem)
                in_d = sum(1 for w in adj_in[u] if w in rem)
                return (out_d * in_d, out_w + in_w, u)
            v = max(rem, key=hub_score)
            rem.remove(v)
            fvs.append(v)

    # Acyclic complement in topological order: left_topo + reversed(right_topo)
    # Then FVS from smallest-hub to largest-hub (reversed(fvs))
    acyclic = left_topo + list(reversed(right_topo))
    order = acyclic + list(reversed(fvs))
    return order, len(fvs), len(acyclic)


def count_inversions_self(self_list):
    """Check if self-pieces sorted by dom_lo are monotone in im_lo; count monotone runs and inversions.
    Since len(self_list) <= a few thousand, exact monotone-run count is O(k), and inversion count
    via Fenwick/merge is O(k log k)."""
    k = len(self_list)
    if k <= 1:
        return True, k, 0
    # self_list is already sorted by dom_lo
    ims = [pc[3] for pc in self_list]
    mono_runs = 1
    is_mono = True
    for j in range(1, k):
        if ims[j] < ims[j - 1]:
            is_mono = False
            mono_runs += 1
    if is_mono:
        return True, 1, 0

    # Merge-sort inversion count
    def merge_count(arr):
        m = len(arr)
        if m <= 1:
            return arr, 0
        mid = m // 2
        left, c1 = merge_count(arr[:mid])
        right, c2 = merge_count(arr[mid:])
        out = []
        i = j = 0
        inv = c1 + c2
        nl = len(left)
        while i < nl and j < len(right):
            if left[i] <= right[j]:
                out.append(left[i]); i += 1
            else:
                out.append(right[j]); j += 1
                inv += nl - i
        out.extend(left[i:])
        out.extend(right[j:])
        return out, inv

    _, inv = merge_count(ims)
    return False, mono_runs, inv


# ---------- Full Instrumented IET-BSE Engine (Append-Only Bucket-Buffer Model) ----------

def run_iet_bse_instrumented(runs, p, order_mode="most_freq_first", verify_bytes=None):
    """
    Append-only bucket-buffer IET-BSE (no BWT validity constraint).
    Each bucket b maintains append-only buffers dom_buf[b] and im_buf[b].
    When b is eliminated:
      - Sequentially scan dom_buf[b] and im_buf[b], filtering dead pieces (partner already eliminated).
      - Sort live dom_list[b] by dom_lo and im_list[b] by im_lo.
      - Classify T_self: zero (k_self==0), monotone, or non-monotone.
      - Resolve F_b via exact inner-RV (with 1-piece Zorich).
      - Append emitted composed pieces (src_b, dst_b, ...) to dom_buf[src_b] and im_buf[dst_b].
    Returns comprehensive metrics dict covering (a), (b), (c), (d).
    """
    t_start = time.perf_counter()
    r0 = len(runs)
    n = sum(w for _, w in runs)

    counts = [0] * 256
    for c, w in runs:
        counts[c] += w
    C = [0] * 257
    for c in range(256):
        C[c + 1] = C[c] + counts[c]
    active_syms = [c for c in range(256) if counts[c] > 0]

    # Compact SLP rules: (0, byte) | (1, w1, w2) | (2, base, k)
    rules = [(0, c) for c in range(256)]

    # Build initial bi-monotone strips
    f_cur = list(C[:256])
    dom_buf = {b: [] for b in active_syms}
    im_buf = {b: [] for b in active_syms}

    pos = 0
    im_b = 0
    while im_b < 256 and C[im_b + 1] == 0:
        im_b += 1

    q_init = 0
    for c, w in runs:
        rem = w
        cur_im = pos
        cur_dom = f_cur[c]
        while rem > 0:
            while C[im_b + 1] <= cur_im:
                im_b += 1
            avail = C[im_b + 1] - cur_im
            take = rem if rem <= avail else avail
            pc = (c, im_b, cur_dom, cur_im, take, c, 1)
            dom_buf[c].append(pc)
            im_buf[im_b].append(pc)
            q_init += 1
            cur_im += take
            cur_dom += take
            rem -= take
        f_cur[c] += w
        pos += w

    # Maintain exact live piece counts per bucket for dynamic order heuristics (O(1) update on emit/elim)
    live_dom_cnt = {b: len(dom_buf[b]) for b in active_syms}
    live_im_cnt = {b: len(im_buf[b]) for b in active_syms}
    live_self_cnt = {b: sum(1 for pc in dom_buf[b] if pc[1] == b) for b in active_syms}
    # For min_self_fill: track adjacency pairs (a -> b) counts
    edge_cnt = {b: defaultdict(int) for b in active_syms}
    for b in active_syms:
        for pc in dom_buf[b]:
            if pc[1] != b:
                edge_cnt[b][pc[1]] += 1

    # Static orders precomputed if applicable
    static_order = None
    fvs_info = None
    if order_mode == "most_freq_first":
        static_order = sorted(active_syms, key=lambda b: (-counts[b], b))
    elif order_mode == "rarest_first_rows":
        static_order = sorted(active_syms, key=lambda b: (counts[b], b))
    elif order_mode == "max_sym":
        static_order = sorted(active_syms, reverse=True)
    elif order_mode == "fvs_acyclic_order":
        static_order, fvs_sz, acyc_sz = build_fvs_acyclic_order(active_syms, dom_buf)
        fvs_info = (fvs_sz, acyc_sz)

    eliminated = [False] * 256
    alive = set(active_syms)

    # Track marker p
    p_pos = p
    p_b = 0
    while C[p_b + 1] <= p_pos:
        p_b += 1
    p_shift = 0
    p_cycle = None

    # ---- Metrics accumulators ----
    q_curr = q_init
    max_q = q_init
    q_violations = 0

    buf_reads_live = 0
    buf_reads_stale = 0
    buf_appends = 2 * q_init  # initial writes to dom_buf & im_buf
    total_sort_items = 0

    # Per-step / per-bucket anatomy
    max_bucket_pieces = 0
    max_bucket_rows = 0
    buckets_gt_2k = 0    # > ~48 KiB L1d
    buckets_gt_10k = 0   # > ~256 KiB i9 L2
    steps_in_gt_2k = 0
    steps_in_gt_10k = 0

    # Self-IET breakdown
    zero_self_buckets = 0
    mono_self_buckets = 0
    nonmono_self_buckets = 0

    steps_zero_self = 0       # pure 1D sequential merge steps (k_self == 0)
    steps_mono_merge = 0      # exit-meets-entry in monotone-self bucket
    steps_mono_self = 0       # self-involved steps in monotone-self bucket
    steps_nonmono_merge = 0   # exit-meets-entry in non-monotone-self bucket
    steps_nonmono_self = 0    # self-involved steps in non-monotone-self bucket
    width1_nonmono_self = 0   # width-1 self steps in non-monotone buckets!
    width1_mono_self = 0
    zorich_skips_total = 0

    nonmono_records = []      # detailed list of EVERY non-monotone self-IET encountered
    emitted_total = 0
    emitted_self_fill = 0     # new self-pieces created in surviving buckets!

    order_idx = 0
    while len(alive) > 1:
        if static_order is not None:
            b = static_order[order_idx]
            order_idx += 1
        elif order_mode == "rarest_first_deg":
            b = min(alive, key=lambda x: (live_dom_cnt[x] + live_im_cnt[x], counts[x], x))
        elif order_mode == "min_self_fill":
            # Minimize (current self pieces, #surviving partners with 2-cycle x<->c, degree)
            def msf_key(x):
                two_cyc = 0
                for c, cnt_xc in edge_cnt[x].items():
                    if cnt_xc > 0 and not eliminated[c] and edge_cnt[c].get(x, 0) > 0:
                        two_cyc += 1
                return (live_self_cnt[x], two_cyc, live_dom_cnt[x] + live_im_cnt[x], x)
            b = min(alive, key=msf_key)
        else:
            raise ValueError(order_mode)

        # 1. Sequential filter of dom_buf[b] and im_buf[b]
        raw_d = dom_buf[b]
        raw_i = im_buf[b]
        d_list = [pc for pc in raw_d if not eliminated[pc[1]]]
        i_list = [pc for pc in raw_i if not eliminated[pc[0]]]
        buf_reads_live += len(d_list) + len(i_list)
        buf_reads_stale += (len(raw_d) - len(d_list)) + (len(raw_i) - len(i_list))

        # 2. Local sort inside bucket b
        d_list.sort(key=lambda x: x[2])
        i_list.sort(key=lambda x: x[3])
        total_sort_items += len(d_list) + len(i_list)

        # Separate self-pieces to classify T_self
        self_list = [pc for pc in d_list if pc[1] == b]
        k_self = len(self_list)
        k_out = len(d_list) - k_self
        k_in = len(i_list) - k_self
        b_pieces = k_in + k_out + k_self
        if b_pieces > max_bucket_pieces:
            max_bucket_pieces = b_pieces
        if b_pieces > 2000:
            buckets_gt_2k += 1
        if b_pieces > 10000:
            buckets_gt_10k += 1

        if k_self == 0:
            b_class = 0
            zero_self_buckets += 1
            is_mono, mono_runs, inv = True, 0, 0
        else:
            is_mono, mono_runs, inv = count_inversions_self(self_list)
            if is_mono:
                b_class = 1
                mono_self_buckets += 1
            else:
                b_class = 2
                nonmono_self_buckets += 1

        # Build local nodes for inner-RV inside F_b
        self_map = {}
        nodes_kind = []
        nodes_len = []
        nodes_word = []
        nodes_wlen = []
        nodes_ext_b = []
        nodes_ext_lo = []

        dom_ids = []
        for pc in d_list:
            db, ib, dlo, ilo, ln, wd, wl = pc
            nid = len(nodes_kind)
            if db == b and ib == b:
                self_map[(dlo, ilo, ln)] = nid
                nodes_kind.append(0)
                nodes_ext_b.append(b)
                nodes_ext_lo.append(-1)
            else:
                nodes_kind.append(1)
                nodes_ext_b.append(ib)
                nodes_ext_lo.append(ilo)
            nodes_len.append(ln)
            nodes_word.append(wd)
            nodes_wlen.append(wl)
            dom_ids.append(nid)

        im_ids = []
        for pc in i_list:
            db, ib, dlo, ilo, ln, wd, wl = pc
            if db == b and ib == b:
                nid = self_map[(dlo, ilo, ln)]
            else:
                nid = len(nodes_kind)
                nodes_kind.append(2)
                nodes_ext_b.append(db)
                nodes_ext_lo.append(dlo)
                nodes_len.append(ln)
                nodes_word.append(wd)
                nodes_wlen.append(wl)
            im_ids.append(nid)

        num_nodes = len(nodes_kind)
        dp = [-1] * num_nodes
        dn = [-1] * num_nodes
        ip = [-1] * num_nodes
        inn = [-1] * num_nodes
        for idx, u in enumerate(dom_ids):
            dp[u] = dom_ids[idx - 1] if idx > 0 else -1
            dn[u] = dom_ids[idx + 1] if idx + 1 < len(dom_ids) else -1
        d_tail = dom_ids[-1]
        for idx, u in enumerate(im_ids):
            ip[u] = im_ids[idx - 1] if idx > 0 else -1
            inn[u] = im_ids[idx + 1] if idx + 1 < len(im_ids) else -1
        i_tail = im_ids[-1]

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

        p_in_b = (p_b == b)
        p_nid = -1
        p_off = 0
        if p_in_b:
            acc = C[b]
            for u in dom_ids:
                if acc <= p_pos < acc + nodes_len[u]:
                    p_nid = u
                    p_off = p_pos - acc
                    break
                acc += nodes_len[u]
            assert p_nid != -1

        emitted = []
        b_merge_steps = 0
        b_self_steps = 0
        b_width1_self = 0
        b_zorich = 0

        while d_tail != -1:
            d = d_tail
            i = i_tail
            if d == i:
                assert nodes_kind[d] == 0
                b_self_steps += 1
                if p_in_b and p_nid == d:
                    p_cycle = (nodes_word[d], p_shift, nodes_wlen[d])
                    p_in_b = False; p_nid = -1; p_b = -1
                d_unlink(d); i_unlink(d)
                continue

            kd, ki = nodes_kind[d], nodes_kind[i]
            ld, li = nodes_len[d], nodes_len[i]

            if kd == 1 and ki == 2:
                b_merge_steps += 1
                take = ld if ld < li else li
                src_b = nodes_ext_b[i]
                dst_b = nodes_ext_b[d]
                src_lo = nodes_ext_lo[i] + (li - take)
                dst_lo = nodes_ext_lo[d] + (ld - take)
                cw = len(rules)
                rules.append((1, nodes_word[i], nodes_word[d]))
                cwl = nodes_wlen[i] + nodes_wlen[d]
                emitted.append((src_b, dst_b, src_lo, dst_lo, take, cw, cwl))
                if p_in_b and p_nid == d and p_off >= ld - take:
                    p_b = src_b
                    p_pos = src_lo + (p_off - (ld - take))
                    p_shift += nodes_wlen[i]
                    p_in_b = False; p_nid = -1
                if ld == take: d_unlink(d)
                else: nodes_len[d] = ld - take
                if li == take: i_unlink(i)
                else: nodes_len[i] = li - take

            elif ld > li:
                b_self_steps += 1
                if li == 1:
                    b_width1_self += 1
                if kd == 0:
                    if inn[d] == i:
                        k = (ld - 1) // li
                        if k > 1:
                            b_zorich += k - 1
                        if p_in_b and p_nid == d and p_off >= ld - k * li:
                            m = (ld - 1 - p_off) // li + 1
                            if ki == 2:
                                p_b = nodes_ext_b[i]
                                p_pos = nodes_ext_lo[i] + (p_off - (ld - m * li))
                                p_shift += nodes_wlen[i] + (m - 1) * nodes_wlen[d]
                                p_in_b = False; p_nid = -1
                            else:
                                p_nid = i
                                p_off = p_off - (ld - m * li)
                                p_shift += nodes_wlen[i] + (m - 1) * nodes_wlen[d]
                        pw = nodes_word[d] if k == 1 else len(rules)
                        if k > 1:
                            rules.append((2, nodes_word[d], k))
                        nw = len(rules)
                        rules.append((1, nodes_word[i], pw))
                        nodes_word[i] = nw
                        nodes_wlen[i] += k * nodes_wlen[d]
                        nodes_len[d] = ld - k * li
                        continue
                    if p_in_b and p_nid == d and p_off >= ld - li:
                        if ki == 2:
                            p_b = nodes_ext_b[i]
                            p_pos = nodes_ext_lo[i] + (p_off - (ld - li))
                            p_shift += nodes_wlen[i]
                            p_in_b = False; p_nid = -1
                        else:
                            p_nid = i
                            p_off -= (ld - li)
                            p_shift += nodes_wlen[i]
                    nw = len(rules)
                    rules.append((1, nodes_word[i], nodes_word[d]))
                    nodes_word[i] = nw
                    nodes_wlen[i] += nodes_wlen[d]
                    nodes_len[d] -= li
                    if inn[d] != i:
                        i_unlink(i); i_ins_after(d, i)
                else:
                    if p_in_b and p_nid == d and p_off >= ld - li:
                        p_nid = i
                        p_off -= (ld - li)
                        p_shift += nodes_wlen[i]
                    nw = len(rules)
                    rules.append((1, nodes_word[i], nodes_word[d]))
                    nodes_word[i] = nw
                    nodes_wlen[i] += nodes_wlen[d]
                    nodes_kind[i] = 1
                    nodes_ext_b[i] = nodes_ext_b[d]
                    nodes_ext_lo[i] = nodes_ext_lo[d] + (ld - li)
                    nodes_len[d] -= li
                    i_unlink(i)

            elif li > ld:
                b_self_steps += 1
                if ld == 1:
                    b_width1_self += 1
                if ki == 0:
                    if dn[i] == d:
                        k = (li - 1) // ld
                        if k > 1:
                            b_zorich += k - 1
                        if p_in_b:
                            if p_nid == d:
                                p_shift += k * nodes_wlen[i]
                            elif p_nid == i and p_off >= li - k * ld:
                                m = (li - 1 - p_off) // ld + 1
                                p_nid = d
                                p_off = p_off - (li - m * ld)
                                p_shift += (k - m) * nodes_wlen[i]
                        pw = nodes_word[i] if k == 1 else len(rules)
                        if k > 1:
                            rules.append((2, nodes_word[i], k))
                        nw = len(rules)
                        rules.append((1, pw, nodes_word[d]))
                        nodes_word[d] = nw
                        nodes_wlen[d] += k * nodes_wlen[i]
                        nodes_len[i] = li - k * ld
                        continue
                    if p_in_b:
                        if p_nid == d:
                            p_shift += nodes_wlen[i]
                        elif p_nid == i and p_off >= li - ld:
                            p_nid = d
                            p_off -= (li - ld)
                    nw = len(rules)
                    rules.append((1, nodes_word[i], nodes_word[d]))
                    nodes_word[d] = nw
                    nodes_wlen[d] += nodes_wlen[i]
                    nodes_len[i] -= ld
                    if dn[i] != d:
                        d_unlink(d); d_ins_after(i, d)
                else:
                    src_lo = nodes_ext_lo[i] + (li - ld)
                    if p_in_b and p_nid == d:
                        p_b = nodes_ext_b[i]
                        p_pos = src_lo + p_off
                        p_shift += nodes_wlen[i]
                        p_in_b = False; p_nid = -1
                    nw = len(rules)
                    rules.append((1, nodes_word[i], nodes_word[d]))
                    nodes_word[d] = nw
                    nodes_wlen[d] += nodes_wlen[i]
                    nodes_kind[d] = 2
                    nodes_ext_b[d] = nodes_ext_b[i]
                    nodes_ext_lo[d] = src_lo
                    nodes_len[i] -= ld
                    d_unlink(d)

            else:
                b_self_steps += 1
                if ld == 1:
                    b_width1_self += 1
                if kd == 0 and ki == 0:
                    if p_in_b and p_nid == d:
                        p_nid = i
                        p_shift += nodes_wlen[i]
                    nw = len(rules)
                    rules.append((1, nodes_word[i], nodes_word[d]))
                    nodes_word[i] = nw
                    nodes_wlen[i] += nodes_wlen[d]
                    if inn[d] == i: i_unlink(d)
                    else: i_unlink(i); i_ins_after(d, i); i_unlink(d)
                    d_unlink(d)
                elif kd == 1 and ki == 0:
                    if p_in_b and p_nid == d:
                        p_nid = i
                        p_shift += nodes_wlen[i]
                    nw = len(rules)
                    rules.append((1, nodes_word[i], nodes_word[d]))
                    nodes_word[i] = nw
                    nodes_wlen[i] += nodes_wlen[d]
                    nodes_kind[i] = 1
                    nodes_ext_b[i] = nodes_ext_b[d]
                    nodes_ext_lo[i] = nodes_ext_lo[d]
                    i_unlink(i); d_unlink(d)
                elif kd == 0 and ki == 2:
                    if p_in_b and p_nid == d:
                        p_b = nodes_ext_b[i]
                        p_pos = nodes_ext_lo[i] + p_off
                        p_shift += nodes_wlen[i]
                        p_in_b = False; p_nid = -1
                    nw = len(rules)
                    rules.append((1, nodes_word[i], nodes_word[d]))
                    nodes_word[d] = nw
                    nodes_wlen[d] += nodes_wlen[i]
                    nodes_kind[d] = 2
                    nodes_ext_b[d] = nodes_ext_b[i]
                    nodes_ext_lo[d] = nodes_ext_lo[i]
                    d_unlink(d); i_unlink(i)

        # Accumulate bucket metrics
        b_tot_steps = b_merge_steps + b_self_steps
        if b_pieces > 2000:
            steps_in_gt_2k += b_tot_steps
        if b_pieces > 10000:
            steps_in_gt_10k += b_tot_steps
        zorich_skips_total += b_zorich

        if b_class == 0:
            steps_zero_self += b_merge_steps
        elif b_class == 1:
            steps_mono_merge += b_merge_steps
            steps_mono_self += b_self_steps
            width1_mono_self += b_width1_self
        else:
            steps_nonmono_merge += b_merge_steps
            steps_nonmono_self += b_self_steps
            width1_nonmono_self += b_width1_self
            nonmono_records.append({
                "b": b,
                "rows": counts[b],
                "k_in": k_in,
                "k_out": k_out,
                "k_self": k_self,
                "mono_runs": mono_runs,
                "inversions": inv,
                "self_steps": b_self_steps,
                "merge_steps": b_merge_steps,
                "width1_self": b_width1_self,
                "emitted": len(emitted),
            })

        # 3. Append-only scatter to surviving buckets & update live counters
        eliminated[b] = True
        alive.remove(b)

        # Decrement live counters for deleted pieces touching b
        for pc in i_list:
            sb = pc[0]
            if sb != b:
                live_dom_cnt[sb] -= 1
                edge_cnt[sb][b] -= 1
        for pc in d_list:
            db = pc[1]
            if db != b:
                live_im_cnt[db] -= 1

        # Append emitted composed pieces
        emitted_total += len(emitted)
        for ep in emitted:
            sb, db = ep[0], ep[1]
            dom_buf[sb].append(ep)
            im_buf[db].append(ep)
            buf_appends += 2
            live_dom_cnt[sb] += 1
            live_im_cnt[db] += 1
            if sb == db:
                live_self_cnt[sb] += 1
                emitted_self_fill += 1
            else:
                edge_cnt[sb][db] += 1

        # Free eliminated bucket buffers
        del dom_buf[b]
        del im_buf[b]

        old_q = q_curr
        q_curr = q_curr - (k_in + k_out + k_self) + len(emitted)
        if q_curr > old_q - 1:
            q_violations += 1
        if q_curr > max_q:
            max_q = q_curr

    # ---- Final single remaining bucket ----
    last_b = next(iter(alive))
    raw_d = dom_buf[last_b]
    raw_i = im_buf[last_b]
    d_last = [pc for pc in raw_d if not eliminated[pc[1]]]
    i_last = [pc for pc in raw_i if not eliminated[pc[0]]]
    buf_reads_live += len(d_last) + len(i_last)
    buf_reads_stale += (len(raw_d) - len(d_last)) + (len(raw_i) - len(i_last))

    d_last.sort(key=lambda x: x[2])
    i_last.sort(key=lambda x: x[3])
    total_sort_items += len(d_last) + len(i_last)

    q_last = len(d_last)
    n_last = counts[last_b]
    if q_last > max_bucket_pieces:
        max_bucket_pieces = q_last

    last_is_mono, last_mono_runs, last_inv = count_inversions_self(d_last)

    # Inner RV on final bucket
    piece_id_map = {(pc[2], pc[3], pc[4]): idx for idx, pc in enumerate(d_last)}
    rL = len(d_last)
    dp = [idx - 1 for idx in range(rL)]
    dn = [idx + 1 if idx + 1 < rL else -1 for idx in range(rL)]
    d_tail = rL - 1
    im_order = [piece_id_map[(pc[2], pc[3], pc[4])] for pc in i_last]
    ip = [-1] * rL
    inn = [-1] * rL
    for idx, u in enumerate(im_order):
        ip[u] = im_order[idx - 1] if idx > 0 else -1
        inn[u] = im_order[idx + 1] if idx + 1 < rL else -1
    i_tail = im_order[-1]

    lam = [pc[4] for pc in d_last]
    word = [pc[5] for pc in d_last]
    wlen = [pc[6] for pc in d_last]

    p_piece = -1
    p_off = 0
    if p_cycle is None:
        assert p_b == last_b
        for idx, pc in enumerate(d_last):
            if pc[2] <= p_pos < pc[2] + pc[4]:
                p_piece = idx
                p_off = p_pos - pc[2]
                break
        assert p_piece != -1

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

    final_rv_steps = 0
    final_width1_steps = 0
    while d_tail != -1:
        d = d_tail
        i = i_tail
        final_rv_steps += 1
        if d == i:
            if p_piece == d:
                p_cycle = (word[d], p_shift, wlen[d])
                p_piece = -1
            d_unlink(d); i_unlink(d)
            continue
        ld, li = lam[d], lam[i]
        if ld == 1 or li == 1:
            final_width1_steps += 1
        if ld > li:
            if inn[d] == i:
                k = (ld - 1) // li
                if k > 1: zorich_skips_total += k - 1
                if p_piece == d and p_off >= ld - k * li:
                    m = (ld - 1 - p_off) // li + 1
                    p_piece = i
                    p_off = p_off - (ld - m * li)
                    p_shift += wlen[i] + (m - 1) * wlen[d]
                pw = word[d] if k == 1 else len(rules)
                if k > 1: rules.append((2, word[d], k))
                nw = len(rules); rules.append((1, word[i], pw))
                word[i] = nw; wlen[i] += k * wlen[d]; lam[d] = ld - k * li
                continue
            if p_piece == d and p_off >= ld - li:
                p_piece = i; p_off -= (ld - li); p_shift += wlen[i]
            nw = len(rules); rules.append((1, word[i], word[d]))
            word[i] = nw; wlen[i] += wlen[d]; lam[d] -= li
            if inn[d] != i:
                i_unlink(i); i_ins_after(d, i)
        elif li > ld:
            if dn[i] == d:
                k = (li - 1) // ld
                if k > 1: zorich_skips_total += k - 1
                if p_piece == d:
                    p_shift += k * wlen[i]
                elif p_piece == i and p_off >= li - k * ld:
                    m = (li - 1 - p_off) // ld + 1
                    p_piece = d
                    p_off = p_off - (li - m * ld)
                    p_shift += (k - m) * wlen[i]
                pw = word[i] if k == 1 else len(rules)
                if k > 1: rules.append((2, word[i], k))
                nw = len(rules); rules.append((1, pw, word[d]))
                word[d] = nw; wlen[d] += k * wlen[i]; lam[i] = li - k * ld
                continue
            if p_piece == d:
                p_shift += wlen[i]
            elif p_piece == i and p_off >= li - ld:
                p_piece = d; p_off -= (li - ld)
            nw = len(rules); rules.append((1, word[i], word[d]))
            word[d] = nw; wlen[d] += wlen[i]; lam[i] -= ld
            if dn[i] != d:
                d_unlink(d); d_ins_after(i, d)
        else:
            if p_piece == d:
                p_piece = i; p_shift += wlen[i]
            nw = len(rules); rules.append((1, word[i], word[d]))
            word[i] = nw; wlen[i] += wlen[d]
            if inn[d] == i: i_unlink(d)
            else: i_unlink(i); i_ins_after(d, i); i_unlink(d)
            d_unlink(d)

    assert p_cycle is not None
    root_wd, sh, wl = p_cycle
    assert wl == n

    # Exact byte verification!
    if verify_bytes is not None:
        cyc = expand_slp_bytes(rules, root_wd)
        s = sh % wl
        decoded = cyc[s:] + cyc[:s]
        assert decoded == verify_bytes, f"BYTE MISMATCH in {order_mode}!"

    # Measure SLP expansion stats
    exp_stats = measure_slp_expansion(rules, root_wd, thresholds=(4, 8, 16))
    dt = time.perf_counter() - t_start

    total_inner_steps = (steps_zero_self + steps_mono_merge + steps_mono_self
                         + steps_nonmono_merge + steps_nonmono_self)
    total_all_steps = total_inner_steps + final_rv_steps

    return {
        "order": order_mode,
        "n": n,
        "r0": r0,
        "sigma": len(active_syms),
        "q_init": q_init,
        "max_q": max_q,
        "q_violations": q_violations,
        "fvs_info": fvs_info,
        # (a) piece touches
        "buf_reads_live": buf_reads_live,
        "buf_reads_stale": buf_reads_stale,
        "buf_reads_total": buf_reads_live + buf_reads_stale,
        "buf_appends": buf_appends,
        "emitted_total": emitted_total,
        "emitted_self_fill": emitted_self_fill,
        "total_sort_items": total_sort_items,
        "max_bucket_pieces": max_bucket_pieces,
        "buckets_gt_2k": buckets_gt_2k,
        "buckets_gt_10k": buckets_gt_10k,
        "steps_in_gt_2k": steps_in_gt_2k,
        "steps_in_gt_10k": steps_in_gt_10k,
        # (b) self-IET anatomy
        "zero_self_buckets": zero_self_buckets,
        "mono_self_buckets": mono_self_buckets,
        "nonmono_self_buckets": nonmono_self_buckets,
        "steps_zero_self": steps_zero_self,
        "steps_mono_merge": steps_mono_merge,
        "steps_mono_self": steps_mono_self,
        "steps_nonmono_merge": steps_nonmono_merge,
        "steps_nonmono_self": steps_nonmono_self,
        "width1_mono_self": width1_mono_self,
        "width1_nonmono_self": width1_nonmono_self,
        "zorich_skips": zorich_skips_total,
        "nonmono_records": nonmono_records,
        # (c) final bucket & grammar
        "last_b": last_b,
        "last_n": n_last,
        "last_q": q_last,
        "last_is_mono": last_is_mono,
        "last_mono_runs": last_mono_runs,
        "last_inv": last_inv,
        "final_rv_steps": final_rv_steps,
        "final_width1_steps": final_width1_steps,
        "total_all_steps": total_all_steps,
        "slp_created": len(rules) - 256,
        "slp_reachable": exp_stats["reachable_rules"],
        "slp_height": exp_stats["slp_height"],
        # (d) expansion
        "exp": exp_stats["by_tau"],
        "wall_s": dt,
    }


def main():
    blocks = load_authenticated_blocks()
    print(f"Authenticated all {len(blocks)} blocks via SHA-256 against /tmp/mibz2-blocks/manifest.json!\n")

    orders = [
        "most_freq_first",
        "rarest_first_rows",
        "rarest_first_deg",
        "fvs_acyclic_order",
        "min_self_fill",
        "max_sym",
    ]

    all_results = {}

    for binfo, raw in blocks:
        fname = binfo["file"]
        n, p = binfo["n"], binfo["primary_index"]
        runs = rle_from_bytes(raw)
        r0 = len(runs)
        assert r0 == binfo["r"]
        m16_max, m16_strip, thick_s = compute_open_m16(runs, p, limit=16)
        ref_bytes = decode_lf_reference_bytes(raw, p)

        print("=" * 110)
        print(f"BLOCK {fname} | n={n} r0={r0} ({r0/n:.5f}) sigma={len(set(raw))} | Open thin_max_runs={m16_max} thin_strips={m16_strip} thick_strips={thick_s}")
        print("=" * 110)

        all_results[fname] = {"meta": {"n": n, "r0": r0, "m16_max": m16_max, "m16_strip": m16_strip}, "orders": {}}

        for om in orders:
            st = run_iet_bse_instrumented(runs, p, order_mode=om, verify_bytes=ref_bytes)
            assert st["q_violations"] == 0, f"q_violation in {fname} {om}!"
            # Strip detailed nonmono list for compact JSON, keep summary + top 5
            nm = st["nonmono_records"]
            max_nm_ratio = max((x["self_steps"] / x["k_self"] for x in nm), default=0.0)
            max_nm_row_ratio = max((x["self_steps"] / x["rows"] for x in nm), default=0.0)
            sum_nm_kself = sum(x["k_self"] for x in nm)
            sum_nm_rows = sum(x["rows"] for x in nm)

            print(f"\n  [{om:17s}] VERIFIED EXACT (0 q_viol, {st['wall_s']:.2f}s)" + (f" FVS={st['fvs_info'][0]}/acyc={st['fvs_info'][1]}" if st["fvs_info"] else ""))
            print(f"    (a) Touches: buf_rd={st['buf_reads_total']:7d} (live={st['buf_reads_live']:6d}, stale={st['buf_reads_stale']:6d}) "
                  f"| buf_app={st['buf_appends']:6d} (emit={st['emitted_total']:6d}={st['emitted_total']/r0:.2f}r, self_fill={st['emitted_self_fill']:5d}) "
                  f"| sort_items={st['total_sort_items']:6d} ({st['total_sort_items']/r0:.2f}r)")
            print(f"        Locality: max_b_pcs={st['max_bucket_pieces']:5d} | b>2k:{st['buckets_gt_2k']:2d} (steps={st['steps_in_gt_2k']:6d}) "
                  f"| b>10k:{st['buckets_gt_10k']:2d} (steps={st['steps_in_gt_10k']:6d})")
            print(f"    (b) Buckets: zero_self={st['zero_self_buckets']:2d} (merge={st['steps_zero_self']:5d}) "
                  f"| mono_self={st['mono_self_buckets']:2d} (mrg={st['steps_mono_merge']:5d}, slf={st['steps_mono_self']:5d}, w1={st['width1_mono_self']:5d}) "
                  f"| NON-MONO={st['nonmono_self_buckets']:2d} (mrg={st['steps_nonmono_merge']:5d}, slf={st['steps_nonmono_self']:5d}, w1={st['width1_nonmono_self']:5d})")
            if nm:
                print(f"        Non-mono T_self^*: sum_k_self={sum_nm_kself:5d}, sum_self_steps={st['steps_nonmono_self']:6d} "
                      f"(avg={st['steps_nonmono_self']/sum_nm_kself:.2f}x k_self, max={max_nm_ratio:.2f}x k_self, "
                      f"vs rows={st['steps_nonmono_self']/sum_nm_rows:.4f}, max_row_ratio={max_nm_row_ratio:.4f})")
            print(f"    (c) Final bucket b={st['last_b']} ({repr(bytes([st['last_b']]))}): last_n={st['last_n']:6d}, last_q={st['last_q']:5d}, "
                  f"mono={st['last_is_mono']} (runs={st['last_mono_runs']}, inv={st['last_inv']}), final_rv={st['final_rv_steps']:6d} (w1={st['final_width1_steps']:5d}) "
                  f"| TOT_STEPS={st['total_all_steps']:6d} ({st['total_all_steps']/r0:.2f}r, {st['total_all_steps']/m16_strip:.2f}*m16)")
            print(f"        Grammar: created={st['slp_created']:6d} ({st['slp_created']/r0:.2f}r), reachable={st['slp_reachable']:6d} ({st['slp_reachable']/r0:.2f}r), height={st['slp_height']:4d}")
            e4 = st["exp"][4]
            e8 = st["exp"][8]
            print(f"    (d) Memo-SLP Exp (tau=4): tot_ops={e4['total_ops']:6d} ({e4['total_ops']/m16_strip:.2f}*m16) "
                  f"[exp={e4['node_expansions']:6d}, memcpy={e4['memcpys']:6d} avg={e4['avg_copy_len']:5.1f}B ({100*e4['memcpy_bytes']/n:4.1f}%), lit={e4['lit_writes']:6d}]")
            print(f"        Memo-SLP Exp (tau=8): tot_ops={e8['total_ops']:6d} ({e8['total_ops']/m16_strip:.2f}*m16) "
                  f"[exp={e8['node_expansions']:6d}, memcpy={e8['memcpys']:6d} avg={e8['avg_copy_len']:5.1f}B ({100*e8['memcpy_bytes']/n:4.1f}%), lit={e8['lit_writes']:6d}]")

            # Print top 5 heaviest non-monotone buckets on block-0.L for the 3 primary orders
            if fname == "block-0.L" and om in ("most_freq_first", "rarest_first_rows", "fvs_acyclic_order", "min_self_fill") and nm:
                top5 = sorted(nm, key=lambda x: -x["self_steps"])[:5]
                print(f"        Top-5 non-mono buckets on block-0.L ({om}):")
                for x in top5:
                    print(f"          b={x['b']:3d} ({repr(bytes([x['b']])):6s}) rows={x['rows']:6d} k_in={x['k_in']:4d} k_out={x['k_out']:4d} "
                          f"k_self={x['k_self']:4d} runs={x['mono_runs']:4d} inv={x['inversions']:7d} "
                          f"self_steps={x['self_steps']:5d} ({x['self_steps']/x['k_self']:.2f}x k_self, {x['self_steps']/x['rows']:.3f}x rows) w1={x['width1_self']:4d}")

            st_save = dict(st)
            st_save["nonmono_top5"] = sorted(nm, key=lambda x: -x["self_steps"])[:5]
            del st_save["nonmono_records"]
            all_results[fname]["orders"][om] = st_save

    with open("/tmp/fcp-b/iet_bse_results.json", "w") as f:
        json.dump(all_results, f, indent=2)
    print("\nSaved full JSON results to /tmp/fcp-b/iet_bse_results.json")


if __name__ == "__main__":
    main()
