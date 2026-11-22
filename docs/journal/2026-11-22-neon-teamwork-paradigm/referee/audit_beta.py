#!/usr/bin/env python3
"""Adversarial Referee Audit of Investigator Beta: BDMR (Pair + Self-Run Contraction)."""
import itertools
import random
import sys

sys.path.insert(0, "/tmp/teamwork-b")
import deep_analysis as db


def audit_self_run_adversarial():
    print("=== 1. AUDITING BETA: SELF-RUN CONTRACTION ADVERSARIAL NON-BWT COLUMNS ===")
    # 1a. Columns where B_c has delta > 0, delta == 0 (fixed point in MIDDLE!), AND delta < 0 in the SAME bucket!
    # Build B_c (for c = 1 in alphabet {0, 1, 2}):
    # Suppose count(0) = 2, count(1) = 10, count(2) = 2 -> B_1 = [2, 12).
    # Put 2 copies of '1' before B_1 (in B_0), so u = 2!
    # Inside B_1 (length 10, 0-based [0..10)):
    #   idx 0: 1 (delta = 2 - 0 = +2 > 0)
    #   idx 1: 0 (S)
    #   idx 2: 2 (S)
    #   idx 3: 1 (delta = 3 - 3 = 0 -> FIXED POINT IN MIDDLE!)
    #   idx 4: 1 (delta = 4 - 4 = 0 -> FIXED POINT IN MIDDLE!)
    #   idx 5: 0 (S)
    #   idx 6: 2 (S)
    #   idx 7: 1 (delta = 5 - 7 = -2 < 0)
    #   idx 8: 1 (delta = 6 - 8 = -2 < 0)
    #   idx 9: 1 (delta = 7 - 9 = -2 < 0)
    # Total '1' in B_1: m = 6 -> [u, v) = [2, 8).
    # Remaining '1' after B_1 (in B_2): 10 - 2 - 6 = 2 copies of '1'!
    # Check counts:
    #   B_0 = [1, 1] (2 copies of 1)
    #   B_1 = [1, 0, 2, 1, 1, 0, 2, 1, 1, 1] (6 copies of 1, 2 of 0, 2 of 2)
    #   B_2 = [1, 1] (2 copies of 1)
    # Total counts: 0: 2, 1: 10, 2: 2! Exact!
    L_mix = [1, 1] + [1, 0, 2, 1, 1, 0, 2, 1, 1, 1] + [1, 1]
    assert len(L_mix) == 14
    for p in range(len(L_mix)):
        ref = db.ref_inverse_bwt(L_mix, p)
        for var in ("A", "B"):
            got, _ = db.decode_recomp(L_mix, p, variant=var)
            assert got == ref, f"FAIL L_mix p={p} var={var}: {got} != {ref}"
    print("[PASS] Mixed-sign + middle-fixed-point bucket (delta > 0, delta == 0, delta < 0 in same B_c): all 14 p PASSED!")

    # 1b. Test ALL 1000 random multi-cycle columns with heavy self-runs (alphabet 3, 4, 5)
    rng = random.Random(999)
    for trial in range(1000):
        n = rng.randint(3, 25)
        sig = rng.choice([2, 3, 4, 5])
        L = [rng.randrange(sig) for _ in range(n)]
        p = rng.randrange(n)
        ref = db.ref_inverse_bwt(L, p)
        for var in ("A", "B"):
            got, _ = db.decode_recomp(L, p, variant=var)
            assert got == ref, f"FAIL random arb L={L} p={p} var={var}"
    print("[PASS] 1000 random multi-cycle arbitrary columns (|Sigma| in 2..5) PASSED Variants A and B!")

    # 1c. QUADRATIC BLOW-UP ATTACK on self_run_contraction_step!
    # Construct a column with ONE bucket B_1 having r_B runs where self_run_contraction_step
    # creates Omega(r_B^2) stack pops OR Omega(r_B^2) landed pieces!
    print("\n=== 2. QUADRATIC BLOW-UP ATTACK ON BETA's self_run_contraction_step ===")
    for q in [10, 20, 40, 80]:
        # Build B_1 of length Lc:
        # Step 1: u = 2*q (put 2*q copies of '1' in B_0).
        # First 2*q positions of B_1: alternating (1, 0) repeated q times -> q D-runs of len 1, q S-runs of len 1.
        #   The q D-elements map to [2q, 3q) as q SEPARATE stack items of width 1!
        # Next: q blocks of (D-run of len q-1, S-run of len 1)? Wait:
        #   After the first 2q positions, delta = (2q + q) - 2q = q!
        #   Each time we put a D-run of len (d - 1) and an S-run of len 1:
        #   Actually just put D-runs of len 1 separated by S-runs of len 0? No, consecutive D-runs merge.
        #   Put D-run of len (q) followed by S-run of len 1, repeated q//2 times!
        #   Wait: if we put (D of len 1, S of len 1) repeated q times, then (D of len q, S of len 1) repeated q//2 times:
        #   Wait: after first 2q positions, delta = q. Each S of len 1 reduces delta by 1!
        #   So put (D of len q, S of len 1) repeated q-1 times -> each of the q-k surviving items hits every D-run!
        B1 = []
        # Part 1: q times (1, 0) -> creates q separate 1-element intervals on stack landing at [2q, 3q)
        for _ in range(q):
            B1.extend([1, 0])
        # Part 2: q-1 times (D of len d, S of len 1) where d shrinks from q down to 2
        for d in range(q, 1, -1):
            B1.extend([1] * d)
            B1.append(0)
        # Remaining S of len 1 at the end so all trajectories terminate
        B1.append(0)
        # Count 0s and 1s in B1:
        n0_in_B1 = B1.count(0)
        n1_in_B1 = B1.count(1)
        # Need count(0) == 2*q (so B_0 has len 2*q, all filled with '1's so u = 2*q!)
        # Right now n0_in_B1 = q + (q - 1) + 1 = 2*q! EXACT!!
        assert n0_in_B1 == 2 * q
        # Need total count(1) == len(B1), currently 2*q (in B0) + n1_in_B1.
        # Put extra '2's in B1? No, we need len(B1) == 2*q + n1_in_B1 + n1_in_B2.
        # Since len(B1) = n0_in_B1 + n1_in_B1 = 2*q + n1_in_B1, we need n1_in_B2 = 0 -> NO B_2 NEEDED!!
        L_quad = [1] * (2 * q) + B1
        assert len(L_quad) == 2 * q + len(B1)
        assert L_quad.count(0) == 2 * q and L_quad.count(1) == len(B1)

        # Instrument self_run_contraction_step on L_quad!
        runs = db.runs_from_seq(L_quad)
        r0 = len(runs)
        sigma = 2
        n = len(L_quad)
        C = [0, 2 * q, n]
        bucket_runs = [db.runs_from_seq(L_quad[:2 * q]), db.runs_from_seq(B1)]
        rules = [('base', 0), ('base', 1)]
        sym_len = [1, 1]
        sym_to_rule = [0, 1]

        # Count exact stack pops by instrumenting directly:
        Lc = len(B1)
        u = 2 * q
        m = n1_in_B1
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
        from bisect import bisect_right
        stack = [(0, u, 1)]
        pops = 0
        landed = []
        while stack:
            lo, hi, k = stack.pop()
            pops += 1
            idx = bisect_right(iv_ends, lo)
            cur = lo
            while cur < hi:
                l, r, tp, d = all_ivs[idx]
                seg_hi = hi if hi < r else r
                if tp == 'S':
                    landed.append((cur, seg_hi, k))
                else:
                    q_lo = (r - 1 - cur) // d + 1
                    q_hi = (r - 1 - (seg_hi - 1)) // d + 1
                    if q_lo == q_hi:
                        stack.append((cur + q_lo * d, seg_hi + q_lo * d, k + q_lo))
                    else:
                        split = r - (q_lo - 1) * d
                        stack.append((cur + q_lo * d, split + q_lo * d, k + q_lo))
                        stack.append((split + q_hi * d, seg_hi + q_hi * d, k + q_hi))
                cur = seg_hi
                idx += 1

        # Also run Beta's actual self_run_contraction_step and verify exactness!
        res = db.self_run_contraction_step(runs, sigma, n, 0, 0, sym_len, sym_to_rule, rules, C, bucket_runs)
        out_runs = res[0]
        got, _ = db.decode_recomp(L_quad, 0, variant="B")
        assert got == db.ref_inverse_bwt(L_quad, 0)
        print(f"  q={q:2d}: n={n:5d}, r0={r0:3d}, stack_pops={pops:5d} (pops/r0={pops/r0:5.2f}, pops/q^2={pops/(q*q):.3f}), "
              f"landed_pieces={len(landed)}, out_runs={len(out_runs)}")


