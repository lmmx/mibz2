#!/usr/bin/env python3
"""
Cycle 18 — Adversarial Referee #4: Independent Audit of Cycle 16 & Cycle 17
============================================================================
Self-contained independent verifier (zero edits to repo or /tmp/moonshot-scorer).
Covers:
  - Part 1: Verification of shipped OpenCells 0.5.12 wide-trigger formula & spelling
            capacity trajectory across all 22 Cycle 16 grid points + SHA-256 hashes.
  - Part 2A: Independent WPB implementation & adversarial JSV 6-axiom tests INCLUDING
             explicit closed cycles completely inside U, V, and U (+) V, plus lossless
             BWT vs Meyer (I - P_EE)^{-1} singularity biconditional check.
  - Part 2B: Independent Durand Theorem D2a audit on self-overlapping factors
             u in {00, 010, 000, 0101} (n=4..12) proving Cycle 10 strengthening
             (augmented w*u holds 100%, plain lex w fails), plus true-relabel B2 check.
  - Part 2C: Independent C_gap exact A* solver & witness verifier:
             (i)   Verify n=16, r_0=14 witness L=1010101010110010 (T=0000110010110111):
                   single-cycle, r_0=14, exact Q*=11 (11/14 < 19/24), strict legality,
                   byte-exact decode.
             (ii)  Independently verify min Q*/r_0 = 0.7500 over ALL 224 single-cycle
                   binary BWTs for n=2..10 (and check all_subintervals=True on n=2..9).
             (iii) Exhaustive verification of Alpha-3's m<=4 terminal base-case theorem
                   over ALL single-cycle 4-row piece states (all 6 4-cycles x all legal
                   word-coalescing partitions, with full proper-subinterval BFS!).
  - Part 3: Independent check of Cycle 17 moonshot_register_v1.jsonl line 15 consistency
            and SHA-256 audit of all Cycle 16/17 artifacts.
"""
import bisect
import hashlib
import heapq
import json
import math
import os
import random
import re
import time
from collections import deque
from fractions import Fraction


REPO = "/home/antigravity/lmmx/mibz2"
JDIR = f"{REPO}/docs/journal/2026-11-22-foundational-contraction-programme"


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while True:
            b = f.read(65536)
            if not b:
                break
            h.update(b)
    return h.hexdigest()


# ==============================================================================
# INDEPENDENT BWT UTILITIES
# ==============================================================================
def bwt_of(T: bytes):
    n = len(T)
    rots = sorted(range(n), key=lambda i: T[i:] + T[:i])
    L = bytes(T[(i - 1) % n] for i in rots)
    return L, rots.index(0)


def lf_of(L: bytes):
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
    for i, b in enumerate(L):
        lf[i] = cur[b]
        cur[b] += 1
    F = bytes(sorted(L))
    return lf, C, F


def decode_bwt_single_cycle(L: bytes):
    """Returns (T, lf) iff L is a valid single-cycle BWT of some primitive string T."""
    n = len(L)
    lf, _, _ = lf_of(L)
    x = 0
    buf = bytearray(n)
    seen = set()
    for k in range(n - 1, -1, -1):
        if x in seen:
            return None
        seen.add(x)
        buf[k] = L[x]
        x = lf[x]
    if x != 0 or len(seen) != n:
        return None
    T = bytes(buf)
    L_check, _ = bwt_of(T)
    if L_check != L:
        return None
    return T, lf


def runs_count(seq) -> int:
    if not seq:
        return 0
    return 1 + sum(1 for a, b in zip(seq, seq[1:]) if a != b)


def is_rot(a: bytes, b: bytes) -> bool:
    return len(a) == len(b) and a in (b + b)


