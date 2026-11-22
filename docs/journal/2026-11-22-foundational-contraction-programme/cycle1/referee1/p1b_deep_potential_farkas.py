#!/tmp/solver-venv/bin/python
"""
FCP Cycle 1 — Part 1B: Linearly-Independent Potential Search + Multi-Round Fib_{2m}+$ Trajectories
+ Exact Rational Farkas Certificates (via Z3) + Sparse Candidate Potential Audit.
"""
import json
import random
import sys
import time
from fractions import Fraction

from ortools.linear_solver import pywraplp
import z3

sys.path.insert(0, "/tmp/fcp-ref")
import p1_potential_lp as p1


def run_bdmr_trajectory(L0, p0, policy="thresh_pref", max_rounds=40):
    """Run full multi-round BDMR trajectory and collect every transition (f0, f1, meta)."""
    L, F, LF, C, p, s = p1.bwt_from_L_only(L0, p0)
    f_cur = p1.compute_features(L, F, LF, C, p, s, include_text_feats=True)
    traj = []
    states = [f_cur]
    for rnd in range(max_rounds):
        n = len(L)
        if n <= 2 or s <= 1:
            break
        # Check self-run
        has_self = any(L[i] == F[i] for i in range(n))
        # Candidate moves
        chosen = None
        if policy == "thresh_pref":
            # Variant B: prefer threshold cut if del >= n/4 else self-run else max-cut
            best_t = None
            for cut in range(1, s):
                in_L = [c < cut for c in range(s)]
                dc = sum(1 for l, f in zip(L, F) if in_L[l] and not in_L[f])
                if best_t is None or dc > best_t[0]:
                    best_t = (dc, in_L, "thresh")
                in_L2 = [c >= cut for c in range(s)]
                dc2 = sum(1 for l, f in zip(L, F) if in_L2[l] and not in_L2[f])
                if dc2 > best_t[0]:
                    best_t = (dc2, in_L2, "thresh")
            if best_t and 4 * best_t[0] >= n:
                res = p1.apply_bdmr_pair(L, F, LF, C, p, s, best_t[1])
                if res:
                    chosen = ("thresh_pair", best_t[1], res)
            if chosen is None and has_self:
                res = p1.apply_bdmr_self(L, F, LF, C, p, s)
                if res:
                    chosen = ("self", None, res)
            if chosen is None and best_t and best_t[0] > 0:
                res = p1.apply_bdmr_pair(L, F, LF, C, p, s, best_t[1])
                if res:
                    chosen = ("thresh_pair", best_t[1], res)
        elif policy == "pure_maxcut":
            # Greedy / exact max-cut (exact for s <= 12, greedy 2-opt for s > 12)
            # First self-run if self-del >= n/4, else max-cut pair, else self-run
            self_del = sum(1 for l, f in zip(L, F) if l == f)
            # Build directed weight matrix W[a][b]
            W = [[0] * s for _ in range(s)]
            for l, f in zip(L, F):
                if l != f:
                    W[l][f] += 1
            if s <= 10:
                best_mc = (0, None)
                for mask in range(1, (1 << s) - 1):
                    in_L = [bool((mask >> c) & 1) for c in range(s)]
                    dc = 0
                    for a in range(s):
                        if in_L[a]:
                            for b in range(s):
                                if not in_L[b]:
                                    dc += W[a][b]
                    if dc > best_mc[0]:
                        best_mc = (dc, in_L)
            else:
                # Greedy deterministic max-cut + 1-flip local search
                in_L = [False] * s
                for c in range(s):
                    gain_L = sum(W[c][b] for b in range(c) if not in_L[b])
                    gain_R = sum(W[a][c] for a in range(c) if in_L[a])
                    in_L[c] = gain_L >= gain_R
                improved = True
                while improved:
                    improved = False
                    for c in range(s):
                        cur = sum(W[c][b] for b in range(s) if not in_L[b]) if in_L[c] else sum(W[a][c] for a in range(s) if in_L[a])
                        alt = sum(W[a][c] for a in range(s) if in_L[a]) if in_L[c] else sum(W[c][b] for b in range(s) if not in_L[b])
                        if alt > cur:
                            in_L[c] = not in_L[c]
                            improved = True
                dc = sum(W[a][b] for a in range(s) for b in range(s) if in_L[a] and not in_L[b])
                best_mc = (dc, in_L)

            if self_del > best_mc[0] and has_self:
                res = p1.apply_bdmr_self(L, F, LF, C, p, s)
                if res:
                    chosen = ("self", None, res)
            elif best_mc[0] > 0:
                res = p1.apply_bdmr_pair(L, F, LF, C, p, s, best_mc[1])
                if res:
                    chosen = ("maxcut_pair", best_mc[1], res)
            elif has_self:
                res = p1.apply_bdmr_self(L, F, LF, C, p, s)
                if res:
                    chosen = ("self", None, res)

        if chosen is None:
            break
        kind, in_L, (L1, p1_new, dcnt) = chosen
        if len(L1) <= 1 or len(set(L1)) <= 1:
            break
        L, F, LF, C, p, s = p1.bwt_from_L_only(L1, p1_new)
        f_nxt = p1.compute_features(L, F, LF, C, p, s, include_text_feats=True)
        traj.append((f_cur, f_nxt, (kind, f"rnd{rnd}", dcnt, f_cur["r"], f_nxt["r"], len(L1))))
        states.append(f_nxt)
        f_cur = f_nxt
    return traj, states


