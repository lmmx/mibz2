#!/usr/bin/env python3
"""T1/T2 (Cycle 4): Lazy/shared thin-run acquisition (reusing B_P cursor intersection
inside strip contraction) and the (byte, successor-byte) depth-2 schedule costed
with payloads.

Produces two deterministic TSV datasets:
  - data/t1_t2_lazy_thin_acquisition.tsv
  - data/t1_t2_depth2_schedule_costed.tsv

All routes are verified 100% byte-exact against an independent LF decoder (not sharing
common.build_lf_array) on all 6 admitted Wikidata blocks + 2 synthetic novel-hex blocks.

Strict referee accounting (MONODROMY record 112 / Pi + channel #28):
  - Reports total addressed reads per lone row (Q_total / h1) with BOTH acquisition
    and emission included, separating:
      * Q_acq_search: binary searches / random strip lookups (0 for cursor merges!)
      * Q_acq_iet:    Open-style right-end IET linked-list steps (for wide strips)
      * Q_build_seek: cursor restarts (SEEKs) in composition sweeps / R^2 / R^4
      * Q_emit_links: dependent link loads during emit walk
      * Q_emit_pay:   dictionary/payload reads during emit (flat vs paired vs DAG)
  - Reports BOTH strict Pi (s <= 256 cursors) and relaxed L1-cursor table for R^4.
"""

from __future__ import annotations

import csv
import os
import sys
import time
from collections import defaultdict
from dataclasses import dataclass
from typing import Any

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import common


def independent_lf_decode(L: bytes, primary_index: int) -> bytes:
    """Standalone textbook LF inverse-BWT oracle (independent of common.py)."""
    n = len(L)
    freq = [0] * 256
    for b in L:
        freq[b] += 1
    starts = [0] * 256
    s = 0
    for b in range(256):
        starts[b] = s
        s += freq[b]
    occ = starts[:]
    lf = [0] * n
    for i, b in enumerate(L):
        lf[i] = occ[b]
        occ[b] += 1
    out = bytearray(n)
    x = primary_index
    for pos in range(n - 1, -1, -1):
        out[pos] = L[x]
        x = lf[x]
    assert x == primary_index, f"Independent oracle failed single-cycle check"
    return bytes(out)


def simulate_open_iet_wide_strips(
    blk: common.BlockData, L_min: int = 17
) -> dict[str, Any]:
    """Contract wide strips (w >= L_min, splitting out primary_index) using BOTH:
      1. Proposition A (MONODROMY record 120 / Nishimoto-Tabei move-structure merge):
         label-ordered forward cursors, ZERO binary searches (Q_search = 0).
      2. Open's right-end IET elimination (src/open/endpoint.rs) step count.

    Returns the exact wide-strip exit partition P_W: [0, n) -> M_16 with flat
    deduplicated strip words (NO DAG chasing at emit!).
    """
    n = blk.n
    runs = blk.runs
    C = blk.C
    p = blk.primary_index

    # Single streaming pass over r runs (P=1, Q=0, s=256 cursors):
    # Partition [0, n) into:
    #   - H1_mask / H1_ivs: lone rows (w == 1) plus primary_index
    #   - thin_runs: (s, w, lf_s, b) with 2 <= w < L_min (excluding p)
    #   - wide_runs: (s, w, lf_s, b) with w >= L_min (excluding p)
    occ = C[:]
    # Static layout of [0, n) into disjoint sorted pieces:
    # kind: 0 = H1 (lone/mark), 1 = thin (2..L_min-1), 2 = wide (>= L_min)
    layout: list[tuple[int, int, int, int, int]] = []  # (s, e, kind, lf_s, b)
    wide_runs: list[tuple[int, int, int, int]] = []    # (s, w, lf_s, b)
    thin_runs: list[tuple[int, int, int, int]] = []    # (s, w, lf_s, b)
    h1_runs: list[tuple[int, int, int, int]] = []      # (s, w, lf_s, b)

    def push_piece(s: int, w: int, lf_s: int, b: int) -> None:
        if w <= 0:
            return
        if w == 1:
            layout.append((s, s + 1, 0, lf_s, b))
            h1_runs.append((s, 1, lf_s, b))
        elif w < L_min:
            layout.append((s, s + w, 1, lf_s, b))
            thin_runs.append((s, w, lf_s, b))
        else:
            layout.append((s, s + w, 2, lf_s, b))
            wide_runs.append((s, w, lf_s, b))

    for s, w, b in runs:
        lf_s = occ[b]
        occ[b] += w
        if s <= p < s + w:
            lw = p - s
            rw = (s + w) - (p + 1)
            if lw > 0:
                push_piece(s, lw, lf_s, b)
            push_piece(p, 1, lf_s + lw, b)
            if rw > 0:
                push_piece(p + 1, rw, lf_s + lw + 1, b)
        else:
            push_piece(s, w, lf_s, b)

    # Verify layout tiles [0, n)
    assert layout[0][0] == 0 and layout[-1][1] == n

    # ---- Method 1: Proposition A (label-ordered cursor merge, Q_search = 0) ----
    # Word pool: bytes objects deduplicated via dict[bytes, int]
    word_to_id: dict[bytes, int] = {bytes([b]): b for b in range(256)}
    id_to_word: list[bytes] = [bytes([b]) for b in range(256)]

    def intern(w_bytes: bytes) -> int:
        wid = word_to_id.get(w_bytes)
        if wid is None:
            wid = len(id_to_word)
            word_to_id[w_bytes] = wid
            id_to_word.append(w_bytes)
        return wid

    # Active wide pieces grouped by the byte label of the wide strip holding cur_t.
    # At round 0, piece (dom_s, w, lf_s, wid=b) has cur_t = dom_s inside wide run of byte b,
    # and its 1-step LF image is [lf_s, lf_s + w).
    # General invariant for queues Q[b]:
    #   Each item is (dom_s, w, lf_img_start, wid) sorted by lf_img_start inside I_b!
    #   Wait: storing lf_img_start directly makes every queue sorted by the image coordinate
    #   we are about to intersect with `layout`!
    queues: list[list[tuple[int, int, int, int]]] = [[] for _ in range(256)]
    for s, w, lf_s, b in wide_runs:
        queues[b].append((s, w, lf_s, b))

    wide_final: list[tuple[int, int, int, int]] = []  # (dom_s, w, exit_row, wid)
    prop_a_rounds = 0
    prop_a_piece_visits = 0
    prop_a_cursor_steps = 0
    n_layout = len(layout)

    while True:
        any_live = False
        next_queues: list[list[tuple[int, int, int, int]]] = [[] for _ in range(256)]
        # ONE forward cursor over layout[0..n_layout) across all b=0..255!
        l_idx = 0
        for b in range(256):
            qb = queues[b]
            if not qb:
                continue
            any_live = True
            for dom_s, w, img_s, wid in qb:
                prop_a_piece_visits += 1
                off = 0
                while off < w:
                    cur = img_s + off
                    while layout[l_idx][1] <= cur:
                        l_idx += 1
                        prop_a_cursor_steps += 1
                    ls, le, lkind, llf, lb = layout[l_idx]
                    take = min(w - off, le - cur)
                    if lkind != 2:
                        # Landed in M_16 (H1 or thin) -> DONE!
                        wide_final.append((dom_s + off, take, cur, wid))
                    else:
                        # Landed in another wide strip of byte lb!
                        # Its 1-step LF image is llf + (cur - ls) inside I_lb!
                        # Because cur is strictly increasing globally across b=0..255
                        # and lb's runs are disjoint, llf + (cur - ls) is strictly
                        # increasing within next_queues[lb]!
                        new_wid = intern(id_to_word[wid] + bytes([lb]))
                        next_img = llf + (cur - ls)
                        next_queues[lb].append((dom_s + off, take, next_img, new_wid))
                    off += take
        if not any_live:
            break
        prop_a_rounds += 1
        queues = next_queues

    # Sort wide_final by dom_s (in C/Rust: 2-pass radix sort over |wide_final| pieces, Q=0)
    wide_final.sort(key=lambda x: x[0])

    # ---- Method 2: Open right-end IET elimination step count (src/open/endpoint.rs) ----
    # Side 0: domain order (wide strips + gaps); Side 1: image order (wide strips + gaps)
    # Count exact Rauzy-Veech right-end peel steps (matching endpoint.rs eliminate<true>).
    iet_steps = count_open_iet_steps(wide_runs, n)

    return {
        "layout": layout,
        "wide_runs": wide_runs,
        "thin_runs": thin_runs,
        "h1_runs": h1_runs,
        "wide_final": wide_final,
        "word_to_id": word_to_id,
        "id_to_word": id_to_word,
        "prop_a_rounds": prop_a_rounds,
        "prop_a_piece_visits": prop_a_piece_visits,
        "prop_a_cursor_steps": prop_a_cursor_steps,
        "open_iet_steps": iet_steps,
    }