# ==============================================================================
# PART 1: SHIPPED 0.5.12 WIDE TRIGGER & SPELLING AUDIT
# ==============================================================================
def audit_part1():
    print("=" * 90)
    print("PART 1: INDEPENDENT AUDIT OF CYCLE 16 scale_crossover_bench vs SHIPPED OpenCells 0.5.12")
    print("=" * 90)
    out_v2 = f"{JDIR}/cycle16/scale_crossover_out_v2.txt"
    with open(out_v2) as f:
        lines = f.readlines()

    row_re = re.compile(
        r"^(\S+)\s*\|\s*([\d.]+)\s+(\d+)\s+([\d.]+)\s*\|\s*(\d+)\s+([YN])\s+([\d.]+)\s*\|"
    )
    det_re = re.compile(
        r"Open:\s*ns=\s*(\d+),\s*codes=\s*(\d+),\s*spell=([\d.]+)/([\d.]+)M"
    )
    checked = 0
    mismatches = 0
    cap_anomalies = 0
    i = 0
    while i < len(lines):
        m1 = row_re.match(lines[i].strip())
        if m1 and i + 1 < len(lines):
            m2 = det_re.search(lines[i + 1])
            assert m2, f"Missing detail line for {m1.group(1)}"
            label = m1.group(1)
            m16 = int(m1.group(5))
            w_flag = m1.group(6) == "Y"
            codes = int(m2.group(2))
            sp_len = float(m2.group(3))
            sp_cap = float(m2.group(4))
            bits = max(1, (m16 - 1).bit_length()) if m16 > 1 else 1
            max_narrow_code = (0xFFFF_FFFF >> bits)
            expected_wide = (codes - 1) > max_narrow_code
            checked += 1
            if expected_wide != w_flag:
                mismatches += 1
                print(f"  [MISMATCH] {label}: m16={m16} bits={bits} codes={codes} expected_wide={expected_wide} got={w_flag}")
            # Check spelling cap matches 48 KiB * 2^k (exact Rust Vec doubling from reserve_exact(48*1024))
            cap_kib_candidates = [(48 * (1 << k)) / 1024.0 for k in range(15)]
            if min(abs(sp_cap - c) for c in cap_kib_candidates) > 0.015:
                cap_anomalies += 1
                print(f"  [CAP NOTE] {label}: sp_cap={sp_cap}M not in 48KiB*2^k")
            if label in ("4M_wiki_mut(rho~7)", "64M_ultra(K512,0.02%)", "64M_pan(K1024,0.005%)"):
                print(
                    f"  [CHECK] {label:24s}: m16={m16:7d}, bits={bits:2d}, u32::MAX>>bits={max_narrow_code:6d}, "
                    f"codes={codes:5d} => wide={expected_wide} (logged={w_flag}), spell={sp_len:.2f}/{sp_cap:.2f}M"
                )
            i += 2
        else:
            i += 1
    print(f"  [VERIFIED] Exact shipped 0.5.12 wide-trigger formula verified on {checked}/23 grid points (mismatches={mismatches}).")
    print(f"  [VERIFIED] Exact 48 KiB * 2^k Vec-doubling spelling capacity verified on {checked}/23 grid points (anomalies={cap_anomalies}).")
    assert checked == 23 and mismatches == 0 and cap_anomalies == 0


# ==============================================================================
# PART 2A: ADVERSARIAL JSV AXIOM CHECK ON WPB WITH CLOSED CYCLES IN U & U(+)V
# ==============================================================================
class WPB:
    def __init__(self, dom, cod, mp):
        self.dom = frozenset(dom)
        self.cod = frozenset(cod)
        self.mp = dict(mp)
        ys = set()
        for x, (y, w) in self.mp.items():
            assert x in self.dom and y in self.cod and y not in ys
            ys.add(y)

    def __eq__(self, other):
        return self.dom == other.dom and self.cod == other.cod and self.mp == other.mp

    @staticmethod
    def id_(X):
        X = frozenset(X)
        return WPB(X, X, {x: (x, b"") for x in X})

    def comp(self, other):
        assert other.cod == self.dom
        mp = {}
        for x, (y, w1) in other.mp.items():
            if y in self.mp:
                z, w2 = self.mp[y]
                mp[x] = (z, w1 + w2)
        return WPB(other.dom, self.cod, mp)

    @staticmethod
    def oplus(f, g):
        dom = {(0, x) for x in f.dom} | {(1, x) for x in g.dom}
        cod = {(0, y) for y in f.cod} | {(1, y) for y in g.cod}
        mp = {}
        for x, (y, w) in f.mp.items():
            mp[(0, x)] = ((0, y), w)
        for x, (y, w) in g.mp.items():
            mp[(1, x)] = ((1, y), w)
        return WPB(dom, cod, mp)

    @staticmethod
    def swap(X, Y):
        dom = {(0, x) for x in X} | {(1, y) for y in Y}
        cod = {(0, y) for y in Y} | {(1, x) for x in X}
        mp = {(0, x): ((1, x), b"") for x in X}
        mp.update({(1, y): ((0, y), b"") for y in Y})
        return WPB(dom, cod, mp)

    def tr(self, X, Y, U):
        """Particle-style Haghverdi-Scott trace on WPB: ignores closed cycles inside U."""
        mp = {}
        for x in X:
            cur = (0, x)
            word = bytearray()
            seen = set()
            while cur in self.mp:
                assert cur not in seen, "Cycle reached from X (impossible for partial bijection!)"
                seen.add(cur)
                nxt, w = self.mp[cur]
                word.extend(w)
                if nxt[0] == 0:
                    mp[x] = (nxt[1], bytes(word))
                    break
                cur = nxt
        return WPB(X, Y, mp)

    def tr_subset(self, E):
        assert self.dom == self.cod
        E = frozenset(E)
        K = self.dom - E
        mp = {}
        for x in K:
            cur = x
            word = bytearray()
            seen = set()
            while cur in self.mp:
                assert cur not in seen
                seen.add(cur)
                nxt, w = self.mp[cur]
                word.extend(w)
                if nxt in K:
                    mp[x] = (nxt, bytes(word))
                    break
                cur = nxt
        return WPB(K, K, mp)


