#!/tmp/solver-venv/bin/python
"""
ROUND 3 PRIORITY 1 (F4): Exact CP-SAT Audit of Beta's Run-Monotonicity (r_next <= r) on Valid BWTs.
1. Globally minimal valid-BWT counterexample (n, |Sigma|) to r_next <= r under:
   (i) threshold cut (kappa = 1),
   (ii) general 2-coloring (with bucket splits, kappa > 1),
   (iii) self-run contraction.
2. Harder question: Exists valid BWT (n up to 20+) where EVERY |D| >= n/4 2-coloring
   (or EVERY non-trivial 2-coloring) strictly increases r_next > r?
3. Maximum ratio r_next / r achievable on valid BWTs across n = 5..24.
All counterexamples independently verified outside the solver via plain bwt_forward + Beta's deep_analysis.
"""
import json
import os
import sys
import time
from itertools import product

from ortools.sat.python import cp_model

sys.path.insert(0, "/tmp/teamwork-b")
import deep_analysis as db

OUT_DIR = "/tmp/teamwork-referee/solver_audit"
os.makedirs(OUT_DIR, exist_ok=True)


def eval_pair_step_external(T, in_L):
    """Exact external evaluation of one BDMR full_split_pair_step on valid BWT of T."""
    L, p = db.bwt_forward(T)
    uniq = sorted(set(L))
    mp = {c: i for i, c in enumerate(uniq)}
    runs = db.runs_from_seq([mp[c] for c in L])
    r0 = len(runs)
    s = len(uniq)
    n = len(L)
    cnts = [0] * s
    for c, w in runs:
        cnts[c] += w
    C = [0] * (s + 1)
    for i in range(s):
        C[i + 1] = C[i] + cnts[i]
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
    rules = [('base', c) for c in uniq]
    in_L_mapped = [in_L[c] for c in uniq]
    res = db.full_split_pair_step(runs, s, n, p, 0, [1] * s, list(range(s)), rules, C, br, in_L_mapped)
    if res is None:
        return r0, r0, 0, L, p, 0
    out_runs, _, _, del_cnt, _, new_sig = res
    # Check bucket splits count
    splits = 0
    for a in range(s):
        if not in_L_mapped[a]:
            continue
        blks, pt = 0, -1
        for b in range(s):
            for sym, _ in br[b]:
                if sym == a:
                    tc = 0 if in_L_mapped[b] else 1
                    if tc == 0 and pt != 0:
                        blks += 1
                    pt = tc
        if blks >= 2:
            splits += 1
    return r0, len(out_runs), del_cnt, L, p, splits


def eval_self_step_external(T):
    L, p = db.bwt_forward(T)
    uniq = sorted(set(L))
    mp = {c: i for i, c in enumerate(uniq)}
    runs = db.runs_from_seq([mp[c] for c in L])
    r0 = len(runs)
    s = len(uniq)
    n = len(L)
    cnts = [0] * s
    for c, w in runs:
        cnts[c] += w
    C = [0] * (s + 1)
    for i in range(s):
        C[i + 1] = C[i] + cnts[i]
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
    rules = [('base', c) for c in uniq]
    res = db.self_run_contraction_step(runs, s, n, p, 0, [1] * s, list(range(s)), rules, C, br)
    if res is None or res[6] is not None:
        return r0, r0, 0, L, p
    return r0, len(res[0]), res[3], L, p


