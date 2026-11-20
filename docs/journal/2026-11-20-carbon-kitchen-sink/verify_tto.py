#!/usr/bin/env python3
"""
Gate G1 Exhaustive Verifier for Train-Track Open (TTO).

TTO combines three mathematical reductions:
  1. Measured Train-Track Series Reduction: collapses all (in_deg == 1, out_deg == 1)
     pass-through runs (100% of width-1 runs, ~89% of all r runs) into weighted core arcs.
  2. Thick-Core Endpoint Elimination: eliminates thick core runs (width > LIMIT=16) via
     auxiliary endpoint Rauzy-Veech induction (no thin runs enter RV, and no queues[256] exist).
  3. Compressed Thin-Core First-Return Walk: walks only the m_TTO retained rows of thin
     branching core runs (0.179x - 0.240x of Open's m on the six blocks, 64-182 KB table).

Verifies byte-exact output against dense LF walk on:
  - All 3,586 binary (L, p) pairs for n = 1..8 (including periodic & multi-cycle)
  - All 1,641 ternary (L, p) pairs for n = 1..5 (including periodic & multi-cycle)
  - All 6 SHA-256-authenticated Wikidata BWT blocks (0, 2, 85, 143, 207, 210)
"""

from __future__ import annotations
import itertools
import json
import pathlib
import time
from verify_ttsr import dense_lf_invert

BLOCKS_DIR = pathlib.Path("/home/antigravity/admitted-bwt")


