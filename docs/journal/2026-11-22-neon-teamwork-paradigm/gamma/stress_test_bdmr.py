#!/usr/bin/env python3
"""
Adversarial stress test of Investigator Beta's BDMR (/tmp/teamwork-b/deep_analysis.py, read-only)
using Phi_BWT(pi, b, K), Phi_BWT_bin(pi, K), and classic BWT-sensitive valid BWT families.
Measures exact max_r/r_0, sum_r/r_0, max_sigma, rounds, and byte-exact recovery.
"""
import math
import random
import sys

sys.path.insert(0, "/tmp/teamwork-b")
sys.path.insert(0, "/tmp/teamwork-c")

import deep_analysis as bdmr
import verify_phi_bwt as phi


def count_runs(L):
    if not L:
        return 0
    r = 1
    for i in range(1, len(L)):
        if L[i] != L[i - 1]:
            r += 1
    return r


def fib_word(k):
    a, b = [0], [0, 1]
    for _ in range(k - 2):
        a, b = b, b + a
    return b


def thue_morse(k):
    s = [0]
    for _ in range(k):
        s = s + [1 - x for x in s]
    return s


def debruijn_bin(k):
    a = [0] * (2 * k)
    seq = []

    def db(t, p):
        if t > k:
            if k % p == 0:
                seq.extend(a[1 : p + 1])
        else:
            a[t] = a[t - p]
            db(t + 1, p)
            for j in range(a[t - p] + 1, 2):
                a[t] = j
                db(t + 1, t)

    db(1, 1)
    return seq


def bit_rev_cycle(M):
    """Single-cycle permutation visiting 0..M-1 in bit-reversed order (M power of 2)."""
    bits = M.bit_length() - 1
    order = []
    for i in range(M):
        r = 0
        x = i
        for _ in range(bits):
            r = (r << 1) | (x & 1)
            x >>= 1
        order.append(r)
    pi = [0] * M
    for i in range(M):
        pi[order[i]] = order[(i + 1) % M]
    return pi


def random_cycle(M, seed=42):
    rng = random.Random(seed)
    order = list(range(M))
    rng.shuffle(order)
    pi = [0] * M
    for i in range(M):
        pi[order[i]] = order[(i + 1) % M]
    return pi


def inv_perm(pi):
    pred = [0] * len(pi)
    for u, v in enumerate(pi):
        pred[v] = u
    return pred


def run_case(name, L, p, expected_T=None):
    n = len(L)
    r0 = count_runs(L)
    out, metrics = bdmr.decode_recomp(L, p, variant="B", collect_metrics=True)
    if expected_T is not None:
        assert out == expected_T, f"BDMR mismatch on {name}!"
    else:
        ref = bdmr.ref_inverse_bwt(L, p)
        assert out == ref, f"BDMR mismatch vs ref on {name}!"

    max_r = metrics["max_r"]
    sum_r = metrics["total_run_work"]
    max_sig = metrics["max_sigma"]
    rounds = metrics["rounds"]
    q_ind = metrics["total_indep_reads"]
    print(
        f"{name:<32} n={n:6d} r0={r0:5d} rds={rounds:2d} "
        f"max_r={max_r:5d} ({max_r/r0:5.2f}x) "
        f"sum_r/r0={sum_r/r0:5.2f}x max_sig={max_sig:5d} ({max_sig/r0:4.2f}x) Q={q_ind:6d}"
    )
    return max_r / r0, sum_r / r0, max_sig / r0


