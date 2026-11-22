#!/usr/bin/env python3
"""
Cycle 17 — Investigator Alpha-3:
  PART 1 (Theorem U_cat [proved] + [verified]):
    Grand Algebraic Unification of ALL 12 BWT Contraction Frameworks inside the
    Traced Symmetric Monoidal Category WPB (Word-Carrying Partial Bijections):
      - Mechanical verification of ALL 6 Joyal-Street-Verity traced monoidal axioms
        (Tightening, Sliding, Vanishing I & II, Superposing, Yanking) & Exact
        Order-Independence / Confluence: Tr_{E2}(Tr_{E1}(Phi, W)) == Tr_{E1 u E2}(Phi, W).
      - Mechanical verification of ALL 12 frameworks against canonical Tr_E(LF, L)
        (exact element-by-element reduced permutation + edge-word equality AND exact
        decoded cyclic text T) across W1b (n=9), T_4 (n=10), L^(4) (n=80), and
        20 random valid single-cycle BWTs (23 x 12 = 276 checks, 0 discrepancies!).

  PART 2 ([bounded-opt] Exact Binary BWT Frontier inside c*_{|Sigma|=2} in [1/2, 19/24]):
    Exhaustive exact Q*_{C_gap}(L) via C-accelerated bucketed A* with Theorem C7
    admissible heuristic h_C7(S) = ceil((Xi(S) - 1)/2) + exact m<=4 terminal theorem:
      - Cross-checked 100% against independent pure-Python A* on ALL n=2..10 (224 BWTs),
        complement symmetry verified on ALL n=2..11 (410 BWTs), proper sub-interval
        joins checked on ALL n=2..8 (69 BWTs).
      - Exhaustive exact Q* over ALL 4,718 valid single-cycle binary BWTs for n=2..15
        (0 unsolved!), plus targeted high-coalescing search over n=16..24, r_0 <= 16.
      - Pure-Python replay & byte-exact decode verification of every extremal witness.
"""
import bisect
import ctypes
import hashlib
import heapq
import os
import random
import subprocess
import sys
import time
from collections import defaultdict
from fractions import Fraction


# ==============================================================================
# BASIC BWT UTILITIES (SELF-CONTAINED, ZERO EXTERNAL IMPORTS)
# ==============================================================================
def bwt_encode(T: bytes):
    n = len(T)
    sa = list(range(n))
    rk = list(T)
    k = 1
    while k < n:
        sa.sort(key=lambda i: (rk[i], rk[(i + k) % n]))
        tmp = [0] * n
        c = 0
        tmp[sa[0]] = 0
        for i in range(1, n):
            p0, p1 = sa[i - 1], sa[i]
            if rk[p0] != rk[p1] or rk[(p0 + k) % n] != rk[(p1 + k) % n]:
                c += 1
            tmp[p1] = c
        rk = tmp
        if c == n - 1:
            break
        k <<= 1
    L = bytes(T[(i - 1) % n] for i in sa)
    return L, sa.index(0)


def build_lf(L: bytes):
    n = len(L)
    cnt = [0] * 256
    for b in L:
        cnt[b] += 1
    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += cnt[c]
    cur = C[:]
    lf = [0] * n
    fl = [0] * n
    for i, b in enumerate(L):
        t = cur[b]
        lf[i] = t
        fl[t] = i
        cur[b] += 1
    F = bytes(sorted(L))
    return lf, fl, C, F


def ibwt_single_cycle(L: bytes):
    """If L is a valid single-cycle BWT, return (T, p=0_anchor_text, lf); else None."""
    n = len(L)
    lf, _, _, _ = build_lf(L)
    x = 0
    out = bytearray(n)
    for k in range(n - 1, -1, -1):
        out[k] = L[x]
        x = lf[x]
    if x != 0:
        return None
    T = bytes(out)
    L_rt, p = bwt_encode(T)
    if L_rt != L:
        return None
    return T, p, lf


def runs_of(seq) -> int:
    if not seq:
        return 0
    return 1 + sum(1 for a, b in zip(seq, seq[1:]) if a != b)


def is_cyclic_rotation(a: bytes, b: bytes) -> bool:
    return len(a) == len(b) and a in (b + b)


def build_Ld(d: int) -> bytes:
    n = (3 ** d) - 1
    k = n // 2
    arr = bytearray([1, 0] * k)
    for e in range(1, d):
        ye = (3 ** d) - (3 ** e) - 2
        arr[ye], arr[ye + 1] = 0, 1
    return bytes(arr)


def gen_lyndon_binary(n: int):
    w = [-1]
    while w:
        w[-1] += 1
        m = len(w)
        if m == n:
            yield tuple(w)
        while len(w) < n:
            w.append(w[-m])
        while w and w[-1] == 1:
            w.pop()


# ==============================================================================
# PART 1: TRACED SYMMETRIC MONOIDAL CATEGORY WPB & 12-FRAMEWORK UNIFICATION
# ==============================================================================
class WPBMorphism:
    """
    Morphism f = (Phi, W): X -> Y in the Traced Symmetric Monoidal Category WPB:
      - dom_set, cod_set: frozensets of hashable vertices (finite ordered sets when ints)
      - mapping: dict[x -> (y, word_bytes)] where Phi is injective (partial bijection)
    """
    def __init__(self, dom_set, cod_set, mapping):
        self.dom_set = frozenset(dom_set)
        self.cod_set = frozenset(cod_set)
        self.mapping = dict(mapping)
        seen_y = set()
        for x, (y, w) in self.mapping.items():
            assert x in self.dom_set, f"{x} not in dom_set"
            assert y in self.cod_set, f"{y} not in cod_set"
            assert y not in seen_y, f"Non-injective Phi: duplicate target {y}"
            assert isinstance(w, bytes)
            seen_y.add(y)

    def __eq__(self, other):
        return (isinstance(other, WPBMorphism)
                and self.dom_set == other.dom_set
                and self.cod_set == other.cod_set
                and self.mapping == other.mapping)

    @staticmethod
    def identity(X):
        X = frozenset(X)
        return WPBMorphism(X, X, {x: (x, b"") for x in X})

    @staticmethod
    def from_bwt(L: bytes):
        n = len(L)
        lf, _, _, _ = build_lf(L)
        V = frozenset(range(n))
        return WPBMorphism(V, V, {i: (lf[i], bytes([L[i]])) for i in range(n)})

    @staticmethod
    def from_pieces(pieces):
        """Convert disjoint interval pieces (lo, hi, d, w_bytes) on ordered subset of Z to WPBMorphism."""
        dom = set()
        cod = set()
        mp = {}
        for lo, hi, d, w in pieces:
            for x in range(lo, hi):
                y = x + d
                dom.add(x)
                cod.add(y)
                mp[x] = (y, w)
        return WPBMorphism(dom, cod, mp)

    def compose(self, other: "WPBMorphism") -> "WPBMorphism":
        """Categorical composition self o other (first apply `other`, then `self`)."""
        assert other.cod_set == self.dom_set
        mp = {}
        for x, (y, w1) in other.mapping.items():
            if y in self.mapping:
                z, w2 = self.mapping[y]
                mp[x] = (z, w1 + w2)
        return WPBMorphism(other.dom_set, self.cod_set, mp)

    @staticmethod
    def oplus(f1: "WPBMorphism", f2: "WPBMorphism") -> "WPBMorphism":
        """Monoidal product f1 (+) f2 using tagged disjoint union (0, x) and (1, x)."""
        dom = {(0, x) for x in f1.dom_set} | {(1, x) for x in f2.dom_set}
        cod = {(0, y) for y in f1.cod_set} | {(1, y) for y in f2.cod_set}
        mp = {}
        for x, (y, w) in f1.mapping.items():
            mp[(0, x)] = ((0, y), w)
        for x, (y, w) in f2.mapping.items():
            mp[(1, x)] = ((1, y), w)
        return WPBMorphism(dom, cod, mp)

    @staticmethod
    def symmetry(X, Y) -> "WPBMorphism":
        X = frozenset(X)
        Y = frozenset(Y)
        dom = {(0, x) for x in X} | {(1, y) for y in Y}
        cod = {(0, y) for y in Y} | {(1, x) for x in X}
        mp = {}
        for x in X:
            mp[(0, x)] = ((1, x), b"")
        for y in Y:
            mp[(1, y)] = ((0, y), b"")
        return WPBMorphism(dom, cod, mp)

    def trace_tagged(self, X, Y, U) -> "WPBMorphism":
        """
        Canonical JSV particle-style feedback trace Tr^U_{X,Y}(f): X -> Y
        for f: (0,X) u (1,U) -> (0,Y) u (1,U).
        By injectivity of Phi, every orbit entering U from X is strictly acyclic!
        """
        X = frozenset(X); Y = frozenset(Y); U = frozenset(U)
        mp = {}
        for x in X:
            cur = (0, x)
            word = bytearray()
            steps = 0
            while cur in self.mapping:
                nxt, w = self.mapping[cur]
                word.extend(w)
                if nxt[0] == 0:
                    mp[x] = (nxt[1], bytes(word))
                    break
                cur = nxt
                steps += 1
                assert steps <= len(U) + 1, "Cycle entered from X — violates injectivity!"
        return WPBMorphism(X, Y, mp)

    def trace_subset(self, E) -> "WPBMorphism":
        """
        Endomorphism trace Tr_E(Phi, W) on K = V \\ E for f: V -> V (E subseteq V).
        """
        assert self.dom_set == self.cod_set
        E = frozenset(E)
        assert E.issubset(self.dom_set)
        K = self.dom_set - E
        mp = {}
        for x in K:
            cur = x
            word = bytearray()
            steps = 0
            while cur in self.mapping:
                nxt, w = self.mapping[cur]
                word.extend(w)
                if nxt in K:
                    mp[x] = (nxt, bytes(word))
                    break
                cur = nxt
                steps += 1
                assert steps <= len(E) + 1
        return WPBMorphism(K, K, mp)

    def compact_order(self) -> "WPBMorphism":
        """Gap-closing order-isomorphism conjugation phi_K o f o phi_K^{-1} onto [0, |K|)."""
        assert self.dom_set == self.cod_set
        ks = sorted(self.dom_set)
        pos = {v: i for i, v in enumerate(ks)}
        V_new = frozenset(range(len(ks)))
        mp = {pos[x]: (pos[y], w) for x, (y, w) in self.mapping.items()}
        return WPBMorphism(V_new, V_new, mp)

    def decode_single_cycle(self) -> bytes:
        """Walk the single cycle from min(V) and reverse concatenated words to obtain T."""
        assert self.dom_set == self.cod_set and len(self.dom_set) >= 1
        start = min(self.dom_set)
        cur = start
        buf = bytearray()
        vis = 0
        while True:
            nxt, w = self.mapping[cur]
            buf.extend(w)
            cur = nxt
            vis += 1
            if cur == start:
                break
            assert vis <= len(self.dom_set)
        assert vis == len(self.dom_set), f"Not a single cycle! vis={vis} vs |V|={len(self.dom_set)}"
        return bytes(buf[::-1])