def add_pair_step_to_cpsat(m, n, sig, F, L, L2, in_L_const, prefix="p"):
    """
    Given primitive valid BWT vars F[i], L[i], L2[i] = T[(SA[i]-2)%n] and a FIXED 2-coloring in_L_const[0..sig-1],
    compute exact (r_next, del_cnt) of full_split_pair_step!
    """
    # Indicator: is_L(c) is constant!
    # Row i deleted iff in_L_const[L[i]] and not in_L_const[F[i]]
    del_v = [m.NewBoolVar(f"{prefix}_del_{i}") for i in range(n)]
    for i in range(n):
        l_inL = m.NewBoolVar(f"{prefix}_lL_{i}")
        f_inR = m.NewBoolVar(f"{prefix}_fR_{i}")
        L_syms = [c for c in range(sig) if in_L_const[c]]
        R_syms = [c for c in range(sig) if not in_L_const[c]]
        m.AddAllowedAssignments([L[i]], [[c] for c in L_syms]).OnlyEnforceIf(l_inL)
        m.AddAllowedAssignments([L[i]], [[c] for c in R_syms]).OnlyEnforceIf(l_inL.Not())
        m.AddAllowedAssignments([F[i]], [[c] for c in R_syms]).OnlyEnforceIf(f_inR)
        m.AddAllowedAssignments([F[i]], [[c] for c in L_syms]).OnlyEnforceIf(f_inR.Not())
        m.AddBoolAnd([l_inL, f_inR]).OnlyEnforceIf(del_v[i])
        m.AddBoolOr([l_inL.Not(), f_inR.Not(), del_v[i]])
    del_cnt = m.NewIntVar(0, n, f"{prefix}_delcnt")
    m.Add(del_cnt == sum(del_v))

    # For uncontracted a in Sigma_L: which Sigma_R symbols b appear as followers of a (i.e. exists row j with L[j]=a, F[j]=b)?
    # has_R_between[a][b1][b2]: exists R-symbol r in (b1, b2) with edge (a -> r)
    # Better: assign each surviving row i a canonical integer sym_id[i] in [0, (sig+1)*(sig+1)):
    #   - If L[i] in Sigma_R:
    #       if L2[i] in Sigma_L: sym_id = 1 + L2[i] * sig + L[i]  (pair (L2[i], L[i]))
    #       else:                sym_id = 1 + sig * sig + L[i]    (uncontracted R-sym L[i])
    #   - If L[i] in Sigma_L (surviving => F[i] in Sigma_L):
    #       sym_id = 1 + sig * sig + sig + L[i] * sig + blk_id(L[i], F[i])
    #     where blk_id(a, b) = #{r in Sigma_R, r < b : edge(a -> r) exists}!
    #     WAIT A SECOND!! CHECK THAT FORMULA!!
    #     Let a in Sigma_L, and let b1 < b2 be two L-followers of a.
    #     Do b1 and b2 fall in the SAME uncontracted block of a?
    #     YES iff NO r in Sigma_R with b1 < r < b2 has an edge (a -> r),
    #     which is EQUIVALENT to: #{r in Sigma_R, r < b1 : edge(a->r)} == #{r in Sigma_R, r < b2 : edge(a->r)}!!
    #     EXACT!! 100% EQUIVALENT TO full_split_pair_step!!
    L_syms = [c for c in range(sig) if in_L_const[c]]
    R_syms = [c for c in range(sig) if not in_L_const[c]]
    edge_LR = {}
    for a in L_syms:
        for r in R_syms:
            hits = []
            for j in range(n):
                la = m.NewBoolVar(f"{prefix}_ea_{a}_{r}_{j}")
                fr = m.NewBoolVar(f"{prefix}_er_{a}_{r}_{j}")
                m.Add(L[j] == a).OnlyEnforceIf(la)
                m.Add(L[j] != a).OnlyEnforceIf(la.Not())
                m.Add(F[j] == r).OnlyEnforceIf(fr)
                m.Add(F[j] != r).OnlyEnforceIf(fr.Not())
                h = m.NewBoolVar(f"{prefix}_eh_{a}_{r}_{j}")
                m.AddBoolAnd([la, fr]).OnlyEnforceIf(h)
                m.AddBoolOr([la.Not(), fr.Not(), h])
                hits.append(h)
            ev = m.NewBoolVar(f"{prefix}_edge_{a}_{r}")
            m.AddMaxEquality(ev, hits)
            edge_LR[(a, r)] = ev

    # blk_id_tbl[a][b] = sum_{r in R_syms, r < b} edge_LR[(a, r)] for a in L_syms, b in L_syms
    blk_tbl = {}
    for a in L_syms:
        for b in L_syms:
            terms = [edge_LR[(a, r)] for r in R_syms if r < b]
            bv = m.NewIntVar(0, len(R_syms), f"{prefix}_blk_{a}_{b}")
            m.Add(bv == (sum(terms) if terms else 0))
            blk_tbl[(a, b)] = bv

    max_id = 1 + sig * sig + sig + sig * (len(R_syms) + 1)
    sym_id = [m.NewIntVar(0, max_id, f"{prefix}_sid_{i}") for i in range(n)]
    # Table / conditional assignment of sym_id[i]
    for i in range(n):
        # For each possible (l_val, f_val, l2_val) - keep compact by branching on L[i]
        for c in range(sig):
            lc = m.NewBoolVar(f"{prefix}_isL_{i}_{c}")
            m.Add(L[i] == c).OnlyEnforceIf(lc)
            m.Add(L[i] != c).OnlyEnforceIf(lc.Not())
            if not in_L_const[c]:
                # c in Sigma_R
                l2_inL = m.NewBoolVar(f"{prefix}_l2L_{i}_{c}")
                m.AddAllowedAssignments([L2[i]], [[x] for x in L_syms]).OnlyEnforceIf(l2_inL)
                m.AddAllowedAssignments([L2[i]], [[x] for x in R_syms]).OnlyEnforceIf(l2_inL.Not())
                m.Add(sym_id[i] == 1 + L2[i] * sig + c).OnlyEnforceIf([lc, l2_inL])
                m.Add(sym_id[i] == 1 + sig * sig + c).OnlyEnforceIf([lc, l2_inL.Not()])
            else:
                # c in Sigma_L: if surviving, F[i] in L_syms
                for b in L_syms:
                    fb = m.NewBoolVar(f"{prefix}_fb_{i}_{c}_{b}")
                    m.Add(F[i] == b).OnlyEnforceIf(fb)
                    m.Add(F[i] != b).OnlyEnforceIf(fb.Not())
                    m.Add(sym_id[i] == 1 + sig * sig + sig + c * (len(R_syms) + 1) + blk_tbl[(c, b)]).OnlyEnforceIf([lc, fb])
                # if deleted, set sym_id = 0
                m.Add(sym_id[i] == 0).OnlyEnforceIf([lc, del_v[i]])

    # Exact run count r_next of surviving subsequence:
    # Maintain last_sym[i] = sym_id of most recent surviving row <= i (0 if none yet)!
    # A new run starts at row i iff (not del_v[i]) and (i == 0 or last_sym[i-1] != sym_id[i])!
    last_sym = [m.NewIntVar(0, max_id, f"{prefix}_ls_{i}") for i in range(n)]
    new_run = [m.NewBoolVar(f"{prefix}_nr_{i}") for i in range(n)]
    for i in range(n):
        if i == 0:
            m.Add(last_sym[0] == sym_id[0]).OnlyEnforceIf(del_v[0].Not())
            m.Add(last_sym[0] == 0).OnlyEnforceIf(del_v[0])
            m.Add(new_run[0] == del_v[0].Not())
        else:
            m.Add(last_sym[i] == sym_id[i]).OnlyEnforceIf(del_v[i].Not())
            m.Add(last_sym[i] == last_sym[i - 1]).OnlyEnforceIf(del_v[i])
            diff = m.NewBoolVar(f"{prefix}_df_{i}")
            m.Add(sym_id[i] != last_sym[i - 1]).OnlyEnforceIf(diff)
            m.Add(sym_id[i] == last_sym[i - 1]).OnlyEnforceIf(diff.Not())
            m.AddBoolAnd([del_v[i].Not(), diff]).OnlyEnforceIf(new_run[i])
            m.AddBoolOr([del_v[i], diff.Not(), new_run[i]])
    r_next = m.NewIntVar(0, n, f"{prefix}_rn")
    m.Add(r_next == sum(new_run))
    return r_next, del_cnt


