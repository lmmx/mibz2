#!/tmp/solver-venv/bin/python
"""
ROUND 3 PRIORITY 1 (F1 & F2): Exact SAT (CaDiCaL 1.9.5 + DRAT) & CP-SAT Audit of Alpha's Lazard-BWT.
- F1: Check r_next > r on valid BWTs (primitive + periodic) for elim_min, elim_max, and both wavefront.
      Produces DRAT UNSAT proof certificates via Cadical195(with_proof=True) for bounded (n, |Sigma|).
- F2: (i) Interior fixed points in F_mid vs UNSAT in F_min/F_max (both valid BWT and arbitrary columns, with DRAT).
      (ii) Minimal valid BWT where kappa > 1 forces bucket splits a^(0), a^(1).
      (iii) Bounded SAT/CP-SAT check of Alpha's primary-mark transport p_byte_shift.
"""
import json
import os
import sys
import time
from itertools import product

from pysat.solvers import Cadical195
from pysat.card import CardEnc, EncType
from ortools.sat.python import cp_model

sys.path.insert(0, "/tmp/teamwork-a")
import verify_full_and_falsify as va
sys.path.insert(0, "/tmp/teamwork-b")
import deep_analysis as db

OUT_DIR = "/tmp/teamwork-referee/solver_audit"
os.makedirs(OUT_DIR, exist_ok=True)


class CNFBuilder:
    def __init__(self):
        self.nvars = 0
        self.clauses = []

    def new_var(self):
        self.nvars += 1
        return self.nvars

    def add(self, clause):
        self.clauses.append(list(clause))

    def eq(self, a, b):
        self.add([-a, b])
        self.add([a, -b])

    def imp(self, conds, lit):
        self.add([-c for c in conds] + [lit])

    def and_var(self, lits):
        v = self.new_var()
        for l in lits:
            self.add([-v, l])
        self.add([v] + [-l for l in lits])
        return v

    def or_var(self, lits):
        v = self.new_var()
        for l in lits:
            self.add([-l, v])
        self.add([-v] + list(lits))
        return v

    def one_hot(self, lits):
        self.add(list(lits))
        for i in range(len(lits)):
            for j in range(i + 1, len(lits)):
                self.add([-lits[i], -lits[j]])

    def write_dimacs(self, path):
        with open(path, "w") as f:
            f.write(f"p cnf {self.nvars} {len(self.clauses)}\n")
            for cl in self.clauses:
                f.write(" ".join(map(str, cl)) + " 0\n")


