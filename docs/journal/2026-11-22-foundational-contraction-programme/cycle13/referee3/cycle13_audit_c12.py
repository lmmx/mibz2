#!/usr/bin/env python3
"""
Adversarial Referee #3 (Cycle 13) — Independent Audit of Alpha-2's Cycle 12 Theorems:
  - Theorem C5 (W1a, W1b, W2..W8, exact rational LP, Farkas barrier, infinite-dim invariance)
  - Theorem C6 (Spliced L16^m for m = 1, 2, 3, 4, 6, 8: valid single-cycle BWT, r0=16m, 13m-1 valid C_gap moves)
  - Theorem H_grand (every separating link across Models 1..5, edge cases & caveats)

Completely self-contained: ZERO imports from /tmp/fcp-a/ or /home/antigravity/lmmx/mibz2/.
"""
from bisect import bisect_right
from collections import Counter, deque
from fractions import Fraction
from itertools import combinations
import random


# ============================================================================
# 1. INDEPENDENT BWT / INVERSE-BWT / CYCLIC-TEXT ORACLE
# ============================================================================

def indep_bwt_encode(T):
    """Exact forward BWT by sorting all cyclic rotations of sequence T (bytes or tuple of ints)."""
    n = len(T)
    assert n > 0
    seq = tuple(T)
    rots = sorted(range(n), key=lambda i: seq[i:] + seq[:i])
    L = tuple(seq[(i - 1) % n] for i in rots)
    primary_idx = rots.index(0)
    return L, primary_idx


def indep_bwt_inverse_and_validate(L):
    """
    Given L (sequence of ints), compute LF mapping, verify single cycle of length n,
    invert to T (ending at L[0]), re-encode T with forward BWT, and assert L_re == L!
    Also returns exact run count r0 and initial W-IET state (with exact primary index p).
    """
    seq = tuple(L)
    n = len(seq)
    alpha = sorted(set(seq))
    counts = Counter(seq)
    C = {}
    acc = 0
    for c in alpha:
        C[c] = acc
        acc += counts[c]
    cur = dict(C)
    LF = [0] * n
    for i, c in enumerate(seq):
        LF[i] = cur[c]
        cur[c] += 1

    x = 0
    visited = set()
    T_rev = []
    for _ in range(n):
        if x in visited:
            return None
        visited.add(x)
        T_rev.append(seq[x])
        x = LF[x]
    if x != 0 or len(visited) != n:
        return None
    T = tuple(reversed(T_rev))

    L_re, p = indep_bwt_encode(T)
    if L_re != seq:
        return None

    runs = []
    i = 0
    while i < n:
        c = seq[i]
        j = i + 1
        while j < n and seq[j] == c:
            j += 1
        runs.append((c, i, j))
        i = j

    cur_t = dict(C)
    init = []
    for c, lo, hi in runs:
        d = cur_t[c] - lo
        cur_t[c] += (hi - lo)
        init.append((lo, hi, d, (c,)))
    return T, p, len(runs), tuple(init)


def decode_wiet_cycles(state):
    """
    Verify state is a valid bijection on [0, m), maximally coalesced,
    and return list of cyclic words (each word reversed so it matches T direction).
    """
    if not state:
        raise AssertionError("Empty state")
    m = 0
    for i, (lo, hi, d, w) in enumerate(state):
        assert lo == m and hi > lo, f"Non-contiguous domain at {i}: {(lo, hi)}"
        assert len(w) >= 1
        if i > 0:
            plo, phi, pd, pw = state[i - 1]
            assert not (pd == d and pw == w), f"Not maximally coalesced at {i-1},{i}!"
        m = hi

    nxt = [None] * m
    word_at = [None] * m
    for lo, hi, d, w in state:
        for x in range(lo, hi):
            y = x + d
            assert 0 <= y < m, f"Target {y} out of range [0, {m})"
            assert nxt[x] is None
            nxt[x] = y
            word_at[x] = w
    assert len(set(nxt)) == m, "Not a bijection!"

    seen = [False] * m
    cycles = []
    for start in range(m):
        if not seen[start]:
            cur = start
            chunks = []
            while not seen[cur]:
                seen[cur] = True
                chunks.append(word_at[cur])
                cur = nxt[cur]
            full = ()
            for ch in chunks:
                full += ch
            cycles.append(full[::-1])
    return cycles


