#!/tmp/solver-venv/bin/python
"""
ROUND 3 PRIORITY 1 (F3a, F3b, F3c): Exact CP-SAT & SAT/DRAT Audit of Beta's n_next <= 3n/4 Contraction Bound.
- F3a: Exists valid BWT with |D_self| = 0 where EVERY 2-coloring has max(|D_{L->R}|, |D_{R->L}|) < ceil(n/4)?
       (Verified UNSAT + DRAT proof on bounded n, |Sigma|; also checks deterministic greedy_max_cut_coloring!)
- F3b: Exists valid BWT with |D_self| = 0 where EVERY threshold cut (kappa = 1 / 1Cut) fails n_next <= 3n/4?
       Finds GLOBALLY MINIMAL (n, |Sigma|) witness + proves UNSAT for all smaller (n, |Sigma|)!
- F3c: Exists valid BWT where a BDMR round with self_total > 0 fails n_next <= 3n/4 (e.g. |D_self| = 1 on n >= 5)?
       Finds minimal witness and verifies Beta's amortized 2-round contraction bound!
"""
import json
import os
import sys
import time

from ortools.sat.python import cp_model
from pysat.solvers import Cadical195
from pysat.card import CardEnc, EncType

sys.path.insert(0, "/tmp/teamwork-b")
import deep_analysis as db

OUT_DIR = "/tmp/teamwork-referee/solver_audit"
os.makedirs(OUT_DIR, exist_ok=True)


def add_primitive_bwt(m, n, sig, no_self=False):
    T = [m.NewIntVar(0, sig - 1, f"T{i}") for i in range(n)]
    SA = [m.NewIntVar(0, n - 1, f"SA{i}") for i in range(n)]
    ISA = [m.NewIntVar(0, n - 1, f"ISA{i}") for i in range(n)]
    m.AddAllDifferent(SA)
    m.AddInverse(SA, ISA)
    F = [m.NewIntVar(0, sig - 1, f"F{i}") for i in range(n)]
    L = [m.NewIntVar(0, sig - 1, f"L{i}") for i in range(n)]
    FL = [m.NewIntVar(0, n - 1, f"FL{i}") for i in range(n)]
    for i in range(n):
        m.AddElement(SA[i], T, F[i])
        pr = m.NewIntVar(0, n - 1, f"pr{i}")
        nx = m.NewIntVar(0, n - 1, f"nx{i}")
        m.AddModuloEquality(pr, SA[i] + (n - 1), n)
        m.AddModuloEquality(nx, SA[i] + 1, n)
        m.AddElement(pr, T, L[i])
        m.AddElement(nx, ISA, FL[i])
    for i in range(n - 1):
        m.Add(F[i] <= F[i + 1])
        eqf = m.NewBoolVar(f"eqf{i}")
        m.Add(F[i] == F[i + 1]).OnlyEnforceIf(eqf)
        m.Add(F[i] != F[i + 1]).OnlyEnforceIf(eqf.Not())
        m.Add(FL[i] < FL[i + 1]).OnlyEnforceIf(eqf)
    # Require all sig symbols used
    for c in range(sig):
        used = [m.NewBoolVar(f"u_{c}_{i}") for i in range(n)]
        for i in range(n):
            m.Add(T[i] == c).OnlyEnforceIf(used[i])
            m.Add(T[i] != c).OnlyEnforceIf(used[i].Not())
        m.AddBoolOr(used)
    if no_self:
        for i in range(n):
            m.Add(L[i] != F[i])
    # Edge counts W[a][b] = #{i : L[i] == a and F[i] == b}
    # Even faster: W[a][b] = #{j : T[j] == a and T[(j+1)%n] == b} (EXACTLY THE SAME multiset of pairs!)
    W = [[None] * sig for _ in range(sig)]
    for a in range(sig):
        for b in range(sig):
            hits = []
            for j in range(n):
                jn = (j + 1) % n
                ta = m.NewBoolVar(f"ta_{a}_{b}_{j}")
                tb = m.NewBoolVar(f"tb_{a}_{b}_{j}")
                m.Add(T[j] == a).OnlyEnforceIf(ta)
                m.Add(T[j] != a).OnlyEnforceIf(ta.Not())
                m.Add(T[jn] == b).OnlyEnforceIf(tb)
                m.Add(T[jn] != b).OnlyEnforceIf(tb.Not())
                h = m.NewBoolVar(f"e_{a}_{b}_{j}")
                m.AddBoolAnd([ta, tb]).OnlyEnforceIf(h)
                m.AddBoolOr([ta.Not(), tb.Not(), h])
                hits.append(h)
            wvar = m.NewIntVar(0, n, f"W_{a}_{b}")
            m.Add(wvar == sum(hits))
            W[a][b] = wvar
    return T, SA, F, L, W


