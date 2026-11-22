#!/tmp/solver-venv/bin/python
"""
Quick 5-second final checks:
  1. Can self-run contraction violate r_next <= r + e_2_self (when 0^1..0^K appear before BOTH 1 and 2)?
  2. Does CORE_10 remain INFEASIBLE even when pair steps are RESTRICTED to loopless BWTs (e_2_self == 0)?
"""
import random
import sys

sys.path.insert(0, "/tmp/fcp-ref")
import p1_potential_lp as p1
import p1b_deep_potential_farkas as p1b


def check_phi_self_kill():
    print("=== CHECK 1: HUNTING COUNTEREXAMPLE TO r_next <= r + e_2_self ===")
    # Try: 0^1 1 2, 0^2 1 3, 0^3 1 4, 0^1 5 2, 0^2 5 3, 0^3 5 4 (TWO follower buckets 1 and 5!)
    for K in [2, 3, 4]:
        for B in [2, 3, 4]:
            # B distinct follower buckets b_1..b_B, each preceded by 0^1..0^K and followed by unique tag!
            T = []
            nxt_sym = 1 + B
            for bi in range(B):
                b = 1 + bi
                for j in range(1, K + 1):
                    T.extend([0] * j + [b, nxt_sym])
                    nxt_sym += 1
            T = tuple(T)
            L, F, LF, C, p, s = p1.bwt_of_word(T)
            f0 = p1.compute_features(L, F, LF, C, p, s, include_text_feats=False)
            res = p1.apply_bdmr_self(L, F, LF, C, p, s)
            f1 = p1.compute_features(*p1.bwt_from_L_only(res[0], 0), include_text_feats=False)
            dr = f1["r"] - f0["r"]
            if dr > f0["e_2_self"]:
                print(f"  [KILLED!] K={K}, B={B} (n={len(T)}, s={s}): r: {f0['r']} -> {f1['r']} (delta_r={dr:+d} > e_2_self={f0['e_2_self']}!)")
                print(f"    T={T[:30]}{'...' if len(T)>30 else ''}")
                print(f"    r_F: {f0['r_F']}->{f1['r_F']}, E_RG: {f0['E_RG']}->{f1['E_RG']}, E_RG_inv: {f0['E_RG_inv']}->{f1['E_RG_inv']}")
                break
        else:
            continue
        break

    # Also find MINIMAL counterexample (smallest n, smallest s) via random + small search!
    best = None
    rng = random.Random(2026)
    for trial in range(4000):
        s = rng.choice([3, 4])
        T = []
        for _ in range(rng.randint(5, 14)):
            c = rng.randrange(s)
            T.extend([c] * (rng.randint(1, 4) if c == 0 else 1))
        T = tuple(T)
        if len(T) < 6 or len(set(T)) < 3 or not p1.is_primitive(T):
            continue
        L, F, LF, C, p, sig = p1.bwt_of_word(T)
        f0 = p1.compute_features(L, F, LF, C, p, sig, include_text_feats=False)
        if f0["e_2_self"] == 0:
            continue
        res = p1.apply_bdmr_self(L, F, LF, C, p, sig)
        if res is None or len(res[0]) < 2 or len(set(res[0])) < 2:
            continue
        f1 = p1.compute_features(*p1.bwt_from_L_only(res[0], 0), include_text_feats=False)
        dr = f1["r"] - f0["r"]
        if dr > f0["e_2_self"]:
            if best is None or (len(T), sig) < (best[0], best[1]):
                best = (len(T), sig, T, f0["r"], f1["r"], f0["e_2_self"], f0["r_F"], f1["r_F"], f0["E_RG"], f1["E_RG"])
    if best:
        print(f"  [MINIMAL RANDOM KILL] n={best[0]}, s={best[1]}, T={best[2]}: r:{best[3]}->{best[4]} (delta_r=+{best[4]-best[3]} > e_2_self={best[5]}), r_F:{best[6]}->{best[7]}, E_RG:{best[8]}->{best[9]}")
    else:
        print("  (No random kill found in 4000 trials)")