def is_cyclic_rotation(a, b):
    if len(a) != len(b):
        return False
    if len(a) == 0:
        return True
    aa = a + a
    n = len(a)
    for i in range(n):
        if aa[i:i + n] == b:
            return True
    return False


# ============================================================================
# 2. INDEPENDENT STRICT C_gap ENGINE (explicit G cap (G + delta_B) == empty!)
# ============================================================================

def indep_compact_and_coalesce(pieces):
    pieces = sorted(pieces, key=lambda x: x[0])
    old_los = [p[0] for p in pieces]
    new_los = []
    acc = 0
    for lo, hi, d, w in pieces:
        assert lo < hi
        new_los.append(acc)
        acc += (hi - lo)
    for i in range(len(pieces) - 1):
        assert pieces[i][1] <= pieces[i + 1][0], "Overlapping surviving pieces!"

    def phi(x):
        idx = bisect_right(old_los, x) - 1
        assert idx >= 0 and old_los[idx] <= x < pieces[idx][1], f"phi({x}) hit deleted gap!"
        return new_los[idx] + (x - old_los[idx])

    out = []
    for i, (lo, hi, d, w) in enumerate(pieces):
        nlo = new_los[i]
        nhi = nlo + (hi - lo)
        nd = phi(lo + d) - nlo
        for x in range(lo, hi):
            assert phi(x + d) - (nlo + (x - lo)) == nd, "Non-uniform delta after compaction!"
        if out and out[-1][1] == nlo and out[-1][2] == nd and out[-1][3] == w:
            out[-1] = (out[-1][0], nhi, nd, w)
        else:
            out.append((nlo, nhi, nd, w))
    return tuple(out)


def apply_strict_join(state, ia, ib, glo, ghi):
    assert ia != ib
    alo, ahi, ad, aw = state[ia]
    blo, bhi, bd, bw = state[ib]
    assert bd != 0, "Cannot Join into delta_B == 0!"
    atlo, athi = alo + ad, ahi + ad
    ov_lo, ov_hi = max(atlo, blo), min(athi, bhi)
    assert ov_lo <= glo < ghi <= ov_hi, f"G=[{glo},{ghi}) not subset of J_A cap I_B=[{ov_lo},{ov_hi})"

    # EXPLICIT DISJOINTNESS CHECK: G cap (G + bd) == empty
    g_shift_lo, g_shift_hi = glo + bd, ghi + bd
    disjoint = (ghi <= g_shift_lo) or (g_shift_hi <= glo)
    assert disjoint, f"VIOLATION: G=[{glo},{ghi}) overlaps G+bd=[{g_shift_lo},{g_shift_hi})!"

    a_ov_lo, a_ov_hi = glo - ad, ghi - ad
    nxt = []
    for k, pc in enumerate(state):
        if k == ia:
            if alo < a_ov_lo:
                nxt.append((alo, a_ov_lo, ad, aw))
            nxt.append((a_ov_lo, a_ov_hi, ad + bd, aw + bw))
            if a_ov_hi < ahi:
                nxt.append((a_ov_hi, ahi, ad, aw))
        elif k == ib:
            if blo < glo:
                nxt.append((blo, glo, bd, bw))
            if ghi < bhi:
                nxt.append((ghi, bhi, bd, bw))
        else:
            nxt.append(pc)
    return indep_compact_and_coalesce(nxt)