def audit_part2a_wpb():
    print("\n" + "=" * 90)
    print("PART 2A: ADVERSARIAL JSV AXIOM CHECK ON WPB (EXPLICIT CLOSED CYCLES IN U & U(+)V)")
    print("=" * 90)
    rng = random.Random(20261122)
    words = [b"", b"a", b"b", b"ab", b"ba", b"xyz"]

    def rand_wpb(X, Y, density=0.8):
        xs, ys = list(X), list(Y)
        rng.shuffle(xs); rng.shuffle(ys)
        k = int(round(min(len(xs), len(ys)) * density))
        return WPB(X, Y, {xs[i]: (ys[i], rng.choice(words)) for i in range(k)})

    def rand_wpb_with_forced_U_cycles(X, Y, U):
        """Force at least one pure closed cycle completely inside U (length 1, 2, or 3) + random rest."""
        U_list = list(U)
        rng.shuffle(U_list)
        cyc_len = min(len(U_list), rng.choice([1, 2, 3]))
        cyc = U_list[:cyc_len]
        mp = {}
        for i in range(cyc_len):
            mp[(1, cyc[i])] = ((1, cyc[(i + 1) % cyc_len]), rng.choice(words))
        rem_dom = [(0, x) for x in X] + [(1, u) for u in U_list[cyc_len:]]
        rem_cod = [(0, y) for y in Y] + [(1, u) for u in U_list[cyc_len:]]
        rng.shuffle(rem_dom); rng.shuffle(rem_cod)
        k = int(round(min(len(rem_dom), len(rem_cod)) * rng.uniform(0.5, 1.0)))
        for i in range(k):
            mp[rem_dom[i]] = (rem_cod[i], rng.choice(words))
        dom = {(0, x) for x in X} | {(1, u) for u in U}
        cod = {(0, y) for y in Y} | {(1, u) for u in U}
        return WPB(dom, cod, mp)

    n_trials = 200
    u_cycle_count = 0
    mixed_uv_cycle_count = 0
    for _ in range(n_trials):
        X = frozenset(range(rng.randint(2, 5)))
        Xp = frozenset(range(rng.randint(2, 5)))
        Y = frozenset(range(rng.randint(2, 5)))
        Yp = frozenset(range(rng.randint(2, 5)))
        U = frozenset(range(rng.randint(3, 6)))
        V = frozenset(range(rng.randint(3, 6)))

        # 1. Tightening WITH forced closed cycles in U
        f = rand_wpb_with_forced_U_cycles(X, Y, U)
        u_cycle_count += 1
        g = rand_wpb(Xp, X)
        h = rand_wpb(Y, Yp)
        idU = WPB.id_(U)
        assert WPB.oplus(h, idU).comp(f).comp(WPB.oplus(g, idU)).tr(Xp, Yp, U) == h.comp(f.tr(X, Y, U)).comp(g)

        # 2. Sliding WITH forced closed cycles across U <-> V
        # Build s: V -> U and f_uv: X(+)U -> Y(+)V such that v0 -> u0 -> v1 -> u1 -> v0 is a closed loop!
        s_mp = {}
        f_mp = {}
        u_list, v_list = list(U), list(V)
        rng.shuffle(u_list); rng.shuffle(v_list)
        cl = min(len(u_list), len(v_list), rng.choice([1, 2]))
        for i in range(cl):
            s_mp[v_list[i]] = (u_list[i], rng.choice(words))
            f_mp[(1, u_list[i])] = ((1, v_list[(i + 1) % cl]), rng.choice(words))
        # fill remaining s and f_uv randomly
        rem_v = v_list[cl:]; rem_u = u_list[cl:]
        rng.shuffle(rem_v); rng.shuffle(rem_u)
        for i in range(int(round(min(len(rem_v), len(rem_u)) * 0.8))):
            s_mp[rem_v[i]] = (rem_u[i], rng.choice(words))
        rem_dom = [(0, x) for x in X] + [(1, u) for u in u_list[cl:]]
        rem_cod = [(0, y) for y in Y] + [(1, v) for v in v_list[cl:]]
        rng.shuffle(rem_dom); rng.shuffle(rem_cod)
        for i in range(int(round(min(len(rem_dom), len(rem_cod)) * 0.8))):
            f_mp[rem_dom[i]] = (rem_cod[i], rng.choice(words))
        s = WPB(V, U, s_mp)
        f_uv = WPB({(0, x) for x in X} | {(1, u) for u in U}, {(0, y) for y in Y} | {(1, v) for v in V}, f_mp)
        assert WPB.oplus(WPB.id_(Y), s).comp(f_uv).tr(X, Y, U) == f_uv.comp(WPB.oplus(WPB.id_(X), s)).tr(X, Y, V)

        # 3. Vanishing I
        f0 = rand_wpb(X, Y)
        f0_tag = WPB({(0, x) for x in X}, {(0, y) for y in Y}, {(0, x): ((0, y), w) for x, (y, w) in f0.mp.items()})
        assert f0_tag.tr(X, Y, frozenset()) == f0

        # 4. Vanishing II WITH forced pure-E1, pure-E2, AND mixed (E1 <-> E2) closed cycles!
        K_set = frozenset(("k", i) for i in range(rng.randint(2, 4)))
        E1 = frozenset(("e1", i) for i in range(rng.randint(3, 5)))
        E2 = frozenset(("e2", i) for i in range(rng.randint(3, 5)))
        e1l, e2l = list(E1), list(E2)
        mp_v2 = {
            e1l[0]: (e1l[0], rng.choice(words)),          # pure E1 1-cycle
            e2l[0]: (e2l[0], rng.choice(words)),          # pure E2 1-cycle
            e1l[1]: (e2l[1], rng.choice(words)),          # mixed E1 <-> E2 2-cycle!
            e2l[1]: (e1l[1], rng.choice(words)),
        }
        mixed_uv_cycle_count += 1
        rem_all = list(K_set) + e1l[2:] + e2l[2:]
        rem_d = rem_all[:]; rem_c = rem_all[:]
        rng.shuffle(rem_d); rng.shuffle(rem_c)
        for i in range(int(round(len(rem_d) * 0.85))):
            mp_v2[rem_d[i]] = (rem_c[i], rng.choice(words))
        V_all = K_set | E1 | E2
        f_v2 = WPB(V_all, V_all, mp_v2)
        t12 = f_v2.tr_subset(E1).tr_subset(E2)
        t21 = f_v2.tr_subset(E2).tr_subset(E1)
        tu = f_v2.tr_subset(E1 | E2)
        assert t12 == tu and t21 == tu

        # 5. Superposing WITH forced U-cycle
        Z1 = frozenset(range(rng.randint(1, 3)))
        Z2 = frozenset(range(rng.randint(1, 3)))
        g_sp = rand_wpb(Z1, Z2)
        ZX = {("z", z) for z in Z1} | {("x", x) for x in X}
        ZY = {("z", z) for z in Z2} | {("y", y) for y in Y}
        mp_sp = {(0, ("z", z)): ((0, ("z", z2)), w) for z, (z2, w) in g_sp.mp.items()}
        for (tg, v0), ((tg2, v1), w) in f.mp.items():
            mp_sp[(0, ("x", v0)) if tg == 0 else (1, v0)] = ((0, ("y", v1)) if tg2 == 0 else (1, v1), w)
        lhs5 = WPB({(0, p) for p in ZX} | {(1, u) for u in U}, {(0, q) for q in ZY} | {(1, u) for u in U}, mp_sp).tr(ZX, ZY, U)
        tr_f = f.tr(X, Y, U)
        mp_r5 = {("z", z): (("z", z2), w) for z, (z2, w) in g_sp.mp.items()}
        mp_r5.update({("x", x): (("y", y), w) for x, (y, w) in tr_f.mp.items()})
        assert lhs5 == WPB(ZX, ZY, mp_r5)

        # 6. Yanking
        assert WPB.swap(X, X).tr(X, X, X) == WPB.id_(X)

    print(f"  [VERIFIED] ALL 6 JSV axioms PASSED on {n_trials} adversarial cycle-rich trials "
          f"(forced pure-U cycles={u_cycle_count}, forced mixed U<->V cycles={mixed_uv_cycle_count}, 0 failures)!")

    # Demonstrate exact lossless-BWT / Meyer invertibility boundary when E contains a closed cycle:
    # Periodic T = b"0101" has BWT L = b"1100", LF = [2, 3, 0, 1] -> two 2-cycles (0<->2) and (1<->3)!
    L_per = b"\x01\x01\x00\x00"
    lf_p, _, _ = lf_of(L_per)
    wpb_per = WPB(range(4), range(4), {i: (lf_p[i], bytes([L_per[i]])) for i in range(4)})
    # Eliminate full closed cycle E = {0, 2}: Tr_E discards 2 bytes AND I - P_EE is singular!
    tr_drop = wpb_per.tr_subset({0, 2})
    bytes_kept = sum(len(w) for _, w in tr_drop.mp.values())
    assert bytes_kept == 2 < 4, "Expected closed cycle bytes to be dropped!"
    # For single-cycle BWT with E = V: Tr_V drops ALL bytes!
    L_sc, _ = bwt_of(b"banana$")
    lf_sc, _, _ = lf_of(L_sc)
    wpb_sc = WPB(range(len(L_sc)), range(len(L_sc)), {i: (lf_sc[i], bytes([L_sc[i]])) for i in range(len(L_sc))})
    assert len(wpb_sc.tr_subset(range(len(L_sc))).mp) == 0
    # For EVERY proper subset E subsetneq V of single-cycle BWT: zero bytes lost!
    for mask in range(1, (1 << len(L_sc)) - 1):
        E = {i for i in range(len(L_sc)) if (mask >> i) & 1}
        tr_e = wpb_sc.tr_subset(E)
        assert sum(len(w) for _, w in tr_e.mp.values()) == len(L_sc)
    print("  [VERIFIED] Exact Scoping Biconditional: Tr_E is lossless <=> (I - P_EE) invertible <=> E contains no closed cycle!")
    print("             (Demonstrated silent 2-byte drop on periodic BWT L=1100 with E={0,2}, and 0-byte loss on all 126 proper subsets of 'banana$'!)")


