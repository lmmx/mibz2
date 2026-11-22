#!/tmp/solver-venv/bin/python
"""
FCP Cycle 1 — Part 1: Exact LP/Z3 Synthesis of Monotone Potential Phi(L, C) Stronger than r,
and Minimal Farkas Certificates when Infeasible.
"""
import json
import math
import random
import sys
import time
from collections import Counter
from itertools import product

import numpy as np
from ortools.linear_solver import pywraplp
import z3

sys.path.insert(0, "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-neon-teamwork-paradigm/beta")
import deep_analysis as db


def bwt_of_word(T):
    """Exact circular BWT of primitive word T (tuple of ints). Returns (L, F, LF, p, uniq)."""
    n = len(T)
    rots = sorted(range(n), key=lambda i: T[i:] + T[:i])
    p = rots.index(0)
    L_raw = [T[(i - 1) % n] for i in rots]
    uniq = sorted(set(L_raw))
    mp = {c: idx for idx, c in enumerate(uniq)}
    L = [mp[c] for c in L_raw]
    F = sorted(L)
    s = len(uniq)
    C = [0] * (s + 1)
    for c in L:
        C[c + 1] += 1
    for i in range(s):
        C[i + 1] += C[i]
    occ = C[:-1].copy()
    LF = [0] * n
    for i, c in enumerate(L):
        LF[i] = occ[c]
        occ[c] += 1
    return L, F, LF, C, p, s


def bwt_from_L_only(L, p=0):
    """Given already-compressed L (values 0..s-1), compute F, LF, C, s."""
    n = len(L)
    uniq = sorted(set(L))
    mp = {c: idx for idx, c in enumerate(uniq)}
    L = [mp[c] for c in L]
    s = len(uniq)
    F = sorted(L)
    C = [0] * (s + 1)
    for c in L:
        C[c + 1] += 1
    for i in range(s):
        C[i + 1] += C[i]
    occ = C[:-1].copy()
    LF = [0] * n
    for i, c in enumerate(L):
        LF[i] = occ[c]
        occ[c] += 1
    return L, F, LF, C, p, s


def invert_bwt(L, LF, p):
    n = len(L)
    T = [0] * n
    cur = p
    for k in range(n - 1, -1, -1):
        T[k] = L[cur]
        cur = LF[cur]
    return tuple(T)