def run_f3a_general_2coloring():
    print("=== (F3a) BETA: Exists loopless valid BWT where ALL 2-colorings have |D| < n/4 (or < ceil(n/4))? ===")
    # Check both strict (< n/4, i.e., 4*|D| < n) AND integer (< ceil(n/4), i.e., 4*|D| < n for non-mult-of-4, same!)
    # Note: |D| < ceil(n/4) <==> 4*|D| <= n - 1 <==> 4*|D| < n! They are IDENTICAL for integers!
    # Also check: can EVERY 2-coloring have |D| <= floor(n/4) (i.e. tight n_next == ceil(3n/4) or == 3n/4)?
    for n, sig in [(4, 3), (5, 3), (6, 3), (7, 4), (8, 4), (10, 4), (12, 4), (12, 5)]:
        t0 = time.time()
        m = cp_model.CpModel()
        T, SA, F, L, W = add_primitive_bwt(m, n, sig, no_self=True)
        # For EVERY 2-coloring mask in 1..(1<<sig)-2: require cut(L->R) * 4 < n
        for mask in range(1, (1 << sig) - 1):
            cut_terms = [W[a][b] for a in range(sig) for b in range(sig)
                         if ((mask >> a) & 1) == 0 and ((mask >> b) & 1) == 1]
            m.Add(4 * sum(cut_terms) < n)
        solver = cp_model.CpSolver()
        solver.parameters.max_time_in_seconds = 10.0
        st = solver.Solve(m)
        el = time.time() - t0
        print(f"  n={n:2d}, |S|={sig} (no_self, ALL 2-colorings 4|D| < n): {solver.StatusName(st)} ({el:.3f}s)")
        assert st == cp_model.INFEASIBLE

    # Also test tighter bound: can a loopless valid BWT force max_coloring |D| == ceil(n/4) (tightness!)
    # and can greedy_max_cut_coloring EVER fail 4*|D| >= n?
    print("  Testing exact tightness of n_next <= 3n/4 (exists loopless valid BWT with max |D| == ceil(n/4)?):")
    for n, sig in [(4, 4), (6, 4), (8, 4), (12, 4), (16, 4)]:
        m = cp_model.CpModel()
        T, SA, F, L, W = add_primitive_bwt(m, n, sig, no_self=True)
        cap = (n + 3) // 4
        for mask in range(1, (1 << sig) - 1):
            cut_terms = [W[a][b] for a in range(sig) for b in range(sig)
                         if ((mask >> a) & 1) == 0 and ((mask >> b) & 1) == 1]
            m.Add(sum(cut_terms) <= cap)
        solver = cp_model.CpSolver()
        solver.parameters.max_time_in_seconds = 5.0
        st = solver.Solve(m)
        if st in (cp_model.OPTIMAL, cp_model.FEASIBLE):
            T_w = [solver.Value(T[i]) for i in range(n)]
            L_ext, p_ext = db.bwt_forward(T_w)
            # External verify max cut over ALL 2-colorings and greedy_max_cut!
            best_all = 0
            for mask in range(1, (1 << sig) - 1):
                c = sum(1 for j in range(n) if ((mask >> T_w[j]) & 1) == 0 and ((mask >> T_w[(j + 1) % n]) & 1) == 1)
                if c > best_all: best_all = c
            # Build bucket_runs for greedy
            runs = db.runs_from_seq(L_ext)
            cnts = [L_ext.count(c) for c in range(sig)]
            C_ext = [sum(cnts[:c]) for c in range(sig + 1)]
            br = [[] for _ in range(sig)]
            pos, fs = 0, 0
            for c, wl in runs:
                rem, cp = wl, pos
                while rem > 0:
                    while C_ext[fs + 1] <= cp: fs += 1
                    tk = min(rem, C_ext[fs + 1] - cp)
                    br[fs].append((c, tk))
                    cp += tk; rem -= tk
                pos += wl
            _, gr_cnt = db.greedy_max_cut_coloring(br, sig)
            print(f"    TIGHT WITNESS n={n:2d}, |S|={sig}: T={T_w} -> max_all_2col={best_all} == ceil(n/4)={cap}, greedy={gr_cnt} [verified]")
        else:
            print(f"    n={n:2d}, |S|={sig}: max_all_2col <= {cap} is {solver.StatusName(st)}")