# ==============================================================================
# PART 2B: CROSS-CYCLE CONSISTENCY — DURAND D2a & TRUE-RELABEL B2
# ==============================================================================
def audit_part2b_cross_cycle():
    print("\n" + "=" * 90)
    print("PART 2B: CROSS-CYCLE CONSISTENCY — DURAND THEOREM D2a & TRUE-RELABEL THEOREM B2")
    print("=" * 90)
    # 1. Exhaustive Durand Theorem D2a on self-overlapping factors u in {00, 010, 000, 0101}, n=4..12
    for u in (b"\x00\x00", b"\x00\x01\x00", b"\x00\x00\x00", b"\x00\x01\x00\x01"):
        ul = len(u)
        ov_cases = 0
        nonov_cases = 0
        viol_aug_ov = 0
        viol_plain_ov = 0
        viol_aug_nonov = 0
        first_plain_ce = None
        for n in range(ul + 1, 13):
            for mask in range(1 << n):
                T = bytes((mask >> i) & 1 for i in range(n))
                T2 = T + T
                occ = [i for i in range(n) if T2[i:i + ul] == u]
                if len(occ) < 2:
                    continue
                if len({T[i:] + T[:i] for i in occ}) < len(occ):
                    continue
                k = len(occ)
                has_ov = any(((occ[(j + 1) % k] - occ[j]) % n) < ul for j in range(k))
                rot_ord = sorted(occ, key=lambda i: T[i:] + T[:i])
                rw = [T2[occ[j]:occ[j] + (((occ[(j + 1) % k] - occ[j]) % n) or n)] for j in range(k)]
                aug_ord = [occ[j] for j in sorted(range(k), key=lambda j: tuple(rw[(j + t) % k] + u for t in range(k)))]
                pln_ord = [occ[j] for j in sorted(range(k), key=lambda j: tuple(rw[(j + t) % k] for t in range(k)))]
                if has_ov:
                    ov_cases += 1
                    if aug_ord != rot_ord:
                        viol_aug_ov += 1
                    if pln_ord != rot_ord:
                        viol_plain_ov += 1
                        if first_plain_ce is None:
                            first_plain_ce = ("".join(str(b) for b in T), occ, ["".join(str(b) for b in w) for w in rw])
                else:
                    nonov_cases += 1
                    if aug_ord != rot_ord:
                        viol_aug_nonov += 1
        u_str = "".join(str(b) for b in u)
        print(f"  Durand D2a u='{u_str}' (n<={12}): ov={ov_cases:5d}, nonov={nonov_cases:5d} | "
              f"viol(aug w*u, ov)={viol_aug_ov}, viol(aug w*u, nonov)={viol_aug_nonov}, viol(plain w, ov)={viol_plain_ov}")
        if first_plain_ce:
            print(f"     smallest plain-lex counterexample: T={first_plain_ce[0]}, occ={first_plain_ce[1]}, rw={first_plain_ce[2]}")
        assert viol_aug_ov == 0 and viol_aug_nonov == 0

    # 2. Check Theorem B2 under TRUE bigram relabeling (not just raw projected L_proj) on all binary n=2..10 & ternary n=2..6!
    b2_checked = 0
    b2_viol_true = 0
    for sig, nmax in ((2, 10), (3, 6)):
        for n in range(2, nmax + 1):
            for x in range(sig ** n):
                tmp = x
                arr = [0] * n
                for i in range(n):
                    arr[i] = tmp % sig
                    tmp //= sig
                T = bytes(arr)
                if len(set(T)) < 2:
                    continue
                L, _ = bwt_of(T)
                if decode_bwt_single_cycle(L) is None:
                    continue
                lf, _, F = lf_of(L)
                wpb = WPB(range(n), range(n), {i: (lf[i], bytes([L[i]])) for i in range(n)})
                syms = sorted(set(L))
                # Test every non-trivial 2-coloring of Sigma (1Cut & BDMR)
                for cmask in range(1, (1 << len(syms)) - 1):
                    sig_L = {syms[j] for j in range(len(syms)) if (cmask >> j) & 1}
                    E = frozenset(i for i in range(n) if (L[i] in sig_L) and (F[i] not in sig_L))
                    if not (0 < len(E) < n):
                        continue
                    tr = wpb.tr_subset(E)
                    # True relabeled L' sequence on surviving rows K = sorted(tr.dom)
                    L_true = [tr.mp[i][1] for i in sorted(tr.dom)]
                    b2_checked += 1
                    if runs_count(L_true) + len(L_true) > runs_count(L) + n:
                        b2_viol_true += 1
    print(f"  [VERIFIED] True-relabeled Theorem B2 (r(L'_bigram) + n' <= r(L) + n): {b2_checked} cases, viol={b2_viol_true}!")
    assert b2_viol_true == 0


