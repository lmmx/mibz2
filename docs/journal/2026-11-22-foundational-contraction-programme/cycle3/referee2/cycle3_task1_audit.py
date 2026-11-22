#!/usr/bin/env python3
"""
Cycle 3 Task 1:
  1.1 Scrutinize Beta's Theorem B2 (`r(L') - r(L) <= #interior_blocks(D) <= |D|`) via:
      - CP-SAT on ARBITRARY (non-BWT) columns L (n=5..9, |Sigma|=3) for:
        (a) BDMR self-run (`Delta r > #interior_blocks(D)` and `Delta r > |D|`)
        (b) Ascent-DAG (`Delta r > #interior_blocks(D)` and `Delta r > |D|`)
        (c) ALL acyclic (L,F)-defined subsets E subset Sigma x Sigma (exhaustive n=4..7, |Sigma|=3)
      - Explicit valid-BWT counterexample showing NON-(L,F)-defined acyclic |D|=1 gives Delta r = +2 > |D|,
        proving the (L,F)-defined hypothesis is BOTH sufficient AND necessary!
  1.2 Adversarial hunt for worst-case `Delta q = +1` accumulation in Beta's `IET-BSE-AHT`
      (`solve_bucket_aht_spiral`) AND Gamma's `Sub-Interval 1-Piece Schur` (`euclid_accelerate_2iet_bucket`):
      - Evolutionary / hill-climbing search on d-IETs (d = 3, 4, 5, 6, 8; N = 10^4..10^6) maximizing max_live_self/q0 and steps
      - Multi-scale Sturmian / Arnoux-Rauzy / bad continued-fraction / valid-BWT stress tests
      - Check whether `q_self > 2*r0 -> boundary-RV fallback` bounds max_q vs worst-case steps!
"""
import itertools
import random
import sys
import time

from ortools.sat.python import cp_model

sys.path.insert(0, "/tmp/fcp-b")
import verify_potentials as bp
import escape_padded_ring as epr

sys.path.insert(0, "/tmp/fcp-c")
import test_unified_schur as g_uni
import explore_iet_schur as g_exp


def count_runs(L):
    if not L:
        return 0
    return sum(1 for i in range(1, len(L)) if L[i] != L[i - 1]) + 1


def count_interior_blocks(in_D):
    n = len(in_D)
    cnt = 0
    i = 0
    while i < n:
        if in_D[i]:
            s = i
            while i < n and in_D[i]:
                i += 1
            e = i - 1
            if s > 0 and e < n - 1:
                cnt += 1
        else:
            i += 1
    return cnt


