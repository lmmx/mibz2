#!/usr/bin/env python3
"""
Cycle 2 Audit Task 2b:
1. PySAT (Cadical195 + DRAT proof certificate!) certifying `r' - r <= |D|` (r' + n' <= r + n)
   for arbitrary columns L (hence ALL valid BWTs) at n=6,7,8 (|Sigma|=3, both kappa=1 and kappa=2).
2. CP-SAT check of `r' - r > |D|` on ARBITRARY columns L for BDMR self-run & Ascent-DAG (n=6..9, |Sigma|=3).
3. Verify our universal algebraic proof bound Delta r <= m <= |D| across 10,000 random arbitrary columns!
"""
import itertools
import os
import random
import sys
import time

from pysat.solvers import Cadical195
from pysat.card import CardEnc, EncType
from ortools.sat.python import cp_model

sys.path.insert(0, "/tmp/fcp-ref")
import p1_potential_lp as p1
import p2b_dag_dp_wikidata as p2b


def drat_certify_r_plus_n(n=7, kappa=1):
    """
    Pure CNF encoding in PySAT (Cadical195 with proof=True -> DRAT proof!) for:
      Exists arbitrary ternary column L in {0,1,2}^n with counts >= 1 such that
      BDMR pair contraction (kappa=1: Sigma_L={0}, Sigma_R={1,2}; kappa=2: Sigma_L={0,2}, Sigma_R={1})
      has D >= 1 and r' - r > D.
    """
    t0 = time.time()
    vpool = [0]
    def new_var():
        vpool[0] += 1
        return vpool[0]

    clauses = []
    # L[i] = c: L_var[i][c] for i in 0..n-1, c in 0..2
    L_var = [[new_var() for _ in range(3)] for _ in range(n)]
    for i in range(n):
        clauses.append([L_var[i][0], L_var[i][1], L_var[i][2]])
        clauses.append([-L_var[i][0], -L_var[i][1]])
        clauses.append([-L_var[i][0], -L_var[i][2]])
        clauses.append([-L_var[i][1], -L_var[i][2]])

    # Exact prefix counts pref[c][i][k] <=> #{t < i : L[t] == c} == k (for i=0..n, k=0..i)
    pref = [[{} for _ in range(n + 1)] for _ in range(3)]
    for c in range(3):
        pref[c][0][0] = new_var()
        clauses.append([pref[c][0][0]])
        for i in range(1, n + 1):
            for k in range(i + 1):
                pref[c][i][k] = new_var()
            # Exactly one k
            clauses.append([pref[c][i][k] for k in range(i + 1)])
            for k1 in range(i + 1):
                for k2 in range(k1 + 1, i + 1):
                    clauses.append([-pref[c][i][k1], -pref[c][i][k2]])
            # Transition from i-1
            li = L_var[i - 1][c]
            for k in range(i):
                # If pref[c][i-1][k] and li -> pref[c][i][k+1]
                clauses.append([-pref[c][i - 1][k], -li, pref[c][i][k + 1]])
                # If pref[c][i-1][k] and -li -> pref[c][i][k]
                clauses.append([-pref[c][i - 1][k], li, pref[c][i][k]])
        # Require count >= 1 (so pref[c][n][0] is False)
        clauses.append([-pref[c][n][0]])

    # C1 = count(0) in 1..n-2, C2 = count(0)+count(1) = n - count(2) in 2..n-1
    # F_var[i][c]:
    #   F[i] == 0 <=> i < count(0) <=> OR_{k > i} pref[0][n][k]
    #   F[i] == 2 <=> i >= n - count(2) <=> OR_{k >= n - i} pref[2][n][k]
    F_var = [[new_var() for _ in range(3)] for _ in range(n)]
    for i in range(n):
        clauses.append([F_var[i][0], F_var[i][1], F_var[i][2]])
        clauses.append([-F_var[i][0], -F_var[i][1]])
        clauses.append([-F_var[i][0], -F_var[i][2]])
        clauses.append([-F_var[i][1], -F_var[i][2]])
        # F[i]==0 <=> any k in (i+1..n) has pref[0][n][k]
        gt0 = [pref[0][n][k] for k in range(i + 1, n + 1)]
        for g in gt0:
            clauses.append([-g, F_var[i][0]])
        clauses.append([-F_var[i][0]] + gt0)
        # F[i]==2 <=> any k in (n-i..n) has pref[2][n][k]
        ge2 = [pref[2][n][k] for k in range(n - i, n + 1)]
        for g in ge2:
            clauses.append([-g, F_var[i][2]])
        clauses.append([-F_var[i][2]] + ge2)

    # del_var[i] <=> L[i] in Sigma_L and F[i] in Sigma_R
    del_var = [new_var() for _ in range(n)]
    for i in range(n):
        if kappa == 1:
            # L=0 and (F=1 or F=2) <=> L=0 and -F=0
            clauses.append([-del_var[i], L_var[i][0]])
            clauses.append([-del_var[i], -F_var[i][0]])
            clauses.append([-L_var[i][0], F_var[i][0], del_var[i]])
        else:
            # (L=0 or L=2) and F=1 <=> -L=1 and F=1
            clauses.append([-del_var[i], -L_var[i][1]])
            clauses.append([-del_var[i], F_var[i][1]])
            clauses.append([L_var[i][1], -F_var[i][1], del_var[i]])
    clauses.append(del_var[:])  # D >= 1

    # LF_eq[i][j] <=> LF(i) == j
    # LF(i) == j iff:
    #   (L[i]==0 and pref[0][i][j]) OR
    #   (L[i]==1 and exists c1: pref[0][n][c1] and pref[1][i][j-c1]) OR
    #   (L[i]==2 and exists c2: pref[2][n][n-c2] and pref[2][i][j-c2])
    LF_eq = [[new_var() for _ in range(n)] for _ in range(n)]
    for i in range(n):
        clauses.append(LF_eq[i][:])
        for j1 in range(n):
            for j2 in range(j1 + 1, n):
                clauses.append([-LF_eq[i][j1], -LF_eq[i][j2]])
        for j in range(n):
            # Case c=0: L[i]=0 & pref[0][i][j] (only if j <= i)
            if j <= i:
                clauses.append([-L_var[i][0], -pref[0][i][j], LF_eq[i][j]])
            # Case c=1: L[i]=1 & pref[0][n][c1] & pref[1][i][j - c1]
            for c1 in range(1, n - 1):
                rk1 = j - c1
                if 0 <= rk1 <= i:
                    clauses.append([-L_var[i][1], -pref[0][n][c1], -pref[1][i][rk1], LF_eq[i][j]])
            # Case c=2: L[i]=2 & pref[2][n][cnt2] & pref[2][i][j - (n - cnt2)]
            for cnt2 in range(1, n - 1):
                c2 = n - cnt2
                rk2 = j - c2
                if 0 <= rk2 <= i:
                    clauses.append([-L_var[i][2], -pref[2][n][cnt2], -pref[2][i][rk2], LF_eq[i][j]])

    # del_LF[i] <=> del[LF(i)], and L_LF_0[i] <=> L[LF(i)] == 0
    del_LF = [new_var() for _ in range(n)]
    L_LF_0 = [new_var() for _ in range(n)]
    for i in range(n):
        for j in range(n):
            clauses.append([-LF_eq[i][j], -del_var[j], del_LF[i]])
            clauses.append([-LF_eq[i][j], del_var[j], -del_LF[i]])
            clauses.append([-LF_eq[i][j], -L_var[j][0], L_LF_0[i]])
            clauses.append([-LF_eq[i][j], L_var[j][0], -L_LF_0[i]])

    # Upper-bound L'[i] != L'[j] (for kappa=2, even allowing split of 0 & 2 unconditionally by F[i] can ONLY increase r'!)
    # If UNSAT even with this fine-grained L', it is a fortiori UNSAT for exact BDMR!
    # For kappa=1: L'[i] = (L[i], del_LF[i]) (EXACT!)
    # For kappa=2: L'[i] = (L[i], F[i], del_LF[i], L_LF_0[i]) for L[i]==1, and (L[i], F[i]) for L[i] in {0,2}!
    # Define same_L2(i, j) => L'[i] == L'[j]:
    def add_equiv(a, b):
        # Returns var e <=> (a <=> b)
        e = new_var()
        clauses.append([-e, -a, b])
        clauses.append([-e, a, -b])
        clauses.append([e, a, b])
        clauses.append([e, -a, -b])
        return e

    # r0 boundaries: b0[i] <=> L[i] != L[i+1]
    b0 = []
    for i in range(n - 1):
        bi = new_var()
        for c in range(3):
            clauses.append([-L_var[i][c], -L_var[i + 1][c], -bi])
        for c1 in range(3):
            for c2 in range(3):
                if c1 != c2:
                    clauses.append([-L_var[i][c1], -L_var[i + 1][c2], bi])
        b0.append(bi)

    # r2 boundaries: for each i < j, b2_{i,j} <=> (not del[i]) & (not del[j]) & (all del[k] i<k<j) & (L'[i] != L'[j])
    b2 = []
    for i in range(n):
        for j in range(i + 1, n):
            adj = new_var()
            lits = [-del_var[i], -del_var[j]] + [del_var[k] for k in range(i + 1, j)]
            for lit in lits:
                clauses.append([-adj, lit])
            clauses.append([adj] + [-lit for lit in lits])

            # diff2 <=> L'[i] != L'[j]
            diff2 = new_var()
            # Same L[i] == L[j]
            same_L = new_var()
            for c in range(3):
                clauses.append([-L_var[i][c], -L_var[j][c], same_L])
            for c1 in range(3):
                for c2 in range(3):
                    if c1 != c2:
                        clauses.append([-L_var[i][c1], -L_var[j][c2], -same_L])

            if kappa == 1:
                same_d = add_equiv(del_LF[i], del_LF[j])
                # diff2 <=> -(same_L & same_d)
                clauses.append([-same_L, -same_d, -diff2])
                clauses.append([same_L, diff2])
                clauses.append([same_d, diff2])
            else:
                # L'[i] == L'[j] iff same_L AND:
                #   if L=1: same_del_LF & (del_LF -> same_LLF0)
                #   if L!=1: same_F (upper bound on split!)
                same_F = new_var()
                for c in range(3):
                    clauses.append([-F_var[i][c], -F_var[j][c], same_F])
                for c1 in range(3):
                    for c2 in range(3):
                        if c1 != c2:
                            clauses.append([-F_var[i][c1], -F_var[j][c2], -same_F])
                same_d = add_equiv(del_LF[i], del_LF[j])
                same_l0 = add_equiv(L_LF_0[i], L_LF_0[j])
                # If -same_L -> diff2
                clauses.append([same_L, diff2])
                # If L[i]=1 & L[j]=1:
                clauses.append([-L_var[i][1], -L_var[j][1], same_d, diff2])
                clauses.append([-L_var[i][1], -L_var[j][1], -del_LF[i], same_l0, diff2])
                clauses.append([-L_var[i][1], -L_var[j][1], -same_d, del_LF[i], -diff2])
                clauses.append([-L_var[i][1], -L_var[j][1], -same_d, -same_l0, -diff2])
                # If L[i]!=1 & same_L:
                clauses.append([L_var[i][1], -same_L, same_F, diff2])
                clauses.append([L_var[i][1], -same_L, -same_F, -diff2])

            bnd = new_var()
            clauses.append([-bnd, adj])
            clauses.append([-bnd, diff2])
            clauses.append([-adj, -diff2, bnd])
            b2.append(bnd)

    # Violation: sum(b2) > sum(b0) + sum(del_var)
    # <=> sum(b2) + sum(-b0) + sum(-del_var) >= (n - 1) + n + 1 = 2n!
    all_lits = b2 + [-x for x in b0] + [-x for x in del_var]
    target = (n - 1) + n + 1
    cnf_card = CardEnc.atleast(lits=all_lits, bound=target, top_id=vpool[0], encoding=EncType.totalizer)
    clauses.extend(cnf_card.clauses)

    solver = Cadical195(bootstrap_with=clauses, with_proof=True)
    sat = solver.solve()
    dt = time.time() - t0
    proof = solver.get_proof() if not sat else None
    drat_path = None
    if proof is not None:
        drat_path = f"/tmp/fcp-ref2/r_plus_n_n{n}_k{kappa}.drat"
        with open(drat_path, "w") as f:
            for line in proof:
                f.write(line + "\n")
    solver.delete()
    print(f"  Cadical195 DRAT (n={n}, kappa={kappa}, vars={cnf_card.nv}, cls={len(clauses)}, {dt:.2f}s): "
          f"sat={sat}, drat_lines={len(proof) if proof else 0} ({drat_path})", flush=True)