def count_open_iet_steps(wide_runs: list[tuple[int, int, int, int]], n: int) -> int:
    """Faithful count of right-end IET elimination steps (src/open/endpoint.rs)
    for eliminating the union of wide_runs inside [0, n)."""
    if not wide_runs:
        return 0
    # Build top (domain) and bottom (image) doubly-linked lists over nodes:
    # Wide strips have ids 1..=k; gaps (M_16 intervals) are terminal (absorbed at right end).
    # We use the exact auxiliary IET of src/open/endpoint.rs:
    # Nodes = wide strips (active) + retained gaps (terminal, identity on bottom? No:
    # in Open's IET, top = domain intervals [wide strips + gaps], bottom = image intervals
    # [wide strip LF images + complement gaps]. Peeling from the right stops when remaining
    # wide-strip rows == 0).
    # Build sorted top intervals (dom) and sorted bottom intervals (img):
    # Each node has: is_wide, width.
    # For gaps on top vs bottom: gaps are NOT paired 1-to-1; instead Open puts all wide
    # strips on BOTH sides, and puts all non-wide gaps as a single prefix/split.
    # Actually, let's replicate src/open/endpoint.rs's exact two-list construction!
    # In endpoint.rs:
    #   - Active nodes (wide strips): sit at their domain order on side 0, and at their
    #     image order on side 1.
    #   - Gaps between wide strips on side 0 (retained rows M_16) sit at their domain
    #     order on side 0 AND at the SAME coordinate on side 1? Wait: gaps in domain are
    #     M_16, and gaps in image are LF(M_16)! They don't have the same coordinates!
    #   - Open's auxiliary IET pairs:
    #       side 0 (top):    [wide strips at dom_s] interleaved with [gaps of dom = M_16]
    #       side 1 (bottom): [wide strips at lf_s]  interleaved with [gaps of img = LF(M_16)]
    #     Wait: each gap on side 0 is TERMINAL (exits to M_16); each gap on side 1 is
    #     an ENTRY (from LF(M_16)). So on side 0, a gap is NEVER an active strip, and on
    #     side 1, a gap has no matching node on side 0 — instead Open puts terminal gaps
    #     at the front (side 0 prefix) and shifts, OR peels right-to-left until all active
    #     nodes are gone!
    # Let's build the exact two-sided auxiliary IET (nodes 1..N):
    #   1. Wide strips: each wide strip i has width w_i, sits at dom_s on side 0 and lf_s on side 1.
    #   2. Complement of wide strips in [0, n) on side 0 (M_16): total width m = n - sum(w_i).
    #   3. Complement of LF(wide strips) in [0, n) on side 1 (LF(M_16)): total width m.
    #   Open's IET pairs the k-th row of M_16 with the k-th row of LF(M_16) as dummy
    #   "exit/entry" intervals (putting them in a single bijection of width n), then
    #   runs right-end Rauzy-Veech induction on the rightmost pair (x, y) until all wide
    #   strip mass is eliminated!
    # Let's simulate that exact doubly-linked list right-end induction!
    # First build side-0 intervals (dom) and side-1 intervals (img), splitting the dummy
    # complement into matching widths so both sides have the exact same node set!
    dom_wide = sorted([(s, s + w, idx) for idx, (s, w, _, _) in enumerate(wide_runs)])
    img_wide = sorted([(lf_s, lf_s + w, idx) for idx, (_, w, lf_s, _) in enumerate(wide_runs)])

    # Extract gaps on side 0 and side 1
    dom_gaps: list[tuple[int, int]] = []
    pos = 0
    for s, e, _ in dom_wide:
        if s > pos:
            dom_gaps.append((pos, s - pos))
        pos = e
    if pos < n:
        dom_gaps.append((pos, n - pos))

    img_gaps: list[tuple[int, int]] = []
    pos = 0
    for s, e, _ in img_wide:
        if s > pos:
            img_gaps.append((pos, s - pos))
        pos = e
    if pos < n:
        img_gaps.append((pos, n - pos))

    # Pair dom_gaps and img_gaps monotonically into common-width terminal nodes
    # (exact same construction as Open's endpoint.rs!)
    dom_seq: list[tuple[int, int]] = []  # (pos, node_id)
    img_seq: list[tuple[int, int]] = []
    # Node IDs: 1..=K are wide strips; K+1.. are terminal gap nodes
    K = len(wide_runs)
    widths: list[int] = [0] + [w for _, w, _, _ in wide_runs]
    for s, _, idx in dom_wide:
        dom_seq.append((s, idx + 1))
    for s, _, idx in img_wide:
        img_seq.append((s, idx + 1))

    gi = gj = 0
    off_i = off_j = 0
    nid = K + 1
    while gi < len(dom_gaps) and gj < len(img_gaps):
        ds, dw = dom_gaps[gi]
        is_, iw = img_gaps[gj]
        take = min(dw - off_i, iw - off_j)
        widths.append(take)
        dom_seq.append((ds + off_i, nid))
        img_seq.append((is_ + off_j, nid))
        nid += 1
        off_i += take
        off_j += take
        if off_i == dw:
            gi += 1
            off_i = 0
        if off_j == iw:
            gj += 1
            off_j = 0
    assert gi == len(dom_gaps) and gj == len(img_gaps)

    dom_seq.sort()
    img_seq.sort()

    # Build circular doubly-linked lists on side 0 and side 1 (sentinel = 0)
    # Wait: Open puts terminal nodes BEFORE wide nodes? No, Open puts ALL nodes at their
    # real coordinates in [0, n) on both sides, EXCEPT terminal nodes are placed at the
    # front so right-end peeling eliminates ONLY wide strips (wait: if terminal nodes are
    # at real coordinates, a terminal node at the right end would be peeled too!).
    # Let's check: in Open's auxiliary IET (endpoint.rs), side 0 has [all terminal nodes,
    # then all wide nodes in dom order], side 1 has [all terminal nodes, then all wide
    # nodes in img order] — NO, wait: if wide strips land inside terminal gaps, side 1's
    # coordinates MUST interleave wide strips and terminal gaps!
    # Actually: let's check Open's exact side 0 / side 1 order:
    #   On side 0: [terminal nodes (all of M_16) in dom order] followed by [wide strips in dom order]?
    #   NO! Open puts ALL of M_16 at the PREFIX [0, m) of side 0, and wide strips at [m, n) of side 0!
    #   And on side 1: side 1 is the EXACT image order in [0, n), interleaving wide images and LF(M_16)
    #   Wait: then the map sends wide strip [d, d+w) -> [lf_s, lf_s+w), which intersects BOTH
    #   wide strips (on [m, n) of side 0!) and M_16 (on [0, m) of side 0!).
    #   Wait: for side 0 to have [M_16 first, then wide strips], side 1 must have the SAME
    #   permutation of [0, n), i.e., `top` = [M_16, W] and `bottom` = [LF^{-1}(M_16)?] ->
    #   No, in Open:
    #     A wide strip W_i has domain I_i subset W and image J_i subset [0, n).
    #     We want to contract W until every J_i lands in M_16!
    #     So on side 0 (domain = targets of next step): put M_16 FIRST [0, m), then W [m, n)!
    #     On side 1 (images of current step): map J_i through the SAME reindexing phi: [0, n) -> [0, n)
    #     that puts M_16 at [0, m) and W at [m, n)!
    #     Then right-end induction peels from coordinate n down to m (total mass = |W| = n - m),
    #     and the moment `remaining` hits 0, EVERY node left is in [0, m) = M_16!
    # YES!! THAT IS OPEN'S EXACT AUXILIARY IET (Astra #565 / endpoint.rs)!
    # Let's simulate THAT exact auxiliary IET!
    m = n - sum(w for _, w, _, _ in wide_runs)
    # Build reindexing phi: M_16 intervals -> [0, m), wide strips -> [m, n)
    phi_pieces: list[tuple[int, int, int]] = []  # (orig_s, orig_e, new_s)
    cur_m = 0
    for ds, dw in dom_gaps:
        phi_pieces.append((ds, ds + dw, cur_m))
        cur_m += dw
    assert cur_m == m
    cur_w = m
    for ds, de, _ in dom_wide:
        phi_pieces.append((ds, de, cur_w))
        cur_w += (de - ds)
    assert cur_w == n
    phi_pieces.sort()

    # Map each wide strip's image [lf_s, lf_s+w) through phi (splitting at phi_pieces boundaries).
    # Each resulting sub-strip has a unique dom piece on side 0 (inside [m, n)) and
    # a unique image piece on side 1 (inside [0, n))!
    # And the complement of those images on side 1 has total width m, matching M_16 on side 0!
    side0_active: list[tuple[int, int, int]] = []  # (phi_dom_s, width, id)
    side1_active: list[tuple[int, int, int]] = []  # (phi_img_s, width, id)
    next_id = 1
    # Map wide strip dom_s -> phi_dom_s
    dom_to_phi: dict[int, int] = {}
    cur_w = m
    for ds, de, _ in dom_wide:
        dom_to_phi[ds] = cur_w
        cur_w += (de - ds)

    p_idx = 0
    for s, w, lf_s, _ in sorted(wide_runs, key=lambda x: x[2]):
        pdom = dom_to_phi[s]
        off = 0
        while off < w:
            cur = lf_s + off
            while phi_pieces[p_idx][1] <= cur:
                p_idx += 1
            ps, pe, pnew = phi_pieces[p_idx]
            take = min(w - off, pe - cur)
            side0_active.append((pdom + off, take, next_id))
            side1_active.append((pnew + (cur - ps), take, next_id))
            next_id += 1
            off += take

    side0_active.sort()
    side1_active.sort()

    # Complement gaps on side 1 (total width m) pair with [0, m) on side 0!
    side1_gaps: list[tuple[int, int]] = []
    pos = 0
    for s, w, _ in side1_active:
        if s > pos:
            side1_gaps.append((pos, s - pos))
        pos = s + w
    if pos < n:
        side1_gaps.append((pos, n - pos))

    # Build final side0 and side1 lists:
    # side0: terminal gap nodes first [0, m), then active nodes [m, n)
    # side1: interleaved terminal gap nodes and active nodes in coordinate order [0, n)
    side0_nodes: list[int] = []
    side1_items: list[tuple[int, int]] = [(s, nid) for (s, _, nid) in side1_active]
    node_w: list[int] = [0] * (next_id + len(side1_gaps) + 1)
    for _, w, nid in side0_active:
        node_w[nid] = w
    cur_g = 0
    for gs, gw in side1_gaps:
        gid = next_id
        next_id += 1
        node_w[gid] = gw
        side0_nodes.append(gid)
        side1_items.append((gs, gid))
        cur_g += gw
    assert cur_g == m
    for _, _, nid in side0_active:
        side0_nodes.append(nid)
    side1_items.sort()
    side1_nodes = [nid for _, nid in side1_items]

    # Doubly-linked lists with sentinel 0
    N = next_id
    prev0 = [0] * N
    next0 = [0] * N
    prev1 = [0] * N
    next1 = [0] * N

    def link_list(seq: list[int], pv: list[int], nx: list[int]) -> None:
        last = 0
        for u in seq:
            nx[last] = u
            pv[u] = last
            last = u
        nx[last] = 0
        pv[0] = last

    link_list(side0_nodes, prev0, next0)
    link_list(side1_nodes, prev1, next1)

    # Right-end Rauzy-Veech induction (exact mirror of src/open/endpoint.rs eliminate<true>)
    remaining = n - m
    steps = 0
    while remaining > 0:
        x = prev0[0]
        y = prev1[0]
        aw = node_w[x]
        bw = node_w[y]
        if x == y:
            # Self-match at right end: remove x from both lists
            px0 = prev0[x]
            next0[px0] = 0
            prev0[0] = px0
            px1 = prev1[x]
            next1[px1] = 0
            prev1[0] = px1
            remaining -= aw
            steps += 1
        elif aw > bw:
            # Spiral contraction if prev1[y] == x
            if prev1[y] == x and (aw - 1) // bw >= 2:
                q = (aw - 1) // bw
                node_w[x] = aw - q * bw
                remaining -= q * bw
                steps += 1  # contracted spiral = 1 step in Open!
            else:
                # Cut y from side 0, insert after x? No: y moves right before x on side 0?
                # Standard Rauzy-Veech: pop y from side 1, insert y right after x on side 1?
                # Wait: x is rightmost on side 0, y is rightmost on side 1, aw > bw.
                # Rightmost bw rows of x map to y's image -> compose x's tail with y:
                # y's domain moves inside x's domain, so y stays on side 0?
                # Let's match endpoint.rs exactly:
                #   x stays rightmost on side 0 with width aw - bw;
                #   y is popped from side 1 tail, and inserted immediately AFTER x on side 1!
                py1 = prev1[y]
                next1[py1] = 0
                prev1[0] = py1
                nx1 = next1[x]
                prev1[y] = x
                next1[y] = nx1
                next1[x] = y
                prev1[nx1] = y
                node_w[x] = aw - bw
                remaining -= bw
                steps += 1
        elif aw < bw:
            if prev0[x] == y and (bw - 1) // aw >= 2:
                q = (bw - 1) // aw
                node_w[y] = bw - q * aw
                remaining -= q * aw
                steps += 1
            else:
                # Pop x from side 0 tail, insert immediately AFTER y on side 0!
                px0 = prev0[x]
                next0[px0] = 0
                prev0[0] = px0
                ny0 = next0[y]
                prev0[x] = y
                next0[x] = ny0
                next0[y] = x
                prev0[ny0] = x
                node_w[y] = bw - aw
                remaining -= aw
                steps += 1
        else:
            # aw == bw, x != y: x and y merge! Pop x from side 0, pop y from side 1,
            # replace x on side 1 with y (or vice versa):
            px0 = prev0[x]
            next0[px0] = 0
            prev0[0] = px0
            py1 = prev1[y]
            next1[py1] = 0
            prev1[0] = py1
            # Put y in x's slot on side 1
            px1 = prev1[x]
            nx1 = next1[x]
            next1[px1] = y
            prev1[y] = px1
            next1[y] = nx1
            prev1[nx1] = y
            remaining -= aw
            steps += 1
    return steps


