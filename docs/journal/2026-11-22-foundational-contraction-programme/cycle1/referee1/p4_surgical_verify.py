#!/tmp/solver-venv/bin/python
"""
FCP Cycle 1 — Part 4: Surgical Verification & Extraction of Exact Witnesses:
  1. Print the exact 1-transition Farkas witness (rle_10) for CORE_10 on maxcut_and_self,
     and the 2-transition Farkas witness (rle_12, rle_18) for CORE_10 on thresh_and_self / all_bdmr!
     Verify whether f(L1) - f(L0) == f(B) (or positive combination) holds EXACTLY!
  2. Test whether Phi_self = r + e_2_self holds on ALL self-run contractions (and hunt via CP-SAT/random for counterexample!).
  3. Test whether RICH_24 / GLOBAL_26 maxcut_and_self & all_bdmr break on larger random RLE words + Fib_20 +$!
  4. Adversarial worst-case piece-count check on 2-Sweep Topological Ascent-DAG DP:
     Can a single run of L=a slice across many P[b] segments so fwd_pieces / r_0 -> infinity?
"""
import json
import random
import sys
import time
from fractions import Fraction

from ortools.sat.python import cp_model
import z3

sys.path.insert(0, "/tmp/fcp-ref")
import p1_potential_lp as p1
import p1b_deep_potential_farkas as p1b
import p2b_dag_dp_wikidata as p2b


def inspect_farkas_witnesses():
    print("=== 1. EXACT EXTRACTION OF 1-TRANSITION & 2-TRANSITION FARKAS CERTIFICATES ON CORE_10 ===")
    # Reproduce the exact rle_0..24 from p3 (same seed 999!)
    rng = random.Random(999)
    # Advance rng identically to p3!
    for n in [14, 20, 30, 45]:
        for s in [2, 3, 4, 5]:
            for idx in range(5):
                w = tuple(rng.randrange(s) for _ in range(n))
    rle_words = {}
    for idx in range(25):
        s = rng.choice([2, 3, 4])
        T = []
        for _ in range(rng.randint(6, 16)):
            c = rng.randrange(s)
            T.extend([c] * rng.randint(1, 8))
        T = tuple(T)
        if len(set(T)) >= 2 and p1.is_primitive(T):
            rle_words[f"rle_{idx}"] = T

    B_core = ["r", "r_circ", "sigma", "r_F", "E_RG", "E_RG_inv", "e_2", "e_2_self", "r_1", "r_F1"]
    for name, in_L in [("rle_10", [False, False, True]), ("rle_12", [False, False, False, True]), ("rle_18", [False, False, False, True])]:
        T = rle_words[name]
        L, F, LF, C, p, s = p1.bwt_of_word(T)
        f0 = p1.compute_features(L, F, LF, C, p, s, include_text_feats=True)
        res = p1.apply_bdmr_pair(L, F, LF, C, p, s, in_L)
        f1 = p1.compute_features(*p1.bwt_from_L_only(res[0], 0), include_text_feats=True)
        diff = {k: f1[k] - f0[k] for k in B_core}
        # Also verify if in_L is truly maxcut for rle_10
        W = [[0] * s for _ in range(s)]
        for l, f in zip(L, F):
            if l != f:
                W[l][f] += 1
        max_dc = max(
            sum(W[a][b] for a in range(s) for b in range(s) if ((m >> a) & 1) and not ((m >> b) & 1))
            for m in range(1, (1 << s) - 1)
        )
        print(f"  {name} (n={len(T)}, s={s}, del={res[2]}, max_cut={max_dc}):")
        print(f"    T = {T}")
        print(f"    L0 runs = {p1.db.runs_from_seq(L)}")
        print(f"    L1 runs = {p1.db.runs_from_seq(res[0])}")
        print(f"    f0 = {[f0[k] for k in B_core]}")
        print(f"    f1 = {[f1[k] for k in B_core]}")
        print(f"    df = {diff}")

    # Check exact identity on rle_10: what state B had f(B) == f1 - f0?!
    # Wait: look at df of rle_10! Is df == f(B) for a concrete binary word B (e.g. T=(0,1))?!
    L_01, F_01, LF_01, C_01, p_01, s_01 = p1.bwt_of_word((0, 1))
    f_01 = p1.compute_features(L_01, F_01, LF_01, C_01, p_01, s_01, include_text_feats=True)
    print(f"  Compare against B = BWT((0, 1)): {[f_01[k] for k in B_core]} (keys={B_core})")