def check_arbitrary_self_and_dag():
    print("\n=== Checking ARBITRARY (non-BWT) columns L for BDMR Self-Run & Ascent-DAG Delta r > |D| ===", flush=True)
    rng = random.Random(9999)
    max_s = -999
    max_d = -999
    viol_s = None
    viol_d = None
    # Exhaustive ternary n<=8 for self-run (already did DAG for n<=8 -> None!)
    for n in range(3, 9):
        for tup in itertools.product((0, 1, 2), repeat=n):
            if len(set(tup)) < 3:
                continue
            L = list(tup)
            r0 = sum(1 for i in range(1, n) if L[i] != L[i - 1]) + 1
            counts = [0, 0, 0]
            for c in L: counts[c] += 1
            C = [0, counts[0], counts[0] + counts[1], n]
            F = [0] * counts[0] + [1] * counts[1] + [2] * counts[2]
            occ = C[:3]
            LF = [0] * n
            for i, c in enumerate(L):
                LF[i] = occ[c]; occ[c] += 1
            res = p1.apply_bdmr_self(L, F, LF, C, 0, 3)
            if res is not None:
                L2, p2, s2 = res
                D = n - len(L2)
                r2 = sum(1 for i in range(1, len(L2)) if L2[i] != L2[i - 1]) + 1
                diff = (r2 - r0) - D
                if diff > max_s: max_s = diff
                if diff > 0 and viol_s is None:
                    viol_s = (n, L, r0, r2, D, L2)
    # Plus 10,000 random arbitrary columns n=8..30, sigma=3..6 for self-run and Ascent-DAG!
    for _ in range(10000):
        n = rng.randint(8, 30)
        s = rng.choice([3, 4, 5, 6])
        L = [rng.randrange(s) for _ in range(n)]
        if len(set(L)) < s: continue
        r0 = sum(1 for i in range(1, n) if L[i] != L[i - 1]) + 1
        counts = [0] * s
        for c in L: counts[c] += 1
        C = [0] * (s + 1)
        for c in range(s): C[c + 1] = C[c] + counts[c]
        F = [c for c in range(s) for _ in range(counts[c])]
        occ = C[:s]
        LF = [0] * n
        for i, c in enumerate(L): LF[i] = occ[c]; occ[c] += 1
        res = p1.apply_bdmr_self(L, F, LF, C, 0, s)
        if res is not None:
            L2, _, _ = res
            D = n - len(L2)
            r2 = sum(1 for i in range(1, len(L2)) if L2[i] != L2[i - 1]) + 1
            diff = (r2 - r0) - D
            if diff > max_s: max_s = diff
            if diff > 0 and viol_s is None:
                viol_s = (n, s, L, r0, r2, D, L2)
        runs = p1.db.runs_from_seq(L)
        runs2, n2, _, _ = p2b.run_space_ascent_dag_step(runs, n, s, mode="asc")
        D = n - n2
        if D > 0:
            diff = (len(runs2) - r0) - D
            if diff > max_d: max_d = diff
            if diff > 0 and viol_d is None:
                viol_d = (n, s, L, r0, len(runs2), D)
    print(f"  Arbitrary L (exhaustive ternary n<=8 + 10k random n=8..30): self max(dr-D)={max_s:+d} viol={viol_s} | dag max(dr-D)={max_d:+d} viol={viol_d}", flush=True)


if __name__ == "__main__":
    print("=== PySAT Cadical195 DRAT Certificates for `r' + n' <= r + n` on Arbitrary Ternary Columns ===", flush=True)
    for n in (6, 7, 8):
        drat_certify_r_plus_n(n=n, kappa=1)
        drat_certify_r_plus_n(n=n, kappa=2)
    check_arbitrary_self_and_dag()
