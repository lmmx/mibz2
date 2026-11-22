#!/usr/bin/env python3
"""
Adversarial Referee #3 (Cycle 15) — Complete Independent Audit of Cycle 14:
  PART 1: Alpha-2's Theorem C6b (fixed-binary L^(d), n=3^d-1, c*_{|Sigma|=2} <= 31/32)
          & Theorem H_grand v2 (T_K=5, r_0/z = Omega(log n), circular vs sentinel '$' BWT,
          independent z_std/z_no verification of cycle14_helper.c).
  PART 2: Gamma-2's Theorem P_pisot (spot-check on 6 blocks, Durand L1 bipartite/sign audit,
          Fibonacci comparison) & Adversarial Stress-Test of Bayer-Diaconis / 99.7% Haar GUID Law
          (exact 16-riffle derivation, UUID anatomy, lag-k correlations, 2D scatter chi2,
          modular stride, cycle structure, within-UUID position transitions on block-0.L & block-143.L).

Zero edits to /home/antigravity/lmmx/mibz2/.
"""
import bisect
import ctypes
import hashlib
import json
import math
import re
import sys
import time
from collections import Counter, defaultdict

import numpy as np


# ============================================================================
# PART 1: INDEPENDENT AUDIT OF ALPHA-2 (THEOREM C6b & THEOREM H_grand v2)
# ============================================================================

def indep_circular_bwt(T: bytes):
    """Exact O(n log n) circular BWT via prefix-doubling."""
    n = len(T)
    sa = list(range(n))
    rk = list(T)
    k = 1
    while k < n:
        sa.sort(key=lambda i: (rk[i], rk[(i + k) % n]))
        tmp = [0] * n
        c = 0
        tmp[sa[0]] = 0
        for idx in range(1, n):
            p0, p1 = sa[idx - 1], sa[idx]
            if rk[p0] != rk[p1] or rk[(p0 + k) % n] != rk[(p1 + k) % n]:
                c += 1
            tmp[p1] = c
        rk = tmp
        if c == n - 1:
            break
        k <<= 1
    L = bytes(T[(i - 1) % n] for i in sa)
    p = sa.index(0)
    r0 = 1 + sum(1 for i in range(1, n) if L[i] != L[i - 1])
    return L, p, r0


def indep_linear_sa_lcp(T: bytes):
    """Exact O(n log n) linear SA + Kasai LCP with unique smallest sentinel at end."""
    n = len(T)
    N = n + 1
    s = [b + 1 for b in T] + [0]
    sa = list(range(N))
    rk = s[:]
    k = 1
    while k < N:
        sa.sort(key=lambda i: (rk[i], rk[(i + k) % N]))
        tmp = [0] * N
        c = 0
        tmp[sa[0]] = 0
        for idx in range(1, N):
            p0, p1 = sa[idx - 1], sa[idx]
            if rk[p0] != rk[p1] or rk[(p0 + k) % N] != rk[(p1 + k) % N]:
                c += 1
            tmp[p1] = c
        rk = tmp
        if c == N - 1:
            break
        k <<= 1
    inv = [0] * N
    for i, p in enumerate(sa):
        inv[p] = i
    lcp = [0] * N
    h = 0
    for i in range(N):
        r = inv[i]
        if r == 0:
            h = 0
            continue
        j = sa[r - 1]
        while i + h < n and j + h < n and T[i + h] == T[j + h]:
            h += 1
        lcp[r] = h
        if h > 0:
            h -= 1
    return sa, inv, lcp


def indep_lz77_exact(T: bytes):
    """Completely independent exact z_std (self-overlapping) and z_no (non-overlapping) via SA+LCP."""
    n = len(T)
    sa, inv, lcp = indep_linear_sa_lcp(T)
    N = n + 1
    # LPF via monotone stack (PSV + NSV)
    lpf = [0] * n
    stk = []
    for i in range(N):
        pos = sa[i]
        cur = lcp[i] if i > 0 else 0
        while stk and stk[-1][0] > pos:
            if stk[-1][1] < cur:
                cur = stk[-1][1]
            stk.pop()
        if pos < n and stk and cur > lpf[pos]:
            lpf[pos] = cur
        stk.append((pos, cur))
    stk = []
    for i in range(N - 1, -1, -1):
        pos = sa[i]
        cur = lcp[i + 1] if i + 1 < N else 0
        while stk and stk[-1][0] > pos:
            if stk[-1][1] < cur:
                cur = stk[-1][1]
            stk.pop()
        if pos < n and stk and cur > lpf[pos]:
            lpf[pos] = cur
        stk.append((pos, cur))

    z_std = 0
    i = 0
    while i < n:
        i += lpf[i] if lpf[i] > 0 else 1
        z_std += 1

    z_no = 0
    i = 0
    while i < n:
        if lpf[i] == 0:
            i += 1
            z_no += 1
            continue
        r = inv[i]
        best = 0
        mn = n
        for k in range(r - 1, -1, -1):
            if lcp[k + 1] < mn:
                mn = lcp[k + 1]
            if mn <= best:
                break
            j = sa[k]
            if j < i:
                cand = mn if mn < (i - j) else (i - j)
                if cand > best:
                    best = cand
        mn = n
        for k in range(r + 1, N):
            if lcp[k] < mn:
                mn = lcp[k]
            if mn <= best:
                break
            j = sa[k]
            if j < i:
                cand = mn if mn < (i - j) else (i - j)
                if cand > best:
                    best = cand
        i += best if best > 0 else 1
        z_no += 1
    return z_std, z_no


