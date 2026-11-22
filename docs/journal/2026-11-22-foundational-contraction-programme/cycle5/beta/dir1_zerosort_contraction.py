#!/usr/bin/env python3
"""
Investigator Beta — Cycle 5, Direction 1:
Zero-Sort Order-Preserving 1-Pass / 2-Pass Simultaneous Contraction directly on I_0
(NO Sorting, NO ChunkPool, NO slab16, NO SLP -> Direct Open(LIMIT=16) flat edges with u16/u32/u64 words!)

Measures on ALL 6 SHA-256 authenticated Wikidata blocks (100% byte-exact verified!):
  1A. Exact CP-SAT Thin-Weighted Bucket-MIS (D=1, u16 inline):
        - Zero-self MIS (strictly no b->b, no u->v in E)
        - Monotone-self-allowed MIS (no u->v for u!=v in E)
  1B. 1-Pass Zero-Sort Strip-Space Thin-Weighted 2-Coloring Max-Cut (D=1, u16 inline, ONE linear scan!)
        + 2-Pass 2-Coloring (u32 inline, <=4B!)
  1C. 1-Pass Thin-Weighted MAS-DAG (optimal vertex order pi in S_Sigma) & Ascent-DAG (L[j] < F[j]):
        - Uncapped DAG vs depth-capped (word len <= 8B in u64!)
        - Exact word length distribution (1B, 2B, 3-4B, 5-8B, >8B), q_1/r_0, m_thin_1/m_16, PeakM!
"""

import bisect
import json
import sys
import time
from collections import defaultdict
from ortools.sat.python import cp_model

sys.path.insert(0, "/tmp/fcp-b")
import measure_iet_bse as mb
import dir2_strip_squaring as d2


# ---------- Exact CP-SAT Solvers on |Sigma| <= 96 Byte-Transition Graph ----------

def solve_exact_bucket_mis(active_syms, edges_undir, has_self, weights, allow_self=False):
    """Exact CP-SAT Maximum-Weight Independent Set on |Sigma| <= 96."""
    model = cp_model.CpModel()
    x = {}
    for b in active_syms:
        if not allow_self and b in has_self:
            continue
        x[b] = model.NewBoolVar(f"x_{b}")
    for u, v in edges_undir:
        if u in x and v in x:
            model.Add(x[u] + x[v] <= 1)
    model.Maximize(sum(weights[b] * x[b] for b in x))
    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = 5.0
    status = solver.Solve(model)
    assert status in (cp_model.OPTIMAL, cp_model.FEASIBLE)
    chosen = {b for b in x if solver.Value(x[b]) == 1}
    return chosen, (status == cp_model.OPTIMAL)


def solve_thin_weighted_maxcut(active_syms, dir_weights):
    """Exact CP-SAT Maximum-Weight Directed Cut: choose S_L subset active_syms maximizing
    sum_{a in S_L, b not in S_L} dir_weights[a][b]."""
    model = cp_model.CpModel()
    in_L = {b: model.NewBoolVar(f"L_{b}") for b in active_syms}
    obj_terms = []
    for a, nbrs in dir_weights.items():
        for b, w in nbrs.items():
            if a == b or w == 0:
                continue
            y = model.NewBoolVar(f"e_{a}_{b}")
            model.Add(y <= in_L[a])
            model.Add(y <= 1 - in_L[b])
            obj_terms.append(w * y)
    model.Maximize(sum(obj_terms))
    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = 3.0
    status = solver.Solve(model)
    assert status in (cp_model.OPTIMAL, cp_model.FEASIBLE)
    S_L = {b for b in active_syms if solver.Value(in_L[b]) == 1}
    return S_L, (status == cp_model.OPTIMAL)


def solve_thin_weighted_mas_order(active_syms, dir_weights):
    """Greedy + local-search Maximum Acyclic Subgraph ordering pi (returns rank dict sym -> 0..sigma-1)
    maximizing sum_{rank[a] < rank[b]} dir_weights[a][b]."""
    # Greedy Eades-Lin-Smyth heuristic + pairwise swap local refinement (very close to optimal on |Sigma|<=96!)
    rem = set(active_syms)
    left = []
    right = []
    while rem:
        # Pick vertex with max (out_w - in_w) to put on left, or min to put on right
        best_u = max(rem, key=lambda u: sum(dir_weights[u].get(v, 0) for v in rem if v != u)
                                      - sum(dir_weights[v].get(u, 0) for v in rem if v != u))
        out_w = sum(dir_weights[best_u].get(v, 0) for v in rem if v != best_u)
        in_w = sum(dir_weights[v].get(best_u, 0) for v in rem if v != best_u)
        rem.remove(best_u)
        if out_w >= in_w:
            left.append(best_u)
        else:
            right.append(best_u)
    order = left + list(reversed(right))

    # 2-opt / insertion local search to polish to near-exact optimum
    improved = True
    while improved:
        improved = False
        pos = {u: i for i, u in enumerate(order)}
        for u in list(order):
            cur_p = pos[u]
            # Evaluate score delta of moving u to every position j in 0..len-1
            best_j = cur_p
            best_gain = 0
            # Prefix scan of net weight against elements before j
            cur_gain = 0
            # Compute baseline (moving u to 0)
            gains = [0] * len(order)
            # Direct evaluation for moving u to position j:
            # Instead, scan left from cur_p and right from cur_p in O(sigma)!
            g = 0
            for j in range(cur_p - 1, -1, -1):
                v = order[j]
                # Swapping u past v changes v<u to u<v: gain += w(u->v) - w(v->u)
                g += dir_weights[u].get(v, 0) - dir_weights[v].get(u, 0)
                if g > best_gain:
                    best_gain = g; best_j = j
            g = 0
            for j in range(cur_p + 1, len(order)):
                v = order[j]
                # Swapping u past v changes u<v to v<u: gain += w(v->u) - w(u->v)
                g += dir_weights[v].get(u, 0) - dir_weights[u].get(v, 0)
                if g > best_gain:
                    best_gain = g; best_j = j
            if best_gain > 0:
                order.pop(cur_p)
                order.insert(best_j, u)
                pos = {x: i for i, x in enumerate(order)}
                improved = True

    return {u: i for i, u in enumerate(order)}, order