def audit_phi_self():
    print("\n=== 2. ADVERSARIAL AUDIT OF Phi_self = r + e_2_self ON SELF-RUN CONTRACTION ===")
    # Hypothesis: For EVERY valid BWT, does self-run contraction satisfy r_next <= r + e_2_self?
    # Wait: what if a SINGLE symbol 0 has multiple run lengths AND multiple L-runs of 0?
    # Construct: T has 0^1, 0^2, 0^3, 0^4 preceded by DIFFERENT letters AND followed by DIFFERENT letters,
    # OR 0^2 and 0^3 interleaved in F_1 bucket!!
    # Check: In F_1 bucket, rows where L=0 correspond to occurrences of `0 1` in T!
    # In T = 0 1 2 0 0 1 3 0 0 0 1 4 0 0 0 0 1 5 ...:
    # Before self-run contraction, inside F_1, ALL four occurrences of `0 1` have L=0 -> ONE single run of L=0 inside F_1!
    # AFTER self-run contraction, those four occurrences become `0^(1) 1`, `0^(2) 1`, `0^(3) 1`, `0^(4) 1`!
    # So inside F_1, that ONE run of `L=0` splits into **FOUR DISTINCT SYMBOLS** `0^(1), 0^(2), 0^(3), 0^(4)` -> +3 NEW RUNS!!
    # WHILE e_2_self == 1 (ONLY ONE self-loop symbol `0`)!!
    # TEST THAT CONSTRUCTION RIGHT NOW!!
    for K in [2, 3, 4, 5, 6]:
        # T = (0^1 1 2) (0^2 1 3) (0^3 1 4) ... (0^K 1 (K+1))
        T = []
        for j in range(1, K + 1):
            T.extend([0] * j + [1, j + 1])
        T = tuple(T)
        L, F, LF, C, p, s = p1.bwt_of_word(T)
        f0 = p1.compute_features(L, F, LF, C, p, s, include_text_feats=True)
        res = p1.apply_bdmr_self(L, F, LF, C, p, s)
        f1 = p1.compute_features(*p1.bwt_from_L_only(res[0], 0), include_text_feats=True)
        print(f"  Multi-len single-run split K={K} (n={len(T):2d}): r: {f0['r']:2d} -> {f1['r']:2d} (delta_r={f1['r']-f0['r']:+d}!), "
              f"e_2_self={f0['e_2_self']}->0, r_F:{f0['r_F']}->{f1['r_F']}, E_RG:{f0['E_RG']}->{f1['E_RG']}, "
              f"ref_L2:{f0['ref_L2']}->{f1['ref_L2']}, r_rev:{f0['r_rev']}->{f1['r_rev']}, delta:{f0['delta']:.2f}->{f1['delta']:.2f}")

    # EVEN BETTER: Make the ENTIRE word use ONLY 3 symbols {0, 1, 2} so sigma=3, e_2_self=1, yet delta_r -> +infinity!
    # Inside F_1, sort order of suffixes `1 ...` is determined by what follows `1`!
    # Put 0^1 1 2^1, 0^2 1 2^2, ..., 0^K 1 2^K? No, 2 has self-loops.
    # Instead use binary suffixes after 1: `0^j 1 (binary encoding of j using {1, 2} without self-loops? Just {1, 2, 3}!)`
    # Use 0^1 1 2 1 2, 0^2 1 2 1 3, 0^3 1 2 2 1 ... or simply:
    # T = prod_{j=1}^K (0^j, 1, x_j) where x_j in {2, 3}^m are distinct non-self-loop strings (Wait: alternating 2,3 has only 2 strings; use {2,3,4}!).
    # Even simpler: ternary {0, 1, 2} with ONLY 0 having self-loops! Can we separate 0^1, 0^2, ..., 0^K inside F_1 using ONLY {1, 2} with NO self-loops on 1,2?
    # Wait: if neither 1 nor 2 has self-loops, only alternating 1 2 1 2 ... is allowed!
    # What if 0^1 1 2 0 1 2 ...? Let's test!
    # Actually the K-letter construction above ALREADY proves delta_r = +(K-2) while e_2_self = 1, wait: check K=3,4,5,6 output!
    # Also check: what about F_1 with ONLY 3 symbols {0,1,2} where 2 ALSO has self-loops (e_2_self=2), or {0,1,2,3} where 1,2,3 have NO self-loops?
    print("  Ternary/Quaternary (e_2_self=1, NO self-loops on 1,2,3) unbounded delta_r check:")
    for K in [3, 4, 5, 6, 8]:
        # Distinct loopless strings over {1,2,3} of length m: start with 1, then distinct loopless tag over {2,3,1}!
        # Easy: tag(j) = 1 followed by base-2 encoding of j where 0->(2,3) and 1->(3,2) -> NO self-loops on 1,2,3!
        m_bits = (K).bit_length()
        T = []
        for j in range(1, K + 1):
            T.extend([0] * j + [1])
            for b in range(m_bits):
                if (j >> b) & 1:
                    T.extend([3, 2])
                else:
                    T.extend([2, 3])
        T = tuple(T)
        L, F, LF, C, p, s = p1.bwt_of_word(T)
        f0 = p1.compute_features(L, F, LF, C, p, s, include_text_feats=True)
        res = p1.apply_bdmr_self(L, F, LF, C, p, s)
        f1 = p1.compute_features(*p1.bwt_from_L_only(res[0], 0), include_text_feats=True)
        print(f"    4-letter (e_2_self=1) K={K}: n={len(T):3d}, r:{f0['r']:2d}->{f1['r']:2d} (delta_r={f1['r']-f0['r']:+d}), "
              f"r_F:{f0['r_F']}->{f1['r_F']}, E_RG:{f0['E_RG']}->{f1['E_RG']}, ref_L2:{f0['ref_L2']}->{f1['ref_L2']}, r_rev:{f0['r_rev']}->{f1['r_rev']}")