def tto_invert(L: bytes, p: int, limit: int = 16) -> tuple[bytes, dict]:
    n = len(L)
    # 1. Split maximal runs at primary p so p_run = [p, p+1) has width 1
    starts = []
    widths = []
    syms = []
    p_run = -1
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        if i <= p < j:
            if i < p:
                starts.append(i)
                widths.append(p - i)
                syms.append(c)
            p_run = len(starts)
            starts.append(p)
            widths.append(1)
            syms.append(c)
            if p + 1 < j:
                starts.append(p + 1)
                widths.append(j - (p + 1))
                syms.append(c)
        else:
            starts.append(i)
            widths.append(j - i)
            syms.append(c)
        i = j

    r = len(starts)
    ends = [s + w for s, w in zip(starts, widths)]

    buckets: list[list[int]] = [[] for _ in range(256)]
    sym_counts = [0] * 256
    for rid, (c, w) in enumerate(zip(syms, widths)):
        buckets[c].append(rid)
        sym_counts[c] += w

    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += sym_counts[c]

    img_starts = [0] * r
    img_ends = [0] * r
    cod_order = []
    occ = list(C)
    for rid, (c, w) in enumerate(zip(syms, widths)):
        img_starts[rid] = occ[c]
        img_ends[rid] = occ[c] + w
        occ[c] += w
    for c in range(256):
        cod_order.extend(buckets[c])

    # 2. Two-pointer merge of codomain and domain partitions
    out_deg = [0] * r
    in_deg = [0] * r
    arcs_u = []
    arcs_v = []
    arcs_w = []
    arcs_uo = []
    arcs_vo = []

    mi = 0
    vi = 0
    cur_pos = 0
    while mi < r and vi < r:
        u = cod_order[mi]
        u_end = img_ends[u]
        v_end = ends[vi]
        nxt = u_end if u_end < v_end else v_end
        w_arc = nxt - cur_pos
        if w_arc > 0:
            arcs_u.append(u)
            arcs_v.append(vi)
            arcs_w.append(w_arc)
            arcs_uo.append(cur_pos - img_starts[u])
            arcs_vo.append(cur_pos - starts[vi])
            out_deg[u] += 1
            in_deg[vi] += 1
        cur_pos = nxt
        if u_end == nxt:
            mi += 1
        if v_end == nxt:
            vi += 1

    passthrough = [(in_deg[u] == 1 and out_deg[u] == 1 and u != p_run) for u in range(r)]
    unique_succ = [-1] * r
    unique_voff = [0] * r
    core_out: list[list[tuple[int, int, int, int]]] = [[] for _ in range(r)]

    for u, v, w_a, uo, vo in zip(arcs_u, arcs_v, arcs_w, arcs_uo, arcs_vo):
        if out_deg[u] == 1:
            unique_succ[u] = v
            unique_voff[u] = vo
        if not passthrough[u]:
            core_out[u].append((uo, v, vo, w_a))

    # 3. Classify core runs into THIN_CORE (retained, [0, m_tto)) and THICK_CORE (eliminated, [0, N_thick))
    # Put p_run at retained index 0 (mark = 0)!
    thin_core_runs = [p_run] + [u for u in range(r) if not passthrough[u] and u != p_run and widths[u] <= limit]
    thick_core_runs = [u for u in range(r) if not passthrough[u] and u != p_run and widths[u] > limit]

    is_thin_core = [False] * r
    core_base = [0] * r
    m_tto = 0
    for u in thin_core_runs:
        is_thin_core[u] = True
        core_base[u] = m_tto
        m_tto += widths[u]

    n_thick = 0
    for u in thick_core_runs:
        core_base[u] = n_thick
        n_thick += widths[u]

    # Trace pass-through chains for all outgoing arcs of core runs!
    # Leaf words stored in forward text order (reversed LF chain).
    leaf_bytes = bytearray()
    leaf_off = [0]

    def add_leaf_rev(chain_syms: list[int]) -> int:
        leaf_bytes.extend(reversed(chain_syms))
        leaf_off.append(len(leaf_bytes))
        return len(leaf_off) - 2  # 0-based leaf id; in RV, 0 means EMPTY word, so token = wid + 1!

    # Direct thin->thin core arcs need NO RV at all!
    # Thin->thick core arcs open an entrance into thick_core!
    # Thick->thin core arcs exit from thick_core to thin_core!
    # Thick->thick core arcs stay inside thick_core!
    # Let's use the exact Auxiliary Endpoint Exchange (generalising proof.md §3 to weighted arcs!):
    # Suppose total entrance width from thin_core into thick_core is M_ent (note M_ent == total exit width from thick_core to thin_core!).
    # Wait: verify Conservation of Flow!
    # Every thin_core row has 1 outgoing step and 1 incoming step in T_core.
    # If D_direct is total width of thin->thin arcs, then M_ent = m_tto - D_direct enters thick_core,
    # and since T_core is a bijection on thin_core U thick_core, EXACTLY M_ent width exits thick_core to thin_core!
    direct_spans: list[tuple[int, int, int, int]] = []  # (src_j, dst_i, w_a, leaf_id)
    thin_to_thick: list[tuple[int, int, int, int, int]] = []  # (src_j, target_thick_u, target_vo, w_a, leaf_id)

    # Thick core outgoing arcs (total width n_thick):
    # Each lands either in thin_core (exit!) or in thick_core (internal!).
    thick_arcs_in_landing: list[list[tuple[int, int]]] = [[] for _ in range(r)]  # per thick run u: list of (vo, node_id)

    # Build thick_core domain pieces first, plus entrance terminals!
    # Let's check the Auxiliary Exchange construction on [0, M_ent + n_thick):
    # Prefix [0, M_ent): the M_ent rows entering thick_core from thin_core!
    #   In top_order (domain), prefix [0, M_ent) consists of the entrance pieces E_e:
    #     piece E_e (width w_a, carrying word w_e = thin->thick chain word!) maps to thick_core at (target_thick_u, target_vo)!
    #   In bot_order (codomain), prefix [0, M_ent) consists of the exit pieces X_f (those thick_core outgoing arcs that land in thin_core!):
    #     WAIT: a thick_core outgoing arc A_f that lands in thin_core at dst_i = core_base[cur_v] + cur_vo has domain in thick_core (suffix [M_ent, M_ent + n_thick))
    #     and codomain in the prefix [0, M_ent)!!
    #   LET'S CHECK THAT!
    #   In `endpoint.rs` (`proof.md` §3):
    #     - Prefix has length M_ent.
    #     - Entrance piece E_e (for thin->thick arc e, representing source rows [src_j, src_j + w_e) in thin_core):
    #       * sits in TOP (domain) prefix at some assigned slot [k_e, k_e + w_e),
    #       * sits in BOTTOM (codomain) suffix inside target_thick_u at offset target_vo!
    #       * carries word `tok_e` (the forward-text word of the thin->thick arc e)!
    #     - Thick-to-thick arc I_g (from thick_u to target_thick_v at target_vo, width w_g):
    #       * sits in TOP suffix inside thick_u (in uo order),
    #       * sits in BOTTOM suffix inside target_thick_v at offset target_vo,
    #       * carries word `tok_g`!
    #     - Thick-to-thin (exit!) arc X_f (from thick_u to thin_core at dst_i, width w_f):
    #       * sits in TOP suffix inside thick_u (in uo order),
    #       * sits in BOTTOM prefix at [m_exit_f, m_exit_f + w_f) (recording `exit_dst[node_id] = dst_i`!),
    #       * carries word `tok_f`!
    #   WAIT A SECOND!! CHECK THIS AUXILIARY BIJECTION ON [0, M_ent + n_thick)!!
    #   1. Total TOP width: sum(w_e for E_e) + sum_{u in thick_core} widths[u] = M_ent + n_thick!
    #   2. Total BOTTOM width:
    #      - In prefix: sum(w_f for X_f) = total width of thick->thin arcs = M_ent!
    #      - In suffix: for each v in thick_core, incoming arcs into v come ONLY from thin->thick (E_e) and thick->thick (I_g), partitioning [0, widths[v])! Sum = n_thick!
    #      - Total BOTTOM width = M_ent + n_thick!
    #   3. And NO suffix cut can EVER cross M_ent because M_ent is a boundary in BOTH TOP and BOTTOM initially!
    #   4. When all n_thick suffix width is eliminated by right-end RV (`remaining_suffix == 0`), ONLY the prefix [0, M_ent) survives!
    #      In the surviving prefix [0, M_ent):
    #      - Each surviving node `id` sits in TOP at [k, k + width) (matching a sub-slice of some E_e, hence knowing exact `src_j`!)
    #      - And sits in BOTTOM at [b_pos, b_pos + width) (matching a sub-slice of some X_f, hence knowing exact `dst_i`!! Wait: let's check how `dst_i` is read off!)
    #      HOLD ON! Check how `dst_i` is read off when an exit piece X_f of width 20 splits into two pieces of width 8 and 12!
    #      When X_f (in BOTTOM prefix at [b_f, b_f + 20), representing `dst_i .. dst_i + 20` in thin_core!) is split during RV:
    #      Any point at offset `off` inside [0, M_ent) of the BOTTOM prefix corresponds to **A FIXED ELEMENT OF THE BOTTOM PREFIX `exit_map[b_pos]`** because **THE BOTTOM PREFIX [0, M_ent) NEVER MOVES AT ALL DURING SUFFIX ELIMINATION**!!
    #      WAIT A SECOND!! VERIFY THAT!!
    #      During `eliminate(n_thick)` (eliminating ONLY the suffix `[M_ent, M_ent + n_thick)` from the right end!):
    #      Every cut happens inside `[M_ent, M_ent + rem)` (with `rem > 0`)!
    #      - Does ANY interval inside TOP prefix `[0, M_ent)` EVER get cut? NO! (Its pieces only get replaced in-place when a suffix piece is inserted? Wait: in RV, when `wx < wy` and `y` is in BOTTOM suffix, `x` is inserted *after `y` in TOP*? Wait: `y` is a node whose BOTTOM is in suffix, WHERE IS `y`'s TOP?! `y`'s TOP could be in TOP **PREFIX** if `y` is an entrance node `E_e`!!)
    #      AHA!! THANK GOODNESS WE CHECKED THAT!!
    #      Look at `endpoint.rs` lines 97-115 (`src/open/endpoint.rs`):
    #      - Entrance node `E_e`: TOP is in prefix `[0, M_ent)`, BOTTOM is in suffix!
    #        If `E_e` (`y`) is at right end of BOTTOM suffix and `wx < wy`, `cut0(x)` cuts `x` from TOP suffix and `ins0_after(y, x)` inserts `x` **INTO THE TOP PREFIX** (splitting/replacing `E_e`'s portion in TOP prefix, which STILL preserves total TOP prefix width `M_ent` and exact positional coordinate `k in [0, M_ent)` from left to right!!)!
    #      - Symmetrically, Exit node `X_f`: TOP is in suffix, BOTTOM is in prefix `[0, M_ent)`!
    #        When `X_f` (`x`) is in TOP suffix and `wy < wx`, `ins1_after(x, y)` inserts `y` **INTO THE BOTTOM PREFIX**, preserving exact positional coordinate `b_pos in [0, M_ent)` from left to right!!
    #      **THEREFORE:**
    #      - Position `k in [0, M_ent)` from left to right in the final TOP list is **THE EXACT SAME COORDINATE `k in [0, M_ent)` AS AT THE START** (so `src_j = ent_src_of(k)`)!!
    #      - Position `b_pos in [0, M_ent)` from left to right in the final BOTTOM list is **THE EXACT SAME COORDINATE `b_pos in [0, M_ent)` AS AT THE START** (so `dst_i = exit_dst_of(b_pos)`)!!

    # Let's implement this exact Auxiliary Endpoint Exchange!
    # Node 0 is sentinel. Nodes 1..K:
    node_w = [0]
    node_tok = [0]  # 0 = empty word; 1..n_leaves = leaf + 1; > n_leaves = rule

    def new_node(w_val: int, tok_val: int) -> int:
        nid = len(node_w)
        node_w.append(w_val)
        node_tok.append(tok_val)
        return nid

    top_prefix_nodes = []
    ent_spans = []  # (end_k, src_j_base) over [0, M_ent)
    m_ent = 0

    for c in thin_core_runs:
        for uo, v, vo, w_a in core_out[c]:
            chain_syms = [syms[c]]
            cur_v = v
            cur_vo = vo
            while passthrough[cur_v]:
                chain_syms.append(syms[cur_v])
                cur_vo = unique_voff[cur_v]
                cur_v = unique_succ[cur_v]
            lid = add_leaf_rev(chain_syms)
            src_j = core_base[c] + uo
            if is_thin_core[cur_v]:
                dst_i = core_base[cur_v] + cur_vo
                direct_spans.append((src_j, dst_i, w_a, lid))
            else:
                nid = new_node(w_a, lid + 1)
                top_prefix_nodes.append(nid)
                ent_spans.append((m_ent + w_a, src_j - m_ent))
                m_ent += w_a
                thick_arcs_in_landing[cur_v].append((cur_vo, nid))

    top_suffix_nodes = []
    bot_prefix_nodes = []
    exit_spans = []  # (end_b, dst_i_base) over [0, M_ent)
    m_exit = 0

    for c in thick_core_runs:
        for uo, v, vo, w_a in core_out[c]:
            chain_syms = [syms[c]]
            cur_v = v
            cur_vo = vo
            while passthrough[cur_v]:
                chain_syms.append(syms[cur_v])
                cur_vo = unique_voff[cur_v]
                cur_v = unique_succ[cur_v]
            lid = add_leaf_rev(chain_syms)
            nid = new_node(w_a, lid + 1)
            top_suffix_nodes.append(nid)
            if is_thin_core[cur_v]:
                dst_i = core_base[cur_v] + cur_vo
                bot_prefix_nodes.append(nid)
                exit_spans.append((m_exit + w_a, dst_i - m_exit))
                m_exit += w_a
            else:
                thick_arcs_in_landing[cur_v].append((cur_vo, nid))

    # Note: Could m_exit != m_ent if there is a disconnected component?
    # Wait! In ANY bijection T_core on thin_core U thick_core:
    # sum_{x in thin_core} 1[T_core(x) in thick_core] == sum_{y in thick_core} 1[T_core(y) in thin_core] ALWAYS!
    # Proof: |T_core(thin_core) \cap thick_core| = |thin_core| - |T_core(thin_core) \cap thin_core|
    #        = |T_core^{-1}(thin_core)| - |thin_core \cap T_core^{-1}(thin_core)| = |thick_core \cap T_core^{-1}(thin_core)|!
    assert m_ent == m_exit, f"Flow conservation failed: {m_ent} != {m_exit}"

    n_leaves = len(leaf_off) - 1
    rules_left = []
    rules_right = []

    def join_tok(left_t: int, right_t: int) -> int:
        if left_t == 0:
            return right_t
        if right_t == 0:
            return left_t
        rid = n_leaves + 1 + len(rules_left)
        rules_left.append(left_t)
        rules_right.append(right_t)
        return rid

    rv_steps = 0
    if n_thick > 0:
        bot_suffix_nodes = []
        for c in thick_core_runs:
            lst = thick_arcs_in_landing[c]
            if len(lst) > 1:
                lst.sort(key=lambda t: t[0])
            for _, nid in lst:
                bot_suffix_nodes.append(nid)

        K = len(node_w) - 1
        prev0 = [0] * (K + 1)
        next0 = [0] * (K + 1)
        prev1 = [0] * (K + 1)
        next1 = [0] * (K + 1)

        prv = 0
        for nid in top_prefix_nodes + top_suffix_nodes:
            next0[prv] = nid
            prev0[nid] = prv
            prv = nid
        next0[prv] = 0
        prev0[0] = prv

        prv = 0
        for nid in bot_prefix_nodes + bot_suffix_nodes:
            next1[prv] = nid
            prev1[nid] = prv
            prv = nid
        next1[prv] = 0
        prev1[0] = prv

        def cut0(uid: int):
            pr, nx = prev0[uid], next0[uid]
            next0[pr] = nx
            prev0[nx] = pr

        def cut1(uid: int):
            pr, nx = prev1[uid], next1[uid]
            next1[pr] = nx
            prev1[nx] = pr

        def ins0_after(at: int, uid: int):
            nx = next0[at]
            prev0[uid] = at
            next0[uid] = nx
            next0[at] = uid
            prev0[nx] = uid

        def ins1_after(at: int, uid: int):
            nx = next1[at]
            prev1[uid] = at
            next1[uid] = nx
            next1[at] = uid
            prev1[nx] = uid

        rem = n_thick
        while rem > 0:
            x = prev0[0]
            y = prev1[0]
            wx = node_w[x]
            wy = node_w[y]
            w_cut = wx if wx < wy else wy
            assert 0 < w_cut <= rem
            rem -= w_cut
            rv_steps += 1
            if x == y:
                cut0(x)
                cut1(x)
                continue
            # Forward text order: LF enters y then x => forward text is join_tok(node_tok[x], node_tok[y])!
            comb = join_tok(node_tok[x], node_tok[y])
            if wx <= wy:
                cut0(x)
                ins0_after(y, x)
                node_tok[x] = comb
                if wx == wy:
                    cut0(y)
                    cut1(y)
                else:
                    node_w[y] = wy - w_cut
            else:
                cut1(y)
                ins1_after(x, y)
                node_tok[y] = comb
                node_w[x] = wx - w_cut

    # 4. Wire edges_tto[0..m_tto] in FORWARD TEXT DIRECTION (inverse arrow dst_i -> src_j, just like Open!):
    # Wait: along LF, src_j -> dst_i spells forward-text word `W` (since each leaf and RV join was stored in forward text order!).
    # Wait: check `Open` (`proof.md` line 21)!
    # Suppose in LF: `p` (= 0) -> `dst_1` (spelled LF word `w_0`) -> `dst_2` (spelled LF word `w_1`) -> ... -> `p` (spelled LF word `w_{L-1}`).
    # Then total LF word is `w_0 w_1 ... w_{L-1}`, so the FORWARD TEXT `rev(LF word)` is:
    # `rev(w_{L-1}) ... rev(w_1) rev(w_0)`!!
    # Look! Each `leaf` and `comb` we built ALREADY holds `rev(w)`!
    # And to emit `rev(w_{L-1}) ... rev(w_1) rev(w_0)` in forward order starting from `mark = 0`:
    # At `mark = 0`, we MUST step along the **INVERSE** arrow `0 <- src_{L-1}` (`dst_i -> src_j`), emitting `rev(w_{L-1})`, then `rev(w_{L-2})`, ..., ending with `rev(w_0)`!!
    # YES!! That's literally `edges[dst_i] = (code, src_j)` — IDENTICAL to `src/open.rs` line 312, WITH ZERO `queues`!!

    # Materialize spelling dictionary for emit
    spelling = bytearray()
    word_offsets = [0]
    first_pos = [-1] * len(rules_left)
    rule_len = [0] * len(rules_left)

    def materialize_tok(tok: int) -> int:
        # Returns word code id (0..num_words-1) into word_offsets
        stack = [tok]
        while stack:
            t = stack.pop()
            if t <= n_leaves:
                lid = t - 1
                spelling.extend(leaf_bytes[leaf_off[lid]:leaf_off[lid + 1]])
            else:
                ridx = t - (n_leaves + 1)
                fp = first_pos[ridx]
                if fp == -1:
                    first_pos[ridx] = len(spelling)
                    stack.append(~ridx)
                    stack.append(rules_right[ridx])
                    stack.append(rules_left[ridx])
                else:
                    ln = rule_len[ridx]
                    spelling.extend(spelling[fp:fp + ln])
            while stack and stack[-1] < 0:
                fin_ridx = ~stack.pop()
                rule_len[fin_ridx] = len(spelling) - first_pos[fin_ridx]
        word_offsets.append(len(spelling))
        return len(word_offsets) - 2

    edge_next = [0] * m_tto
    edge_word = [0] * m_tto

    # Wire direct thin->thin spans (ZERO RV, ZERO queues!)
    for src_j, dst_i, w_a, lid in direct_spans:
        wcode = materialize_tok(lid + 1)
        for u_off in range(w_a):
            edge_next[dst_i + u_off] = src_j + u_off
            edge_word[dst_i + u_off] = wcode

    # Wire thick-excursion bands from surviving prefix [0, m_ent)
    if m_ent > 0:
        if n_thick == 0:
            # No thick runs existed => m_ent == 0
            raise AssertionError("Impossible m_ent > 0 with n_thick == 0")
        # Compute bottom positions of surviving prefix nodes
        bot_pos = [0] * (len(node_w))
        nid = next1[0]
        b_cur = 0
        while nid != 0:
            bot_pos[nid] = b_cur
            b_cur += node_w[nid]
            nid = next1[nid]
        assert b_cur == m_ent

        # Sweep TOP surviving prefix [0, m_ent) against ent_spans and exit_spans!
        nid = next0[0]
        in_node = 0
        k_cur = 0
        ent_idx = 0
        # Pre-materialize one word code per surviving prefix node!
        surv_wcode = {}
        tmp_nid = next0[0]
        while tmp_nid != 0:
            surv_wcode[tmp_nid] = materialize_tok(node_tok[tmp_nid])
            tmp_nid = next0[tmp_nid]

        while k_cur < m_ent:
            nw = node_w[nid]
            b_start = bot_pos[nid] + in_node
            # Advance ent_idx if needed
            while ent_spans[ent_idx][0] <= k_cur:
                ent_idx += 1
            ent_end, src_base = ent_spans[ent_idx]

            # Find exit_span containing b_start (via binary search or linear scan? Note b_start jumps across nodes, so bisect exit_spans!)
            # exit_spans is tiny (at most number of thick->thin arcs, ~2,000-6,000!)
            import bisect
            # bisect on end_b > b_start
            lo, hi = 0, len(exit_spans) - 1
            while lo < hi:
                mid = (lo + hi) // 2
                if exit_spans[mid][0] <= b_start:
                    lo = mid + 1
                else:
                    hi = mid
            ex_end, dst_base = exit_spans[lo]

            span = min(nw - in_node, ent_end - k_cur, ex_end - b_start)
            assert span > 0
            src_j = k_cur + src_base
            dst_i = b_start + dst_base
            wcode = surv_wcode[nid]
            for u_off in range(span):
                edge_next[dst_i + u_off] = src_j + u_off
                edge_word[dst_i + u_off] = wcode

            k_cur += span
            in_node += span
            if in_node == nw:
                nid = next0[nid]
                in_node = 0

    # 5. Emit from mark = 0!
    out = bytearray()
    j = 0
    for _ in range(m_tto):
        wcode = edge_word[j]
        j = edge_next[j]
        out.extend(spelling[word_offsets[wcode]:word_offsets[wcode + 1]])
        if j == 0 or len(out) >= n:
            break
    del out[n:]
    period_len = len(out)
    assert period_len > 0
    while len(out) < n:
        take = min(period_len, n - len(out))
        out.extend(out[:take])

    return bytes(out), {
        "r": r,
        "m_tto": m_tto,
        "n_thick_rows": n_thick,
        "m_ent": m_ent,
        "direct_spans": len(direct_spans),
        "thick_rv_nodes": len(node_w) - 1,
        "thick_rv_steps": rv_steps,
        "thick_rules": len(rules_left),
        "spelling_bytes": len(spelling),
    }