def solve_lp_clean(transitions, feat_names, states, allow_const=False, l1_reg=False):
    d = len(feat_names)
    solver = pywraplp.Solver.CreateSolver("GLOP")
    w = [solver.NumVar(-500.0, 500.0, name) for name in feat_names]
    c0 = solver.NumVar(-500.0, 500.0, "c0") if allow_const else None
    M_var = solver.NumVar(0.0, 500.0, "M_ratio")

    for bi, f in enumerate(states):
        expr = solver.Sum(w[j] * f[feat_names[j]] for j in range(d))
        if c0 is not None:
            expr = expr + c0
        solver.Add(expr >= f["r"])
        solver.Add(expr <= M_var * f["r"])

    for f0, f1, _ in transitions:
        diff = [f1[feat_names[j]] - f0[feat_names[j]] for j in range(d)]
        if any(x != 0 for x in diff):
            solver.Add(solver.Sum(w[j] * diff[j] for j in range(d)) <= 0.0)

    if l1_reg:
        abs_w = [solver.NumVar(0.0, 500.0, f"aw_{j}") for j in range(d)]
        for j in range(d):
            solver.Add(abs_w[j] >= w[j])
            solver.Add(abs_w[j] >= -w[j])
        solver.Minimize(M_var + 0.05 * solver.Sum(abs_w))
    else:
        solver.Minimize(M_var)

    st = solver.Solve()
    if st == pywraplp.Solver.OPTIMAL:
        wts = {feat_names[j]: round(w[j].solution_value(), 5) for j in range(d) if abs(w[j].solution_value()) > 1e-5}
        if c0 is not None and abs(c0.solution_value()) > 1e-5:
            wts["const"] = round(c0.solution_value(), 5)
        return "FEASIBLE", M_var.solution_value(), wts
    return "INFEASIBLE", None, None