def is_primitive(T):
    n = len(T)
    for d in range(1, n):
        if n % d == 0 and T == T[:d] * (n // d):
            return False
    return True


def is_lyndon(T):
    n = len(T)
    for i in range(1, n):
        rot = T[i:] + T[:i]
        if rot <= T:
            return False
    return True


def compute_features(L, F, LF, C, p, s, include_text_feats=True):
    """Compute a rich dictionary of run-scale combinatorial invariants of a valid BWT."""
    n = len(L)
    FL = [0] * n
    for i, j in enumerate(LF):
        FL[j] = i

    # 1. Basic run boundaries
    D_bar = {0} | {i for i in range(1, n) if L[i] != L[i - 1]}
    C_bar = set(C[:s])
    r = len(D_bar)
    r_circ = r if L[0] != L[-1] else max(1, r - 1)
    r_F = len(D_bar | C_bar)
    cap_DC = len(D_bar & C_bar)
    straddle = r_F - r  # = s - cap_DC

    # 2. LF-image and FL-image refinements (Run-Graph edges!)
    LF_D = {LF[i] for i in D_bar}
    FL_D = {FL[i] for i in D_bar}
    # Note: C_bar is always a subset of LF_D.
    E_RG = len(D_bar | LF_D)           # common refinement of L-runs and LF(L-runs)
    E_RG_inv = len(D_bar | FL_D)       # common refinement of L-runs and FL(L-runs)
    cap_D_LFD = len(D_bar & LF_D)
    cap_D_FLD = len(D_bar & FL_D)
    # 3-way refinement D_bar | C_bar | FL_D
    ref_DCF = len(D_bar | C_bar | FL_D)
    # 2-step LF refinement
    LF2_D = {LF[LF[i]] for i in D_bar}
    ref_L2 = len(D_bar | LF_D | LF2_D)

    # 3. Context runs: L2[i] = L[LF[i]], F2[i] = F[FL[i]]
    L2 = [L[LF[i]] for i in range(n)]
    F2 = [F[FL[i]] for i in range(n)]
    r_L2 = 1 + sum(1 for i in range(1, n) if L2[i] != L2[i - 1])
    r_L2L = 1 + sum(1 for i in range(1, n) if (L2[i], L[i]) != (L2[i - 1], L[i - 1]))
    r_L2LF = 1 + sum(1 for i in range(1, n) if (L2[i], L[i], F[i]) != (L2[i - 1], L[i - 1], F[i - 1]))
    r_FF2 = 1 + sum(1 for i in range(1, n) if (F[i], F2[i]) != (F[i - 1], F2[i - 1]))
    r_LFF2 = 1 + sum(1 for i in range(1, n) if (L[i], F[i], F2[i]) != (L[i - 1], F[i - 1], F2[i - 1]))

    # 4. Bigrams & run widths
    bigrams = set(zip(L, F))
    e_2 = len(bigrams)
    e_2_self = sum(1 for a, b in bigrams if a == b)
    e_2_neq = e_2 - e_2_self
    trigrams = set(zip(L2, L, F))
    e_3 = len(trigrams)

    # Width-1 run counts
    D_sorted = sorted(D_bar) + [n]
    r_1 = sum(1 for k in range(r) if D_sorted[k + 1] - D_sorted[k] == 1)
    r_ge2 = r - r_1
    DC_sorted = sorted(D_bar | C_bar) + [n]
    r_F1 = sum(1 for k in range(r_F) if DC_sorted[k + 1] - DC_sorted[k] == 1)

    # Descents in run-head sequence
    run_heads = [L[D_sorted[k]] for k in range(r)]
    r_des = sum(1 for k in range(r - 1) if run_heads[k] > run_heads[k + 1])
    r_asc = (r - 1) - r_des

    # Run-graph connected components and cycle rank
    # Vertices are L-runs 0..r-1; edge u->v if LF(I_u) intersects I_v
    run_of = [0] * n
    for k in range(r):
        for pos in range(D_sorted[k], D_sorted[k + 1]):
            run_of[pos] = k
    rg_edges = set()
    for i in range(n):
        rg_edges.add((run_of[i], run_of[LF[i]]))
    rg_E_uniq = len(rg_edges)
    rg_self = sum(1 for u, v in rg_edges if u == v)

    # Same for (L,F)-runs
    frun_of = [0] * n
    for k in range(r_F):
        for pos in range(DC_sorted[k], DC_sorted[k + 1]):
            frun_of[pos] = k
    frg_edges = set((frun_of[i], frun_of[LF[i]]) for i in range(n))
    frg_E_uniq = len(frg_edges)

    # Multi-run excess per symbol and per bigram
    # r - s = excess L-runs; r_F - e_2 = excess (L,F)-runs
    # Max runs of any single symbol
    sym_run_cnt = Counter(run_heads)
    max_sym_runs = max(sym_run_cnt.values())
    # Symbols with >= 2 runs
    s_multi = sum(1 for v in sym_run_cnt.values() if v >= 2)
    # Distinct L-symbols appearing in each F-bucket summed = e_2
    # Sum over F-buckets of (#runs in F-bucket) = r_F

    feats = {
        "r": r,
        "r_circ": r_circ,
        "sigma": s,
        "r_F": r_F,
        "cap_DC": cap_DC,
        "E_RG": E_RG,
        "E_RG_inv": E_RG_inv,
        "cap_D_LFD": cap_D_LFD,
        "cap_D_FLD": cap_D_FLD,
        "ref_DCF": ref_DCF,
        "ref_L2": ref_L2,
        "r_L2": r_L2,
        "r_L2L": r_L2L,
        "r_L2LF": r_L2LF,
        "r_FF2": r_FF2,
        "r_LFF2": r_LFF2,
        "e_2": e_2,
        "e_2_self": e_2_self,
        "e_2_neq": e_2_neq,
        "e_3": e_3,
        "r_1": r_1,
        "r_F1": r_F1,
        "r_des": r_des,
        "rg_E_uniq": rg_E_uniq,
        "rg_self": rg_self,
        "frg_E_uniq": frg_E_uniq,
        "max_sym_runs": max_sym_runs,
        "s_multi": s_multi,
        "const_1": 1,
    }

    if include_text_feats:
        T = invert_bwt(L, LF, p)
        # Reversed BWT runs r(T^rev)
        Trev = T[::-1]
        Lrev, _, _, _, _, _ = bwt_of_word(Trev)
        r_rev = 1 + sum(1 for i in range(1, n) if Lrev[i] != Lrev[i - 1])
        feats["r_rev"] = r_rev
        # Substring complexity delta = max_k d_k(T)/k (exact for circular T)
        T2 = T + T
        best_delta = 0.0
        for k in range(1, n + 1):
            dk = len({T2[i:i + k] for i in range(n)})
            if dk / k > best_delta:
                best_delta = dk / k
        feats["delta"] = best_delta
        # Circular runs in T itself (NOT run-scale on general words, track separately!)
        r_text = sum(1 for i in range(n) if T[i] != T[(i - 1) % n])
        feats["r_text"] = max(1, r_text)
        feats["n"] = n

    return feats


def apply_bdmr_pair(L, F, LF, C, p, s, in_L):
    """Exact BDMR full_split_pair_step directly on (L, C, p). Returns (L_new, p_new, del_cnt, splits) or None."""
    n = len(L)
    runs = db.runs_from_seq(L)
    br = [[] for _ in range(s)]
    pos, fs = 0, 0
    for c, w in runs:
        rem, cp = w, pos
        while rem > 0:
            while C[fs + 1] <= cp:
                fs += 1
            tk = min(rem, C[fs + 1] - cp)
            br[fs].append((c, tk))
            cp += tk
            rem -= tk
        pos += w
    rules = [('base', c) for c in range(s)]
    res = db.full_split_pair_step(runs, s, n, p, 0, [1] * s, list(range(s)), rules, C, br, in_L)
    if res is None:
        return None
    out_runs, _, new_p, del_cnt, _, new_sig = res
    L_new = []
    for c, w in out_runs:
        L_new.extend([c] * w)
    return L_new, new_p, del_cnt


def apply_bdmr_self(L, F, LF, C, p, s):
    n = len(L)
    runs = db.runs_from_seq(L)
    br = [[] for _ in range(s)]
    pos, fs = 0, 0
    for c, w in runs:
        rem, cp = w, pos
        while rem > 0:
            while C[fs + 1] <= cp:
                fs += 1
            tk = min(rem, C[fs + 1] - cp)
            br[fs].append((c, tk))
            cp += tk
            rem -= tk
        pos += w
    rules = [('base', c) for c in range(s)]
    res = db.self_run_contraction_step(runs, s, n, p, 0, [1] * s, list(range(s)), rules, C, br)
    if res is None or res[6] is not None:
        return None
    out_runs, _, new_p, del_cnt, _, new_sig, _ = res
    L_new = []
    for c, w in out_runs:
        L_new.extend([c] * w)
    return L_new, new_p, del_cnt


def classify_coloring(in_L, F, L):
    s = len(in_L)
    # kappa: number of contiguous intervals of True in in_L
    kappa = 0
    prev = False
    for b in in_L:
        if b and not prev:
            kappa += 1
        prev = b
    # e_R: transitions R->R
    e_R = sum(1 for l, f in zip(L, F) if (not in_L[l]) and (not in_L[f]))
    # Also check flipped kappa (threshold cut can have Sigma_L as prefix OR suffix!)
    # Wait: in 1Cut-BWT, kappa=1 means Sigma_L is a single contiguous interval (prefix, suffix, or middle).
    # Actually Lazard uses |Sigma_R|=1 at min or max.
    is_lazard = (sum(1 for x in in_L if not x) == 1) and (not in_L[0] or not in_L[-1]) and (e_R == 0)
    return kappa, e_R, is_lazard


def gen_lyndon(n, k):
    """Generate all Lyndon words of length n over 0..k-1 using Duval/FKM."""
    w = [-1]
    while w:
        w[-1] += 1
        m = len(w)
        if m == n:
            yield tuple(w)
        while len(w) < n:
            w.append(w[-m])
        while w and w[-1] == k - 1:
            w.pop()


def solve_potential_lp(transitions, feat_names, BWT_feats, label="", require_r_lower=True):
    """
    Search for w in R^d such that:
      - For every transition (f0, f1): w^T (f1 - f0) <= 0
      - Normalize: w_r = 1 (or w^T f(B) >= r(B) for all BWTs)
      - Optionally require w_i >= 0? NO! Allow signed weights first (more general!),
        subject to w^T f(B) >= r(B) for all BWTs B (so Phi bounds r from above!)
        AND w^T f(B) <= M * r(B) (or just check if ANY w with w^T f(B) >= r(B) exists!).
    Wait: if we don't bound w^T f(B) from above, could w pick some combination?
    Note all feat_names here are RUN-SCALE (we exclude n and r_text from feat_names!).
    Let's test BOTH:
      Mode 1 (Unrestricted signed w, Phi(B) >= r(B) for all B):
        If INFEASIBLE, NO signed linear combination of feat_names can bound r and be monotone!
      Mode 2 (Nonnegative w >= 0 with w_r >= 1): stricter.
    """
    d = len(feat_names)
    idx = {name: i for i, name in enumerate(feat_names)}

    solver = pywraplp.Solver.CreateSolver("GLOP")
    w = [solver.NumVar(-1000.0, 1000.0, name) for name in feat_names]
    M_var = solver.NumVar(0.0, 1000.0, "M_ratio")

    # Lower bound: Phi(B) >= r(B) for all observed BWTs B
    # Upper bound: Phi(B) <= M_var * r(B) (minimizing M_var!)
    for bi, f in enumerate(BWT_feats):
        expr = solver.Sum(w[j] * f[feat_names[j]] for j in range(d))
        if require_r_lower:
            solver.Add(expr >= f["r"], f"lb_{bi}")
            solver.Add(expr <= M_var * f["r"], f"ub_{bi}")

    # Monotonicity constraints: w^T (f1 - f0) <= 0
    trans_cons = []
    for ti, (f0, f1, meta) in enumerate(transitions):
        diff = [f1[feat_names[j]] - f0[feat_names[j]] for j in range(d)]
        if all(x == 0 for x in diff):
            trans_cons.append(None)
            continue
        c = solver.Add(solver.Sum(w[j] * diff[j] for j in range(d)) <= 0.0, f"tr_{ti}")
        trans_cons.append(c)

    solver.Minimize(M_var)
    status = solver.Solve()
    if status == pywraplp.Solver.OPTIMAL:
        weights = {feat_names[j]: round(w[j].solution_value(), 6) for j in range(d) if abs(w[j].solution_value()) > 1e-6}
        return "FEASIBLE", M_var.solution_value(), weights
    else:
        return "INFEASIBLE", None, None


def find_farkas_certificate(transitions, feat_names, BWT_feats):
    """
    Use LP phase-1 / dual to find a small Farkas certificate of infeasibility:
    maximize sum_b u_b * r(b) - ... wait:
    Primal: exists w in R^d:
      A_tr w <= 0    (monotonicity)
      -A_lb w <= -r  (Phi >= r)
    Farkas Dual: exists y >= 0 (on transitions), u >= 0 (on BWTs) such that:
      A_tr^T y - A_lb^T u = 0
      r^T u = 1.
    Minimize sum(y) + sum(u) (L1-sparse Farkas certificate!!).
    """
    d = len(feat_names)
    solver = pywraplp.Solver.CreateSolver("GLOP")
    # Filter distinct diff vectors to keep LP small and fast
    y_vars = []
    diffs = []
    metas = []
    seen = {}
    for ti, (f0, f1, meta) in enumerate(transitions):
        diff = tuple(f1[name] - f0[name] for name in feat_names)
        if all(x == 0 for x in diff):
            continue
        if diff not in seen:
            seen[diff] = len(diffs)
            diffs.append(diff)
            metas.append(meta)
            y_vars.append(solver.NumVar(0.0, solver.infinity(), f"y_{len(diffs)-1}"))

    u_vars = [solver.NumVar(0.0, solver.infinity(), f"u_{bi}") for bi in range(len(BWT_feats))]

    # Balance constraints for each feature j
    for j, name in enumerate(feat_names):
        lhs = solver.Sum(diffs[k][j] * y_vars[k] for k in range(len(diffs)) if diffs[k][j] != 0)
        rhs = solver.Sum(BWT_feats[bi][name] * u_vars[bi] for bi in range(len(BWT_feats)) if BWT_feats[bi][name] != 0)
        solver.Add(lhs == rhs)

    # Normalization: sum u_b * r(b) == 1
    solver.Add(solver.Sum(BWT_feats[bi]["r"] * u_vars[bi] for bi in range(len(BWT_feats))) == 1.0)

    # Minimize L1 norm for maximal sparsity
    solver.Minimize(solver.Sum(y_vars) + solver.Sum(u_vars))
    status = solver.Solve()
    if status != pywraplp.Solver.OPTIMAL:
        return None
    nz_y = [(k, y_vars[k].solution_value(), metas[k], diffs[k]) for k in range(len(diffs)) if y_vars[k].solution_value() > 1e-7]
    nz_u = [(bi, u_vars[bi].solution_value(), BWT_feats[bi]["r"]) for bi in range(len(BWT_feats)) if u_vars[bi].solution_value() > 1e-7]
    return {"num_y": len(nz_y), "num_u": len(nz_u), "y": nz_y, "u": nz_u}


def main():
    t0 = time.time()
    # Build diverse corpus of valid primitive BWTs
    words = []
    # All binary Lyndon words n=3..9
    for n in range(3, 10):
        for w in gen_lyndon(n, 2):
            if len(set(w)) >= 2:
                words.append(w)
    # All ternary Lyndon words n=3..7
    for n in range(3, 8):
        for w in gen_lyndon(n, 3):
            if len(set(w)) == 3:
                words.append(w)
    # All quaternary Lyndon words n=4..5
    for n in range(4, 6):
        for w in gen_lyndon(n, 4):
            if len(set(w)) == 4:
                words.append(w)

    # Add F4 witnesses and Fibonacci words + $
    special = [
        (0, 0, 1, 1, 2),
        (0, 0, 2, 0, 1),
        (0, 0, 1, 0, 1),
        (1, 2, 3, 2, 0),
        (0, 0, 1, 1, 2, 1, 2, 0, 0, 1),
    ]
    # Fibonacci words + $ (0=$ < 1=a < 2=b)
    fa, fb = (1,), (1, 2)
    for m in range(3, 9):
        fa, fb = fb, fb + fa
        special.append(fb + (0,))
    # Random ternary/quaternary n=8..14
    rng = random.Random(20261122)
    for n in [8, 9, 10, 12, 14]:
        for s in [3, 4]:
            for _ in range(30):
                w = tuple(rng.randrange(s) for _ in range(n))
                if len(set(w)) >= 2 and is_primitive(w):
                    special.append(w)

    # Deduplicate by canonical rotation (minimal rotation)
    seen_words = set()
    uniq_words = []
    for w in words + special:
        if not is_primitive(w):
            continue
        n = len(w)
        # Canonicalize alphabet to 0..s-1 preserving order
        u = sorted(set(w))
        mp = {c: i for i, c in enumerate(u)}
        wc = tuple(mp[c] for c in w)
        canon = min(wc[i:] + wc[:i] for i in range(n))
        if canon not in seen_words:
            seen_words.add(canon)
            uniq_words.append(canon)

    print(f"Corpus: {len(uniq_words)} unique primitive words ({time.time()-t0:.2f}s)")

    # Collect transitions by move class
    trans_by_class = {
        "all_bdmr": [],
        "pair_all": [],
        "self_only": [],
        "thresh_and_self": [],
        "maxcut_and_self": [],
        "maxcut_pair_only": [],
        "ge_n4_pair_only": [],
        "lazard": [],
    }
    BWT_feats_map = {}

    def get_feats_from_L(L, p):
        key = tuple(L)
        if key not in BWT_feats_map:
            Lc, F, LF, C, p0, s = bwt_from_L_only(L, p)
            BWT_feats_map[key] = compute_features(Lc, F, LF, C, p0, s, include_text_feats=True)
        return BWT_feats_map[key]

    for w in uniq_words:
        L, F, LF, C, p, s = bwt_of_word(w)
        f0 = get_feats_from_L(L, p)
        n = len(L)

        # Self-run contraction
        res_self = apply_bdmr_self(L, F, LF, C, p, s)
        if res_self is not None:
            L1, p1, dcnt = res_self
            if len(L1) >= 2 and len(set(L1)) >= 2:
                f1 = get_feats_from_L(L1, p1)
                meta = ("self", w, None, dcnt, f0["r"], f1["r"])
                trans_by_class["all_bdmr"].append((f0, f1, meta))
                trans_by_class["self_only"].append((f0, f1, meta))
                trans_by_class["thresh_and_self"].append((f0, f1, meta))
                trans_by_class["maxcut_and_self"].append((f0, f1, meta))

        # All non-trivial 2-colorings (s <= 5 -> at most 30)
        if s <= 5:
            pair_results = []
            max_del = -1
            for mask in range(1, (1 << s) - 1):
                in_L = [bool((mask >> c) & 1) for c in range(s)]
                res_p = apply_bdmr_pair(L, F, LF, C, p, s, in_L)
                if res_p is None:
                    continue
                L1, p1, dcnt = res_p
                if len(L1) < 2 or len(set(L1)) < 2:
                    continue
                kappa, e_R, is_laz = classify_coloring(in_L, F, L)
                pair_results.append((in_L, L1, p1, dcnt, kappa, e_R, is_laz))
                if dcnt > max_del:
                    max_del = dcnt

            for in_L, L1, p1, dcnt, kappa, e_R, is_laz in pair_results:
                f1 = get_feats_from_L(L1, p1)
                meta = ("pair", w, in_L, dcnt, f0["r"], f1["r"])
                trans_by_class["all_bdmr"].append((f0, f1, meta))
                trans_by_class["pair_all"].append((f0, f1, meta))
                if kappa == 1:
                    trans_by_class["thresh_and_self"].append((f0, f1, meta))
                if dcnt == max_del and max_del > 0:
                    trans_by_class["maxcut_and_self"].append((f0, f1, meta))
                    trans_by_class["maxcut_pair_only"].append((f0, f1, meta))
                if 4 * dcnt >= n:
                    trans_by_class["ge_n4_pair_only"].append((f0, f1, meta))
                if is_laz:
                    trans_by_class["lazard"].append((f0, f1, meta))

    BWT_feats = list(BWT_feats_map.values())
    print(f"Unique BWT states: {len(BWT_feats)} ({time.time()-t0:.2f}s)")
    for k, v in trans_by_class.items():
        print(f"  class {k:20s}: {len(v)} transitions")

    # Feature sets to test:
    # Set A: Pure BWT/run-space local invariants (no text-length n, no r_text)
    feats_local = [
        "r", "r_circ", "sigma", "r_F", "cap_DC",
        "E_RG", "E_RG_inv", "cap_D_LFD", "cap_D_FLD",
        "ref_DCF", "ref_L2",
        "r_L2", "r_L2L", "r_L2LF", "r_FF2", "r_LFF2",
        "e_2", "e_2_self", "e_2_neq", "e_3",
        "r_1", "r_F1", "r_des",
        "rg_E_uniq", "rg_self", "frg_E_uniq",
        "max_sym_runs", "s_multi", "const_1",
    ]
    # Set B: Set A + global text-compression invariants (r_rev, delta) — still O(r polylog n)!
    feats_plus_global = feats_local + ["r_rev", "delta"]
    # Set C: Set B + r_text (NOT run-scale, sanity check!)
    feats_with_rtext = feats_plus_global + ["r_text"]

    print("\n=== LP POTENTIAL SYNTHESIS (Phi(B) >= r(B), signed weights allowed, minimizing max Phi/r) ===")
    results = {}
    for fset_name, fset in [("LOCAL_29", feats_local), ("PLUS_RREV_DELTA_31", feats_plus_global), ("WITH_RTEXT_32", feats_with_rtext)]:
        print(f"\n--- Feature Set: {fset_name} ---")
        results[fset_name] = {}
        for cls_name, trs in trans_by_class.items():
            st, Mval, wts = solve_potential_lp(trs, fset, BWT_feats, cls_name)
            print(f"  {cls_name:20s} -> {st:10s}" + (f"  max(Phi/r)={Mval:.4f}  wts={wts}" if st == "FEASIBLE" else ""))
            results[fset_name][cls_name] = {"status": st, "M": Mval, "weights": wts}
            if st == "INFEASIBLE" and fset_name in ("LOCAL_29", "PLUS_RREV_DELTA_31"):
                cert = find_farkas_certificate(trs, fset, BWT_feats)
                if cert:
                    print(f"    [FARKAS CERT] {cert['num_y']} transitions, {cert['num_u']} BWT lower-bounds:")
                    for k, val, meta, _ in cert["y"][:10]:
                        print(f"      y={val:.4f}  {meta[0]} T={meta[1]} inL={meta[2]} del={meta[3]} r:{meta[4]}->{meta[5]}")
                    results[fset_name][cls_name]["farkas_summary"] = {
                        "num_y": cert["num_y"],
                        "num_u": cert["num_u"],
                        "top_y": [(round(val, 6), meta[0], list(meta[1]), meta[2], meta[3], meta[4], meta[5]) for _, val, meta, _ in cert["y"][:12]],
                    }

    # ALSO: What if we drop "Phi(B) >= r(B)" and just check if ANY NONZERO w has w^T (f1 - f0) <= 0 on all_bdmr
    # AND w^T f(B) >= 0, w^T f(B_max) >= 1 (i.e. ANY non-constant monotone linear functional on LOCAL_29!)?
    print("\n=== CHECKING FOR *ANY* NON-CONSTANT MONOTONE LINEAR INVARIANT (not even requiring Phi >= r) ===")
    for fset_name, fset in [("LOCAL_29", feats_local), ("PLUS_RREV_DELTA_31", feats_plus_global)]:
        for cls_name in ["all_bdmr", "pair_all", "thresh_and_self", "maxcut_and_self", "maxcut_pair_only"]:
            trs = trans_by_class[cls_name]
            # Check for each individual feature g whether exists w with w^T df <= 0, w^T f >= 0, and w^T f(B) >= 1 for some B (actually maximize w^T (f(B_big) - f(B_small)))
            # Even sharper: maximize sum_B w^T f(B) subject to w^T df <= 0, -1 <= w_j <= 1, and w^T f(B) >= 0, w[const_1] == 0.
            d = len(fset)
            solver = pywraplp.Solver.CreateSolver("GLOP")
            w = [solver.NumVar(-10.0, 10.0, name) for name in fset]
            solver.Add(w[fset.index("const_1")] == 0.0)
            for f in BWT_feats:
                solver.Add(solver.Sum(w[j] * f[fset[j]] for j in range(d)) >= 0.0)
            for f0, f1, _ in trs:
                diff = [f1[fset[j]] - f0[fset[j]] for j in range(d)]
                if any(x != 0 for x in diff):
                    solver.Add(solver.Sum(w[j] * diff[j] for j in range(d)) <= 0.0)
            # Maximize average Phi(B)
            solver.Maximize(solver.Sum(w[j] * sum(f[fset[j]] for f in BWT_feats) / len(BWT_feats) for j in range(d)))
            st = solver.Solve()
            if st == pywraplp.Solver.OPTIMAL:
                obj = solver.Objective().Value()
                wts = {fset[j]: round(w[j].solution_value(), 5) for j in range(d) if abs(w[j].solution_value()) > 1e-5}
                print(f"  [{fset_name}] {cls_name:18s}: max avg Phi = {obj:.4f}  wts={wts}")
            else:
                print(f"  [{fset_name}] {cls_name:18s}: status={st}")

    with open("/tmp/fcp-ref/p1_results.json", "w") as fp:
        json.dump(results, fp, indent=2, default=str)
    print(f"\nDone in {time.time()-t0:.2f}s")


if __name__ == "__main__":
    main()
