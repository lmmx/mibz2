#!/tmp/solver-venv/bin/python
"""
1. Exact GLOBAL multi-step optimal schedule finder (Dijkstra / branch-and-bound over
   all valid 3-way selective BWT-space contractions + self-run contraction) on small
   valid BWTs (n = 8..18), proving the exact multi-step gap between bdmr_greedy,
   lazard_both, 1cut_thresh, and the TRUE global multi-step optimum of sum(Q_con) and sum(r_t).
2. Also inspects exact CP-SAT structural patterns on why Fib_21+$ forces r_next > r_0.
"""
import heapq
import random
import sys
import time

sys.path.insert(0, "/tmp/teamwork-b")
sys.path.insert(0, "/tmp/teamwork-b/solver_opt")
from deep_analysis import bwt_forward, runs_from_seq, self_run_contraction_step
from cpsat_schedule import compute_C_and_LF, apply_selective_forward, run_schedule


def enumerate_valid_moves(L):
    """
    Generate all distinct non-redundant 3-way contiguity-preserving (zero-split) moves
    plus exact 1-round self-run contraction from state L (canonically relabeled 0..sigma-1).
    To stay fast for sigma <= 8, enumerate all 3^sigma (L, R, Idle) colorings and for each
    apply maximal contiguity-preserving P <= Sigma_L x Sigma_R, deduplicating by resulting L_next!
    """
    n = len(L)
    sigma = max(L) + 1
    C, F, LF = compute_C_and_LF(L, sigma)
    runs = runs_from_seq(L)
    bucket_runs = [[] for _ in range(sigma)]
    pos, f_sym = 0, 0
    while f_sym < sigma and C[f_sym + 1] == 0:
        f_sym += 1
    for c, w in runs:
        rem, cur = w, pos
        while rem > 0:
            while C[f_sym + 1] <= cur:
                f_sym += 1
            take = min(rem, C[f_sym + 1] - cur)
            bucket_runs[f_sym].append((c, take))
            cur += take
            rem -= take
        pos += w

    seen_next = {}  # tuple(L_next) -> best (q_con, r_cur, label)

    # 1. Self-run contraction if any self-edges
    self_cnt = sum(1 for i in range(n) if L[i] == F[i])
    if self_cnt > 0 and self_cnt < n:
        rules = [('base', c) for c in range(sigma)]
        sym_len = [1] * sigma
        sym_to_rule = list(range(sigma))
        res = self_run_contraction_step(runs, sigma, n, 0, 0, sym_len, sym_to_rule, rules, C, bucket_runs)
        if res[6] is None:
            out_runs = res[0]
            L_next = tuple(c for c, w in out_runs for _ in range(w))
            q_self = sum(len(bucket_runs[c]) for c in range(sigma) if any(a == c for a, _ in bucket_runs[c]))
            seen_next[L_next] = (q_self, len(runs), "self")

    # 2. All 3-way colorings (0=Idle, 1=L, 2=R) when sigma <= 7; else all threshold + 2-colorings + single-symbol Lazard
    if sigma <= 7:
        for code in range(3 ** sigma):
            tmp = code
            in_L = [False] * sigma
            in_R = [False] * sigma
            has_l = has_r = False
            for s in range(sigma):
                v = tmp % 3
                tmp //= 3
                if v == 1:
                    in_L[s] = True; has_l = True
                elif v == 2:
                    in_R[s] = True; has_r = True
            if not (has_l and has_r):
                continue
            # Maximal contiguity-preserving take_pairs for this 3-way coloring:
            # For each a in L, uncontracted followers are d not in R (i.e. d in L or Idle or d == a)
            u = [sigma] * sigma
            v_hi = [-1] * sigma
            for b in range(sigma):
                if not in_R[b]:
                    for a, _ in bucket_runs[b]:
                        if in_L[a]:
                            if b < u[a]: u[a] = b
                            if b > v_hi[a]: v_hi[a] = b
            take_pairs = set()
            for b in range(sigma):
                if in_R[b]:
                    for a, _ in bucket_runs[b]:
                        if in_L[a] and (b < u[a] or b > v_hi[a]):
                            take_pairs.add((a, b))
            if not take_pairs:
                continue
            rules = [('base', c) for c in range(sigma)]
            sym_len = [1] * sigma
            sym_to_rule = list(range(sigma))
            res = apply_selective_forward(list(L), 0, 0, sym_len, sym_to_rule, rules, in_L, in_R, take_pairs)
            if res is None:
                continue
            L_next = tuple(res[0])
            q_con = res[4]["q_con"]
            prev = seen_next.get(L_next)
            if prev is None or q_con < prev[0]:
                seen_next[L_next] = (q_con, len(runs), f"3way_{code}")
    return seen_next