def main():
    print("=== BDMR (Variant B: Full Bucket-Split Max-Cut) Adversarial & Phi_BWT Stress Test ===\n")

    # 1. Phi_BWT(pi, b, K) — random single-cycle permutations
    for M, b, K in [(128, 2, 1), (128, 2, 16), (256, 4, 16), (512, 16, 32), (1024, 16, 32)]:
        pi = random_cycle(M, seed=100 + M)
        T, L_dig = phi.build_T_pi(pi, b, K)
        L, p = phi.tc0_construct_L_pi(inv_perm(pi), b, K, L_dig)
        run_case(f"Phi_BWT(rand M={M},b={b},K={K})", L, p, T)

    # 2. Phi_BWT with bit-reversal permutation
    for M, b, K in [(256, 2, 8), (512, 4, 16)]:
        pi = bit_rev_cycle(M)
        T, L_dig = phi.build_T_pi(pi, b, K)
        L, p = phi.tc0_construct_L_pi(inv_perm(pi), b, K, L_dig)
        run_case(f"Phi_BWT(bitrev M={M},b={b},K={K})", L, p, T)

    # 3. Binary Phi_BWT_bin (|Sigma| = 2)
    for M, K in [(64, 2), (256, 16), (512, 16)]:
        pi = random_cycle(M, seed=200 + M)
        T, L_bits = phi.build_T_pi_binary(pi, K)
        L, p = phi.tc0_construct_L_pi_binary(inv_perm(pi), K, L_bits)
        run_case(f"Phi_BWT_bin(rand M={M},K={K})", L, p, T)

    # 4. Classic BWT-sensitive / low-r0 families!
    print("\n--- Low-r0 & BWT-Blowup Families ---")
    for k in [14, 18, 21]:
        T = fib_word(k)
        L, p = bdmr.bwt_forward(T)
        run_case(f"Fibonacci F_{k}", L, p, T)

    for k in [14, 18, 21]:
        T = fib_word(k)[::-1]
        L, p = bdmr.bwt_forward(T)
        run_case(f"Rev-Fibonacci F_{k}^R", L, p, T)

    # Perturbed Fibonacci: 1-bit flip in middle of Fibonacci word (Giuliani et al. r-sensitivity!)
    for k in [16, 20]:
        T = fib_word(k)
        T[len(T) // 2] ^= 1
        L, p = bdmr.bwt_forward(T)
        run_case(f"1-edit Fibonacci F_{k}", L, p, T)

    for k in [10, 12, 14]:
        T = thue_morse(k)
        L, p = bdmr.bwt_forward(T)
        run_case(f"Thue-Morse TM_{k}", L, p, T)

    for k in [8, 10, 12]:
        T = debruijn_bin(k)
        L, p = bdmr.bwt_forward(T)
        run_case(f"DeBruijn(2,{k})", L, p, T)

    # Ramp word: b a^1 b a^2 ... b a^m
    for m_ramp in [40, 100]:
        T = []
        for j in range(1, m_ramp + 1):
            T.append(1)
            T.extend([0] * j)
        L, p = bdmr.bwt_forward(T)
        run_case(f"Ramp b a^1..b a^{m_ramp}", L, p, T)

    # Random binary & ternary & byte words
    rng = random.Random(777)
    for sig, n_rnd in [(2, 8192), (4, 8192), (16, 8192)]:
        T = [rng.randrange(sig) for _ in range(n_rnd)]
        L, p = bdmr.bwt_forward(T)
        run_case(f"Random(sig={sig},n={n_rnd})", L, p, T)

    # Adversarial multi-step Fibonacci-like + Christoffel / Sturmian with high slope
    # Standard word with continued fraction [1, 1, ..., 1, 100] (r0 = 2!)
    for m_tail in [16, 64]:
        # Standard Sturmian word for (F_14, F_13) modified: or simple (0^{m_tail} 1) + 1 edit!
        T = ([0] * m_tail + [1]) * 64
        T[0] = 1  # break periodicity with 1 edit!
        L, p = bdmr.bwt_forward(T)
        run_case(f"1-edit (0^{m_tail} 1)^64", L, p, T)

    # Adversarial Giuliani et al. (2021) family: s_k = F_k with last two chars swapped / appended $
    for k in [14, 18, 21]:
        T = fib_word(k) + [2]  # append sentinel $ -> r0 = O(1), rev has O(log n)
        L, p = bdmr.bwt_forward(T)
        run_case(f"Fib_{k}+$", L, p, T)

    # Adversarial "Fibonacci-interleaved" / nested period word:
    # Concatenation of distinct powers of Fibonacci / nested a^i b^j
    # Try to maximize max_r / r0 by making pair-contraction split long runs across many rounds!
    for d in [10, 12, 14]:
        # Standard word of slope (p, q) with large partial quotients? Actually try binary Gray / paperfolding!
        # Paperfolding sequence of length 2^d - 1:
        seq = []
        for i in range(1, 1 << d):
            tz = (i & -i).bit_length() - 1
            seq.append(((i >> (tz + 1)) & 1))
        L, p = bdmr.bwt_forward(seq)
        run_case(f"Paperfolding 2^{d}-1", L, p, seq)


if __name__ == "__main__":
    main()