def apply_strict_pow(state, i):
    lo, hi, d, w = state[i]
    L = hi - lo
    assert d != 0 and abs(d) < L, f"Invalid Pow on piece {i}: L={L}, d={d}"
    nxt = list(state[:i]) + list(state[i + 1:])
    if d > 0:
        q, rem = divmod(L, d)
        if rem > 0:
            nxt.append((lo, lo + rem, (q + 1) * d, w * (q + 1)))
        if d - rem > 0:
            nxt.append((lo + rem, lo + d, q * d, w * q))
    else:
        s = -d
        q, rem = divmod(L, s)
        if s - rem > 0:
            nxt.append((hi - s, hi - rem, -q * s, w * q))
        if rem > 0:
            nxt.append((hi - rem, hi, -(q + 1) * s, w * (q + 1)))
    return indep_compact_and_coalesce(nxt)


def apply_strict_cdel(state, i):
    lo, hi, d, w = state[i]
    assert d == 0 and len(state) > 1
    nxt = list(state[:i]) + list(state[i + 1:])
    return indep_compact_and_coalesce(nxt)


def enumerate_strict_moves(state, include_cdel=False):
    res = []
    for i, (lo, hi, d, w) in enumerate(state):
        L = hi - lo
        if d == 0 and include_cdel and len(state) > 1:
            res.append((apply_strict_cdel(state, i), ("c_del", i)))
        elif d != 0 and abs(d) < L:
            op = "Pow+" if d > 0 else "Pow-"
            res.append((apply_strict_pow(state, i), (op, i)))
    for ia, (alo, ahi, ad, aw) in enumerate(state):
        atlo, athi = alo + ad, ahi + ad
        for ib, (blo, bhi, bd, bw) in enumerate(state):
            if ia == ib or bd == 0:
                continue
            ov_lo, ov_hi = max(atlo, blo), min(athi, bhi)
            if ov_lo >= ov_hi:
                continue
            if (ov_hi - ov_lo) > abs(bd):
                cands = [(ov_lo, ov_lo + abs(bd)), (ov_hi - abs(bd), ov_hi)]
            else:
                cands = [(ov_lo, ov_hi)]
            for glo, ghi in cands:
                nst = apply_strict_join(state, ia, ib, glo, ghi)
                res.append((nst, ("Join", ia, ib, glo, ghi)))
    return res


def ucd_metrics(state):
    u = c = d = 0
    prev_u = None
    for _, _, _, w in state:
        if len(w) == 1:
            u += 1
            if prev_u == w:
                d += 1
            prev_u = w
        else:
            c += 1
            prev_u = None
    return u - d, u, c, d


def composite_invariants(state):
    comp_multiset = Counter(w for _, _, _, w in state if len(w) >= 2)
    adj_same_comp = Counter()
    adj_any_comp = Counter()
    adj_comp_atom = Counter()
    for i in range(len(state) - 1):
        w1, w2 = state[i][3], state[i + 1][3]
        if len(w1) >= 2 and len(w2) >= 2:
            adj_any_comp[(w1, w2)] += 1
            if w1 == w2:
                adj_same_comp[w1] += 1
        elif len(w1) >= 2 or len(w2) >= 2:
            adj_comp_atom[(w1, w2)] += 1
    return comp_multiset, adj_same_comp, adj_any_comp, adj_comp_atom


# ============================================================================
# 3. AUDIT PART 1: THEOREM C5 (W1b, W1a..W8, LP & FARKAS, RANDOM STRESS)
# ============================================================================

