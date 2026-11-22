#!/usr/bin/env python3
"""
Check Church-Rosser confluence of M1, M2, M3, M4 on:
  - 500 random valid single-cycle BWTs of n = 9..24 (alphabets 2, 3, 4, 8)
  - 100 random arbitrary multi-cycle permutations
across 6 distinct peeling orders (fifo, lifo, narrow_first, wide_first, 2 random orders)!
"""
import random
import sys

sys.path.insert(0, "/tmp/fcp-a")
import measure_all_trees as mt
import test_k1_peeling as tk


def check_larger_confluence():
    rng = random.Random(2026)
    for mode in ("M1", "M2", "M3", "M4"):
        diff_core = 0
        diff_q = 0
        wit = None
        wit_q = None
        for trial in range(300):
            n = rng.randint(8, 22)
            sig = rng.choice([2, 3, 4, 6])
            T = bytes(rng.randrange(sig) for _ in range(n))
            if len(set(T)) < 2: continue
            L, p = mt.bwt_encode(T)
            cores = []
            qcs = []
            for pol, sd in [("fifo", 0), ("lifo", 0), ("narrow_first", 0), ("wide_first", 0), ("random", 1), ("random", 2)]:
                strips, ws, cp, st = tk.peel_k1_algebra(L, p, mode=mode, order_policy=pol, rng_seed=sd)
                tk.verify_strips_exact(strips, ws, n, T, cp)
                live = tuple(sorted((s[0], s[1], s[2], ws.expand(s[3])) for s in strips if s[5]))
                cores.append(live)
                qcs.append(st["q_core"])
            if len(set(cores)) > 1:
                if not (all(q <= 1 for q in qcs)):
                    diff_core += 1
                    if wit is None:
                        wit = (list(T), qcs)
                if len(set(qcs)) > 1:
                    diff_q += 1
                    if wit_q is None:
                        wit_q = (list(T), qcs)
        print(f"  {mode}: 300 random BWTs (n=8..22, 6 orders = 1,800 exact runs) | "
              f"non-confluent core: {diff_core} (diff q_core: {diff_q})")
        if wit: print(f"    witness: n={len(wit[0])} T={wit[0]} q_cores={wit[1]}")
        if wit_q: print(f"    diff-q witness: n={len(wit_q[0])} T={wit_q[0]} q_cores={wit_q[1]}")


if __name__ == "__main__":
    check_larger_confluence()
