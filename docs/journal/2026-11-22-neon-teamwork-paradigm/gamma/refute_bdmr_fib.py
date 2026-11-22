#!/usr/bin/env python3
"""
Exact verification of the infinite valid single-cycle BWT family T_m = Fib_{2m} + $ ($ > 1 > 0)
with constant r_0 = 4:
1. Verifies exact closed-form BWT L(Fib_{2m}+$) = [(1, F_{2m-2}-1), (2, 1), (0, F_{2m-1}), (1, 1)], p = F_{2m-2}-1
   for 2m = 8, 10, 12, 14, 16, 18, 20 against exact bwt_forward.
2. Shows Beta's exact Variant B (with 0.9*mc_cnt threshold preference) blows up to
   max_r = Theta(log n) and sum_r = Theta(r_0 log^2 n) up to 2m = 80 (n = 3.79e16),
   refuting O(r_0 log n) worst-case work on valid BWTs.
3. Shows pure greedy_max_cut_coloring (dropping threshold preference) collapses max_r back to 5
   (constant) and sum_r = O(r_0 log n).
4. Cross-checks Alpha's Lazard-BWT on Fib_{2m}+$ (max_r = 4 constant by proved r_t <= r_0 invariant).
"""
import sys

sys.path.insert(0, "/tmp/teamwork-b")
sys.path.insert(0, "/tmp/teamwork-a")

import deep_analysis as bdmr
import verify_full_and_falsify as alpha


def fib_word(k):
    a, b = [0], [0, 1]
    for _ in range(k - 2):
        a, b = b, b + a
    return b


def run_bdmr_from_runs(runs_init, p_init, use_thresh_shortcut=True):
    runs = list(runs_init)
    n = sum(w for _, w in runs)
    sigma = 3
    p = p_init
    p_offset = 0
    rules = [("lit", c) for c in range(sigma)]
    sym_len = [1] * sigma
    sym_to_rule = list(range(sigma))
    rounds = 0
    max_r = len(runs)
    sum_r = 0
    max_sig = sigma
    while n > 1:
        r = len(runs)
        if r > max_r:
            max_r = r
        if sigma > max_sig:
            max_sig = sigma
        sum_r += r
        counts = [0] * sigma
        for c, w in runs:
            counts[c] += w
        C = [0] * (sigma + 1)
        for c in range(sigma):
            C[c + 1] = C[c] + counts[c]
        bucket_runs = [[] for _ in range(sigma)]
        diff_asc = [0] * (sigma + 1)
        diff_desc = [0] * (sigma + 1)
        self_total = 0
        pos = 0
        f_sym = 0
        while f_sym < sigma and C[f_sym + 1] == 0:
            f_sym += 1
        for c, w in runs:
            rem = w
            cur_pos = pos
            while rem > 0:
                while C[f_sym + 1] <= cur_pos:
                    f_sym += 1
                avail = C[f_sym + 1] - cur_pos
                take = rem if rem <= avail else avail
                bucket_runs[f_sym].append((c, take))
                if c == f_sym:
                    self_total += take
                elif c < f_sym:
                    diff_asc[c + 1] += take
                    diff_asc[f_sym + 1] -= take
                else:
                    diff_desc[f_sym + 1] += take
                    diff_desc[c + 1] -= take
                cur_pos += take
                rem -= take
            pos += w
        if self_total == n:
            break
        if self_total > 0 and (not use_thresh_shortcut or self_total >= 0.05 * n):
            res = bdmr.self_run_contraction_step(
                runs, sigma, n, p, p_offset, sym_len, sym_to_rule, rules, C, bucket_runs
            )
            if res[6] is not None:
                break
            runs, p, p_offset, del_cnt, sym_to_rule, sigma, _ = res
            n -= del_cnt
            rounds += 1
            continue
        in_L_mc, mc_cnt = bdmr.greedy_max_cut_coloring(bucket_runs, sigma)
        if use_thresh_shortcut:
            best_th_cnt = 0
            best_dir = "asc"
            best_th = -1
            acc = 0
            for th in range(1, sigma):
                acc += diff_asc[th]
                if acc > best_th_cnt:
                    best_th_cnt = acc
                    best_dir = "asc"
                    best_th = th
            acc = 0
            for th in range(1, sigma):
                acc += diff_desc[th]
                if acc > best_th_cnt:
                    best_th_cnt = acc
                    best_dir = "desc"
                    best_th = th
            if best_th_cnt >= 0.9 * mc_cnt and best_th != -1:
                in_L = [(c < best_th) if best_dir == "asc" else (c >= best_th) for c in range(sigma)]
            else:
                in_L = in_L_mc
        else:
            in_L = in_L_mc
        runs, p, p_offset, del_cnt, sym_to_rule, sigma = bdmr.full_split_pair_step(
            runs, sigma, n, p, p_offset, sym_len, sym_to_rule, rules, C, bucket_runs, in_L
        )
        n -= del_cnt
        rounds += 1
    return rounds, max_r, sum_r, max_sig


def main():
    F = [1, 1]
    for _ in range(85):
        F.append(F[-1] + F[-2])

    print("=== 1. Exact Closed-Form Verification of L(Fib_{2m} + $) for 2m = 8..20 ===")
    for k in [8, 10, 12, 14, 16, 18, 20]:
        T = fib_word(k) + [2]
        L, p = bdmr.bwt_forward(T)
        runs = bdmr.runs_from_seq(L)
        expected = [(1, F[k - 2] - 1), (2, 1), (0, F[k - 1]), (1, 1)]
        assert runs == expected and p == F[k - 2] - 1, (k, runs, expected, p)
        # Also verify Alpha's Lazard-BWT on exact bytes
        T_a, st_a = alpha.lazard_decode(bytes(L), p, mode="both", measure=True)
        assert list(T_a) == T and st_a["max_live_r"] == 4
    print("[VERIFIED] Closed form L(Fib_{2m}+$) = [(1, F_{2m-2}-1), (2,1), (0, F_{2m-1}), (1,1)] and Alpha max_r=4 PASSED for 2m=8..20.\n")

    print("=== 2. Scaling up to k=80 (n=3.79e16): Beta Variant B (thresh) vs Pure Max-Cut ===")
    print(f"{'k':>3} {'n':>18} | {'VarB rds':>8} {'VarB max_r':>10} {'VarB sum_r':>10} {'sr/k^2':>7} | {'MC max_r':>8} {'MC sum_r':>8}")
    print("-" * 86)
    for k in [8, 12, 16, 20, 24, 30, 40, 50, 60, 80]:
        f_km2, f_km1 = F[k - 2], F[k - 1]
        runs = [(1, f_km2 - 1), (2, 1), (0, f_km1), (1, 1)]
        n = f_km2 + f_km1 + 1
        rds_b, mr_b, sr_b, _ = run_bdmr_from_runs(runs, f_km2 - 1, use_thresh_shortcut=True)
        rds_m, mr_m, sr_m, _ = run_bdmr_from_runs(runs, f_km2 - 1, use_thresh_shortcut=False)
        print(f"{k:3d} {n:18d} | {rds_b:8d} {mr_b:10d} {sr_b:10d} {sr_b/(k*k):7.3f} | {mr_m:8d} {sr_m:8d}")


if __name__ == "__main__":
    main()