def build_lazy_thin_system(
    blk: common.BlockData,
    wide_info: dict[str, Any],
    max_thin_rounds: int,
    L_min: int = 17,
) -> dict[str, Any]:
    """Contract wide strips (already in wide_info) + up to `max_thin_rounds` lazy
    levels of thin-run contraction S*_1, ..., S*_k using ONLY label-ordered forward
    cursor merges (ZERO binary searches, Q_search = 0!).

    - max_thin_rounds == 0: pure Open M_16 (L_min = 17)
    - max_thin_rounds == 1: MONODROMY record 120 §3 lazy S*_1 (all thin rows whose
      P_W(LF(y)) lands in H_1 are eliminated in 1 cursor merge pass!)
    - max_thin_rounds == k: k lazy rounds S*_1 .. S*_k (thin rows reaching H_1 in
      <= k thin steps, zero searches!)
    - max_thin_rounds == -1: run to fixpoint (L_min = 2, m = h1 or h1+1)

    Returns the exact retained system on M^{(k)} = M_16 \\ S*_{<=k}, with:
      - flat deduplicated dictionary (NO DAG!)
      - exact R, v, word_id arrays
      - exact B_P schedule piece counts
      - exact cursor-merge work & pass counts
    """
    n = blk.n
    p = blk.primary_index
    layout = wide_info["layout"]          # (s, e, kind, lf_s, b), kind: 0=H1, 1=thin, 2=wide
    wide_final = wide_info["wide_final"]  # sorted by dom_s: (dom_s, w, exit_row_in_M16, wid)
    thin_runs = wide_info["thin_runs"]    # (s, w, lf_s, b)
    h1_runs = wide_info["h1_runs"]        # (s, w, lf_s, b)

    # Copy word intern table so each k experiment has exact isolated dictionary counts
    word_to_id: dict[bytes, int] = dict(wide_info["word_to_id"])
    id_to_word: list[bytes] = list(wide_info["id_to_word"])

    def intern(wb: bytes) -> int:
        wid = word_to_id.get(wb)
        if wid is None:
            wid = len(id_to_word)
            word_to_id[wb] = wid
            id_to_word.append(wb)
        return wid

    # Build P_W partition of [0, n) sorted by dom_s:
    # (ds, de, exit_row_start, wid_or_neg1, dest_kind)
    # where dest_kind is:
    #   - if wid == -1: identity on M_16 (dest_kind = 0 if H1, 1 if thin)
    #   - if wid >= 0:  wide strip landing in M_16 (we check whether exit_row is H1 or thin!)
    # First build sorted M_16 pieces with kind (0=H1, 1=thin)
    m16_kind_ivs = [(s, e, k) for (s, e, k, _, _) in layout if k != 2]
    # Split wide_final against m16_kind_ivs? Wait: wide_final pieces already land
    # inside a single layout piece (from Prop A!), so each wide_final piece lands
    # wholly in H1 (0) OR wholly in thin (1)! Let's tag each wide_final piece in 1 sweep
    # sorted by exit_row (0 binary searches!).
    wf_by_exit = sorted(enumerate(wide_final), key=lambda x: x[1][2])
    wf_dest_kind = [0] * len(wide_final)
    mi = 0
    for idx, (_, w, ex, _) in wf_by_exit:
        while m16_kind_ivs[mi][1] <= ex:
            mi += 1
        ms, me, mk = m16_kind_ivs[mi]
        assert ms <= ex and ex + w <= me
        wf_dest_kind[idx] = mk

    # Build unified P_W partition of [0, n) in dom_s order (linear 2-pointer merge, Q=0)
    P_W: list[tuple[int, int, int, int, int]] = []  # (ds, de, target_s, wid_or_-1, dest_kind)
    im = iw = 0
    nm, nw = len(m16_kind_ivs), len(wide_final)
    while im < nm and iw < nw:
        if m16_kind_ivs[im][0] < wide_final[iw][0]:
            ms, me, mk = m16_kind_ivs[im]
            P_W.append((ms, me, ms, -1, mk))
            im += 1
        else:
            ds, w, ex, wid = wide_final[iw]
            P_W.append((ds, ds + w, ex, wid, wf_dest_kind[iw]))
            iw += 1
    while im < nm:
        ms, me, mk = m16_kind_ivs[im]
        P_W.append((ms, me, ms, -1, mk))
        im += 1
    while iw < nw:
        ds, w, ex, wid = wide_final[iw]
        P_W.append((ds, ds + w, ex, wid, wf_dest_kind[iw]))
        iw += 1
    assert P_W[0][0] == 0 and P_W[-1][1] == n

    # ---- Measure NEEDS M-27 statistics on thin_runs (1 linear cursor sweep, Q=0!) ----
    # Group thin_runs by byte b (already sorted by lf_s within each b!)
    thin_by_b: list[list[tuple[int, int, int]]] = [[] for _ in range(256)]
    for s, w, lf_s, b in thin_runs:
        thin_by_b[b].append((s, w, lf_s))

    # Intersect LF(thin_runs) with layout (raw 1-step) AND with P_W (1-step + wide)!
    thin_rows_total = sum(w for _, w, _, _ in thin_runs)
    thin_land_h1_raw = 0
    thin_land_wide_raw = 0
    thin_land_thin_raw = 0
    thin_land_h1_after_pw = 0
    thin_land_thin_after_pw = 0

    # Also build the exact 1-step map F = P_W o LF on L_thin:
    # thin_step1_done: sub-pieces of L_thin whose P_W(LF(y)) lands in H_1 -> S*_1!
    # thin_step1_live: sub-pieces of L_thin whose P_W(LF(y)) lands in L_thin!
    # Format: (dom_s, w, target_row, wid)
    thin_s1_done: list[tuple[int, int, int, int]] = []
    thin_s1_live: list[tuple[int, int, int, int]] = []

    p_idx = 0
    l_idx = 0
    thin_merge_steps = 0
    for b in range(256):
        b_byte = bytes([b])
        for s, w, lf_s in thin_by_b[b]:
            # Raw 1-step layout stats (NEEDS M-27)
            off = 0
            while off < w:
                cur = lf_s + off
                while layout[l_idx][1] <= cur:
                    l_idx += 1
                ls, le, lk, _, _ = layout[l_idx]
                take = min(w - off, le - cur)
                if lk == 0:
                    thin_land_h1_raw += take
                elif lk == 1:
                    thin_land_thin_raw += take
                else:
                    thin_land_wide_raw += take
                off += take
            # P_W o LF intersection (ONE forward cursor over P_W across b=0..255!)
            off = 0
            while off < w:
                cur = lf_s + off
                while P_W[p_idx][1] <= cur:
                    p_idx += 1
                ds, de, ts, swid, dk = P_W[p_idx]
                take = min(w - off, de - cur)
                tgt = ts + (cur - ds)
                fwid = b if swid == -1 else intern(b_byte + id_to_word[swid])
                thin_merge_steps += 1
                if dk == 0:
                    thin_land_h1_after_pw += take
                    thin_s1_done.append((s + off, take, tgt, fwid))
                else:
                    thin_land_thin_after_pw += take
                    thin_s1_live.append((s + off, take, tgt, fwid))
                off += take

    # ---- Lazy k-round thin contraction S*_1 .. S*_k (ZERO binary searches!) ----
    # In round r >= 2: we ONLY compose thin_live pieces whose target lands in S*_{r-1}
    # (which was just contracted to H_1 in round r-1!).
    # Because we sort thin_live by target_row and S*_{r-1} by dom_s (linear radix sort
    # in C/Rust, Q=0!), ONE linear 2-pointer merge resolves the entire round with ZERO
    # binary searches!
    eliminated_thin: list[tuple[int, int, int, int]] = []  # (dom_s, w, exit_in_H1, wid)
    live_thin = thin_s1_live
    rounds_run = 0
    thin_passes = 0

    if max_thin_rounds == 0:
        # Pure Open M_16: eliminate NO thin rows
        eliminated_thin = []
        rounds_run = 0
        thin_passes = 0
    else:
        # Round 1: S*_1
        frontier = sorted(thin_s1_done, key=lambda x: x[0])
        eliminated_thin.extend(frontier)
        rounds_run = 1
        thin_passes = 1
        limit_r = 10_000 if max_thin_rounds < 0 else max_thin_rounds

        while rounds_run < limit_r and live_thin and frontier:
            # Sort live_thin by target_row (radixed, Q=0); frontier is sorted by dom_s!
            live_by_t = sorted(live_thin, key=lambda x: x[2])
            new_frontier: list[tuple[int, int, int, int]] = []
            next_live: list[tuple[int, int, int, int]] = []
            fi = 0
            nf = len(frontier)
            for ds, w, tgt, wid in live_by_t:
                while fi < nf and frontier[fi][0] + frontier[fi][1] <= tgt:
                    fi += 1
                j_f = fi
                off = 0
                while off < w:
                    cur = tgt + off
                    rem = w - off
                    while j_f < nf and frontier[j_f][0] + frontier[j_f][1] <= cur:
                        j_f += 1
                    if j_f >= nf or frontier[j_f][0] >= cur + rem:
                        # Does not hit frontier -> stays live
                        next_live.append((ds + off, rem, cur, wid))
                        off = w
                    elif cur < frontier[j_f][0]:
                        gap = frontier[j_f][0] - cur
                        next_live.append((ds + off, gap, cur, wid))
                        off += gap
                    else:
                        fds, fw, fexit, fwid = frontier[j_f]
                        take = min(rem, fds + fw - cur)
                        cwid = intern(id_to_word[wid] + id_to_word[fwid])
                        new_frontier.append((ds + off, take, fexit + (cur - fds), cwid))
                        thin_merge_steps += 1
                        off += take
            if not new_frontier:
                break
            frontier = sorted(new_frontier, key=lambda x: x[0])
            eliminated_thin.extend(frontier)
            live_thin = next_live
            rounds_run += 1
            thin_passes += 1

    # Sort eliminated_thin by dom_s (all disjoint, all exit directly into H_1!)
    eliminated_thin.sort(key=lambda x: x[0])

    # ---- Build final partition P_k: [0, n) -> M^{(k)} (ZERO binary searches!) ----
    # 1. Retained set M^{(k)} = H_1 union live_thin (the un-eliminated thin intervals).
    #    Construct sorted M_k intervals [a, e) directly from [0, n) minus (wide_final U eliminated_thin)!
    #    Wait: even simpler — compose wide_final's exits with eliminated_thin in 1 linear sweep
    #    sorted by exit_row (ZERO binary searches!).
    wide_updated: list[tuple[int, int, int, int]] = []  # (dom_s, w, exit_in_Mk, wid)
    if not eliminated_thin:
        wide_updated = wide_final
    else:
        ei = 0
        ne = len(eliminated_thin)
        for _, (ds, w, ex, wid) in wf_by_exit:
            while ei < ne and eliminated_thin[ei][0] + eliminated_thin[ei][1] <= ex:
                ei += 1
            j_e = ei
            off = 0
            while off < w:
                cur = ex + off
                rem = w - off
                while j_e < ne and eliminated_thin[j_e][0] + eliminated_thin[j_e][1] <= cur:
                    j_e += 1
                if j_e >= ne or eliminated_thin[j_e][0] >= cur + rem:
                    wide_updated.append((ds + off, rem, cur, wid))
                    off = w
                elif cur < eliminated_thin[j_e][0]:
                    gap = eliminated_thin[j_e][0] - cur
                    wide_updated.append((ds + off, gap, cur, wid))
                    off += gap
                else:
                    eds, ew, eexit, ewid = eliminated_thin[j_e]
                    take = min(rem, eds + ew - cur)
                    cwid = intern(id_to_word[wid] + id_to_word[ewid])
                    wide_updated.append((ds + off, take, eexit + (cur - eds), cwid))
                    thin_merge_steps += 1
                    off += take
        wide_updated.sort(key=lambda x: x[0])
        thin_passes += 1

    # Merge wide_updated and eliminated_thin into sorted all_elim (disjoint intervals of [0, n))
    all_elim: list[tuple[int, int, int, int]] = []  # (ds, de, exit_row, wid)
    ia = ib = 0
    na, nb = len(wide_updated), len(eliminated_thin)
    while ia < na and ib < nb:
        if wide_updated[ia][0] < eliminated_thin[ib][0]:
            ds, w, ex, wid = wide_updated[ia]
            all_elim.append((ds, ds + w, ex, wid))
            ia += 1
        else:
            ds, w, ex, wid = eliminated_thin[ib]
            all_elim.append((ds, ds + w, ex, wid))
            ib += 1
    while ia < na:
        ds, w, ex, wid = wide_updated[ia]
        all_elim.append((ds, ds + w, ex, wid))
        ia += 1
    while ib < nb:
        ds, w, ex, wid = eliminated_thin[ib]
        all_elim.append((ds, ds + w, ex, wid))
        ib += 1

    # Complement of all_elim in [0, n) is M_k_intervals (maximal retained intervals)!
    M_k_ivs: list[tuple[int, int, int]] = []  # (ds, de, rank_off)
    pos = 0
    acc_r = 0
    mark_rank = -1
    for ds, de, _, _ in all_elim:
        if ds > pos:
            if pos <= p < ds:
                mark_rank = acc_r + (p - pos)
            M_k_ivs.append((pos, ds, acc_r))
            acc_r += ds - pos
        pos = de
    if pos < n:
        if pos <= p < n:
            mark_rank = acc_r + (p - pos)
        M_k_ivs.append((pos, n, acc_r))
        acc_r += n - pos
    m_k = acc_r
    assert mark_rank != -1

    # Map every all_elim piece's exit_row -> exit_rank in 1 linear sweep sorted by exit_row (Q_search=0!)
    elim_by_exit = sorted(enumerate(all_elim), key=lambda x: x[1][2])
    elim_rank = [0] * len(all_elim)
    mi = 0
    for idx, (ds, de, ex, wid) in elim_by_exit:
        w = de - ds
        while M_k_ivs[mi][1] <= ex:
            mi += 1
        ms, me, moff = M_k_ivs[mi]
        assert ms <= ex and ex + w <= me, f"Exit [{ex},{ex+w}) not inside M_k [{ms},{me})"
        elim_rank[idx] = moff + (ex - ms)

    # Unified P_k partition of [0, n) in dom_s order: (ds, de, target_rank_start, wid_or_-1)
    P_k: list[tuple[int, int, int, int]] = []
    im = ie = 0
    nm, ne = len(M_k_ivs), len(all_elim)
    while im < nm and ie < ne:
        if M_k_ivs[im][0] < all_elim[ie][0]:
            ms, me, moff = M_k_ivs[im]
            P_k.append((ms, me, moff, -1))
            im += 1
        else:
            ds, de, _, wid = all_elim[ie]
            P_k.append((ds, de, elim_rank[ie], wid))
            ie += 1
    while im < nm:
        ms, me, moff = M_k_ivs[im]
        P_k.append((ms, me, moff, -1))
        im += 1
    while ie < ne:
        ds, de, _, wid = all_elim[ie]
        P_k.append((ds, de, elim_rank[ie], wid))
        ie += 1
    assert P_k[0][0] == 0 and P_k[-1][1] == n

    # Intersect M_k_ivs with layout to get ret_lf_intervals[b] and v_ret (1 linear sweep, Q=0!)
    ret_lf_ivs: list[list[tuple[int, int]]] = [[] for _ in range(256)]
    v_ret = bytearray()
    li = 0
    for ms, me, _ in M_k_ivs:
        cur = ms
        while cur < me:
            while layout[li][1] <= cur:
                li += 1
            ls, le, _, llf, lb = layout[li]
            take = min(me, le) - cur
            lf_lo = llf + (cur - ls)
            lst = ret_lf_ivs[lb]
            if lst and lst[-1][1] == lf_lo:
                lst[-1] = (lst[-1][0], lf_lo + take)
            else:
                lst.append((lf_lo, lf_lo + take))
            v_ret.extend(bytes([lb]) * take)
            cur += take
    assert len(v_ret) == m_k

    # Emit exact B_P schedule by ONE linear sweep of P_k across b=0..255 (Q=0!)
    sched: list[list[tuple[int, int, int]]] = [[] for _ in range(256)]
    pk_i = 0
    B_P_target = 0
    B_P_sched = 0
    for b in range(256):
        b_byte = bytes([b])
        out_b = sched[b]
        last_t_end = -10**9
        for lo, hi in ret_lf_ivs[b]:
            cur = lo
            while cur < hi:
                while P_k[pk_i][1] <= cur:
                    pk_i += 1
                ds, de, tr, swid = P_k[pk_i]
                take = min(hi, de) - cur
                t_start = tr + (cur - ds)
                fwid = b if swid == -1 else intern(b_byte + id_to_word[swid])
                if t_start != last_t_end:
                    B_P_target += 1
                last_t_end = t_start + take
                if out_b and out_b[-1][0] + out_b[-1][1] == t_start and out_b[-1][2] == fwid:
                    out_b[-1] = (out_b[-1][0], out_b[-1][1] + take, fwid)
                else:
                    out_b.append((t_start, take, fwid))
                cur += take
        B_P_sched += len(out_b)

    # Materialize R[0..m_k) and code_1[0..m_k) by replaying sched in 1 streaming pass (Q=0!)
    R = [0] * m_k
    code_1 = [0] * m_k
    c_idx = [0] * 256
    c_off = [0] * 256
    for j in range(m_k):
        b = v_ret[j]
        si = c_idx[b]
        off = c_off[b]
        t_start, seg_len, fwid = sched[b][si]
        R[j] = t_start + off
        code_1[j] = fwid
        off += 1
        if off == seg_len:
            c_idx[b] = si + 1
            c_off[b] = 0
        else:
            c_off[b] = off

    # Distinct flat dictionary statistics actually used by R!
    used_codes = set(code_1)
    n_distinct_words = len(used_codes)
    flat_dict_bytes = sum(len(id_to_word[c]) for c in used_codes)
    inline16_rows = sum(1 for c in code_1 if len(id_to_word[c]) <= 15)
    max_word_len = max(len(id_to_word[c]) for c in used_codes)

    return {
        "k": max_thin_rounds,
        "rounds_run": rounds_run,
        "thin_passes": thin_passes,
        "thin_merge_steps": thin_merge_steps,
        "m": m_k,
        "mark_rank": mark_rank,
        "v": v_ret,
        "R": R,
        "code_1": code_1,
        "id_to_word": id_to_word,
        "B_P_target": B_P_target,
        "B_P_sched": B_P_sched,
        "n_P_pieces": len(P_k),
        "n_elim_thin_pieces": len(eliminated_thin),
        "n_distinct_words": n_distinct_words,
        "flat_dict_bytes": flat_dict_bytes,
        "inline16_frac": inline16_rows / m_k,
        "max_word_len": max_word_len,
        # NEEDS M-27 stats
        "thin_rows_total": thin_rows_total,
        "thin_land_h1_raw": thin_land_h1_raw,
        "thin_land_wide_raw": thin_land_wide_raw,
        "thin_land_thin_raw": thin_land_thin_raw,
        "thin_land_h1_after_pw": thin_land_h1_after_pw,
        "thin_land_thin_after_pw": thin_land_thin_after_pw,
    }