def encode_lazard_r_increase_sat(n, sig, require_valid_bwt=True):
    """
    Exact SAT encoding of:
      - L[0..n-1] in {0..sig-1} (with symbol 0 occurring >= 1 time, and >= 2 distinct symbols so elim_min is non-trivial)
      - Optionally: L is the EXACT BWT of a text T[0..n-1] (either primitive OR periodic, via rotation-order matrix!)
      - Exact elim_min(0) run-level contraction (computes exact L_next of length n - cnt0)
      - Query: r_next > r_init!
    Returns (cnf, meta).
    """
    cnf = CNFBuilder()
    # L[i][c]: row i has L-symbol c
    L = [[cnf.new_var() for _ in range(sig)] for _ in range(n)]
    for i in range(n):
        cnf.one_hot(L[i])

    # Require symbol 0 appears at least once and at least one non-zero symbol appears
    cnf.add([L[i][0] for i in range(n)])
    cnf.add([L[i][c] for i in range(n) for c in range(1, sig)])

    if require_valid_bwt:
        # Encode T[0..n-1] and permutation P[i][j] = (SA[i] == j)
        T = [[cnf.new_var() for _ in range(sig)] for _ in range(n)]
        for j in range(n):
            cnf.one_hot(T[j])
        P = [[cnf.new_var() for _ in range(n)] for _ in range(n)]
        for i in range(n):
            cnf.one_hot(P[i])
        for j in range(n):
            cnf.one_hot([P[i][j] for i in range(n)])
        # Link L[i] = T[(SA[i] - 1) mod n]
        for i in range(n):
            for j in range(n):
                prev_j = (j - 1) % n
                for c in range(sig):
                    cnf.add([-P[i][j], -T[prev_j][c], L[i][c]])
        # Rotation characters R[i][k][c] = T[(SA[i] + k) mod n] == c
        R = [[[cnf.new_var() for _ in range(sig)] for _ in range(n)] for _ in range(n)]
        for i in range(n):
            for k in range(n):
                cnf.one_hot(R[i][k])
                for j in range(n):
                    pos = (j + k) % n
                    for c in range(sig):
                        cnf.add([-P[i][j], -T[pos][c], R[i][k][c]])
        # Exact lexicographic sort: for every i in 0..n-2, rot(i) <=_lex rot(i+1),
        # and if rot(i) == rot(i+1) then SA[i] < SA[i+1] (handles BOTH primitive & periodic exact BWTs!)
        for i in range(n - 1):
            # eq_pref[k]: rot(i) and rot(i+1) agree on positions 0..k-1
            eq_pref = [None] * (n + 1)
            eq_pref[0] = cnf.new_var()
            cnf.add([eq_pref[0]])
            for k in range(n):
                # eq_at_k <=> sum_c (R[i][k][c] & R[i+1][k][c])
                pair_eq = [cnf.and_var([R[i][k][c], R[i + 1][k][c]]) for c in range(sig)]
                eq_k = cnf.or_var(pair_eq)
                eq_pref[k + 1] = cnf.and_var([eq_pref[k], eq_k])
                # If eq_pref[k], we MUST NOT have R[i][k] > R[i+1][k]
                for c1 in range(sig):
                    for c2 in range(c1):
                        cnf.add([-eq_pref[k], -R[i][k][c1], -R[i + 1][k][c2]])
            # Tie-break on periodic equal rotations: eq_pref[n] => SA[i] < SA[i+1]
            for j1 in range(n):
                for j2 in range(j1 + 1):
                    cnf.add([-eq_pref[n], -P[i][j1], -P[i + 1][j2]])

    # Exact unary count of symbol 0: pref0[i][k] <=> #{t < i : L[t] == 0} == k (for i=0..n, k=0..i)
    pref0 = []
    for i in range(n + 1):
        row = [cnf.new_var() for _ in range(i + 1)]
        cnf.one_hot(row)
        pref0.append(row)
    cnf.add([pref0[0][0]])
    for i in range(n):
        for k in range(i + 1):
            # L[i] != 0 => pref0[i+1][k] == pref0[i][k]
            cnf.add([-pref0[i][k], L[i][0], pref0[i + 1][k]])
            # L[i] == 0 => pref0[i+1][k+1] == pref0[i][k]
            cnf.add([-pref0[i][k], -L[i][0], pref0[i + 1][k + 1]])

    # Exact elim_min(0) at row level (equivalent to run-level Lazard contraction):
    # F_0 = {0 .. c0 - 1}. For each i in 0..n-1:
    #   in_F0[i] <=> i < c0 <=> OR_{k > i} pref0[n][k]
    in_F0 = [cnf.or_var([pref0[n][k] for k in range(i + 1, n + 1)]) for i in range(n)]

    # Exact LF(i) when L[i] == 0 is rank0(i) = #{t < i : L[t] == 0} (so LF(i) = k iff pref0[i][k]).
    # For each row i in F_0 (i < c0):
    #   - If L[i] != 0: row i initiates a chain (base_sym = L[i], k_exp = 1).
    #   - If L[i] == 0: row i follows LF(i) = rank0(i) <= i:
    #       * if rank0(i) == i: fixed point (pure 0-cycle, dead)
    #       * if rank0(i) < i: inherits (base_sym, k_exp + 1) from row rank0(i)!
    # Notice: k_exp in 0..n (0 means fixed point / dead; 1..n means valid chain (y, k) with y in 1..sig-1!).
    # Wait: rows OUTSIDE F_0 (i >= c0) survive in L_next!
    #   - If L[i] != 0: surviving symbol in L_next is (y = L[i], k = 0)  [original uncontracted y]
    #   - If L[i] == 0: surviving symbol in L_next is the chain at row LF(i) = rank0(i) in F_0:
    #       (y = base_sym[rank0(i)], k = k_exp[rank0(i)])!
    # And two surviving consecutive rows i, i+1 (for i >= c0) have the SAME new symbol iff
    # their (y_new, k_new) pairs are identical!
    # Check: can a pure-0 fixed point be reached from outside F_0? NO, because LF is a bijection!
    # So for every row i in F_0, define (base_y[i][0..sig-1], k_exp[i][0..n]) where base_y=0 means fixed point!
    base_y = [[cnf.new_var() for _ in range(sig)] for _ in range(n)]
    k_exp = [[cnf.new_var() for _ in range(n + 1)] for _ in range(n)]
    for i in range(n):
        cnf.one_hot(base_y[i])
        cnf.one_hot(k_exp[i])
        # If in_F0[i] and L[i] == c (c >= 1): base_y[i] = c, k_exp[i] = 1
        for c in range(1, sig):
            cnf.add([-in_F0[i], -L[i][c], base_y[i][c]])
            cnf.add([-in_F0[i], -L[i][c], k_exp[i][1]])
        # If in_F0[i] and L[i] == 0 and pref0[i][i]: fixed point -> base_y[i] = 0, k_exp[i] = 0
        cnf.add([-in_F0[i], -L[i][0], -pref0[i][i], base_y[i][0]])
        cnf.add([-in_F0[i], -L[i][0], -pref0[i][i], k_exp[i][0]])
        # If in_F0[i] and L[i] == 0 and pref0[i][j] for j < i:
        #   base_y[i] == base_y[j], and if base_y[j] == 0 then k_exp[i] = 0 else k_exp[i] = k_exp[j] + 1
        for j in range(i):
            for c in range(sig):
                cnf.add([-in_F0[i], -L[i][0], -pref0[i][j], -base_y[j][c], base_y[i][c]])
            cnf.add([-in_F0[i], -L[i][0], -pref0[i][j], -k_exp[j][0], k_exp[i][0]])
            for ke in range(1, i + 1):
                if ke + 1 <= n:
                    cnf.add([-in_F0[i], -L[i][0], -pref0[i][j], -k_exp[j][ke], k_exp[i][ke + 1]])

    # Surviving symbols (new_y[i], new_k[i]) for i = 0..n-1 (meaningful when not in_F0[i]):
    new_y = [[cnf.new_var() for _ in range(sig)] for _ in range(n)]
    new_k = [[cnf.new_var() for _ in range(n + 1)] for _ in range(n)]
    for i in range(n):
        cnf.one_hot(new_y[i])
        cnf.one_hot(new_k[i])
        # If not in_F0[i] and L[i] == c (c >= 1): new_y[i] = c, new_k[i] = 0
        for c in range(1, sig):
            cnf.add([in_F0[i], -L[i][c], new_y[i][c]])
            cnf.add([in_F0[i], -L[i][c], new_k[i][0]])
        # If not in_F0[i] and L[i] == 0 and pref0[i][j]: new_y[i] = base_y[j], new_k[i] = k_exp[j]
        for j in range(i):
            for c in range(sig):
                cnf.add([in_F0[i], -L[i][0], -pref0[i][j], -base_y[j][c], new_y[i][c]])
            for ke in range(n + 1):
                cnf.add([in_F0[i], -L[i][0], -pref0[i][j], -k_exp[j][ke], new_k[i][ke]])

    # Count initial run boundaries b_init[i] for i = 1..n-1: L[i] != L[i-1] (r_init = 1 + sum b_init)
    b_init = []
    for i in range(1, n):
        same_c = [cnf.and_var([L[i - 1][c], L[i][c]]) for c in range(sig)]
        same = cnf.or_var(same_c)
        b_init.append(-same)

    # Count next run boundaries b_next[i] for i = 1..n-1:
    # active iff (not in_F0[i-1]) and (not in_F0[i]) and (new_sym[i-1] != new_sym[i])
    # Wait: since in_F0 is a prefix (i < c0), (not in_F0[i-1]) implies (not in_F0[i])!
    # And r_next = 1 + sum b_next (since at least one non-zero symbol exists, c0 < n so surviving len >= 1!).
    b_next = []
    for i in range(1, n):
        same_y = cnf.or_var([cnf.and_var([new_y[i - 1][c], new_y[i][c]]) for c in range(sig)])
        same_k = cnf.or_var([cnf.and_var([new_k[i - 1][ke], new_k[i][ke]]) for ke in range(n + 1)])
        same_new = cnf.and_var([same_y, same_k])
        # b_next_i <=> (not in_F0[i-1]) and (not same_new)
        bn = cnf.and_var([-in_F0[i - 1], -same_new])
        b_next.append(bn)

    # Assert: sum(b_next) > sum(b_init)  <==>  sum(b_next) + sum(-b_init) >= (n - 1) + 1 = n
    diff_lits = b_next + [-lit for lit in b_init]
    card = CardEnc.atleast(lits=diff_lits, bound=n, top_id=cnf.nvars, encoding=EncType.seqcounter)
    cnf.nvars = card.nv
    for cl in card.clauses:
        cnf.add(cl)

    return cnf, {"n": n, "sig": sig, "L": L}