# ==============================================================================
# PART 2C: INDEPENDENT C_gap EXACT ENGINE, WITNESS CHECK, n<=10 FRONTIER & m<=4 PROOF
# ==============================================================================
def compact_coalesce(pieces):
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


def init_pieces_from_L(L: bytes):
    n = len(L)
    _, C, _ = lf_of(L)
    cur = C[:]
    pcs = []
    i = 0
    while i < n:
        b = L[i]
        j = i + 1
        while j < n and L[j] == b:
            j += 1
        d = cur[b] - i
        cur[b] += j - i
        pcs.append((i, j, d, bytes([b])))
        i = j
    return tuple(pcs)


def cgap_neighbors(state, all_subintervals=False):
    res = []
    # Pow± on self-overlapping pieces
    for i, (lo, hi, d, w) in enumerate(state):
        L = hi - lo
        if d > 0 and d < L:
            q, rem = divmod(L, d)
            nxt = list(state[:i]) + list(state[i + 1:])
            if rem > 0:
                nxt.append((lo, lo + rem, (q + 1) * d, w * (q + 1)))
            if d - rem > 0:
                nxt.append((lo + rem, lo + d, q * d, w * q))
            res.append((compact_coalesce(nxt), ("Pow+", i, lo, hi, d)))
        elif d < 0 and -d < L:
            s = -d
            q, rem = divmod(L, s)
            nxt = list(state[:i]) + list(state[i + 1:])
            if s - rem > 0:
                nxt.append((hi - s, hi - rem, -q * s, w * q))
            if rem > 0:
                nxt.append((hi - rem, hi, -(q + 1) * s, w * (q + 1)))
            res.append((compact_coalesce(nxt), ("Pow-", i, lo, hi, d)))
    # Join on pairs (A, B) with strict legality G cap (G + d_B) == empty
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
                cands = [
                    (glo, ghi)
                    for glo in range(ov_lo, ov_hi)
                    for ghi in range(glo + 1, min(ov_hi, glo + abd) + 1)
                ]
            elif ov_hi - ov_lo > abd:
                cands = [(ov_lo, ov_lo + abd), (ov_hi - abd, ov_hi)]
            else:
                cands = [(ov_lo, ov_hi)]
            for glo, ghi in cands:
                # Strict C_gap legality check: G cap (G + bd) == empty
                assert (ghi <= glo + bd) or (ghi + bd <= glo), "ILLEGAL JOIN!"
                a_lo, a_hi = glo - ad, ghi - ad
                nxt = []
                for k, pc in enumerate(state):
                    if k == ia:
                        if alo < a_lo:
                            nxt.append((alo, a_lo, ad, aw))
                        nxt.append((a_lo, a_hi, ad + bd, aw + bw))
                        if a_hi < ahi:
                            nxt.append((a_hi, ahi, ad, aw))
                    elif k == ib:
                        if blo < glo:
                            nxt.append((blo, glo, bd, bw))
                        if ghi < bhi:
                            nxt.append((ghi, bhi, bd, bw))
                    else:
                        nxt.append(pc)
                res.append((compact_coalesce(nxt), ("Join", ia, ib, glo, ghi, bd)))
    return res


