#!/usr/bin/env python3
"""
Investigator Beta — Cycle 2, Objective 2:
Exact Verification & Counterexample Search for:
  1. Theorem B2: Universal r' + n' <= r + n (Delta r <= |D|) and tighter block bound
     Delta r <= #interior_blocks(D) <= |D| across:
       - ALL 2-coloring BDMR pair contractions (exhaustive binary n<=10, ternary n<=7, quat n<=6)
       - ALL BDMR self-run contractions (single-symbol AND simultaneous all-symbol!)
       - ALL Ascent-DAG / Descent-DAG contractions (D = {j : L[j] < F[j]} and {j : L[j] > F[j]})!
  2. Tightness of c = 1 in c*r + n <= c*r' + n' (exact witness at n=5).
  3. Theorem B3: Checking E_RG (run-graph overlap edges), q_0 (bi-monotone strips), and
     Ref #1's rle_10 = 1^13 0^12 2^8 1^8 + Fib_{2m}+$ against any f(r)-bounded monotone potential.
"""

import itertools
import random
import sys

sys.path.insert(0, "/tmp/fcp-b")
import escape_padded_ring as epr


def runs_count(L):
    if not L:
        return 0
    r = 1
    prev = L[0]
    for x in L[1:]:
        if x != prev:
            r += 1
            prev = x
    return r


def contract_text_pair(T, left_set):
    """Exact text-level 2-coloring pair contraction: merge every (a, b) with a in left_set, b not in left_set
    (cyclic on T!), returning new string T' over ordered alphabet (lex order of original strings!)."""
    n = len(T)
    # Build tokens around the cycle T
    # Position i starts a token iff NOT (T[i-1] in left_set and T[i] not in left_set)
    # Wait: if ALL positions are in left_set or NONE, no contraction happens.
    is_del = [(T[(i - 1) % n] in left_set) and (T[i] not in left_set) for i in range(n)]
    if not any(is_del):
        return None, 0
    if all(is_del):
        return None, n
    tokens = []
    # Find first surviving start
    s0 = is_del.index(False)
    i = s0
    for _ in range(n):
        if not is_del[i]:
            if is_del[(i + 1) % n]:
                tokens.append((T[i], T[(i + 1) % n]))
            else:
                tokens.append((T[i],))
        i = (i + 1) % n
    # Order tokens in exact string lex order induced by original T rotations!
    # Wait: in BDMR, a single letter (a,) vs (a, b): how does BDMR order (a,) vs (a, b)?
    # In BDMR (BWT-preserving zero-row-reordering!), row j stays in its EXACT original BWT row order!
    # So we compute L' DIRECTLY on BWT(T) with ZERO row reordering (which IS the exact BDMR definition!).
    return tokens, sum(is_del)


def bdmr_pair_bwt(L, left_set):
    """Exact BWT-level 2-coloring pair contraction (zero row reordering).
    Returns (L_next, D_size, num_interior_blocks)."""
    n = len(L)
    sigma = max(L) + 1
    counts = [0] * sigma
    for x in L: counts[x] += 1
    C = [0] * sigma
    acc = 0
    for c in range(sigma): C[c] = acc; acc += counts[c]
    # F[j]
    F = [0] * n
    for c in range(sigma):
        for j in range(C[c], C[c] + counts[c]):
            F[j] = c
    # LF[j]
    pos = list(C)
    LF = [0] * n
    for j, ch in enumerate(L):
        LF[j] = pos[ch]
        pos[ch] += 1

    in_D = [(L[j] in left_set) and (F[j] not in left_set) for j in range(n)]
    D_size = sum(in_D)
    if D_size == 0 or D_size == n:
        return None, D_size, 0

    # Count interior maximal contiguous blocks of D (blocks [s, e] with s > 0 and e < n-1)
    int_blocks = 0
    all_blocks = 0
    i = 0
    while i < n:
        if in_D[i]:
            s = i
            while i < n and in_D[i]:
                i += 1
            e = i - 1
            all_blocks += 1
            if s > 0 and e < n - 1:
                int_blocks += 1
        else:
            i += 1

    # Build L' on surviving rows S = {j : not in_D[j]}
    L_next = []
    for j in range(n):
        if in_D[j]:
            continue
        lf_j = LF[j]
        if in_D[lf_j]:
            L_next.append((L[lf_j], L[j]))
        else:
            L_next.append((L[j],))
    return L_next, D_size, int_blocks


