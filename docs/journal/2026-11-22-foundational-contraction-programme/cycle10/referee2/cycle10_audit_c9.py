#!/usr/bin/env python3
"""
Cycle 10 Task 1: Independent Adversarial Audit of ALL THREE Cycle 9 Theoretical Packages:
  1. Beta's Theorems D0-D2b (cycle9/beta/cycle9_defect_durand.py):
     - Theorem D1 (r0 - |Sigma| <= nu_mod(LF) <= r0): exhaustive binary n<=10, ternary n<=7, unary, Christoffel!
     - Theorem D2a (Durand BWT-order preservation): CRITICAL SELF-OVERLAPPING FACTOR TEST (u="00", u="010" on n<=10!)
       + spot-check block-0.L (`"` -> 6,133)!
  2. Alpha's Theorems T0, T1, P1 & S2_dt_heavy = 1.587 r_0 (cycle9/alpha/test_train_track_folding.py):
     - Theorem T0 (ill_adj == dual_ill == q0 - |V|): exhaustive binary n<=8, ternary n<=6, unary, periodic, r0=1!
     - Independent verification of S2_dt_heavy on block-0.L & block-210.L + 5 adversarial killers
       (3-phase, AltRing, Low-r Padded-Ring, SymMidRun, Fib_20+$)!
  3. Gamma's Thick-Morse 0.0% Bands Invariance (cycle9/gamma/cycle9_thick_morse_audit.py):
     - Spot-check 51.9% Flush+Strict-Interior & exact Bands invariance on block-0.L!
"""
import hashlib
import itertools
import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-foundational-contraction-programme/cycle9/beta")
import cycle9_defect_durand as dd

sys.path.insert(0, "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-foundational-contraction-programme/cycle9/alpha")
import test_train_track_folding as ttf
import bwt_core as bc

sys.path.insert(0, "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-foundational-contraction-programme/cycle9/gamma")
import cycle9_thick_morse_audit as tma