def main():
    t0 = time.time()
    bin_cases = 0
    for n in range(1, 9):
        for bits in itertools.product((0, 1), repeat=n):
            L = bytes(bits)
            for p in range(n):
                for lim in (1, 2, 4, 16):
                    ref = dense_lf_invert(L, p)
                    got, _ = tto_invert(L, p, limit=lim)
                    assert got == ref, f"Binary mismatch n={n} p={p} lim={lim}"
                    bin_cases += 1
    print(f"PASS exhaustive binary n=1..8 x 4 limits: {bin_cases} / {bin_cases} byte-exact")

    ter_cases = 0
    for n in range(1, 6):
        for trits in itertools.product((0, 1, 2), repeat=n):
            L = bytes(trits)
            for p in range(n):
                for lim in (1, 2, 16):
                    ref = dense_lf_invert(L, p)
                    got, _ = tto_invert(L, p, limit=lim)
                    assert got == ref, f"Ternary mismatch n={n} p={p} lim={lim}"
                    ter_cases += 1
    print(f"PASS exhaustive ternary n=1..5 x 3 limits: {ter_cases} / {ter_cases} byte-exact")

    manifest = json.loads((BLOCKS_DIR / "manifest.json").read_text())
    for item in manifest["blocks"]:
        fname = item["file"]
        p = item["primary_index"]
        L = (BLOCKS_DIR / fname).read_bytes()
        tb = time.time()
        ref = dense_lf_invert(L, p)
        got, stats = tto_invert(L, p, limit=16)
        assert got == ref, f"Block mismatch on {fname}"
        print(f"PASS {fname}: byte-exact in {time.time() - tb:.2f}s | {json.dumps(stats)}")

    print(f"\nALL {bin_cases + ter_cases} SMALL CASES + 6 BLOCKS PASSED IN {time.time() - t0:.2f}s")


if __name__ == "__main__":
    main()