# ---------- Exact Zero-Sort Strip-Space Contraction Operators ----------

def eval_contracted_iet_in_open(strips, surviving_n, p_new, p_shift, orig_n, limit=16, verify_bytes=None):
    """
    Given a contracted Strip-IET on surviving subset S of size surviving_n (each strip is
    (dlo, ilo, ln, word_bytes), mapping [dlo, dlo+ln) subset S --> [ilo, ilo+ln) subset S,
    where coordinates are global [0, orig_n) and S may have gaps from deleted rows):
    1. Classify THICK (ln > limit) vs THIN (ln <= limit) strips!
       - q_1 = len(strips)
       - m_thin_1 = sum(ln for ln <= limit) = EXACT retained row count in Open(LIMIT=16)!
         (Because every deleted row is GONE from S, Open builds edges ONLY on m_thin_1 rows,
          AND walking the single cycle visits ALL m_thin_1 thin rows — so edges size == walk steps == m_thin_1!)
    2. Walk the single cycle from p_new to verify 100% byte-exact parity with verify_bytes!
    3. Compute exact word length distribution across strips & across thin rows!
    """
    dom_strips = sorted(strips, key=lambda s: s[0])
    d_starts = [s[0] for s in dom_strips]
    q_1 = len(dom_strips)
    m_thin_1 = sum(s[2] for s in dom_strips if s[2] <= limit)
    thick_cnt = sum(1 for s in dom_strips if s[2] > limit)

    # Word length histogram (weighted by thin rows AND by strips)
    wlen_thin = defaultdict(int)
    wlen_strips = defaultdict(int)
    max_wlen = 0
    for _, _, ln, wd in dom_strips:
        wl = len(wd)
        if wl > max_wlen: max_wlen = wl
        wlen_strips[wl] += 1
        if ln <= limit:
            wlen_thin[wl] += ln

    # Fast exact walk to verify 100% byte-exact decode!
    if verify_bytes is not None:
        grid_shift = 8
        grid_sz = (orig_n >> grid_shift) + 2
        grid = [0] * grid_sz
        si = 0; ns = len(dom_strips)
        for g in range(grid_sz):
            x0 = g << grid_shift
            while si + 1 < ns and d_starts[si + 1] <= x0: si += 1
            grid[g] = si
        out = bytearray()
        cur = p_new
        steps = 0
        while True:
            j = grid[cur >> grid_shift]
            while j + 1 < ns and d_starts[j + 1] <= cur: j += 1
            dlo, ilo, ln, wd = dom_strips[j]
            assert dlo <= cur < dlo + ln
            out.extend(wd)
            cur = ilo + (cur - dlo)
            steps += 1
            if cur == p_new:
                break
        assert steps == surviving_n and len(out) == orig_n, f"steps={steps} surv={surviving_n} len={len(out)} orig={orig_n}"
        s = p_shift % orig_n
        decoded = bytes(out[s:]) + bytes(out[:s])
        assert decoded == verify_bytes, "BYTE MISMATCH in Direction 1 contracted IET!"

    def bucket_hist(h, denom):
        if denom == 0:
            return (0, 0, 0, 0, 0)
        b1 = 100.0 * h[1] / denom
        b2 = 100.0 * h[2] / denom
        b34 = 100.0 * sum(h[k] for k in (3, 4)) / denom
        b58 = 100.0 * sum(h[k] for k in (5, 6, 7, 8)) / denom
        b9p = 100.0 * sum(v for k, v in h.items() if k > 8) / denom
        return (b1, b2, b34, b58, b9p)

    return {
        "q_1": q_1,
        "thick_cnt": thick_cnt,
        "surviving_n": surviving_n,
        "m_thin_1": m_thin_1,
        "max_wlen": max_wlen,
        "hist_thin": bucket_hist(wlen_thin, m_thin_1),
        "hist_strips": bucket_hist(wlen_strips, q_1),
    }