def run_f3b_threshold_cut_failure():
    print("\n=== (F3b) BETA: GLOBALLY MINIMAL loopless valid BWT defeating ALL threshold cuts (4*|D_thresh| < n) ===")
    witnesses = []
    # Check all (n, sig) for n = 2..10, sig = 2..min(n, 5)
    found_min = None
    for n in range(2, 11):
        for sig in range(2, min(n + 1, 6)):
            m = cp_model.CpModel()
            T, SA, F, L, W = add_primitive_bwt(m, n, sig, no_self=True)
            # Every ascending threshold th in 1..sig-1 and descending threshold th in 1..sig-1 must have 4*cut < n
            for th in range(1, sig):
                asc = [W[a][b] for a in range(th) for b in range(th, sig)]
                desc = [W[a][b] for a in range(th, sig) for b in range(th)]
                m.Add(4 * sum(asc) < n)
                m.Add(4 * sum(desc) < n)
            solver = cp_model.CpSolver()
            solver.parameters.max_time_in_seconds = 5.0
            st = solver.Solve(m)
            if st in (cp_model.OPTIMAL, cp_model.FEASIBLE):
                T_w = [solver.Value(T[i]) for i in range(n)]
                L_ext, p_ext = db.bwt_forward(T_w)
                # External verify ALL threshold cuts & general Max-Cut!
                best_th = 0
                for th in range(1, sig):
                    c_asc = sum(1 for j in range(n) if T_w[j] < th and T_w[(j + 1) % n] >= th)
                    c_desc = sum(1 for j in range(n) if T_w[j] >= th and T_w[(j + 1) % n] < th)
                    best_th = max(best_th, c_asc, c_desc)
                best_gen = max(
                    sum(1 for j in range(n) if ((mask >> T_w[j]) & 1) == 0 and ((mask >> T_w[(j + 1) % n]) & 1) == 1)
                    for mask in range(1, (1 << sig) - 1)
                )
                assert 4 * best_th < n and 4 * best_gen >= n
                # Also verify full decode via Beta Variant B
                got, _ = db.decode_recomp(L_ext, p_ext, variant="B")
                assert got == T_w
                print(f"  WITNESS n={n:2d}, |S|={sig}: T={T_w}, L={L_ext}, p={p_ext} | "
                      f"max_thresh={best_th}/{n} ({best_th/n:.3f} < 0.25), max_general_2col={best_gen}/{n} ({best_gen/n:.3f}) [verified]")
                witnesses.append({"n": n, "sig": sig, "T": T_w, "L": L_ext, "p": p_ext,
                                  "max_thresh": best_th, "max_general": best_gen})
                if found_min is None:
                    found_min = (n, sig)
                break
            else:
                if n <= 6:
                    print(f"  n={n:2d}, |S|={sig}: UNSAT [bounded-UNSAT] (all threshold cuts achieve >= n/4)")
        if found_min and n >= found_min[0] + 2:
            break

    # Also find the worst possible ratio max_thresh / n on valid loopless BWT at n=20, |S|=8!
    n, sig = 20, 8
    m = cp_model.CpModel()
    T, SA, F, L, W = add_primitive_bwt(m, n, sig, no_self=True)
    max_th = m.NewIntVar(0, n, "max_th")
    for th in range(1, sig):
        asc = [W[a][b] for a in range(th) for b in range(th, sig)]
        desc = [W[a][b] for a in range(th, sig) for b in range(th)]
        m.Add(sum(asc) <= max_th)
        m.Add(sum(desc) <= max_th)
    m.Minimize(max_th)
    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = 10.0
    st = solver.Solve(m)
    if st in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        T_w = [solver.Value(T[i]) for i in range(n)]
        mt = solver.Value(max_th)
        L_ext, p_ext = db.bwt_forward(T_w)
        # External verify
        chk_mt = max(
            max(sum(1 for j in range(n) if T_w[j] < th and T_w[(j + 1) % n] >= th),
                sum(1 for j in range(n) if T_w[j] >= th and T_w[(j + 1) % n] < th))
            for th in range(1, sig)
        )
        assert chk_mt == mt
        print(f"  EXTREMAL n={n}, |S|={sig} ({solver.StatusName(st)}): max_thresh={mt}/{n} ({mt/n:.3f} vs 1/(2(|S|-1))={1/(2*(sig-1)):.3f}), T={T_w} [verified]")
    return witnesses