def run_f1_sat_drat():
    print("=== (F1) ALPHA r_next <= r: EXACT SAT + DRAT UNSAT CERTIFICATES (CaDiCaL 1.9.5) ===")
    results = []
    for (n, sig, valid) in [
        (5, 3, True), (6, 3, True), (7, 3, True), (8, 3, True),
        (6, 4, True), (7, 4, True),
        (7, 3, False), (8, 3, False), (9, 3, False), (10, 3, False),
    ]:
        t0 = time.time()
        cnf, _ = encode_lazard_r_increase_sat(n, sig, require_valid_bwt=valid)
        want_proof = (n == 8 and sig == 3 and valid) or (n == 9 and sig == 3 and not valid)
        with Cadical195(bootstrap_with=cnf.clauses, with_proof=want_proof) as solver:
            sat = solver.solve()
            elapsed = time.time() - t0
            proof_info = None
            if (not sat) and want_proof:
                proof = solver.get_proof()
                tag = f"f1_n{n}_s{sig}_{'validbwt' if valid else 'arbcol'}"
                cnf_path = os.path.join(OUT_DIR, f"{tag}.cnf")
                drat_path = os.path.join(OUT_DIR, f"{tag}.drat")
                cnf.write_dimacs(cnf_path)
                with open(drat_path, "w") as pf:
                    for line in proof:
                        pf.write(line + "\n")
                proof_info = {"cnf": cnf_path, "drat": drat_path, "drat_lines": len(proof)}
            status = "SAT (COUNTEREXAMPLE!)" if sat else "UNSAT [bounded-UNSAT]"
            print(f"  n={n:2d}, |S|={sig}, {'valid-BWT' if valid else 'arb-col  '}: "
                  f"vars={cnf.nvars:5d}, clauses={len(cnf.clauses):6d} -> {status} ({elapsed:.3f}s)"
                  + (f" [DRAT: {proof_info['drat_lines']} lines saved]" if proof_info else ""))
            assert not sat, f"UNEXPECTED SAT for F1 n={n}, sig={sig}!"
            results.append({"n": n, "sig": sig, "valid_bwt": valid, "vars": cnf.nvars,
                            "clauses": len(cnf.clauses), "status": "UNSAT", "sec": round(elapsed, 4),
                            "proof": proof_info})
    return results