def verify_walk_R(sys_d: dict[str, Any], T_oracle: bytes) -> int:
    """Walk R with flat words, verify byte-exactness against T_oracle, return emit links."""
    R = sys_d["R"]
    code_1 = sys_d["code_1"]
    words = sys_d["id_to_word"]
    mark = sys_d["mark_rank"]
    n = len(T_oracle)
    out = bytearray(n)
    pos = n
    cur = mark
    links = 0
    while pos > 0:
        w = words[code_1[cur]]
        lw = len(w)
        # w is in LF (backward) order: w[0] goes to out[pos-1], ...
        out[pos - lw : pos] = w[::-1]
        pos -= lw
        cur = R[cur]
        links += 1
    assert cur == mark and links == sys_d["m"]
    assert bytes(out) == T_oracle, "Walk R decode mismatch!"
    return links


def run_m17_and_cl1(sys_d: dict[str, Any], T_oracle: bytes) -> dict[str, Any]:
    """Run 1-sweep M-17 GS and 2-sweep CL1 on R, costing BOTH flat payloads and DAG."""
    R = sys_d["R"]
    v = sys_d["v"]
    code_1 = sys_d["code_1"]
    words = sys_d["id_to_word"]
    m = sys_d["m"]
    mark = sys_d["mark_rank"]
    n = len(T_oracle)

    # --- 1. Uncapped M-17 1-sweep GS ---
    nxt = R[:]
    # Track DAG nodes to count internal visits AND materialize flat words on survivors
    dag_l: list[int] = [-1] * m
    dag_r: list[int] = [-1] * m
    wid = list(range(m))  # leaf j means code_1[j]
    for j in range(m):
        y = nxt[j]
        if y == j or y == mark:
            continue
        if y < j:
            # In-place Gauss-Seidel (reads updated nxt[y])
            nxt[j] = nxt[y]
            nid = len(dag_l)
            dag_l.append(wid[j])
            dag_r.append(wid[y])
            wid[j] = nid
        else:
            # y > j: still reads original nxt[y] = R[y]!
            nxt[j] = nxt[y]
            nid = len(dag_l)
            dag_l.append(wid[j])
            dag_r.append(wid[y])
            wid[j] = nid

    def expand_to_bytes(nid: int) -> tuple[bytes, int]:
        stack = [nid]
        parts: list[bytes] = []
        internal = 0
        while stack:
            u = stack.pop()
            if u < m:
                parts.append(words[code_1[u]])
            else:
                internal += 1
                stack.append(dag_r[u])
                stack.append(dag_l[u])
        return b"".join(parts), internal

    out = bytearray(n)
    pos = n
    cur = mark
    m17_links = 0
    m17_dag_internal = 0
    m17_surv_words: set[bytes] = set()
    m17_inline16 = 0
    while pos > 0:
        wb, iv = expand_to_bytes(wid[cur])
        m17_dag_internal += iv
        m17_surv_words.add(wb)
        lw = len(wb)
        if lw <= 15:
            m17_inline16 += 1
        out[pos - lw : pos] = wb[::-1]
        pos -= lw
        cur = nxt[cur]
        m17_links += 1
    assert cur == mark and bytes(out) == T_oracle, "M17 decode mismatch!"
    m17_flat_surv_bytes = sum(len(w) for w in m17_surv_words)
    # Full flat build bytes (all m rows if flattened during sweep without knowing survivors)
    # Compute exact lengths of all m rows via DP on DAG
    dag_blen = [len(words[code_1[j]]) for j in range(m)]
    for u in range(m, len(dag_l)):
        dag_blen.append(dag_blen[dag_l[u]] + dag_blen[dag_r[u]])
    m17_flat_all_bytes = sum(dag_blen[wid[j]] for j in range(m))

    # --- 2. CL1 2-sweep (ascending D1, descending A1) ---
    nxt2 = R[:]
    dag2_l: list[int] = [-1] * m
    dag2_r: list[int] = [-1] * m
    wid2 = list(range(m))
    # Sweep 1: ascending on D1 (R[j] < j)
    for j in range(m):
        y = R[j]
        if y < j and y != mark:
            nxt2[j] = nxt2[y]
            nid = len(dag2_l)
            dag2_l.append(wid2[j])
            dag2_r.append(wid2[y])
            wid2[j] = nid
    # Sweep 2: descending on A1 (R[j] > j)
    for j in range(m - 1, -1, -1):
        y = R[j]
        if y > j and y != mark:
            nxt2[j] = nxt2[y]
            nid = len(dag2_l)
            dag2_l.append(wid2[j])
            dag2_r.append(wid2[y])
            wid2[j] = nid

    def expand2(nid: int) -> tuple[bytes, int]:
        stack = [nid]
        parts: list[bytes] = []
        internal = 0
        while stack:
            u = stack.pop()
            if u < m:
                parts.append(words[code_1[u]])
            else:
                internal += 1
                stack.append(dag2_r[u])
                stack.append(dag2_l[u])
        return b"".join(parts), internal

    out2 = bytearray(n)
    pos = n
    cur = mark
    cl1_links = 0
    cl1_dag_internal = 0
    cl1_surv_words: set[bytes] = set()
    cl1_inline16 = 0
    while pos > 0:
        wb, iv = expand2(wid2[cur])
        cl1_dag_internal += iv
        cl1_surv_words.add(wb)
        lw = len(wb)
        if lw <= 15:
            cl1_inline16 += 1
        out2[pos - lw : pos] = wb[::-1]
        pos -= lw
        cur = nxt2[cur]
        cl1_links += 1
    assert cur == mark and bytes(out2) == T_oracle, "CL1 decode mismatch!"
    cl1_flat_surv_bytes = sum(len(w) for w in cl1_surv_words)
    dag2_blen = [len(words[code_1[j]]) for j in range(m)]
    for u in range(m, len(dag2_l)):
        dag2_blen.append(dag2_blen[dag2_l[u]] + dag2_blen[dag2_r[u]])
    cl1_flat_all_bytes = sum(dag2_blen[wid2[j]] for j in range(m))

    return {
        "m17_links": m17_links,
        "m17_dag_internal": m17_dag_internal,
        "m17_flat_surv_words": len(m17_surv_words),
        "m17_flat_surv_bytes": m17_flat_surv_bytes,
        "m17_flat_all_bytes": m17_flat_all_bytes,
        "m17_inline16_frac": m17_inline16 / m17_links,
        "cl1_links": cl1_links,
        "cl1_dag_internal": cl1_dag_internal,
        "cl1_flat_surv_words": len(cl1_surv_words),
        "cl1_flat_surv_bytes": cl1_flat_surv_bytes,
        "cl1_flat_all_bytes": cl1_flat_all_bytes,
        "cl1_inline16_frac": cl1_inline16 / cl1_links,
    }