def exact_farkas_z3(transitions, feat_names, states, allow_const=False):
    """
    Find a minimal sparse Farkas certificate with GLOP first, then verify EXACTLY in Z3 Rationals!
    Primal: exists w (and c0 if allow_const):
      (f1 - f0)^T w <= 0          for all transitions
      -f^T w (- c0) <= -r(f)      for all states
    Dual (Farkas): exists y >= 0, u >= 0:
      sum_k y_k (f1_k - f0_k) == sum_b u_b f_b
      (and sum_b u_b == 0 if allow_const -> impossible since r(b) >= 1 and sum u_b r(b) = 1? Wait: if c0 free, sum u_b == 0 impossible!
       Wait: with c0 free, primal is Phi(b) = w^T f(b) + c0 >= r(b) AND Phi(b) <= M r(b)!
       If we drop M r(b), Phi(b) >= r(b) is ALWAYS feasible by taking w=0, c0=max(r)!!
       AHA!! THAT'S WHY const_1=8.0 MADE IT FEASIBLE EARLIER!!)
    """
    assert not allow_const, "Farkas without UB requires homogeneous (allow_const=False)"
    d = len(feat_names)
    solver = pywraplp.Solver.CreateSolver("GLOP")
    diffs = []
    metas = []
    seen = {}
    for f0, f1, meta in transitions:
        diff = tuple(f1[name] - f0[name] for name in feat_names)
        if all(x == 0 for x in diff):
            continue
        if diff not in seen:
            seen[diff] = len(diffs)
            diffs.append(diff)
            metas.append(meta)
    y = [solver.NumVar(0.0, solver.infinity(), f"y_{k}") for k in range(len(diffs))]
    u = [solver.NumVar(0.0, solver.infinity(), f"u_{b}") for b in range(len(states))]

    for j, name in enumerate(feat_names):
        lhs = solver.Sum(diffs[k][j] * y[k] for k in range(len(diffs)) if diffs[k][j] != 0)
        rhs = solver.Sum(states[b][name] * u[b] for b in range(len(states)) if states[b][name] != 0)
        solver.Add(lhs == rhs)
    solver.Add(solver.Sum(states[b]["r"] * u[b] for b in range(len(states))) == 1.0)
    solver.Minimize(solver.Sum(y) + solver.Sum(u))
    if solver.Solve() != pywraplp.Solver.OPTIMAL:
        return None

    nz_y = [k for k in range(len(diffs)) if y[k].solution_value() > 1e-8]
    nz_u = [b for b in range(len(states)) if u[b].solution_value() > 1e-8]

    # Exact rational verification in Z3 over the sparse support!
    zs = z3.Solver()
    zy = [z3.Real(f"zy_{i}") for i in range(len(nz_y))]
    zu = [z3.Real(f"zu_{i}") for i in range(len(nz_u))]
    for v in zy + zu:
        zs.add(v >= 0)
    for j, name in enumerate(feat_names):
        lhs = z3.Sum([z3.Q(int(round(diffs[nz_y[i]][j] * 1000000)), 1000000) * zy[i] for i in range(len(nz_y))])
        rhs = z3.Sum([z3.Q(int(round(states[nz_u[i]][name] * 1000000)), 1000000) * zu[i] for i in range(len(nz_u))])
        zs.add(lhs == rhs)
    zs.add(z3.Sum([states[nz_u[i]]["r"] * zu[i] for i in range(len(nz_u))]) == 1)
    chk = zs.check()
    exact_ok = (chk == z3.sat)
    exact_y = []
    exact_u = []
    if exact_ok:
        mdl = zs.model()
        for i, k in enumerate(nz_y):
            val = mdl.eval(zy[i]).as_fraction()
            if val > 0:
                exact_y.append((str(val), metas[k]))
        for i, b in enumerate(nz_u):
            val = mdl.eval(zu[i]).as_fraction()
            if val > 0:
                exact_u.append((str(val), states[b]["r"], states[b]["n"]))
    return {"exact_z3_verified": exact_ok, "num_y": len(exact_y), "num_u": len(exact_u), "y": exact_y, "u": exact_u}


