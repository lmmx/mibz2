#!/usr/bin/env python3
"""Task 2b: Check if Low-r Padded-Ring forces Theta(n) (omega(r)) RV-Zorich steps in Beta's iet_bse
under ALL 6 elimination orders as K grows while r stays O(1)!"""
import itertools
import sys

sys.path.insert(0, "/tmp/fcp-b")
import explore_triage as beta_et


def de_bruijn(k, n_order):
    a = [0] * (k * n_order)
    seq = []
    def db(t, p):
        if t > n_order:
            if n_order % p == 0:
                seq.extend(a[1:p + 1])
        else:
            a[t] = a[t - p]
            db(t + 1, p)
            for j in range(a[t - p] + 1, k):
                a[t] = j
                db(t + 1, t)
    db(1, 1)
    return seq


def iet_bse_forced_order(runs, p, forced_order):
    """Exact copy of Beta's iet_bse inner logic with forced bucket elimination order,
    to verify whether EVERY order pays Theta(n) steps as K scales."""
    # Simply monkey-patch? No, Beta's iet_bse supports order_mode in ('min_deg', 'freq_asc', 'freq_desc', 'asc', 'desc').
    # Wait: let's inspect lines 1120-1180 of explore_triage.py? Even cleaner: rename the 3 letters {0,1,2}
    # by all 6 permutations of ASCII codes [0, 1, 2] and run order_mode='asc'!
    # Because changing alphabet order changes BWT! Wait: NO, changing alphabet order changes the BWT!
    # So we must NOT permute the alphabet; instead we'll run a tiny wrapper copying lines 624-1180?
    # Let's check what order_modes beta_et.iet_bse accepts first, and also check directly!
    pass


def main():
    # First test Beta's iet_bse(min_deg, freq_asc, freq_desc, asc, desc) on Low-r Padded-Ring k=4 as K = 4, 16, 64, 256!
    print("=== Scaling K on Low-r Padded-Ring (k=4, |Sigma|=3, r=39 CONSTANT!) ===")
    db2 = de_bruijn(2, 4)
    for K in [4, 16, 64, 256]:
        w_ab = b"".join((b"a" if x == 0 else b"b") * K for x in db2)
        w_bc = b"".join((b"b" if x == 0 else b"c") * K for x in db2)
        w_ca = b"".join((b"c" if x == 0 else b"a") * K for x in db2)
        T = w_ab + w_bc + w_ca
        L, p = beta_et.bwt_encode(T)
        runs = beta_et.rle_from_L(L)
        r = len(runs)
        print(f"\nK={K:3d}, n={len(T):5d}, r={r:3d} (r/n={r/len(T):.5f}):")
        for om in ("min_deg", "asc", "desc"):
            try:
                out, st = beta_et.iet_bse(runs, p, order_mode=om, build_slp=True)
                assert bytes(out) == T
                print(
                    f"  order={om:8s} | max_self_q={st['max_self_pieces']:3d} | "
                    f"self_rv={st['total_self_rv_steps']:5d} final_rv={st['final_rv_steps']:5d} "
                    f"total_steps={st['total_steps']:5d} ({st['total_steps']/len(T):.3f}n, {st['total_steps']/r:6.2f}r)"
                )
            except Exception as e:
                print(f"  order={om:8s} | ERROR: {e}")

    # Also test Symmetric-Middle-Run minimal 3-letter T = M^K A M^K Z!
    print("\n=== Scaling K on 3-letter Symmetric-Middle-Run T = M^K A M^K Z (r=4 CONSTANT!) ===")
    for K in [4, 16, 64, 256]:
        T = b"M" * K + b"A" + b"M" * K + b"Z"
        L, p = beta_et.bwt_encode(T)
        runs = beta_et.rle_from_L(L)
        r = len(runs)
        print(f"K={K:3d}, n={len(T):4d}, r={r}:")
        for om in ("min_deg", "asc", "desc"):
            out, st = beta_et.iet_bse(runs, p, order_mode=om, build_slp=True)
            assert bytes(out) == T
            print(
                f"  order={om:8s} | max_self_q={st['max_self_pieces']:3d} | "
                f"self_rv={st['total_self_rv_steps']:5d} final_rv={st['final_rv_steps']:5d} "
                f"total_steps={st['total_steps']:5d} ({st['total_steps']/r:6.2f}r)"
            )


if __name__ == "__main__":
    main()