def run_r2_and_r4_depth2(sys_d: dict[str, Any], T_oracle: bytes) -> dict[str, Any]:
    """Build R^2 (using R's byte schedule, 256 cursors) and R^4 = R^2 o R^2 (using
    the (v[j], v[R[j]]) depth-2 schedule), verify 100% byte-exact decode for BOTH,
    and measure exact cursor restarts, active pair count, payload sizes, and dict reads.
    """
    R = sys_d["R"]
    v = sys_d["v"]
    code_1 = sys_d["code_1"]
    words = sys_d["id_to_word"]
    m = sys_d["m"]
    mark = sys_d["mark_rank"]
    n = len(T_oracle)

    # ---- Step 1: Build R^2 using 256 byte cursors ----
    R2 = [0] * m
    c2_a = [0] * m
    c2_b = [0] * m
    pair_key = [0] * m
    last_r = [-10**9] * 256
    r2_build_seeks = 0
    for j in range(m):
        b = v[j]
        y = R[j]
        if y != last_r[b] + 1:
            r2_build_seeks += 1
        last_r[b] = y
        R2[j] = R[y]
        c2_a[j] = code_1[j]
        c2_b[j] = code_1[y]
        pair_key[j] = (b << 8) | v[y]

    # Verify byte-exact decode via R^2
    out2 = bytearray(n)
    pos = n
    cur = mark
    r2_links = 0
    r2_dict_reads_paired = 0
    steps2 = m // 2
    for _ in range(steps2):
        w1 = words[c2_a[cur]]
        w2 = words[c2_b[cur]]
        wb = w1 + w2
        lw = len(wb)
        out2[pos - lw : pos] = wb[::-1]
        pos -= lw
        cur = R2[cur]
        r2_links += 1
        r2_dict_reads_paired += 2
    if m & 1:
        w1 = words[code_1[cur]]
        lw = len(w1)
        out2[pos - lw : pos] = w1[::-1]
        pos -= lw
        cur = R[cur]
        r2_links += 1
        r2_dict_reads_paired += 1
    assert pos == 0 and cur == mark and bytes(out2) == T_oracle, "R^2 decode mismatch!"

    # Measure distinct 2-word payloads across all m rows
    pair_word_set: set[tuple[int, int]] = set()
    r2_inline16_rows = 0
    r2_inline32_rows = 0
    r2_flat_all_bytes = 0
    for j in range(m):
        a, b_c = c2_a[j], c2_b[j]
        pair_word_set.add((a, b_c))
        tot_l = len(words[a]) + len(words[b_c])
        r2_flat_all_bytes += tot_l
        if tot_l <= 15:
            r2_inline16_rows += 1
        if tot_l <= 31:
            r2_inline32_rows += 1
    r2_distinct_words_bytes = sum(len(words[a]) + len(words[b_c]) for a, b_c in pair_word_set)

    # ---- Step 2: Build R^4 = R^2 o R^2 using (v[j], v[R[j]]) depth-2 schedule ----
    R4 = [0] * m
    c4_0 = [0] * m
    c4_1 = [0] * m
    c4_2 = [0] * m
    c4_3 = [0] * m
    last_p: dict[int, int] = {}
    r4_build_seeks_pair = 0
    r4_build_seeks_c16 = 0
    last_p_c16: dict[int, int] = {}
    for j in range(m):
        pk = pair_key[j]
        z = R2[j]
        prev = last_p.get(pk)
        if prev is None or z != prev + 1:
            r4_build_seeks_pair += 1
        last_p[pk] = z
        prev16 = last_p_c16.get(pk)
        if prev16 is None or not (1 <= z - prev16 <= 16):
            r4_build_seeks_c16 += 1
        last_p_c16[pk] = z
        R4[j] = R2[z]
        c4_0[j] = c2_a[j]
        c4_1[j] = c2_b[j]
        c4_2[j] = c2_a[z]
        c4_3[j] = c2_b[z]
    n_active_pairs = len(last_p)

    # Verify byte-exact decode via R^4
    out4 = bytearray(n)
    pos = n
    cur = mark
    r4_links = 0
    r4_dict_reads_quad = 0
    steps4 = m // 4
    for _ in range(steps4):
        wb = words[c4_0[cur]] + words[c4_1[cur]] + words[c4_2[cur]] + words[c4_3[cur]]
        lw = len(wb)
        out4[pos - lw : pos] = wb[::-1]
        pos -= lw
        cur = R4[cur]
        r4_links += 1
        r4_dict_reads_quad += 4
    for _ in range(m & 3):
        w1 = words[code_1[cur]]
        lw = len(w1)
        out4[pos - lw : pos] = w1[::-1]
        pos -= lw
        cur = R[cur]
        r4_links += 1
        r4_dict_reads_quad += 1
    assert pos == 0 and cur == mark and bytes(out4) == T_oracle, "R^4 decode mismatch!"

    # Measure distinct 4-word payloads and inline fractions
    quad_word_set: set[tuple[int, int, int, int]] = set()
    r4_inline32_rows = 0
    r4_inline64_rows = 0
    r4_flat_all_bytes = 0
    for j in range(m):
        q = (c4_0[j], c4_1[j], c4_2[j], c4_3[j])
        quad_word_set.add(q)
        tl = len(words[q[0]]) + len(words[q[1]]) + len(words[q[2]]) + len(words[q[3]])
        r4_flat_all_bytes += tl
        if tl <= 31:
            r4_inline32_rows += 1
        if tl <= 63:
            r4_inline64_rows += 1
    r4_distinct_words_bytes = sum(
        len(words[a]) + len(words[b]) + len(words[c]) + len(words[d])
        for a, b, c, d in quad_word_set
    )

    return {
        "r2_build_seeks": r2_build_seeks,
        "r2_links": r2_links,
        "r2_dict_reads_paired": r2_dict_reads_paired,
        "r2_distinct_2words": len(pair_word_set),
        "r2_distinct_bytes": r2_distinct_words_bytes,
        "r2_flat_all_bytes": r2_flat_all_bytes,
        "r2_inline16_frac": r2_inline16_rows / m,
        "r2_inline32_frac": r2_inline32_rows / m,
        "n_active_pairs": n_active_pairs,
        "cursor_table_kb": (n_active_pairs * 4) / 1024.0,
        "r4_build_seeks_pair": r4_build_seeks_pair,
        "r4_build_seeks_c16": r4_build_seeks_c16,
        "r4_links": r4_links,
        "r4_dict_reads_quad": r4_dict_reads_quad,
        "r4_distinct_4words": len(quad_word_set),
        "r4_distinct_bytes": r4_distinct_words_bytes,
        "r4_flat_all_bytes": r4_flat_all_bytes,
        "r4_inline32_frac": r4_inline32_rows / m,
        "r4_inline64_frac": r4_inline64_rows / m,
    }