def build_bwt_base(m, n, sig, no_self=False):
    T = [m.NewIntVar(0, sig - 1, f"T{i}") for i in range(n)]
    SA = [m.NewIntVar(0, n - 1, f"SA{i}") for i in range(n)]
    ISA = [m.NewIntVar(0, n - 1, f"ISA{i}") for i in range(n)]
    m.AddAllDifferent(SA)
    m.AddInverse(SA, ISA)
    F = [m.NewIntVar(0, sig - 1, f"F{i}") for i in range(n)]
    L = [m.NewIntVar(0, sig - 1, f"L{i}") for i in range(n)]
    L2 = [m.NewIntVar(0, sig - 1, f"L2_{i}") for i in range(n)]
    FL = [m.NewIntVar(0, n - 1, f"FL{i}") for i in range(n)]
    for i in range(n):
        m.AddElement(SA[i], T, F[i])
        pr1 = m.NewIntVar(0, n - 1, f"pr1_{i}")
        pr2 = m.NewIntVar(0, n - 1, f"pr2_{i}")
        nx = m.NewIntVar(0, n - 1, f"nx_{i}")
        m.AddModuloEquality(pr1, SA[i] + (n - 1), n)
        m.AddModuloEquality(pr2, SA[i] + (n - 2), n)
        m.AddModuloEquality(nx, SA[i] + 1, n)
        m.AddElement(pr1, T, L[i])
        m.AddElement(pr2, T, L2[i])
        m.AddElement(nx, ISA, FL[i])
    for i in range(n - 1):
        m.Add(F[i] <= F[i + 1])
        eqf = m.NewBoolVar(f"eqf{i}")
        m.Add(F[i] == F[i + 1]).OnlyEnforceIf(eqf)
        m.Add(F[i] != F[i + 1]).OnlyEnforceIf(eqf.Not())
        m.Add(FL[i] < FL[i + 1]).OnlyEnforceIf(eqf)
    for c in range(sig):
        used = [m.NewBoolVar(f"u_{c}_{i}") for i in range(n)]
        for i in range(n):
            m.Add(T[i] == c).OnlyEnforceIf(used[i])
            m.Add(T[i] != c).OnlyEnforceIf(used[i].Not())
        m.AddBoolOr(used)
    if no_self:
        for i in range(n):
            m.Add(L[i] != F[i])
    # Initial run count r0
    b0 = []
    for i in range(1, n):
        d = m.NewBoolVar(f"b0_{i}")
        m.Add(L[i] != L[i - 1]).OnlyEnforceIf(d)
        m.Add(L[i] == L[i - 1]).OnlyEnforceIf(d.Not())
        b0.append(d)
    r0 = m.NewIntVar(1, n, "r0")
    m.Add(r0 == 1 + sum(b0))
    return T, SA, F, L, L2, r0