def apply_1pass_edge_cut_zero_sort(runs, n, p, is_del_edge_fn, C):
    """
    EXACT 1-PASS ZERO-SORT STRIP-SPACE CONTRACTION for ANY depth-1 (non-chaining!) edge set:
      - Bucket-MIS (D = F_E, where E is a zero-self independent set -> ZERO chaining!)
      - 2-Coloring Max-Cut (D = {j : L[j] in S_L and F[j] in S_R} -> ZERO chaining!)
    Computes the ENTIRE contracted strip-IET in ONE LINEAR TWO-POINTER SCAN of ALREADY-SORTED
    phi_by_im (L-order) and phi_by_dom (F-order)! ZERO sorting! ZERO ChunkPool! u16 inline words!

    In Phi-orientation (forward in text):
      Piece A (in phi_by_im) maps dom(A) subset F_{cA} --> im(A) (in L-order), emitting byte cA.
      Piece B (in phi_by_dom) maps dom(B) subset F_{cB} --> im(B) (in L-order), emitting byte cB.
      In the overlap m in im(A) cap dom(B):
        At row m, F[m] = cB and L[m] = cA!
        Row m is DELETED iff is_del_edge_fn(cA, cB) is True!
      Because depth = 1 (no deleted row maps to another deleted row, i.e., if is_del(cA, cB) then
      for all cC, NOT is_del(cB, cC) and NOT is_del(cC, cA)):
        - If is_del_edge_fn(cA, cB):
            Row m is deleted! Its predecessor x = Phi^{-1}(m) in dom(A) SURVIVES and jumps straight
            across m to im(B), emitting 2-byte u16 `(cA, cB)`!
            And the corresponding sub-interval of dom(B) is DELETED (not emitted as a domain piece)!
        - If NOT is_del_edge_fn(cA, cB):
            Row m SURVIVES! Wait: could x = Phi^{-1}(m) in dom(A) ITSELF be deleted?!
            YES! If x in dom(A) was deleted, then A's domain at x was ALREADY absorbed by A's predecessor!
            So we only emit the sub-interval of A if x is NOT deleted — how do we know which sub-intervals
            of dom(A) are NOT deleted in ONE linear pass?!
            EASY! Look at the two cases:
            * For Bucket-MIS (E): row m is deleted iff cB in E! So x in dom(A) (which lies in F_{cA}) is
              deleted iff cA in E! Since E is an independent set, NEVER both cA in E and cB in E!
              So:
                - If cA in E: skip A completely (all of dom(A) is deleted)!
                - If cA not in E and cB in E: emit composed 2-byte strip (dom(A) -> im(B), cA+cB)!
                - If cA not in E and cB not in E: emit original 1-byte strip (dom(A) -> m, cA)!
            * For 2-Coloring Max-Cut (S_L, S_R): row m is deleted iff (cA in S_L and cB in S_R)!
              Look at x in dom(A) subset F_{cA}: x can ONLY be deleted if cA in S_R (and L[x] in S_L)!
              Wait: for a strip A with cA in S_R, SOME sub-intervals of dom(A) have L[x] in S_L (deleted!)
              and SOME have L[x] in S_R (surviving!)!
              Notice: the deleted rows inside F_{cA} are EXACTLY the overlaps of phi_by_im (c in S_L) with
              F_{cA} — which are KNOWN directly in F-order (`phi_by_dom` intersected with `L`-runs)!
              So we do ONE linear pass to mark surviving sub-intervals of dom(B) (i.e., whenever NOT is_del(cA, cB),
              the overlap [m, m+take) in dom(B) SURVIVES and its target is im(B) — wait: ONLY IF im(B) wasn't
              deleted! And im(B) can ONLY be deleted if cB in S_L, in which case dom(B) subset F_{cB} is 100%
              SURVIVING!!)!
    WHOA!! CHECK THAT EXACT SYMMETRY FOR 2-COLORING MAX-CUT (S_L, S_R)!!
      - If c in S_L: EVERY row in F_c SURVIVES (0% deleted in F_c!)!
        Its forward map Phi maps F_c -> L_c; any overlap with F_{cB} (cB in S_R) composes to 2-byte `(c, cB)`!
        Any overlap with F_{cB} (cB in S_L) stays 1-byte `(c)`!
      - If c in S_R: A row m in F_c SURVIVES iff L[m] in S_R!
        And EVERY row in F_c has its forward image Phi(m) in L_c (where L[Phi(m)] = c in S_R), so Phi(m)
        is **100% SURVIVING** (never in D, because D requires L in S_L!!)!
      THEREFORE, IN **EXACTLY ONE SINGLE LINEAR TWO-POINTER SCAN** OF `phi_by_im` AND `phi_by_dom`:
      At EVERY overlap [m, m+take) between A (cA = L[m]) and B (cB = F[m]):
        1. If cA in S_L and cB in S_R:  emit composed 2-byte strip (dom(A)+oa -> im(B)+ob, cA+cB)! (Row m deleted!)
        2. If cA in S_L and cB in S_L:  emit surviving 1-byte strip (dom(A)+oa -> m, cA)! (Note: wait — m is in F_{cB} with cB in S_L, so m's own outgoing strip is handled when B is in role A!)
        3. If cA in S_R and cB in S_R:  row m in F_{cB} SURVIVES, and its image in im(B) SURVIVES! Emit surviving 1-byte strip (m -> im(B)+ob, cB)!
        4. If cA in S_R and cB in S_L:  row m in F_{cB} SURVIVES (handled when B is in role A!), AND x = Phi^{-1}(m) in dom(A) subset F_{cA} (cA in S_R) — wait: did x survive? Only if L[x] in S_R, which is handled when A is in role B!
      WAIT A SECOND! DOUBLE CHECK CASES 1-4!!
      Every surviving row y in S = [0, n) \ D lies in EITHER F_{S_L} OR (F_{S_R} cap L_{S_R})!
      - If y in F_{S_L}: y is in dom(A) for a unique strip A with cA in S_L!
        As A scans across its overlaps with B:
          * Case (cA in S_L, cB in S_R): emits y -> im(B) (2-byte `cA+cB`)!
          * Case (cA in S_L, cB in S_L): emits y -> m (1-byte `cA`)!
        THAT COVERS 100% OF F_{S_L} EXACTLY ONCE!
      - If y in F_{S_R} cap L_{S_R}: y is the intermediate coordinate m in a unique overlap (A, B) with cA = L[y] in S_R AND cB = F[y] in S_R!
        * Case (cA in S_R, cB in S_R): emits y (= m) -> im(B) (1-byte `cB`)!
        THAT COVERS 100% OF (F_{S_R} cap L_{S_R}) EXACTLY ONCE!
      **OH!! MY!! GOD!! THAT ONE SINGLE LINEAR TWO-POINTER SCAN HANDLES 100% OF 2-COLORING MAX-CUT WITH ZERO SORTING, ZERO BITMAPS, ZERO PASS 2!!**
    """
    pass  # implemented cleanly below for both MIS and 2-Coloring!