def build_Ld_indep(d: int) -> bytes:
    n = (3 ** d) - 1
    k = n // 2
    arr = bytearray([1, 0] * k)
    for e in range(1, d):
        ye = (3 ** d) - (3 ** e) - 2
        assert ye % 2 == 0 and arr[ye] == 1 and arr[ye + 1] == 0
        arr[ye], arr[ye + 1] = 0, 1
    return bytes(arr)


def indep_ibwt_single_cycle(L: bytes):
    n = len(L)
    cnt = [0] * 256
    for b in L:
        cnt[b] += 1
    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += cnt[c]
    occ = [0] * 256
    lf = [0] * n
    for i, b in enumerate(L):
        lf[i] = C[b] + occ[b]
        occ[b] += 1
    cur = 0
    out = bytearray(n)
    vis = 0
    for k in range(n - 1, -1, -1):
        out[k] = L[cur]
        cur = lf[cur]
        vis += 1
    assert cur == 0 and vis == n
    pieces = []
    i = 0
    while i < n:
        j = i + 1
        while j < n and L[j] == L[i]:
            j += 1
        pieces.append((i, j, lf[i] - i, bytes([L[i]])))
        i = j
    return bytes(out), pieces, lf


def strict_compact_coalesce(pieces):
    pieces.sort(key=lambda x: x[0])
    starts = [p[0] for p in pieces]
    ends = [p[1] for p in pieces]
    pref = [0] * (len(pieces) + 1)
    for i, (lo, hi, _, _) in enumerate(pieces):
        assert lo < hi
        if i > 0:
            assert ends[i - 1] <= lo, "Overlapping pieces!"
        pref[i + 1] = pref[i] + (hi - lo)

    def phi(x):
        idx = bisect.bisect_right(starts, x) - 1
        assert 0 <= idx < len(pieces) and starts[idx] <= x < ends[idx]
        return pref[idx] + (x - starts[idx])

    out = []
    for i, (lo, hi, d, w) in enumerate(pieces):
        nlo, nhi = pref[i], pref[i + 1]
        nt = phi(lo + d)
        nd = nt - nlo
        # Spot-check endpoint consistency: phi(hi-1 + d) - (nhi-1) == nd
        assert phi(hi - 1 + d) - (nhi - 1) == nd, "Non-uniform delta after compaction!"
        if out and out[-1][1] == nlo and out[-1][2] == nd and out[-1][3] == w:
            out[-1] = (out[-1][0], nhi, nd, w)
        else:
            out.append((nlo, nhi, nd, w))
    return out