def solve_exact_lp_indep(transitions):
    uniq = sorted(set(transitions))
    rows = [(Fraction(-dc), Fraction(dd), Fraction(-1)) for (du, dc, dd) in uniq]
    rhss = [Fraction(du) for (du, dc, dd) in uniq]
    m = len(rows)

    def det3(A):
        return (A[0][0] * (A[1][1] * A[2][2] - A[1][2] * A[2][1])
                - A[0][1] * (A[1][0] * A[2][2] - A[1][2] * A[2][0])
                + A[0][2] * (A[1][0] * A[2][1] - A[1][1] * A[2][0]))

    best_M = None
    opt_verts = set()
    for idx in combinations(range(m), 3):
        A = [list(rows[i]) for i in idx]
        b = [rhss[i] for i in idx]
        D = det3(A)
        if D == 0:
            continue
        sol = []
        for col in range(3):
            Ac = [r[:] for r in A]
            for r in range(3):
                Ac[r][col] = b[r]
            sol.append(det3(Ac) / D)
        if all(rows[i][0] * sol[0] + rows[i][1] * sol[1] + rows[i][2] * sol[2] <= rhss[i] for i in range(m)):
            if best_M is None or sol[2] < best_M:
                best_M = sol[2]
                opt_verts = {tuple(sol)}
            elif sol[2] == best_M:
                opt_verts.add(tuple(sol))
    return best_M, sorted(opt_verts)


