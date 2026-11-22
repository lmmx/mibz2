#!/tmp/solver-venv/bin/python
"""
FCP Cycle 1 — Part 3: Adversarial Kill-Tests on:
  (A) All LP-synthesized candidate potentials from Part 1B (multi-length self-runs, true Gamma Fib_{2m}+$,
      large random BWTs) -> re-solve LP & extract exact Z3-verified Farkas certificates!
  (B) Our own 2-Sweep Topological Ascent/Descent-DAG Contraction (worst-case piece blowup family,
      Fib_{2m}+$, Symmetric-Middle-Run, and full 6-block Wikidata audit vs Open m_16 & BDMR).
"""
import json
import os
import random
import sys
import time

from ortools.linear_solver import pywraplp
import z3

sys.path.insert(0, "/tmp/fcp-ref")
import p1_potential_lp as p1
import p1b_deep_potential_farkas as p1b
import p2b_dag_dp_wikidata as p2b


def test_part_A_adversarial_potentials():
    print("=========================================================================")
    print("PART A: ADVERSARIAL KILL-TEST OF CANDIDATE POTENTIALS & EXACT Z3 FARKAS")
    print("=========================================================================")
    # Candidate 1: Phi_self = r + e_2_self
    # Adversarial family: T_K = 0^1 1 0^2 1 0^3 1 ... 0^K 1 2  (ONLY ONE self-loop symbol 0, K distinct run lengths!)
    print("\n--- A1. Killing Phi_self = r + e_2_self on T_K = (0^1 1)(0^2 1)...(0^K 1) 2 ---")
    adv_words = []
    for K in [3, 4, 5, 6, 8, 10]:
        T = []
        for j in range(1, K + 1):
            T.extend([0] * j + [1])
        T.append(2)
        T = tuple(T)
        L, F, LF, C, p, s = p1.bwt_of_word(T)
        f0 = p1.compute_features(L, F, LF, C, p, s, include_text_feats=True)
        res = p1.apply_bdmr_self(L, F, LF, C, p, s)
        L1, p1n, dc = res
        f1 = p1.compute_features(*p1.bwt_from_L_only(L1, p1n), include_text_feats=True)
        print(f"  K={K:2d} (n={len(T):2d}): r: {f0['r']:2d} -> {f1['r']:2d} (delta_r={f1['r']-f0['r']:+d}), "
              f"e_2_self: {f0['e_2_self']}->{f1['e_2_self']}, "
              f"r_F: {f0['r_F']}->{f1['r_F']}, E_RG: {f0['E_RG']}->{f1['E_RG']}, "
              f"ref_L2: {f0['ref_L2']}->{f1['ref_L2']}, e_3: {f0['e_3']}->{f1['e_3']}, "
              f"sigma: {f0['sigma']}->{f1['sigma']}")
        adv_words.append((f"self_blow_{K}", T))

    # Also test Variant B (with $=max vs $=min) on Fib_{2m}+$!
    # Wait: in Gamma's refute_bdmr_fib.py, sentinel $ = 0 < a=1 < b=2, and Variant B does threshold cuts with NO self-run first,
    # OR let's check what Fib_{2m}+$ does when we test EVERY threshold cut AND Gamma's exact Variant B!
    print("\n--- A2. Testing Fib_{2m}+$ (m=4..12) under Gamma Variant B (threshold-preferring) ---")
    fa, fb = (1,), (1, 2)
    fibs = []
    for m in range(3, 15):
        fa, fb = fb, fb + fa
        if m in (6, 8, 10, 12, 14):
            fibs.append((f"Fib_{m}_min$", fb + (0,)))
            fibs.append((f"Fib_{m}_max$", fb + (3,)))
    for name, T in fibs[:6]:
        L, F, LF, C, p, s = p1.bwt_of_word(T)
        r0 = 1 + sum(1 for i in range(1, len(L)) if L[i] != L[i - 1])
        # Run Beta's deep_analysis Variant B style: check max r along thresh_pref vs pure_maxcut
        traj_t, st_t = p1b.run_bdmr_trajectory(L, p, policy="thresh_pref")
        traj_m, st_m = p1b.run_bdmr_trajectory(L, p, policy="pure_maxcut")
        print(f"  {name:12s} n={len(T):4d} r0={r0}: thresh_pref max_r={max(f['r'] for f in st_t):2d} (r_seq={[f['r'] for f in st_t[:10]]}), "
              f"pure_maxcut max_r={max(f['r'] for f in st_m):2d}")

    # Test ALL Part 1B synthesized candidate potentials on the adversarial self-run family + larger random words!
    # Also construct pair-contraction blowup family:
    # Take T_pair(K): many distinct bigrams (0, b_j) where 0 is split into K sub-symbols!
    # e.g. alphabet 0..2K, in_L = even, in_R = odd: 0 followed by 2, 4, 6, ... separated by 1, 3, 5, ... -> 0 splits into K+1 sub-symbols!
    print("\n--- A3. Pair-Contraction Bucket-Split Blowup Family (0 splits into K blocks) ---")
    pair_adv = []
    for K in [3, 4, 5, 6]:
        # Letters 0..2K: in_L = {0, 2, 4, ..., 2K}, in_R = {1, 3, ..., 2K-1}
        # Word contains bigrams (0, 0), (0, 1), (0, 2), (0, 3), ..., (0, 2K) -> 0 has K+1 uncontracted blocks!
        # Put two copies of each (0, 2j) preceded by different letters or arranged so L=0 is a SINGLE run in BWT?
        # Even simpler: T = 0 0 1 0 2 0 3 0 4 ... 0 (2K-1) 0 (2K)
        T = [0]
        for x in range(1, 2 * K + 1):
            T.extend([0, x])
        T = tuple(T)
        L, F, LF, C, p, s = p1.bwt_of_word(T)
        in_L = [(c % 2 == 0) for c in range(s)]
        f0 = p1.compute_features(L, F, LF, C, p, s, include_text_feats=True)
        res = p1.apply_bdmr_pair(L, F, LF, C, p, s, in_L)
        L1, p1n, dc = res
        f1 = p1.compute_features(*p1.bwt_from_L_only(L1, p1n), include_text_feats=True)
        print(f"  Split K={K} (n={len(T):2d}, s={s:2d}): r: {f0['r']:2d} -> {f1['r']:2d}, sigma: {f0['sigma']}->{f1['sigma']}, "
              f"r_F: {f0['r_F']}->{f1['r_F']}, E_RG: {f0['E_RG']}->{f1['E_RG']}, ref_L2: {f0['ref_L2']}->{f1['ref_L2']}, del={dc}/{len(T)}")
        pair_adv.append((f"split_{K}", T, in_L))

    # A4. Collect enlarged transition dataset (small + adv_words + pair_adv + random n=15..60 + Fib) and re-solve LP!
    print("\n--- A4. Re-solving Homogeneous LP + Exact Z3 Farkas on Enlarged Adversarial Dataset ---")
    B_core = ["r", "r_circ", "sigma", "r_F", "E_RG", "E_RG_inv", "e_2", "e_2_self", "r_1", "r_F1"]
    B_rich = B_core + ["ref_DCF", "ref_L2", "r_L2", "r_L2L", "r_L2LF", "r_FF2", "r_LFF2", "e_3", "r_des", "rg_E_uniq", "rg_self", "frg_E_uniq", "max_sym_runs", "s_multi"]
    B_global = B_rich + ["r_rev", "delta"]

    tr_all, tr_pair, tr_self, tr_thresh, tr_maxcut, tr_maxcut_pair = [], [], [], [], [], []
    states_map = {}

    def reg(f):
        key = tuple(f[k] for k in B_rich) + (f["n"],)
        states_map[key] = f

    # Include small Lyndon n=3..7
    small_w = []
    for n in range(3, 8):
        for k in ([2, 3] if n <= 6 else [2]):
            for w in p1.gen_lyndon(n, k):
                if len(set(w)) >= 2:
                    small_w.append(("lyn", w))
    # Adversarial multi-length self-run words (also with 2 distinct self-loop symbols, and repeating pattern 0^j 1^j)
    for K in range(2, 11):
        T = []
        for j in range(1, K + 1):
            T.extend([0] * j + [1])
        T.append(2)
        small_w.append((f"selfA_{K}", tuple(T)))
        # Variant with 0^j 1 0^j 2 -> even more runs created!
        T2 = []
        for j in range(1, K + 1):
            T2.extend([0] * j + [1] + [0] * j + [2])
        T2.append(3)
        if K <= 7:
            small_w.append((f"selfB_{K}", tuple(T2)))
        # Variant with runs of 0 and 1
        T3 = []
        for j in range(1, K + 1):
            T3.extend([0] * j + [1] * j)
        T3.append(2)
        if K <= 7:
            small_w.append((f"selfC_{K}", tuple(T3)))
    # Random words n=14..50, s=2..6 (plus binary/ternary high-run-length words!)
    rng = random.Random(999)
    for n in [14, 20, 30, 45]:
        for s in [2, 3, 4, 5]:
            for idx in range(5):
                w = tuple(rng.randrange(s) for _ in range(n))
                if len(set(w)) >= 2 and p1.is_primitive(w):
                    small_w.append((f"r_{n}_{s}_{idx}", w))
    # Random RLE words (long runs of varying lengths!)
    for idx in range(25):
        s = rng.choice([2, 3, 4])
        T = []
        for _ in range(rng.randint(6, 16)):
            c = rng.randrange(s)
            T.extend([c] * rng.randint(1, 8))
        T = tuple(T)
        if len(set(T)) >= 2 and p1.is_primitive(T):
            small_w.append((f"rle_{idx}", T))

    # Add Fib trajectories too
    for name, w in fibs:
        small_w.append((name, w))

    for name, w in small_w:
        L, F, LF, C, p, s = p1.bwt_of_word(w)
        f0 = p1.compute_features(L, F, LF, C, p, s, include_text_feats=True)
        reg(f0)
        # Self-run
        res_s = p1.apply_bdmr_self(L, F, LF, C, p, s)
        if res_s and len(res_s[0]) >= 2 and len(set(res_s[0])) >= 2:
            f1 = p1.compute_features(*p1.bwt_from_L_only(res_s[0], 0), include_text_feats=True)
            reg(f1)
            meta = ("self", name, f"n={len(w)}", res_s[2], f0["r"], f1["r"])
            tr_all.append((f0, f1, meta))
            tr_self.append((f0, f1, meta))
            tr_thresh.append((f0, f1, meta))
            tr_maxcut.append((f0, f1, meta))
        # Pair steps
        if s <= 5:
            best_d = -1
            cands = []
            for mask in range(1, (1 << s) - 1):
                in_L = [bool((mask >> c) & 1) for c in range(s)]
                res_p = p1.apply_bdmr_pair(L, F, LF, C, p, s, in_L)
                if res_p and len(res_p[0]) >= 2 and len(set(res_p[0])) >= 2:
                    kappa, _, _ = p1.classify_coloring(in_L, F, L)
                    cands.append((in_L, res_p, kappa))
                    if res_p[2] > best_d:
                        best_d = res_p[2]
            for in_L, (L1, p1n, dc), kappa in cands:
                f1 = p1.compute_features(*p1.bwt_from_L_only(L1, 0), include_text_feats=True)
                reg(f1)
                meta = ("pair", name, in_L, dc, f0["r"], f1["r"])
                tr_all.append((f0, f1, meta))
                tr_pair.append((f0, f1, meta))
                if kappa == 1:
                    tr_thresh.append((f0, f1, meta))
                if dc == best_d and best_d > 0:
                    tr_maxcut.append((f0, f1, meta))
                    tr_maxcut_pair.append((f0, f1, meta))

    # Add explicit split_K transitions
    for name, T, in_L in pair_adv:
        L, F, LF, C, p, s = p1.bwt_of_word(T)
        f0 = p1.compute_features(L, F, LF, C, p, s, include_text_feats=True)
        reg(f0)
        res_p = p1.apply_bdmr_pair(L, F, LF, C, p, s, in_L)
        f1 = p1.compute_features(*p1.bwt_from_L_only(res_p[0], 0), include_text_feats=True)
        reg(f1)
        meta = ("pair", name, "even/odd", res_p[2], f0["r"], f1["r"])
        tr_all.append((f0, f1, meta))
        tr_pair.append((f0, f1, meta))

    # Add Gamma's EXACT Variant B trajectory on Fib_{2m}+$ ($=2 > 1 > 0, closed-form L)!
    # Exact Fib_{2m} with 2m=14 (n=378) and 2m=16 (n=988): Variant B triggers r0=4 -> r_max=9..11!
    fib_nums = [0, 1]
    for _ in range(25):
        fib_nums.append(fib_nums[-1] + fib_nums[-2])
    for two_m in [10, 12, 14, 16]:
        # L(Fib_{2m}+$) = [(1, F_{2m-2}-1), (2, 1), (0, F_{2m-1}), (1, 1)]
        L_fib = [1] * (fib_nums[two_m - 2] - 1) + [2] + [0] * fib_nums[two_m - 1] + [1]
        # Run Gamma's exact Variant B (0.05*n self, 0.9*mc thresh) step by step!
        Lcur = L_fib
        f_cur = p1.compute_features(*p1.bwt_from_L_only(Lcur, 0), include_text_feats=True)
        reg(f_cur)
        r_seq = [f_cur["r"]]
        for rnd in range(30):
            L, F, LF, C, p, s = p1.bwt_from_L_only(Lcur, 0)
            n = len(L)
            if n <= 2 or s <= 1:
                break
            self_tot = sum(1 for l, f in zip(L, F) if l == f)
            if self_tot == n:
                break
            if self_tot >= 0.05 * n:
                res = p1.apply_bdmr_self(L, F, LF, C, p, s)
                kind = "self"
            else:
                # Greedy max-cut vs best threshold (>= 0.9*mc)
                W = [[0] * s for _ in range(s)]
                for l, f in zip(L, F):
                    if l != f:
                        W[l][f] += 1
                in_L_mc = [False] * s
                for c in range(s):
                    gL = sum(W[c][b] for b in range(c) if not in_L_mc[b])
                    gR = sum(W[a][c] for a in range(c) if in_L_mc[a])
                    in_L_mc[c] = gL >= gR
                mc_cnt = sum(W[a][b] for a in range(s) for b in range(s) if in_L_mc[a] and not in_L_mc[b])
                best_th = (0, None)
                for cut in range(1, s):
                    for in_L_t in ([c < cut for c in range(s)], [c >= cut for c in range(s)]):
                        dc = sum(W[a][b] for a in range(s) for b in range(s) if in_L_t[a] and not in_L_t[b])
                        if dc > best_th[0]:
                            best_th = (dc, in_L_t)
                if best_th[0] >= 0.9 * mc_cnt and best_th[0] > 0:
                    res = p1.apply_bdmr_pair(L, F, LF, C, p, s, best_th[1])
                    kind = "varB_thresh"
                elif mc_cnt > 0:
                    res = p1.apply_bdmr_pair(L, F, LF, C, p, s, in_L_mc)
                    kind = "varB_mc"
                else:
                    res = p1.apply_bdmr_self(L, F, LF, C, p, s)
                    kind = "self"
            if res is None or len(res[0]) <= 1 or len(set(res[0])) <= 1:
                break
            Lcur = res[0]
            f_nxt = p1.compute_features(*p1.bwt_from_L_only(Lcur, 0), include_text_feats=True)
            reg(f_nxt)
            r_seq.append(f_nxt["r"])
            m2 = (kind, f"GammaFib_{two_m}", f"rnd{rnd}", res[2], f_cur["r"], f_nxt["r"])
            tr_all.append((f_cur, f_nxt, m2))
            if kind == "self":
                tr_self.append((f_cur, f_nxt, m2))
                tr_thresh.append((f_cur, f_nxt, m2))
            elif kind == "varB_thresh":
                tr_pair.append((f_cur, f_nxt, m2))
                tr_thresh.append((f_cur, f_nxt, m2))
            else:
                tr_pair.append((f_cur, f_nxt, m2))
            f_cur = f_nxt
        print(f"  GammaFib_{two_m} (n={len(L_fib)}): Variant B r_seq={r_seq} (max_r={max(r_seq)})")

    states = list(states_map.values())
    print(f"  Enlarged dataset: {len(states)} states | all={len(tr_all)}, pair={len(tr_pair)}, self={len(tr_self)}, thresh={len(tr_thresh)}, maxcut={len(tr_maxcut)}")

    farkas_out = {}
    for bname, basis in [("CORE_10", B_core), ("RICH_24", B_rich), ("GLOBAL_26", B_global)]:
        print(f"\n  [{bname}] Homogeneous LP (const=0, Phi >= r):")
        farkas_out[bname] = {}
        for cname, trs in [("all_bdmr", tr_all), ("pair_all", tr_pair), ("self_only", tr_self),
                           ("thresh_and_self", tr_thresh), ("maxcut_and_self", tr_maxcut), ("maxcut_pair_only", tr_maxcut_pair)]:
            st, Mval, wts = p1b.solve_lp_clean(trs, basis, states, allow_const=False, l1_reg=True)
            print(f"    {cname:18s} -> {st:10s}" + (f"  M={Mval:.4f} wts={wts}" if st == "FEASIBLE" else ""))
            rec = {"status": st, "M": Mval, "weights": wts}
            if st == "INFEASIBLE":
                # Also check with const allowed!
                st_c, M_c, _ = p1b.solve_lp_clean(trs, basis, states, allow_const=True, l1_reg=False)
                print(f"      (with +const allowed & Phi<=M*r: {st_c}" + (f" M={M_c:.4f})" if st_c == "FEASIBLE" else ")"))
                rec["with_const"] = st_c
                if bname in ("CORE_10", "RICH_24"):
                    cert = p1b.exact_farkas_z3(trs, basis, states, allow_const=False)
                    if cert:
                        print(f"      [EXACT Z3 FARKAS] verified={cert['exact_z3_verified']} | {cert['num_y']} transitions, {cert['num_u']} states:")
                        for coeff, meta in cert["y"][:6]:
                            print(f"        y={coeff}  {meta}")
                        rec["farkas"] = cert
            farkas_out[bname][cname] = rec

    with open("/tmp/fcp-ref/p3_farkas.json", "w") as fp:
        json.dump(farkas_out, fp, indent=2, default=str)