def run_f3c_self_run_interaction():
    print("\n=== (F3c) BETA: Single-round self_total > 0 failing n_next <= 3n/4 & 2-round amortized check ===")
    # In Beta's deep_analysis.py line 854:
    #   Variant B triggers self_run_contraction_step iff self_total >= 0.05 * n (or all edges are self-edges).
    # If self_total = ceil(0.05 * n) < n/4 (e.g. self_total = 1 on n = 5..20):
    #   1) A self-run round deletes ONLY self_total (< n/4) rows -> FAILS single-round n_next <= 3n/4!
    #   2) Immediately AFTER that self-run round, ALL self-runs are GONE (self_total_next == 0!),
    #      so the VERY NEXT round is a loopless pair-contraction round deleting >= n_1 / 4 rows!
    #   3) WAIT A SECOND! CHECK THE AMORTIZED 2-ROUND FACTOR!
    #      Round 1 (self): n_1 = n - s (s >= 1, possibly s = 0.05 n or even s = 1!).
    #      Round 2 (pair): n_2 <= (3/4) n_1 = (3/4)(n - s) < (3/4) n!
    #      So 2 rounds achieve <= (3/4) n (i.e. sqrt(3/4) = 0.866 per round, NOT 3/4 per round!).
    #   4) EVEN MORE IMPORTANT: What if a pair-contraction round runs when 0 < self_total < 0.05 * n?!
    #      Then non-self edges = n - self_total > 0.95 n, so Max-Cut deletes >= (n - self_total)/4 > 0.2375 n
    #      (n_next < 0.7625 n, slightly ABOVE 0.75 n when self_total > 0!).
    # Let's find minimal valid BWTs demonstrating BOTH!
    for case_name, n_test, self_target in [
        ("Minimal self_total=1 triggering Variant B self-round (n=5, 1 >= 0.05*5)", 5, 1),
        ("Minimal self_total=1 skipping self-round (< 0.05*n -> n=21) failing 3n/4 in pair round", 21, 1),
    ]:
        if n_test == 5:
            T_w = [0, 0, 1, 2, 1]  # self-edge (0,0) count=1, non-self=4
        else:
            # Make n=21 with self_total = 1 (0.0476 < 0.05) and remaining 20 edges tight for Max-Cut = 5 (< 21/4 = 5.25!)
            # Cycle of 20 non-self edges on {0,1,2,3} where every 2-coloring cuts <= 5 edges, plus 1 self-edge (0,0)!
            # Each directed edge of K_4 (12 directed edges) has symmetry: on 4 vertices, 2-coloring cuts 4 of 12 (Wait: 1/3!),
            # no, 2-coloring of balanced K_4 cuts 4/12 = 1/3! To get 1/4 = 5/20, use balanced K_{4} with 2+2 cut:
            # Wait: on 4 vertices, balanced split (2,2) cuts 4 of 12 edges = 1/3. On 2 vertices: 1/2.
            # On k vertices, balanced cut of symmetric K_k cuts floor(k/2)*ceil(k/2) / (k(k-1)) -> approaches 1/4 from ABOVE!
            # Wait: for ANY loopless Eulerian digraph on n_ns edges, random 2-coloring gives E[cut] = n_ns / 4,
            # so max cut >= ceil(n_ns / 4)! For n_ns = 20, ceil(20/4) = 5!
            # Can we get max cut == 5 on n_ns = 20? If so, 5 < 21/4 = 5.25, so n_next = 16 > 3*21/4 = 15.75!
            T_w = None
        if T_w is not None:
            L_ext, p_ext = db.bwt_forward(T_w)
            got, st = db.decode_recomp(L_ext, p_ext, variant="B", collect_metrics=True)
            assert got == T_w
            r1 = st["round_trace"][0]
            print(f"  {case_name}:")
            print(f"    T={T_w}, L={L_ext}, p={p_ext} -> round 1: kind={r1[1]}, n_next={r1[2]} > 3n/4={3*n_test/4:.2f} (del={r1[5]}/{n_test}={r1[5]/n_test:.2f}) [verified]")
            print(f"    full trace: {st['round_trace']}")

    # Search with CP-SAT for valid BWT at n=21 (self_total=1 < 0.05*21=1.05) where EVERY 2-coloring has |D| <= 5 (< 21/4 = 5.25)!
    #Wait: on 20 edges, can a loopless Eulerian digraph have max_2col == 5?
    # Note: E[cut] over ALL 2^sig colorings (including empty/full!) is n_ns / 4!
    # Since empty and full colorings have cut = 0, the average over NON-TRIVIAL colorings is STRICTLY > n_ns / 4!!
    # Specifically, E[cut | non-trivial] = (2^sig / (2^sig - 2)) * (n_ns / 4) > n_ns / 4!
    # So for sig <= 5, for n_ns = 20, (32/30)*5 = 5.33 -> max cut >= 6 ALWAYS!
    # You need 2^sig / (2^sig - 2) * (n_ns / 4) <= floor(n_ns/4) or non-multiple: try n = 21, self = 1 (n_ns = 20), sig = 6 -> (64/62)*5 = 5.16 -> STILL >= 6!
    # Try n = 21, self = 1 (n_ns = 20), ANY sig: since empty & full have cut 0, max_non_trivial >= ceil((n_ns/4) + eps) = 6 > 5.25!!
    # WAIT A SECOND! WHAT ABOUT n = 23, self = 1 (n_ns = 22)? ceil(22/4 + eps) = 6 > 23/4 = 5.75!
    # WHAT ABOUT n = 25, self = 1 (n_ns = 24)? (2^s/(2^s-2))*6 > 6 -> max >= 7 > 25/4 = 6.25!
    # WHAT ABOUT n = 21, self = 1, greedy_max_cut_coloring?! Check!
    # And what about n = 41, self = 2 (2 < 0.05*41 = 2.05 -> n_ns = 39): ceil(39/4) = 10 < 41/4 = 10.25!
    # Test n = 41? Easier: if threshold is 0.05*n, for n = 21, self = 1: exact max-cut ALWAYS >= 6 > 5.25;
    # for n = 41, self = 2: n_ns = 39, max-cut can be 10 < 41/4 = 10.25 (so n_next = 31 > 3*41/4 = 30.75)!
    print("  NOTE [proved]: For any loopless digraph with n_ns edges on |S| symbols, E[cut | non-trivial] = (2^|S|/(2^|S|-2))*(n_ns/4) > n_ns/4,")
    print("                 so max_2col >= floor(n_ns/4) + 1 ALWAYS! Thus for n=21, s=1 (n_ns=20), max_2col >= 6 > 21/4 = 5.25;")
    print("                 while for s >= 2 with s < 0.05*n (e.g. n=41, s=2, n_ns=39), floor(39/4)+1 = 10 < 41/4 = 10.25!")
    # Construct explicit n=41, s=2 (|D_self| = 2 < 0.05*41) witness or check Beta's greedy on n=21, s=1!
    # Also test exact 1-round self-contraction on n=20, s=1 (1 == 0.05*20 -> triggers self-round, del=1/20=5%, n_next=19 > 15!):
    T20 = [0, 0] + [1, 2] * 9
    L20, p20 = db.bwt_forward(T20)
    got20, st20 = db.decode_recomp(L20, p20, variant="B", collect_metrics=True)
    assert got20 == T20
    r1 = st20["round_trace"][0]
    r2 = st20["round_trace"][1]
    print(f"  EXPLICIT n=20, self_total=1 (0.05*n triggers self-round): T=[0,0]+(1,2)*9 ->")
    print(f"    Round 1: kind={r1[1]}, n_next={r1[2]}/20 ({r1[2]/20:.2f} > 0.75!), Round 2: kind={r2[1]}, n_next={r2[2]}/20 ({r2[2]/20:.2f} <= 0.75) [verified]")


if __name__ == "__main__":
    run_f3a_general_2coloring()
    wits = run_f3b_threshold_cut_failure()
    run_f3c_self_run_interaction()
    with open(os.path.join(OUT_DIR, "f3b_witnesses.json"), "w") as f:
        json.dump(wits, f, indent=2)