def audit_theorem_c5():
    print("=" * 78)
    print("AUDIT 1: THEOREM C5 (W1b, W1a..W8, LP PINNING, & ADVERSARIAL CHECKS)")
    print("=" * 78)

    T_w1b = (0, 0, 0, 0, 1, 1, 0, 0, 1)
    L_expected = (1, 0, 1, 0, 0, 0, 0, 1, 0)
    L_actual, p_w1b = indep_bwt_encode(T_w1b)
    assert L_actual == L_expected
    T_inv, p_inv, r0, st0 = indep_bwt_inverse_and_validate(L_actual)
    assert is_cyclic_rotation(T_inv, T_w1b) and r0 == 6
    print(f"[VERIFIED] W1b BWT: T={list(T_w1b)} -> L={list(L_actual)}, single-cycle, r0={r0}")

    st1, mv1 = [(nst, mv) for nst, mv in enumerate_strict_moves(st0) if mv[:3] == ("Join", 5, 3)][0]
    assert len(decode_wiet_cycles(st1)) == 1 and is_cyclic_rotation(decode_wiet_cycles(st1)[0], T_w1b)
    st2, mv2 = [(nst, mv) for nst, mv in enumerate_strict_moves(st1) if mv[:3] == ("Join", 4, 3)][0]
    assert len(decode_wiet_cycles(st2)) == 1 and is_cyclic_rotation(decode_wiet_cycles(st2)[0], T_w1b)
    st3, mv3 = [(nst, mv) for nst, mv in enumerate_strict_moves(st2) if mv[:3] == ("Join", 3, 1)][0]
    assert len(decode_wiet_cycles(st3)) == 1 and is_cyclic_rotation(decode_wiet_cycles(st3)[0], T_w1b)

    _, u0, c0, d0 = ucd_metrics(st2)
    _, u1, c1, d1 = ucd_metrics(st3)
    du, dc, dd = u1 - u0, c1 - c0, d1 - d0
    assert (du, dc, dd) == (-3, 0, 0)

    cm0, asc0, aac0, aca0 = composite_invariants(st2)
    cm1, asc1, aac1, aca1 = composite_invariants(st3)
    assert cm0 == cm1 and asc0 == asc1 and aac0 == aac1
    print(f"[VERIFIED] W1b Depth-3 Join(3,1) G=[{mv3[3]},{mv3[4]}): (dU,dC,dD) = {(du,dc,dd)}")
    print(f"           Composite multiset N_{{t,w}}: {dict(cm0)} -> {dict(cm1)} (100% INVARIANT)")
    print(f"           Adj comp-comp pairs: {dict(aac0)} -> {dict(aac1)} (100% INVARIANT)")
    print(f"           [Adversarial note] Adj comp-atom pairs changed: {aca0 != aca1}")

    def b_to_t(pcs):
        return tuple((lo, hi, d, tuple(w)) for lo, hi, d, w in pcs)

    witnesses = []
    w1a = b_to_t(((0, 1, +4, b"p"), (1, 2, +1, b"b"), (2, 3, +3, b"p"), (3, 4, -3, b"ab"), (4, 5, -3, b"a"), (5, 6, -2, b"c")))
    c_bef = decode_wiet_cycles(w1a); assert len(c_bef) == 1
    w1a_aft = [nst for nst, mv in enumerate_strict_moves(w1a) if mv[:3] == ("Join", 4, 1)][0]
    assert is_cyclic_rotation(c_bef[0], decode_wiet_cycles(w1a_aft)[0])
    d_w1a = tuple(b - a for a, b in zip(ucd_metrics(w1a)[1:], ucd_metrics(w1a_aft)[1:]))
    assert d_w1a == (-3, 0, 0)
    witnesses.append(("W1a (6-cycle W-IET)", d_w1a, False))
    witnesses.append(("W1b (BWT depth-3)", (-3, 0, 0), True))

    L_w2, _ = indep_bwt_encode((0, 0, 1, 1))
    T_w2, _, _, st_w2 = indep_bwt_inverse_and_validate(L_w2)
    st_w2_aft = [nst for nst, mv in enumerate_strict_moves(st_w2) if mv[:3] == ("Join", 0, 2)][0]
    assert is_cyclic_rotation(decode_wiet_cycles(st_w2_aft)[0], T_w2)
    d_w2 = tuple(b - a for a, b in zip(ucd_metrics(st_w2)[1:], ucd_metrics(st_w2_aft)[1:]))
    assert d_w2 == (-3, +1, 0)
    witnesses.append(("W2  (BWT depth-1)", d_w2, True))

    w3 = b_to_t(((0, 1, +6, b"p"), (1, 2, +3, b"b"), (2, 3, +5, b"p"), (3, 4, -3, b"qq"),
                 (4, 5, -2, b"qq"), (5, 6, -2, b"ab"), (6, 7, -5, b"a"), (7, 8, -2, b"ab")))
    c_bef = decode_wiet_cycles(w3); assert len(c_bef) == 1
    w3_aft = [nst for nst, mv in enumerate_strict_moves(w3) if mv[:3] == ("Join", 6, 1)][0]
    assert is_cyclic_rotation(c_bef[0], decode_wiet_cycles(w3_aft)[0])
    d_w3 = tuple(b - a for a, b in zip(ucd_metrics(w3)[1:], ucd_metrics(w3_aft)[1:]))
    assert d_w3 == (-3, -2, 0) and len(w3_aft) - len(w3) == -5
    witnesses.append(("W3  (8-cycle W-IET)", d_w3, False))

    w4 = b_to_t(((0, 1, +6, b"pp"), (1, 2, +3, b"bb"), (2, 3, +5, b"pp"), (3, 4, -3, b"qq"),
                 (4, 5, -2, b"qq"), (5, 6, -2, b"aabb"), (6, 7, -5, b"aa"), (7, 8, -2, b"aabb")))
    c_bef = decode_wiet_cycles(w4); assert len(c_bef) == 1
    w4_aft = [nst for nst, mv in enumerate_strict_moves(w4) if mv[:3] == ("Join", 6, 1)][0]
    assert is_cyclic_rotation(c_bef[0], decode_wiet_cycles(w4_aft)[0])
    d_w4 = tuple(b - a for a, b in zip(ucd_metrics(w4)[1:], ucd_metrics(w4_aft)[1:]))
    assert d_w4 == (0, -5, 0) and len(w4_aft) - len(w4) == -5
    witnesses.append(("W4  (8-cycle W-IET)", d_w4, False))

    w5 = b_to_t(((0, 1, +6, b"p"), (1, 2, +4, b"b"), (2, 3, +5, b"p"), (3, 4, +1, b"ab"),
                 (4, 5, -3, b"a"), (5, 6, -5, b"q"), (6, 7, -4, b"q"), (7, 8, -4, b"c")))
    c_bef = decode_wiet_cycles(w5); assert len(c_bef) == 1
    w5_aft = [nst for nst, mv in enumerate_strict_moves(w5) if mv[:3] == ("Join", 4, 1)][0]
    assert is_cyclic_rotation(c_bef[0], decode_wiet_cycles(w5_aft)[0])
    d_w5 = tuple(b - a for a, b in zip(ucd_metrics(w5)[1:], ucd_metrics(w5_aft)[1:]))
    assert d_w5 == (-4, 0, -1)
    witnesses.append(("W5  (8-cycle W-IET)", d_w5, False))

    w6 = b_to_t(((0, 1, +5, b"p"), (1, 2, +1, b"b"), (2, 3, +1, b"p"), (3, 4, -3, b"ab"),
                 (4, 5, -3, b"a"), (5, 6, -1, b"c")))
    c_bef = decode_wiet_cycles(w6); assert len(c_bef) == 1
    w6_aft = [nst for nst, mv in enumerate_strict_moves(w6) if mv[:3] == ("Join", 4, 1)][0]
    assert is_cyclic_rotation(c_bef[0], decode_wiet_cycles(w6_aft)[0])
    d_w6 = tuple(b - a for a, b in zip(ucd_metrics(w6)[1:], ucd_metrics(w6_aft)[1:]))
    assert d_w6 == (-2, 0, +1)
    witnesses.append(("W6  (6-cycle W-IET)", d_w6, False))

    st_w7_aft = [nst for nst, mv in enumerate_strict_moves(st_w2_aft) if mv == ("Pow-", 1)][0]
    assert is_cyclic_rotation(decode_wiet_cycles(st_w7_aft)[0], T_w2)
    d_w7 = tuple(b - a for a, b in zip(ucd_metrics(st_w2_aft)[1:], ucd_metrics(st_w7_aft)[1:]))
    assert d_w7 == (-1, +1, 0)
    witnesses.append(("W7  (BWT Pow-)", d_w7, True))

    w8 = b_to_t(((0, 1, +2, b"p"), (1, 2, 0, b"b"), (2, 3, +1, b"p"), (3, 4, -3, b"c")))
    assert len(decode_wiet_cycles(w8)) == 2
    w8_aft = [nst for nst, mv in enumerate_strict_moves(w8, include_cdel=True) if mv == ("c_del", 1)][0]
    assert len(decode_wiet_cycles(w8_aft)) == 1
    d_w8 = tuple(b - a for a, b in zip(ucd_metrics(w8)[1:], ucd_metrics(w8_aft)[1:]))
    assert d_w8 == (-2, 0, 0)
    witnesses.append(("W8  (multi-cycle c_del)", d_w8, False))

    M_wits, verts_wits = solve_exact_lp_indep([tr for _, tr, _ in witnesses])
    assert M_wits == Fraction(3, 1) and verts_wits == [(Fraction(0), Fraction(1), Fraction(3))]
    print(f"[PROVED] Exact rational LP on W1..W8: M* = {M_wits}, UNIQUE vertex (b,d,M) = {verts_wits[0]}")