def check_loopless_core10():
    print("\n=== CHECK 2: LOOPLESS-ONLY PAIR STEPS (e_2_self == 0) ON CORE_10 ===")
    B_core = ["r", "r_circ", "sigma", "r_F", "E_RG", "E_RG_inv", "e_2", "r_1", "r_F1"]  # e_2_self is 0!
    tr_ll_all, tr_ll_thresh, tr_ll_mc = [], [], []
    states_map = {}

    def reg(f):
        states_map[tuple(f[k] for k in B_core)] = f

    rng = random.Random(12345)
    words = []
    # Small loopless Lyndon words
    for n in range(4, 9):
        for w in p1.gen_lyndon(n, 3):
            if len(set(w)) == 3 and all(w[i] != w[(i + 1) % n] for i in range(n)):
                words.append(("lyn3", w))
    # Random loopless words n=10..36, s=3..5
    for n in [10, 14, 20, 30]:
        for s in [3, 4, 5]:
            for idx in range(20):
                T = [rng.randrange(s)]
                for _ in range(n - 1):
                    c = rng.randrange(s - 1)
                    if c >= T[-1]:
                        c += 1
                    T.append(c)
                if T[-1] == T[0]:
                    continue
                T = tuple(T)
                if len(set(T)) >= 3 and p1.is_primitive(T):
                    words.append((f"ll_{n}_{s}_{idx}", T))

    for name, T in words:
        L, F, LF, C, p, s = p1.bwt_of_word(T)
        f0 = p1.compute_features(L, F, LF, C, p, s, include_text_feats=False)
        assert f0["e_2_self"] == 0
        reg(f0)
        best_d = -1
        cands = []
        for mask in range(1, (1 << s) - 1):
            in_L = [bool((mask >> c) & 1) for c in range(s)]
            res = p1.apply_bdmr_pair(L, F, LF, C, p, s, in_L)
            if res and len(res[0]) >= 2 and len(set(res[0])) >= 2:
                kappa, _, _ = p1.classify_coloring(in_L, F, L)
                cands.append((in_L, res, kappa))
                if res[2] > best_d:
                    best_d = res[2]
        for in_L, (L1, _, dc), kappa in cands:
            f1 = p1.compute_features(*p1.bwt_from_L_only(L1, 0), include_text_feats=False)
            reg(f1)
            meta = ("ll_pair", name, in_L, dc, f0["r"], f1["r"])
            tr_ll_all.append((f0, f1, meta))
            if kappa == 1:
                tr_ll_thresh.append((f0, f1, meta))
            if dc == best_d and best_d > 0:
                tr_ll_mc.append((f0, f1, meta))

    states = list(states_map.values())
    print(f"  Loopless corpus: {len(words)} words, {len(states)} states | all={len(tr_ll_all)}, thresh={len(tr_ll_thresh)}, mc={len(tr_ll_mc)}")
    for cname, trs in [("ll_pair_all", tr_ll_all), ("ll_thresh", tr_ll_thresh), ("ll_maxcut", tr_ll_mc)]:
        st, Mval, wts = p1b.solve_lp_clean(trs, B_core, states, allow_const=False, l1_reg=True)
        print(f"  {cname:14s} -> {st:10s}" + (f"  M={Mval:.4f} wts={wts}" if st == "FEASIBLE" else ""))
        if st == "INFEASIBLE":
            cert = p1b.exact_farkas_z3(trs, B_core, states, allow_const=False)
            if cert:
                print(f"    [EXACT Z3 FARKAS] verified={cert['exact_z3_verified']} | {cert['num_y']} transitions, {cert['num_u']} states")
                for coeff, meta in cert["y"][:5]:
                    print(f"      y={coeff}  {meta}")


if __name__ == "__main__":
    check_phi_self_kill()
    check_loopless_core10()