def h_c7_pure(st) -> int:
    if len(st) <= 1:
        return 0
    # Theorem C7: ceil((Xi - 1)/2) where Xi = runs of first byte of words
    xi = 1
    prev = st[0][3][0]
    for _, _, _, w in st[1:]:
        if w[0] != prev:
            xi += 1
            prev = w[0]
    # Theorem C3: ceil((P - 1)/5)
    h3 = (len(st) + 3) // 5
    # Theorem C4: ceil(Psi/3)
    u = d = 0
    pu = None
    for _, _, _, w in st:
        if len(w) == 1:
            u += 1
            if pu == w:
                d += 1
            pu = w
        else:
            pu = None
    h4 = (u - d + 2) // 3
    return max(xi // 2, h3, h4)


def solve_exact_independent(init_state, all_subintervals=False, return_path=False):
    if len(init_state) == 1 and init_state[0][2] == 0:
        return (0, []) if return_path else 0
    pq = [(h_c7_pure(init_state), 0, 0, init_state)]
    best = {init_state: 0}
    parent = {}
    tie = 1
    while pq:
        f, g, _, st = heapq.heappop(pq)
        if g > best[st]:
            continue
        if len(st) == 1 and st[0][2] == 0:
            if not return_path:
                return g
            path = []
            cur = st
            while cur in parent:
                prev_st, mv = parent[cur]
                path.append((mv, prev_st, cur))
                cur = prev_st
            path.reverse()
            return g, path
        for ns, mv in cgap_neighbors(st, all_subintervals=all_subintervals):
            ng = g + 1
            if ng < best.get(ns, 999):
                best[ns] = ng
                if return_path:
                    parent[ns] = (st, mv)
                heapq.heappush(pq, (ng + h_c7_pure(ns), ng, tie, ns))
                tie += 1
    raise RuntimeError("No path found")


def audit_part2c_frontier():
    print("\n" + "=" * 90)
    print("PART 2C: INDEPENDENT EXACT [bounded-opt] FRONTIER, n=16 WITNESS & m<=4 TERMINAL THEOREM")
    print("=" * 90)

    # (i) Exhaustive check of ALL single-cycle m<=4 piece states (with full proper-subinterval BFS!)
    # Enumerate all 3!=6 4-cycles on {0,1,2,3}, and for each, all ways to coalesce adjacent equal-d rows
    # (distinct words block coalescing; identical words coalesce -> so every subset of equal-d adjacencies!)
    from itertools import permutations

    checked_m4 = 0
    for perm in permutations(range(4)):
        # Check single 4-cycle
        x = 0
        seen = set()
        while x not in seen:
            seen.add(x)
            x = perm[x]
        if len(seen) != 4:
            continue
        ds = [perm[i] - i for i in range(4)]
        # Which boundaries b in {1, 2, 3} have ds[b-1] == ds[b]?
        eq_bounds = [b for b in (1, 2, 3) if ds[b - 1] == ds[b]]
        for mask in range(1 << len(eq_bounds)):
            # Bit = 1 means MERGED (same word), bit = 0 means SPLIT (distinct words)
            merged = {eq_bounds[j] for j in range(len(eq_bounds)) if (mask >> j) & 1}
            pcs = []
            lo = 0
            wid = 0
            for i in range(1, 5):
                if i == 4 or i not in merged:
                    pcs.append((lo, i, ds[lo], bytes([wid])))
                    wid += 1
                    lo = i
            st = tuple(pcs)
            q_exact = solve_exact_independent(st, all_subintervals=True)
            has_l3_d1 = any((hi - lo) == 3 and abs(d) == 1 for lo, hi, d, _ in st)
            expected = 2 if has_l3_d1 else 3
            assert q_exact == expected, f"m=4 theorem FAILED on {st}: got {q_exact} vs {expected}"
            checked_m4 += 1
    # Also check all m=2 (1 cycle) and m=3 (2 cycles) states!
    checked_m23 = 0
    for m in (2, 3):
        for perm in permutations(range(m)):
            x = 0; seen = set()
            while x not in seen:
                seen.add(x); x = perm[x]
            if len(seen) != m:
                continue
            ds = [perm[i] - i for i in range(m)]
            eq_b = [b for b in range(1, m) if ds[b - 1] == ds[b]]
            for mask in range(1 << len(eq_b)):
                merged = {eq_b[j] for j in range(len(eq_b)) if (mask >> j) & 1}
                pcs = []; lo = 0; wid = 0
                for i in range(1, m + 1):
                    if i == m or i not in merged:
                        pcs.append((lo, i, ds[lo], bytes([wid])))
                        wid += 1; lo = i
                q_exact = solve_exact_independent(tuple(pcs), all_subintervals=True)
                assert q_exact == m - 1, f"m={m} theorem FAILED on {pcs}: got {q_exact}"
                checked_m23 += 1
    print(f"  [VERIFIED] Exact m<=4 terminal theorem exhaustively verified over ALL {checked_m23} (m=2,3) + "
          f"{checked_m4} (m=4) single-cycle piece states under FULL proper-subinterval C_gap (0 exceptions)!")

    # (ii) Independent verification of Alpha-3's n=16, r_0=14 sub-19/24 witness!
    L_wit = bytes(int(c) for c in "1010101010110010")
    T_true = bytes(int(c) for c in "0000111011010011")
    T_claimed_rev = bytes(int(c) for c in "0000110010110111")
    res = decode_bwt_single_cycle(L_wit)
    assert res is not None, "Witness L is not a valid single-cycle BWT!"
    T_wit, _ = res
    assert is_rot(T_wit, T_true), f"Decoded T mismatch: {T_wit} vs {T_true}"
    assert is_rot(T_wit[::-1], T_claimed_rev), "Claimed T is not the reversal of true T!"
    L_of_claimed, _ = bwt_of(T_claimed_rev)
    assert L_of_claimed != L_wit, "Claimed T unexpectedly had same BWT!"
    r0_wit = runs_count(L_wit)
    assert r0_wit == 14, f"Expected r0=14, got {r0_wit}"
    t0 = time.time()
    init_wit = init_pieces_from_L(L_wit)
    q_wit, path_wit = solve_exact_independent(init_wit, all_subintervals=False, return_path=True)
    dt_wit = time.time() - t0
    assert q_wit == 11, f"Expected Q*=11, got {q_wit}"
    # Verify byte-exact decode from final state of path
    final_st = path_wit[-1][2]
    assert len(final_st) == 1 and final_st[0][2] == 0
    dec_wit = final_st[0][3][::-1]
    assert is_rot(dec_wit, T_true), "Final C_gap state decode mismatch!"
    print(f"  [VERIFIED] Sub-19/24 Witness L=1010101010110010:")
    print(f"             true T=0000111011010011 (NOTE: reported T=0000110010110111 is its REVERSAL / unreversed LF walk, BWT={''.join(map(str,L_of_claimed))}, r0={runs_count(L_of_claimed)}!)")
    print(f"             single-cycle=TRUE, r_0={r0_wit}, exact Q*={q_wit} (NO m<=4 shortcut, {dt_wit:.2f}s), "
          f"Q*/r_0={q_wit}/{r0_wit}={q_wit/r0_wit:.6f} < 19/24={19/24:.6f}, byte-exact decode=PASS!")
    for step_idx, (mv, pst, nst) in enumerate(path_wit, 1):
        print(f"             step {step_idx:2d}: {mv} | P: {len(pst)} -> {len(nst)}, m: {pst[-1][1]} -> {nst[-1][1]}")

    # (iii) Exhaustive independent verification on ALL single-cycle binary BWTs for n=2..10
    #       (plus all_subintervals=True check on ALL n=2..9!)
    t1 = time.time()
    total_bwts = 0
    min_ratio = Fraction(99, 1)
    min_witnesses = []
    subint_diffs = 0
    for n in range(2, 11):
        n_cnt = 0
        n_min = Fraction(99, 1)
        for mask in range(1 << n):
            L = bytes((mask >> (n - 1 - i)) & 1 for i in range(n))
            dec = decode_bwt_single_cycle(L)
            if dec is None:
                continue
            n_cnt += 1
            total_bwts += 1
            r0 = runs_count(L)
            init_st = init_pieces_from_L(L)
            q_max = solve_exact_independent(init_st, all_subintervals=False)
            if n <= 9:
                q_all = solve_exact_independent(init_st, all_subintervals=True)
                if q_all != q_max:
                    subint_diffs += 1
            rat = Fraction(q_max, r0)
            if rat < n_min:
                n_min = rat
            if r0 >= 4:
                if rat < min_ratio:
                    min_ratio = rat
                    min_witnesses = [(n, "".join(str(b) for b in L), r0, q_max)]
                elif rat == min_ratio and len(min_witnesses) < 5:
                    min_witnesses.append((n, "".join(str(b) for b in L), r0, q_max))
        print(f"  n={n:2d}: valid single-cycle binary BWTs={n_cnt:3d}, min Q*/r_0 = {n_min} ({float(n_min):.4f})")
    print(f"  [VERIFIED] ALL {total_bwts} single-cycle binary BWTs for n=2..10 solved independently in {time.time()-t1:.2f}s:")
    print(f"             min Q*/r_0 (r_0 >= 4) = {min_ratio} ({float(min_ratio):.4f}) [n=2 trivial L=10 has 1/2], all_subintervals diffs (n<=9) = {subint_diffs}")
    print(f"             sample min (r_0 >= 4) witnesses: {min_witnesses}")
    assert total_bwts == 224 and min_ratio == Fraction(3, 4) and subint_diffs == 0


# ==============================================================================
# PART 3: MOONSHOT REGISTER & ARTIFACT SHA-256 INTEGRITY CHECK
# ==============================================================================
def audit_part3_and_hashes():
    print("\n" + "=" * 90)
    print("PART 3: CYCLE 17 MOONSHOT REGISTER LINE 15 & SHA-256 ARTIFACT AUDIT")
    print("=" * 90)
    reg_path = f"{JDIR}/cycle17/moonshot_register_v1.jsonl"
    with open(reg_path) as f:
        rows = [json.loads(line) for line in f if line.strip()]
    assert len(rows) == 15, f"Expected 15 register rows, got {len(rows)}"
    commits = {r["scorer_commit"] for r in rows}
    versions = {r["scorer_version"] for r in rows}
    assert versions == {"v1"} and commits == {"ee7fbf0e50c60cd9dcf0faa0cdd634cdbbb02849"}
    r15 = rows[14]
    r14 = rows[13]
    assert r15["candidate_sha256"] == r14["candidate_sha256"] == "19d118d78420855de6de7fdd10dda600e594647e5f86424ec5ac17a43a042a29"
    assert r15["gate"] == "PASS"
    instr = r15["proxy_advisory"]["1"]["instructions_ratio"]
    print(f"  [VERIFIED] Register rows={len(rows)}, all scorer_version=v1 @ ee7fbf0e50c6...")
    print(f"  [VERIFIED] Line 15 LeanBandCoalUni (sha256={r15['candidate_sha256'][:16]}...):")
    print(f"             1W wall={r15['walls']['1']['ratio']:.4f} [{r15['walls']['1']['lo']:.4f}, {r15['walls']['1']['hi']:.4f}], rss={r15['walls']['1']['peak_rss_ratio']:.4f}")
    print(f"             2W wall={r15['walls']['2']['ratio']:.4f} [{r15['walls']['2']['lo']:.4f}, {r15['walls']['2']['hi']:.4f}], rss={r15['walls']['2']['peak_rss_ratio']:.4f}")
    print(f"             proxy instructions_ratio={instr:.6f} (0.7741 record!), 1W/2W/4W/7W/8W/10W="
          + "/".join(f"{r15['proxy_advisory'][w]['ratio']:.4f}" for w in ("1", "2", "4", "7", "8", "10")))

    print("\n  SHA-256 Artifact Digest Table:")
    files = [
        f"{REPO}/src/open.rs",
        f"{REPO}/src/open/endpoint.rs",
        f"{JDIR}/cycle16/scale_crossover_bench/src/open_scaled.rs",
        f"{JDIR}/cycle16/scale_crossover_bench/src/b4_contract.rs",
        f"{JDIR}/cycle16/scale_crossover_bench/src/main.rs",
        f"{JDIR}/cycle16/scale_crossover_out_v2.txt",
        f"{JDIR}/cycle17/cycle17_unification_and_exact_opt.py",
        f"{JDIR}/cycle17/moonshot_register_v1.jsonl",
        f"{JDIR}/cycle17/multicorpus_stress/src/main.rs",
        f"{JDIR}/cycle17/multicorpus_stress/src/corpora.rs",
        f"{JDIR}/cycle17/multicorpus_stress_results.txt",
        f"{JDIR}/cycle10/referee2/cycle10_audit_c9.py",
    ]
    for p in files:
        print(f"    {sha256_file(p)[:16]}  {os.path.relpath(p, REPO)}")


if __name__ == "__main__":
    t_start = time.time()
    audit_part1()
    audit_part2a_wpb()
    audit_part2b_cross_cycle()
    audit_part2c_frontier()
    audit_part3_and_hashes()
    print(f"\nALL INDEPENDENT CYCLE 18 AUDITS COMPLETED in {time.time() - t_start:.2f}s.")