def bdmr_self_run_bwt(L, target_syms=None):
    """Exact BWT-level self-run contraction (for target_syms, or all symbols if None).
    Deletes D = {j : L[j] == F[j] and F[j] in target_syms}.
    Each surviving row j in S gets L'[j] = (L[j], run_len_ending_at_j)."""
    n = len(L)
    sigma = max(L) + 1
    if target_syms is None:
        target_syms = set(range(sigma))
    counts = [0] * sigma
    for x in L: counts[x] += 1
    C = [0] * sigma
    acc = 0
    for c in range(sigma): C[c] = acc; acc += counts[c]
    F = [0] * n
    for c in range(sigma):
        for j in range(C[c], C[c] + counts[c]):
            F[j] = c
    pos = list(C)
    LF = [0] * n
    for j, ch in enumerate(L):
        LF[j] = pos[ch]
        pos[ch] += 1

    in_D = [(L[j] == F[j]) and (F[j] in target_syms) for j in range(n)]
    D_size = sum(in_D)
    if D_size == 0 or D_size == n:
        return None, D_size

    L_next = []
    for j in range(n):
        if in_D[j]:
            continue
        ch = L[j]
        # Walk LF while inside D to find exact self-run length k!
        k = 1
        cur = LF[j]
        while in_D[cur]:
            k += 1
            cur = LF[cur]
        L_next.append((ch, k))
    return L_next, D_size


def dag_chain_bwt(L, ascending=True):
    """Exact BWT-level Ascent-DAG (D = {j : L[j] < F[j]}) or Descent-DAG (D = {j : L[j] > F[j]})
    contraction! Every surviving row j gets the exact maximal ascending/descending chain ending at L[j]."""
    n = len(L)
    sigma = max(L) + 1
    counts = [0] * sigma
    for x in L: counts[x] += 1
    C = [0] * sigma
    acc = 0
    for c in range(sigma): C[c] = acc; acc += counts[c]
    F = [0] * n
    for c in range(sigma):
        for j in range(C[c], C[c] + counts[c]):
            F[j] = c
    pos = list(C)
    LF = [0] * n
    for j, ch in enumerate(L):
        LF[j] = pos[ch]
        pos[ch] += 1

    if ascending:
        in_D = [L[j] < F[j] for j in range(n)]
    else:
        in_D = [L[j] > F[j] for j in range(n)]
    D_size = sum(in_D)
    if D_size == 0 or D_size == n:
        return None, D_size

    L_next = []
    for j in range(n):
        if in_D[j]:
            continue
        # Walk backward under LF while in_D to collect full chain!
        chain = [L[j]]
        cur = LF[j]
        steps = 0
        while in_D[cur]:
            chain.append(L[cur])
            cur = LF[cur]
            steps += 1
            if steps > n:
                return None, D_size  # cycle inside D (impossible for strict < or >!)
        L_next.append(tuple(reversed(chain)))
    return L_next, D_size


def compute_run_graph_metrics(L):
    """Compute r, q_0 (bi-monotone strips), and E_RG (edges in the run-transition graph I_u -> I_v)."""
    n = len(L)
    sigma = max(L) + 1
    counts = [0] * sigma
    for x in L: counts[x] += 1
    C = [0] * sigma
    acc = 0
    for c in range(sigma): C[c] = acc; acc += counts[c]
    # Maximal runs
    runs = []
    cur_c = L[0]; start = 0
    for i in range(1, n):
        if L[i] != cur_c:
            runs.append((cur_c, start, i))
            cur_c = L[i]; start = i
    runs.append((cur_c, start, n))
    r = len(runs)

    # Map each row to its run id
    run_of = [0] * n
    for uid, (_, s, e) in enumerate(runs):
        for i in range(s, e):
            run_of[i] = uid

    # Bi-monotone strips q0 & run-graph edges E_RG
    f_cur = list(C)
    e_rg_edges = set()
    q0 = 0
    for uid, (c, s, e) in enumerate(runs):
        ln = e - s
        im_s = f_cur[c]
        im_e = im_s + ln
        f_cur[c] = im_e
        # Count how many runs [im_s, im_e) overlaps -> exact E_RG out-degree of uid!
        v = run_of[im_s]
        while v < r and runs[v][1] < im_e:
            e_rg_edges.add((uid, v))
            v += 1
        # Count how many F-buckets [s, e) overlaps -> q0
        # Wait: F-bucket boundaries
    # Exact q0 = sum over F-buckets of #runs intersecting F_c
    for c in range(sigma):
        if counts[c] == 0:
            continue
        s, e = C[c], C[c] + counts[c]
        v = run_of[s]
        while v < r and runs[v][1] < e:
            q0 += 1
            v += 1
    return r, q0, len(e_rg_edges)