def part1_audit_theorem_b2():
    print("=== 1.1 AUDITING BETA'S THEOREM B2 (ALL (L,F)-DEFINED ACYCLIC DELETIONS) ===", flush=True)

    # (A) Exhaustive check over ALL 2^9 = 512 subsets E subset {0,1,2}^2 (including self-loops!)
    #     on ALL arbitrary ternary columns L of length n = 4..7 (whenever D = {(L,F) in E} is acyclic on LF)!
    #     Test BOTH: Delta r <= #interior_blocks(D) AND Delta r <= |D|!
    #     AND check if bucket-split relabelling ever breaks it!
    checked_arb = 0
    viol_blk = None
    viol_D = None
    max_dr_minus_blk = -999
    max_dr_minus_D = -999

    # Test all 25 non-empty acyclic E without self-loops + 200 random E with self-loops on n=5,6,7
    # Wait: test ALL 511 non-empty E on ALL ternary columns n=5 (243 cols) & n=6 (729 cols -> sample 150 cols + all valid BWTs)!
    cols_to_test = []
    for n in (5, 6):
        for tup in itertools.product((0, 1, 2), repeat=n):
            if len(set(tup)) == 3:
                cols_to_test.append(list(tup))
    # Add random n=8..12 arbitrary columns
    rng = random.Random(12345)
    for _ in range(300):
        n = rng.randint(8, 12)
        L = [rng.randrange(3) for _ in range(n)]
        if len(set(L)) == 3:
            cols_to_test.append(L)

    # Precompute all 511 non-empty masks of 3x3
    all_E = []
    for mask in range(1, 1 << 9):
        E = set()
        for a in range(3):
            for b in range(3):
                if (mask >> (a * 3 + b)) & 1:
                    E.add((a, b))
        all_E.append(E)

    # Sample 60 E's for speed on n=6 (all 511 on n=5!)
    for L in cols_to_test:
        n = len(L)
        r0 = count_runs(L)
        cnts = [L.count(0), L.count(1), L.count(2)]
        C = [0, cnts[0], cnts[0] + cnts[1], n]
        F = [0] * cnts[0] + [1] * cnts[1] + [2] * cnts[2]
        occ = C[:3]
        LF = [0] * n
        for i, c in enumerate(L):
            LF[i] = occ[c]; occ[c] += 1
        E_list = all_E if n == 5 else rng.sample(all_E, 40)
        for E in E_list:
            in_D = [(L[i], F[i]) in E for i in range(n)]
            D_sz = sum(in_D)
            if D_sz == 0 or D_sz == n:
                continue
            # Check acyclicity of LF restricted to D, and compute L_next (with bucket splits!)
            acyclic = True
            # Compute maximal chain ending at surviving row j (walking LF while in D)
            # AND also include surviving bucket-component ID of the landing row `cur`!
            # Compute surviving contiguous component ID inside each F-bucket
            comp_id = [-1] * n
            cid = 0
            for i in range(n):
                if not in_D[i]:
                    if i == 0 or in_D[i - 1] or F[i] != F[i - 1]:
                        cid += 1
                    comp_id[i] = cid
            L_next = []
            for j in range(n):
                if in_D[j]:
                    continue
                chain = [L[j]]
                cur = LF[j]
                steps = 0
                while in_D[cur]:
                    chain.append(L[cur])
                    cur = LF[cur]
                    steps += 1
                    if steps > n:
                        acyclic = False
                        break
                if not acyclic:
                    break
                # Include comp_id[cur] (EXACT bucket-split relabelling!)
                L_next.append((tuple(chain), comp_id[cur]))
            if not acyclic:
                continue
            checked_arb += 1
            r2 = count_runs(L_next)
            iblk = count_interior_blocks(in_D)
            dr = r2 - r0
            if dr - iblk > max_dr_minus_blk:
                max_dr_minus_blk = dr - iblk
            if dr - D_sz > max_dr_minus_D:
                max_dr_minus_D = dr - D_sz
            if dr > iblk and viol_blk is None:
                viol_blk = (n, L, sorted(E), r0, r2, iblk, D_sz)
            if dr > D_sz and viol_D is None:
                viol_D = (n, L, sorted(E), r0, r2, iblk, D_sz)
    print(f"  General acyclic (L,F)-defined E (WITH bucket splits, {checked_arb} checks): "
          f"max(dr - #int_blk)={max_dr_minus_blk:+d} (viol={viol_blk}), max(dr - |D|)={max_dr_minus_D:+d} (viol={viol_D})", flush=True)

    # (B) CP-SAT on ARBITRARY ternary columns L (n=6..9) for BOTH BDMR self-run AND Ascent-DAG
    #     testing `r' - r > #interior_blocks(D)` (stronger than `> |D|`)!
    for mode in ("self_run", "ascent_dag"):
        for n in (6, 7, 8, 9):
            t0 = time.time()
            model = cp_model.CpModel()
            L_v = [model.NewIntVar(0, 2, f"L_{i}") for i in range(n)]
            cnt = [model.NewIntVar(1, n - 2, f"cnt_{c}") for c in range(3)]
            for c in range(3):
                bs = []
                for i in range(n):
                    b = model.NewBoolVar(f"is_{i}_{c}")
                    model.Add(L_v[i] == c).OnlyEnforceIf(b)
                    model.Add(L_v[i] != c).OnlyEnforceIf(b.Not())
                    bs.append(b)
                model.Add(sum(bs) == cnt[c])
            model.Add(sum(cnt) == n)
            c1 = cnt[0]
            c2 = model.NewIntVar(2, n - 1, "c2")
            model.Add(c2 == cnt[0] + cnt[1])
            F_v = [model.NewIntVar(0, 2, f"F_{i}") for i in range(n)]
            for i in range(n):
                b0 = model.NewBoolVar(f"f0_{i}"); b2 = model.NewBoolVar(f"f2_{i}")
                model.Add(i < c1).OnlyEnforceIf(b0); model.Add(i >= c1).OnlyEnforceIf(b0.Not())
                model.Add(i >= c2).OnlyEnforceIf(b2); model.Add(i < c2).OnlyEnforceIf(b2.Not())
                model.Add(F_v[i] == 0).OnlyEnforceIf(b0)
                model.Add(F_v[i] == 2).OnlyEnforceIf(b2)
                model.Add(F_v[i] == 1).OnlyEnforceIf([b0.Not(), b2.Not()])
            LF_v = [model.NewIntVar(0, n - 1, f"LF_{i}") for i in range(n)]
            rank_c = [[model.NewIntVar(0, n, f"rk_{c}_{i}") for i in range(n)] for c in range(3)]
            for c in range(3):
                for i in range(n):
                    bi = model.NewBoolVar(f"eq_{i}_{c}")
                    model.Add(L_v[i] == c).OnlyEnforceIf(bi); model.Add(L_v[i] != c).OnlyEnforceIf(bi.Not())
                    if i == 0:
                        model.Add(rank_c[c][0] == 0)
                    else:
                        bp_v = model.NewBoolVar(f"eqp_{i}_{c}")
                        model.Add(L_v[i - 1] == c).OnlyEnforceIf(bp_v); model.Add(L_v[i - 1] != c).OnlyEnforceIf(bp_v.Not())
                        model.Add(rank_c[c][i] == rank_c[c][i - 1] + bp_v)
                    base = 0 if c == 0 else (c1 if c == 1 else c2)
                    model.Add(LF_v[i] == base + rank_c[c][i]).OnlyEnforceIf(bi)
            model.AddAllDifferent(LF_v)

            del_v = [model.NewBoolVar(f"del_{i}") for i in range(n)]
            for i in range(n):
                if mode == "self_run":
                    model.Add(L_v[i] == F_v[i]).OnlyEnforceIf(del_v[i])
                    model.Add(L_v[i] != F_v[i]).OnlyEnforceIf(del_v[i].Not())
                else:
                    model.Add(L_v[i] < F_v[i]).OnlyEnforceIf(del_v[i])
                    model.Add(L_v[i] >= F_v[i]).OnlyEnforceIf(del_v[i].Not())
            D_var = model.NewIntVar(1, n - 1, "D")
            model.Add(D_var == sum(del_v))

            # For self_run: forbid any closed cycle inside D (forbid L[i]==F[i] on a fixed point or all-deleted bucket!)
            # Actually for self_run, encode chain length k[i] in 0..n (0 if not del, else 1 + k[LF[i]]) -> forces acyclicity!
            if mode == "self_run":
                k_del = [model.NewIntVar(0, n, f"kd_{i}") for i in range(n)]
                k_lf = [model.NewIntVar(0, n, f"klf_{i}") for i in range(n)]
                for i in range(n):
                    model.AddElement(LF_v[i], k_del, k_lf[i])
                    model.Add(k_del[i] == 0).OnlyEnforceIf(del_v[i].Not())
                    model.Add(k_del[i] == 1 + k_lf[i]).OnlyEnforceIf(del_v[i])
                # L2[i] = L[i] * (n+1) + k_lf[i]
                L2_v = [model.NewIntVar(0, 3 * (n + 1), f"L2_{i}") for i in range(n)]
                for i in range(n):
                    model.Add(L2_v[i] == L_v[i] * (n + 1) + k_lf[i])
            else:
                # Ascent-DAG (0 < 1 < 2): max chain length <= 3!
                # Encode exact chain code for row i: code[i] in 0..26 (base-3 tuple of chain ending at i if in D, else 0)!
                # Wait: for surviving row j, L2[j] = (L[j], chain_ending_at_LF[j]).
                # Chain length starting at LF[j] inside D is at most 2 (since 0 < 1 < 2):
                #   step 1: x1 = LF[j]. If del[x1], L[x1] < F[x1] = L[j] (so L[x1] in {0, 1}).
                #   step 2: x2 = LF[x1]. If del[x1] and del[x2], L[x2] = 0, L[x1] = 1, L[j] = 2!
                # So L2[j] is uniquely encoded in 0..31 by (L[j], del[x1], L[x1] if del[x1] else 0, del[x1] & del[x2])!
                d1 = [model.NewBoolVar(f"d1_{i}") for i in range(n)]
                l1 = [model.NewIntVar(0, 2, f"l1_{i}") for i in range(n)]
                x2 = [model.NewIntVar(0, n - 1, f"x2_{i}") for i in range(n)]
                d2 = [model.NewBoolVar(f"d2_{i}") for i in range(n)]
                d12 = [model.NewBoolVar(f"d12_{i}") for i in range(n)]
                L2_v = [model.NewIntVar(0, 40, f"L2_{i}") for i in range(n)]
                for i in range(n):
                    model.AddElement(LF_v[i], del_v, d1[i])
                    model.AddElement(LF_v[i], L_v, l1[i])
                    model.AddElement(LF_v[i], LF_v, x2[i])
                    model.AddElement(x2[i], del_v, d2[i])
                    model.AddBoolAnd([d1[i], d2[i]]).OnlyEnforceIf(d12[i])
                    model.AddBoolOr([d1[i].Not(), d2[i].Not()]).OnlyEnforceIf(d12[i].Not())
                    l1_eff = model.NewIntVar(0, 2, f"l1e_{i}")
                    model.Add(l1_eff == l1[i]).OnlyEnforceIf(d1[i])
                    model.Add(l1_eff == 0).OnlyEnforceIf(d1[i].Not())
                    model.Add(L2_v[i] == L_v[i] * 12 + d1[i] * 6 + l1_eff * 2 + d12[i])

            # Count interior blocks of D:
            # Start of an interior block at s in 1..n-2: not del[s-1] and del[s] and exists e>=s with del[e] and not del[e+1]?
            # Simpler: #interior_blocks = #blocks - del[0] - del[n-1] (wait: if all deleted, D<=n-1 so del[0] and del[n-1] can't be same block!)
            # #blocks = del[0] + sum_{i=1..n-1} (not del[i-1] and del[i])
            # So #interior_blocks = sum_{i=1..n-1} (not del[i-1] and del[i]) - del[n-1]!!
            # Wait: check! If del = [1, 0, 1]: starts at i=0 and i=2 -> sum_{i>=1} = 1, minus del[2]=1 -> 0 interior blocks! EXACT!
            # If del = [0, 1, 1, 0]: starts at i=1 -> sum = 1, minus del[3]=0 -> 1 interior block! EXACT!
            # If del = [1, 1, 0]: starts at i=0 -> sum_{i>=1} = 0, minus del[2]=0 -> 0 interior blocks! EXACT!
            blk_starts = []
            for i in range(1, n):
                bs = model.NewBoolVar(f"bs_{i}")
                model.AddBoolAnd([del_v[i - 1].Not(), del_v[i]]).OnlyEnforceIf(bs)
                model.AddBoolOr([del_v[i - 1], del_v[i].Not()]).OnlyEnforceIf(bs.Not())
                blk_starts.append(bs)
            iblk_var = model.NewIntVar(0, n, "iblk")
            model.Add(iblk_var == sum(blk_starts) - del_v[n - 1])

            rb0 = []
            for i in range(n - 1):
                b = model.NewBoolVar(f"rb0_{i}")
                model.Add(L_v[i] != L_v[i + 1]).OnlyEnforceIf(b)
                model.Add(L_v[i] == L_v[i + 1]).OnlyEnforceIf(b.Not())
                rb0.append(b)
            r0_var = model.NewIntVar(1, n, "r0")
            model.Add(r0_var == 1 + sum(rb0))

            rb2 = []
            for i in range(n):
                for j in range(i + 1, n):
                    adj = model.NewBoolVar(f"adj_{i}_{j}")
                    conds = [del_v[i].Not(), del_v[j].Not()] + [del_v[k] for k in range(i + 1, j)]
                    model.AddBoolAnd(conds).OnlyEnforceIf(adj)
                    model.AddBoolOr([c.Not() for c in conds]).OnlyEnforceIf(adj.Not())
                    dif = model.NewBoolVar(f"dif_{i}_{j}")
                    model.Add(L2_v[i] != L2_v[j]).OnlyEnforceIf(dif)
                    model.Add(L2_v[i] == L2_v[j]).OnlyEnforceIf(dif.Not())
                    bnd = model.NewBoolVar(f"bnd_{i}_{j}")
                    model.AddBoolAnd([adj, dif]).OnlyEnforceIf(bnd)
                    model.AddBoolOr([adj.Not(), dif.Not()]).OnlyEnforceIf(bnd.Not())
                    rb2.append(bnd)
            r2_var = model.NewIntVar(1, n, "r2")
            model.Add(r2_var == 1 + sum(rb2))

            # Test STRONGER claim: r2 - r0 > #interior_blocks(D)!
            model.Add(r2_var - r0_var > iblk_var)

            solver = cp_model.CpSolver()
            solver.parameters.max_time_in_seconds = 10.0
            st = solver.Solve(model)
            dt = time.time() - t0
            print(f"  CP-SAT [{mode:10s} n={n}] testing (r'-r > #int_blocks) ({dt:.2f}s): {solver.StatusName(st)}", flush=True)

    # (C) Explicit valid-BWT counterexample proving NON-(L,F)-defined acyclic |D|=1 CAN give Delta r = +2 > |D|!
    # Need a valid BWT where some row z has L[z-1]=L[z]=L[z+1]=a, F[z-1]=F[z]=F[z+1]=b (same (L,F)!),
    # and deleting ONLY z (|D|=1) leaves r_del = r0 (0 destroyed), while FL(z) splits a b-run by +2 -> Delta r = +2 > 1 = |D|!
    # Check small valid binary BWTs!
    found_non_lf = None
    for n in range(6, 12):
        for mask in range(1 << n):
            T = [(mask >> i) & 1 for i in range(n)]
            if len(set(T)) < 2: continue
            L, p = epr.bwt_encode_fast(T)
            r0 = count_runs(L)
            cnt0 = L.count(0)
            F = [0] * cnt0 + [1] * (n - cnt0)
            occ = [0, cnt0]
            LF = [0] * n
            for i, c in enumerate(L): LF[i] = occ[c]; occ[c] += 1
            for z in range(1, n - 1):
                if L[z - 1] == L[z] == L[z + 1] and F[z - 1] == F[z] == F[z + 1]:
                    # Delete ONLY z!
                    L2 = []
                    for j in range(n):
                        if j == z: continue
                        if LF[j] == z:
                            L2.append((L[z], L[j]))
                        else:
                            L2.append((L[j],))
                    r2 = count_runs(L2)
                    if r2 - r0 > 1:
                        found_non_lf = (n, T, L, z, r0, r2, r2 - r0)
                        break
            if found_non_lf:
                break
        if found_non_lf:
            break
    print(f"  NECESSITY WITNESS (non-(L,F)-defined |D|=1 on valid binary BWT): {found_non_lf}", flush=True)