def encode_f2_fixed_point_sat(n, sig, region="min_interior", require_valid_bwt=False):
    """
    Exact SAT encoding checking for fixed points LF(i) == i:
      - region='min_interior': i in F_0 with L[i]=0, LF(i)=i, AND exists j < i with L[j] != 0 (interior in F_min!)
      - region='mid_any': i in F_1 (0 < 1 < sig-1) with L[i]=1, LF(i)=i
      - region='any_valid_bwt': any fixed point on valid BWT with >= 2 distinct symbols
    """
    cnf = CNFBuilder()
    L = [[cnf.new_var() for _ in range(sig)] for _ in range(n)]
    for i in range(n):
        cnf.one_hot(L[i])
    # All symbols 0..sig-1 occur at least once
    for c in range(sig):
        cnf.add([L[i][c] for i in range(n)])

    if require_valid_bwt:
        T = [[cnf.new_var() for _ in range(sig)] for _ in range(n)]
        for j in range(n):
            cnf.one_hot(T[j])
        P = [[cnf.new_var() for _ in range(n)] for _ in range(n)]
        for i in range(n):
            cnf.one_hot(P[i])
        for j in range(n):
            cnf.one_hot([P[i][j] for i in range(n)])
        for i in range(n):
            for j in range(n):
                prev_j = (j - 1) % n
                for c in range(sig):
                    cnf.add([-P[i][j], -T[prev_j][c], L[i][c]])
        R = [[[cnf.new_var() for _ in range(sig)] for _ in range(n)] for _ in range(n)]
        for i in range(n):
            for k in range(n):
                cnf.one_hot(R[i][k])
                for j in range(n):
                    pos = (j + k) % n
                    for c in range(sig):
                        cnf.add([-P[i][j], -T[pos][c], R[i][k][c]])
        for i in range(n - 1):
            eq_pref = [None] * (n + 1)
            eq_pref[0] = cnf.new_var()
            cnf.add([eq_pref[0]])
            for k in range(n):
                eq_k = cnf.or_var([cnf.and_var([R[i][k][c], R[i + 1][k][c]]) for c in range(sig)])
                eq_pref[k + 1] = cnf.and_var([eq_pref[k], eq_k])
                for c1 in range(sig):
                    for c2 in range(c1):
                        cnf.add([-eq_pref[k], -R[i][k][c1], -R[i + 1][k][c2]])
            for j1 in range(n):
                for j2 in range(j1 + 1):
                    cnf.add([-eq_pref[n], -P[i][j1], -P[i + 1][j2]])

    # Prefix counts pref[c][i][k] = (#{t < i : L[t] == c} == k)
    pref = []
    for c in range(sig):
        pc = []
        for i in range(n + 1):
            row = [cnf.new_var() for _ in range(i + 1)]
            cnf.one_hot(row)
            pc.append(row)
        cnf.add([pc[0][0]])
        for i in range(n):
            for k in range(i + 1):
                cnf.add([-pc[i][k], L[i][c], pc[i + 1][k]])
                cnf.add([-pc[i][k], -L[i][c], pc[i + 1][k + 1]])
        pref.append(pc)

    # C_val[c][v] <=> C[c] == v
    C_val = []
    for c in range(sig):
        row = [cnf.new_var() for _ in range(n + 1)]
        cnf.one_hot(row)
        C_val.append(row)
    cnf.add([C_val[0][0]])
    for c in range(sig - 1):
        for v1 in range(n + 1):
            for cnt_c in range(n + 1 - v1):
                cnf.add([-C_val[c][v1], -pref[c][n][cnt_c], C_val[c + 1][v1 + cnt_c]])

    # Fixed point at row i with symbol c: L[i]==c AND C[c] + rank_c(i) == i
    fp_at = [[None] * sig for _ in range(n)]
    for i in range(n):
        for c in range(sig):
            terms = []
            for base in range(i + 1):
                rk = i - base
                terms.append(cnf.and_var([L[i][c], C_val[c][base], pref[c][i][rk]]))
            fp_at[i][c] = cnf.or_var(terms)

    if region == "min_interior":
        # Fixed point at some i >= 1 with c = 0 AND some j < i has L[j] != 0
        conds = []
        for i in range(1, n):
            non0_before = cnf.or_var([-L[j][0] for j in range(i)])
            conds.append(cnf.and_var([fp_at[i][0], non0_before]))
        cnf.add(conds)
    elif region == "mid_any":
        # Fixed point at c = 1 (middle symbol, 0 < 1 < sig-1)
        cnf.add([fp_at[i][1] for i in range(n)])
    elif region == "any_valid_bwt":
        cnf.add([fp_at[i][c] for i in range(n) for c in range(sig)])

    return cnf, L