def audit_1_beta_d0_d2():
    print("=== 1. AUDITING BETA'S THEOREMS D0-D2b ===", flush=True)

    # 1A. Theorem D1: r0 - |Sigma| <= nu_mod(LF) <= r0
    # Wait: HOW is nu_mod(LF) defined? Count i in 0..n-1 where (LF[(i+1)%n] - LF[i]) % n != 1 (cyclic)
    # OR non-cyclic i in 0..n-2? Let's test BOTH cyclic and linear definitions!
    viol_lin = None; viol_cyc = None
    min_lin_gap = 999; min_cyc_gap = 999
    checked = 0
    for base, nmax in [(2, 10), (3, 7)]:
        for n in range(1, nmax + 1):
            for code in range(base ** n):
                T = bytes((code // (base ** i)) % base for i in range(n))
                L, p = dd.bwt_encode_small(T)
                sig = len(set(L))
                r0 = len(dd.rle_from_bytes(L))
                # Build LF
                cnt = [0] * 3
                for x in L: cnt[x] += 1
                C = [0, cnt[0], cnt[0] + cnt[1]]
                occ = C[:]
                LF = [0] * n
                for i, x in enumerate(L): LF[i] = occ[x]; occ[x] += 1
                # Modular shift Delta(i) = (LF[i] - i) % n
                # Discontinuities: Delta(i) != Delta((i+1)%n) (cyclic) vs i=0..n-2 (linear)
                deltas = [(LF[i] - i) % n for i in range(n)]
                nu_cyc = sum(1 for i in range(n) if deltas[i] != deltas[(i + 1) % n])
                nu_lin = sum(1 for i in range(n - 1) if deltas[i] != deltas[i + 1])
                checked += 1
                if not (r0 - sig <= nu_cyc <= r0) and viol_cyc is None:
                    viol_cyc = (list(T), list(L), r0, sig, nu_cyc, deltas)
                if not (r0 - sig <= nu_lin <= r0) and viol_lin is None:
                    viol_lin = (list(T), list(L), r0, sig, nu_lin, deltas)
                if nu_cyc - (r0 - sig) < min_cyc_gap: min_cyc_gap = nu_cyc - (r0 - sig)
                if nu_lin - (r0 - sig) < min_lin_gap: min_lin_gap = nu_lin - (r0 - sig)
    print(f"  1A. Theorem D1 ({checked} binary n<=10 + ternary n<=7 BWTs):", flush=True)
    print(f"      Cyclic nu_mod : viol={viol_cyc}, min(nu_cyc - (r0-|Sigma|))={min_cyc_gap}", flush=True)
    print(f"      Linear nu_mod : viol={viol_lin}, min(nu_lin - (r0-|Sigma|))={min_lin_gap}", flush=True)

    # 1B. Theorem D2a (Durand BWT-order preservation): TEST SELF-OVERLAPPING FACTORS u="00" and u="010"!
    # Suppose u occurs at cyclic positions pos_0 < pos_1 < ... < pos_{k-1} in T (ALL occurrences, even overlapping!).
    # Wait: if occurrences overlap, what is the "derived sequence T^(1)"?
    #   - With overlapping occurrences (standard Durand return words: r_i = T[pos_i .. pos_{i+1}), where len(r_i) can be < len(u)!):
    #     Does the cyclic BWT order of T^(1) under augmented order w*u < w'*u STILL match the BWT order of [lo_u, hi_u) in BWT(T)?!
    #   - Check BOTH: (i) standard Durand return words (all occurrences, overlapping allowed!) under w*u < w'*u,
    #                 (ii) standard Durand return words under plain lex order w < w'!
    overlap_tests = 0
    viol_aug_overlap = None
    viol_plain_overlap = None
    viol_aug_nonoverlap = None
    for u_bytes in (b"\x00\x00", b"\x00\x01\x00"):
        ul = len(u_bytes)
        for n in range(ul + 2, 11):
            for mask in range(1 << n):
                T = bytes((mask >> i) & 1 for i in range(n))
                # Find all cyclic occurrences of u_bytes
                T2 = T + T
                occ = [i for i in range(n) if T2[i:i + ul] == u_bytes]
                if len(occ) < 2:
                    continue
                has_overlap = any(((occ[(j + 1) % len(occ)] - occ[j]) % n) < ul for j in range(len(occ)))
                # True BWT order of these occurrences in T (sorted by cyclic rotation T[i:] + T[:i])
                rot_order = sorted(occ, key=lambda i: T[i:] + T[:i])
                # Return words r_j = T[occ[j] .. occ[j+1]) cyclically
                k = len(occ)
                ret_words = []
                for j in range(k):
                    a = occ[j]
                    b = occ[(j + 1) % k]
                    ln = (b - a) % n
                    if ln == 0: ln = n
                    ret_words.append(T2[a:a + ln])
                # Derived sequence T^(1) = [ret_words[0], ..., ret_words[k-1]]
                # Augmented order on return words: w1 < w2 iff w1 + u < w2 + u (byte lex!)
                # Wait: to compare cyclic rotations of T^(1) under augmented order:
                #   Compare tuple (ret_words[(j+t)%k] + u_bytes for t in range(k))!
                aug_order = [occ[j] for j in sorted(range(k), key=lambda j: tuple(ret_words[(j + t) % k] + u_bytes for t in range(k)))]
                plain_order = [occ[j] for j in sorted(range(k), key=lambda j: tuple(ret_words[(j + t) % k] for t in range(k)))]
                # Check if distinct rotations (ignore periodic ties by comparing expanded strings)
                if len(set(T[i:] + T[:i] for i in occ)) < k:
                    continue
                overlap_tests += 1
                if has_overlap:
                    if aug_order != rot_order and viol_aug_overlap is None:
                        viol_aug_overlap = (list(u_bytes), list(T), occ, [list(w) for w in ret_words], rot_order, aug_order)
                    if plain_order != rot_order and viol_plain_overlap is None:
                        viol_plain_overlap = (list(u_bytes), list(T), occ, [list(w) for w in ret_words], rot_order, plain_order)
                else:
                    if aug_order != rot_order and viol_aug_nonoverlap is None:
                        viol_aug_nonoverlap = (list(u_bytes), list(T), occ, rot_words, rot_order, aug_order)
    print(f"  1B. Theorem D2a Self-Overlapping Factor Audit (u in {{00, 010}}, n<=10, {overlap_tests} tests):", flush=True)
    print(f"      Non-overlapping augmented (w*u) viol : {viol_aug_nonoverlap}", flush=True)
    print(f"      OVERLAPPING augmented (w*u) viol     : {viol_aug_overlap}", flush=True)
    print(f"      OVERLAPPING plain lex (w) viol       : {viol_plain_overlap}", flush=True)

    # 1C. Spot-check block-0.L Durand `"` -> 6,133
    blocks = dd.load_authenticated_blocks()
    b0, raw0 = blocks[0]
    LF0, C0, T0, orb0 = dd.build_lf_and_decode(raw0, b0["primary_index"])
    # Check exact single-letter return words for '"' (ord('"') = 34)
    q_ch = ord('"')
    occ_q = [t for t, ch in enumerate(T0) if ch == q_ch]
    T02 = T0 + T0
    # Return words & BWT(T^(1)) run count:
    k = len(occ_q)
    rws = [T02[occ_q[j]:occ_q[j] + ((occ_q[(j + 1) % k] - occ_q[j]) % len(T0))] for j in range(k)]
    # Sort indices 0..k-1 by original BWT row orb0[occ_q[j]] (exact F_'"' order!)
    idx_by_bwt = sorted(range(k), key=lambda j: orb0[occ_q[j]])
    # Preceding return word in BWT(T^(1)) is rws[(j - 1) % k]!
    bwt_t1 = [rws[(j - 1) % k] for j in idx_by_bwt]
    r_bwt_t1 = 1 + sum(1 for i in range(1, k) if bwt_t1[i] != bwt_t1[i - 1])
    print(f"  1C. block-0.L Durand F_'\"' spot-check: |F_'\"'|={k}, BWT(T^(1))_r={r_bwt_t1} (Beta claimed 6,133)", flush=True)


def audit_2_alpha_train_track():
    print("\n=== 2. AUDITING ALPHA'S THEOREMS T0, T1, P1 & S2_dt_heavy ===", flush=True)

    # 2A. Theorem T0: ill_adj == dual_ill == q0 - |V| on exhaustive binary n<=8 & ternary n<=6 (incl. unary, periodic, r0=1!)
    t0_checked = 0; t0_viol = None
    for base, nmax in [(2, 8), (3, 6)]:
        for n in range(1, nmax + 1):
            for code in range(base ** n):
                L = bytes((code // (base ** i)) % base for i in range(n))
                for p in ([0] if n >= 6 else range(n)):
                    st, *_ = ttf.build_measured_train_track(L, p)
                    q0, num_sw = st["q0"], st["num_sw"]
                    ia, da = st["illegal_turn_adj"], st["dual_illegal_adj"]
                    t0_checked += 1
                    if not (ia == da == q0 - num_sw) and t0_viol is None:
                        t0_viol = (list(L), p, q0, num_sw, ia, da)
    print(f"  2A. Theorem T0 ({t0_checked} arbitrary binary n<=8 + ternary n<=6, incl. unary/periodic/r0=1): viol={t0_viol}", flush=True)
    # Check what happens WITHOUT isolate_p (using r0 instead of q0):
    # Note: Alpha's code uses q0 (which splits the run containing p into up to 3 strips), so q0 != r0!
    # Wait: does the identity STILL hold for pure r0 (without isolate_p)? YES! Proof:
    #   At switch k with d_in incoming and d_out outgoing intervals tiling [a, b) with NO internal coincident
    #   breakpoints: total overlap edges = d_in + d_out - 1!
    #   Hence sum_out (h_out - 1) = (d_in + d_out - 1) - d_out = d_in - 1,
    #   and sum_in (h_in - 1) = (d_in + d_out - 1) - d_in = d_out - 1!
    #   Summing over all |V| switches: sum (d_in - 1) = |E| - |V| = sum (d_out - 1)!!
    #   That's a 3-line EXACT topological identity for ANY switch partition B_sync = D_0 cap C_0!

    # 2B. Verify S2_dt_heavy on block-0.L & block-210.L + ALL 5 adversarial killers!
    blocks = dd.load_authenticated_blocks()
    for blk, raw in [blocks[0], blocks[4]]:
        ref = bc.ref_ibwt(raw, blk["primary_index"])
        dec, st = ttf.fold_train_track(raw, blk["primary_index"], schedule="S2_dt_heavy")
        assert dec == ref, f"S2_dt_heavy EXACTNESS FAIL on {blk['file']}"
        r0 = st["r0"]
        print(
            f"  2B. {blk['file']:11s} S2_dt_heavy: ops={st['ops']:6d} ({st['ops']/r0:.3f} r0) | "
            f"r1={st['r1']} r2={st['r2']} r3={st['r3']} r3s={st['r3s']} sp={st['sp']} | "
            f"q_core={st['q_core']} slp={st['slp']} [EXACT]",
            flush=True,
        )

    # 5 Adversarial killers on S2_dt_heavy!
    def de_bruijn(k):
        a = [0] * (2 * k); seq = []
        def db(t, p):
            if t > k:
                if k % p == 0: seq.extend(a[1:p + 1])
            else:
                a[t] = a[t - p]; db(t + 1, p)
                for j in range(a[t - p] + 1, 2): a[t] = j; db(t + 1, t)
        db(1, 1); return seq
    db2 = de_bruijn(4)
    f0, f1 = [0], [0, 1]
    while len(f1) < 6765: f0, f1 = f1, f1 + f0
    fib20 = bytes(f1 + [2])

    killers = [
        ("3-phase K=128", bytes([0, 1]*128 + [2] + [0, 0, 1]*128 + [2] + [0, 1, 1]*128 + [2])),
        ("AltRing k=128", bytes([0, 1]*128 + [1, 2]*128 + [2, 0]*128)),
        ("Low-r Padded-Ring K=64", bytes([0 if x==0 else 1 for x in db2 for _ in range(64)] + [1 if x==0 else 2 for x in db2 for _ in range(64)] + [2 if x==0 else 0 for x in db2 for _ in range(64)])),
        ("SymMidRun K=128", bytes([1]*128 + [0] + [1]*128 + [2])),
        ("Fib_20+$", fib20),
    ]
    print("  2C. S2_dt_heavy on 5 Adversarial Constant-r Killers:", flush=True)
    for name, T in killers:
        # Fast BWT encode
        n = len(T)
        sa = list(range(n)); rk = list(T); k = 1
        while k < n:
            sa.sort(key=lambda i: (rk[i], rk[(i + k) % n]))
            tmp = [0] * n
            for j in range(1, n):
                a, b = sa[j - 1], sa[j]
                tmp[b] = tmp[a] + (1 if (rk[a], rk[(a + k) % n]) != (rk[b], rk[(b + k) % n]) else 0)
            rk = tmp
            if rk[sa[-1]] == n - 1: break
            k <<= 1
        L = bytes(T[(i - 1) % n] for i in sa)
        p = sa.index(0)
        dec, st = ttf.fold_train_track(L, p, schedule="S2_dt_heavy")
        assert dec == T, f"S2_dt_heavy FAIL on {name}"
        print(f"      [{name:22s}] n={n:5d} r0={st['r0']:2d} | ops={st['ops']:5d} ({st['ops']/st['r0']:6.2f} r0) | "
              f"r1={st['r1']} r2={st['r2']} r3={st['r3']} r3s={st['r3s']} sp={st['sp']} q_core={st['q_core']} [EXACT]", flush=True)


def audit_3_gamma_thick_morse():
    print("\n=== 3. AUDITING GAMMA'S THICK-MORSE 0.0% BANDS INVARIANCE ON block-0.L ===", flush=True)
    blocks = tma.load_blocks()
    blk0, data0 = blocks[0]
    strips, thin_by_byte, m_ret = tma.extract_thick_and_thin(data0, blk0["primary_index"], limit=16)
    c = tma.census_thick_relationships(strips)
    ns = c["ns"]
    pct_r1 = 100.0 * c["r1_exact"] / ns
    pct_fl_int = 100.0 * (c["flush_incl"] + c["strict_int"]) / ns
    print(f"  block-0.L (W=16): ns={ns}, r1_exact={c['r1_exact']} ({pct_r1:.1f}%), "
          f"flush={c['flush_incl']}, strict_int={c['strict_int']} (flush+strict={pct_fl_int:.1f}%), "
          f"m2={c['m2_full_tile']}, m3={c['m3_full_tile']}", flush=True)


if __name__ == "__main__":
    audit_1_beta_d0_d2()
    audit_2_alpha_train_track()
    audit_3_gamma_thick_morse()