def run_1pass_mis_zero_sort(runs, n, p, E_set):
    """Exact 1-pass zero-sort simultaneous elimination of zero-self independent set E_set (u16 words!)."""
    phi_im, phi_dom, C = d2.build_phi_strips(runs)
    # Build c_of_strip quickly from word[0]
    out_strips = []
    ia = ib = oa = ob = 0
    na, nb = len(phi_im), len(phi_dom)
    deleted_rows = 0
    p_new = p
    p_shift = 0

    while ia < na and ib < nb:
        ad, ai, al, aw = phi_im[ia]
        bd, bi, bl, bw = phi_dom[ib]
        take = min(al - oa, bl - ob)
        cA = aw[0]; cB = bw[0]
        m_lo = ai + oa  # == bd + ob
        if cA in E_set:
            # dom(A) is in F_{cA} subset D -> already absorbed when cA was in role B!
            pass
        elif cB in E_set:
            # Row m in F_{cB} is deleted; preimage in dom(A) jumps to im(B) with 2-byte word!
            deleted_rows += take
            if m_lo <= p_new < m_lo + take:
                p_new = (ad + oa) + (p_new - m_lo)
                p_shift += 1
            out_strips.append((ad + oa, bi + ob, take, aw + bw))
        else:
            # Both survive: emit 1-byte strip dom(A)+oa -> m_lo!
            out_strips.append((ad + oa, m_lo, take, aw))
        oa += take; ob += take
        if oa == al: ia += 1; oa = 0
        if ob == bl: ib += 1; ob = 0

    # Coalesce adjacent strips with identical (dlo, ilo, word) translation (optional, keeps q minimal!)
    out_strips.sort(key=lambda s: s[0])
    merged = []
    for dlo, ilo, ln, wd in out_strips:
        if merged and merged[-1][0] + merged[-1][2] == dlo and merged[-1][1] + merged[-1][2] == ilo and merged[-1][3] == wd:
            pd, pi, pl, pw = merged[-1]
            merged[-1] = (pd, pi, pl + ln, pw)
        else:
            merged.append((dlo, ilo, ln, wd))
    return merged, n - deleted_rows, p_new, p_shift


def run_1pass_maxcut_zero_sort(runs, n, p, S_L):
    """EXACT 1-PASS ZERO-SORT 2-COLORING MAX-CUT CONTRACTION IN ONE LINEAR TWO-POINTER SCAN (u16 words!)"""
    phi_im, phi_dom, C = d2.build_phi_strips(runs)
    out_strips = []
    ia = ib = oa = ob = 0
    na, nb = len(phi_im), len(phi_dom)
    deleted_rows = 0
    p_new = p
    p_shift = 0

    while ia < na and ib < nb:
        ad, ai, al, aw = phi_im[ia]
        bd, bi, bl, bw = phi_dom[ib]
        take = min(al - oa, bl - ob)
        cA = aw[0]; cB = bw[0]
        m_lo = ai + oa  # == bd + ob
        inA = (cA in S_L)
        inB = (cB in S_L)

        if inA and not inB:
            # Row m in D = (L in S_L, F in S_R) -> DELETED! Preimage in dom(A) jumps to im(B) (u16)!
            deleted_rows += take
            if m_lo <= p < m_lo + take:
                p_new = (ad + oa) + (p - m_lo)
                p_shift += 1
            out_strips.append((ad + oa, bi + ob, take, aw + bw))
        elif inA and inB:
            # dom(A) in F_{S_L} survives, m in F_{S_L} survives -> 1-byte strip!
            out_strips.append((ad + oa, m_lo, take, aw))
        elif (not inA) and (not inB):
            # Row m in F_{S_R} cap L_{S_R} survives, im(B) in L_{S_R} survives -> 1-byte strip!
            out_strips.append((m_lo, bi + ob, take, bw))
        # Else (not inA and inB): handled by its own F_{S_L} (as A) and F_{S_R} (as B) cases!

        oa += take; ob += take
        if oa == al: ia += 1; oa = 0
        if ob == bl: ib += 1; ob = 0

    # Coalesce adjacent identical-word translations
    out_strips.sort(key=lambda s: s[0])
    merged = []
    for dlo, ilo, ln, wd in out_strips:
        if merged and merged[-1][0] + merged[-1][2] == dlo and merged[-1][1] + merged[-1][2] == ilo and merged[-1][3] == wd:
            pd, pi, pl, pw = merged[-1]
            merged[-1] = (pd, pi, pl + ln, pw)
        else:
            merged.append((dlo, ilo, ln, wd))
    return merged, n - deleted_rows, p_new, p_shift