def verify_jsv_axioms():
    """Mechanically verify ALL 6 JSV axioms of WPB on random partial bijections + Yanking."""
    rng = random.Random(17001)
    alphabet = [b"a", b"b", b"c", b"ab", b"bc", b""]

    def rand_pb(X, Y, density=0.75):
        xs = list(X); ys = list(Y)
        rng.shuffle(xs); rng.shuffle(ys)
        k = int(round(min(len(xs), len(ys)) * density))
        mp = {xs[i]: (ys[i], rng.choice(alphabet)) for i in range(k)}
        return WPBMorphism(X, Y, mp)

    checks = 0
    for trial in range(40):
        X = frozenset(range(rng.randint(2, 5)))
        Xp = frozenset(range(rng.randint(2, 5)))
        Y = frozenset(range(rng.randint(2, 5)))
        Yp = frozenset(range(rng.randint(2, 5)))
        U = frozenset(range(rng.randint(2, 5)))
        V = frozenset(range(rng.randint(2, 5)))

        # 1. Tightening: Tr^U((h (+) id_U) o f o (g (+) id_U)) == h o Tr^U(f) o g
        dom_f = {(0, x) for x in X} | {(1, u) for u in U}
        cod_f = {(0, y) for y in Y} | {(1, u) for u in U}
        f = rand_pb(dom_f, cod_f)
        g = rand_pb(Xp, X)
        h = rand_pb(Y, Yp)
        idU = WPBMorphism.identity(U)
        lhs = WPBMorphism.oplus(h, idU).compose(f).compose(WPBMorphism.oplus(g, idU)).trace_tagged(Xp, Yp, U)
        rhs = h.compose(f.trace_tagged(X, Y, U)).compose(g)
        assert lhs == rhs, "Tightening failed!"

        # 2. Sliding: Tr^U((id_Y (+) s) o f_uv) == Tr^V(f_uv o (id_X (+) s))
        dom_fuv = {(0, x) for x in X} | {(1, u) for u in U}
        cod_fuv = {(0, y) for y in Y} | {(1, v) for v in V}
        f_uv = rand_pb(dom_fuv, cod_fuv)
        s = rand_pb(V, U)
        lhs2 = WPBMorphism.oplus(WPBMorphism.identity(Y), s).compose(f_uv).trace_tagged(X, Y, U)
        rhs2 = f_uv.compose(WPBMorphism.oplus(WPBMorphism.identity(X), s)).trace_tagged(X, Y, V)
        assert lhs2 == rhs2, "Sliding failed!"

        # 3. Vanishing I: Tr^{empty}(f_0) == f_0 (modulo tag stripping)
        f0 = rand_pb(X, Y)
        f0_tag = WPBMorphism({(0, x) for x in X}, {(0, y) for y in Y},
                             {(0, x): ((0, y), w) for x, (y, w) in f0.mapping.items()})
        assert f0_tag.trace_tagged(X, Y, frozenset()) == f0, "Vanishing I failed!"

        # 4. Vanishing II & Order-Independence on endomorphism over V_big = K u E1 u E2:
        n_big = rng.randint(6, 12)
        V_big = frozenset(range(n_big))
        f_big = rand_pb(V_big, V_big, density=0.85)
        verts = list(V_big); rng.shuffle(verts)
        k1 = rng.randint(1, n_big // 3)
        k2 = rng.randint(1, n_big // 3)
        E1 = frozenset(verts[:k1])
        E2 = frozenset(verts[k1:k1 + k2])
        t12 = f_big.trace_subset(E1).trace_subset(E2)
        t21 = f_big.trace_subset(E2).trace_subset(E1)
        t_u = f_big.trace_subset(E1 | E2)
        assert t12 == t_u and t21 == t_u, "Vanishing II / Order-Independence failed!"

        # 5. Superposing: Tr^U(g_sp (+) f) == g_sp (+) Tr^U(f) (checking tagged equivalence)
        Z1 = frozenset(range(rng.randint(1, 3)))
        Z2 = frozenset(range(rng.randint(1, 3)))
        g_sp = rand_pb(Z1, Z2)
        # Build (Z1 u X) (+) U -> (Z2 u Y) (+) U
        ZX = {("z", z) for z in Z1} | {("x", x) for x in X}
        ZY = {("z", z) for z in Z2} | {("y", y) for y in Y}
        dom_sp = {(0, p) for p in ZX} | {(1, u) for u in U}
        cod_sp = {(0, q) for q in ZY} | {(1, u) for u in U}
        mp_sp = {}
        for z, (z2, w) in g_sp.mapping.items():
            mp_sp[(0, ("z", z))] = ((0, ("z", z2)), w)
        for (tg, v0), ((tg2, v1), w) in f.mapping.items():
            src = (0, ("x", v0)) if tg == 0 else (1, v0)
            dst = (0, ("y", v1)) if tg2 == 0 else (1, v1)
            mp_sp[src] = (dst, w)
        lhs5 = WPBMorphism(dom_sp, cod_sp, mp_sp).trace_tagged(ZX, ZY, U)
        tr_f = f.trace_tagged(X, Y, U)
        mp_r5 = {}
        for z, (z2, w) in g_sp.mapping.items():
            mp_r5[("z", z)] = (("z", z2), w)
        for x, (y, w) in tr_f.mapping.items():
            mp_r5[("x", x)] = (("y", y), w)
        rhs5 = WPBMorphism(ZX, ZY, mp_r5)
        assert lhs5 == rhs5, "Superposing failed!"

        # 6. Yanking: Tr^X(sigma_{X,X}) == id_X
        sig = WPBMorphism.symmetry(X, X)
        assert sig.trace_tagged(X, X, X) == WPBMorphism.identity(X), "Yanking failed!"
        checks += 6
    return checks


# --- Helpers for the 12 Frameworks ---
def bwt_initial_pieces(L: bytes, isolate_p: int = -1):
    """Build maximal-run W-IET pieces (lo, hi, delta, word_bytes) from L (optionally isolating [p, p+1))."""
    n = len(L)
    lf, _, _, _ = build_lf(L)
    pieces = []
    i = 0
    while i < n:
        if i == isolate_p:
            pieces.append((i, i + 1, lf[i] - i, bytes([L[i]])))
            i += 1
            continue
        j = i + 1
        while j < n and L[j] == L[i] and j != isolate_p:
            j += 1
        pieces.append((i, j, lf[i] - i, bytes([L[i]])))
        i = j
    return pieces


def wiet_eliminate_subset(pieces, elim_set):
    """
    Monotone run-splitting W-IET elimination of subset `elim_set` (with Euclidean Pow spirals
    on self-overlapping pieces inside elim_set), returning pieces on V \\ elim_set WITHOUT gap closing.
    """
    # Split pieces at boundaries of elim_set so every piece is either fully in elim_set or disjoint
    work = []
    for lo, hi, d, w in pieces:
        cur = lo
        while cur < hi:
            in_e = cur in elim_set
            nxt = cur + 1
            while nxt < hi and ((nxt in elim_set) == in_e):
                nxt += 1
            work.append((cur, nxt, d, w))
            cur = nxt

    keep = [p for p in work if p[0] not in elim_set]
    elim = [p for p in work if p[0] in elim_set]
    elim.sort(key=lambda x: x[0])

    # Collapse any self-overlapping eliminated pieces via Pow (Euclidean spiral)
    # Wait: if an eliminated piece E_i has J cap I != empty, Pow collapses it in O(1)
    # and replaces it with non-self-overlapping pieces whose targets exit I!
    # General safe W-IET composition: iteratively route `keep` pieces through `elim`
    # (first collapsing self-loops inside `elim` by Pow!).
    e_starts = [p[0] for p in elim]
    resolved = []
    stack = list(keep)
    while stack:
        lo, hi, d, w = stack.pop()
        tlo, thi = lo + d, hi + d
        if not elim:
            resolved.append((lo, hi, d, w))
            continue
        # Intersect [tlo, thi) with eliminated intervals
        idx = max(0, bisect.bisect_right(e_starts, tlo) - 1)
        pos = tlo
        while pos < thi:
            if idx >= len(elim):
                resolved.append((pos - d, thi - d, d, w))
                break
            elo, ehi, ed, ew = elim[idx]
            if ehi <= pos:
                idx += 1
                continue
            if elo >= thi:
                resolved.append((pos - d, thi - d, d, w))
                break
            if pos < elo:
                resolved.append((pos - d, elo - d, d, w))
                pos = elo
            ov_hi = min(thi, ehi)
            # If elim[idx] is self-overlapping (0 < |ed| < ehi - elo), apply Pow jump directly!
            eL = ehi - elo
            if ed > 0 and ed < eL:
                # For each x in [pos, ov_hi), jump q = (ehi - 1 - x)//ed + 1 steps inside elim[idx]!
                sub_pos = pos
                while sub_pos < ov_hi:
                    q = (ehi - 1 - sub_pos) // ed + 1
                    # Max x with same q: ehi - 1 - (q - 1)*ed
                    max_x = min(ov_hi - 1, ehi - 1 - (q - 1) * ed)
                    seg_hi = max_x + 1
                    stack.append((sub_pos - d, seg_hi - d, d + q * ed, w + ew * q))
                    sub_pos = seg_hi
            elif ed < 0 and (-ed) < eL:
                sd = -ed
                sub_pos = pos
                while sub_pos < ov_hi:
                    q = (sub_pos - elo) // sd + 1
                    max_x = min(ov_hi - 1, elo + q * sd - 1)
                    seg_hi = max_x + 1
                    stack.append((sub_pos - d, seg_hi - d, d - q * sd, w + ew * q))
                    sub_pos = seg_hi
            else:
                stack.append((pos - d, ov_hi - d, d + ed, w + ew))
            pos = ov_hi
            if pos == ehi:
                idx += 1
    resolved.sort(key=lambda x: x[0])
    return resolved


def exact_rational_rank(mat) -> int:
    """Exact rank over Q via Gaussian elimination using fractions.Fraction."""
    if not mat:
        return 0
    m = len(mat)
    n = len(mat[0])
    A = [[Fraction(v) for v in row] for row in mat]
    r = 0
    for c in range(n):
        piv = None
        for i in range(r, m):
            if A[i][c] != 0:
                piv = i
                break
        if piv is None:
            continue
        A[r], A[piv] = A[piv], A[r]
        inv = Fraction(1, A[r][c])
        for j in range(c, n):
            A[r][j] *= inv
        for i in range(m):
            if i != r and A[i][c] != 0:
                fac = A[i][c]
                for j in range(c, n):
                    A[i][j] -= fac * A[r][j]
        r += 1
        if r == m:
            break
    return r


def check_gko_theorem_e1(perm):
    """Verify Theorem E1: rank_Q(P - Z P Z^T) == n - c([P, +1]) == nu_mod - c_{>=2}."""
    n = len(perm)
    # Commutator kappa(x) = P^{-1}((P(x) + 1) mod n) - 1 mod n  <=> P^{-1} Z P Z^{-1}
    inv = [0] * n
    for i, v in enumerate(perm):
        inv[v] = i
    # [P, +1] cycles: x -> (P^{-1}((P(x) + 1) % n) - 1) % n ... wait:
    # Let's check exact definition in cycle10: P - Z P Z^T where Z is cyclic shift (Z e_i = e_{(i+1)%n})
    # (P - Z P Z^T)_{i, j} = P_{i,j} - P_{(i-1)%n, (j-1)%n}.
    # Notice P - Z P Z^T = (I - Z P Z^T P^T) P, so rank(P - Z P Z^T) = rank(I - Q) where Q = Z P Z^{-1} P^{-1}
    # is a permutation matrix! And rank_Q(I - Q) = n - (#cycles of Q)!
    # Q(x) = (P((inv[x] - 1) % n) + 1) % n
    Q = [(perm[(inv[x] - 1) % n] + 1) % n for x in range(n)]
    vis = [False] * n
    c_tot = 0
    c_ge2 = 0
    for i in range(n):
        if not vis[i]:
            c_tot += 1
            sz = 0
            cur = i
            while not vis[cur]:
                vis[cur] = True
                cur = Q[cur]
                sz += 1
            if sz >= 2:
                c_ge2 += 1
    nu_mod = sum(1 for i in range(n) if (perm[(i + 1) % n] - perm[i]) % n != 1)
    assert n - c_tot == nu_mod - c_ge2, f"Cycle identity failed: {n - c_tot} vs {nu_mod - c_ge2}"
    if n <= 36:
        mat = [[0] * n for _ in range(n)]
        for j in range(n):
            i = perm[j]
            mat[i][j] += 1
            mat[(i + 1) % n][(j + 1) % n] -= 1
        rk = exact_rational_rank(mat)
        assert rk == n - c_tot, f"Theorem E1 rank mismatch: {rk} vs {n - c_tot}"
        return rk
    return n - c_tot


def run_framework_1_redheffer(wpb_f: WPBMorphism, T: bytes, rng: random.Random):
    """1. Redheffer Star Product: S1 ★ S2 == Tr_{E1 u E2}(LF, L) and full decode."""
    n = len(wpb_f.dom_set)
    verts = list(range(n))
    rng.shuffle(verts)
    k1 = max(1, n // 3)
    k2 = max(1, n // 3)
    E1 = frozenset(verts[:k1])
    E2 = frozenset(verts[k1:k1 + k2])
    K = wpb_f.dom_set - (E1 | E2)
    # Build Redheffer cascade: eliminate E1 first (S1), then E2 (S2), vs direct joint trace
    s1 = wpb_f.trace_subset(E1)
    s12 = s1.trace_subset(E2)
    s21 = wpb_f.trace_subset(E2).trace_subset(E1)
    direct = wpb_f.trace_subset(E1 | E2)
    assert s12 == direct and s21 == direct
    # Finish to single anchor 0
    full = s12.trace_subset(K - {min(K)})
    assert is_cyclic_rotation(full.decode_single_cycle(), T)


def run_framework_2_bucket_schur(L: bytes, T: bytes, wpb_f: WPBMorphism):
    """2. Bucket-Schur B4 / W-IET: eliminate buckets in asc vs desc orders -> confluent + exact."""
    n = len(L)
    syms = sorted(set(L))
    _, _, _, F = build_lf(L)
    buckets = {c: frozenset(i for i in range(n) if F[i] == c) for c in syms}

    # Ascending order: eliminate all buckets except last
    pcs_asc = bwt_initial_pieces(L)
    elim_so_far = set()
    for c in syms[:-1]:
        pcs_asc = wiet_eliminate_subset(pcs_asc, buckets[c])
        elim_so_far |= set(buckets[c])
        assert WPBMorphism.from_pieces(pcs_asc) == wpb_f.trace_subset(elim_so_far)

    # Descending order: eliminate all buckets except first
    pcs_desc = bwt_initial_pieces(L)
    elim_d = set()
    for c in reversed(syms[1:]):
        pcs_desc = wiet_eliminate_subset(pcs_desc, buckets[c])
        elim_d |= set(buckets[c])
        assert WPBMorphism.from_pieces(pcs_desc) == wpb_f.trace_subset(elim_d)

    # Both decode exact T
    assert is_cyclic_rotation(WPBMorphism.from_pieces(pcs_asc).decode_single_cycle(), T)
    assert is_cyclic_rotation(WPBMorphism.from_pieces(pcs_desc).decode_single_cycle(), T)


def run_framework_3_meyer(L: bytes, wpb_f: WPBMorphism, rng: random.Random):
    """3. Meyer Stochastic Complement P/E = P_KK + P_KE (I - P_EE)^{-1} P_EK == U_mat(Tr_E)."""
    n = len(L)
    lf, _, _, _ = build_lf(L)
    verts = list(range(n)); rng.shuffle(verts)
    E = frozenset(verts[:max(1, n // 2)])
    K = sorted(wpb_f.dom_set - E)
    E_list = sorted(E)
    k_idx = {v: i for i, v in enumerate(K)}
    e_idx = {v: i for i, v in enumerate(E_list)}
    mK = len(K); mE = len(E_list)

    # Compute exact Meyer stochastic complement via exact sparse Neumann series
    # (exact because P_EE is strictly nilpotent on any proper subset of a single cycle!)
    # Column j in K: start with e_j, apply P_KK + sum_{t>=0} P_KE P_EE^t P_EK
    meyer_perm = {}
    for x in K:
        y = lf[x]
        if y in k_idx:
            meyer_perm[x] = y
        else:
            # Exact geometric series (I - P_EE)^{-1} applied to P_EK e_x
            vec = y
            steps = 0
            while vec in e_idx:
                vec = lf[vec]
                steps += 1
                assert steps <= mE + 1
            meyer_perm[x] = vec

    tr = wpb_f.trace_subset(E)
    for x in K:
        assert tr.mapping[x][0] == meyer_perm[x]

    # Also verify explicit matrix inversion (I - P_EE)^{-1} for n <= 20
    if n <= 20:
        # Build (I - P_EE) and verify (I - P_EE) * N == I where N = sum_{t=0}^{mE-1} P_EE^t
        N = [[0] * mE for _ in range(mE)]
        for j, ej in enumerate(E_list):
            cur = ej
            while cur in e_idx:
                N[e_idx[cur]][j] += 1
                cur = lf[cur]
        # Compute P_KK + P_KE * N * P_EK
        S = [[0] * mK for _ in range(mK)]
        for j, x in enumerate(K):
            y = lf[x]
            if y in k_idx:
                S[k_idx[y]][j] += 1
            else:
                ej = e_idx[y]
                for i_e, ev in enumerate(E_list):
                    if N[i_e][ej] and lf[ev] in k_idx:
                        S[k_idx[lf[ev]]][j] += N[i_e][ej]
        for j, x in enumerate(K):
            for i, y in enumerate(K):
                expected = 1 if tr.mapping[x][0] == y else 0
                assert S[i][j] == expected


def run_framework_4_gko(L: bytes, wpb_f: WPBMorphism):
    """4. GKO Displacement Generators / Stein Rank Theorem E1 on LF AND on traced Tr_E(LF)."""
    n = len(L)
    lf, _, _, _ = build_lf(L)
    check_gko_theorem_e1(lf)
    # Also trace down to |K| <= min(20, n//2) and verify exact Gaussian rank on compacted Tr_E!
    keep_sz = min(20, max(3, n // 2))
    E = frozenset(range(keep_sz, n))
    tr_c = wpb_f.trace_subset(E).compact_order()
    perm_k = [tr_c.mapping[i][0] for i in range(keep_sz)]
    check_gko_theorem_e1(perm_k)


def run_framework_5_durand(L: bytes, T: bytes, wpb_f: WPBMorphism):
    """5. Durand Derived Sequences (Theorem D2a): BWT-validity preserved & exact match with Tr_{V\\F_c}."""
    n = len(L)
    # Pick a letter c occurring >= 2 times (else if all occur once, pick first letter)
    counts = defaultdict(int)
    for b in T:
        counts[b] += 1
    c = max(counts.items(), key=lambda kv: (kv[1], -kv[0]))[0]
    u = bytes([c])
    occ = [t for t in range(n) if T[t] == c]
    rwords = []
    for i in range(len(occ)):
        a = occ[i]
        b = occ[(i + 1) % len(occ)]
        w = T[a:b] if b > a else (T[a:] + T[:b])
        rwords.append(w)
    # Augmented order w + u < w' + u (Theorem D2a)
    uniq_aug = sorted(set(rwords), key=lambda w: w + u)
    w2id = {w: i for i, w in enumerate(uniq_aug)}
    T1 = [w2id[w] for w in rwords]
    m1 = len(T1)
    rots1 = sorted(range(m1), key=lambda i: tuple(T1[i:] + T1[:i]))
    L1_words = [uniq_aug[T1[(i - 1) % m1]] for i in rots1]

    # Compare against compacted categorical trace Tr_{V \ F_c}(LF, L)!
    _, _, _, F = build_lf(L)
    F_c = frozenset(i for i in range(n) if F[i] == c)
    tr_dur = wpb_f.trace_subset(wpb_f.dom_set - F_c).compact_order()
    # Notice: LF walks backward, so tr_dur edge words are reversed return words!
    for i in range(m1):
        fwd_word = tr_dur.mapping[i][1][::-1]
        assert fwd_word == L1_words[i], f"Durand D2a word mismatch at {i}: {fwd_word} vs {L1_words[i]}"
    # Verify LF of BWT(T1) matches tr_dur's permutation identically!
    cnt1 = defaultdict(int)
    for x in [T1[(i - 1) % m1] for i in rots1]:
        cnt1[x] += 1
    C1 = {}
    acc = 0
    for s in sorted(cnt1):
        C1[s] = acc
        acc += cnt1[s]
    cur1 = dict(C1)
    for i, ridx in enumerate(rots1):
        sym = T1[(ridx - 1) % m1]
        lf1_i = cur1[sym]
        cur1[sym] += 1
        assert tr_dur.mapping[i][0] == lf1_i, "Durand D2a LF permutation mismatch!"
    assert is_cyclic_rotation(tr_dur.decode_single_cycle(), T)


def run_framework_6_reverse_sais(L: bytes, T: bytes, wpb_f: WPBMorphism):
    """6. Reverse SA-IS (Theorem S1-S2): BWT-space LMS split, BWT-validity preserved, exact match with Tr_{V\\LMS}."""
    n = len(L)
    lf, fl, C, F = build_lf(L)
    pcs = bwt_initial_pieces(L)
    # Theorem S1: split[c] = C[c] + sum_{L_u=c, delta_u > 0} len_u
    l_cnt = [0] * 256
    for lo, hi, d, w in pcs:
        if d > 0:
            l_cnt[w[0]] += (hi - lo)
    split = [C[c] + l_cnt[c] for c in range(256)]
    lms_rows = frozenset(i for i in range(n) if i >= split[F[i]] and L[i] > F[i])
    # Cross-check local-min characterization LF(i) > i and FL(i) > i
    lms_min = frozenset(i for i in range(n) if lf[i] > i and fl[i] > i)
    assert lms_rows == lms_min and len(lms_rows) >= 1

    # Categorical trace onto LMS rows
    tr_lms = wpb_f.trace_subset(wpb_f.dom_set - lms_rows).compact_order()
    assert is_cyclic_rotation(tr_lms.decode_single_cycle(), T)

    # Verify Theorem S2: BWT of reduced LMS-substring string T^(1) matches tr_lms identically!
    # Map each BWT row to its circular text index via SA; tag each text position with is_S (L=0 < S=1)
    sa = sorted(range(n), key=lambda i: T[i:] + T[:i])
    isa = [0] * n
    for r, pos in enumerate(sa):
        isa[pos] = r
    is_s = [1 if isa[t] >= split[T[t]] else 0 for t in range(n)]
    lms_text_pos = sorted(sa[r] for r in lms_rows)
    m1 = len(lms_text_pos)
    lms_substrings = []
    for k in range(m1):
        a = lms_text_pos[k]
        b = lms_text_pos[(k + 1) % m1]
        # Exact SA-IS LMS-substring key: sequence of (char, is_S) from a through closing LMS position b!
        idxs = list(range(a, b + 1)) if b > a else (list(range(a, n)) + list(range(0, b + 1)))
        seg = bytes(T[t] for t in idxs[:-1])
        key = tuple((T[t], is_s[t]) for t in idxs)
        lms_substrings.append((seg, key))
    uniq_keys = sorted(set(x[1] for x in lms_substrings))
    rk_map = {s: i for i, s in enumerate(uniq_keys)}
    T1 = [rk_map[x[1]] for x in lms_substrings]
    rots1 = sorted(range(m1), key=lambda i: tuple(T1[i:] + T1[:i]))
    L1_segs = [lms_substrings[(i - 1) % m1][0] for i in rots1]
    for i in range(m1):
        assert tr_lms.mapping[i][1][::-1] == L1_segs[i], "Reverse SA-IS S2 word mismatch!"
    # Verify exact LF permutation match
    cnt1 = defaultdict(int)
    for ridx in rots1:
        cnt1[T1[(ridx - 1) % m1]] += 1
    C1 = {}
    acc = 0
    for s in sorted(cnt1):
        C1[s] = acc
        acc += cnt1[s]
    cur1 = dict(C1)
    for i, ridx in enumerate(rots1):
        sym = T1[(ridx - 1) % m1]
        lf1_i = cur1[sym]
        cur1[sym] += 1
        assert tr_lms.mapping[i][0] == lf1_i, "Reverse SA-IS S2 LF mismatch!"


def run_framework_7_train_track(L: bytes, T: bytes, wpb_f: WPBMorphism):
    """7. Dunfield-Thurston Train-Track Folding (Theorem T0-T1, S2_dt_heavy): exact trace at every step."""
    n = len(L)
    pcs = bwt_initial_pieces(L)
    q0 = len(pcs)
    # Verify Theorem T0: ill_adj == dual_ill == q0 - |V_sw| where V_sw = switches from B_sync = D0 cap C0
    D0 = {p[0] for p in pcs} | {n}
    C0 = {p[0] + p[2] for p in pcs} | {n}
    B_sync = sorted(D0 & C0)
    num_switches = len(B_sync) - 1
    # Count illegal turns at switches: sum_v (d_in(v) - 1) == q0 - |V_sw| and sum_v (d_out(v) - 1) == q0 - |V_sw|
    d_in = [0] * num_switches
    d_out = [0] * num_switches
    for lo, hi, d, _ in pcs:
        sv_out = bisect.bisect_right(B_sync, lo) - 1
        sv_in = bisect.bisect_right(B_sync, lo + d) - 1
        d_out[sv_out] += 1
        d_in[sv_in] += 1
    assert sum(x - 1 for x in d_in) == q0 - num_switches
    assert sum(x - 1 for x in d_out) == q0 - num_switches

    # Execute S2_dt_heavy (heaviest-band split / fold first) eliminating overlaps until 1 piece remains
    alive_pcs = list(pcs)
    elim_total = set()
    steps = 0
    while len(alive_pcs) > 1:
        # Check Pow spiral first on any self-overlapping branch
        sp_idx = next((i for i, (lo, hi, d, _) in enumerate(alive_pcs) if d != 0 and abs(d) < hi - lo), None)
        if sp_idx is not None:
            lo, hi, d, w = alive_pcs[sp_idx]
            e_sub = set(range(lo + d, hi)) if d > 0 else set(range(lo, hi + d))
            elim_total |= e_sub
            alive_pcs = wiet_eliminate_subset(alive_pcs, e_sub)
        else:
            # S2_dt_heavy: pick heaviest overlapping band J_u cap I_v (u != v)
            best = None
            for ia, (alo, ahi, ad, _) in enumerate(alive_pcs):
                atlo, athi = alo + ad, ahi + ad
                for ib, (blo, bhi, bd, _) in enumerate(alive_pcs):
                    if ia == ib:
                        continue
                    ov_lo = max(atlo, blo); ov_hi = min(athi, bhi)
                    if ov_lo < ov_hi:
                        w_ov = min(ov_hi - ov_lo, abs(bd))
                        if best is None or w_ov > best[0]:
                            best = (w_ov, ov_lo, ov_lo + w_ov)
            assert best is not None
            e_sub = set(range(best[1], best[2]))
            elim_total |= e_sub
            alive_pcs = wiet_eliminate_subset(alive_pcs, e_sub)
        steps += 1
        # Verify exact WPB trace equality at midpoint and end!
        if steps <= 3 or len(alive_pcs) <= 2:
            assert WPBMorphism.from_pieces(alive_pcs) == wpb_f.trace_subset(elim_total)
    assert len(alive_pcs) == 1
    assert WPBMorphism.from_pieces(alive_pcs) == wpb_f.trace_subset(elim_total)
    assert is_cyclic_rotation(WPBMorphism.from_pieces(alive_pcs).decode_single_cycle(), T)


def run_framework_8_morse_ap4(L: bytes, T: bytes, wpb_f: WPBMorphism):
    """8. Discrete Morse AP4_max (acyclic out_deg==1 thin forest, Theorem M1): bijective + exact trace."""
    pcs = bwt_initial_pieces(L)
    q0 = len(pcs)
    starts = [p[0] for p in pcs]
    # Build out_deg == 1 parent pointer on runs
    out_target = [-1] * q0
    for u, (lo, hi, d, _) in enumerate(pcs):
        tlo, thi = lo + d, hi + d
        v = bisect.bisect_right(starts, tlo) - 1
        if pcs[v][0] <= tlo and thi <= pcs[v][1] and v != u:
            out_target[u] = v
    # Break any cycles in out_target to guarantee maximal acyclic forest F_AP4
    state = [0] * q0  # 0=unvis, 1=visiting, 2=done
    for u in range(q0):
        if state[u] != 0 or out_target[u] == -1:
            continue
        path = []
        cur = u
        while cur != -1 and state[cur] == 0:
            state[cur] = 1
            path.append(cur)
            cur = out_target[cur]
        if cur != -1 and state[cur] == 1:
            # Break cycle at last edge into cur
            out_target[path[-1]] = -1
        for x in path:
            state[x] = 2

    # In AP4_max, eliminating the images J_u of forest strips u (Wait: J_u subseteq I_{v(u)} are pairwise
    # disjoint because LF is bijective!). Let E_ap4 = union_{out_target[u] != -1} J_u.
    # Notice: each forest strip u absorbs its target slice inside v(u) with ZERO splitting of u!
    E_ap4 = set()
    for u, v in enumerate(out_target):
        if v != -1:
            lo, hi, d, _ = pcs[u]
            E_ap4.update(range(lo + d, hi + d))
    if len(E_ap4) == len(L):
        # Safety: never eliminate all rows
        E_ap4.remove(0)
    res_pcs = wiet_eliminate_subset(pcs, E_ap4)
    m_ap4 = WPBMorphism.from_pieces(res_pcs)
    assert m_ap4 == wpb_f.trace_subset(E_ap4)
    assert is_cyclic_rotation(m_ap4.decode_single_cycle(), T)


def run_framework_9_open_rv(L: bytes, T: bytes, wpb_f: WPBMorphism):
    """9. Open Right-Boundary RV + spirals.rs: eliminate thick strips w >= W -> exact Tr_{V\\K_W}."""
    pcs = bwt_initial_pieces(L)
    max_w = max(hi - lo for lo, hi, _, _ in pcs)
    W = 4 if (len(L) >= 40 and max_w >= 4) else (2 if max_w >= 2 else 2)
    E_thick = set()
    for lo, hi, _, _ in pcs:
        if hi - lo >= W:
            E_thick.update(range(lo, hi))
    if len(E_thick) == len(L):
        E_thick.remove(0)
    res_pcs = wiet_eliminate_subset(pcs, E_thick)
    m_open = WPBMorphism.from_pieces(res_pcs)
    assert m_open == wpb_f.trace_subset(E_thick)
    assert is_cyclic_rotation(m_open.decode_single_cycle(), T)


def run_framework_10_fullrv_slp(L: bytes, p: int, T: bytes, wpb_f: WPBMorphism):
    """10. FullRV_SLP (LIMIT = 0 global RV -> root SLP): exact Tr_{V\\{p}} + SLP decode."""
    n = len(L)
    pcs = bwt_initial_pieces(L, isolate_p=p)
    # Right-boundary RV with explicit SLP DAG nodes!
    slp_nodes = []  # (kind, a, b): ('lit', byte, 0) or ('cat', id1, id2) or ('rep', id1, k)
    lit_id = {}
    def get_lit(b):
        if b not in lit_id:
            lit_id[b] = len(slp_nodes)
            slp_nodes.append(("lit", b, 0))
        return lit_id[b]

    def slp_cat(a, b):
        idx = len(slp_nodes)
        slp_nodes.append(("cat", a, b))
        return idx

    def slp_rep(a, k):
        if k == 1:
            return a
        idx = len(slp_nodes)
        slp_nodes.append(("rep", a, k))
        return idx

    def slp_eval(nid, memo):
        if nid in memo:
            return memo[nid]
        kind, a, b = slp_nodes[nid]
        if kind == "lit":
            res = bytes([a])
        elif kind == "cat":
            res = slp_eval(a, memo) + slp_eval(b, memo)
        else:
            res = slp_eval(a, memo) * b
        memo[nid] = res
        return res

    # Eliminate all rows except anchor p in right-to-left order via W-IET + SLP
    E_all = frozenset(i for i in range(n) if i != p)
    # Track SLP IDs directly via right-to-left single-piece elimination (exact LIMIT=0 RV!)
    alive = [(lo, hi, d, get_lit(w[0])) for lo, hi, d, w in pcs]
    # Remove non-anchor pieces from right to left (largest lo != p)
    while len(alive) > 1:
        # Pick rightmost piece whose domain != [p, p+1)
        idx = len(alive) - 1
        while alive[idx][0] == p and alive[idx][1] == p + 1:
            idx -= 1
        lo, hi, d, wid = alive[idx]
        # Collapse self-overlap on `alive[idx]` first if present (spirals.rs!)
        L_pc = hi - lo
        if d != 0 and abs(d) < L_pc:
            rem_pcs = alive[:idx] + alive[idx + 1:]
            if d > 0:
                q, r = divmod(L_pc, d)
                if r > 0: rem_pcs.append((lo, lo + r, (q + 1) * d, slp_rep(wid, q + 1)))
                if d - r > 0: rem_pcs.append((lo + r, lo + d, q * d, slp_rep(wid, q)))
            else:
                s = -d; q, r = divmod(L_pc, s)
                if s - r > 0: rem_pcs.append((hi - s, hi - r, -q * s, slp_rep(wid, q)))
                if r > 0: rem_pcs.append((hi - r, hi, -(q + 1) * s, slp_rep(wid, q + 1)))
            alive = sorted(rem_pcs, key=lambda x: x[0])
            continue
        # Otherwise alive[idx] has zero self-overlap: splice it out of every incoming piece!
        nxt_alive = []
        for k, (alo, ahi, ad, aw) in enumerate(alive):
            if k == idx:
                continue
            tlo, thi = alo + ad, ahi + ad
            ov_lo = max(tlo, lo); ov_hi = min(thi, hi)
            if ov_lo >= ov_hi:
                nxt_alive.append((alo, ahi, ad, aw))
            else:
                a_ov_lo = ov_lo - ad; a_ov_hi = ov_hi - ad
                if alo < a_ov_lo:
                    nxt_alive.append((alo, a_ov_lo, ad, aw))
                nxt_alive.append((a_ov_lo, a_ov_hi, ad + d, slp_cat(aw, wid)))
                if a_ov_hi < ahi:
                    nxt_alive.append((a_ov_hi, ahi, ad, aw))
        alive = sorted(nxt_alive, key=lambda x: x[0])

    assert len(alive) == 1 and alive[0][0] == p and alive[0][1] == p + 1 and alive[0][2] == 0
    memo = {}
    root_word = slp_eval(alive[0][3], memo)
    tr_p = wpb_f.trace_subset(E_all)
    assert tr_p.mapping[p] == (p, root_word)
    assert root_word[::-1] == T  # Exact anchor p restores T (not just rotation!)


def run_framework_11_lazard(L: bytes, T: bytes, wpb_f: WPBMorphism):
    """11. Lazard-BWT (extremal full absorption, split-free kappa=1): BWT-validity preserved & exact Tr_{F_c}."""
    n = len(L)
    _, _, _, F = build_lf(L)
    syms = sorted(set(L))
    # Prefer extremal letter c in {min, max} with e_R == 0 (pure split-free 1-step Lazard)
    chosen = None
    pure_lazard = False
    for c in (syms[-1], syms[0]):
        e_R = sum(1 for l, f in zip(L, F) if l == c and f == c)
        if e_R == 0:
            chosen = c
            pure_lazard = True
            break
    if chosen is None:
        # Both extremal letters have self-runs: standard Lazard self-run collapse + full absorption of c_max
        chosen = syms[-1]
    E_c = frozenset(i for i in range(n) if F[i] == chosen)
    # Note: eliminating E_c = F_c absorbs every occurrence of `chosen` into its predecessor letter
    # (when e_R == 0, every bigram is (a, c) with a != c; when e_R > 0, (a, c^k) with k >= 1!).
    tr_laz = wpb_f.trace_subset(E_c).compact_order()
    assert is_cyclic_rotation(tr_laz.decode_single_cycle(), T)

    # Verify BWT-validity preservation: contract T by absorbing `chosen` into predecessor (a c^k)
    # and verify exact BWT match!
    non_c = [t for t in range(n) if T[t] != chosen]
    words_T = []
    for i in range(len(non_c)):
        a = non_c[i]
        b = non_c[(i + 1) % len(non_c)]
        w = T[a:b] if b > a else (T[a:] + T[:b])
        words_T.append(w)
    # Lexicographic order on absorbed symbols: since `chosen` is extremal (max or min),
    # comparing w1 vs w2 under augmented order (with suffix of next non-c letter) == standard lex order
    # when c == max (Wait: for a c^k vs a c^{k'} followed by b < c, longer c-run is LARGER!).
    # Exact order: for c == max, (a, c^k) sorts by (a, +k); for c == min, (a, c^k) sorts by (a, -k)!
    sign = 1 if chosen == syms[-1] else -1
    uniq = sorted(set(words_T), key=lambda w: (w[0], sign * len(w)))
    w2id = {w: i for i, w in enumerate(uniq)}
    T1 = [w2id[w] for w in words_T]
    m1 = len(T1)
    rots1 = sorted(range(m1), key=lambda i: tuple(T1[i:] + T1[:i]))
    L1_words = [uniq[T1[(i - 1) % m1]] for i in rots1]
    for i in range(m1):
        assert tr_laz.mapping[i][1][::-1] == L1_words[i], f"Lazard BWT-validity mismatch at {i}!"
    if pure_lazard:
        # Verify monotone run count r(L') <= r(L) under full absorption (MONODROMY Record 90)
        assert runs_of(L1_words) <= runs_of(L)


def run_framework_12_1cut_bdmr(L: bytes, T: bytes, wpb_f: WPBMorphism):
    """12. 1Cut-BWT (kappa=1 threshold cut) & BDMR (kappa>=2 pair contraction): exact Tr_E + Thm B2."""
    n = len(L)
    _, _, _, F = build_lf(L)
    syms = sorted(set(L))
    # (a) 1Cut-BWT: threshold cut Sigma_L = {syms[0]}, Sigma_R = syms[1:] (kappa = 1)
    sig_L_1cut = {syms[0]}
    E_1cut = frozenset(i for i in range(n) if (L[i] in sig_L_1cut) and (F[i] not in sig_L_1cut))
    assert 0 < len(E_1cut) < n
    tr_1cut = wpb_f.trace_subset(E_1cut)
    # Check W-IET agrees identically with Tr_{E_1cut}
    pcs_1cut = wiet_eliminate_subset(bwt_initial_pieces(L), E_1cut)
    assert WPBMorphism.from_pieces(pcs_1cut) == tr_1cut
    assert is_cyclic_rotation(tr_1cut.decode_single_cycle(), T)
    # Theorem B2 (Universal r + n potential) on projected L|_{V \ E}:
    L_proj_1cut = [L[i] for i in range(n) if i not in E_1cut]
    assert runs_of(L_proj_1cut) + len(L_proj_1cut) <= runs_of(L) + n

    # (b) BDMR: when |Sigma| >= 3 use kappa = 2 non-contiguous Sigma_L = {syms[0], syms[-1]}, Sigma_R = {syms[1]}!
    if len(syms) >= 3:
        sig_L_bdmr = {syms[0], syms[-1]}
    else:
        sig_L_bdmr = {syms[-1]}
    E_bdmr = frozenset(i for i in range(n) if (L[i] in sig_L_bdmr) and (F[i] not in sig_L_bdmr))
    if 0 < len(E_bdmr) < n:
        tr_bdmr = wpb_f.trace_subset(E_bdmr)
        pcs_bdmr = wiet_eliminate_subset(bwt_initial_pieces(L), E_bdmr)
        assert WPBMorphism.from_pieces(pcs_bdmr) == tr_bdmr
        assert is_cyclic_rotation(tr_bdmr.decode_single_cycle(), T)
        L_proj_bdmr = [L[i] for i in range(n) if i not in E_bdmr]
        assert runs_of(L_proj_bdmr) + len(L_proj_bdmr) <= runs_of(L) + n


def run_part1_verification():
    print("=" * 100)
    print("PART 1: THEOREM U_cat — TRACED SYMMETRIC MONOIDAL CATEGORY WPB & 12-FRAMEWORK UNIFICATION")
    print("=" * 100)
    t0 = time.time()
    n_ax = verify_jsv_axioms()
    print(f"  [VERIFIED] All 6 Joyal-Street-Verity WPB axioms + Order-Independence: {n_ax} random tests, 0 failures!")

    # Build the exact 23-instance benchmark suite: W1b (n=9), T_4 (n=10), L^(4) (n=80), + 20 random single-cycle BWTs
    instances = []
    # 1. W1b (n=9)
    T_w1b = bytes([0, 0, 0, 0, 1, 1, 0, 0, 1])
    L_w1b, p_w1b = bwt_encode(T_w1b)
    instances.append(("W1b (n=9)", T_w1b, L_w1b, p_w1b))
    # 2. T_4 (n=10, M^4 A M^4 Z)
    T_t4 = bytes([1, 1, 1, 1, 0, 1, 1, 1, 1, 2])
    L_t4, p_t4 = bwt_encode(T_t4)
    instances.append(("T_4 (n=10)", T_t4, L_t4, p_t4))
    # 3. L^(4) (n=80, Cycle 14-15 binary family)
    L_l4 = build_Ld(4)
    T_l4, p_l4, _ = ibwt_single_cycle(L_l4)
    instances.append(("L^(4) (n=80)", T_l4, L_l4, p_l4))
    # 4. 20 random valid single-cycle BWTs (n in [8, 36], |Sigma| in {2, 3, 4})
    rng = random.Random(20261117)
    idx_r = 0
    while idx_r < 20:
        n = rng.randint(8, 36)
        sig = [2, 3, 4][idx_r % 3]
        T = bytes(rng.randrange(sig) for _ in range(n))
        if len(set(T)) < sig:
            continue
        L, p = bwt_encode(T)
        if ibwt_single_cycle(L) is None:
            continue
        idx_r += 1
        instances.append((f"rand_{idx_r:02d} (n={n},s={sig})", T, L, p))

    fw_counts = [0] * 12
    for name, T, L, p in instances:
        wpb_f = WPBMorphism.from_bwt(L)
        assert is_cyclic_rotation(wpb_f.decode_single_cycle(), T)
        run_framework_1_redheffer(wpb_f, T, rng); fw_counts[0] += 1
        run_framework_2_bucket_schur(L, T, wpb_f); fw_counts[1] += 1
        run_framework_3_meyer(L, wpb_f, rng); fw_counts[2] += 1
        run_framework_4_gko(L, wpb_f); fw_counts[3] += 1
        run_framework_5_durand(L, T, wpb_f); fw_counts[4] += 1
        run_framework_6_reverse_sais(L, T, wpb_f); fw_counts[5] += 1
        run_framework_7_train_track(L, T, wpb_f); fw_counts[6] += 1
        run_framework_8_morse_ap4(L, T, wpb_f); fw_counts[7] += 1
        run_framework_9_open_rv(L, T, wpb_f); fw_counts[8] += 1
        run_framework_10_fullrv_slp(L, p, T, wpb_f); fw_counts[9] += 1
        run_framework_11_lazard(L, T, wpb_f); fw_counts[10] += 1
        run_framework_12_1cut_bdmr(L, T, wpb_f); fw_counts[11] += 1

    fw_names = [
        "1. Redheffer Star Product",
        "2. Bucket-Schur B4 / W-IET",
        "3. Meyer Stochastic Complement",
        "4. GKO Displacement (Thm E1)",
        "5. Durand Derived Seq (Thm D2a)",
        "6. Reverse SA-IS (Thm S1-S2)",
        "7. Dunfield-Thurston TT (Thm T0-T1)",
        "8. Discrete Morse AP4_max (Thm M1)",
        "9. Open Right-Boundary RV",
        "10. FullRV_SLP (LIMIT=0 -> SLP)",
        "11. Lazard-BWT (Split-Free)",
        "12. 1Cut-BWT & BDMR (Thm B2)",
    ]
    print(f"  [VERIFIED] Exact Tr_E(LF, L) confluence & decode across {len(instances)} BWTs (W1b, T_4, L^(4), 20 random):")
    for fn, cnt in zip(fw_names, fw_counts):
        print(f"    {fn:36s}: {cnt:2d}/{len(instances)} EXACT MATCH (0 discrepancies)")
    print(f"  Part 1 total: {sum(fw_counts)}/{12 * len(instances)} framework-instance checks PASSED in {time.time() - t0:.2f}s!\n")


# ==============================================================================
# PART 2: EXACT [bounded-opt] BINARY BWT FRONTIER INSIDE c*_{|Sigma|=2} in [1/2, 19/24]
# ==============================================================================
C_EXACT_SRC = r"""
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define MAX_P 32
#define HT_BITS 22
#define HT_SIZE (1u << HT_BITS)
#define HT_MASK (HT_SIZE - 1u)

typedef struct {
    uint8_t lo, hi;
    int8_t d;
    uint8_t pad;
    uint64_t w; /* (len << 32) | bits (lossless for binary words len <= 32) */
} Piece;

typedef struct {
    uint8_t p;
    uint8_t m;
    uint8_t pad[6];
    Piece pc[MAX_P];
} State;

typedef struct {
    uint8_t kind; /* 0=Pow+, 1=Pow-, 2=Join_lo, 3=Join_hi */
    uint8_t ia;
    uint8_t ib;
    uint8_t pad;
} MoveRec;

static inline uint64_t w_concat(uint64_t a, uint64_t b) {
    uint32_t la = (uint32_t)(a >> 32);
    uint32_t lb = (uint32_t)(b >> 32);
    return ((uint64_t)(la + lb) << 32) | ((a & 0xFFFFFFFFull) | ((b & 0xFFFFFFFFull) << la));
}

static inline uint64_t w_repeat(uint64_t a, int q) {
    uint32_t la = (uint32_t)(a >> 32);
    uint64_t ba = a & 0xFFFFFFFFull;
    uint64_t out = 0;
    uint32_t shift = 0;
    for (int i = 0; i < q; i++) {
        out |= (ba << shift);
        shift += la;
    }
    return ((uint64_t)shift << 32) | out;
}

static inline uint64_t hash_state(const State *s) {
    uint64_t h = 14695981039346656037ull;
    h ^= (uint64_t)s->p | ((uint64_t)s->m << 8);
    h *= 1099511628211ull;
    for (int i = 0; i < s->p; i++) {
        uint64_t v = (uint64_t)s->pc[i].lo | ((uint64_t)s->pc[i].hi << 8)
                   | ((uint64_t)(uint8_t)s->pc[i].d << 16);
        h ^= v;
        h *= 1099511628211ull;
        h ^= s->pc[i].w;
        h *= 1099511628211ull;
    }
    return h ? h : 1ull;
}

static inline int eq_state(const State *a, const State *b) {
    if (a->p != b->p || a->m != b->m) return 0;
    for (int i = 0; i < a->p; i++) {
        if (a->pc[i].lo != b->pc[i].lo || a->pc[i].hi != b->pc[i].hi ||
            a->pc[i].d != b->pc[i].d || a->pc[i].w != b->pc[i].w)
            return 0;
    }
    return 1;
}

static void compact_and_coalesce(Piece *arr, int cnt, State *out) {
    for (int i = 1; i < cnt; i++) {
        Piece key = arr[i];
        int j = i - 1;
        while (j >= 0 && arr[j].lo > key.lo) {
            arr[j + 1] = arr[j];
            j--;
        }
        arr[j + 1] = key;
    }
    uint8_t new_lo[MAX_P + 4];
    int acc = 0;
    for (int i = 0; i < cnt; i++) {
        new_lo[i] = (uint8_t)acc;
        acc += (arr[i].hi - arr[i].lo);
    }
    out->m = (uint8_t)acc;
    int op = 0;
    for (int i = 0; i < cnt; i++) {
        int nlo = new_lo[i];
        int nhi = nlo + (arr[i].hi - arr[i].lo);
        int target = (int)arr[i].lo + (int)arr[i].d;
        int lo = 0, hi = cnt - 1;
        while (lo < hi) {
            int mid = (lo + hi + 1) >> 1;
            if (arr[mid].lo <= target) lo = mid;
            else hi = mid - 1;
        }
        int nt = (int)new_lo[lo] + (target - (int)arr[lo].lo);
        int nd = nt - nlo;
        if (op > 0 && out->pc[op - 1].hi == nlo && out->pc[op - 1].d == nd && out->pc[op - 1].w == arr[i].w) {
            out->pc[op - 1].hi = (uint8_t)nhi;
        } else {
            out->pc[op].lo = (uint8_t)nlo;
            out->pc[op].hi = (uint8_t)nhi;
            out->pc[op].d = (int8_t)nd;
            out->pc[op].pad = 0;
            out->pc[op].w = arr[i].w;
            op++;
        }
    }
    out->p = (uint8_t)op;
}

static inline int exact_terminal(const State *s) {
    if (s->m == 1) return 0;
    if (s->m == 2) return 1;
    if (s->m == 3) return 2;
    if (s->m == 4) {
        for (int i = 0; i < s->p; i++) {
            int L = s->pc[i].hi - s->pc[i].lo;
            int ad = s->pc[i].d < 0 ? -s->pc[i].d : s->pc[i].d;
            if (L == 3 && ad == 1) return 2;
        }
        return 3;
    }
    return -1;
}

static inline int h_admissible(const State *s, int use_c7) {
    if (s->p <= 1) return 0;
    int h = (s->p - 1 + 4) / 5; /* Theorem C3 */
    int u = 0, d = 0, prev_u = -1;
    for (int i = 0; i < s->p; i++) {
        uint32_t wl = (uint32_t)(s->pc[i].w >> 32);
        if (wl == 1) {
            int b = (int)(s->pc[i].w & 1ull);
            u++;
            if (prev_u == b) d++;
            prev_u = b;
        } else {
            prev_u = -1;
        }
    }
    int h4 = (u - d + 2) / 3; /* Theorem C4 */
    if (h4 > h) h = h4;
    if (use_c7) {
        int xi = 1;
        int prev_b = (int)(s->pc[0].w & 1ull);
        for (int i = 1; i < s->p; i++) {
            int b = (int)(s->pc[i].w & 1ull);
            if (b != prev_b) {
                xi++;
                prev_b = b;
            }
        }
        int h7 = xi / 2; /* Theorem C7: ceil((Xi - 1) / 2) */
        if (h7 > h) h = h7;
    }
    if (s->m >= 3 && h < 2) h = 2;
    return h;
}

static int gen_neighbors(const State *s, State *out_arr, MoveRec *mv_arr, int max_out) {
    int n_out = 0;
    Piece tmp[MAX_P + 4];
    int p = s->p;

    for (int i = 0; i < p; i++) {
        int lo = s->pc[i].lo, hi = s->pc[i].hi, d = s->pc[i].d;
        uint64_t w = s->pc[i].w;
        int L = hi - lo;
        if (d > 0 && d < L) {
            int q = L / d, rem = L % d;
            int cnt = 0;
            for (int k = 0; k < p; k++) if (k != i) tmp[cnt++] = s->pc[k];
            if (rem > 0) tmp[cnt++] = (Piece){(uint8_t)lo, (uint8_t)(lo + rem), (int8_t)((q + 1) * d), 0, w_repeat(w, q + 1)};
            if (d - rem > 0) tmp[cnt++] = (Piece){(uint8_t)(lo + rem), (uint8_t)(lo + d), (int8_t)(q * d), 0, w_repeat(w, q)};
            compact_and_coalesce(tmp, cnt, &out_arr[n_out]);
            if (mv_arr) mv_arr[n_out] = (MoveRec){0, (uint8_t)i, 0, 0};
            n_out++;
            if (n_out >= max_out) return n_out;
        } else if (d < 0 && (-d) < L) {
            int sd = -d, q = L / sd, rem = L % sd;
            int cnt = 0;
            for (int k = 0; k < p; k++) if (k != i) tmp[cnt++] = s->pc[k];
            if (sd - rem > 0) tmp[cnt++] = (Piece){(uint8_t)(hi - sd), (uint8_t)(hi - rem), (int8_t)(-q * sd), 0, w_repeat(w, q)};
            if (rem > 0) tmp[cnt++] = (Piece){(uint8_t)(hi - rem), (uint8_t)hi, (int8_t)(-(q + 1) * sd), 0, w_repeat(w, q + 1)};
            compact_and_coalesce(tmp, cnt, &out_arr[n_out]);
            if (mv_arr) mv_arr[n_out] = (MoveRec){1, (uint8_t)i, 0, 0};
            n_out++;
            if (n_out >= max_out) return n_out;
        }
    }

    for (int ia = 0; ia < p; ia++) {
        int alo = s->pc[ia].lo, ahi = s->pc[ia].hi, ad = s->pc[ia].d;
        uint64_t aw = s->pc[ia].w;
        int atlo = alo + ad, athi = ahi + ad;
        for (int ib = 0; ib < p; ib++) {
            if (ia == ib) continue;
            int bd = s->pc[ib].d;
            if (bd == 0) continue;
            int blo = s->pc[ib].lo, bhi = s->pc[ib].hi;
            int ov_lo = atlo > blo ? atlo : blo;
            int ov_hi = athi < bhi ? athi : bhi;
            if (ov_lo >= ov_hi) continue;
            uint64_t bw = s->pc[ib].w;
            int abd = bd < 0 ? -bd : bd;
            int cands[2][2];
            int nc = 0;
            if (ov_hi - ov_lo > abd) {
                cands[0][0] = ov_lo; cands[0][1] = ov_lo + abd;
                cands[1][0] = ov_hi - abd; cands[1][1] = ov_hi;
                nc = 2;
            } else {
                cands[0][0] = ov_lo; cands[0][1] = ov_hi;
                nc = 1;
            }
            uint64_t abw = w_concat(aw, bw);
            for (int c = 0; c < nc; c++) {
                int glo = cands[c][0], ghi = cands[c][1];
                int a_lo = glo - ad, a_hi = ghi - ad;
                int cnt = 0;
                for (int k = 0; k < p; k++) {
                    if (k == ia) {
                        if (alo < a_lo) tmp[cnt++] = (Piece){(uint8_t)alo, (uint8_t)a_lo, (int8_t)ad, 0, aw};
                        tmp[cnt++] = (Piece){(uint8_t)a_lo, (uint8_t)a_hi, (int8_t)(ad + bd), 0, abw};
                        if (a_hi < ahi) tmp[cnt++] = (Piece){(uint8_t)a_hi, (uint8_t)ahi, (int8_t)ad, 0, aw};
                    } else if (k == ib) {
                        if (blo < glo) tmp[cnt++] = (Piece){(uint8_t)blo, (uint8_t)glo, (int8_t)bd, 0, bw};
                        if (ghi < bhi) tmp[cnt++] = (Piece){(uint8_t)ghi, (uint8_t)bhi, (int8_t)bd, 0, bw};
                    } else {
                        tmp[cnt++] = s->pc[k];
                    }
                }
                compact_and_coalesce(tmp, cnt, &out_arr[n_out]);
                if (mv_arr) mv_arr[n_out] = (MoveRec){(uint8_t)(2 + c), (uint8_t)ia, (uint8_t)ib, 0};
                n_out++;
                if (n_out >= max_out) return n_out;
            }
        }
    }
    return n_out;
}

static int greedy_ub(const State *init) {
    int best_q = init->m - 1;
    State nbs[256];
    for (int mode = 0; mode < 3; mode++) {
        State cur = *init;
        int g = 0;
        while (g < best_q) {
            int term = exact_terminal(&cur);
            if (term >= 0) {
                if (g + term < best_q) best_q = g + term;
                break;
            }
            int nn = gen_neighbors(&cur, nbs, NULL, 256);
            if (nn == 0) break;
            int bi = 0, bsc = 1000000;
            for (int i = 0; i < nn; i++) {
                int sc;
                if (mode == 0) sc = nbs[i].p * 64 + nbs[i].m;
                else if (mode == 1) sc = h_admissible(&nbs[i], 1) * 256 + nbs[i].p * 16 + nbs[i].m;
                else sc = nbs[i].m * 64 + nbs[i].p;
                if (sc < bsc) { bsc = sc; bi = i; }
            }
            cur = nbs[bi];
            g++;
        }
    }
    return best_q;
}

typedef struct {
    uint64_t hash;
    int32_t pool_idx;
} HTEntry;

typedef struct {
    State st;
    uint8_t g;
    uint8_t f;
    MoveRec mv;
    int32_t parent;
} PoolNode;

typedef struct {
    int32_t pool_idx;
    int32_t next;
    uint8_t g;
} QEntry;

static HTEntry *g_ht = NULL;
static uint32_t *g_ht_gen = NULL;
static uint32_t g_cur_gen = 1;
static PoolNode *g_pool = NULL;
static int32_t g_pool_cap = 0;
static QEntry *g_q = NULL;
static int32_t g_q_cap = 0;

static void ensure_alloc(int32_t cap) {
    if (!g_ht) {
        g_ht = (HTEntry *)malloc(sizeof(HTEntry) * HT_SIZE);
        g_ht_gen = (uint32_t *)calloc(HT_SIZE, sizeof(uint32_t));
    }
    if (cap > g_pool_cap) {
        g_pool = (PoolNode *)realloc(g_pool, sizeof(PoolNode) * cap);
        g_pool_cap = cap;
    }
    int32_t qcap = cap * 2;
    if (qcap > g_q_cap) {
        g_q = (QEntry *)realloc(g_q, sizeof(QEntry) * qcap);
        g_q_cap = qcap;
    }
}

static int32_t ht_find_or_insert(const State *st, uint8_t g, uint8_t f, int32_t parent, MoveRec mv,
                                 int32_t *pool_len, int32_t max_nodes, int *ins_flag) {
    uint64_t h = hash_state(st);
    uint32_t idx = (uint32_t)(h & HT_MASK);
    while (g_ht_gen[idx] == g_cur_gen) {
        if (g_ht[idx].hash == h) {
            int32_t pi = g_ht[idx].pool_idx;
            if (eq_state(&g_pool[pi].st, st)) {
                if (g < g_pool[pi].g) {
                    g_pool[pi].g = g;
                    g_pool[pi].f = f;
                    g_pool[pi].parent = parent;
                    g_pool[pi].mv = mv;
                    *ins_flag = 1;
                    return pi;
                }
                *ins_flag = 0;
                return pi;
            }
        }
        idx = (idx + 1u) & HT_MASK;
    }
    if (*pool_len >= max_nodes) {
        *ins_flag = -1;
        return -1;
    }
    int32_t pi = (*pool_len)++;
    g_pool[pi].st = *st;
    g_pool[pi].g = g;
    g_pool[pi].f = f;
    g_pool[pi].parent = parent;
    g_pool[pi].mv = mv;
    g_ht_gen[idx] = g_cur_gen;
    g_ht[idx].hash = h;
    g_ht[idx].pool_idx = pi;
    *ins_flag = 1;
    return pi;
}

static void init_from_L(const uint8_t *L, int n, State *init) {
    int cnt0 = 0;
    for (int i = 0; i < n; i++) if (L[i] == 0) cnt0++;
    int cur[2] = {0, cnt0};
    memset(init, 0, sizeof(*init));
    init->m = (uint8_t)n;
    int p = 0;
    for (int i = 0; i < n; ) {
        int b = L[i];
        int j = i + 1;
        while (j < n && L[j] == b) j++;
        int d = cur[b] - i;
        cur[b] += (j - i);
        init->pc[p].lo = (uint8_t)i;
        init->pc[p].hi = (uint8_t)j;
        init->pc[p].d = (int8_t)d;
        init->pc[p].w = (1ull << 32) | (uint64_t)b;
        p++;
        i = j;
    }
    init->p = (uint8_t)p;
}

/*
 * Fast exact Q* solver (with optional full path reconstruction when out_moves != NULL).
 * When out_moves != NULL, disables greedy UB cutoff and m<=4 shortcut so we always
 * trace an explicit exact move sequence all the way to P = 1!
 */
int solve_exact_binary(const uint8_t *L, int n, int use_c7, int use_term4, int max_nodes,
                       int *out_exp, int *out_r0, uint8_t *out_moves) {
    State init;
    init_from_L(L, n, &init);
    *out_r0 = init.p;
    *out_exp = 0;

    if (init.p == 1) return 0;
    int want_path = (out_moves != NULL);
    if (use_term4 && !want_path) {
        int t = exact_terminal(&init);
        if (t >= 0) return t;
    }

    int ub = want_path ? (n + 1) : greedy_ub(&init);
    int h0 = h_admissible(&init, use_c7);
    if (!want_path && h0 >= ub) return ub;

    ensure_alloc(max_nodes);
    g_cur_gen++;
    if (g_cur_gen == 0) {
        memset(g_ht_gen, 0, sizeof(uint32_t) * HT_SIZE);
        g_cur_gen = 1;
    }

    int32_t buckets[64];
    for (int i = 0; i < 64; i++) buckets[i] = -1;

    int32_t pool_len = 0;
    int32_t q_len = 0;
    int ins = 0;
    MoveRec mv0 = {0, 0, 0, 0};
    int32_t root_i = ht_find_or_insert(&init, 0, (uint8_t)h0, -1, mv0, &pool_len, max_nodes, &ins);
    g_q[q_len] = (QEntry){root_i, -1, 0};
    buckets[h0] = q_len++;

    int cur_f = h0;
    int expansions = 0;
    State nbs[256];
    MoveRec mvs[256];

    while (cur_f < ub) {
        while (cur_f < ub && buckets[cur_f] == -1) cur_f++;
        if (cur_f >= ub) break;

        int32_t qi = buckets[cur_f];
        buckets[cur_f] = g_q[qi].next;
        int32_t idx = g_q[qi].pool_idx;
        if (g_q[qi].g > g_pool[idx].g) continue; /* stale entry */

        State st = g_pool[idx].st;
        int g = g_pool[idx].g;

        if (st.p == 1 && st.pc[0].d == 0) {
            *out_exp = expansions;
            if (want_path) {
                int32_t c = idx;
                for (int step = g - 1; step >= 0; step--) {
                    out_moves[3 * step + 0] = g_pool[c].mv.kind;
                    out_moves[3 * step + 1] = g_pool[c].mv.ia;
                    out_moves[3 * step + 2] = g_pool[c].mv.ib;
                    c = g_pool[c].parent;
                }
            }
            return g;
        }

        expansions++;
        int nn = gen_neighbors(&st, nbs, want_path ? mvs : NULL, 256);
        int ng = g + 1;
        for (int i = 0; i < nn; i++) {
            if (use_term4 && !want_path) {
                int t = exact_terminal(&nbs[i]);
                if (t >= 0) {
                    if (ng + t < ub) {
                        ub = ng + t;
                        if (ub <= cur_f) {
                            *out_exp = expansions;
                            return ub;
                        }
                    }
                    continue;
                }
            } else if (!want_path) {
                if (nbs[i].p == 1 && nbs[i].pc[0].d == 0) {
                    if (ng < ub) {
                        ub = ng;
                        if (ub <= cur_f) {
                            *out_exp = expansions;
                            return ub;
                        }
                    }
                    continue;
                }
            }
            int nh = h_admissible(&nbs[i], use_c7);
            int nf = ng + nh;
            if (!want_path && nf >= ub) continue;
            if (want_path && nf > ub) continue;
            int ins_flag = 0;
            MoveRec mvi = want_path ? mvs[i] : mv0;
            int32_t ni = ht_find_or_insert(&nbs[i], (uint8_t)ng, (uint8_t)nf, idx, mvi,
                                           &pool_len, max_nodes, &ins_flag);
            if (ins_flag == -1 || q_len >= g_q_cap) {
                *out_exp = expansions;
                return -1;
            }
            if (ins_flag == 1) {
                g_q[q_len] = (QEntry){ni, buckets[nf], (uint8_t)ng};
                buckets[nf] = q_len++;
                if (nf < cur_f) cur_f = nf;
            }
        }
    }

    *out_exp = expansions;
    return ub;
}
"""


# --- Independent Pure-Python C_gap Engine for 100% Cross-Verification & Witness Replay ---
def py_compact_and_coalesce(pieces):
    pieces = sorted(pieces, key=lambda x: x[0])
    starts = [p[0] for p in pieces]
    ends = [p[1] for p in pieces]
    pref = [0] * (len(pieces) + 1)
    for i, (lo, hi, _, _) in enumerate(pieces):
        assert lo < hi
        if i > 0:
            assert ends[i - 1] <= lo
        pref[i + 1] = pref[i] + (hi - lo)

    def phi(x):
        idx = bisect.bisect_right(starts, x) - 1
        assert 0 <= idx < len(pieces) and starts[idx] <= x < ends[idx]
        return pref[idx] + (x - starts[idx])

    out = []
    for i, (lo, hi, d, w) in enumerate(pieces):
        nlo, nhi = pref[i], pref[i + 1]
        nd = phi(lo + d) - nlo
        assert phi(hi - 1 + d) - (nhi - 1) == nd
        if out and out[-1][1] == nlo and out[-1][2] == nd and out[-1][3] == w:
            out[-1] = (out[-1][0], nhi, nd, w)
        else:
            out.append((nlo, nhi, nd, w))
    return tuple(out)


def py_xi(st) -> int:
    r = 1
    prev = st[0][3][0]
    for _, _, _, w in st[1:]:
        if w[0] != prev:
            r += 1
            prev = w[0]
    return r


def py_psi(st) -> int:
    u = d = 0
    prev = None
    for _, _, _, w in st:
        if len(w) == 1:
            u += 1
            if prev == w:
                d += 1
            prev = w
        else:
            prev = None
    return u - d


def py_neighbors_cgap(state, all_subintervals=False):
    out = []
    for i, (lo, hi, d, w) in enumerate(state):
        L = hi - lo
        if d > 0 and d < L:
            q, rem = divmod(L, d)
            nxt = list(state[:i]) + list(state[i + 1:])
            if rem > 0: nxt.append((lo, lo + rem, (q + 1) * d, w * (q + 1)))
            if d - rem > 0: nxt.append((lo + rem, lo + d, q * d, w * q))
            out.append((py_compact_and_coalesce(nxt), ("Pow+", i)))
        elif d < 0 and (-d) < L:
            s = -d; q, rem = divmod(L, s)
            nxt = list(state[:i]) + list(state[i + 1:])
            if s - rem > 0: nxt.append((hi - s, hi - rem, -q * s, w * q))
            if rem > 0: nxt.append((hi - rem, hi, -(q + 1) * s, w * (q + 1)))
            out.append((py_compact_and_coalesce(nxt), ("Pow-", i)))
    for ia, (alo, ahi, ad, aw) in enumerate(state):
        atlo, athi = alo + ad, ahi + ad
        for ib, (blo, bhi, bd, bw) in enumerate(state):
            if ia == ib or bd == 0:
                continue
            ov_lo, ov_hi = max(atlo, blo), min(athi, bhi)
            if ov_lo >= ov_hi:
                continue
            abd = abs(bd)
            if all_subintervals:
                cands = [(glo, ghi) for glo in range(ov_lo, ov_hi)
                         for ghi in range(glo + 1, min(ov_hi, glo + abd) + 1)]
            elif (ov_hi - ov_lo) > abd:
                cands = [(ov_lo, ov_lo + abd), (ov_hi - abd, ov_hi)]
            else:
                cands = [(ov_lo, ov_hi)]
            for glo, ghi in cands:
                assert (ghi <= glo + bd) or (ghi + bd <= glo)
                a_lo, a_hi = glo - ad, ghi - ad
                nxt = []
                for k, pc in enumerate(state):
                    if k == ia:
                        if alo < a_lo: nxt.append((alo, a_lo, ad, aw))
                        nxt.append((a_lo, a_hi, ad + bd, aw + bw))
                        if a_hi < ahi: nxt.append((a_hi, ahi, ad, aw))
                    elif k == ib:
                        if blo < glo: nxt.append((blo, glo, bd, bw))
                        if ghi < bhi: nxt.append((ghi, bhi, bd, bw))
                    else:
                        nxt.append(pc)
                out.append((py_compact_and_coalesce(nxt), ("Join", ia, ib)))
    return out


def py_solve_exact(L: bytes, all_subintervals=False):
    init = tuple(bwt_initial_pieces(L))
    if len(init) == 1:
        return 0
    def h_fn(st):
        if len(st) <= 1:
            return 0
        return max(py_xi(st) // 2, (len(st) + 3) // 5, (py_psi(st) + 2) // 3)
    pq = [(h_fn(init), 0, 0, init)]
    best = {init: 0}
    tie = 1
    while pq:
        f, g, _, st = heapq.heappop(pq)
        if g > best[st]:
            continue
        if len(st) == 1 and st[0][2] == 0:
            return g
        for ns, _ in py_neighbors_cgap(st, all_subintervals=all_subintervals):
            ng = g + 1
            if ng < best.get(ns, 999):
                best[ns] = ng
                heapq.heappush(pq, (ng + h_fn(ns), ng, tie, ns))
                tie += 1
    raise RuntimeError("No path found!")


def py_replay_and_verify(L: bytes, T: bytes, moves_raw, q_star: int):
    """Independently replay a C-reconstructed optimal path in pure Python, asserting C3/C4/C7 + byte-exact decode!"""
    st = tuple(bwt_initial_pieces(L))
    dp_seq = []
    dxi_seq = []
    mv_names = []
    for s in range(q_star):
        kind = moves_raw[3 * s + 0]
        ia = moves_raw[3 * s + 1]
        ib = moves_raw[3 * s + 2]
        p0, xi0, psi0 = len(st), py_xi(st), py_psi(st)
        if kind == 0:  # Pow+
            lo, hi, d, w = st[ia]
            Lp = hi - lo; q, rem = divmod(Lp, d)
            nxt = list(st[:ia]) + list(st[ia + 1:])
            if rem > 0: nxt.append((lo, lo + rem, (q + 1) * d, w * (q + 1)))
            if d - rem > 0: nxt.append((lo + rem, lo + d, q * d, w * q))
            st = py_compact_and_coalesce(nxt)
            mv_names.append(f"Pow+({ia})")
        elif kind == 1:  # Pow-
            lo, hi, d, w = st[ia]
            Lp = hi - lo; sd = -d; q, rem = divmod(Lp, sd)
            nxt = list(st[:ia]) + list(st[ia + 1:])
            if sd - rem > 0: nxt.append((hi - sd, hi - rem, -q * sd, w * q))
            if rem > 0: nxt.append((hi - rem, hi, -(q + 1) * sd, w * (q + 1)))
            st = py_compact_and_coalesce(nxt)
            mv_names.append(f"Pow-({ia})")
        else:  # Join_lo (2) or Join_hi (3)
            alo, ahi, ad, aw = st[ia]
            blo, bhi, bd, bw = st[ib]
            ov_lo, ov_hi = max(alo + ad, blo), min(ahi + ad, bhi)
            abd = abs(bd)
            if (ov_hi - ov_lo) > abd:
                glo, ghi = (ov_lo, ov_lo + abd) if kind == 2 else (ov_hi - abd, ov_hi)
            else:
                glo, ghi = ov_lo, ov_hi
            assert (ghi <= glo + bd) or (ghi + bd <= glo)
            a_lo, a_hi = glo - ad, ghi - ad
            nxt = []
            for k, pc in enumerate(st):
                if k == ia:
                    if alo < a_lo: nxt.append((alo, a_lo, ad, aw))
                    nxt.append((a_lo, a_hi, ad + bd, aw + bw))
                    if a_hi < ahi: nxt.append((a_hi, ahi, ad, aw))
                elif k == ib:
                    if blo < glo: nxt.append((blo, glo, bd, bw))
                    if ghi < bhi: nxt.append((ghi, bhi, bd, bw))
                else:
                    nxt.append(pc)
            st = py_compact_and_coalesce(nxt)
            mv_names.append(f"J({ia},{ib},w={ghi-glo})")
        dp = len(st) - p0
        dxi = py_xi(st) - xi0
        dpsi = py_psi(st) - psi0
        if kind in (0, 1):
            assert dp >= -3 and dxi == 0 and dpsi >= -1
        else:
            assert dp >= -5 and -2 <= dxi <= 0 and dpsi >= -3
        dp_seq.append(dp)
        dxi_seq.append(dxi)
    assert len(st) == 1 and st[0][2] == 0
    dec = st[0][3][::-1]
    assert is_cyclic_rotation(dec, T), f"Decoded text mismatch!"
    return dp_seq, dxi_seq, mv_names


def compile_c_solver():
    os.makedirs("/tmp/fcp-a", exist_ok=True)
    c_path = "/tmp/fcp-a/_cgap_exact_c17.c"
    so_path = "/tmp/fcp-a/_cgap_exact_c17.so"
    with open(c_path, "w") as f:
        f.write(C_EXACT_SRC)
    subprocess.check_call(["gcc", "-O3", "-shared", "-fPIC", c_path, "-o", so_path])
    lib = ctypes.CDLL(so_path)
    lib.solve_exact_binary.argtypes = [
        ctypes.c_char_p, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_int,
        ctypes.POINTER(ctypes.c_int), ctypes.POINTER(ctypes.c_int), ctypes.c_void_p,
    ]
    lib.solve_exact_binary.restype = ctypes.c_int
    return lib


def run_part2_exact_frontier():
    print("=" * 100)
    print("PART 2: EXACT [bounded-opt] BINARY BWT FRONTIER INSIDE c*_{|Sigma|=2} in [1/2, 19/24]")
    print("=" * 100)
    lib = compile_c_solver()

    def c_solve(L: bytes, use_c7=1, use_term4=1, max_nodes=2500000, want_path=False):
        exp_c = ctypes.c_int(0)
        r0_c = ctypes.c_int(0)
        mv_buf = (ctypes.c_uint8 * (3 * 64))() if want_path else None
        ptr = ctypes.cast(mv_buf, ctypes.c_void_p) if want_path else None
        q = lib.solve_exact_binary(L, len(L), use_c7, use_term4, max_nodes,
                                   ctypes.byref(exp_c), ctypes.byref(r0_c), ptr)
        return q, r0_c.value, exp_c.value, (bytes(mv_buf) if want_path else None)

    # 2A. Independent Cross-Verification (C vs Pure-Python, Complement Symmetry, Sub-Interval Joins)
    t_v = time.time()
    py_checked = 0
    sub_checked = 0
    sym_checked = 0
    for n in range(2, 12):
        res_map = {}
        for T_tup in gen_lyndon_binary(n):
            T = bytes(T_tup)
            L, _ = bwt_encode(T)
            q_c, r0_c, _, _ = c_solve(L, use_c7=1, use_term4=1)
            if n <= 10:
                q_c_noterm, _, _, _ = c_solve(L, use_c7=1, use_term4=0)
                q_py = py_solve_exact(L, all_subintervals=False)
                assert q_c == q_c_noterm == q_py, f"Mismatch on n={n} T={T}: {q_c} vs {q_py}"
                py_checked += 1
            if n <= 8:
                q_sub = py_solve_exact(L, all_subintervals=True)
                assert q_sub == q_c, f"Sub-interval beat maximal Join on n={n} T={T}!"
                sub_checked += 1
            res_map[T_tup] = q_c
        for T_tup, q_val in res_map.items():
            comp = tuple(1 - x for x in T_tup)
            comp_min = min(comp[i:] + comp[:i] for i in range(n))
            assert res_map[comp_min] == q_val, f"Complement symmetry failed on {T_tup}!"
            sym_checked += 1
    print(f"  [VERIFIED] C vs Pure-Python exact A* (n=2..10): {py_checked} Lyndon BWTs, 0 mismatches")
    print(f"  [VERIFIED] Exact complement symmetry Q*(T)==Q*(~T) (n=2..11): {sym_checked} Lyndon BWTs, 0 mismatches")
    print(f"  [VERIFIED] Proper sub-interval joins == maximal joins (n=2..8): {sub_checked} Lyndon BWTs, 0 mismatches ({time.time()-t_v:.2f}s)")

    # 2B. Theorem C7 Heuristic Acceleration Benchmark (n=13..14, r_0 >= 12)
    high_r0_bench = []
    for n in (13, 14):
        for T_tup in gen_lyndon_binary(n):
            L, _ = bwt_encode(bytes(T_tup))
            if runs_of(L) >= 12:
                high_r0_bench.append(L)
    exp_no_c7 = exp_c7 = 0
    t1 = time.time()
    for L in high_r0_bench:
        q0, _, e0, _ = c_solve(L, use_c7=0, use_term4=1)
        exp_no_c7 += e0
    dt0 = time.time() - t1
    t2 = time.time()
    for L in high_r0_bench:
        q1, _, e1, _ = c_solve(L, use_c7=1, use_term4=1)
        exp_c7 += e1
    dt1 = time.time() - t2
    print(f"  [VERIFIED] Theorem C7 acceleration (r0>=12, n=13..14, {len(high_r0_bench)} BWTs): "
          f"C4-only={exp_no_c7:,} exp ({dt0:.3f}s) -> +C7={exp_c7:,} exp ({dt1:.3f}s) "
          f"[{-100*(1 - exp_c7/exp_no_c7):.1f}% expansions]")

    # 2C. EXHAUSTIVE [bounded-opt] OVER ALL VALID SINGLE-CYCLE BINARY BWTs FOR n = 2..15 (4,718 BWTs!)
    print("\n--- 2C. EXHAUSTIVE [bounded-opt] TABLE PER n (ALL VALID SINGLE-CYCLE BINARY BWTs, n = 2..15) ---")
    print(f"  {'n':>2} | {'L_2(n)':>6} | {'Unsolv':>6} | {'r0>=4':>5} | {'min Q*/r0 (r0>=4)':>22} | {'max Q*/r0 (r0>=4)':>22} | {'Min Witness T':>17} | {'Time':>6}")
    print("  " + "-" * 105)
    t_exh = time.time()
    total_lyndon = 0
    total_r0ge4 = 0
    total_lt_34 = 0
    total_le_34_r0ge6 = 0
    by_r0 = defaultdict(lambda: [999, None, -1, None, 0])  # min_q, min_w, max_q, max_w, count
    per_n_min_wits = []

    for n in range(2, 16):
        tn = time.time()
        cnt = 0
        cnt_ge4 = 0
        unsolved = 0
        min_rat = 999.0; min_w = None
        max_rat = -1.0; max_w = None
        seen_q = {}
        for T_tup in gen_lyndon_binary(n):
            cnt += 1
            comp = tuple(1 - x for x in T_tup)
            comp_min = min(comp[i:] + comp[:i] for i in range(n))
            if comp_min in seen_q:
                q, r0 = seen_q[comp_min]
            else:
                L, _ = bwt_encode(bytes(T_tup))
                q, r0, _, _ = c_solve(L, use_c7=1, use_term4=1, max_nodes=2500000)
                if q < 0:
                    unsolved += 1
                    continue
                seen_q[T_tup] = (q, r0)
            rec = by_r0[r0]
            rec[4] += 1
            if q < rec[0]:
                rec[0] = q
                rec[1] = (n, T_tup)
            if q > rec[2]:
                rec[2] = q
                rec[3] = (n, T_tup)
            if r0 >= 4:
                cnt_ge4 += 1
                rat = q / r0
                if q * 4 < 3 * r0:
                    total_lt_34 += 1
                if r0 >= 6 and q * 4 <= 3 * r0:
                    total_le_34_r0ge6 += 1
                if (rat, q) < (min_rat, min_w[2] if min_w else 999):
                    min_rat = rat
                    min_w = (T_tup, r0, q)
                if (rat, q) > (max_rat, max_w[2] if max_w else -1):
                    max_rat = rat
                    max_w = (T_tup, r0, q)
        total_lyndon += cnt
        total_r0ge4 += cnt_ge4
        dt = time.time() - tn
        if min_w:
            mn_s = f"{min_rat:.4f} (Q*={min_w[2]:2d}/r0={min_w[1]:2d})"
            mx_s = f"{max_rat:.4f} (Q*={max_w[2]:2d}/r0={max_w[1]:2d})"
            tw_s = "".join(map(str, min_w[0]))
            per_n_min_wits.append((n, min_w[0], min_w[1], min_w[2]))
        else:
            mn_s = mx_s = tw_s = "-"
        print(f"  {n:2d} | {cnt:6d} | {unsolved:6d} | {cnt_ge4:5d} | {mn_s:>22s} | {mx_s:>22s} | {tw_s:>17s} | {dt:5.2f}s", flush=True)
        assert unsolved == 0

    print(f"  TOTAL EXHAUSTIVE n=2..15: {total_lyndon:,} valid single-cycle binary BWTs (0 unsolved!) in {time.time()-t_exh:.2f}s")
    print(f"  [BOUNDED-OPT] Instances with r0>=4 & Q*/r0 < 3/4 (0.7500): {total_lt_34}/{total_r0ge4}")
    print(f"  [BOUNDED-OPT] Instances with r0>=6 & Q*/r0 <= 3/4 (0.7500): {total_le_34_r0ge6}")

    print("\n--- 2D. EXHAUSTIVE [bounded-opt] BREAKDOWN BY r_0 (ALL n = 2..15) ---")
    print(f"  {'r0':>3} | {'Count':>5} | {'min Q*':>6} | {'min Q*/r0':>9} | {'Min Witness (n, T)':>24} | {'max Q*':>6} | {'max Q*/r0':>9} | {'Max Witness (n, T)':>24}")
    print("  " + "-" * 102)
    r0_min_wits = []
    for r0 in sorted(by_r0):
        mn, mn_w, mx, mx_w, cnt = by_r0[r0]
        r0_min_wits.append((mn_w[0], mn_w[1], r0, mn))
        mn_str = f"n={mn_w[0]:2d} {''.join(map(str, mn_w[1]))}"
        mx_str = f"n={mx_w[0]:2d} {''.join(map(str, mx_w[1]))}"
        print(f"  {r0:3d} | {cnt:5d} | {mn:6d} | {mn/r0:9.4f} | {mn_str:>24s} | {mx:6d} | {mx/r0:9.4f} | {mx_str:>24s}")

    # 2E. TARGETED HIGH-COALESCING SEARCH OVER n = 16..24 (r_0 <= 16 + near-alternating n=16..22)
    print("\n--- 2E. TARGETED HIGH-COALESCING EXACT SEARCH (n = 16..24, r_0 <= 16) ---")
    t_targ = time.time()
    rng = random.Random(20261117)
    targ_cands = set()

    # (i) Near-alternating sparse-swap perturbations of (10...) and (01...) for n = 16..22
    for n in range(16, 23):
        for start in (1, 0):
            base = [(start ^ (i & 1)) for i in range(n)]
            L0 = bytes(base)
            res = ibwt_single_cycle(L0)
            if res is not None and runs_of(L0) <= 18:
                targ_cands.add((n, res[0], L0))
            for p1 in range(n - 1):
                a = base[:]; a[p1], a[p1 + 1] = a[p1 + 1], a[p1]
                L = bytes(a)
                if runs_of(L) <= 16:
                    res = ibwt_single_cycle(L)
                    if res is not None:
                        targ_cands.add((n, res[0], L))
            for p1 in range(n):
                a = base[:]; a[p1] ^= 1
                L = bytes(a)
                if runs_of(L) <= 16:
                    res = ibwt_single_cycle(L)
                    if res is not None:
                        targ_cands.add((n, res[0], L))
            for p1 in range(n - 1):
                for p2 in range(p1 + 2, n - 1):
                    a = base[:]
                    a[p1], a[p1 + 1] = a[p1 + 1], a[p1]
                    a[p2], a[p2 + 1] = a[p2 + 1], a[p2]
                    L = bytes(a)
                    if runs_of(L) <= 16:
                        res = ibwt_single_cycle(L)
                        if res is not None:
                            targ_cands.add((n, res[0], L))

    # (ii) Classic structured words: Thue-Morse T_4 (n=16), Period-Doubling PD_4 (n=16), Fibonacci F_7 (n=21)
    tm4 = [0]
    for _ in range(4): tm4 = tm4 + [1 - x for x in tm4]
    pd4 = [0]
    for _ in range(4): pd4 = [y for x in pd4 for y in ([0, 1] if x == 0 else [0, 0])]
    fa, fb = [0], [0, 1]
    while len(fb) < 21: fa, fb = fb, fb + fa
    for arr in (tm4, pd4, fb):
        T = bytes(arr)
        L, _ = bwt_encode(T)
        if ibwt_single_cycle(L) is not None:
            targ_cands.add((len(T), T, L))

    # (iii) Stretched variants of the n=16 Q*=11/r0=14 witness L=1011001010101010 (n=17..24)
    w16_L = [1, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0]
    for pos in range(len(w16_L)):
        for extra in (1, 2, 4, 6, 8):
            arr = w16_L[:pos] + [w16_L[pos]] * extra + w16_L[pos:]
            if len(arr) <= 24:
                L = bytes(arr)
                res = ibwt_single_cycle(L)
                if res is not None:
                    targ_cands.add((len(L), res[0], L))

    # (iv) Random high-r0 (10..16) single-cycle binary BWTs for n = 16..24
    for n in range(16, 25):
        for _ in range(500):
            p_alt = rng.uniform(0.55, 0.92)
            arr = [rng.randrange(2)]
            for _i in range(n - 1):
                arr.append(arr[-1] ^ (1 if rng.random() < p_alt else 0))
            L = bytes(arr)
            if 6 <= runs_of(L) <= 16:
                res = ibwt_single_cycle(L)
                if res is not None:
                    targ_cands.add((n, res[0], L))

    # Solve balanced sample across r0 in {2, 6, 8, 10, 12, 14, 16, 18} within ~12s budget
    by_r0_pool = defaultdict(list)
    for n, T, L in sorted(targ_cands, key=lambda x: (x[0], x[2])):
        by_r0_pool[runs_of(L)].append((n, T, L))

    targ_results = []
    targ_solved = 0
    targ_unsolved = 0
    targ_lt_34 = 0
    for r0 in sorted(by_r0_pool):
        pool = by_r0_pool[r0]
        # Prioritize smallest n first (fastest exact solve, plus diverse sample)
        cap = 45 if r0 <= 14 else (20 if r0 == 16 else 4)
        t_r = time.time()
        r_best = None
        r_sol = 0
        for n, T, L in pool[:cap]:
            if time.time() - t_r > 2.5:
                break
            q, r0_v, exp, _ = c_solve(L, use_c7=1, use_term4=1, max_nodes=1200000)
            if q < 0:
                targ_unsolved += 1
                continue
            targ_solved += 1
            r_sol += 1
            rat = q / r0_v
            if r0_v >= 4 and q * 4 < 3 * r0_v:
                targ_lt_34 += 1
            targ_results.append((rat, q, r0_v, n, T, L, exp))
            if r_best is None or (rat, q, n) < (r_best[0], r_best[1], r_best[2]):
                r_best = (rat, q, n, T, L, exp)
        if r_best:
            print(f"  targeted r0={r0:2d} (pool={len(pool):4d}, solved={r_sol:2d}): "
                  f"MIN={r_best[0]:.4f} (Q*={r_best[1]:2d}/r0={r0:2d}, n={r_best[2]:2d}) "
                  f"L={''.join(map(str, r_best[4]))}", flush=True)

    targ_results.sort()
    print(f"  Targeted n=16..24 summary: solved={targ_solved}, unsolved={targ_unsolved}, "
          f"Q*/r0 < 3/4 (r0>=4): {targ_lt_34}/{targ_solved} ({time.time()-t_targ:.2f}s)")

    # 2F. Pure-Python Replay & Byte-Exact Decode Verification of Key Extremal Witnesses!
    print("\n--- 2F. PURE-PYTHON REPLAY & BYTE-EXACT VERIFICATION OF EXTREMAL OPTIMAL TRACES ---")
    witnesses_to_replay = [
        ("n=4  r0=4  (global min 3/4)",  bytes([0, 0, 1, 1])),
        ("n=7  r0=6  (min r0=6, 5/6)",   bytes([0, 0, 0, 1, 1, 0, 1])),
        ("n=9  r0=8  (min r0=8, 7/8)",   bytes([0, 0, 0, 1, 0, 0, 1, 1, 1])),
        ("n=10 r0=10 (min n<=15 r0>=6, 4/5)", bytes([0, 0, 0, 1, 0, 1, 1, 1, 0, 1])),
        ("n=12 r0=12 (min r0=12, 5/6)",  bytes([0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 1, 1])),
        ("n=15 r0=14 (min r0=14 n<=15)", bytes([0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1])),
    ]
    # Add top 2 targeted sub-19/24 witnesses (r0 >= 6)
    sub_1924 = [x for x in targ_results if x[2] >= 6 and x[1] * 24 < 19 * x[2]]
    seen_t = set()
    for rat, q, r0, n, T, L, _ in sub_1924:
        if (n, r0) not in seen_t and len(seen_t) < 3:
            seen_t.add((n, r0))
            witnesses_to_replay.append((f"n={n:2d} r0={r0:2d} (sub-19/24! {q}/{r0})", T))

    for label, T in witnesses_to_replay:
        L, _ = bwt_encode(T)
        q, r0, exp, mvs = c_solve(L, use_c7=1, use_term4=0, max_nodes=2500000, want_path=True)
        assert q >= 0
        dp_seq, dxi_seq, mv_names = py_replay_and_verify(L, T, mvs, q)
        print(f"  [VERIFIED] {label:34s}: Q*={q:2d}/r0={r0:2d} ({q/r0:.4f}) | "
              f"L={''.join(map(str,L))} | dP={dp_seq} | dXi={dxi_seq}")


def main():
    t_all = time.time()
    run_part1_verification()
    run_part2_exact_frontier()
    h = hashlib.sha256(open(__file__, "rb").read()).hexdigest()
    print("\n" + "=" * 100)
    print(f"ALL CYCLE 17 PART 1 & PART 2 CHECKS PASSED 100% EXACT in {time.time() - t_all:.2f}s!")
    print(f"SHA-256 ({__file__}): {h}")
    print("=" * 100)


if __name__ == "__main__":
    main()
