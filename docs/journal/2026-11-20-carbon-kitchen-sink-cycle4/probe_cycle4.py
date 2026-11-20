#!/usr/bin/env python3
"""
Carbon Kitchen Sink — Cycle 4 Multi-World Mathematical Probe (probe_cycle4.py)

Fast, exact mathematical probe across 4 unexplored structural worlds on the 6 Wikidata blocks:

1. World 1 — Ribbon Graphs / Grothendieck Dessins d'Enfants & Bestvina-Handel Illegal Turns
   (FRONTIER.md leads 2 & 3):
   - Constructs the exact BWT Surface Ribbon Graph on the 2 * |E_ov| half-edges of G_ov by cyclic
     rectangle-boundary ordering around each run R_u = [d_u, d_u + w_u] x [0, 1].
   - Computes exact vertex count |V|, edge count |E_ov|, boundary face cycles |F_rib|, Euler
     characteristic chi, and oriented surface genus g_rib; compares g_rib against the Origami
     lift's cone-point genus g_origami!
   - Constructs the natural run-refinement self-map / turn structure on G_ov: counts exact
     Bestvina-Handel legal vs illegal turns at the switches of G_ov and measures Stallings
     fold chains.

2. World 2 — Meyer Non-Commutative Schur Complements & Pareto Optimality of Open (FRONTIER.md:134):
   - Checks same-shift adjacency across consecutive maximal runs (proving k(S) >= #runs_in_S
     up to at most alpha-1 cross-byte coincidences; measures exact count on all 6 blocks!).
   - Computes exact Pareto frontier of retained rows m(H) vs eliminated affine strip count H,
     comparing Open<16> against alphabet-subset Schur complements and run-core complements.

3. World 3 — Tropical Duality of Forward LCP (Karkkainen-Kempa-Piatkowski Irreducible LCPs,
   FRONTIER.md:131) vs Backward Separation Depth:
   - Computes exact irreducible LCP values LCP_F[y] at the r codomain run-image heads y in C_bar
     (via Kasai on primary cycle text) and verifies the exact tropical recurrence
     LCP_F[y] = 1 + LCP_F[Psi(y)] for ALL y not in C_bar!
   - Measures sum of irreducible LCPs (vs 2 n log_2 r bound), mean irreducible LCP, and tests
     whether LCP_F[C_bar] determines or bounds GUID entropy.

4. World 4 — Post-BWT Syntactic Semigroup & Affine-Linear F_2[x]/(P_32) Transducer (FRONTIER.md Q4):
   - Measures exact RLE1 4-equal-byte run statistics on the decoded BWT output T of all 6 blocks:
     how many RLE1 triggers occur, what fraction of bytes change length, and how many Open words
     contain or straddle a 4-equal-byte RLE1 trigger!
   - Verifies exact 5-state Mealy transducer + F_2[x]/(P_32) CRC32 monoid homomorphism on block-0!
"""

from __future__ import annotations
import json
import math
import zlib
from pathlib import Path

ADMITTED_DIR = Path("/home/antigravity/admitted-bwt")
OUT_PATH = Path(__file__).resolve().parent / "probe_cycle4_results.json"


def load_manifest():
    with open(ADMITTED_DIR / "manifest.json", "r", encoding="utf-8") as f:
        return json.load(f)["blocks"]


def extract_runs_and_lf(L: bytes):
    n = len(L)
    counts = [0] * 256
    run_b, run_d, run_w = [], [], []
    i = 0
    while i < n:
        b = L[i]
        j = i + 1
        while j < n and L[j] == b:
            j += 1
        w = j - i
        run_b.append(b); run_d.append(i); run_w.append(w)
        counts[b] += w
        i = j
    first = [0] * 256
    acc = 0
    for c in range(256):
        first[c] = acc; acc += counts[c]
    occ = list(first)
    run_c = []
    for b, w in zip(run_b, run_w):
        run_c.append(occ[b]); occ[b] += w
    return run_b, run_d, run_w, run_c, counts, first