def test_part_B_adversarial_dag_dp():
    print("\n=========================================================================")
    print("PART B: ADVERSARIAL AUDIT OF 2-SWEEP TOPOLOGICAL ASCENT/DESCENT-DAG DP")
    print("=========================================================================")
    # 1. Test on Fib_{2m}+$ up to m=16 (n=1598)!
    print("\n--- B1. Multi-Round Alternating Ascent/Descent-DAG on Fib_{2m}+$ ---")
    fa, fb = (1,), (1, 2)
    for m in range(3, 19):
        fa, fb = fb, fb + fa
        if m in (8, 12, 16, 18):
            T = fb + (0,)
            L, F, LF, C, p, s = p1.bwt_of_word(T)
            runs = []
            for c in L:
                if runs and runs[-1][0] == c:
                    runs[-1] = (c, runs[-1][1] + 1)
                else:
                    runs.append((c, 1))
            r0 = len(runs)
            cur_runs, cur_n, cur_s = runs, len(L), s
            max_r = r0
            tot_r = 0
            tot_pieces = 0
            rnd = 0
            while cur_n > 2 and rnd < 40:
                # Choose whichever of asc or desc deletes more!
                # First check if self-runs exist and dominate
                # For clean test, check asc vs desc directly
                _, ma, _, sta = p2b.run_space_ascent_dag_step(cur_runs, cur_n, cur_s, mode="asc")
                _, md, _, std = p2b.run_space_ascent_dag_step(cur_runs, cur_n, cur_s, mode="desc")
                if ma <= md and ma < cur_n:
                    cur_runs, cur_n, cur_s, st = p2b.run_space_ascent_dag_step(cur_runs, cur_n, cur_s, mode="asc")
                elif md < cur_n:
                    cur_runs, cur_n, cur_s, st = p2b.run_space_ascent_dag_step(cur_runs, cur_n, cur_s, mode="desc")
                else:
                    break
                max_r = max(max_r, st["r_S"])
                tot_r += st["r0"]
                tot_pieces += st["fwd_pieces"] + st["bwd_pieces"] + st["emit_pieces"]
                rnd += 1
                if st["del_pct"] < 1.0:
                    # Do a self-run step if stalled on self-runs
                    Ltmp = []
                    for c, w in cur_runs:
                        Ltmp.extend([c] * w)
                    Lt, Ft, LFt, Ct, pt, st_s = p1.bwt_from_L_only(Ltmp, 0)
                    res_s = p1.apply_bdmr_self(Lt, Ft, LFt, Ct, pt, st_s)
                    if res_s is None:
                        break
                    L1, _, _ = res_s
                    if len(L1) <= 1 or len(set(L1)) <= 1:
                        break
                    cur_runs = []
                    for c in L1:
                        if cur_runs and cur_runs[-1][0] == c:
                            cur_runs[-1] = (c, cur_runs[-1][1] + 1)
                        else:
                            cur_runs.append((c, 1))
                    cur_n = len(L1)
                    cur_s = max(L1) + 1
                    max_r = max(max_r, len(cur_runs))
            print(f"  Fib_{m:2d}+$ (n={len(T):4d}, r0={r0}): rnds={rnd:2d}, final_n={cur_n}, max_r={max_r}, sum_r/r0={tot_r/r0:.2f}, tot_pieces/r0={tot_pieces/r0:.2f}")

    # 2. ADVERSARIAL PIECE-BLOWUP ATTACK on 2-Sweep Topological DAG-DP!
    # Question: Can ONE round of Ascent-DAG (L < F) have fwd_pieces / r0 or bwd_pieces / r0 = Omega(sigma) or Omega(n/r0)?!
    # THINK! Suppose T has ascending chains 0 1 2 ... (s-1) with MANY slightly different subsets of {0..s-1}
    # appearing so that P[b] has MANY segments, AND a SINGLE L-run of a < b spans ALL of F_b!!
    # TRY: T = concatenation of ALL (or many) ascending subsequences separated by 0, e.g.:
    #   For s = 8, 12, 16: take a word with small r_0 (or moderate r_0) where each bucket F_b is partitioned
    #   into MANY u_ext words, and a single run of L=0 covers F_1, single run of L=1 covers F_2, etc.!
    #   EVEN BETTER: Binary counter / De Bruijn / ascending tree!
    #   In T: take 2^k blocks, each block is: 0 followed by a distinct increasing subsequence of (1, 2, ..., 2k) of length k!
    #   Wait: EVERY block starts with 0, so L has ~n/k runs? No, r0 can still be small if we repeat?
    #   WAIT A SECOND! Can a SINGLE L-run of width w slice across M segments of P[b] where M >> r_0?!
    #   TEST THIS! Suppose a single run (0, w) lies in F_1, and P[1] has M segments.
    #   How did P[1] get M segments?! Each segment boundary in P[1] had to come from SOME run boundary in F_2..F_{s-1},
    #   OR from pulling back P[2] through MULTIPLE runs of L=1 in F_2!
    #   AHA!! MULTIPLICATIVE CASCADE!!
    #   Suppose:
    #     - L=2 has 2 runs in F_3 (splitting P[2] into 2x as many segments as P[3]!)
    #     - L=1 has 2 runs in F_2 (splitting P[1] into 2x as many segments as P[2] -> 4x!)
    #     - L=0 has 2 runs in F_1 (splitting P[0] into 2x as many segments as P[1] -> 8x!)
    #   WAIT A SECOND!! CAN WE CONSTRUCT A VALID BWT WHERE |P[0]| = 2^s WHILE r_0 = O(s)?!
    #   CHECK!! For a word T of length n:
    #   Every segment of P[a] corresponds to a DISTINCT ascending string u_ext starting at a!
    #   To have 2^k distinct ascending strings of length k starting at 0:
    #   Let alphabet be {0, 1, 2, ..., 2k} and T = concatenation of ALL 2^k words of the form:
    #     0, (1 or 2), (3 or 4), (5 or 6), ..., (2k-1 or 2k)  (wait: each has length k+1, ALL are strictly increasing!)
    #   How many runs does L of THAT word have? Let's measure r_0 vs |P| vs fwd_pieces vs r_S!
    print("\n--- B2. Adversarial Exponential-Cascade Attack on 2-Sweep DAG-DP ---")
    for k in [3, 4, 5, 6, 7]:
        # Word of length (k+1) * 2^k: all 2^k choices of (0, b_1, b_2, ..., b_k) where b_j in {2j-1, 2j}
        # Arrange in Gray-code or lexicographic order so r_0 is as SMALL as possible!
        T = []
        for mask in range(1 << k):
            T.append(0)
            for j in range(k):
                T.append(2 * j + 1 + ((mask >> j) & 1))
        T = tuple(T)
        L, F, LF, C, p, s = p1.bwt_of_word(T)
        runs = []
        for c in L:
            if runs and runs[-1][0] == c:
                runs[-1] = (c, runs[-1][1] + 1)
            else:
                runs.append((c, 1))
        _, m, new_s, st = p2b.run_space_ascent_dag_step(runs, len(L), s, mode="asc")
        print(f"  Cascade k={k} (n={len(T):4d}, s={s:2d}): r0={st['r0']:4d} -> r_S={st['r_S']:4d} ({st['r_ratio']:.2f}x!), "
              f"|P|={st['P_segs']:4d} ({st['P_segs']/st['r0']:.2f} r0), "
              f"fwd={st['fwd_pieces']:4d}, bwd={st['bwd_pieces']:4d}, emit={st['emit_pieces']:4d}, "
              f"tot_pieces/r0={st['total_pieces_per_r0']:.2f}x, del={st['del_pct']:.1f}%")

    # Also test Symmetric-Middle-Run (K=64, n=4224, r=130)!
    print("\n--- B3. Symmetric-Middle-Run Attack (which killed Lazard-BWT) ---")
    # T = prod_{i=0}^{63} (0, 1^64, 2) ? Wait: let's build Symmetric-Middle-Run with unique IDs or 0 1^K 2!
    # Wait: in Round 3, Symmetric-Middle-Run was prod_{j=0}^{M-1} (0, 1^K, 2, id(j)) -> primitive!
    K = 32
    T_smr = []
    for j in range(16):
        # 0 < 1 < 2 < 3 < 4: middle letter 2 repeated K times!
        bits = [(j >> b) & 1 for b in range(4)]
        T_smr.extend([0] + [2] * K + [4] + [1 if b == 0 else 3 for b in bits])
    T_smr = tuple(T_smr)
    L, F, LF, C, p, s = p1.bwt_of_word(T_smr)
    runs = []
    for c in L:
        if runs and runs[-1][0] == c:
            runs[-1] = (c, runs[-1][1] + 1)
        else:
            runs.append((c, 1))
    _, m, new_s, st = p2b.run_space_ascent_dag_step(runs, len(L), s, mode="asc")
    print(f"  SymMidRun (n={len(T_smr)}, r0={st['r0']}): 1-shot Asc -> m={m} ({st['del_pct']:.1f}% del), r_S={st['r_S']} ({st['r_ratio']:.2f}x), pieces/r0={st['total_pieces_per_r0']:.2f}")

    # 3. Full 6-block Wikidata comparison to T <= 4096 (using max(asc, desc) + self-run)!
    print("\n--- B4. Full 6-Block Wikidata Multi-Round DAG-DP (to T <= 4096) ---")
    block_files = sorted(f for f in os.listdir("/tmp/mibz2-blocks") if f.endswith(".L"))
    for fname in block_files:
        data, n, runs = p2b.load_block(os.path.join("/tmp/mibz2-blocks", fname))
        r0 = len(runs)
        cur_runs, cur_n, cur_s = runs, n, 256
        tot_r = 0
        tot_pieces = 0
        tot_fwd = 0
        tot_bwd = 0
        tot_emit = 0
        rnd = 0
        while cur_n > 4096 and rnd < 15:
            mode = "asc" if (rnd % 2 == 0) else "desc"
            cur_runs, cur_n, cur_s, st = p2b.run_space_ascent_dag_step(cur_runs, cur_n, cur_s, mode=mode)
            tot_r += st["r0"]
            tot_fwd += st["fwd_pieces"]
            tot_bwd += st["bwd_pieces"]
            tot_emit += st["emit_pieces"]
            tot_pieces += st["fwd_pieces"] + st["bwd_pieces"] + st["emit_pieces"]
            rnd += 1
        print(f"  {fname:12s} | r0={r0:6d} | rnds={rnd} -> n={cur_n:4d}, r={len(cur_runs):4d} | "
              f"sum(r_t)/r0={tot_r/r0:.3f}x | pieces(F/B/E)/r0=({tot_fwd/r0:.2f}/{tot_bwd/r0:.2f}/{tot_emit/r0:.2f}) = {tot_pieces/r0:.2f} r0")


if __name__ == "__main__":
    t0 = time.time()
    test_part_A_adversarial_potentials()
    test_part_B_adversarial_dag_dp()
    print(f"\nDone in {time.time()-t0:.2f}s")