def test_dag_dp_piece_explosion():
    print("\n=== 3. ADVERSARIAL PIECE-EXPLOSION ATTACK ON 2-SWEEP TOPOLOGICAL ASCENT-DAG DP ===")
    # Can ONE round of Ascent-DAG (L < F) have fwd_pieces / r0 -> infinity?!
    # THINK! Suppose T has a single long run of `L=0` inside `F_1` (so width w = K),
    # and inside `F_1`, P[1] has K distinct segments (distinct ascending words starting at 1)!
    # How can P[1] have K distinct segments while r_0 is SMALL (say O(1) or O(log K))?!
    # WAIT A SECOND!
    # Look at P[b]: a segment boundary in P[b] occurs ONLY at a suffix where the ascending run from b changes!
    # Every distinct ascending word u_ext starting at b occupies ONE contiguous segment of P[b] (Zero-Bucket-Split Theorem!).
    # So |P[b]| = #{distinct ascending words starting at b occurring in T}!
    # AND in Sweep 1, when we intersect a run (L=a, w) inside F_b (a < b) with P[b], EVERY piece created in F_a
    # produces a DISTINCT segment of P[a] (because a + w_1 != a + w_2 when w_1 != w_2!), EXCEPT at the two ends!
    # WAIT A SECOND!! STOP!! FREEZE!! READ WHAT I JUST WROTE!!
    # In Sweep 1, EVERY internal intersection of a run (a, w) in F_b with P[b] produces a **NEW DISTINCT WORD (a, cwid)** in P[a]!!
    # Wait: can TWO different runs of L=a in F_b intersect the SAME segment of P[b]?
    # Yes, at most 1 boundary per run of L=a!
    # So `fwd_pieces <= r_F + |P|` ALWAYS!
    # NOW ASK: Can `|P|` itself be MUCH larger than `r_0`?!
    # YES or NO?!
    # What is `|P|`? `|P| = #{distinct ascending words u_ext starting at ANY letter in T}`!
    # Can a word T have `r_0 = O(s)` (or `O(log n)`) while having `Omega(2^s)` or `Omega(s^2)` distinct ascending words?!
    # CHECK! To have K distinct ascending words starting at 0, we need K distinct right-contexts of 0 in T!
    # Wait: a single suffix `0 1 2 3 ... (s-1)` has only ONE ascending word per start!
    # To get MANY distinct ascending words, we need MANY distinct subsets/subsequences of `0 < 1 < ... < s-1`!
    # What if T = `0 1 2 ... (s-1)` repeated with slight variations (e.g. skipping 1 element at a time:
    #   for j in 1..s-2: emit `0, 1, ..., j-1, j+1, ..., s-1`)?!
    # Let's test THAT AND `0 1 2 ... (s-1)` with 2-element skips!
    for s in [6, 10, 14, 20, 30]:
        # Skip-1 family: for j in 1..s-1, emit ascending chain 0..s-1 missing j!
        T = []
        for j in range(1, s):
            T.extend([c for c in range(s) if c != j])
        T = tuple(T)
        L, F, LF, C, p, sig = p1.bwt_of_word(T)
        runs = p1.db.runs_from_seq(L)
        _, m, new_s, st = p2b.run_space_ascent_dag_step(runs, len(L), sig, mode="asc")
        print(f"  Skip-1 (s={s:2d}, n={len(T):4d}): r0={st['r0']:3d} -> r_S={st['r_S']:3d} ({st['r_ratio']:.2f}x), "
              f"|P|={st['P_segs']:4d} ({st['P_segs']/st['r0']:.2f} r0!), "
              f"fwd={st['fwd_pieces']:4d} ({st['fwd_pieces']/st['r0']:.2f} r0), "
              f"bwd={st['bwd_pieces']:4d} ({st['bwd_pieces']/st['r0']:.2f} r0), "
              f"tot/r0={st['total_pieces_per_r0']:.2f}x")

    # Skip-1 with REPEATED prefix/suffix to keep r0 tiny while |P| = Theta(s^2)!!
    # Wait: in Skip-1 above, look at |P|/r0! Let's see what it prints!
    # Also test: T = (0 1 2 ... s-1) + (0 1 2 ... s-2) + (0 1 2 ... s-3) + ... + (0 1) + (0) (prefix triangle!)
    print("  Prefix-Triangle family: prod_{k=1}^{s-1} (0, 1, ..., k):")
    for s in [6, 10, 16, 24, 36]:
        T = []
        for k in range(1, s):
            T.extend(range(k + 1))
        T = tuple(T)
        L, F, LF, C, p, sig = p1.bwt_of_word(T)
        runs = p1.db.runs_from_seq(L)
        _, m, new_s, st = p2b.run_space_ascent_dag_step(runs, len(L), sig, mode="asc")
        print(f"  Tri (s={s:2d}, n={len(T):4d}): r0={st['r0']:3d} -> r_S={st['r_S']:3d} ({st['r_ratio']:.2f}x), "
              f"|P|={st['P_segs']:4d} ({st['P_segs']/st['r0']:.2f} r0!), "
              f"fwd={st['fwd_pieces']:4d} ({st['fwd_pieces']/st['r0']:.2f} r0), "
              f"bwd={st['bwd_pieces']:4d} ({st['bwd_pieces']/st['r0']:.2f} r0), "
              f"tot/r0={st['total_pieces_per_r0']:.2f}x")


if __name__ == "__main__":
    t0 = time.time()
    inspect_farkas_witnesses()
    audit_phi_self()
    test_dag_dp_piece_explosion()
    print(f"\nDone in {time.time()-t0:.2f}s")
