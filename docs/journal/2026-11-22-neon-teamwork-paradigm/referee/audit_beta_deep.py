#!/usr/bin/env python3
"""Deep falsifier for Beta's BDMR:
1. Search for VALID BWTs that maximize max_r / r_0 and sum_r / r_0.
2. Check whether Theta(r^2) stack_pops in self_run_contraction_step happens on VALID BWTs,
   AND whether landed_pieces (actual output runs!) can ALSO be Omega(r^2)!
3. Measure all 6 Wikidata blocks on Variant B (greedy) and Variant B-Parallel and check claimed numbers.
"""
import random
import sys
from bisect import bisect_right

sys.path.insert(0, "/tmp/teamwork-b")
import deep_analysis as db
import measure_costs_detail as dm


def test_bdmr_max_r_attack():
    print("=== 1. ATTACKING max_r / r_0 ON VALID BWTs IN BDMR (Variant B & B-Parallel) ===")
    # Idea 1: Small r_0 strings with many distinct characters or structured runs
    # Try: Sturmian words (slope p/q), Fibonacci, Tribonacci, Thue-Morse, Episturmian,
    # repeated short patterns with perturbations, random BWTs with small r_0!
    candidates = []

    # 1a. Tribonacci word (r_0 is tiny!)
    a, b, c = [0], [0, 1], [0, 1, 0, 2]
    while len(c) < 20000:
        a, b, c = b, c, c + b + a
    candidates.append((f"Tribonacci (n={len(c)})", c))

    # 1b. Episturmian 4-letter word: s_0 = [0], s_{k+1} = s_k + [k%4] + s_k
    s = [0]
    for k in range(1, 14):
        s = s + [k % 4] + s
    candidates.append((f"Episturmian-4 (n={len(s)})", s))

    # 1c. Episturmian 8-letter word:
    s8 = [0]
    for k in range(1, 14):
        s8 = s8 + [k % 8] + s8
    candidates.append((f"Episturmian-8 (n={len(s8)})", s8))

    # 1d. Concatenation of many different Sturmian / Fibonacci rotations or power-free words
    # Look at what happens when we construct T with many runs straddling bucket boundaries!
    # Try: T = (0 1)^k (0 2)^k (0 3)^k ... (0 m)^k
    T_star = []
    for sym in range(1, 33):
        T_star.extend([0, sym] * 200)
    candidates.append((f"Star(0,1..32)^200 (n={len(T_star)})", T_star))

    # 1e. Random sparse edits on a periodic word: (0 1 2 3)^4000 with 20 random edits
    rng = random.Random(2026)
    T_ed = [i % 4 for i in range(16000)]
    for _ in range(20):
        T_ed[rng.randrange(len(T_ed))] = rng.randrange(4, 12)
    candidates.append((f"Per4+20edits (n={len(T_ed)})", T_ed))

    # 1f. Multi-scale nested runs: (a^1 b^1) (a^2 b^2) ... (a^k b^k)
    T_nest = []
    for k in range(1, 100):
        T_nest.extend([0] * k + [1] * k)
    candidates.append((f"Nested a^k b^k (n={len(T_nest)})", T_nest))

    # 1g. Middle-run attack from Alpha's falsifier!
    low_tags = [[(i // 8), (i % 8)] for i in range(64)]
    high_tags = [[10 + (i // 8), 10 + (i % 8)] for i in range(64)]
    rng.shuffle(low_tags)
    rng.shuffle(high_tags)
    T_mid = []
    for i in range(64):
        T_mid.extend(low_tags[i] + [8] * 64 + high_tags[i])
    candidates.append((f"Middle-run K=64 (n={len(T_mid)})", T_mid))

    # 1h. Random walk over small alphabet with long stay prob (Markov runs)
    T_mk = []
    cur = 0
    for _ in range(20000):
        if rng.random() < 0.02:
            cur = rng.randrange(16)
        T_mk.append(cur)
    candidates.append((f"Markov-runs16 (n={len(T_mk)})", T_mk))

    # 1i. Exhaustive search on small valid BWTs (n=1..12 binary, n=1..8 ternary) for MAX (max_r / r_0)!
    best_ratio = 0.0
    best_small = None
    for n in range(4, 13):
        for mask in range(1 << n):
            T = [(mask >> i) & 1 for i in range(n)]
            L, p = db.bwt_forward(T)
            r0 = len(db.runs_from_seq(L))
            _, st = db.decode_recomp(L, p, variant="B")
            ratio = st["max_r"] / r0
            if ratio > best_ratio:
                best_ratio = ratio
                best_small = (T, r0, st["max_r"], st["rounds"], st["total_run_work"] / r0)
    print(f"  Exhaustive binary n<=12 worst max_r/r0 = {best_ratio:.2f}x: T={best_small[0]} (r0={best_small[1]}, max_r={best_small[2]}, rounds={best_small[3]}, sum_r/r0={best_small[4]:.2f}x)")

    for name, T in candidates:
        L, p = db.bwt_forward(T)
        r0 = len(db.runs_from_seq(L))
        got, st = db.decode_recomp(L, p, variant="B", collect_metrics=True)
        assert got == T, f"MISMATCH on {name}"
        print(f"  {name:30s} | r0={r0:5d} | rnd={st['rounds']:2d}(s={st['self_rounds']:2d}) | "
              f"max_r/r0={st['max_r']/r0:5.2f}x | sum_r/r0={st['total_run_work']/r0:6.2f}x | "
              f"max_sig={st['max_sigma']:4d} | rules/r0={st['total_rules']/r0:5.2f}x | h={st['slp_height']:2d}")


def test_self_run_valid_bwt_quadratic():
    print("\n=== 2. TESTING SELF-RUN CONTRACTION ON VALID BWT WITH MANY SELF-RUNS (Nested a^k b_i) ===")
    # Construct a VALID BWT where symbol 0 has MANY self-runs inside B_0!
    # Take T = 0^1 1 0^2 2 0^3 3 ... 0^q q (or over binary alphabet: T = 0^1 1 0^2 1 0^3 1 ... 0^q 1)!
    # In binary T = 0^1 1 0^2 1 0^3 1 ... 0^q 1:
    # Rotations starting with 0^k 1 ... are sorted in DECREASING order of k!
    # Wait: check how many runs B_0 has, and how many stack pops self_run_contraction_step takes!
    for q in [20, 40, 80, 160]:
        # Make T with distinct suffixes after 0^k using binary tags so B_0 has alternating 0 and 1!
        T = []
        for k in range(1, q + 1):
            T.extend([0] * k + [1, 1] + ([0, 1] if (k % 2 == 0) else [1, 0]))
        L, p = db.bwt_forward(T)
        runs = db.runs_from_seq(L)
        r0 = len(runs)
        n = len(L)
        c0 = L.count(0)
        C = [0, c0, n]
        bucket_runs = [db.runs_from_seq(L[:c0]), db.runs_from_seq(L[c0:])]
        # Instrument self_run_contraction_step on bucket 0 (c=0, u=0! Wait: u=0 means ALL self-runs are delta <= 0!)
        # For c=1, u = count of 1 in B_0 > 0, let's instrument BOTH c=0 and c=1!
        tot_pops = 0
        tot_landed = 0
        for c in (0, 1):
            Lc = C[c + 1] - C[c]
            u = sum(w for a, w in bucket_runs[0] if a == c) if c == 1 else 0
            m = sum(w for a, w in bucket_runs[c] if a == c)
            if m == 0:
                continue
            v = u + m
            d_runs, s_runs = [], []
            off, t_cur = 0, u
            for a, w in bucket_runs[c]:
                if a == c:
                    d_runs.append((off, off + w, t_cur - off))
                    t_cur += w
                else:
                    s_runs.append((off, off + w))
                off += w
            all_ivs = sorted([(l, r, 'S', 0) for l, r in s_runs] + [(l, r, 'D', d) for l, r, d in d_runs])
            iv_ends = [iv[1] for iv in all_ivs]
            stack = []
            if u > 0: stack.append((0, u, 1))
            if v < Lc: stack.append((v, Lc, 1))
            pops = 0
            landed = 0
            while stack:
                lo, hi, k = stack.pop()
                pops += 1
                idx = bisect_right(iv_ends, lo)
                cur = lo
                while cur < hi:
                    l, r, tp, d = all_ivs[idx]
                    seg_hi = hi if hi < r else r
                    if tp == 'S':
                        landed += 1
                    else:
                        if d > 0:
                            q_lo = (r - 1 - cur) // d + 1
                            q_hi = (r - 1 - (seg_hi - 1)) // d + 1
                            if q_lo == q_hi:
                                stack.append((cur + q_lo * d, seg_hi + q_lo * d, k + q_lo))
                            else:
                                split = r - (q_lo - 1) * d
                                stack.append((cur + q_lo * d, split + q_lo * d, k + q_lo))
                                stack.append((split + q_hi * d, seg_hi + q_hi * d, k + q_hi))
                        else:
                            ad = -d
                            q_lo = (cur - l) // ad + 1
                            q_hi = (seg_hi - 1 - l) // ad + 1
                            if q_lo == q_hi:
                                stack.append((cur - q_lo * ad, seg_hi - q_lo * ad, k + q_lo))
                            else:
                                split = l + q_lo * ad
                                stack.append((cur - q_lo * ad, split - q_lo * ad, k + q_lo))
                                stack.append((split - q_hi * ad, seg_hi - q_hi * ad, k + q_hi))
                    cur = seg_hi
                    idx += 1
            tot_pops += pops
            tot_landed += landed
        print(f"  Valid BWT nested-0^k q={q:3d}: n={n:5d}, r0={r0:4d}, stack_pops={tot_pops:6d} (pops/r0={tot_pops/r0:6.2f}), landed={tot_landed:4d}")

    # Also test: Can a VALID BWT achieve quadratic stack_pops on a MIDDLE symbol (c = 1 in {0, 1, 2})?
    # Build T with symbol '1' having runs 1^k preceded by '0' (for some) and '2' (for others) so u > 0 AND v < Lc!
    print("\n  Testing VALID BWT middle-symbol self-run (c=1 in {0,1,2}):")
    for q in [20, 40, 80]:
        # Make T = concat_{k=1..q} (0 1^k 2 1 0 1 2) -> creates interleaved 1s and 0s/2s in B_1!
        T = []
        for k in range(1, q + 1):
            # Vary what follows 1^k so rotations interleave!
            T.extend([0] + [1] * k + [2, 0] + [1] * k + [0, 2])
        L, p = db.bwt_forward(T)
        runs = db.runs_from_seq(L)
        r0 = len(runs)
        n = len(L)
        cnts = [L.count(0), L.count(1), L.count(2)]
        C = [0, cnts[0], cnts[0] + cnts[1], n]
        bucket_runs = [db.runs_from_seq(L[C[c]:C[c+1]]) for c in range(3)]
        c = 1
        Lc = C[2] - C[1]
        u = sum(w for a, w in bucket_runs[0] if a == 1)
        m = sum(w for a, w in bucket_runs[1] if a == 1)
        v = u + m
        d_runs, s_runs = [], []
        off, t_cur = 0, u
        for a, w in bucket_runs[1]:
            if a == 1:
                d_runs.append((off, off + w, t_cur - off))
                t_cur += w
            else:
                s_runs.append((off, off + w))
            off += w
        all_ivs = sorted([(l, r, 'S', 0) for l, r in s_runs] + [(l, r, 'D', d) for l, r, d in d_runs])
        iv_ends = [iv[1] for iv in all_ivs]
        stack = []
        if u > 0: stack.append((0, u, 1))
        if v < Lc: stack.append((v, Lc, 1))
        pops = 0
        landed = 0
        while stack:
            lo, hi, k = stack.pop()
            pops += 1
            idx = bisect_right(iv_ends, lo)
            cur = lo
            while cur < hi:
                l, r, tp, d = all_ivs[idx]
                seg_hi = hi if hi < r else r
                if tp == 'S':
                    landed += 1
                else:
                    if d > 0:
                        q_lo = (r - 1 - cur) // d + 1
                        q_hi = (r - 1 - (seg_hi - 1)) // d + 1
                        if q_lo == q_hi:
                            stack.append((cur + q_lo * d, seg_hi + q_lo * d, k + q_lo))
                        else:
                            split = r - (q_lo - 1) * d
                            stack.append((cur + q_lo * d, split + q_lo * d, k + q_lo))
                            stack.append((split + q_hi * d, seg_hi + q_hi * d, k + q_hi))
                    else:
                        ad = -d
                        q_lo = (cur - l) // ad + 1
                        q_hi = (seg_hi - 1 - l) // ad + 1
                        if q_lo == q_hi:
                            stack.append((cur - q_lo * ad, seg_hi - q_lo * ad, k + q_lo))
                        else:
                            split = l + q_lo * ad
                            stack.append((cur - q_lo * ad, split - q_lo * ad, k + q_lo))
                            stack.append((split - q_hi * ad, seg_hi - q_hi * ad, k + q_hi))
                cur = seg_hi
                idx += 1
        print(f"    Valid BWT mid-1^k q={q:3d}: n={n:5d}, r0={r0:4d}, B_1_runs={len(bucket_runs[1]):4d}, stack_pops={pops:6d} (pops/B1r={pops/len(bucket_runs[1]):6.2f}), landed={landed:4d}")


def test_wikidata_beta():
    print("\n=== 3. VERIFYING BETA'S CLAIMED NUMBERS ON ALL 6 WIKIDATA BLOCKS ===")
    dm.main()


if __name__ == "__main__":
    test_bdmr_max_r_attack()
    test_self_run_valid_bwt_quadratic()
    test_wikidata_beta()