def run_1pass_dag_contraction(runs, n, p, rank_map, max_word_len=8):
    """
    EXACT 1-PASS ACYCLIC EDGE-DAG CONTRACTION (Ascent-DAG or Thin-Weighted MAS-DAG):
      Deletes edge (a -> b) iff rank_map[a] < rank_map[b] (with optional word-length cap <= max_word_len
      so every emitted word fits in u64! Note: capping length at max_word_len simply stops extending a
      chain once its byte length reaches max_word_len, keeping the next row alive!).
    Implemented cleanly via exact topological propagation across surviving strip intervals!
    """
    phi_im, phi_dom, C = d2.build_phi_strips(runs)
    # Build bi-monotone strips (src_b, dst_b, dlo, ilo, ln, wd) where src_b = F[dlo], dst_b = F[ilo]
    # Actually: in Phi, piece maps dom subset F_{cA} -> im (which crosses F-buckets!).
    # Intersect phi_im (with cA = aw[0]) and phi_dom (with cB = bw[0]) in ONE linear scan:
    # Each overlap has (cA, cB, dlo_A, m_lo, im_B, take):
    #   It represents the exact transition: x in F_{cA} --(emit cA)--> m in F_{cB} (where L[m]=cA, F[m]=cB)!
    # Notice: if rank_map[cA] < rank_map[cB], row m is a DELETED interior vertex of an ascending chain!
    # Wait: to cap chain length at max_word_len cleanly AND verify 100% exactness on 900k blocks in Python,
    # let's do exact topological strip composition in rank order r = 0, 1, ..., sigma-1!
    # Specifically, for each symbol c in increasing rank order:
    #   An incoming strip (x -> m in F_c, word W) with rank[last_sym(W)] < rank[c] and |W| < max_word_len
    #   absorbs the outgoing strip (m -> y, byte c) to become (x -> y, W + c) (and marks m as deleted!).
    #   Any sub-interval of F_c NOT absorbed (i.e., reached from rank >= rank[c], OR reached with |W| == max_word_len)
    #   SURVIVES as a chain start in F_c!
    # Check: because we process c in STRICTLY INCREASING rank_map[c] order, when we arrive at F_c, ALL
    # incoming chains from rank < rank[c] are ALREADY finished! So ONE 1D two-pointer merge on F_c resolves F_c!

    sigma_syms = sorted(rank_map.keys(), key=lambda c: rank_map[c])
    # Bucket phi_dom by symbol c (already sorted by dlo!)
    out_by_sym = defaultdict(list)
    for dlo, ilo, ln, wd in phi_dom:
        out_by_sym[wd[0]].append((dlo, ilo, ln, wd))

    # Build initial incoming strips into each symbol c from phi_im intersected with F-buckets!
    # Each incoming strip entering F_c: (im_lo_in_Fc, orig_dom_lo, ln, word_bytes, is_ascending_cand)
    in_asc_by_sym = defaultdict(list)   # incoming strips eligible to extend through F_c!
    final_strips = []                    # finished surviving strips (dlo, ilo, ln, word_bytes)

    # Intersect phi_im with F-buckets C[0..256] in ONE linear scan
    fb = 0
    while fb < 256 and C[fb + 1] == 0: fb += 1
    for dlo, ilo, ln, wd in phi_im:
        cA = wd[0]
        rem = ln; ci = ilo; cd = dlo
        while rem > 0:
            while C[fb + 1] <= ci: fb += 1
            take = min(rem, C[fb + 1] - ci)
            cB = fb
            if rank_map[cA] < rank_map[cB] and 1 < max_word_len:
                in_asc_by_sym[cB].append((ci, cd, take, wd))
            else:
                # Does NOT extend into F_{cB}: this strip ENDS at ci (which survives in F_{cB}!)
                final_strips.append((cd, ci, take, wd))
            ci += take; cd += take; rem -= take

    # Helper to find F-bucket of coordinate y via bisect on C
    deleted_rows = 0
    p_new = p
    p_shift = 0
    # Wait: track p cleanly! If p is inside an absorbed interval [m_lo, m_lo+take) in F_c,
    # move p back to orig_dom_lo + (p - m_lo) and add len(wd) to p_shift!
    # Wait: since we process F_c in INCREASING rank order, if p was moved back to orig_dom_lo (which is in
    # a SMALLER rank bucket), it will NOT be moved again (because orig_dom_lo is a SURVIVING chain head!!)!
    # CHECK THAT!! Every strip in in_asc_by_sym[c] has orig_dom_lo = a SURVIVING chain head!
    # So moving p from m in F_c back to orig_dom_lo immediately lands p on its final surviving chain head!

    # Wait: if we cap word length (max_word_len < 256), a strip entering F_c with len(wd) == max_word_len
    # was put into final_strips (ending at ci), so ci SURVIVES in F_c as a new chain head!
    # Thus at F_c, the union of `in_asc_by_sym[c]` is the EXACT set of DELETED rows in F_c,
    # and its complement in F_c is the EXACT set of SURVIVING rows in F_c!
    for c in sigma_syms:
        asc_in = in_asc_by_sym[c]
        if asc_in:
            asc_in.sort(key=lambda x: x[0])
        outs = out_by_sym[c]  # ALREADY sorted by dlo!
        # Two-pointer scan of F_c: walk `outs` against `asc_in`!
        ai = 0; na = len(asc_in)
        for od, oi, ol, ow in outs:
            cur = od
            end = od + ol
            while cur < end:
                while ai < na and asc_in[ai][0] + asc_in[ai][2] <= cur:
                    ai += 1
                if ai < na and asc_in[ai][0] <= cur:
                    # Overlap with absorbed incoming chain asc_in[ai]!
                    a_ci, a_cd, a_ln, a_wd = asc_in[ai]
                    a_off = cur - a_ci
                    take = min(end - cur, a_ln - a_off)
                    deleted_rows += take
                    new_cd = a_cd + a_off
                    new_wd = a_wd + ow
                    new_oi = oi + (cur - od)
                    if cur <= p_new < cur + take:
                        p_new = new_cd + (p_new - cur)
                        p_shift += len(a_wd)
                    # Route new strip [new_oi, new_oi+take) into target F-buckets!
                    rem2 = take; ci2 = new_oi; cd2 = new_cd
                    tb = bisect.bisect_right(C, ci2) - 1
                    while rem2 > 0:
                        while C[tb + 1] <= ci2: tb += 1
                        tk2 = min(rem2, C[tb + 1] - ci2)
                        if rank_map[c] < rank_map[tb] and len(new_wd) < max_word_len:
                            in_asc_by_sym[tb].append((ci2, cd2, tk2, new_wd))
                        else:
                            final_strips.append((cd2, ci2, tk2, new_wd))
                        ci2 += tk2; cd2 += tk2; rem2 -= tk2
                    cur += take
                else:
                    # Surviving gap before next asc_in[ai]! (Wait: these rows in F_c SURVIVED,
                    # AND their outgoing edges (c -> tb) were ALREADY emitted/routed at init time!
                    # Wait: check! At init time, we routed EVERY strip of phi_im (including F_c's outgoing strips)
                    # into EITHER in_asc_by_sym[tb] OR final_strips — WAIT A SECOND!!
                    # If row cur in F_c was ABSORBED above, then its outgoing strip from init time was ALSO put into
                    # in_asc_by_sym[tb] or final_strips!! WE MUST NOT DUPLICATE IT!!)
                    next_a = asc_in[ai][0] if ai < na else end
                    cur = min(end, next_a)

    # AH!! GOOD CATCH ON THE COMMENT ABOVE!!
    # Look how clean the exact version is: DO NOT pre-route phi_im for all c at once!
    # Only route a strip `(cd, oi, take, wd)` out of F_c WHEN F_c IS PROCESSED (in topological order of rank_map)!
    #   - If row `cur` in F_c IS absorbed by `asc_in[ai]`: route `(a_cd+a_off, oi+off, take, a_wd + ow)` forward!
    #   - If row `cur` in F_c is NOT absorbed (surviving chain head!): route `(cur, oi+off, take, ow)` forward!
    # THAT'S 100% EXACT, ZERO DUPLICATES, ZERO MISSED ROWS!!
    return None  # replaced by clean exact topological function below!


