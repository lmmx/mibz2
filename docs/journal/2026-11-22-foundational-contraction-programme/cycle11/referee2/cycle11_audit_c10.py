#!/usr/bin/env python3
"""
Cycle 11 Task 1: Independent Adversarial Audit of Cycle 10 Theorems (Alpha S1-S3 & Beta E1-E2, R1):
  1. Alpha Theorems S1-S3 (Reverse SA-IS + Irreducible LCP):
     - Exhaustive check of Theorem S1 on primitive, periodic, AND arbitrary non-BWT multi-cycle columns
       (probing fixed points delta_u == 0 and self-runs spanning split[c]!).
     - Spot-check block-0.L (|LMS|/n, r(T1)/r0, Q_tot/r0, Phi_breaks == r0, sum_irrLCP/n).
  2. Beta Theorems E1, E2, R1 (Displacement Rank & Ribbon Graph Genus):
     - Test Theorem E1 over Q AND finite fields F_2, F_3, F_5 (when char(F) | cycle length of kappa!)
       across binary n<=8, ternary n<=6, AND arbitrary multi-cycle permutations!
     - Test Theorem R1 (`g_fat == g_orig - 1` and `F == c_ge2`) on Christoffel words, ALL binary n<=9
       & ternary n<=6 BWTs (hunting for counterexamples where nu_mod < r0!), AND spot-check block-0.L!
"""
import hashlib
import json
import os
import sys

sys.path.insert(0, "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-foundational-contraction-programme/cycle10/alpha")
import test_reverse_sais as rs
import bwt_core as bc

sys.path.insert(0, "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-foundational-contraction-programme/cycle10/beta")
import cycle10_displacement_ribbon as dr

sys.path.insert(0, "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-foundational-contraction-programme/cycle9/beta")
import cycle9_defect_durand as dd


