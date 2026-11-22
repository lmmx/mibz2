#!/usr/bin/env python3
"""Test N = 10^3 .. 10^12 scaling of Beta's IET-BSE-AHT max_live_self & steps on IrratRev d-IETs + Tribonacci."""
import math
import random
import sys

sys.path.insert(0, "/tmp/fcp-ref2")
from cycle3_task1_audit import run_aht_on_iet

sys.path.insert(0, "/tmp/fcp-b")
import escape_padded_ring as epr


def main():
    irrat_weights = {
        4: [1.0, math.sqrt(2) - 1, math.sqrt(3) - 1, math.sqrt(5) - 2],
        6: [1.0, math.sqrt(2) - 1, math.sqrt(3) - 1, math.sqrt(5) - 2, math.sqrt(7) - 2, math.pi - 3],
        8: [1.0, math.sqrt(2) - 1, math.sqrt(3) - 1, math.sqrt(5) - 2, math.sqrt(7) - 2, math.sqrt(11) - 3, math.sqrt(13) - 3, math.pi - 3],
    }
    print("=== 1. Scaling N = 10^3 .. 10^12 on IrratRev d-IET in Beta AHT (NO fallback) ===")
    for d in (4, 6, 8):
        ws = irrat_weights[d]
        sw = sum(ws)
        perm = list(reversed(range(d)))
        for exp in (3, 5, 7, 9, 12):
            N = 10 ** exp
            lens = [max(2, int(round(N * w / sw))) for w in ws]
            lens[-1] = max(2, N - sum(lens[:-1]))
            N_act = sum(lens)
            src = [0]
            for l in lens[:-1]:
                src.append(src[-1] + l)
            dst = [0] * d
            acc = 0
            for pi in perm:
                dst[pi] = acc
                acc += lens[pi]
            pcs = [(src[i], dst[i], lens[i] - (1 if i == d - 1 else 0), i) for i in range(d)]
            exit_p = [(N_act - 1, 1, d - 1)]
            in_p = [(dst[d - 1] + lens[d - 1] - 1, 1, d - 1)]
            st = run_aht_on_iet(pcs, in_p, exit_p)
            ms = st["max_live_self"]
            c1, c2, spl, p2 = st["c1_steps"], st["c2_steps"], st["c2_splits"], st["p2_self"]
            tot = c1 + c2 + p2
            print(f"  d={d} N=10^{exp:2d} | max_q={ms:4d} ({ms/d:5.2f}d) | c1={c1:4d} c2={c2:4d} spl={spl:4d} p2={p2:4d} tot={tot:4d}", flush=True)

    print("\n=== 2. Hill-climbed 4-IET at N = 10^9 ===")
    rng = random.Random(77)
    N = 10 ** 9
    d = 4
    best = (0, None)
    for _ in range(15):
        cuts = sorted(rng.sample(range(1, N - 1), d - 1))
        perm = [3, 2, 1, 0]
        cur = 0
        for it in range(60):
            nc = list(cuts)
            if it > 0:
                idx = rng.randrange(d - 1)
                lo_b = 1 if idx == 0 else nc[idx - 1] + 1
                hi_b = N - 2 if idx == d - 2 else nc[idx + 1] - 1
                nc[idx] = rng.randint(lo_b, hi_b)
            lens = [nc[0], nc[1] - nc[0], nc[2] - nc[1], N - nc[2]]
            src = [0, lens[0], lens[0] + lens[1], lens[0] + lens[1] + lens[2]]
            dst = [0] * 4
            acc = 0
            for pi in perm:
                dst[pi] = acc
                acc += lens[pi]
            pcs = [(src[i], dst[i], lens[i] - (1 if i == 3 else 0), i) for i in range(4)]
            st = run_aht_on_iet(pcs, [(dst[3] + lens[3] - 1, 1, 3)], [(N - 1, 1, 3)])
            sc = st["max_live_self"]
            if sc >= cur:
                cuts, cur = nc, sc
            if sc > best[0]:
                best = (sc, st)
    print(f"  Best 4-IET at N=10^9: max_live_self={best[0]} ({best[0]/4:.2f}x d), st={best[1]}", flush=True)

    print("\n=== 3. Valid BWT Tribonacci in Beta AHT ===")
    for m in (10, 14, 18):
        T = epr.make_tribonacci(m)
        L, p = epr.bwt_encode_fast(T)
        runs = []
        i = 0
        while i < len(L):
            j = i + 1
            while j < len(L) and L[j] == L[i]:
                j += 1
            runs.append((L[i], j - i))
            i = j
        st = epr.run_iet_bse_aht(runs, p, order_mode="rarest_first_deg", aht_thresh=2, verify_seq=T)
        ms = st["max_live_self"]
        print(f"  Tribonacci m={m:2d} n={len(T):6d} r0={len(runs):2d} | max_live_self={ms:2d} "
              f"c1={st['c1_steps']:3d} c2={st['c2_steps']:3d} spl={st['c2_splits']:3d} p2={st['p2_self']:3d} tot={st['total_steps']:4d}")


if __name__ == "__main__":
    main()