def strict_cgap_verify_Ld(d: int):
    L = build_Ld_indep(d)
    n = len(L)
    k = n // 2
    r0_exp = (3 ** d) - 2 * d + 1

    # 1. Verify LTE d-cycle structure BEFORE swaps on L_0 = [1, 0]^k
    L0 = bytes([1, 0] * k)
    lf0 = [(k + i // 2) if (i % 2 == 0) else (i // 2) for i in range(n)]
    fl0 = [0] * n
    for i, v in enumerate(lf0):
        fl0[v] = i
    # Verify exact congruence FL_0(y) + 1 == 2(y+1) mod 3^d
    for y in range(n):
        assert (fl0[y] + 1) % (3 ** d) == (2 * (y + 1)) % (3 ** d)
    # Count exact cycles of LF_0
    seen = bytearray(n)
    cyc_lens = []
    for i in range(n):
        if not seen[i]:
            c = i
            ln = 0
            while not seen[c]:
                seen[c] = 1
                c = lf0[c]
                ln += 1
            cyc_lens.append(ln)
    assert len(cyc_lens) == d and sorted(cyc_lens) == sorted(2 * (3 ** e) for e in range(d))

    # 2. Verify L^(d) single-cycle, exact r_0, and forward-BWT round-trip
    T, st, lf = indep_ibwt_single_cycle(L)
    assert len(st) == r0_exp
    L_rt, _, r0_rt = indep_circular_bwt(T)
    assert L_rt == L and r0_rt == r0_exp

    # 3. Verify 4-zone disjointness & strict bounds below y_{d-1}
    u_lo = (k + 3) // 4
    u_hi = (5 * k - 4) // 16
    w_giant = u_hi - u_lo
    y_min = (3 ** d) - (3 ** (d - 1)) - 2
    B_ones = [2 * u for u in range(u_lo, u_hi)]
    B_zeros = [2 * u + 1 for u in range(u_lo, u_hi - 1)]
    A_keep = [4 * u + 1 for u in range(u_lo, u_hi)]
    A_del = [x for x in range(4 * u_lo + 2, 4 * (u_hi - 1) + 1) if (x - 1) % 4 != 0]
    Pred_A_del = [2 * (x - k) for x in A_del]
    LF_B = [k + u for u in range(u_lo, u_hi)]

    # Verify Pred_A_del really are the exact LF-predecessors of A_del, and LF_B == LF(B_ones)
    for x, px in zip(A_del, Pred_A_del):
        assert lf[px] == x
    for b, tb in zip(B_ones, LF_B):
        assert lf[b] == tb
    for u in range(u_lo, u_hi - 1):
        assert lf[4 * u + 3] == 2 * u + 1
    for u in range(u_lo, u_hi):
        assert lf[4 * u + 1] == 2 * u

    z1 = set(B_ones) | set(B_zeros)
    z2 = set(A_keep) | set(A_del)
    z3 = set(Pred_A_del)
    z4 = set(LF_B)
    all_z = [z1, z2, z3, z4]
    for i in range(4):
        assert max(all_z[i]) < y_min, f"Zone {i} reaches swap y_min={y_min}!"
        for j in range(i + 1, 4):
            assert not (all_z[i] & all_z[j]), f"Zones {i},{j} overlap!"

    # 4. Execute strict C_gap contraction (enforcing G cap (G + delta_B) == empty on EVERY Join!)
    alive = list(range(n))

    def o2c(ox):
        idx = bisect.bisect_left(alive, ox)
        assert idx < len(alive) and alive[idx] == ox
        return idx

    def find_piece(st_l, starts, cx):
        idx = bisect.bisect_right(starts, cx) - 1
        assert st_l[idx][0] <= cx < st_l[idx][1]
        return idx

    def strict_join(st_l, ia, ib):
        assert ia != ib
        alo, ahi, ad, aw = st_l[ia]
        blo, bhi, bd, bw = st_l[ib]
        assert bd != 0
        ov_lo, ov_hi = max(alo + ad, blo), min(ahi + ad, bhi)
        assert ov_lo < ov_hi
        if ov_hi - ov_lo > abs(bd):
            ov_hi = ov_lo + abs(bd)
        # Explicit disjointness check: G cap (G + bd) == empty!
        assert (ov_hi <= ov_lo + bd) or (ov_hi + bd <= ov_lo), "SELF-OVERLAP VIOLATION!"
        a_lo, a_hi = ov_lo - ad, ov_hi - ad
        nxt = []
        for idx, pc in enumerate(st_l):
            if idx == ia:
                if alo < a_lo:
                    nxt.append((alo, a_lo, ad, aw))
                nxt.append((a_lo, a_hi, ad + bd, aw + bw))
                if a_hi < ahi:
                    nxt.append((a_hi, ahi, ad, aw))
            elif idx == ib:
                if blo < ov_lo:
                    nxt.append((blo, ov_lo, bd, bw))
                if ov_hi < bhi:
                    nxt.append((ov_hi, bhi, bd, bw))
            else:
                nxt.append(pc)
        del alive[ov_lo:ov_hi]
        return strict_compact_coalesce(nxt), ov_hi - ov_lo

    def strict_pow(st_l, i):
        lo, hi, dd, w = st_l[i]
        Lp = hi - lo
        assert dd != 0 and abs(dd) < Lp
        nxt = list(st_l[:i]) + list(st_l[i + 1:])
        if dd > 0:
            q, rem = divmod(Lp, dd)
            if rem > 0:
                nxt.append((lo, lo + rem, (q + 1) * dd, w * (q + 1)))
            if dd - rem > 0:
                nxt.append((lo + rem, lo + dd, q * dd, w * q))
            del alive[lo + dd:hi]
        else:
            s = -dd
            q, rem = divmod(Lp, s)
            if s - rem > 0:
                nxt.append((hi - s, hi - rem, -q * s, w * q))
            if rem > 0:
                nxt.append((hi - rem, hi, -(q + 1) * s, w * (q + 1)))
            del alive[lo:hi - s]
        return strict_compact_coalesce(nxt), Lp - abs(dd)

    steps = 0
    sum_disc = 0

    # Phase 1: delete B_zeros via 4u+3
    for u in range(u_lo, u_hi - 1):
        starts = [p[0] for p in st]
        ia = find_piece(st, starts, o2c(4 * u + 3))
        ib = find_piece(st, starts, o2c(2 * u + 1))
        st, gl = strict_join(st, ia, ib)
        steps += 1
        sum_disc += gl - 1

    # Phase 2: delete A_del via Pred_A_del
    for x, px in zip(A_del, Pred_A_del):
        starts = [p[0] for p in st]
        cx = o2c(x)
        ib = find_piece(st, starts, cx)
        ia = find_piece(st, starts, o2c(px))
        st, gl = strict_join(st, ia, ib)
        steps += 1
        sum_disc += gl - 1

    # Phase 3: Giant Join(A*, B*) of exact width w_giant
    starts = [p[0] for p in st]
    ia = find_piece(st, starts, o2c(A_keep[0]))
    ib = find_piece(st, starts, o2c(B_ones[0]))
    assert st[ia][1] - st[ia][0] == w_giant and st[ib][1] - st[ib][0] == w_giant
    assert st[ia][0] + st[ia][2] == st[ib][0]
    st, gl = strict_join(st, ia, ib)
    assert gl == w_giant
    steps += 1
    sum_disc += gl - 1

    # Phase 4: Greedy completion (Pow first, else widest valid Join)
    while not (len(st) == 1 and st[0][2] == 0):
        pi = next((i for i, (lo, hi, dd, _) in enumerate(st) if dd != 0 and abs(dd) < hi - lo), None)
        if pi is not None:
            st, dl = strict_pow(st, pi)
            steps += 1
            sum_disc += dl - 1
            continue
        starts = [p[0] for p in st]
        best = None
        for ia, (alo, ahi, ad, _) in enumerate(st):
            atlo, athi = alo + ad, ahi + ad
            ib = max(0, bisect.bisect_right(starts, atlo) - 1)
            while ib < len(st):
                blo, bhi, bd, _ = st[ib]
                if blo >= athi:
                    break
                if ib != ia and bd != 0:
                    ov_lo = atlo if atlo > blo else blo
                    ov_hi = athi if athi < bhi else bhi
                    if ov_lo < ov_hi:
                        gl = min(ov_hi - ov_lo, abs(bd))
                        sc = (-gl, (ahi - alo) + (bhi - blo))
                        if best is None or sc < best[0]:
                            best = (sc, ia, ib)
                ib += 1
        st, gl = strict_join(st, best[1], best[2])
        steps += 1
        sum_disc += gl - 1

    # Verify Width-Discharge Identity and 100% byte-exact cyclic reconstruction!
    assert steps == (n - 1) - sum_disc
    assert steps <= (n - 1) - (w_giant - 1)
    dec = st[0][3][::-1]
    assert len(dec) == n and T in (dec + dec)
    return n, r0_exp, w_giant, steps, sum_disc


def debruijn_lyndon_indep(k: int) -> bytes:
    a = [0] * (2 * k)
    seq = []
    def db(t, p):
        if t > k:
            if k % p == 0:
                seq.extend(a[1:p + 1])
        else:
            a[t] = a[t - p]
            db(t + 1, p)
            for j in range(a[t - p] + 1, 2):
                a[t] = j
                db(t + 1, t)
    db(1, 1)
    return bytes(seq)


def audit_part1():
    print("=" * 88)
    print("PART 1: INDEPENDENT AUDIT OF ALPHA-2 (THEOREM C6b & THEOREM H_grand v2)")
    print("=" * 88)

    # 1. Theorem C6b verification for d = 4, 5, 6, 7
    print("  1.1 Strict C_gap verification of L^(d) (LTE d-cycles + swaps + 4-zone + exact decode):")
    for d in (4, 5, 6, 7):
        n, r0, wg, q_act, disc = strict_cgap_verify_Ld(d)
        q_ub = (n - 1) - (wg - 1)
        print(f"    [VERIFIED] d={d}: n={n:4d}, r_0={r0:4d}, w_giant={wg:3d} | "
              f"Q_ub/r_0={q_ub/r0:.5f}, Q_act={q_act:4d} ({q_act/r0:.5f} < 31/32={31/32:.5f}) | "
              f"sum_disc={disc:4d}, Width-Discharge & byte-exact decode PASS")

    # 2. C helper audit vs independent LZ77 on T^(d) (d=4..8)
    lib = ctypes.CDLL("/tmp/fcp-a/libcycle14.so")
    lib.exact_circular_bwt.argtypes = [ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.POINTER(ctypes.c_int), ctypes.POINTER(ctypes.c_int)]
    lib.exact_circular_bwt.restype = ctypes.c_int
    lib.exact_lz77_counts.argtypes = [ctypes.c_char_p, ctypes.c_int, ctypes.POINTER(ctypes.c_int), ctypes.POINTER(ctypes.c_int), ctypes.POINTER(ctypes.c_int)]

    print("\n  1.2 Independent z_std / z_no cross-check vs cycle14_helper.c on T^(d) (d=4..8):")
    for d in range(4, 9):
        L = build_Ld_indep(d)
        T, _, _ = indep_ibwt_single_cycle(L)
        zs_ind, zno_ind = indep_lz77_exact(T)
        zs_c, zl_c, zno_c = ctypes.c_int(0), ctypes.c_int(0), ctypes.c_int(0)
        lib.exact_lz77_counts(T, len(T), ctypes.byref(zs_c), ctypes.byref(zl_c), ctypes.byref(zno_c))
        assert zs_ind == zs_c.value and zno_ind == zno_c.value
        print(f"    [VERIFIED] d={d} (n={len(T):4d}): z_std={zs_ind:3d} (C={zs_c.value}), z_no={zno_ind:3d} (C={zno_c.value}) — EXACT MATCH")

    # 3. Circular BWT vs Sentinel-terminated '$' BWT on T^(d) (d=4..10) & de Bruijn B_k (k=4..14)
    print("\n  1.3 Circular BWT vs Sentinel-Terminated ('$' min & '$' max) BWT r_0 invariance:")
    def c_r0(s: bytes):
        sc = ctypes.c_int(0); p0 = ctypes.c_int(0)
        return lib.exact_circular_bwt(s, len(s), None, ctypes.byref(sc), ctypes.byref(p0)), sc.value

    print("    Family T^(d) (d=4..10):")
    for d in range(4, 11):
        L = build_Ld_indep(d)
        T, _, _ = indep_ibwt_single_cycle(L)
        r_circ, sc_c = c_r0(T)
        T_smin = bytes([b + 1 for b in T] + [0])
        T_smax = bytes(list(T) + [2])
        r_smin, sc_min = c_r0(T_smin)
        r_smax, sc_max = c_r0(T_smax)
        assert sc_c == 1 and sc_min == 1 and sc_max == 1
        print(f"      d={d:2d} (n={len(T):5d}): r_circ={r_circ:5d}, r(T+$min)={r_smin:5d} (delta={r_smin-r_circ:+4d}), "
              f"r(T+$max)={r_smax:5d} (delta={r_smax-r_circ:+4d})")

    print("    Family de Bruijn B_k (k=4..14):")
    for k in range(4, 15):
        T = debruijn_lyndon_indep(k)
        r_circ, sc_c = c_r0(T)
        T_smin = bytes([b + 1 for b in T] + [0])
        T_smax = bytes(list(T) + [2])
        r_smin, sc_min = c_r0(T_smin)
        r_smax, sc_max = c_r0(T_smax)
        assert sc_c == 1 and sc_min == 1 and sc_max == 1
        print(f"      k={k:2d} (n={len(T):5d}): r_circ={r_circ:5d}, r(T+$min)={r_smin:5d} (delta={r_smin-r_circ:+4d}), "
              f"r(T+$max)={r_smax:5d} (delta={r_smax-r_circ:+4d})")


# ============================================================================
# PART 2: INDEPENDENT AUDIT OF GAMMA-2 (THEOREM P_pisot & BAYER-DIACONIS GUID)
# ============================================================================

def audit_part2():
    print("\n" + "=" * 88)
    print("PART 2: INDEPENDENT AUDIT OF GAMMA-2 (THEOREM P_pisot & GUID HAAR MIXING)")
    print("=" * 88)

    # 2.1 Exact analytic 16-riffle derivation
    p_analytic = 1.0 - (15.0 / 16.0) ** 8
    print(f"  2.1 Exact 16-riffle analytic law: P(1 <= Delta <= 8) = 1 - (15/16)^8 = {p_analytic:.10f} (~0.4033) [PROVED]")

    # Load cycle8 loader (read-only) to inspect exact same 6 blocks & GAP4_hex
    sys.path.insert(0, "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-foundational-contraction-programme/cycle8/gamma")
    import cycle8_ap4_text_anatomy as c8
    HEX_BYTES = set(b"0123456789abcdefABCDEF")

    blocks = c8.load_blocks()

    # 2.2 Audit Durand L1 spectral gap == 1.00 (CHECK BIPARTITE QUOTE PARITY: lambda_2 = -lambda_1?!)
    print("\n  2.2 Audit of Gamma-2's Durand L1 Spectral Gap = 1.00 (JSON Quote-Parity Bipartiteness Check!):")
    for entry, raw in blocks:
        n = entry["n"]
        p = entry["primary_index"]
        strips, LF, T, row_to_tpos = c8.build_strips_and_decode(raw, p)
        occ_q = [i for i, ch in enumerate(T) if ch == ord('"')]
        m_q = len(occ_q)
        rwords = [T[occ_q[i]:occ_q[i + 1]] if i + 1 < m_q else T[occ_q[i]:] + T[:occ_q[0]] for i in range(m_q)]
        rw_cnt = Counter(rwords)
        d1 = len(rw_cnt)
        vocab = [w for w, _ in rw_cnt.most_common()]
        w2id = {w: idx for idx, w in enumerate(vocab)}
        seq1 = [w2id[w] for w in rwords]
        # Check parity of quote index (even vs odd occurrence of '"')!
        parity_cnt = defaultdict(Counter)
        for idx, sym in enumerate(seq1):
            parity_cnt[sym][idx & 1] += 1
        pure_parity = sum(max(c[0], c[1]) for c in parity_cnt.values()) / m_q

        # Check actual complex eigenvalues (signed real parts!) of Top-400 A_top AND Markov P_top
        K_top = min(400, d1)
        A_top = np.zeros((K_top, K_top), dtype=np.float64)
        for i in range(m_q):
            u, v = seq1[i], seq1[(i + 1) % m_q]
            if u < K_top and v < K_top:
                A_top[v, u] += 1.0
        eigs = sorted(np.linalg.eigvals(A_top), key=lambda z: -abs(z))
        P_top = A_top / np.maximum(A_top.sum(axis=0)[None, :], 1.0)
        e_mk = sorted(np.linalg.eigvals(P_top), key=lambda z: -abs(z))
        # Check Level-2 d2
        occ_w0 = [i for i, s in enumerate(seq1) if s == 0]
        d2 = len(set(tuple(seq1[occ_w0[i]:occ_w0[i+1]] if i+1 < len(occ_w0) else seq1[occ_w0[i]:]+seq1[:occ_w0[0]]) for i in range(len(occ_w0))))
        print(f"    {entry['file']:11s}: d1={d1:4d}->d2={d2:4d} | quote-parity purity={100*pure_parity:.2f}% | "
              f"l1={eigs[0].real:+.1f}, l2={eigs[1].real:+.1f}{eigs[1].imag:+.1f}j, |l3|={abs(eigs[2]):.1f} (|l1|/|l3|={abs(eigs[0])/abs(eigs[2]):.2f}) | "
              f"Mk mu2={e_mk[1].real:+.4f}{e_mk[1].imag:+.4f}j, |mu3|={abs(e_mk[2]):.4f}")

    # Also test Gamma-2's 1B.3 bigram-count matrix on the EXACT Fibonacci Pisot word!
    s_fib = "a"
    for _ in range(15):
        s_fib = "".join("ab" if ch == "a" else "a" for ch in s_fib)
    occ_a = [i for i, ch in enumerate(s_fib) if ch == "a"]
    rw_f = [s_fib[occ_a[i]:occ_a[i + 1]] if i + 1 < len(occ_a) else s_fib[occ_a[i]:] + s_fib[:occ_a[0]] for i in range(len(occ_a))]
    vf = sorted(set(rw_f))
    f2i = {w: i for i, w in enumerate(vf)}
    A_fib = np.zeros((len(vf), len(vf)))
    for i in range(len(rw_f)):
        A_fib[f2i[rw_f[(i + 1) % len(rw_f)]], f2i[rw_f[i]]] += 1.0
    ef = sorted(np.linalg.eigvals(A_fib), key=lambda z: -abs(z))
    print(f"    [CONTROL: Exact Pisot Fibonacci F_17 bigram count matrix A_dur]: "
          f"l1={ef[0].real:+.1f}, l2={ef[1].real:+.1f} -> dim(|l|>1) = {sum(abs(z)>1 for z in ef)} (NOT 1! Unnormalized count matrix scales O(n)!)")

    # 2.3 Killer Adversarial Stress-Test of "99.7% Haar-Random" on GAP4_hex (block-0.L & block-143.L)
    print("\n  2.3 KILLER ADVERSARIAL STRESS-TEST OF `GAP4_hex` PERMUTATION (block-0.L & block-143.L):")
    rng = np.random.default_rng(99999)
    uuid_re = re.compile(rb'Q\d+\$[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}')

    for entry, raw in blocks:
        if entry["file"] not in ("block-0.L", "block-143.L"):
            continue
        n = entry["n"]
        p = entry["primary_index"]
        strips, LF, T, row_to_tpos = c8.build_strips_and_decode(raw, p)
        tpos_to_row = [0] * n
        for r, t in enumerate(row_to_tpos):
            tpos_to_row[t] = r
        tags = c8.classify_wikidata_text(T)
        ap4 = c8.run_ap4_max_with_reasons(strips, n, p, cap_len=15)

        print(f"\n    === {entry['file']} (n={n}) ===")
        # (a) Inspect actual GUID text anatomy!
        matches = list(uuid_re.finditer(T))
        print(f"    [UUID Anatomy] Regex Q\\d+\\$UUID matches: {len(matches)}")
        if matches:
            samples = [m.group(0).decode("ascii") for m in matches[:6]]
            print(f"      Samples: {samples}")
            # Check RFC 4122 version nibble (char after 2nd dash) and variant nibble (char after 3rd dash)
            ver_nibbles = Counter()
            var_nibbles = Counter()
            case_counts = Counter()
            for m in matches:
                u = m.group(0).split(b"$")[1].decode("ascii")
                ver_nibbles[u[14]] += 1
                var_nibbles[u[19].lower()] += 1
                if u == u.lower():
                    case_counts["lower"] += 1
                elif u == u.upper():
                    case_counts["upper"] += 1
                else:
                    case_counts["mixed"] += 1
            print(f"      UUID version nibble (pos 14): {dict(ver_nibbles.most_common(6))}")
            print(f"      UUID variant nibble (pos 19): {dict(var_nibbles.most_common(6))} | case={dict(case_counts)}")

        # Map each text position inside a matched UUID to its offset inside the 36-char UUID (0..35, or -1 if Q-id/other)
        uuid_off = np.full(n, -1, dtype=np.int16)
        uuid_id = np.full(n, -1, dtype=np.int32)
        for uid, m in enumerate(matches):
            dollar = m.group(0).index(b"$")
            u_start = m.start() + dollar + 1
            for off in range(36):
                if u_start + off < n:
                    uuid_off[u_start + off] = off
                    uuid_id[u_start + off] = uid

        # Extract exact GAP4_hex
        ap4_rows = ap4[2]
        GAP4 = [r for r in ap4_rows if tags[row_to_tpos[r]] == c8.CLS_GUID]
        GAP4_hex = sorted(r for r in GAP4 if T[row_to_tpos[r]] in HEX_BYTES and T[(row_to_tpos[r] + 1) % n] in HEX_BYTES)
        m = len(GAP4_hex)
        r2rk = {r: i for i, r in enumerate(GAP4_hex)}
        G_by_t = sorted(GAP4_hex, key=lambda r: row_to_tpos[r])
        pi = np.empty(m, dtype=np.int64)
        for idx in range(m):
            pi[r2rk[G_by_t[idx]]] = r2rk[G_by_t[(idx + 1) % m]]

        F_bytes = np.array([T[(row_to_tpos[r] + 1) % n] for r in GAP4_hex], dtype=np.uint8)
        tpos_arr = np.array([row_to_tpos[r] for r in GAP4_hex], dtype=np.int64)
        # Note: row_to_tpos[r] is the L-byte position; F-byte is at (t + 1) % n!
        f_tpos = (tpos_arr + 1) % n
        f_uoff = uuid_off[f_tpos]
        f_uid = uuid_id[f_tpos]

        # (b) Check byte frequency uniformity in GAP4_hex!
        fcnt = Counter(chr(b) for b in F_bytes)
        in_uuid_frac = float(np.mean(f_uoff >= 0))
        in_qid_frac = 1.0 - in_uuid_frac
        print(f"    [GAP4_hex |G|={m}] in 36-char UUID={100*in_uuid_frac:.2f}%, outside (Q-id digits/other)={100*in_qid_frac:.2f}%")
        print(f"      F-byte counts: {dict(sorted(fcnt.items()))}")

        # (c) Check cycle structure of pi (and pi^2, pi^3)!
        def count_cycles(perm):
            vis = bytearray(len(perm))
            lens = []
            for s in range(len(perm)):
                if not vis[s]:
                    c = s; ln = 0
                    while not vis[c]:
                        vis[c] = 1; c = perm[c]; ln += 1
                    lens.append(ln)
            return len(lens), sorted(lens, reverse=True)[:5]

        c1, _ = count_cycles(pi)
        print(f"    [Cycle Structure] pi has EXACTLY {c1} cycle (len {m}) BY CONSTRUCTION (single-cycle Ind_G(FL) vs Haar E[cycles]={math.log(m)+0.5772:.2f})")

        # (d) Multi-step / lag-k correlations (k = 1, 2, 3, 4) vs Haar-random single m-cycle!
        # Generate 20 Haar-random single m-cycles for exact null baseline
        haar_cycles = []
        for _ in range(20):
            ord_r = rng.permutation(m)
            hc = np.empty(m, dtype=np.int64)
            hc[ord_r[:-1]] = ord_r[1:]
            hc[ord_r[-1]] = ord_r[0]
            haar_cycles.append(hc)

        print("    [Multi-step pi^k & Spatial lag-k correlations]:")
        cur_p = pi.copy()
        for k in (1, 2, 3, 4):
            # Adjacent difference of pi^k: |pi^k[i+1] - pi^k[i]| <= 8
            dc = (((cur_p[1:] - cur_p[:-1]) + m // 2) % m) - (m // 2)
            p_n8 = float(np.mean(np.abs(dc) <= 8))
            # Spatial lag-k on pi: |pi[i+k] - pi[i]| <= 8
            dk = (((pi[k:] - pi[:-k]) + m // 2) % m) - (m // 2)
            p_lagk_n8 = float(np.mean(np.abs(dk) <= 8))
            # Same on Haar null
            h_n8 = 17.0 / m
            # Check how often pi^k lands inside SAME UUID
            same_uuid = float(np.mean((f_uid >= 0) & (f_uid == f_uid[cur_p])))
            # Adjacent text step median
            dt_step = (f_tpos[cur_p] - f_tpos) % n
            med_dt = float(np.median(dt_step))
            print(f"      k={k}: P(|pi^k[i+1]-pi^k[i]|<=8)={p_n8:.5f} (Haar {h_n8:.5f}, ratio={p_n8/h_n8:.2f}x) | "
                  f"P(|pi[i+k]-pi[i]|<=8)={p_lagk_n8:.5f} ({p_lagk_n8/h_n8:.2f}x) | same_UUID={100*same_uuid:.1f}% | med_dt={med_dt:.0f}B")
            cur_p = cur_p[pi]

        # (e) 2D Scatter Grid (16x16 and 32x32) Chi-squared/df & Mutual Information I(row_bin; pi(row)_bin)!
        for nb in (16, 32):
            bx = (np.arange(m) * nb) // m
            by = (pi * nb) // m
            grid = np.zeros((nb, nb), dtype=np.float64)
            np.add.at(grid, (bx, by), 1.0)
            exp = m / (nb * nb)
            chi2_2d = float(np.sum((grid - exp) ** 2 / exp) / (nb * nb - 1))
            # Mutual information in bits
            pxy = grid / m
            px = pxy.sum(axis=1, keepdims=True)
            py = pxy.sum(axis=0, keepdims=True)
            nz = pxy > 0
            mi_bits = float(np.sum(pxy[nz] * np.log2(pxy[nz] / (px * py)[nz])))
            # Compare against 20 Haar single-cycles
            mc_chi2, mc_mi = [], []
            for hc in haar_cycles:
                hy = (hc * nb) // m
                hg = np.zeros((nb, nb), dtype=np.float64)
                np.add.at(hg, (bx, hy), 1.0)
                mc_chi2.append(float(np.sum((hg - exp) ** 2 / exp) / (nb * nb - 1)))
                hxy = hg / m
                nz2 = hxy > 0
                mc_mi.append(float(np.sum(hxy[nz2] * np.log2(hxy[nz2] / (px * py)[nz2]))))
            print(f"    [2D Scatter {nb}x{nb}] chi2/df = {chi2_2d:.3f} (Haar MC={np.mean(mc_chi2):.3f} +/- {np.std(mc_chi2):.3f}) | "
                  f"MI = {mi_bits:.4f} bits (Haar MC={np.mean(mc_mi):.4f} bits, excess={mi_bits - np.mean(mc_mi):+.4f} bits)")

        # (f) Within-bucket 2D Scatter (u_loc vs v_loc on 16x16 grid, stripping hex-bucket marginals!)
        uniq_b = np.unique(F_bytes)
        blo = {int(b): int(np.flatnonzero(F_bytes == b)[0]) for b in uniq_b}
        bsz = {int(b): int(np.sum(F_bytes == b)) for b in uniq_b}
        u_loc = np.array([(i - blo[int(F_bytes[i])]) / bsz[int(F_bytes[i])] for i in range(m)])
        dst_F = F_bytes[pi]
        v_loc = np.array([(pi[i] - blo[int(dst_F[i])]) / bsz[int(dst_F[i])] for i in range(m)])
        nb = 16
        ux = np.minimum(nb - 1, (u_loc * nb).astype(np.int64))
        vy = np.minimum(nb - 1, (v_loc * nb).astype(np.int64))
        g_wb = np.zeros((nb, nb), dtype=np.float64)
        np.add.at(g_wb, (ux, vy), 1.0)
        exp_wb = m / (nb * nb)
        chi2_wb_2d = float(np.sum((g_wb - exp_wb) ** 2 / exp_wb) / (nb * nb - 1))
        pxy = g_wb / m
        nz = pxy > 0
        mi_wb = float(np.sum(pxy[nz] * np.log2(pxy[nz] / (pxy.sum(1, keepdims=True) * pxy.sum(0, keepdims=True))[nz])))
        print(f"    [Within-Bucket 2D 16x16 (u_loc, v_loc)] chi2/df = {chi2_wb_2d:.3f} (Haar ~1.00) | MI = {mi_wb:.4f} bits")

        # (g) UUID-position clustering & transition determinism!
        # Check: in BWT row order (rank i), are UUID offsets `f_uoff` clustered (e.g. near dashes/end-of-UUID)?
        # And how strongly does `f_uoff[i]` predict `f_uoff[pi[i]]`?
        valid_u = (f_uoff >= 0) & (f_uoff[pi] >= 0)
        if np.any(valid_u):
            # Mutual information I(uoff[i]; uoff[pi[i]]) and I(u_loc_bin; uoff[i])!
            u1 = f_uoff[valid_u]
            u2 = f_uoff[pi][valid_u]
            tbl = np.zeros((36, 36), dtype=np.float64)
            np.add.at(tbl, (u1, u2), 1.0)
            p12 = tbl / tbl.sum()
            p1 = p12.sum(axis=1, keepdims=True)
            p2 = p12.sum(axis=0, keepdims=True)
            nz = p12 > 0
            mi_uoff = float(np.sum(p12[nz] * np.log2(p12[nz] / (p1 * p2)[nz])))
            # Also check MI(u_loc 16-bin; f_uoff) — does BWT within-bucket position encode UUID offset?!
            tbl2 = np.zeros((16, 36), dtype=np.float64)
            np.add.at(tbl2, (ux[f_uoff >= 0], f_uoff[f_uoff >= 0]), 1.0)
            q12 = tbl2 / tbl2.sum()
            nz2 = q12 > 0
            mi_pos_uoff = float(np.sum(q12[nz2] * np.log2(q12[nz2] / (q12.sum(1, keepdims=True) * q12.sum(0, keepdims=True))[nz2])))
            print(f"    [UUID-Offset Structure] MI(uoff[i], uoff[pi[i]]) = {mi_uoff:.3f} bits | "
                  f"MI(BWT within-bucket rank, uoff) = {mi_pos_uoff:.3f} bits!")

        # (h) Modular stride check: for moduli M in (2, 4, 8, 16, 32, 36): chi2/df of (pi[i] - i) % M
        mod_chi2 = []
        d_raw = pi - np.arange(m)
        for M in (2, 4, 8, 16, 32, 36):
            cnts = np.bincount(d_raw % M, minlength=M)
            c2 = float(np.sum((cnts - m / M) ** 2 / (m / M)) / (M - 1))
            mod_chi2.append(f"mod{M}:{c2:.2f}")
        print(f"    [Modular Stride chi2/df of (pi[i]-i)%M]: {', '.join(mod_chi2)}")


if __name__ == "__main__":
    audit_part1()
    audit_part2()