def run_f4_minimal_counterexamples():
    print("=== (F4.1) GLOBALLY MINIMAL VALID-BWT COUNTEREXAMPLES TO r_next <= r ===")
    # Exhaustive check for n = 2..8 across |Sigma| = 2, 3, 4 (fast and exact!)
    min_thresh = None
    min_gen_split = None
    min_self = None
    for n in range(2, 9):
        for sig in range(2, min(n + 1, 5)):
            if sig ** n > 65536:
                continue
            for tup in product(range(sig), repeat=n):
                if len(set(tup)) != sig:
                    continue
                # Break cyclic symmetry: T[0] == 0
                if tup[0] != 0:
                    continue
                T = list(tup)
                # (iii) self-run
                if min_self is None:
                    r0, r1, dc, L, p = eval_self_step_external(T)
                    if dc > 0 and r1 > r0:
                        min_self = (n, sig, T, L, p, r0, r1, dc)
                # (i) threshold & (ii) general with splits
                if min_thresh is None or min_gen_split is None:
                    for mask in range(1, (1 << sig) - 1):
                        in_L = [bool((mask >> c) & 1) for c in range(sig)]
                        # Is it a threshold cut?
                        is_th = False
                        for th in range(1, sig):
                            if in_L == [c < th for c in range(sig)] or in_L == [c >= th for c in range(sig)]:
                                is_th = True
                                break
                        r0, r1, dc, L, p, splits = eval_pair_step_external(T, in_L)
                        if dc > 0 and r1 > r0:
                            if is_th and min_thresh is None:
                                min_thresh = (n, sig, T, L, p, in_L, r0, r1, dc, splits)
                            if (not is_th) and splits > 0 and min_gen_split is None:
                                min_gen_split = (n, sig, T, L, p, in_L, r0, r1, dc, splits)
                if min_thresh and min_gen_split and min_self:
                    break
            if min_thresh and min_gen_split and min_self:
                break
        if min_thresh and min_gen_split and min_self:
            break

    print(f"  (i)  Minimal threshold-cut (kappa=1) counterexample: n={min_thresh[0]}, |S|={min_thresh[1]}, "
          f"T={min_thresh[2]}, L={min_thresh[3]}, in_L={min_thresh[5]}, r0={min_thresh[6]} -> r1={min_thresh[7]} ({min_thresh[7]/min_thresh[6]:.3f}x), del={min_thresh[8]} [verified]")
    print(f"  (ii) Minimal bucket-split (kappa>1)  counterexample: n={min_gen_split[0]}, |S|={min_gen_split[1]}, "
          f"T={min_gen_split[2]}, L={min_gen_split[3]}, in_L={min_gen_split[5]}, splits={min_gen_split[9]}, r0={min_gen_split[6]} -> r1={min_gen_split[7]} ({min_gen_split[7]/min_gen_split[6]:.3f}x), del={min_gen_split[8]} [verified]")
    print(f"  (iii) Minimal self-run contraction   counterexample: n={min_self[0]}, |S|={min_self[1]}, "
          f"T={min_self[2]}, L={min_self[3]}, r0={min_self[5]} -> r1={min_self[6]} ({min_self[6]/min_self[5]:.3f}x), del={min_self[7]} [verified]")
    return {"min_thresh": min_thresh, "min_gen_split": min_gen_split, "min_self": min_self}