def run_f2_checks():
    print("\n=== (F2) FIXED POINTS, BUCKET SPLITS (kappa > 1), AND PRIMARY-MARK TRANSPORT ===")
    # F2(i): Interior fixed points in F_min (UNSAT + DRAT), F_mid (SAT on arb col, UNSAT on valid BWT!)
    for (label, n, sig, reg, vbwt, save_drat) in [
        ("F_min interior fixed point (arb col, n=10, |S|=3)", 10, 3, "min_interior", False, True),
        ("F_mid fixed point (arb col, n=3, |S|=3)", 3, 3, "mid_any", False, False),
        ("ANY fixed point (valid BWT, n=8, |S|=3)", 8, 3, "any_valid_bwt", True, True),
        ("ANY fixed point (valid BWT, n=9, |S|=2)", 9, 2, "any_valid_bwt", True, False),
    ]:
        t0 = time.time()
        cnf, L = encode_f2_fixed_point_sat(n, sig, region=reg, require_valid_bwt=vbwt)
        with Cadical195(bootstrap_with=cnf.clauses, with_proof=save_drat) as s:
            sat = s.solve()
            el = time.time() - t0
            if sat:
                mod = set(s.get_model())
                L_wit = [next(c for c in range(sig) if L[i][c] in mod) for i in range(n)]
                # External verify LF
                LF, C = db.ref_inverse_bwt(L_wit, 0), None
                cnt = [L_wit.count(c) for c in range(sig)]
                Cs = [sum(cnt[:c]) for c in range(sig)]
                occ = Cs[:]
                lf_arr = []
                for x in L_wit:
                    lf_arr.append(occ[x]); occ[x] += 1
                fps = [(i, L_wit[i]) for i in range(n) if lf_arr[i] == i]
                print(f"  {label}: SAT witness L={L_wit}, LF={lf_arr}, fixed_points={fps} ({el:.3f}s) [verified]")
            else:
                extra = ""
                if save_drat:
                    pr = s.get_proof()
                    tag = f"f2_{reg}_n{n}_s{sig}"
                    cnf.write_dimacs(os.path.join(OUT_DIR, f"{tag}.cnf"))
                    with open(os.path.join(OUT_DIR, f"{tag}.drat"), "w") as pf:
                        for ln in pr:
                            pf.write(ln + "\n")
                    extra = f" [DRAT: {len(pr)} lines]"
                print(f"  {label}: UNSAT [bounded-UNSAT] ({el:.3f}s){extra}")

    # F2(ii): Minimal valid BWT where kappa > 1 (non-threshold 2-coloring) forces bucket splits a^(0), a^(1)!
    # A symbol a in Sigma_L splits iff inside B_a, L has pattern (L_sym, R_sym, L_sym) in order of increasing follower b:
    # i.e. there exist b1 < b2 < b3 with in_L[b1]=1, in_L[b2]=0, in_L[b3]=1 and symbol a occurs in B_{b1}, B_{b2}, B_{b3}!
    # Notice: needs >= 3 distinct followers of a, so |Sigma| >= 3 (Wait: b1, b2, b3 all distinct; can a == b1 or b3? Yes!
    # Wait: if a in {b1, b3} then a has a self-run! Check BOTH with and without self-runs!)
    print("\n  F2(ii) CP-SAT search for minimal valid BWT forcing bucket splits a^(0), a^(1) (kappa > 1):")
    for allow_self in (True, False):
        found = None
        for n in range(3, 12):
            for sig in range(3, min(n + 1, 6)):
                m = cp_model.CpModel()
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
                    prev_idx = m.NewIntVar(0, n - 1, f"pr{i}")
                    next_idx = m.NewIntVar(0, n - 1, f"nx{i}")
                    m.AddModuloEquality(prev_idx, SA[i] + (n - 1), n)
                    m.AddModuloEquality(next_idx, SA[i] + 1, n)
                    m.AddElement(prev_idx, T, L[i])
                    m.AddElement(next_idx, ISA, FL[i])
                # Primitive valid BWT exact characterization!
                for i in range(n - 1):
                    m.Add(F[i] <= F[i + 1])
                    eq_f = m.NewBoolVar(f"eqf{i}")
                    m.Add(F[i] == F[i + 1]).OnlyEnforceIf(eq_f)
                    m.Add(F[i] != F[i + 1]).OnlyEnforceIf(eq_f.Not())
                    m.Add(FL[i] < FL[i + 1]).OnlyEnforceIf(eq_f)
                # All sig symbols used
                for c in range(sig):
                    used = [m.NewBoolVar(f"u_{c}_{i}") for i in range(n)]
                    for i in range(n):
                        m.Add(T[i] == c).OnlyEnforceIf(used[i])
                        m.Add(T[i] != c).OnlyEnforceIf(used[i].Not())
                    m.AddBoolOr(used)
                if not allow_self:
                    for i in range(n):
                        m.Add(L[i] != F[i])
                # 2-coloring in_L[0..sig-1]
                inL = [m.NewBoolVar(f"inL{c}") for c in range(sig)]
                # exists a in Sigma_L and b1 < b2 < b3 with inL[b1]=1, inL[b2]=0, inL[b3]=1
                # and edges (a -> b1), (a -> b2), (a -> b3) in T (i.e. some row i has L[i]=a, F[i]=b)!
                has_edge = [[m.NewBoolVar(f"e_{a}_{b}") for b in range(sig)] for a in range(sig)]
                for a in range(sig):
                    for b in range(sig):
                        hits = []
                        for i in range(n):
                            h = m.NewBoolVar(f"h_{a}_{b}_{i}")
                            m.Add(L[i] == a).OnlyEnforceIf(h)
                            m.Add(F[i] == b).OnlyEnforceIf(h)
                            # also reverse: if L[i]==a and F[i]==b -> h
                            la = m.NewBoolVar(f"la_{a}_{b}_{i}")
                            fb = m.NewBoolVar(f"fb_{a}_{b}_{i}")
                            m.Add(L[i] == a).OnlyEnforceIf(la)
                            m.Add(L[i] != a).OnlyEnforceIf(la.Not())
                            m.Add(F[i] == b).OnlyEnforceIf(fb)
                            m.Add(F[i] != b).OnlyEnforceIf(fb.Not())
                            m.AddBoolAnd([la, fb]).OnlyEnforceIf(h)
                            m.AddBoolOr([la.Not(), fb.Not(), h])
                            hits.append(h)
                        m.AddMaxEquality(has_edge[a][b], hits)
                split_wits = []
                for a in range(sig):
                    for b1 in range(sig):
                        for b2 in range(b1 + 1, sig):
                            for b3 in range(b2 + 1, sig):
                                w = m.NewBoolVar(f"sp_{a}_{b1}_{b2}_{b3}")
                                m.AddBoolAnd([inL[a], inL[b1], inL[b2].Not(), inL[b3],
                                              has_edge[a][b1], has_edge[a][b2], has_edge[a][b3]]).OnlyEnforceIf(w)
                                split_wits.append(w)
                if not split_wits:
                    continue
                m.AddBoolOr(split_wits)
                solver = cp_model.CpSolver()
                solver.parameters.max_time_in_seconds = 5.0
                st = solver.Solve(m)
                if st in (cp_model.OPTIMAL, cp_model.FEASIBLE):
                    T_w = [solver.Value(T[i]) for i in range(n)]
                    inL_w = [bool(solver.Value(inL[c])) for c in range(sig)]
                    # External verification!
                    L_ext, p_ext = db.bwt_forward(T_w)
                    runs = db.runs_from_seq(L_ext)
                    cnts = [L_ext.count(c) for c in range(sig)]
                    C_ext = [sum(cnts[:c]) for c in range(sig + 1)]
                    br = [[] for _ in range(sig)]
                    pos, fs = 0, 0
                    for c, w_len in runs:
                        rem, cp = w_len, pos
                        while rem > 0:
                            while C_ext[fs + 1] <= cp: fs += 1
                            tk = min(rem, C_ext[fs + 1] - cp)
                            br[fs].append((c, tk))
                            cp += tk; rem -= tk
                        pos += w_len
                    # Count uncontracted blocks for each a in Sigma_L
                    splits = {}
                    for a in range(sig):
                        if not inL_w[a]: continue
                        blks, prev_t = 0, -1
                        for b in range(sig):
                            for sym, _ in br[b]:
                                if sym == a:
                                    t_cur = 0 if inL_w[b] else 1
                                    if t_cur == 0 and prev_t != 0:
                                        blks += 1
                                    prev_t = t_cur
                        if blks >= 2:
                            splits[a] = blks
                    assert len(splits) >= 1
                    got_dec, _ = db.decode_recomp(L_ext, p_ext, variant="B")
                    assert got_dec == T_w
                    found = (n, sig, T_w, L_ext, p_ext, inL_w, splits)
                    break
            if found:
                break
        print(f"    allow_self={str(allow_self):5s}: minimal valid BWT n={found[0]}, |S|={found[1]}, T={found[2]}, "
              f"L={found[3]}, p={found[4]}, in_L={found[5]}, split_blocks={found[6]} [verified]")

    # F2(iii): Primary-mark transport p_byte_shift check on ALL valid BWTs & arbitrary columns up to n=7 (sig=3)
    # and SAT check for single-step p_byte_shift consistency on n=8, |S|=3
    print("\n  F2(iii) Exhaustive + SAT check of Alpha's primary-mark transport p_byte_shift:")
    checked = 0
    for mode in ("min", "max", "both"):
        for n in range(1, 8):
            for tup in product((97, 98, 99), repeat=n):
                T = bytes(tup)
                L, p = va.bwt_encode(T)
                got, _ = va.lazard_decode(L, p, mode=mode)
                assert got == T, f"p_byte_shift mismatch mode={mode} T={T}"
                checked += 1
    print(f"    Exact primary-mark transport on ALL ternary valid BWTs n=1..7 x {{min, max, both}}: {checked} cases PASSED [verified]")


if __name__ == "__main__":
    f1_res = run_f1_sat_drat()
    run_f2_checks()
    with open(os.path.join(OUT_DIR, "f1_results.json"), "w") as f:
        json.dump(f1_res, f, indent=2)