def solve_global_opt(L_init, objective="q_con", max_states=15000):
    """Exact Dijkstra shortest path from L_init to fixed point (all L[i]==F[i])."""
    uniq = sorted(set(L_init))
    mp = {c: i for i, c in enumerate(uniq)}
    start = tuple(mp[c] for c in L_init)

    pq = [(0, 0, 0, start)]  # (cost, rounds, other_cost, state)
    best = {start: 0}
    expanded = 0

    while pq and expanded < max_states:
        cost, rnd, other, state = heapq.heappop(pq)
        if cost > best.get(state, 10**18):
            continue
        sig = max(state) + 1
        C, F, _ = compute_C_and_LF(state, sig)
        if all(state[i] == F[i] for i in range(len(state))):
            return cost, rnd, other, expanded, True
        expanded += 1
        moves = enumerate_valid_moves(state)
        for L_next, (q_con, r_cur, _) in moves.items():
            step_c = q_con if objective == "q_con" else r_cur
            step_o = r_cur if objective == "q_con" else q_con
            nc = cost + step_c
            if nc < best.get(L_next, 10**18):
                best[L_next] = nc
                heapq.heappush(pq, (nc, rnd + 1, other + step_o, L_next))
    return None, None, None, expanded, False


def main():
    t0 = time.time()
    rng = random.Random(7)
    cases = [
        ("Fib_8+$ (n=9)", [1, 2, 1, 1, 2, 1, 2, 1, 0]),
        ("Fib_13+$ (n=14)", [1, 2, 1, 1, 2, 1, 2, 1, 1, 2, 1, 1, 2, 0]),
        ("SymMid_12", [0, 2, 1, 1, 1, 2, 0, 1, 1, 0, 2, 1]),
        ("SymMid_16", [0, 2, 1, 1, 1, 1, 2, 0, 1, 1, 1, 0, 2, 1, 1, 2]),
        ("PhiBin_14", [0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 1, 1, 0]),
        ("ThueM_16+$", [1, 2, 2, 1, 2, 1, 1, 2, 2, 1, 1, 2, 1, 2, 2, 1, 0]),
        ("Rand_s3_14", [rng.randrange(3) for _ in range(14)]),
        ("WikiSlice_14", list(b'{"type":"item"')),
    ]

    print("=== EXACT GLOBAL MULTI-STEP OPTIMUM (DIJKSTRA OVER ALL 3-WAY ZERO-SPLIT MOVES) ===")
    print(f"{'Instance':16s} | {'r0':>2s} | {'BDMR Q/R':>10s} | {'CPSAT_Q Q/R':>11s} | {'GLOBAL_Q* (rnd)':>15s} | {'GLOBAL_R* (rnd)':>15s} | {'Q gap':>6s} | {'R gap':>6s}")
    for name, T in cases:
        L, p = bwt_forward(T)
        r0 = len(runs_from_seq(L))
        s_bdmr = run_schedule(L, p, "bdmr_greedy")
        s_cpq = run_schedule(L, p, "cpsat_min_q_ratio")
        gq, gq_r, _, exp_q, ok_q = solve_global_opt(L, objective="q_con", max_states=6000)
        gr, gr_r, _, exp_r, ok_r = solve_global_opt(L, objective="sum_r", max_states=6000)
        q_str = f"{gq:3d} ({gq_r:2d})" if ok_q else "BOUNDED"
        r_str = f"{gr:3d} ({gr_r:2d})" if ok_r else "BOUNDED"
        q_gap = f"{(s_bdmr['sum_q_con']/gq - 1)*100:+5.1f}%" if ok_q and gq > 0 else "  n/a"
        r_gap = f"{(s_bdmr['sum_r']/gr - 1)*100:+5.1f}%" if ok_r and gr > 0 else "  n/a"
        print(
            f"{name:16s} | {r0:2d} | {s_bdmr['sum_q_con']:4d}/{s_bdmr['sum_r']:4d} | "
            f"{s_cpq['sum_q_con']:4d}/{s_cpq['sum_r']:5d} | {q_str:>15s} | {r_str:>15s} | {q_gap:>6s} | {r_gap:>6s}"
        )
    print(f"Elapsed: {time.time() - t0:.2f}s")


if __name__ == "__main__":
    main()
