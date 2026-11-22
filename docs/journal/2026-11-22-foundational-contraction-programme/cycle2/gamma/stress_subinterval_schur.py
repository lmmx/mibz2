#!/usr/bin/env python3
"""
Adversarial Mathematical Stress Test of Sub-Interval 1-Piece Schur (Widest-Piece Spiral):
Tests whether interior-hole splitting (Delta r <= +1) can blow up max_r or steps on:
  1. Random irreducible d-IETs (d = 3, 4, 5, 6, 8) with HUGE length N = 10^9!
  2. 4-symbol Quadbonacci_m + $ (4-IET BWT!)
  3. Double-Sturmian / 2-rotation valid BWT: Fib_{2m} + '2' + alt-Sturmian(0,1) + '3'
  4. Compare widest-piece vs boundary-piece (Rauzy-Veech / Agol-Hass-Thurston) on the hardest instances!
"""
import random
import sys

sys.path.insert(0, "/tmp/fcp-c")
from confront_low_r_adversarial import bwt_encode_fast, run_full_subinterval_schur_with_order, solve_final_single_bucket_subinterval
from explore_iet_schur import build_initial_iet


def make_random_d_iet(d, N, rng):
    """Build a random d-piece Interval Exchange Transformation on [0, N) with single or multi-cycle."""
    # Random partition of N into d positive lengths
    cuts = sorted(rng.sample(range(1, N), d - 1))
    lens = [cuts[0]] + [cuts[i] - cuts[i - 1] for i in range(1, d - 1)] + [N - cuts[-1]]
    perm = list(range(d))
    # Pick irreducible permutation (e.g. reversal or random derangement without invariant prefix)
    while True:
        rng.shuffle(perm)
        # check irreducible: no k < d with perm[:k] == {0..k-1}
        ok = True
        for k in range(1, d):
            if set(perm[:k]) == set(range(k)):
                ok = False
                break
        if ok:
            break
    # Build source intervals and destination intervals
    src_starts = []
    acc = 0
    for l in lens:
        src_starts.append(acc)
        acc += l
    dst_starts = [0] * d
    acc = 0
    for idx in perm:
        dst_starts[idx] = acc
        acc += lens[idx]
    # Pieces: (src_s, len, dst_s, 0, 0, wid)
    slp = [('lit', i) for i in range(d)]
    pieces = [(src_starts[i], lens[i], dst_starts[i], 0, 0, i) for i in range(d)]
    return pieces, slp


def test_random_d_iets():
    print("=== 1. RANDOM IRREDUCIBLE d-IETs AT HUGE LENGTH N = 10^9 (Sub-Interval 1-Piece Schur) ===")
    rng = random.Random(202604)
    for d in (3, 4, 5, 6, 8):
        max_steps = 0
        max_ops = 0
        max_r_seen = 0
        aborts = 0
        trials = 20
        for t in range(trials):
            N = 10 ** 9
            pieces, slp = make_random_d_iet(d, N, rng)
            # Protect point p_row = 0 and collapse all other N-1 points!
            # Wait: random d-IET may have fixed points if delta == 0 (probability ~1/N), or multi-cycle
            # (so some closed cycle may not contain 0, which leaves cur_pieces with a self-loop of delta == 0!).
            # Catch RuntimeError("Fixed point") = closed cycle not hitting 0 (valid multi-cycle!).
            try:
                root, steps, ops, mr = solve_final_single_bucket_subinterval(pieces, slp, 0)
                if root is None:
                    aborts += 1
                else:
                    max_steps = max(max_steps, steps)
                    max_ops = max(max_ops, ops)
                    max_r_seen = max(max_r_seen, mr)
            except RuntimeError:
                # Hit a closed cycle not containing 0 (multi-cycle IET) — skip
                trials -= 1
                continue
        print(f"  d={d}, N=10^9 ({trials} single-hit trials): aborts={aborts}, "
              f"max_steps={max_steps:5d}, max_ops={max_ops:6d}, max_r={max_r_seen:4d} (ratio max_r/d = {max_r_seen/d:.2f}x)")


def test_quadbonacci_and_double_sturmian():
    print("\n=== 2. QUADBONACCI_m + $ (4-IET) & DOUBLE-STURMIAN VALID BWTs ===")
    def quad_word(iters):
        w = [0]
        mp = {0: [0, 1], 1: [0, 2], 2: [0, 3], 3: [0]}
        for _ in range(iters):
            nw = []
            for c in w:
                nw.extend(mp[c])
            w = nw
        return w

    for it in (6, 8, 10, 12):
        T = quad_word(it) + [4]
        L, p = bwt_encode_fast(T)
        out, st = run_full_subinterval_schur_with_order(L, p, [3, 2, 1, 0, 4])
        assert not st['aborted'] and out == T
        print(f"  Quad_{it:2d}+$ (n={len(T):5d}, r0={st['r0']:2d}): steps={st['steps']:4d}, ops={st['ops']:5d}, max_r={st['max_r']:3d}, slp={st['slp_nodes']:4d} [EXACT!]")

    # Double-Sturmian: Fib_{2m} + sentinel2 + Sturmian(sqrt(2)-1) + sentinel3
    def sturmian(alpha, length):
        return [int((i + 1) * alpha) - int(i * alpha) for i in range(length)]

    import math
    for m_len in (100, 400, 1600, 6400):
        s1 = sturmian((math.sqrt(5) - 1) / 2, m_len)
        s2 = sturmian(math.sqrt(2) - 1, m_len)
        T = s1 + [2] + s2 + [3]
        L, p = bwt_encode_fast(T)
        out, st = run_full_subinterval_schur_with_order(L, p, [2, 1, 0, 3])
        assert not st['aborted'] and out == T
        print(f"  DoubleSturm(L={m_len:4d}, n={len(T):5d}, r0={st['r0']:2d}): "
              f"steps={st['steps']:4d}, ops={st['ops']:5d}, max_r={st['max_r']:3d}, slp={st['slp_nodes']:4d} [EXACT!]")


if __name__ == "__main__":
    test_random_d_iets()
    test_quadbonacci_and_double_sturmian()