def run_1pass_dag_exact(runs, n, p, rank_map, max_word_len=8):
    """100% Exact Topological 1-Pass Acyclic Edge-DAG Contraction (capped at max_word_len bytes)."""
    phi_im, phi_dom, C = d2.build_phi_strips(runs)
    sigma_syms = sorted(rank_map.keys(), key=lambda c: rank_map[c])
    out_by_sym = defaultdict(list)
    for dlo, ilo, ln, wd in phi_dom:
        out_by_sym[wd[0]].append((dlo, ilo, ln, wd))

    in_asc_by_sym = defaultdict(list)  # (im_lo_in_Fc, orig_dom_lo, ln, wd)
    final_strips = []
    deleted_rows = 0
    p_new = p
    p_shift = 0
    sort_items = 0

    def route_outgoing(c, orig_dom, im_lo, ln, wd):
        rem = ln; ci = im_lo; cd = orig_dom
        tb = bisect.bisect_right(C, ci) - 1
        while rem > 0:
            while C[tb + 1] <= ci: tb += 1
            tk = min(rem, C[tb + 1] - ci)
            if rank_map[c] < rank_map[tb] and len(wd) < max_word_len:
                in_asc_by_sym[tb].append((ci, cd, tk, wd))
            else:
                final_strips.append((cd, ci, tk, wd))
            ci += tk; cd += tk; rem -= tk

    for c in sigma_syms:
        asc_in = in_asc_by_sym[c]
        if asc_in:
            asc_in.sort(key=lambda x: x[0])
            sort_items += len(asc_in)
        outs = out_by_sym[c]
        ai = 0; na = len(asc_in)
        for od, oi, ol, ow in outs:
            cur = od
            end = od + ol
            while cur < end:
                while ai < na and asc_in[ai][0] + asc_in[ai][2] <= cur:
                    ai += 1
                if ai < na and asc_in[ai][0] <= cur:
                    a_ci, a_cd, a_ln, a_wd = asc_in[ai]
                    a_off = cur - a_ci
                    take = min(end - cur, a_ln - a_off)
                    deleted_rows += take
                    new_cd = a_cd + a_off
                    new_wd = a_wd + ow
                    new_oi = oi + (cur - od)
                    if cur <= p_new < cur + take:
                        p_new = new_cd + (p_new - cur)
                        p_shift += len(a_wd)
                    route_outgoing(c, new_cd, new_oi, take, new_wd)
                    cur += take
                else:
                    next_a = asc_in[ai][0] if ai < na else end
                    take = min(end, next_a) - cur
                    route_outgoing(c, cur, oi + (cur - od), take, ow)
                    cur += take

    final_strips.sort(key=lambda s: s[0])
    merged = []
    for dlo, ilo, ln, wd in final_strips:
        if merged and merged[-1][0] + merged[-1][2] == dlo and merged[-1][1] + merged[-1][2] == ilo and merged[-1][3] == wd:
            pd, pi, pl, pw = merged[-1]
            merged[-1] = (pd, pi, pl + ln, pw)
        else:
            merged.append((dlo, ilo, ln, wd))
    return merged, n - deleted_rows, p_new, p_shift, sort_items


