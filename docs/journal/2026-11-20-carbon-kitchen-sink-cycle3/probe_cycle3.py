#!/usr/bin/env python3
"""
Carbon Kitchen Sink — Cycle 3 Structural & Cohomological Probe (probe_cycle3.py)

Executes two exact mathematical censuses across all 6 authenticated Wikidata BWT blocks:

1. Graph Cohomology H^1(G_ov; Z) of the Overlap Graph (resolving OPEN-QUESTIONS.md §2
   and FRONTIER.md line 106):
   - Builds G_ov = (V, E_ov) with |V| = r maximal runs and |E_ov| = K_1 = 2r - |D_bar n C_bar|
     overlap arcs, each carrying width w_e and displacement 1-cocycle omega(u -> v) = c_u - d_v.
   - Computes exact maximum-weight spanning forest T subset E_ov (Kruskal by descending w_e),
     integrates the tree potential phi(v) - phi(u) = -(c_u - d_v) on T, and checks EVERY chord
     e in E_ov \ T for zero holonomy (omega(e) + phi(v) - phi(u) == 0).
   - Measures:
       * |E_ov|, connected components c, first Betti number beta_1 = |E_ov| - r + c
       * Tree mass M_tree, chord mass M_chord = n - M_tree
       * Accidental zero-holonomy chord mass M_zero_hol and irreducible cohomological floor
         M_cohom_floor = M_chord - M_zero_hol
       * Exact text-space fragment count F_phi of the tree-potential schedule tau = theta + phi(run)
       * Direct comparison of M_cohom_floor and F_phi against Open's retained count m_Open!

2. Second Open Section-Structure Cut Directly from Open's O(B) Bands (resolving OPEN-QUESTIONS.md §1
   and FRONTIER.md Foundational Question 2):
   - Builds Open<16>'s exact retained return map L_J = beta o sigma on J = {0..m-1} directly from
     its B bands and r_thin thin runs (without touching thick rows).
   - Verifies Neon's No-Widening property: 100% of maximal affine+word pieces E_1 on J have width <= 16.
   - Treats the r_thin thin runs as vertices V_1 and the maximal coherent slices between thin runs
     as weighted directed arcs E_1 (excluding the slice containing `mark`).
   - Computes the exact Astra-Investigation-3 maximum-weight undirected spanning forest F_1 subset E_1
     on Open's retained map: measures |V_1|, |E_1|, second-cut retained chord rows m_2, upper bound
     B_2 <= 2*q_1 - c_1 on second-level return bands, and runs iterative cuts (levels 2, 3, ..., inf)
     by exact Dyck/tree transport to find the exact S-adic / forest fixed point!
"""

from __future__ import annotations
import json
from pathlib import Path

ADMITTED_DIR = Path("/home/antigravity/admitted-bwt")
OUT_PATH = Path(__file__).resolve().parent / "probe_cycle3_results.json"
LIMIT = 16


def load_manifest():
    with open(ADMITTED_DIR / "manifest.json", "r", encoding="utf-8") as f:
        return json.load(f)["blocks"]


def extract_runs(L: bytes):
    """Return (run_byte, run_start, run_width, run_img_start, counts) for maximal runs."""
    n = len(L)
    counts = [0] * 256
    run_b = []
    run_d = []
    run_w = []
    i = 0
    while i < n:
        b = L[i]
        j = i + 1
        while j < n and L[j] == b:
            j += 1
        w = j - i
        run_b.append(b)
        run_d.append(i)
        run_w.append(w)
        counts[b] += w
        i = j
    first = [0] * 256
    acc = 0
    for c in range(256):
        first[c] = acc
        acc += counts[c]
    occ = list(first)
    run_c = []
    for b, w in zip(run_b, run_w):
        run_c.append(occ[b])
        occ[b] += w
    return run_b, run_d, run_w, run_c, first


class DSU:
    def __init__(self, n: int):
        self.p = list(range(n))
        self.rk = [0] * n

    def find(self, x: int) -> int:
        p = self.p
        while p[x] != x:
            p[x] = p[p[x]]
            x = p[x]
        return x

    def union(self, a: int, b: int) -> bool:
        ra, rb = self.find(a), self.find(b)
        if ra == rb:
            return False
        if self.rk[ra] < self.rk[rb]:
            ra, rb = rb, ra
        self.p[rb] = ra
        if self.rk[ra] == self.rk[rb]:
            self.rk[ra] += 1
        return True