def audit_max_r_blowup():
    print("\n=== 3. WORST-CASE max_r / r_0 AND sum_r / r_0 ATTACK ON VALID BWTs ===")
    cases = []

    # 3a. Fibonacci F_22
    a, b = [0], [0, 1]
    while len(b) < 28000:
        a, b = b, b + a
    cases.append((f"Fibonacci (n={len(b)})", b))

    # 3b. Thue-Morse n=16384
    tm = [0]
    while len(tm) < 16384:
        tm = tm + [1 - x for x in tm]
    cases.append((f"Thue-Morse (n={len(tm)})", tm))

    # 3c. Periodic-with-tiny-perturbation (very small r0, large n!)
    # (0 1 2 ... 63)^500 with 1 byte flipped
    T_pp = [i % 64 for i in range(64 * 500)]
    T_pp[0] = 64
    cases.append((f"Near-periodic 64 (n={len(T_pp)})", T_pp))

    # 3d. Gamma's Phi_BWT (single-cycle random permutation)
    sys.path.insert(0, "/tmp/teamwork-c")
    import verify_phi_bwt as vp
    rng = random.Random(777)
    for M, b_base, K in [(500, 4, 16), (1000, 16, 32)]:
        perm = list(range(M))
        rng.shuffle(perm)
        pi = [0] * M
        for i in range(M):
            pi[perm[i]] = perm[(i + 1) % M]
        T_phi, _ = vp.build_T_pi(pi, b_base, K)
        cases.append((f"Phi_BWT(M={M},b={b_base},K={K},n={len(T_phi)})", T_phi))

    # 3e. Binary de Bruijn sequence of order 13 (n=8192)
    def debruijn_bin(k):
        a = [0] * (2 * k)
        seq = []
        def db_rec(t, p):
            if t > k:
                if k % p == 0:
                    seq.extend(a[1:p + 1])
            else:
                a[t] = a[t - p]
                db_rec(t + 1, p)
                for j in range(a[t - p] + 1, 2):
                    a[t] = j
                    db_rec(t + 1, t)
        db_rec(1, 1)
        return seq
    db13 = debruijn_bin(13)
    cases.append((f"DeBruijn(2,13) (n={len(db13)})", db13))

    # 3f. Highly repetitive LZ/grammar-compressible word: copy-with-edits (32 copies of 1024-byte random block, 1 edit each)
    base_blk = [rng.randrange(16) for _ in range(1024)]
    T_rep = []
    for k in range(32):
        blk = base_blk[:]
        blk[(k * 31) % 1024] = (blk[(k * 31) % 1024] + 1) % 16
        T_rep.extend(blk)
    cases.append((f"32x1KB-1edit (n={len(T_rep)})", T_rep))

    # 3g. Power of a random word of len 100 with 1 sentinel at end: W^200 + '$'
    W = [rng.randrange(1, 17) for _ in range(100)]
    T_pow = W * 200 + [0]
    cases.append((f"Rand100^200+$ (n={len(T_pow)})", T_pow))

    # 3h. Repeatedly doubled random-reversal (palindromic / low-r)
    S_pal = [rng.randrange(4) for _ in range(16)]
    for k in range(10):
        S_pal = S_pal + [4 + (k % 4)] + S_pal[::-1]
    cases.append((f"Palindromic-fold (n={len(S_pal)})", S_pal))

    for name, T in cases:
        L, p = db.bwt_forward(T)
        r0 = len(db.runs_from_seq(L))
        for var in ("B",):
            got, st = db.decode_recomp(L, p, variant=var, collect_metrics=True)
            assert got == T, f"MISMATCH on {name}!"
            print(f"  {name:34s} | r0={r0:5d} | rounds={st['rounds']:2d} (self={st['self_rounds']:2d}) | "
                  f"max_r/r0={st['max_r']/r0:5.2f}x | sum_r/r0={st['total_run_work']/r0:6.2f}x | "
                  f"max_sig={st['max_sigma']:5d} | rules/r0={st['total_rules']/r0:5.2f}x | h={st['slp_height']:2d}")


if __name__ == "__main__":
    audit_self_run_adversarial()
    audit_max_r_blowup()