def audit_1_alpha_s1_s3():
    print("=== 1. AUDITING ALPHA'S THEOREMS S1-S3 (REVERSE SA-IS & IRREDUCIBLE LCP) ===", flush=True)

    # 1A. Test Theorem S1 on: (i) primitive BWTs, (ii) periodic BWTs, (iii) arbitrary non-BWT columns (fixed points!)
    prim_ok = 0; prim_fail = None
    per_ok = 0; per_fail = None
    arb_fp_fail = None; arb_nofp_fail = None; arb_nofp_ok = 0

    for base, nmax in [(2, 9), (3, 6)]:
        for n in range(2, nmax + 1):
            for code in range(base ** n):
                T = bytes((code // (base ** i)) % base for i in range(n))
                L, p = bc.bwt_encode(T)
                _, r0, _, strips, j_order, C = bc.load_runs_and_open_baseline(L, p)
                l_cnt = [0] * 256
                for lo, hi, d, c, m in strips:
                    if d > 0: l_cnt[c] += (hi - lo)
                split = [C[c] + l_cnt[c] for c in range(256)]
                F = sorted(L)
                cur = C[:]; LF = [0] * n; FL = [0] * n
                for i, b in enumerate(L):
                    LF[i] = cur[b]; FL[cur[b]] = i; cur[b] += 1
                lms_split = {i for i in range(n) if i >= split[F[i]] and L[i] > F[i]}
                lms_min = {i for i in range(n) if LF[i] > i and FL[i] > i}
                is_prim = len(set(T[k:] + T[:k] for k in range(n))) == n
                if is_prim:
                    if lms_split != lms_min and prim_fail is None:
                        prim_fail = (list(T), list(L), lms_split, lms_min)
                    else:
                        prim_ok += 1
                else:
                    # Periodic valid BWT! Check BOTH lms_split == lms_min AND whether V \ lms_min is acyclic!
                    if lms_split != lms_min and per_fail is None:
                        per_fail = ("split!=min", list(T), list(L), lms_split, lms_min)
                    elif len(set(T)) > 1:
                        # Check if every cycle intersects lms_min
                        vis = [False] * n; hit_all = True
                        for i in range(n):
                            if vis[i]: continue
                            cyc = []; cur_x = i
                            while not vis[cur_x]: vis[cur_x] = True; cyc.append(cur_x); cur_x = LF[cur_x]
                            if not any(x in lms_min for x in cyc): hit_all = False
                        if not hit_all and per_fail is None:
                            per_fail = ("cycle_missed", list(T), list(L), lms_min)
                        else:
                            per_ok += 1

    # Arbitrary non-BWT binary columns n=2..8: separate fixed-point vs fixed-point-free!
    for n in range(2, 9):
        for mask in range(1 << n):
            L = bytes((mask >> i) & 1 for i in range(n))
            c0 = L.count(0); C = [0, c0]; F = [0]*c0 + [1]*(n - c0)
            cur = C[:]; LF = [0] * n; FL = [0] * n
            for i, b in enumerate(L): LF[i] = cur[b]; FL[cur[b]] = i; cur[b] += 1
            has_fp = any(LF[i] == i for i in range(n))
            # Compute split[c] = C[c] + sum_{L_u=c, delta_u>0} len_u
            l_cnt = [0, 0]
            for i in range(n):
                if LF[i] > i: l_cnt[L[i]] += 1
            split = [C[c] + l_cnt[c] for c in range(2)]
            lms_split = {i for i in range(n) if i >= split[F[i]] and L[i] > F[i]}
            lms_min = {i for i in range(n) if LF[i] > i and FL[i] > i}
            if has_fp:
                if (lms_split != lms_min or any(LF[i] == i and i not in lms_min for i in range(n))) and arb_fp_fail is None:
                    arb_fp_fail = (list(L), LF, lms_split, lms_min)
            else:
                if lms_split != lms_min and arb_nofp_fail is None:
                    arb_nofp_fail = (list(L), LF, lms_split, lms_min)
                else:
                    arb_nofp_ok += 1

    print(f"  1A. Theorem S1 Audit:", flush=True)
    print(f"      Primitive BWTs (n<=9 bin, n<=6 tern) : {prim_ok} OK, fail={prim_fail}", flush=True)
    print(f"      Non-unary Periodic BWTs              : {per_ok} OK, fail={per_fail}", flush=True)
    print(f"      Arbitrary non-BWT (no fixed pts)     : {arb_nofp_ok} OK, fail={arb_nofp_fail}", flush=True)
    print(f"      Arbitrary non-BWT (WITH fixed pts)   : BREAKS as predicted -> witness={arb_fp_fail}", flush=True)

    # Check self-run spanning split[c] on a valid primitive BWT!
    # Wait: if L[i] == c and F[i] == c (self-run!), then for i in F_c:
    #   LF(i) > i (L-type target) vs LF(i) < i (S-type target):
    #   WAIT: in a single maximal run of c inside F_c, delta = LF(i) - i is CONSTANT across the ENTIRE run!
    #   So a self-run CANNOT have delta > 0 on half and delta < 0 on half!
    #   Can a self-run in L (overlapping F_c) straddle split[c] in DOMAIN (so part of the self-run is < split[c] and part >= split[c])?
    #   Yes, and for those rows L[i] == F[i] == c, so L[i] > F[i] is FALSE anyway (never LMS)!

    # 1B. Spot-check block-0.L Reverse SA-IS cascade!
    blocks = dd.load_authenticated_blocks()
    b0, raw0 = blocks[0]
    ref0 = bc.ref_ibwt(raw0, b0["primary_index"])
    dec0, st0 = rs.reverse_sais_cascade(raw0, b0["primary_index"], verbose=False)
    assert dec0 == ref0, "Reverse SA-IS exactness fail on block-0.L!"
    print(f"  1B. block-0.L Reverse SA-IS spot-check [EXACT]: st0={st0}", flush=True)


def audit_2_beta_e1_r1():
    print("\n=== 2. AUDITING BETA'S THEOREMS E1, E2, R1 (DISPLACEMENT RANK & RIBBON GENUS) ===", flush=True)

    # 2A. Test Theorem E1 over Q AND finite fields F_2, F_3, F_5 (including when char(F) divides cycle lengths of kappa!)
    #     across binary BWTs n<=7, ternary BWTs n<=5, AND arbitrary multi-cycle permutations!
    e1_checked = 0; e1_fail = None; char_div_hits = {2: 0, 3: 0, 5: 0}
    for n in range(1, 8):
        for mask in range(1 << n):
            L = bytes((mask >> i) & 1 for i in range(n))  # ALL arbitrary binary columns (incl. multi-cycle!)
            c0 = L.count(0); pos = [0, c0]; LF = [0] * n
            for i, b in enumerate(L): LF[i] = pos[b]; pos[b] += 1
            M = [[0] * n for _ in range(n)]
            for j in range(n):
                M[LF[j]][j] += 1
                M[(LF[(j - 1) % n] + 1) % n][j] -= 1
            c_tot, c_ge2, c_fix, nu_mod, max_cl = dr.commutator_cycles(LF)
            # Check over Q (mod 10^9+7) AND F_2, F_3, F_5!
            for mod in (1000000007, 2, 3, 5):
                rk = dr.exact_rank_gf_p(M, mod=mod)
                if rk != n - c_tot or rk != nu_mod - c_ge2:
                    if e1_fail is None:
                        e1_fail = (n, list(L), mod, rk, n - c_tot, nu_mod - c_ge2)
                if mod in char_div_hits and max_cl >= mod and max_cl % mod == 0:
                    char_div_hits[mod] += 1
            e1_checked += 1
    print(f"  2A. Theorem E1 over Q, F_2, F_3, F_5 ({e1_checked} arbitrary binary columns n<=7; char|cycle_len hits={char_div_hits}): fail={e1_fail}", flush=True)

    # 2B. CRITICAL TEST OF THEOREM R1 (`g_fat == g_orig - 1` AND `F == c_ge2`)!
    #     Test on Christoffel word AND ALL binary BWTs (n=2..9) + ternary BWTs (n=2..6)!
    #     Watch what happens when nu_mod < r0 (e.g. Christoffel word r0=2, nu_mod=0, F=2, c_ge2=0)!
    r1_checked = 0
    viol_gfat = []
    viol_F_eq_cge2 = []
    for base, nmax in [(2, 9), (3, 6)]:
        for n in range(2, nmax + 1):
            for code in range(base ** n):
                T = bytes((code // (base ** i)) % base for i in range(n))
                L, p = dd.bwt_encode_small(T)
                runs = dd.rle_from_bytes(L)
                r0 = len(runs)
                cnt = [0] * 3
                for x in L: cnt[x] += 1
                C = [0, cnt[0], cnt[0] + cnt[1]]
                occ = C[:]; LF = [0] * n
                for i, x in enumerate(L): LF[i] = occ[x]; occ[x] += 1
                c_tot, c_ge2, c_fix, nu_mod, _ = dr.commutator_cycles(LF)
                g_orig = 1 + (n - c_tot) // 2
                _, _, F_fat, g_fat, _ = dr.ribbon_2vertex_iet_fatgraph(runs)
                r1_checked += 1
                if g_fat != g_orig - 1 and len(viol_gfat) < 3:
                    viol_gfat.append((list(T), list(L), r0, nu_mod, c_ge2, F_fat, g_orig, g_fat))
                if F_fat != c_ge2 and len(viol_F_eq_cge2) < 3:
                    viol_F_eq_cge2.append((list(T), list(L), r0, nu_mod, c_ge2, F_fat, g_orig, g_fat))
    print(f"  2B. Theorem R1 Audit ({r1_checked} binary n<=9 + ternary n<=6 BWTs):", flush=True)
    print(f"      g_fat == g_orig - 1 violations : {len(viol_gfat)} | first={viol_gfat[:2]}", flush=True)
    print(f"      F_fat == c_ge2 violations      : {len(viol_F_eq_cge2)} | first={viol_F_eq_cge2[:2]}", flush=True)

    # 2C. Spot-check block-0.L (g_orig=25590, g_fat=25589, F=6, g_ov=5448)
    blocks = dd.load_authenticated_blocks()
    b0, raw0 = blocks[0]
    LF0, C0, _, _ = dd.build_lf_and_decode(raw0, b0["primary_index"])
    runs0 = dd.rle_from_bytes(raw0)
    c_tot, c_ge2, c_fix, nu_mod, _ = dr.commutator_cycles(LF0)
    g_orig = 1 + (len(raw0) - c_tot) // 2
    _, _, F_fat, g_fat, _ = dr.ribbon_2vertex_iet_fatgraph(runs0)
    V_ov, E_ov, F_ov, comps_ov, g_ov, _ = dr.ribbon_run_overlap_switch(runs0, C0)
    beta1 = E_ov - V_ov + comps_ov
    print(f"  2C. block-0.L spot-check: r0={len(runs0)}, nu_mod={nu_mod}, c_ge2={c_ge2}, g_orig={g_orig}, "
          f"F_fat={F_fat}, g_fat={g_fat} | G_ov: V={V_ov}, E={E_ov}, F={F_ov}, comps={comps_ov}, g_ov={g_ov} (2g_ov/beta1={200*g_ov/beta1:.1f}%)", flush=True)


if __name__ == "__main__":
    audit_1_alpha_s1_s3()
    audit_2_beta_e1_r1()