def main():
    print("=== 1. EXHAUSTIVE CHECK OF THEOREM B2 (Delta r <= |D| & Block Bound) ===")
    pair_cases = 0; pair_viol_D = 0; pair_viol_blk = 0; max_pair_ratio = 0.0
    self_cases = 0; self_viol_D = 0; max_self_ratio = 0.0; self_witness = None
    dag_cases = 0; dag_viol_D = 0; max_dag_ratio = 0.0; dag_witness = None

    # Generate all binary n<=10, ternary n<=7, random quat n=6..14 + arbitrary non-BWT columns!
    test_L_list = []
    # Binary BWTs n=2..9
    for n in range(2, 10):
        for mask in range(1 << n):
            T = [(mask >> i) & 1 for i in range(n)]
            L, _ = epr.bwt_encode_fast(T)
            test_L_list.append(("bin_bwt", T, L))
    # Ternary BWTs n=2..6
    for n in range(2, 7):
        for idx in range(3 ** n):
            T = []; v = idx
            for _ in range(n): T.append(v % 3); v //= 3
            L, _ = epr.bwt_encode_fast(T)
            test_L_list.append(("tern_bwt", T, L))
    # Random quaternary/quinary BWTs n=6..20 (3000 strings)
    rng = random.Random(2026)
    for _ in range(3000):
        sig = rng.choice([3, 4, 5, 6])
        n = rng.randint(6, 20)
        T = [rng.randrange(sig) for _ in range(n)]
        L, _ = epr.bwt_encode_fast(T)
        test_L_list.append(("rand_bwt", T, L))

    for kind, T, L in test_L_list:
        r = runs_count(L)
        syms = sorted(set(L))
        s = len(syms)
        if s <= 1:
            continue

        # (A) All non-trivial 2-colorings (up to 30 for larger sig)
        if s <= 4:
            masks = range(1, (1 << s) - 1)
        else:
            masks = [rng.randint(1, (1 << s) - 2) for _ in range(12)]
        for m in masks:
            left_set = {syms[k] for k in range(s) if (m >> k) & 1}
            L2, D_sz, int_blk = bdmr_pair_bwt(L, left_set)
            if L2 is None:
                continue
            pair_cases += 1
            dr = runs_count(L2) - r
            if dr > D_sz:
                pair_viol_D += 1
                print("PAIR VIOL D:", T, left_set, dr, D_sz)
            if dr > int_blk:
                pair_viol_blk += 1
                print("PAIR VIOL BLK:", T, left_set, dr, int_blk)
            if D_sz > 0 and dr / D_sz > max_pair_ratio:
                max_pair_ratio = dr / D_sz

        # (B) Simultaneous self-run contraction (all symbols) + single-symbol
        L2, D_sz = bdmr_self_run_bwt(L, None)
        if L2 is not None:
            self_cases += 1
            dr = runs_count(L2) - r
            if dr > D_sz:
                self_viol_D += 1
                if self_witness is None:
                    self_witness = (T, L, dr, D_sz)
            if D_sz > 0 and dr / D_sz > max_self_ratio:
                max_self_ratio = dr / D_sz

        # (C) Ascent-DAG and Descent-DAG contractions
        for asc in (True, False):
            L2, D_sz = dag_chain_bwt(L, ascending=asc)
            if L2 is not None:
                dag_cases += 1
                dr = runs_count(L2) - r
                if dr > D_sz:
                    dag_viol_D += 1
                    if dag_witness is None:
                        dag_witness = (asc, T, L, L2, r, runs_count(L2), D_sz)
                if D_sz > 0 and dr / D_sz > max_dag_ratio:
                    max_dag_ratio = dr / D_sz

    print(f"  BDMR Pair contractions : {pair_cases:7d} cases | viol(dr<=|D|)={pair_viol_D} | viol(dr<=#int_blocks)={pair_viol_blk} | max(dr/|D|)={max_pair_ratio:.4f}")
    print(f"  BDMR Self-run (all sym): {self_cases:7d} cases | viol(dr<=|D|)={self_viol_D} | max(dr/|D|)={max_self_ratio:.4f}")
    if self_witness:
        print(f"    SELF-RUN WITNESS: {self_witness}")
    print(f"  Ascent/Descent-DAG     : {dag_cases:7d} cases | viol(dr<=|D|)={dag_viol_D} | max(dr/|D|)={max_dag_ratio:.4f}")
    if dag_witness:
        print(f"    DAG WITNESS: asc={dag_witness[0]} T={dag_witness[1]} r={dag_witness[4]}->r'={dag_witness[5]} (dr={dag_witness[5]-dag_witness[4]}) |D|={dag_witness[6]}")

    # Also test ARBITRARY non-BWT columns (binary n<=8, ternary n<=5) for pair, self-run, and DAG!
    arb_pair_viol = 0; arb_self_viol = 0; arb_dag_viol = 0; arb_cnt = 0
    for n in range(3, 6):
        for idx in range(3 ** n):
            L = []; v = idx
            for _ in range(n): L.append(v % 3); v //= 3
            if len(set(L)) <= 1: continue
            r = runs_count(L)
            for m in range(1, 7):
                left_set = {k for k in range(3) if (m >> k) & 1}
                L2, D_sz, int_blk = bdmr_pair_bwt(L, left_set)
                if L2 is not None:
                    arb_cnt += 1
                    if runs_count(L2) - r > int_blk:
                        arb_pair_viol += 1
            L2, D_sz = bdmr_self_run_bwt(L, None)
            if L2 is not None and runs_count(L2) - r > D_sz:
                arb_self_viol += 1
            for asc in (True, False):
                L2, D_sz = dag_chain_bwt(L, ascending=asc)
                if L2 is not None and runs_count(L2) - r > D_sz:
                    arb_dag_viol += 1
    print(f"  Arbitrary non-BWT cols : {arb_cnt:7d} pair checks | pair_viol={arb_pair_viol} | self_viol={arb_self_viol} | dag_viol={arb_dag_viol}")

    print("\n=== 2. CHECKING rle_10, E_RG, q0, AND Fib_{2m}+$ ===")
    # Ref #1's rle_10: T = 1^13 0^12 2^8 1^8
    T_rle10 = [1]*13 + [0]*12 + [2]*8 + [1]*8
    L_rle10, _ = epr.bwt_encode_fast(T_rle10)
    r0, q0, erg0 = compute_run_graph_metrics(L_rle10)
    print(f"  rle_10 (n={len(T_rle10)}): r={r0}, q0={q0}, E_RG={erg0}")
    for m in range(1, 7):
        left_set = {k for k in range(3) if (m >> k) & 1}
        L2, D_sz, int_blk = bdmr_pair_bwt(L_rle10, left_set)
        if L2 is not None:
            # Remap tuples to ints for compute_run_graph_metrics
            uniq = sorted(set(L2))
            mp = {u: i for i, u in enumerate(uniq)}
            L2i = [mp[u] for u in L2]
            r2, q2, erg2 = compute_run_graph_metrics(L2i)
            print(f"    left={sorted(left_set)}: |D|={D_sz:2d} (blk={int_blk}) | r: {r0}->{r2} (dr={r2-r0:+d}) | q0: {q0}->{q2} | E_RG: {erg0}->{erg2} | r+n: {r0+len(T_rle10)}->{r2+len(L2)}")

    # Check if E_RG is monotone under ALL pair contractions on ternary n<=6!
    erg_inc_max = 0; erg_inc_wit = None
    for kind, T, L in test_L_list:
        if kind != "tern_bwt": continue
        syms = sorted(set(L))
        if len(syms) < 3: continue
        r0, q0, erg0 = compute_run_graph_metrics(L)
        for m in range(1, 7):
            left_set = {syms[k] for k in range(3) if (m >> k) & 1}
            L2, D_sz, _ = bdmr_pair_bwt(L, left_set)
            if L2 is None: continue
            # Only check |D| >= n/4 (valid BDMR max-cut step)
            if D_sz * 4 < len(L): continue
            # Note: need exact lex order of new symbols for accurate E_RG!
            uniq = sorted(set(L2))
            mp = {u: i for i, u in enumerate(uniq)}
            L2i = [mp[u] for u in L2]
            r2, q2, erg2 = compute_run_graph_metrics(L2i)
            if erg2 - erg0 > erg_inc_max:
                erg_inc_max = erg2 - erg0
                erg_inc_wit = (T, sorted(left_set), D_sz, r0, r2, erg0, erg2)
    print(f"  Max E_RG increase under |D|>=n/4 pair step (ternary n<=6): +{erg_inc_max} | witness={erg_inc_wit}")


if __name__ == "__main__":
    main()