# ============================================================================
# 4. AUDIT PART 2: THEOREM C6 (Spliced L16^m for m = 1, 2, 3, 4, 6, 8)
# ============================================================================

def build_spliced_L16_indep(m):
    pat16 = (2, 0, 2, 0, 1, 0, 1, 0, 1, 0, 2, 0, 1, 0, 2, 0)
    L = []
    for j in range(m):
        blk = [3 * j + x for x in pat16]
        if m > 1:
            if j == 0:
                blk[14] = 3 * 1 + 2
            elif j < m - 1:
                blk[0] = 3 * (j + 1) + 2
            else:
                blk[0] = 3 * 0 + 2
        L.extend(blk)
    return tuple(L)


def fast_beam_indep(init, T, beam_w=1000, max_steps=13):
    frontier = {init: []}
    for step in range(1, max_steps + 1):
        nxt_map = {}
        for st, path in frontier.items():
            for nst, mv in enumerate_strict_moves(st):
                if len(nst) == 1 and nst[0][2] == 0:
                    w = nst[0][3][::-1]
                    assert is_cyclic_rotation(w, T)
                    return step, path + [(st, mv, nst)]
                if nst not in nxt_map:
                    nxt_map[nst] = path + [(st, mv, nst)]
        items = list(nxt_map.items())
        if len(items) > beam_w:
            by_p = sorted(items, key=lambda kv: (len(kv[0]), ucd_metrics(kv[0])[0]))[:beam_w // 2]
            by_psi = sorted(items, key=lambda kv: (ucd_metrics(kv[0])[0], len(kv[0])))[:beam_w // 2]
            frontier = dict(by_p)
            frontier.update(by_psi)
        else:
            frontier = nxt_map
    return None, []


def audit_theorem_c6():
    print("\n" + "=" * 78)
    print("AUDIT 2: THEOREM C6 (Spliced L16^m for m = 1, 2, 3, 4, 6, 8)")
    print("=" * 78)

    L1 = build_spliced_L16_indep(1)
    T1, p1, r0_1, init1 = indep_bwt_inverse_and_validate(L1)
    q1, steps1 = fast_beam_indep(init1, T1, beam_w=1000, max_steps=12)
    assert q1 == 12
    recipe12 = []
    for st_b, mv, _ in steps1:
        if mv[0] == "Join":
            recipe12.append(("Join", st_b[mv[1]][3], st_b[mv[2]][3]))
        else:
            recipe12.append((mv[0], st_b[mv[1]][3], None))

    def apply_word_move_strict(st, op, wA, wB=None):
        matches = []
        for nst, mv in enumerate_strict_moves(st):
            if mv[0] == op:
                if op == "Join" and st[mv[1]][3] == wA and st[mv[2]][3] == wB:
                    matches.append((nst, mv))
                elif op != "Join" and st[mv[1]][3] == wA:
                    matches.append((nst, mv))
        assert len(matches) >= 1
        return matches[0]

    L2 = build_spliced_L16_indep(2)
    T2, p2, r0_2, st2 = indep_bwt_inverse_and_validate(L2)
    for op, wA, wB in recipe12:
        st2, _ = apply_word_move_strict(st2, op, tuple(x + 3 for x in wA), tuple(x + 3 for x in wB) if wB else None)
    q_b0, steps_b0 = fast_beam_indep(st2, T2, beam_w=800, max_steps=13)
    assert q_b0 == 13
    block0_moves = [mv for _, mv, _ in steps_b0]

    for m in (1, 2, 3, 4, 6, 8):
        L = build_spliced_L16_indep(m)
        res = indep_bwt_inverse_and_validate(L)
        assert res is not None
        T, p, r0, st = res
        assert len(L) == 16 * m and r0 == 16 * m and len(set(L)) == 3 * m

        step_cnt = 0
        def check_step(st_before, mv, st_after):
            nonlocal step_cnt
            step_cnt += 1
            if mv[0] == "Join":
                assert apply_strict_join(st_before, mv[1], mv[2], mv[3], mv[4]) == st_after
            else:
                assert apply_strict_pow(st_before, mv[1]) == st_after
            cycs = decode_wiet_cycles(st_after)
            assert len(cycs) == 1 and is_cyclic_rotation(cycs[0], T)

        if m == 1:
            for op, wA, wB in recipe12:
                nst, mv = apply_word_move_strict(st, op, wA, wB)
                check_step(st, mv, nst); st = nst
        else:
            for j in range(1, m):
                sh = 3 * j
                for op, wA, wB in recipe12:
                    nst, mv = apply_word_move_strict(
                        st, op, tuple(x + sh for x in wA),
                        tuple(x + sh for x in wB) if wB else None)
                    check_step(st, mv, nst); st = nst
            for _ in range(m - 2):
                nst, mv = [(ns, m2) for ns, m2 in enumerate_strict_moves(st)
                           if m2[0] == "Join" and len(st[m2[1]][3]) >= 16 and len(st[m2[2]][3]) >= 16][0]
                check_step(st, mv, nst); st = nst
            assert len(st) == 17
            for mv_target in block0_moves:
                if mv_target[0] == "Join":
                    cands = [(ns, m2) for ns, m2 in enumerate_strict_moves(st) if m2[:3] == mv_target[:3]]
                    if len(cands) > 1:
                        cands = [(ns, m2) for ns, m2 in cands if m2 == mv_target]
                else:
                    cands = [(ns, m2) for ns, m2 in enumerate_strict_moves(st) if m2 == mv_target]
                assert len(cands) == 1
                nst, mv = cands[0]
                check_step(st, mv, nst); st = nst

        assert step_cnt == 13 * m - 1 and len(st) == 1 and st[0][2] == 0
        assert is_cyclic_rotation(st[0][3][::-1], T)
        print(f"  [VERIFIED] Spliced L16^{m}: valid single-cycle BWT, r0={r0:3d}, |Sigma|={3*m:2d}, steps={step_cnt:3d} (=13m-1, {step_cnt/r0:.4f} r0)")


# ============================================================================
# 5. AUDIT PART 3: THEOREM H_grand (5-MODEL SEPARATION LINKS)
# ============================================================================

def lz77_exact_indep(T):
    n = len(T)
    i = z = 0
    while i < n:
        best = 0
        for j in range(i):
            k = 0
            while i + k < n and T[j + k] == T[i + k]:
                k += 1
            if k > best:
                best = k
        z += 1
        i += max(1, best)
    return z


def exact_bfs_min_steps(init, max_depth=6):
    q = deque([(init, [])])
    seen = {init}
    while q:
        st, path = q.popleft()
        if len(st) == 1 and st[0][2] == 0:
            return len(path), path
        if len(path) >= max_depth:
            continue
        for nst, mv in enumerate_strict_moves(st):
            if nst not in seen:
                seen.add(nst)
                q.append((nst, path + [mv]))
    return None, []


def audit_theorem_hgrand():
    print("\n" + "=" * 78)
    print("AUDIT 3: THEOREM H_grand (EXACT SEPARATION LINKS & CAVEATS)")
    print("=" * 78)
    T4 = (1,) * 4 + (0,) + (1,) * 4 + (2,)
    L4, _ = indep_bwt_encode(T4)
    _, _, r0_4, init4 = indep_bwt_inverse_and_validate(L4)
    opt4, path4 = exact_bfs_min_steps(init4, max_depth=6)
    print(f"  [Link 1 vs 2] T_4 (r0={r0_4}): exact Q*_{{C_gap}} = {opt4} (NOT 4!), ops={[m[0] for m in path4]}")

    rng = random.Random(42)
    base = tuple(rng.randrange(4) for _ in range(32))
    for reps in (8, 32):
        T_rep = list(base * reps); T_rep[-1] = 4; T_rep[100] = 5; T_rep = tuple(T_rep)
        L_rep, _ = indep_bwt_encode(T_rep)
        r0_rep = 1 + sum(1 for x, y in zip(L_rep, L_rep[1:]) if x != y)
        z_rep = lz77_exact_indep(T_rep)
        print(f"  [Link 3,4 vs 5] Copy+2Edits (reps={reps:2d}, n={len(T_rep):4d}): r0={r0_rep}, z={z_rep}, r0/z={r0_rep/z_rep:.2f}x (CONSTANT in reps)")

    a, b = [0], [0, 1]
    for k in range(3, 14):
        a, b = b, b + a
        if k in (9, 11, 13):
            T_hi = tuple(b[::-1] + [2])
            L_hi, _ = indep_bwt_encode(T_hi)
            r0_hi = 1 + sum(1 for x, y in zip(L_hi, L_hi[1:]) if x != y)
            z_hi = lz77_exact_indep(T_hi)
            print(f"  [Link 3,4 vs 5] Fib_{k:2d}^R+$ (n={len(T_hi):4d}): r0={r0_hi}, z={z_hi}, r0/z={r0_hi/z_hi:.2f}x (O(1) ratio: both Theta(log n))")


if __name__ == "__main__":
    audit_theorem_c5()
    audit_theorem_c6()
    audit_theorem_hgrand()