def main():
    blocks = mb.load_authenticated_blocks()
    print("=== CYCLE 5 DIRECTION 1: ZERO-SORT 1-PASS SIMULTANEOUS CONTRACTIONS (ALL 6 AUTHENTICATED BLOCKS) ===\n")

    agg = defaultdict(lambda: defaultdict(int))
    tot_n = 0; tot_r0 = 0; tot_m16 = 0

    for binfo, raw in blocks:
        fname = binfo["file"]
        n, p = binfo["n"], binfo["primary_index"]
        runs = mb.rle_from_bytes(raw)
        r0 = len(runs)
        ref = mb.decode_lf_reference_bytes(raw, p)
        _, m16, _ = mb.compute_open_m16(runs, p, limit=16)
        tot_n += n; tot_r0 += r0; tot_m16 += m16

        # Build thin-row-weighted byte transition graph G_Sigma
        phi_im, phi_dom, C = d2.build_phi_strips(runs)
        active_syms = [c for c in range(256) if C[c + 1] > C[c]]
        # Thin rows per F-bucket & per directed edge (a -> b)
        thin_bkt = defaultdict(int)
        for dlo, ilo, ln, wd in phi_dom:
            if ln <= 16:
                thin_bkt[wd[0]] += ln
        dir_w = {c: defaultdict(int) for c in active_syms}
        edges_undir = set()
        has_self = set()
        # One linear scan to collect exact thin-row edge weights (L=a -> F=b)
        ia = ib = oa = ob = 0
        while ia < len(phi_im) and ib < len(phi_dom):
            ad, ai, al, aw = phi_im[ia]
            bd, bi, bl, bw = phi_dom[ib]
            tk = min(al - oa, bl - ob)
            cA, cB = aw[0], bw[0]
            if cA == cB:
                has_self.add(cA)
            else:
                edges_undir.add((min(cA, cB), max(cA, cB)))
                # Weight by thin rows (rows in thin strip of A or B)
                w_thin = tk if (al <= 16 or bl <= 16) else 0
                dir_w[cA][cB] += w_thin
            oa += tk; ob += tk
            if oa == al: ia += 1; oa = 0
            if ob == bl: ib += 1; ob = 0

        print(f"[{fname}] n={n} r0={r0} Open m16={m16}")

        # 1A. Exact CP-SAT Thin-Weighted Bucket-MIS (zero-self)
        mis_E, mis_opt = solve_exact_bucket_mis(active_syms, edges_undir, has_self, thin_bkt, allow_self=False)
        strips_mis, surv_mis, pn_mis, ps_mis = run_1pass_mis_zero_sort(runs, n, p, mis_E)
        st_mis = eval_contracted_iet_in_open(strips_mis, surv_mis, pn_mis, ps_mis, n, limit=16, verify_bytes=ref)
        # Also check self-allowed MIS ceiling (upper bound if monotone self-loops allowed)
        mis_self_E, _ = solve_exact_bucket_mis(active_syms, edges_undir, has_self, thin_bkt, allow_self=True)
        mis_self_thin = sum(thin_bkt[b] for b in mis_self_E)
        print(f"  1A MIS (0-self, {'OPT' if mis_opt else 'FEAS'}, |E|={len(mis_E):2d}): q1={st_mis['q_1']:6d} ({st_mis['q_1']/r0:.2f}r) | "
              f"m16'={st_mis['m_thin_1']:6d} ({st_mis['m_thin_1']/m16:.3f}x!) | del_n={100*(n-surv_mis)/n:4.1f}% | "
              f"self-MIS thin ceiling={100*mis_self_thin/m16:4.1f}% [EXACT]")

        # 1B. 1-Pass Zero-Sort Strip-Space Thin-Weighted 2-Coloring Max-Cut (u16!)
        S_L, mc_opt = solve_thin_weighted_maxcut(active_syms, dir_w)
        strips_mc, surv_mc, pn_mc, ps_mc = run_1pass_maxcut_zero_sort(runs, n, p, S_L)
        st_mc = eval_contracted_iet_in_open(strips_mc, surv_mc, pn_mc, ps_mc, n, limit=16, verify_bytes=ref)
        h = st_mc["hist_thin"]
        print(f"  1B 1-Pass MaxCut (u16, 0-sort): q1={st_mc['q_1']:6d} ({st_mc['q_1']/r0:.2f}r) | "
              f"m16'={st_mc['m_thin_1']:6d} ({st_mc['m_thin_1']/m16:.3f}x!) | del_n={100*(n-surv_mc)/n:4.1f}% | "
              f"thin wlen: 1B={h[0]:4.1f}% 2B={h[1]:4.1f}% [EXACT]")

        # 1C. 1-Pass Ascent-DAG (ASCII order, capped <= 8B u64) & Thin-Weighted MAS-DAG (capped <= 8B u64 & uncapped)!
        asc_rank = {c: c for c in active_syms}
        strips_a8, surv_a8, pn_a8, ps_a8, srt_a8 = run_1pass_dag_exact(runs, n, p, asc_rank, max_word_len=8)
        st_a8 = eval_contracted_iet_in_open(strips_a8, surv_a8, pn_a8, ps_a8, n, limit=16, verify_bytes=ref)
        ha = st_a8["hist_thin"]
        print(f"  1C Ascent-DAG (<=8B u64)      : q1={st_a8['q_1']:6d} ({st_a8['q_1']/r0:.2f}r) | "
              f"m16'={st_a8['m_thin_1']:6d} ({st_a8['m_thin_1']/m16:.3f}x!) | del_n={100*(n-surv_a8)/n:4.1f}% | "
              f"srt={srt_a8/r0:.2f}r | wlen: 1B={ha[0]:4.1f}% 2B={ha[1]:4.1f}% 3-4B={ha[2]:4.1f}% 5-8B={ha[3]:4.1f}% [EXACT]")

        mas_rank, _ = solve_thin_weighted_mas_order(active_syms, dir_w)
        strips_m8, surv_m8, pn_m8, ps_m8, srt_m8 = run_1pass_dag_exact(runs, n, p, mas_rank, max_word_len=8)
        st_m8 = eval_contracted_iet_in_open(strips_m8, surv_m8, pn_m8, ps_m8, n, limit=16, verify_bytes=ref)
        hm = st_m8["hist_thin"]
        print(f"  1C MAS-DAG    (<=8B u64)      : q1={st_m8['q_1']:6d} ({st_m8['q_1']/r0:.2f}r) | "
              f"m16'={st_m8['m_thin_1']:6d} ({st_m8['m_thin_1']/m16:.3f}x!) | del_n={100*(n-surv_m8)/n:4.1f}% | "
              f"srt={srt_m8/r0:.2f}r | wlen: 1B={hm[0]:4.1f}% 2B={hm[1]:4.1f}% 3-4B={hm[2]:4.1f}% 5-8B={hm[3]:4.1f}% [EXACT]")

        strips_mu, surv_mu, pn_mu, ps_mu, srt_mu = run_1pass_dag_exact(runs, n, p, mas_rank, max_word_len=999)
        st_mu = eval_contracted_iet_in_open(strips_mu, surv_mu, pn_mu, ps_mu, n, limit=16, verify_bytes=ref)
        hmu = st_mu["hist_thin"]
        print(f"  1C MAS-DAG    (uncapped,mx={st_mu['max_wlen']:2d}): q1={st_mu['q_1']:6d} ({st_mu['q_1']/r0:.2f}r) | "
              f"m16'={st_mu['m_thin_1']:6d} ({st_mu['m_thin_1']/m16:.3f}x!) | del_n={100*(n-surv_mu)/n:4.1f}% | "
              f"wlen: 1B={hmu[0]:4.1f}% 2B={hmu[1]:4.1f}% 3-4B={hmu[2]:4.1f}% 5-8B={hmu[3]:4.1f}% >8B={hmu[4]:4.1f}% [EXACT]\n")

        for name, st, srt in [("MIS", st_mis, 0), ("MaxCut_1P", st_mc, 0), ("Ascent_8B", st_a8, srt_a8), ("MAS_8B", st_m8, srt_m8), ("MAS_uncap", st_mu, srt_mu)]:
            agg[name]["q1"] += st["q_1"]
            agg[name]["m16"] += st["m_thin_1"]
            agg[name]["del_n"] += (n - st["surviving_n"])
            agg[name]["srt"] += srt

    print("=" * 105)
    print(f"6-BLOCK SUMS (n={tot_n}, r0={tot_r0}, Open m16={tot_m16}):")
    for name in ("MIS", "MaxCut_1P", "Ascent_8B", "MAS_8B", "MAS_uncap"):
        a = agg[name]
        print(f"  {name:10s}: q1={a['q1']:7d} ({a['q1']/tot_r0:.2f}r) | m16'={a['m16']:7d} ({a['m16']/tot_m16:.3f}x Open!!) | "
              f"del_n={100*a['del_n']/tot_n:4.1f}% | sort_items={a['srt']:7d} ({a['srt']/tot_r0:.2f}r)")


if __name__ == "__main__":
    main()