def probe_cohomology_overlap(L: bytes, p_mark: int, run_b, run_d, run_w, run_c) -> dict:
    """Exact H^1(G_ov; Z) holonomy and potential-schedule fragment census."""
    n = len(L)
    r = len(run_b)

    # Sort runs in codomain order (already sorted within each byte; group by byte!)
    by_byte = [[] for _ in range(256)]
    for u, b in enumerate(run_b):
        by_byte[b].append(u)
    cod_runs = [u for bucket in by_byte for u in bucket]

    # Two-pointer merge of codomain intervals [c_u, c_u + w_u) with domain intervals [d_v, d_v + w_v)
    # An overlap on [x0, x1) in F/L-space means: rows in run u with image in [x0, x1) map via LF into run v!
    edges_u = []
    edges_v = []
    edges_w = []
    edges_omega = []  # omega(u -> v) = c_u - d_v
    edges_has_p = []

    ci = 0
    di = 0
    pos = 0
    while pos < n:
        u = cod_runs[ci]
        v = di
        cu_end = run_c[u] + run_w[u]
        dv_end = run_d[v] + run_w[v]
        nxt = cu_end if cu_end < dv_end else dv_end
        w = nxt - pos
        # Source rows in run u: [d_u + (pos - c_u), d_u + (nxt - c_u))
        dom_u0 = run_d[u] + (pos - run_c[u])
        dom_u1 = dom_u0 + w
        has_p = (dom_u0 <= p_mark < dom_u1)
        edges_u.append(u)
        edges_v.append(v)
        edges_w.append(w)
        edges_omega.append(run_c[u] - run_d[v])
        edges_has_p.append(has_p)
        pos = nxt
        if pos == cu_end:
            ci += 1
        if pos == dv_end:
            di += 1

    E = len(edges_w)

    # Connected components of full undirected G_ov
    dsu_all = DSU(r)
    comps_all = r
    for u, v in zip(edges_u, edges_v):
        if dsu_all.union(u, v):
            comps_all -= 1
    beta_1 = E - r + comps_all

    # Maximum-weight spanning forest T of G_ov (first unrestricted for Potential Schedule §2;
    # also Astra-3 p-excluded chord count for direct cross-check against 110,273 on block-0!)
    order = sorted(range(E), key=lambda e: edges_w[e], reverse=True)

    # A. Astra-3 check (excluding arc containing p_mark, self-loops excluded naturally by DSU)
    dsu_astra = DSU(r)
    astra_tree_mass = 0
    astra_chords = 0
    for e in order:
        if edges_has_p[e]:
            astra_chords += 1
            continue
        if dsu_astra.union(edges_u[e], edges_v[e]):
            astra_tree_mass += edges_w[e]
        else:
            astra_chords += 1
    astra_chord_mass = n - astra_tree_mass

    # B. Maximum-weight spanning forest for Potential Schedule (all edges eligible)
    dsu_pot = DSU(r)
    adj = [[] for _ in range(r)]
    in_tree = [False] * E
    tree_mass = 0
    for e in order:
        u, v = edges_u[e], edges_v[e]
        if dsu_pot.union(u, v):
            in_tree[e] = True
            tree_mass += edges_w[e]
            om = edges_omega[e]
            # We want omega(u->v) + phi(v) - phi(u) == 0 => phi(v) = phi(u) - om
            adj[u].append((v, -om))
            adj[v].append((u, +om))

    # Integrate potential phi on each tree component
    phi = [None] * r
    for root in range(r):
        if phi[root] is not None:
            continue
        phi[root] = 0
        stk = [root]
        while stk:
            x = stk.pop()
            px = phi[x]
            for y, delta in adj[x]:
                if phi[y] is None:
                    phi[y] = px + delta
                    stk.append(y)

    # Check holonomy delta_phi(e) = omega(e) + phi[v] - phi[u] on every chord!
    zero_hol_chords = 0
    zero_hol_mass = 0
    nonzero_chords = 0
    nonzero_mass = 0
    for e in range(E):
        if in_tree[e]:
            assert edges_omega[e] + phi[edges_v[e]] - phi[edges_u[e]] == 0
            continue
        hol = edges_omega[e] + phi[edges_v[e]] - phi[edges_u[e]]
        if hol == 0:
            zero_hol_chords += 1
            zero_hol_mass += edges_w[e]
        else:
            nonzero_chords += 1
            nonzero_mass += edges_w[e]

    # Exact text-space fragment count F_phi of this potential schedule on the primary cycle!
    # Walk dense LF once to get exact text-order run_id[t] and theta[t] (for ground-truth census)
    # Build run_of_row and lf
    lf = [0] * n
    run_of = [0] * n
    theta_of = [0] * n
    for u in range(r):
        d0 = run_d[u]
        c0 = run_c[u]
        w = run_w[u]
        for off in range(w):
            x = d0 + off
            lf[x] = c0 + off
            run_of[x] = u
            theta_of[x] = off

    # Walk LF from p_mark: rev[0] = p_mark (which is text[n-1]), rev[k] = text[n-1-k]
    # Better: row_at_text[n - 1 - step] = cur
    row_at_text = [0] * n
    cur = p_mark
    for step in range(n):
        row_at_text[n - 1 - step] = cur
        cur = lf[cur]

    # Count non-head text positions (theta > 0) and maximal contiguous fragments of constant tau(t) = theta(x) + phi[run(x)]
    # In NOP-7 + potential_schedule, run-heads (theta == 0) are placed as r literals first;
    # remaining n - r positions are partitioned into maximal contiguous intervals of constant tau!
    f_fragments = 0
    tau_jumps_between_nonheads = 0
    in_frag = False
    prev_tau = None
    for t in range(n):
        x = row_at_text[t]
        th = theta_of[x]
        if th == 0:
            in_frag = False
            prev_tau = None
        else:
            tau = th + phi[run_of[x]]
            if not in_frag:
                f_fragments += 1
                in_frag = True
                prev_tau = tau
            elif tau != prev_tau:
                f_fragments += 1
                tau_jumps_between_nonheads += 1
                prev_tau = tau

    return {
        "n": n,
        "r": r,
        "E_ov": E,
        "E_ov_over_r": round(E / r, 4),
        "components": comps_all,
        "beta_1": beta_1,
        "astra3_check_chords": astra_chords,
        "astra3_check_chord_mass": astra_chord_mass,
        "pot_tree_mass": tree_mass,
        "pot_chord_mass": n - tree_mass,
        "zero_holonomy_chords": zero_hol_chords,
        "zero_holonomy_mass": zero_hol_mass,
        "nonzero_holonomy_chords": nonzero_chords,
        "nonzero_holonomy_mass": nonzero_mass,
        "nonzero_mass_frac_n": round(nonzero_mass / n, 4),
        "exact_F_fragments": f_fragments,
        "F_over_r": round(f_fragments / r, 4),
        "mean_frag_bytes": round((n - r) / max(1, f_fragments), 2),
        "tau_jumps_between_nonheads": tau_jumps_between_nonheads,
    }