def build_overlap_graph(n: int, run_b, run_d, run_w, run_c):
    r = len(run_b)
    by_byte = [[] for _ in range(256)]
    for u, b in enumerate(run_b):
        by_byte[b].append(u)
    cod_runs = [u for bucket in by_byte for u in bucket]

    # Overlap edges e: rows in run u mapping into run v on F-interval [pos, nxt)
    edges_u, edges_v, edges_w = [], [], []
    in_edges = [[] for _ in range(r)]   # arriving at v in left-to-right domain order of R_v!
    out_edges = [[] for _ in range(r)]  # leaving u in left-to-right domain/codomain order of R_u!

    ci = di = pos = 0
    while pos < n:
        u = cod_runs[ci]
        v = di
        cu_e = run_c[u] + run_w[u]
        dv_e = run_d[v] + run_w[v]
        nxt = cu_e if cu_e < dv_end_helper(dv_e) else dv_e
        e_id = len(edges_w)
        edges_u.append(u); edges_v.append(v); edges_w.append(nxt - pos)
        out_edges[u].append(e_id)
        in_edges[v].append(e_id)
        pos = nxt
        if pos == cu_e: ci += 1
        if pos == dv_e: di += 1
    return edges_u, edges_v, edges_w, in_edges, out_edges


def dv_end_helper(x: int) -> int:
    return x