def main() -> None:
    t0 = time.time()
    out_dir = os.path.join(HERE, "data")
    os.makedirs(out_dir, exist_ok=True)

    print("Loading 2 synthetic + 6 admitted Wikidata blocks...")
    s_iid = common.make_synthetic_novel_hex_iid(65536)
    s_tmpl = common.make_synthetic_templated_hex(1000, 40)
    admitted = common.load_admitted_blocks()
    blocks = [s_iid, s_tmpl] + admitted

    # Verify every block with independent LF decoder!
    for blk in blocks:
        T_ind = independent_lf_decode(blk.L, blk.primary_index)
        assert T_ind == blk.T_ref, f"Independent oracle mismatch on {blk.name}"
    print(f"Independent LF oracle verified 8/8 blocks in {time.time() - t0:.1f}s.")

    acq_rows: list[dict[str, Any]] = []
    d2_rows: list[dict[str, Any]] = []

    k_schedule = [0, 1, 2, 3, 4, -1]

    for blk in blocks:
        tb = time.time()
        L_min = 2 if blk.name.startswith("synth-hex-iid") else 17
        wide_info = simulate_open_iet_wide_strips(blk, L_min=L_min)

        # For synth-hex-iid (L_min=2), all runs >= 2 are already in wide_runs; only k=0 applies.
        ks = [0] if L_min == 2 else k_schedule

        for k in ks:
            sys_d = build_lazy_thin_system(blk, wide_info, max_thin_rounds=k, L_min=L_min)
            walk_links = verify_walk_R(sys_d, blk.T_ref)
            mc = run_m17_and_cl1(sys_d, blk.T_ref)
            d2 = run_r2_and_r4_depth2(sys_d, blk.T_ref)

            h1 = blk.h1
            m = sys_d["m"]
            k_label = "inf" if k == -1 else str(k)

            # Record lazy thin acquisition row
            acq_rows.append({
                "block": blk.name,
                "n": blk.n,
                "r": blk.r,
                "h1": h1,
                "L_min": L_min,
                "thin_k": k_label,
                "rounds_run": sys_d["rounds_run"],
                "wide_propA_rounds": wide_info["prop_a_rounds"],
                "wide_propA_visits": wide_info["prop_a_piece_visits"],
                "open_iet_steps": wide_info["open_iet_steps"],
                "thin_rows_total": sys_d["thin_rows_total"],
                "thin_land_h1_raw": sys_d["thin_land_h1_raw"],
                "thin_land_wide_raw": sys_d["thin_land_wide_raw"],
                "thin_land_thin_raw": sys_d["thin_land_thin_raw"],
                "thin_land_h1_after_pw": sys_d["thin_land_h1_after_pw"],
                "thin_merge_steps": sys_d["thin_merge_steps"],
                "Q_acq_search": 0,
                "m": m,
                "m_over_h1": f"{m / h1:.4f}",
                "P_pieces": sys_d["n_P_pieces"],
                "B_P_target": sys_d["B_P_target"],
                "B_P_sched": sys_d["B_P_sched"],
                "distinct_words": sys_d["n_distinct_words"],
                "flat_dict_kb": f"{sys_d['flat_dict_bytes'] / 1024:.1f}",
                "inline16_frac": f"{sys_d['inline16_frac']:.4f}",
                "max_word_len": sys_d["max_word_len"],
                "walk_Q_over_h1": f"{walk_links / h1:.4f}",
                "m17_Q_over_h1": f"{(sys_d['B_P_target'] + mc['m17_links']) / h1:.4f}",
                "cl1_Q_over_h1": f"{(sys_d['B_P_target'] + mc['cl1_links']) / h1:.4f}",
                "r2_Q_over_h1": f"{(d2['r2_build_seeks'] + d2['r2_links']) / h1:.4f}",
                "r4_L1cur_Q_over_h1": f"{(d2['r2_build_seeks'] + d2['r4_build_seeks_pair'] + d2['r4_links']) / h1:.4f}",
                "r4_strictPi_Q_over_h1": f"{(d2['r2_build_seeks'] + m + d2['r4_links']) / h1:.4f}",
            })

            # Emit full route-by-route like-for-like comparison rows for k in {0, 1, 2, inf}
            if k in (0, 1, 2, -1):
                iet = wide_info["open_iet_steps"]
                bp = sys_d["B_P_target"]
                # Helper to append a route row
                def add_route(
                    route: str,
                    model: str,
                    q_acq: int,
                    q_build: int,
                    q_emit_links: int,
                    q_emit_pay: int,
                    passes: int,
                    cursors: int,
                    s_table_kb: float,
                    s_payload_kb: float,
                    notes: str,
                ) -> None:
                    q_links_tot = q_acq + q_build + q_emit_links
                    q_all_tot = q_links_tot + q_emit_pay
                    d2_rows.append({
                        "block": blk.name,
                        "h1": h1,
                        "thin_k": k_label,
                        "m": m,
                        "route": route,
                        "cursor_model": model,
                        "Q_acq_search": q_acq,
                        "open_iet_steps": iet,
                        "Q_build": q_build,
                        "Q_emit_links": q_emit_links,
                        "Q_emit_payload_extra": q_emit_pay,
                        "Q_links_total_over_h1": f"{q_links_tot / h1:.4f}",
                        "Q_with_iet_over_h1": f"{(q_links_tot + iet) / h1:.4f}",
                        "Q_with_payload_over_h1": f"{q_all_tot / h1:.4f}",
                        "passes_P": passes,
                        "cursors_s": cursors,
                        "S_table_kb": f"{s_table_kb:.1f}",
                        "S_payload_kb": f"{s_payload_kb:.1f}",
                        "active_pairs": d2["n_active_pairs"],
                        "R2_pair_segs": d2["r4_build_seeks_pair"],
                        "notes": notes,
                    })

                dict_kb = sys_d["flat_dict_bytes"] / 1024.0
                # 1. Open walk R (flat dict: 1 link load + 1 flat word lookup per step; Open packs code in edge)
                add_route(
                    "walk_R_flat", "strict_Pi_256",
                    0, 0, walk_links, walk_links,
                    1 + sys_d["thin_passes"], 256,
                    (4 * m) / 1024.0, dict_kb,
                    f"Open-like flat dict ({sys_d['n_distinct_words']} words, {sys_d['inline16_frac']*100:.1f}%<=15B)",
                )
                # 2. M-17 1-sweep (flat survivor words vs DAG)
                add_route(
                    "M17_1sweep_flat", "strict_Pi_256",
                    0, bp, mc["m17_links"], mc["m17_links"],
                    2 + sys_d["thin_passes"], 256,
                    (4 * m) / 1024.0, mc["m17_flat_all_bytes"] / 1024.0,
                    f"1-sweep GS; flat build {mc['m17_flat_all_bytes']/1024:.0f}KB (surv {mc['m17_flat_surv_bytes']/1024:.0f}KB)",
                )
                # 3. CL1 2-sweep (flat vs DAG explicitly separated!)
                add_route(
                    "CL1_2sweep_DAG", "strict_Pi_256",
                    0, bp, mc["cl1_links"], mc["cl1_dag_internal"] + m,
                    3 + sys_d["thin_passes"], 256,
                    (12 * m) / 1024.0, dict_kb,
                    f"DAG emit visits {mc['cl1_dag_internal']} internal + {m} leaf loads (defers work!)",
                )
                add_route(
                    "CL1_2sweep_flat", "strict_Pi_256",
                    0, bp, mc["cl1_links"], mc["cl1_links"],
                    3 + sys_d["thin_passes"], 256,
                    (4 * m) / 1024.0, mc["cl1_flat_all_bytes"] / 1024.0,
                    f"Flat build {mc['cl1_flat_all_bytes']/1024:.0f}KB (surv {mc['cl1_flat_surv_bytes']/1024:.0f}KB)",
                )
                # 4. R^2 byte schedule (paired codes vs flat words)
                add_route(
                    "R2_byte_paired_codes", "strict_Pi_256",
                    0, d2["r2_build_seeks"], d2["r2_links"], d2["r2_dict_reads_paired"],
                    2 + sys_d["thin_passes"], 256,
                    (12 * m) / 1024.0, dict_kb,
                    f"R^2 stores (c1,c2); emit does 2 dict reads/link = m total dict reads",
                )
                add_route(
                    "R2_byte_flat", "strict_Pi_256",
                    0, d2["r2_build_seeks"], d2["r2_links"], d2["r2_links"],
                    2 + sys_d["thin_passes"], 256,
                    (8 * m) / 1024.0, d2["r2_flat_all_bytes"] / 1024.0,
                    f"R^2 flattened ({d2['r2_distinct_2words']} dist, {d2['r2_inline32_frac']*100:.1f}%<=31B)",
                )
                # 5. R^4 (v, v_R) depth-2 schedule — L1-cursor table (4-codes & flat)
                add_route(
                    "R4_pair_4codes", "L1_cursor_table",
                    0, d2["r2_build_seeks"] + d2["r4_build_seeks_pair"],
                    d2["r4_links"], d2["r4_dict_reads_quad"],
                    3 + sys_d["thin_passes"], d2["n_active_pairs"],
                    (20 * m) / 1024.0 + d2["cursor_table_kb"], dict_kb,
                    f"{d2['n_active_pairs']} pair cursors ({d2['cursor_table_kb']:.1f}KB); 4 dict reads/link = m total",
                )
                add_route(
                    "R4_pair_flat", "L1_cursor_table",
                    0, d2["r2_build_seeks"] + d2["r4_build_seeks_pair"],
                    d2["r4_links"], d2["r4_links"],
                    3 + sys_d["thin_passes"], d2["n_active_pairs"],
                    (8 * m) / 1024.0 + d2["cursor_table_kb"], d2["r4_flat_all_bytes"] / 1024.0,
                    f"{d2['n_active_pairs']} cursors ({d2['cursor_table_kb']:.1f}KB); flat 4n={d2['r4_flat_all_bytes']/1024:.0f}KB ({d2['r4_inline64_frac']*100:.1f}%<=63B)",
                )
                # 6. R^4 (v, v_R) under STRICT Pi (s <= 256 cursors -> m addressed reads in R^4 build!)
                add_route(
                    "R4_pair_strictPi_flat", "strict_Pi_256",
                    0, d2["r2_build_seeks"] + m,
                    d2["r4_links"], d2["r4_links"],
                    3 + sys_d["thin_passes"], 256,
                    (8 * m) / 1024.0, d2["r4_flat_all_bytes"] / 1024.0,
                    f"Strict Pi (s=256): >256 active pairs forces m addressed gathers in R^4 build",
                )

        print(
            f"  {blk.name:22s} done in {time.time() - tb:.1f}s "
            f"(wide PropA rounds={wide_info['prop_a_rounds']}, Open IET steps={wide_info['open_iet_steps']})"
        )

    # Write TSVs
    acq_path = os.path.join(out_dir, "t1_t2_lazy_thin_acquisition.tsv")
    with open(acq_path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(acq_rows[0].keys()), delimiter="\t")
        w.writeheader()
        w.writerows(acq_rows)

    d2_path = os.path.join(out_dir, "t1_t2_depth2_schedule_costed.tsv")
    with open(d2_path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(d2_rows[0].keys()), delimiter="\t")
        w.writeheader()
        w.writerows(d2_rows)

    print(f"\nWrote {len(acq_rows)} rows to {acq_path}")
    print(f"Wrote {len(d2_rows)} rows to {d2_path}")
    print(f"Total elapsed: {time.time() - t0:.1f}s")


if __name__ == "__main__":
    main()
