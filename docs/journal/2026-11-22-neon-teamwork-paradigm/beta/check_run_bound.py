#!/usr/bin/env python3
"""
Exact mathematical search for smallest valid-BWT counterexamples to r_{t+1} <= r_t
under:
  (1) Threshold pair contraction (zero bucket splits)
  (2) General bucket-split 2-coloring pair contraction
  (3) Lazard extreme-bucket deletion (when B_c has 0 self-edges and ALL entries in B_c contract)
"""
import sys
sys.path.insert(0, "/tmp/teamwork-b")
from deep_analysis import bwt_forward, runs_from_seq, full_split_pair_step

def one_step_r(L, p, in_L):
    uniq = sorted(set(L))
    mp = {c: i for i, c in enumerate(uniq)}
    L0 = [mp[c] for c in L]
    sigma = len(uniq)
    n = len(L0)
    runs = runs_from_seq(L0)
    r0 = len(runs)
    counts = [0] * sigma
    for c, w in runs:
        counts[c] += w
    C = [0] * (sigma + 1)
    tot = 0
    for c in range(sigma):
        C[c] = tot
        tot += counts[c]
    C[sigma] = tot
    bucket_runs = [[] for _ in range(sigma)]
    pos, f_sym = 0, 0
    while f_sym < sigma and C[f_sym + 1] == 0:
        f_sym += 1
    for c, w in runs:
        rem, cur_pos = w, pos
        while rem > 0:
            while C[f_sym + 1] <= cur_pos:
                f_sym += 1
            take = min(rem, C[f_sym + 1] - cur_pos)
            bucket_runs[f_sym].append((c, take))
            cur_pos += take
            rem -= take
        pos += w
    rules = [('base', c) for c in uniq]
    sym_len = [1] * sigma
    sym_to_rule = list(range(sigma))
    res = full_split_pair_step(runs, sigma, n, p, 0, sym_len, sym_to_rule, rules, C, bucket_runs, in_L)
    if res is None:
        return r0, r0, []
    return r0, len(res[0]), res[0]

# Search valid binary & ternary BWTs for smallest counterexample to r1 <= r0 under threshold cut!
best_thresh_ex = None
best_ratio_thresh = 1.0
best_gen_ex = None
best_ratio_gen = 1.0

for base, max_n in [(2, 12), (3, 8)]:
    for n in range(3, max_n + 1):
        for z in range(base ** n):
            T = []
            tmp = z
            for _ in range(n):
                T.append(tmp % base)
                tmp //= base
            if len(set(T)) < base:
                continue
            # Check primitive (single-cycle BWT)
            if any(T == T[:d] * (n // d) for d in range(1, n) if n % d == 0):
                continue
            L, p = bwt_forward(T)
            sigma = base
            # Threshold cuts (asc & desc)
            for th in range(1, sigma):
                for asc in (True, False):
                    in_L = [s < th for s in range(sigma)] if asc else [s >= th for s in range(sigma)]
                    r0, r1, out_runs = one_step_r(L, p, in_L)
                    if r1 > r0:
                        if best_thresh_ex is None or (len(T) < len(best_thresh_ex[0])) or (len(T) == len(best_thresh_ex[0]) and r1 / r0 > best_thresh_ex[5] / best_thresh_ex[4]):
                            best_thresh_ex = (T, L, p, in_L, r0, r1, out_runs)
                        if r1 / r0 > best_ratio_thresh:
                            best_ratio_thresh = r1 / r0
            # Non-threshold 2-colorings (only exist for base >= 3)
            if sigma >= 3:
                for mask in range(1, (1 << sigma) - 1):
                    in_L = [bool((mask >> s) & 1) for s in range(sigma)]
                    # Check if non-threshold
                    is_th = any(in_L == [s < th for s in range(sigma)] or in_L == [s >= th for s in range(sigma)] for th in range(1, sigma))
                    if is_th:
                        continue
                    r0, r1, out_runs = one_step_r(L, p, in_L)
                    if r1 > r0:
                        if best_gen_ex is None or len(T) < len(best_gen_ex[0]):
                            best_gen_ex = (T, L, p, in_L, r0, r1, out_runs)
                        if r1 / r0 > best_ratio_gen:
                            best_ratio_gen = r1 / r0

print("Smallest valid-BWT counterexample to r1 <= r0 under THRESHOLD cut:")
print("  T=", best_thresh_ex[0], "L=", best_thresh_ex[1], "p=", best_thresh_ex[2])
print("  in_L=", best_thresh_ex[3], "r0=", best_thresh_ex[4], "-> r1=", best_thresh_ex[5], "out_runs=", best_thresh_ex[6])
print(f"  Max r1/r0 seen (threshold): {best_ratio_thresh:.3f}")

if best_gen_ex:
    print("\nSmallest valid-BWT counterexample to r1 <= r0 under NON-THRESHOLD 2-coloring:")
    print("  T=", best_gen_ex[0], "L=", best_gen_ex[1], "p=", best_gen_ex[2])
    print("  in_L=", best_gen_ex[3], "r0=", best_gen_ex[4], "-> r1=", best_gen_ex[5], "out_runs=", best_gen_ex[6])
    print(f"  Max r1/r0 seen (non-threshold): {best_ratio_gen:.3f}")