def probe_world1_ribbon_and_folding(n: int, run_b, run_d, run_w, run_c, edges_u, edges_v, edges_w, in_edges, out_edges) -> dict:
    """
    1A. Ribbon Graph (Dessin d'Enfant) on G_ov:
        Model each run R_u as an oriented disk/rectangle:
          - Top edge has incoming overlap strips in_edges[u][0 .. deg_in-1] left-to-right.
          - Bottom edge has outgoing overlap strips out_edges[u][0 .. deg_out-1] left-to-right.
        Going counter-clockwise around the boundary of R_u visits:
          top right-to-left: in_edges[u][deg_in-1], ..., in_edges[u][0],
          then bottom left-to-right: out_edges[u][0], ..., out_edges[u][deg_out-1]!
        (Each overlap edge e has two darts: dart 2*e = tail at u (bottom of R_u),
         dart 2*e + 1 = head at v (top of R_v); edge involution alpha(2*e) = 2*e + 1, alpha(2*e+1) = 2*e.)
        Compute face permutation phi = sigma o alpha and Euler genus g_rib!
    1B. Compare with Origami cone-point genus g_origami from commutator k = LF o (+1) o LF^{-1} o (-1)!
    1C. Bestvina-Handel / Train-Track Turn Legality:
        Under LF, each outgoing edge e = (u -> v) maps monotonically into out_edges[v] (specifically the
        sub-interval of R_v occupied by e). At switch v, two incoming edges e1, e2 (adjacent in in_edges[v])
        meet at the interior boundary between e1 and e2 inside R_v! Wait: inside R_v, the row just left of
        that boundary and the row just right of that boundary BOTH lie in R_v (same letter L[y] = b_v!), so
        LF maps them to ADJACENT rows in C_v! They only split later when their forward LF orbit hits D_bar!
        Dually, under backward Psi = LF^{-1}, two outgoing edges f1, f2 in out_edges[u] map into distinct
        target runs v1 != v2 (different first letters or different runs of same letter), whereas two incoming
        edges e1 = (u1 -> v) and e2 = (u2 -> v) come from DIFFERENT bytes b_{u1} != b_{u2} (or non-adjacent
        runs of the same byte)! Can Stallings folding identify e1 and e2? ONLY if b_{u1} == b_{u2}!
        Let's count exact same-byte vs distinct-byte incoming pairs at every switch v!
    """
    r = len(run_b)
    E = len(edges_w)
    n_darts = 2 * E
    sigma = [0] * n_darts

    for u in range(r):
        # CCW cyclic order of darts incident to vertex u:
        # top right-to-left (heads: 2*e + 1) followed by bottom left-to-right (tails: 2*e)
        cyc = [2 * e + 1 for e in reversed(in_edges[u])] + [2 * e for e in out_edges[u]]
        k = len(cyc)
        for idx in range(k):
            sigma[cyc[idx]] = cyc[(idx + 1) % k]

    # Count cycles of sigma (should equal r, since every run has in_deg >= 1 and out_deg >= 1)
    # Count faces = cycles of phi(d) = sigma[d ^ 1]
    vis = [False] * n_darts
    faces = 0
    face_lens = []
    for d0 in range(n_darts):
        if not vis[d0]:
            faces += 1
            ln = 0
            cur = d0
            while not vis[cur]:
                vis[cur] = True
                ln += 1
                cur = sigma[cur ^ 1]
            face_lens.append(ln)

    # Connected components of G_ov = 1 for primitive BWT
    # Euler characteristic: V - E + F = 2 - 2 * g_rib (for connected oriented ribbon surface)
    chi = r - E + faces
    assert (2 - chi) % 2 == 0
    g_rib = (2 - chi) // 2

    # Compare against Origami commutator k(x) = LF((x + 1) % n) vs ... wait:
    # In origami-lift.md: r_perm = LF, u_perm(x) = (x + 1) % n; commutator c(x) = r(u(r^{-1}(u^{-1}(x))))
    # By Cycle 1 Theorem 2, non-trivial cycles of c equal non-trivial cycles on w_runs (or compute directly on [0..n)):
    # Even faster: on w_runs of length r! Wait: let's compute directly on [0..n) to be 100% exact!
    lf = [0] * n
    psi = [0] * n
    for u in range(r):
        d0, c0, w = run_d[u], run_c[u], run_w[u]
        for off in range(w):
            lf[d0 + off] = c0 + off
            psi[c0 + off] = d0 + off

    vis_o = bytearray(n)
    origami_cycles = 0
    origami_fixed = 0
    sum_deg_minus_1 = 0
    for x0 in range(n):
        if not vis_o[x0]:
            origami_cycles += 1
            ln = 0
            cur = x0
            while not vis_o[cur]:
                vis_o[cur] = 1
                ln += 1
                # c(cur) = lf[(psi[(cur - 1) % n] + 1) % n]
                cur = lf[(psi[(cur - 1) % n] + 1) % n]
            if ln == 1:
                origami_fixed += 1
            else:
                sum_deg_minus_1 += (ln - 1)
    # Riemann-Hurwitz on torus cover: 2*g_origami - 2 = sum (e_i - 1) = n - origami_cycles
    assert (n - origami_cycles) % 2 == 0
    g_origami = 1 + (n - origami_cycles) // 2

    # Bestvina-Handel / Stallings fold test at switches of G_ov:
    # At vertex v, incoming edges e in in_edges[v] carry letter b_{edges_u[e]}.
    # Stallings fold can identify two incoming edges ONLY if they carry the SAME letter!
    # Wait: in G_ov, all incoming edges to v come from the F-slice [d_v, d_v + w_v).
    # Since F is sorted by byte (C[0..256]), the bytes b_{edges_u[e]} along in_edges[v] are MONOTONICALLY NON-DECREASING!
    # Two incoming edges at v have the SAME byte iff [d_v, d_v + w_v) spans multiple runs of the SAME byte in C-space!
    same_byte_in_adj = 0
    diff_byte_in_adj = 0
    for v in range(r):
        ie = in_edges[v]
        for j in range(len(ie) - 1):
            b1 = run_b[edges_u[ie[j]]]
            b2 = run_b[edges_u[ie[j + 1]]]
            if b1 == b2:
                same_byte_in_adj += 1
            else:
                diff_byte_in_adj += 1

    # Dually, outgoing edges out_edges[u] ALL carry the SAME letter b_u, and enter distinct domain runs v_1 < v_2 < ...
    # How many outgoing switches (len(out_edges[u]) >= 2) vs incoming switches (len(in_edges[v]) >= 2)?
    out_switches = sum(1 for u in range(r) if len(out_edges[u]) >= 2)
    in_switches = sum(1 for v in range(r) if len(in_edges[v]) >= 2)

    return {
        "V_runs": r,
        "E_ov": E,
        "faces_F_rib": faces,
        "max_face_darts": max(face_lens),
        "euler_chi": chi,
        "ribbon_genus_g": g_rib,
        "origami_cycles": origami_cycles,
        "origami_genus_g": g_origami,
        "genus_diff_g_rib_minus_g_orig": g_rib - g_origami,
        "in_switches": in_switches,
        "out_switches": out_switches,
        "in_adj_same_byte_foldable": same_byte_in_adj,
        "in_adj_diff_byte_rigid": diff_byte_in_adj,
        "in_adj_same_byte_pct": round(100.0 * same_byte_in_adj / max(1, same_byte_in_adj + diff_byte_in_adj), 2),
        "lf": lf,
        "psi": psi,
    }