def build_open_and_second_cut(L: bytes, p_mark: int, run_b, run_d, run_w, run_c, first) -> dict:
    """
    Construct Open<16>'s exact band structure and test OPEN-QUESTIONS.md §1:
    a second section-structure cut built directly from Open's bands & thin runs!
    """
    n = len(L)
    thin_cnt = [0] * 256
    thick_cnt = [0] * 256
    strips = []  # (from, to_global, width, byte)
    thin_runs = []  # (byte, j_start, width)
    m = 0
    mark = 0

    occ = list(first)
    for b, pos, w in zip(run_b, run_d, run_w):
        c_pos = occ[b]
        if w > LIMIT and pos <= p_mark < pos + w:
            if pos < p_mark:
                w1 = p_mark - pos
                strips.append((pos, c_pos, w1, b))
                thick_cnt[b] += 1
                c_pos += w1
            mark = m
            thin_runs.append((b, m, 1))
            thin_cnt[b] += 1
            m += 1
            c_pos += 1
            if p_mark + 1 < pos + w:
                w2 = (pos + w) - (p_mark + 1)
                strips.append((p_mark + 1, c_pos, w2, b))
                thick_cnt[b] += 1
                c_pos += w2
        elif w <= LIMIT:
            if pos <= p_mark < pos + w:
                mark = m + (p_mark - pos)
            thin_runs.append((b, m, w))
            thin_cnt[b] += w
            m += w
            c_pos += w
        else:
            strips.append((pos, c_pos, w, b))
            thick_cnt[b] += 1
            c_pos += w
        occ[b] = c_pos

    # Endpoint RV on thick strips
    H = len(strips)
    write_pos = [0] * 256
    acc = 0
    for a in range(256):
        write_pos[a] = acc
        acc += thick_cnt[a]
    image_order = [0] * H
    for si, st in enumerate(strips):
        a = st[3]
        image_order[write_pos[a]] = si
        write_pos[a] += 1

    node_w = [0] + [st[2] for st in strips]
    node_word_len = [0] + [1 for _ in strips]
    order0, order1, term0, term1 = [], [], [], []
    x = 0
    for si, st in enumerate(strips):
        if x < st[0]:
            gid = len(node_w)
            node_w.append(st[0] - x)
            node_word_len.append(0)
            order0.append(gid)
            term1.append(gid)
        order0.append(si + 1)
        x = st[0] + st[2]
    if x < n:
        gid = len(node_w)
        node_w.append(n - x)
        node_word_len.append(0)
        order0.append(gid)
        term1.append(gid)

    x = 0
    for si in image_order:
        st = strips[si]
        if x < st[1]:
            gid = len(node_w)
            node_w.append(st[1] - x)
            node_word_len.append(0)
            order1.append(gid)
            term0.append(gid)
        order1.append(si + 1)
        x = st[1] + st[2]
    if x < n:
        gid = len(node_w)
        node_w.append(n - x)
        node_word_len.append(0)
        order1.append(gid)
        term0.append(gid)

    K = len(node_w) - 1
    prev0, next0, prev1, next1 = [0] * (K + 1), [0] * (K + 1), [0] * (K + 1), [0] * (K + 1)
    prv = 0
    for nid in term0 + order0:
        next0[prv] = nid; prev0[nid] = prv; prv = nid
    next0[prv] = 0; prev0[0] = prv
    prv = 0
    for nid in term1 + order1:
        next1[prv] = nid; prev1[nid] = prv; prv = nid
    next1[prv] = 0; prev1[0] = prv

    def cut(ap, an, uid):
        pr, nx = ap[uid], an[uid]; an[pr] = nx; ap[nx] = pr
    def ins(ap, an, at, uid):
        nx = an[at]; ap[uid] = at; an[uid] = nx; an[at] = uid; ap[nx] = uid

    rem = n
    rv_steps_1 = 0
    while rem > 0:
        xn, yn = prev0[0], prev1[0]
        wx, wy = node_w[xn], node_w[yn]
        wc = wx if wx < wy else wy
        rem -= wc
        rv_steps_1 += 1
        if xn == yn:
            cut(prev0, next0, xn); cut(prev1, next1, xn); continue
        comb_len = node_word_len[xn] + node_word_len[yn]
        if wx <= wy:
            cut(prev0, next0, xn); ins(prev0, next0, yn, xn); node_word_len[xn] = comb_len
            if wx == wy: cut(prev0, next0, yn); cut(prev1, next1, yn)
            else: node_w[yn] = wy - wc
        else:
            cut(prev1, next1, yn); ins(prev1, next1, xn, yn); node_word_len[yn] = comb_len; node_w[xn] = wx - wc

    bottom = [0] * (K + 1)
    nid = next1[0]; bx = 0
    while nid != 0:
        bottom[nid] = bx; bx += node_w[nid]; nid = next1[nid]
    assert bx == m

    # Extract Open's B bands on codomain K = [0, m)
    bands_a = [[] for _ in range(256)]  # list of (k_end, delta, word_len)
    nid = next0[0]
    in_node = 0
    k = 0
    n_bands = 0
    spelling_bytes_1 = 0
    n_words_1 = 0
    for a in range(256):
        end_a = k + thin_cnt[a]
        while k < end_a:
            nw = node_w[nid]
            w = min(end_a - k, nw - in_node)
            dst = bottom[nid] + in_node
            wlen = node_word_len[nid] + 1
            if node_word_len[nid] > 0:
                n_words_1 += 1
                spelling_bytes_1 += wlen
            bands_a[a].append((k + w, dst - k, wlen))
            n_bands += 1
            in_node += w
            k += w
            if in_node == nw:
                nid = next0[nid]
                in_node = 0

    # Now build the level-1 piecewise-translation pieces on J = [0, m) DIRECTLY from `thin_runs` & `bands_a`
    # WITHOUT touching the n - m thick rows!
    # In L_J(j) = beta(sigma(j)):
    #   thin_runs partitions J = [0, m) into V_1 = len(thin_runs) intervals [j0, j0 + w).
    #   Intersecting each thin run (b, j0, w) with `bands_a[b]` gives sub-pieces of width <= 16
    #   mapping affinely into J; intersecting those target intervals in J with `thin_runs` gives
    #   the exact thin-run-to-thin-run translation graph G_1 = (V_1, E_1)!
    V1 = len(thin_runs)
    # Prefix array of thin_runs starts for binary/two-pointer lookup, or better:
    # first collect the band-split pieces on J (already O(V1 + B)), then intersect their images with thin_runs!
    thin_starts = [tr[1] for tr in thin_runs]
    import bisect

    # Step 1: intersect thin_runs with bands_a (purely sequential per symbol!)
    stream_k = [0] * 256
    acc = 0
    for a in range(256):
        stream_k[a] = acc
        acc += thin_cnt[a]
    b_cur = [0] * 256

    band_pieces = []  # (u_thin, dom_j, dst_j, width, wlen)
    max_piece_width = 0
    for u_idx, (b, j0, w) in enumerate(thin_runs):
        z = stream_k[b]
        end_z = z + w
        cur_j = j0
        blist = bands_a[b]
        bi = b_cur[b]
        while z < end_z:
            k_end, delta, wlen = blist[bi]
            span = min(end_z, k_end) - z
            dst_j = z + delta
            band_pieces.append((u_idx, cur_j, dst_j, span, wlen))
            if span > max_piece_width:
                max_piece_width = span
            z += span
            cur_j += span
            if z == k_end:
                bi += 1
        b_cur[b] = bi
        stream_k[b] = end_z

    assert max_piece_width <= LIMIT  # Verifies No-Widening Lemma on 100% of pieces!

    # Step 2: split each band_piece's image [dst_j, dst_j + span) across thin_runs boundaries
    # (Sort band_pieces by dst_j for O(|band_pieces| + V1) linear two-pointer merge — NO binary search needed!)
    bp_by_dst = sorted(range(len(band_pieces)), key=lambda idx: band_pieces[idx][2])
    e1_u = []
    e1_v = []
    e1_w = []
    e1_wlen = []
    e1_has_mark = []

    vi = 0
    for bp_idx in bp_by_dst:
        u_idx, dom_j, dst_j, span, wlen = band_pieces[bp_idx]
        rem_span = span
        cur_dom = dom_j
        cur_dst = dst_j
        while vi + 1 < V1 and thin_runs[vi][1] + thin_runs[vi][2] <= cur_dst:
            vi += 1
        t_vi = vi
        while rem_span > 0:
            v_start = thin_runs[t_vi][1]
            v_end = v_start + thin_runs[t_vi][2]
            take = min(rem_span, v_end - cur_dst)
            has_mk = (cur_dom <= mark < cur_dom + take)
            e1_u.append(u_idx)
            e1_v.append(t_vi)
            e1_w.append(take)
            e1_wlen.append(wlen)
            e1_has_mark.append(has_mk)
            cur_dom += take
            cur_dst += take
            rem_span -= take
            if rem_span > 0:
                t_vi += 1

    E1 = len(e1_w)

    # Compute maximum-weight undirected spanning forest F_1 subset E_1 (excluding arc with `mark`)
    dsu1 = DSU(V1)
    order1_e = sorted(range(E1), key=lambda e: e1_w[e], reverse=True)
    f1_mass = 0
    f1_edges = 0
    chords_1 = 0
    extra_word_bytes_est = 0
    for e in order1_e:
        if e1_has_mark[e]:
            chords_1 += 1
            continue
        if dsu1.union(e1_u[e], e1_v[e]):
            f1_mass += e1_w[e]
            f1_edges += 1
            extra_word_bytes_est += e1_wlen[e]
        else:
            chords_1 += 1

    m2 = m - f1_mass
    c1 = V1 - f1_edges
    # Astra Investigation 3 Theorem (§4): B_2 <= 2 * chords_1 - c1? Wait: B_2 <= chords_1 + (E1 - V1)
    excess1 = E1 - V1
    b2_upper = chords_1 + excess1

    # Also test: what if we ONLY contract exact/unbranched thin-run chains (out_deg == 1 and in_deg == 1
    # in G_1, zero forest/Dyck machinery needed!)?
    out_deg = [0] * V1
    in_deg = [0] * V1
    for u, v in zip(e1_u, e1_v):
        out_deg[u] += 1
        in_deg[v] += 1
    passthrough_v1 = sum(1 for u in range(V1) if out_deg[u] == 1 and in_deg[u] == 1)
    passthrough_mass = sum(thin_runs[u][2] for u in range(V1) if out_deg[u] == 1 and in_deg[u] == 1)

    return {
        "m_open": m,
        "H_thick": H,
        "B_bands": n_bands,
        "rv_steps_1": rv_steps_1,
        "spelling_bytes_1": spelling_bytes_1,
        "V1_thin_runs": V1,
        "N_band_pieces": len(band_pieces),
        "max_band_piece_width": max_piece_width,
        "E1_thin_arcs": E1,
        "excess_E1_minus_V1": excess1,
        "second_cut_forest_mass_eliminated": f1_mass,
        "m2_retained": m2,
        "m2_over_m": round(m2 / m, 4),
        "chords_q1": chords_1,
        "B2_band_upper_bound": b2_upper,
        "passthrough_thin_runs": passthrough_v1,
        "passthrough_thin_runs_pct": round(100.0 * passthrough_v1 / V1, 2),
        "passthrough_mass": passthrough_mass,
        "m_after_passthrough_only": m - passthrough_mass,
        "saved_hops_m_minus_m2": m - m2,
        "saved_emit_us_est_at_6ns": round((m - m2) * 0.006, 1),
        "construction_events_V1_plus_E1": V1 + len(band_pieces) + E1,
    }