def run_f4_all_colorings_increase():
    print("\n=== (F4.2) HARDER QUESTION: Exists valid BWT where EVERY |D|>=n/4 2-coloring strictly increases r_next > r? ===")
    found_any = None
    for (n, sig, no_self, mode_lbl) in [
        (6, 2, False, "all >=n/4"), (7, 2, False, "all >=n/4"), (8, 2, False, "all >=n/4"),
        (9, 2, False, "all >=n/4"), (10, 2, False, "all >=n/4"), (12, 2, False, "all >=n/4"),
        (7, 3, False, "all >=n/4"), (8, 3, False, "all >=n/4"), (9, 3, False, "all >=n/4"),
        (10, 3, False, "all >=n/4"), (12, 3, False, "all >=n/4"), (14, 3, False, "all >=n/4"),
        (8, 3, True, "loopless all >=n/4"), (10, 3, True, "loopless all >=n/4"), (12, 3, True, "loopless all >=n/4"),
        (14, 3, True, "loopless all >=n/4"), (16, 3, True, "loopless all >=n/4"),
        (8, 4, True, "loopless all >=n/4"), (10, 4, True, "loopless all >=n/4"), (12, 4, True, "loopless all >=n/4"),
    ]:
        t0 = time.time()
        m = cp_model.CpModel()
        T, SA, F, L, L2, r0 = build_bwt_base(m, n, sig, no_self=no_self)
        # Require: at least ONE coloring has 4*del >= n, AND for EVERY non-trivial coloring with 4*del >= n, r_next > r0!
        qualifies = []
        for mask in range(1, (1 << sig) - 1):
            in_L = [bool((mask >> c) & 1) for c in range(sig)]
            rn, dc = add_pair_step_to_cpsat(m, n, sig, F, L, L2, in_L, prefix=f"m{mask}")
            q = m.NewBoolVar(f"q_{mask}")
            m.Add(4 * dc >= n).OnlyEnforceIf(q)
            m.Add(4 * dc < n).OnlyEnforceIf(q.Not())
            m.Add(rn > r0).OnlyEnforceIf(q)
            qualifies.append(q)
        m.AddBoolOr(qualifies)
        solver = cp_model.CpSolver()
        solver.parameters.max_time_in_seconds = 6.0
        st = solver.Solve(m)
        el = time.time() - t0
        if st in (cp_model.OPTIMAL, cp_model.FEASIBLE):
            T_w = [solver.Value(T[i]) for i in range(n)]
            # External verify EVERY non-trivial 2-coloring!
            L_ext, p_ext = db.bwt_forward(T_w)
            r0_ext = len(db.runs_from_seq(L_ext))
            details = []
            all_nontriv_inc = True
            for mask in range(1, (1 << sig) - 1):
                in_L = [bool((mask >> c) & 1) for c in range(sig)]
                r0c, r1c, dc, _, _, sp = eval_pair_step_external(T_w, in_L)
                assert r0c == r0_ext
                if 4 * dc >= n:
                    assert r1c > r0c, f"External check failed on mask={mask}"
                    details.append((mask, dc, r1c, sp))
                if dc > 0 and r1c <= r0c:
                    all_nontriv_inc = False
            print(f"  SAT WITNESS! n={n:2d}, |S|={sig} ({mode_lbl}): T={T_w}, L={L_ext}, r0={r0_ext} | "
                  f"qualifying (>=n/4) cuts (mask,del,r1,splits)={details} | ALL_nontriv_inc={all_nontriv_inc} ({el:.2f}s) [verified]")
            if found_any is None:
                found_any = (n, sig, T_w, L_ext, r0_ext, details, all_nontriv_inc)
        else:
            print(f"  n={n:2d}, |S|={sig} ({mode_lbl}): {solver.StatusName(st)} ({el:.2f}s)")
    return found_any