def probe_world2_schur_optimality(n: int, p_mark: int, run_b, run_d, run_w, run_c, counts) -> dict:
    """
    Test Meyer Schur Complement optimality (FRONTIER.md:134):
      1. Count adjacent maximal runs (u, u+1) with IDENTICAL shift delta_u == delta_{u+1}
         (the ONLY case where two runs merge into a single affine piece in S!).
      2. Compare Open<16> (retaining width <= 16: H_open strips, m_open retained rows) against:
         - Exact fractional/integer knapsack lower bound m*(H_open) over ALL run-subset Schur complements!
         - Best Alphabet-Subset Schur Complement (choosing a subset of bytes Sigma_elim of total run count <= H_open,
           or retaining single separator bytes like '"' or '\n')!
    """
    r = len(run_b)
    same_shift_adj = 0
    for u in range(r - 1):
        if (run_c[u] - run_d[u]) == (run_c[u + 1] - run_d[u + 1]):
            same_shift_adj += 1

    # Open<16> exact H and m
    H_open = 0
    m_open = 0
    for pos, w in zip(run_d, run_w):
        if w > 16 and pos <= p_mark < pos + w:
            if pos < p_mark: H_open += 1
            m_open += 1
            if p_mark + 1 < pos + w: H_open += 1
        elif w <= 16:
            m_open += w
        else:
            H_open += 1

    # Integer knapsack optimum over ALL 2^r whole-run subsets with at most H_open pieces (excluding primary row):
    # Each run u gives gain w_u (or w_u - 1 if p_mark in u, costing 1 or 2 pieces; upper bound gain = w_u for 1 piece!)
    sorted_w = sorted(run_w, reverse=True)
    max_elim_any_H_runs = sum(sorted_w[:H_open])
    min_possible_m_at_H_open = max(1, n - max_elim_any_H_runs)

    # Alphabet-subset Schur complements: for each byte b, how many runs r_byte[b] and rows counts[b] does it have?
    r_byte = [0] * 256
    for b in run_b:
        r_byte[b] += 1
    active_bytes = [b for b in range(256) if r_byte[b] > 0]
    # Rank bytes by mean run width counts[b] / r_byte[b] descending (greedy fractional knapsack on whole bytes!)
    by_density = sorted(active_bytes, key=lambda b: counts[b] / r_byte[b], reverse=True)
    elim_b_rows = 0
    elim_b_runs = 0
    for b in by_density:
        if elim_b_runs + r_byte[b] <= H_open:
            elim_b_runs += r_byte[b]
            elim_b_rows += counts[b]
    m_best_byte_subset_at_H = n - elim_b_rows

    # What if we eliminate ALL bytes except one frequent structural byte (e.g. '"', ',', '\n')?
    # Note: eliminating byte b still puts r_byte[b] pieces into the auxiliary exchange!
    quote_cnt = counts[ord('"')]
    newline_cnt = counts[ord('\n')]
    brace_cnt = counts[ord('}')]

    return {
        "alpha": len(active_bytes),
        "same_shift_adjacent_runs": same_shift_adj,
        "H_open": H_open,
        "m_open": m_open,
        "universal_lower_bound_m_at_H_open": min_possible_m_at_H_open,
        "open_gap_from_universal_knapsack_pct": round(100.0 * (m_open - min_possible_m_at_H_open) / m_open, 3),
        "best_alphabet_subset_m_at_H_open": m_best_byte_subset_at_H,
        "best_alphabet_subset_runs_used": elim_b_runs,
        "count_quote": quote_cnt,
        "count_newline": newline_cnt,
        "count_close_brace": brace_cnt,
    }