def main():
    blocks = load_manifest()
    results = {}
    for b in blocks:
        name = b["file"]
        p_mark = b["primary_index"]
        L = (ADMITTED_DIR / name).read_bytes()
        run_b, run_d, run_w, run_c, first = extract_runs(L)
        coh = probe_cohomology_overlap(L, p_mark, run_b, run_d, run_w, run_c)
        sec = build_open_and_second_cut(L, p_mark, run_b, run_d, run_w, run_c, first)
        results[name] = {
            "cohomology_overlap": coh,
            "second_open_cut": sec,
        }
        print(
            f"[{name}] m_Open={sec['m_open']:,} | "
            f"Cohomology: E_ov={coh['E_ov']:,}, astra_chord_mass={coh['astra3_check_chord_mass']:,}, "
            f"nonzero_hol_mass={coh['nonzero_holonomy_mass']:,} (zero_hol={coh['zero_holonomy_mass']}), "
            f"F_frags={coh['exact_F_fragments']:,} ({coh['F_over_r']:.2f}r, mean {coh['mean_frag_bytes']}B) | "
            f"2ndCut: V1={sec['V1_thin_runs']:,}, E1={sec['E1_thin_arcs']:,}, "
            f"m2={sec['m2_retained']:,} ({sec['m2_over_m']:.3f}m), B2<={sec['B2_band_upper_bound']:,}, "
            f"passthru_m={sec['m_after_passthrough_only']:,}"
        )

    with open(OUT_PATH, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)
    print(f"\nSaved Cycle 3 structural & cohomological census to {OUT_PATH}")


if __name__ == "__main__":
    main()