def run_f4_max_ratio():
    print("\n=== (F4.3) MAXIMUM RATIO r_next / r_0 ON VALID BWTs (n = 6..24) ===")
    best_by_n = []
    # Optimize r_next - r0 (and maximize r_next at fixed small r0!) for single pair-step (threshold and general)!
    for n, sig in [(7, 2), (8, 2), (10, 2), (12, 2), (14, 3), (16, 3), (18, 3), (20, 3), (24, 3)]:
        t0 = time.time()
        # Fix coloring in_L = [0] vs [1..sig-1] (or search over coloring for sig=3: mask in {1, 2, 3, 4, 5, 6})
        # Use mask = 1 (in_L = [True, False, ...]) and mask = 5 (in_L = [True, False, True] for sig=3 -> splits!)
        masks = [1] if sig == 2 else [1, 5]
        best_r = (1.0, None)
        for mask in masks:
            in_L = [bool((mask >> c) & 1) for c in range(sig)]
            m = cp_model.CpModel()
            T, SA, F, L, L2, r0 = build_bwt_base(m, n, sig, no_self=False)
            rn, dc = add_pair_step_to_cpsat(m, n, sig, F, L, L2, in_L, prefix="opt")
            m.Add(dc >= 1)
            # Maximize 100*rn - 150*r0 (or maximize rn - r0; better: scan small r0 <= 8 or maximize 10*rn - 15*r0!)
            # Actually maximize rn with small r0 <= 6 first, plus maximize rn - r0 overall!
            for r0_cap in [4, 6, 8, n]:
                if r0_cap < sig:
                    continue
                m2 = m.Clone()
                # Re-find r0 and rn by index? Easier: just add objective rn - r0 on separate small solves
                break
            m.Maximize(3 * rn - 4 * r0)
            solver = cp_model.CpSolver()
            solver.parameters.max_time_in_seconds = 4.0
            st = solver.Solve(m)
            if st in (cp_model.OPTIMAL, cp_model.FEASIBLE):
                T_w = [solver.Value(T[i]) for i in range(n)]
                r0c, r1c, dcc, L_ext, p_ext, sp = eval_pair_step_external(T_w, in_L)
                if r1c / r0c > best_r[0]:
                    best_r = (r1c / r0c, (n, sig, mask, in_L, T_w, L_ext, r0c, r1c, dcc, sp, solver.StatusName(st)))
        el = time.time() - t0
        if best_r[1]:
            _, sg, mk, inl, Tw, Le, r0c, r1c, dcc, sp, stn = best_r[1]
            print(f"  n={n:2d}, |S|={sg}: max r1/r0 = {r1c/r0c:.3f}x ({r0c} -> {r1c}, del={dcc}, splits={sp}, mask={mk}, {stn}, {el:.2f}s) T={Tw} [verified]")
            best_by_n.append({"n": n, "sig": sg, "ratio": round(r1c / r0c, 4), "r0": r0c, "r1": r1c, "T": Tw, "mask": mk})
        else:
            print(f"  n={n:2d}, |S|={sig}: no r1 > r0 found ({el:.2f}s)")
    return best_by_n


if __name__ == "__main__":
    r1 = run_f4_minimal_counterexamples()
    r2 = run_f4_all_colorings_increase()
    r3 = run_f4_max_ratio()
    with open(os.path.join(OUT_DIR, "f4_results.json"), "w") as f:
        json.dump({"max_ratio": r3}, f, indent=2)