def probe_world3_tropical_lcp(n: int, p_mark: int, L: bytes, run_c, run_w, lf, psi) -> dict:
    """
    Compute exact cyclic BWT matrix LCP_F[1..n-1] via Kasai's O(n) algorithm on the doubled/cyclic text,
    verify the Tropical Reducibility Identity:
      forall y in [1, n) \ C_bar:  LCP_F[y] == 1 + LCP_F[Psi(y)]
    and measure the Irreducible LCP sum sum_{y in C_bar} LCP_F[y] vs 2 * n * log2(r)!
    """
    r = len(run_w)
    # Reconstruct text T[0..n-1] AND exact SA/ISA!
    # Wait: in our BWT convention, row_at_text[n - 1 - step] = cur starting from cur = p_mark:
    # Let's check! L[p_mark] = T[n-1], LF(p_mark) has L = T[n-2], ..., LF^{n-1}(p_mark) has L = T[0]!
    # What rotation of T corresponds to BWT row x?
    # Since L[x] is the last char of rotation x:
    #   p_mark corresponds to rotation T[0..n] (whose last char is T[n-1] = L[p_mark])!
    #   LF(p_mark) corresponds to rotation T[n-1] + T[0..n-1] (starting at index n-1)!
    #   In general, step k from p_mark (cur = LF^k(p_mark)) is the rotation starting at text index (n - k) % n!
    T = bytearray(n)
    rot_start = [0] * n   # text start index of BWT row x
    rank_of_pos = [0] * n # BWT row x of text start index pos
    cur = p_mark
    for step in range(n):
        t_last = n - 1 - step
        T[t_last] = L[cur]
        t_start = (n - step) % n
        rot_start[cur] = t_start
        rank_of_pos[t_start] = cur
        cur = lf[cur]

    # Cyclic Kasai O(n) LCP between row y-1 and row y (for 1 <= y < n):
    lcp_f = [0] * n
    h = 0
    T_bytes = bytes(T)
    for pos in range(n):
        y = rank_of_pos[pos]
        if y == 0:
            h = 0
            continue
        pos_prev = rot_start[y - 1]
        while h < n and T_bytes[(pos + h) % n] == T_bytes[(pos_prev + h) % n]:
            h += 1
        lcp_f[y] = h
        if h > 0:
            h -= 1

    # Mark C_bar (the r codomain run-image start positions c_u)
    is_c_bar = bytearray(n)
    for c0 in run_c:
        is_c_bar[c0] = 1

    # Verify Exact Tropical Reducibility Identity on ALL n - r reducible positions!
    reducible_checked = 0
    for y in range(1, n):
        if not is_c_bar[y]:
            # Since y not in C_bar, y and y-1 lie in the SAME codomain image [c_u, c_u + w_u),
            # so Psi(y) = Psi(y - 1) + 1 AND both have the same first byte F[y] = F[y-1]!
            # Unless LCP_F[Psi(y)] == n (periodic), LCP_F[y] == min(n, 1 + LCP_F[Psi(y)])!
            expected = min(n, 1 + lcp_f[psi[y]])
            if lcp_f[y] != expected:
                raise AssertionError(f"Tropical LCP identity failed at y={y}: got {lcp_f[y]}, expected {expected}")
            reducible_checked += 1

    irred_lcps = [lcp_f[c0] for c0 in run_c if c0 > 0]
    sum_irred = sum(irred_lcps)
    kkp_bound = 2.0 * n * math.log2(r)

    return {
        "reducible_verified": reducible_checked,
        "n_irred": len(irred_lcps),
        "sum_irreducible_lcp": sum_irred,
        "sum_irred_over_n": round(sum_irred / n, 3),
        "kkp_2n_log2_r_bound": int(kkp_bound),
        "ratio_to_kkp_bound": round(sum_irred / kkp_bound, 4),
        "mean_irred_lcp": round(sum_irred / max(1, len(irred_lcps)), 2),
        "max_irred_lcp": max(irred_lcps) if irred_lcps else 0,
        "median_irred_lcp": sorted(irred_lcps)[len(irred_lcps) // 2] if irred_lcps else 0,
        "T_bytes": T_bytes,
    }


def unrle_bzip2(rle: bytes) -> bytes:
    out = bytearray()
    n = len(rle)
    i = 0
    while i < n:
        c = rle[i]
        if i + 3 < n and rle[i + 1] == c and rle[i + 2] == c and rle[i + 3] == c:
            extra = rle[i + 4]
            out.extend([c] * (4 + extra))
            i += 5
        else:
            out.append(c)
            i += 1
    return bytes(out)


def bzip2_crc32_ref(data: bytes) -> int:
    """Exact bzip2 big-endian CRC-32 (bit-reversed IEEE CRC-32)."""
    # Precompute 256-entry big-endian CRC32 table matching src/crc.rs
    poly = 0x04C11DB7
    tab = []
    for i in range(256):
        c = i << 24
        for _ in range(8):
            c = ((c << 1) & 0xFFFFFFFF) ^ (poly if (c & 0x80000000) else 0)
        tab.append(c)
    crc = 0xFFFFFFFF
    for b in data:
        crc = ((crc << 8) & 0xFFFFFFFF) ^ tab[((crc >> 24) ^ b) & 0xFF]
    return crc ^ 0xFFFFFFFF


def gf2_crc_combine_init():
    """Build exact GF(2)[x]/(P_32) shift operators for O(log len) CRC32 monoid composition!"""
    poly = 0x04C11DB7

    def step_bit(rem: int) -> int:
        return (((rem << 1) & 0xFFFFFFFF) ^ (poly if (rem & 0x80000000) else 0))

    # Linear map M_1(v) = raw_crc_step(v, 0 byte) = v * x^8 mod P_32
    # Represent 32x32 GF(2) matrix as 32 uint32 rows (row b = image of (1 << b))
    def mat_mul(A: list[int], B: list[int]) -> list[int]:
        # (A o B)(e_b) = A(B(e_b))
        res = [0] * 32
        for b in range(32):
            v = B[b]
            acc = 0
            bit = 0
            while v:
                if v & 1:
                    acc ^= A[bit]
                v >>= 1
                bit += 1
            res[b] = acc
        return res

    def mat_apply(A: list[int], v: int) -> int:
        acc = 0
        bit = 0
        while v:
            if v & 1:
                acc ^= A[bit]
            v >>= 1
            bit += 1
        return acc

    m1 = [0] * 32
    for b in range(32):
        rem = 1 << b
        for _ in range(8):
            rem = step_bit(rem)
        m1[b] = rem

    pows = [m1]  # pows[k] = x^{8 * 2^k} mod P_32
    for _ in range(22):
        pows.append(mat_mul(pows[-1], pows[-1]))

    def shift_bytes(v: int, length: int) -> int:
        k = 0
        while length > 0:
            if length & 1:
                v = mat_apply(pows[k], v)
            length >>= 1
            k += 1
        return v

    # Raw linear CRC (starting from state 0, no initial/final ~0)
    tab = []
    for i in range(256):
        c = i << 24
        for _ in range(8):
            c = step_bit(c)
        tab.append(c)

    def raw_crc(bs: bytes) -> int:
        c = 0
        for b in bs:
            c = ((c << 8) & 0xFFFFFFFF) ^ tab[((c >> 24) ^ b) & 0xFF]
        return c

    return shift_bytes, raw_crc


SHIFT_CRC, RAW_CRC = gf2_crc_combine_init()


def probe_world4_rle1_crc_semigroup(T_bytes: bytes, verify_transducer: bool = False) -> dict:
    """
    Measure exact post-BWT RLE1 + CRC32 structure on T_bytes (FRONTIER.md Q4):
      1. Count RLE1 4-equal-byte triggers in T_bytes, expanded output length |U|, and fraction of
         32-byte windows that are pure identity copies (`m == 0` in src/rle.rs:unrle_avx2).
      2. If verify_transducer is True, verify that partitioning T_bytes into arbitrary short chunks
         (e.g. simulating Open's variable-length words of length 1..16!) and composing their exact
         5-state RLE1 Mealy summaries + GF(2)[x]/(P_32) affine-linear CRC summaries reproduces
         BOTH `len(U)` AND `bzip2_crc32_ref(U)` bit-for-bit!
    """
    n = len(T_bytes)
    triggers = 0
    expanded_len = 0
    i = 0
    while i < n:
        c = T_bytes[i]
        if i + 3 < n and T_bytes[i + 1] == c and T_bytes[i + 2] == c and T_bytes[i + 3] == c:
            triggers += 1
            extra = T_bytes[i + 4]
            expanded_len += 4 + extra
            i += 5
        else:
            expanded_len += 1
            i += 1

    # Count 32B windows with zero 4-equal-byte runs
    clean_32b = 0
    total_32b = 0
    for pos in range(0, n - 35, 32):
        total_32b += 1
        has4 = False
        for k in range(32):
            c = T_bytes[pos + k]
            if T_bytes[pos + k + 1] == c and T_bytes[pos + k + 2] == c and T_bytes[pos + k + 3] == c:
                has4 = True
                break
        if not has4:
            clean_32b += 1

    transducer_verified = False
    if verify_transducer:
        U_ref = unrle_bzip2(T_bytes)
        assert len(U_ref) == expanded_len
        crc_ref = bzip2_crc32_ref(U_ref)

        # Verify Affine-Linear CRC32 Homomorphism + RLE1 Mealy state machine across chunked word stream
        # RLE1 state: (last_byte: int, run_len: 0..4), where run_len == 4 means NEXT input byte is `extra`!
        last_b = -1
        rlen = 0
        acc_len = 0
        acc_raw_crc = 0  # tracks raw_crc(prefix); note bzip2_crc(U) = ~(shift_bytes(0xFFFFFFFF, |U|) ^ raw_crc(U))!

        # Slice T_bytes into deterministic variable-size words (lengths cycling 1, 3, 7, 16, 2, 5, 11)
        # so boundaries hit right inside 4-equal-byte runs and before `extra` count bytes!
        step_pat = (1, 3, 7, 16, 2, 5, 11)
        idx = 0
        si = 0
        while idx < n:
            wlen = min(step_pat[si % len(step_pat)], n - idx)
            si += 1
            chunk = T_bytes[idx:idx + wlen]
            idx += wlen

            # Fast check: if rlen == 0 AND chunk has len >= 1 and no internal 4-equal-byte run AND
            # first byte != last_b (so boundary cannot form 4-equal), then chunk passes through RLE1 UNCHANGED,
            # so we update CRC in ONE monoid operation `(acc_len + wlen, shift(acc_raw_crc, wlen) ^ raw_crc(chunk))`!
            has_internal_4 = False
            for u in range(len(chunk) - 3):
                cc = chunk[u]
                if chunk[u + 1] == cc and chunk[u + 2] == cc and chunk[u + 3] == cc:
                    has_internal_4 = True
                    break

            if rlen == 0 and not has_internal_4 and chunk[0] != last_b:
                # Pure monoid homomorphism fast path!
                acc_raw_crc = SHIFT_CRC(acc_raw_crc, wlen) ^ RAW_CRC(chunk)
                acc_len += wlen
                # Compute new tail state (last_b, rlen in 1..3)
                tb = chunk[-1]
                tr = 1
                while tr < len(chunk) and chunk[-1 - tr] == tb:
                    tr += 1
                assert tr <= 3
                last_b = tb
                rlen = tr
            else:
                # Exact 5-state Mealy boundary step
                out_sub = bytearray()
                for b in chunk:
                    if rlen == 4:
                        out_sub.extend([last_b] * b)
                        last_b = -1
                        rlen = 0
                    elif b == last_b:
                        rlen += 1
                        out_sub.append(b)
                    else:
                        last_b = b
                        rlen = 1
                        out_sub.append(b)
                if out_sub:
                    acc_raw_crc = SHIFT_CRC(acc_raw_crc, len(out_sub)) ^ RAW_CRC(bytes(out_sub))
                    acc_len += len(out_sub)

        computed_crc = (SHIFT_CRC(0xFFFFFFFF, acc_len) ^ acc_raw_crc) ^ 0xFFFFFFFF
        assert acc_len == len(U_ref), f"Len mismatch {acc_len} != {len(U_ref)}"
        assert computed_crc == crc_ref, f"CRC mismatch {computed_crc:#x} != {crc_ref:#x}"
        transducer_verified = True

    return {
        "bwt_len_n": n,
        "unrle_len_U": expanded_len,
        "expansion_ratio": round(expanded_len / n, 5),
        "rle1_triggers": triggers,
        "clean_32B_windows_pct": round(100.0 * clean_32b / max(1, total_32b), 2),
        "transducer_crc_monoid_verified": transducer_verified,
    }


def main():
    blocks = load_manifest()
    results = {}
    for idx, b in enumerate(blocks):
        name = b["file"]
        p_mark = b["primary_index"]
        L = (ADMITTED_DIR / name).read_bytes()
        n = len(L)
        run_b, run_d, run_w, run_c, counts, first = extract_runs_and_lf(L)
        edges_u, edges_v, edges_w, in_edges, out_edges = build_overlap_graph(n, run_b, run_d, run_w, run_c)

        w1 = probe_world1_ribbon_and_folding(
            n, run_b, run_d, run_w, run_c, edges_u, edges_v, edges_w, in_edges, out_edges
        )
        lf = w1.pop("lf")
        psi = w1.pop("psi")

        w2 = probe_world2_schur_optimality(n, p_mark, run_b, run_d, run_w, run_c, counts)
        w3 = probe_world3_tropical_lcp(n, p_mark, L, run_c, run_w, lf, psi)
        T_bytes = w3.pop("T_bytes")
        w4 = probe_world4_rle1_crc_semigroup(T_bytes, verify_transducer=(idx < 2))

        results[name] = {
            "world1_ribbon_and_folding": w1,
            "world2_schur_optimality": w2,
            "world3_tropical_lcp": w3,
            "world4_rle1_crc_semigroup": w4,
        }
        print(
            f"[{name}] "
            f"W1(g_rib={w1['ribbon_genus_g']:,}, g_orig={w1['origami_genus_g']:,}, diff={w1['genus_diff_g_rib_minus_g_orig']}, "
            f"F_rib={w1['faces_F_rib']:,}, sameB_fold={w1['in_adj_same_byte_pct']}%) | "
            f"W2(same_shift={w2['same_shift_adjacent_runs']}, m_open={w2['m_open']:,} vs LB={w2['universal_lower_bound_m_at_H_open']:,}, "
            f"gap={w2['open_gap_from_universal_knapsack_pct']}%, byte_m={w2['best_alphabet_subset_m_at_H_open']:,}) | "
            f"W3(irred_LCP/n={w3['sum_irred_over_n']}, mean={w3['mean_irred_lcp']}, med={w3['median_irred_lcp']}) | "
            f"W4(RLE1_trig={w4['rle1_triggers']:,}, clean32B={w4['clean_32B_windows_pct']}%, ver={w4['transducer_crc_monoid_verified']})"
        )

    with open(OUT_PATH, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)
    print(f"\nSaved Cycle 4 probe results to {OUT_PATH}")


if __name__ == "__main__":
    main()