def main():
    t0 = time.time()
    # Strictly linearly independent run-scale basis (NO const_1, NO collinear redundancies!)
    # Note: we exclude delta (non-integer) from exact integer basis first, then test with delta too.
    B_core = ["r", "r_circ", "sigma", "r_F", "E_RG", "E_RG_inv", "e_2", "e_2_self", "r_1", "r_F1"]
    B_rich = B_core + ["ref_DCF", "ref_L2", "r_L2", "r_L2L", "r_L2LF", "r_FF2", "r_LFF2", "e_3", "r_des", "rg_E_uniq", "rg_self", "frg_E_uniq", "max_sym_runs", "s_multi"]
    B_global = B_rich + ["r_rev", "delta"]

    # Build corpus: small Lyndon words + multi-round trajectories of Fib_{2m}+$, Thue-Morse, random n=15..60
    words = []
    for n in range(3, 9):
        for w in p1.gen_lyndon(n, 2):
            if len(set(w)) >= 2:
                words.append(w)
    for n in range(3, 7):
        for w in p1.gen_lyndon(n, 3):
            if len(set(w)) == 3:
                words.append(w)
    special = [
        (0, 0, 1, 1, 2), (0, 0, 2, 0, 1), (0, 0, 1, 0, 1), (1, 2, 3, 2, 0),
        (0, 0, 1, 1, 2, 1, 2, 0, 0, 1),
    ]
    # Fibonacci + $ up to Fib_14 + $ (len 377+1=378; Fib_16=987+1=988)
    fa, fb = (1,), (1, 2)
    fib_words = []
    for m in range(3, 15):
        fa, fb = fb, fb + fa
        if m % 2 == 0 or m <= 10:
            fib_words.append((f"Fib_{m}+$", fb + (0,)))
    # Thue-Morse
    tm = (0,)
    for k in range(1, 7):
        tm = tm + tuple(1 - x for x in tm)
        if k >= 3:
            fib_words.append((f"TM_{k}+$", tm + (2,)))
    # Random words n=14..45
    rng = random.Random(2026)
    rand_words = []
    for n in [12, 16, 24, 36]:
        for s in [2, 3, 4]:
            for idx in range(6):
                w = tuple(rng.randrange(s) for _ in range(n))
                if len(set(w)) >= 2 and p1.is_primitive(w):
                    rand_words.append((f"rnd_{n}_{s}_{idx}", w))

    # Collect transitions
    tr_all = []
    tr_pair = []
    tr_self = []
    tr_thresh_self = []
    tr_maxcut_self = []
    tr_maxcut_pair = []
    tr_lazard = []
    all_states = {}

    def add_state(f):
        key = tuple(f[k] for k in B_rich) + (f["n"],)
        if key not in all_states:
            all_states[key] = f

    # 1. Exhaustive 1-step from small Lyndon + special
    seen_w = set()
    for w in words + special:
        if not p1.is_primitive(w):
            continue
        u = sorted(set(w))
        mp = {c: i for i, c in enumerate(u)}
        wc = tuple(mp[c] for c in w)
        canon = min(wc[i:] + wc[:i] for i in range(len(wc)))
        if canon in seen_w:
            continue
        seen_w.add(canon)
        L, F, LF, C, p, s = p1.bwt_of_word(canon)
        f0 = p1.compute_features(L, F, LF, C, p, s, include_text_feats=True)
        add_state(f0)
        res_s = p1.apply_bdmr_self(L, F, LF, C, p, s)
        if res_s and len(res_s[0]) >= 2 and len(set(res_s[0])) >= 2:
            L1, p1n, dc = res_s
            f1 = p1.compute_features(*p1.bwt_from_L_only(L1, p1n), include_text_feats=True)
            add_state(f1)
            meta = ("self", canon, None, dc, f0["r"], f1["r"])
            tr_all.append((f0, f1, meta))
            tr_self.append((f0, f1, meta))
            tr_thresh_self.append((f0, f1, meta))
            tr_maxcut_self.append((f0, f1, meta))
        if s <= 4:
            best_d = -1
            cands = []
            for mask in range(1, (1 << s) - 1):
                in_L = [bool((mask >> c) & 1) for c in range(s)]
                res_p = p1.apply_bdmr_pair(L, F, LF, C, p, s, in_L)
                if res_p and len(res_p[0]) >= 2 and len(set(res_p[0])) >= 2:
                    kappa, e_R, is_laz = p1.classify_coloring(in_L, F, L)
                    cands.append((in_L, res_p, kappa, is_laz))
                    if res_p[2] > best_d:
                        best_d = res_p[2]
            for in_L, (L1, p1n, dc), kappa, is_laz in cands:
                f1 = p1.compute_features(*p1.bwt_from_L_only(L1, p1n), include_text_feats=True)
                add_state(f1)
                meta = ("pair", canon, in_L, dc, f0["r"], f1["r"])
                tr_all.append((f0, f1, meta))
                tr_pair.append((f0, f1, meta))
                if kappa == 1:
                    tr_thresh_self.append((f0, f1, meta))
                if dc == best_d and best_d > 0:
                    tr_maxcut_self.append((f0, f1, meta))
                    tr_maxcut_pair.append((f0, f1, meta))
                if is_laz:
                    tr_lazard.append((f0, f1, meta))

    # 2. Multi-round trajectories on Fib_{2m}+$, TM, and random words!
    max_r_fib_thresh = 0
    max_r_fib_mc = 0
    for name, w in fib_words + rand_words:
        L, F, LF, C, p, s = p1.bwt_of_word(w)
        traj_t, st_t = run_bdmr_trajectory(L, p, policy="thresh_pref")
        for f in st_t:
            add_state(f)
        for f0, f1, meta in traj_t:
            m2 = (meta[0], name, meta[1], meta[2], meta[3], meta[4])
            tr_all.append((f0, f1, m2))
            tr_thresh_self.append((f0, f1, m2))
            if meta[0] == "self":
                tr_self.append((f0, f1, m2))
            else:
                tr_pair.append((f0, f1, m2))
            if name.startswith("Fib_"):
                max_r_fib_thresh = max(max_r_fib_thresh, f1["r"])

        traj_m, st_m = run_bdmr_trajectory(L, p, policy="pure_maxcut")
        for f in st_m:
            add_state(f)
        for f0, f1, meta in traj_m:
            m2 = (meta[0], name, meta[1], meta[2], meta[3], meta[4])
            tr_all.append((f0, f1, m2))
            tr_maxcut_self.append((f0, f1, m2))
            if meta[0] == "self":
                tr_self.append((f0, f1, m2))
            else:
                tr_pair.append((f0, f1, m2))
                tr_maxcut_pair.append((f0, f1, m2))
            if name.startswith("Fib_"):
                max_r_fib_mc = max(max_r_fib_mc, f1["r"])

    states = list(all_states.values())
    print(f"Dataset ready in {time.time()-t0:.2f}s: {len(states)} states | Fib max_r: thresh_pref={max_r_fib_thresh}, pure_maxcut={max_r_fib_mc}")
    classes = [
        ("all_bdmr", tr_all),
        ("pair_all", tr_pair),
        ("self_only", tr_self),
        ("thresh_and_self", tr_thresh_self),
        ("maxcut_and_self", tr_maxcut_self),
        ("maxcut_pair_only", tr_maxcut_pair),
        ("lazard", tr_lazard),
    ]
    for cname, trs in classes:
        print(f"  {cname:18s}: {len(trs)} transitions")

    out = {}
    for bname, basis in [("CORE_10", B_core), ("RICH_24", B_rich), ("GLOBAL_26", B_global)]:
        print(f"\n=== HOMOGENEOUS LP (const=0, Phi(B) >= r(B)) on {bname} ===")
        out[bname] = {}
        for cname, trs in classes:
            st, Mval, wts = solve_lp_clean(trs, basis, states, allow_const=False, l1_reg=True)
            print(f"  {cname:18s} -> {st:10s}" + (f"  max(Phi/r)={Mval:.4f}  wts={wts}" if st == "FEASIBLE" else ""))
            rec = {"status": st, "M": Mval, "weights": wts}
            if st == "INFEASIBLE":
                # Also check with allow_const=True (bounded M_ratio)
                st_c, Mval_c, _ = solve_lp_clean(trs, basis, states, allow_const=True, l1_reg=False)
                print(f"    (with +const allowed: {st_c}" + (f" M={Mval_c:.4f})" if st_c == "FEASIBLE" else ")"))
                rec["with_const"] = st_c
                if bname in ("CORE_10", "RICH_24"):
                    cert = exact_farkas_z3(trs, basis, states, allow_const=False)
                    if cert:
                        print(f"    [EXACT Z3 FARKAS] verified={cert['exact_z3_verified']} | {cert['num_y']} transitions, {cert['num_u']} states:")
                        for coeff, meta in cert["y"][:6]:
                            print(f"      y={coeff}  {meta}")
                        rec["farkas"] = cert
            out[bname][cname] = rec

    with open("/tmp/fcp-ref/p1b_results.json", "w") as fp:
        json.dump(out, fp, indent=2, default=str)
    print(f"\nDone in {time.time()-t0:.2f}s")


if __name__ == "__main__":
    main()