def run_aht_on_iet(pcs, in_p, exit_p, b=0, aht_thresh=2):
    """Run Beta's exact solve_bucket_aht_spiral on a synthetic open IET and return (max_live_self, steps, splits)."""
    d_list = []
    i_list = []
    rules = [(0, i) for i in range(100)]
    for dlo, ilo, ln, wd in pcs:
        d_list.append((b, b, dlo, ilo, ln, wd, 1))
        i_list.append((b, b, dlo, ilo, ln, wd, 1))
    for dlo, ln, wd in exit_p:
        d_list.append((b, 99, dlo, 0, ln, wd, 1))
    for ilo, ln, wd in in_p:
        i_list.append((99, b, 0, ilo, ln, wd, 1))
    p_state = [-1, 0, 0, None]
    emitted, st = epr.solve_bucket_aht_spiral(b, d_list, i_list, rules, p_state, aht_thresh=aht_thresh, allow_split=True)
    return st


def part2_attack_aht_and_subinterval():
    print("\n=== 1.2 ADVERSARIAL HUNT FOR Delta_q = +1 BLOWUP IN IET-BSE-AHT & SUB-INTERVAL 1-PIECE SCHUR ===", flush=True)

    # 1. Evolutionary / hill-climbing adversarial search over d-IETs (d = 4, 6, 8, 12) with N = 10^5
    #    maximizing (max_live_self / d) AND total_steps = case1 + case2 + phase2!
    rng = random.Random(2026)
    for d in (4, 6, 8, 12):
        N = 100000
        best_ratio = 0.0
        best_steps = 0
        best_case = None

        def make_iet(cuts, perm):
            lens = [cuts[0]] + [cuts[i] - cuts[i - 1] for i in range(1, d - 1)] + [N - cuts[-1]]
            src = [0]
            for l in lens[:-1]: src.append(src[-1] + l)
            dst = [0] * d
            acc = 0
            for pi in perm:
                dst[pi] = acc; acc += lens[pi]
            # Carve width-1 exit at N-1
            pcs = []
            exit_p = []
            in_p = []
            for i in range(d):
                if i == d - 1 and lens[i] > 1:
                    pcs.append((src[i], dst[i], lens[i] - 1, i))
                    exit_p.append((N - 1, 1, i))
                    in_p.append((dst[i] + lens[i] - 1, 1, i))
                elif i == d - 1:
                    exit_p.append((N - 1, 1, i))
                    in_p.append((dst[i], 1, i))
                else:
                    pcs.append((src[i], dst[i], lens[i], i))
            return pcs, in_p, exit_p

        # Random restarts + hill climbing mutations on cuts & perm
        for restart in range(25):
            cuts = sorted(rng.sample(range(1, N - 1), d - 1))
            perm = list(range(d))
            rng.shuffle(perm)
            cur_score = 0
            for it in range(40):
                # Mutate cuts or perm
                ncuts = list(cuts)
                nperm = list(perm)
                if it > 0:
                    if rng.random() < 0.7:
                        idx = rng.randrange(d - 1)
                        lo_b = 1 if idx == 0 else ncuts[idx - 1] + 1
                        hi_b = (N - 2) if idx == d - 2 else ncuts[idx + 1] - 1
                        if lo_b < hi_b:
                            if rng.random() < 0.5:
                                ncuts[idx] = rng.randint(lo_b, hi_b)
                            else:
                                delta = rng.randint(-max(1, (hi_b - lo_b) // 10), max(1, (hi_b - lo_b) // 10))
                                ncuts[idx] = max(lo_b, min(hi_b, ncuts[idx] + delta))
                    else:
                        i1, i2 = rng.sample(range(d), 2)
                        nperm[i1], nperm[i2] = nperm[i2], nperm[i1]
                pcs, in_p, exit_p = make_iet(ncuts, nperm)
                st = run_aht_on_iet(pcs, in_p, exit_p)
                tot_steps = st["c1_steps"] + st["c2_steps"] + st["p2_self"]
                ratio = st["max_live_self"] / d
                score = st["max_live_self"] * 10 + tot_steps
                if score > cur_score:
                    cuts, perm, cur_score = ncuts, nperm, score
                if ratio > best_ratio or (ratio == best_ratio and tot_steps > best_steps):
                    best_ratio = ratio
                    best_steps = tot_steps
                    best_case = (st["max_live_self"], st["c1_steps"], st["c2_steps"], st["c2_splits"], st["p2_self"], ncuts, nperm)
        ms, c1, c2, spl, p2s, _, _ = best_case
        print(f"  Adversarial {d:2d}-IET (N={N}, 1000 evals): max_live_self={ms:3d} ({ms/d:.2f}x d), "
              f"c1={c1:3d}, c2={c2:3d}, c2_splits={spl:3d}, p2_self={p2s:3d}, tot_steps={c1+c2+p2s:4d}", flush=True)

    # 2. Targeted worst-case `Delta q = +1` cascade:
    #    Scale N = 10^3, 10^4, 10^5, 10^6 on an IRRATIONAL-RATIO d-IET (d = 4, 6, 8)
    print("\n--- Scaling N = 10^3 .. 10^6 on Algebraic-Irrational d-IETs (d=4, 6, 8) ---", flush=True)
    import math
    irrat_weights = {
        4: [1.0, math.sqrt(2) - 1, math.sqrt(3) - 1, math.sqrt(5) - 2],
        6: [1.0, math.sqrt(2) - 1, math.sqrt(3) - 1, math.sqrt(5) - 2, math.sqrt(7) - 2, math.pi - 3],
        8: [1.0, math.sqrt(2) - 1, math.sqrt(3) - 1, math.sqrt(5) - 2, math.sqrt(7) - 2, math.sqrt(11) - 3, math.sqrt(13) - 3, math.pi - 3],
    }
    for d in (4, 6, 8):
        ws = irrat_weights[d]
        s_w = sum(ws)
        perm = list(reversed(range(d)))
        for N in (1000, 10000, 100000, 1000000):
            lens = [max(2, int(round(N * w / s_w))) for w in ws]
            lens[-1] = max(2, N - sum(lens[:-1]))
            N_act = sum(lens)
            src = [0]
            for l in lens[:-1]: src.append(src[-1] + l)
            dst = [0] * d
            acc = 0
            for pi in perm: dst[pi] = acc; acc += lens[pi]
            pcs = [(src[i], dst[i], lens[i] - (1 if i == d - 1 else 0), i) for i in range(d)]
            exit_p = [(N_act - 1, 1, d - 1)]
            in_p = [(dst[d - 1] + lens[d - 1] - 1, 1, d - 1)]
            st = run_aht_on_iet(pcs, in_p, exit_p)
            slp_g = [('lit', i) for i in range(d + 2)]
            g_pcs = [(dlo, ln, ilo, 0, 0, wd) for dlo, ilo, ln, wd in pcs] + [(N_act - 1, 1, 0, 0, 99, d - 1)]
            g_in = [(0, 1, dst[d - 1] + lens[d - 1] - 1, 99, 0, d)]
            _, g_steps, g_ops = g_uni.euclid_accelerate_2iet_bucket(g_pcs, g_in, slp_g)
            print(
                f"  IrratRev {d}-IET N={N_act:7d} | Beta AHT: max_q={st['max_live_self']:3d}({st['max_live_self']/d:.2f}d) "
                f"c1={st['c1_steps']:3d} c2={st['c2_steps']:3d} spl={st['c2_splits']:3d} p2={st['p2_self']:3d} | "
                f"Gamma SubInt: steps={g_steps:3d} ops={g_ops:4d}",
                flush=True,
            )

    # 3. Also test a valid BWT family with HUGE irrational-like / Tribonacci + multi-tag structure!
    print("\n--- Valid BWT Stress Test: Multi-Scale Tribonacci in Beta AHT ---", flush=True)
    for m in (10, 14, 18):
        T = epr.make_tribonacci(m)
        L, p = epr.bwt_encode_fast(T)
        runs = []
        i = 0
        while i < len(L):
            j = i + 1
            while j < len(L) and L[j] == L[i]: j += 1
            runs.append((L[i], j - i)); i = j
        st = epr.run_iet_bse_aht(runs, p, order_mode="rarest_first_deg", aht_thresh=2, verify_seq=T)
        print(f"  Tribonacci m={m:2d} n={len(T):6d} r0={len(runs):2d} | AHT: max_live_self={st['max_live_self']:2d} "
              f"c1={st['c1_steps']:3d} c2={st['c2_steps']:3d} p2={st['p2_self']:3d} tot={st['total_steps']:4d}", flush=True)


if __name__ == "__main__":
    part1_audit_theorem_b2()
    part2_attack_aht_and_subinterval()
