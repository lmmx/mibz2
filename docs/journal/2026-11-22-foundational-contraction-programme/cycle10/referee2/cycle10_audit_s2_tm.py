#!/usr/bin/env python3
"""Cycle 10 Task 1.2 & 1.3: S2_dt_heavy on block-0/210 + 5 killers + Thick-Morse on block-0."""
import sys
sys.path.insert(0, "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-foundational-contraction-programme/cycle9/alpha")
import test_train_track_folding as ttf
import bwt_core as bc
sys.path.insert(0, "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-foundational-contraction-programme/cycle9/beta")
import cycle9_defect_durand as dd
sys.path.insert(0, "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-foundational-contraction-programme/cycle9/gamma")
import cycle9_thick_morse_audit as tma


def main():
    blocks = dd.load_authenticated_blocks()
    print("=== 2B. S2_dt_heavy ON block-0.L & block-210.L ===", flush=True)
    for blk, raw in [blocks[0], blocks[4]]:
        ref = bc.ref_ibwt(raw, blk["primary_index"])
        dec, st = ttf.fold_train_track(raw, blk["primary_index"], schedule="S2_dt_heavy")
        assert dec == ref
        fn = blk["file"]
        r0 = st["r0"]
        tot_ops = st.get("ops", st.get("total_ops", st.get("tot_ops", sum(v for k, v in st.items() if k in ("r1", "r2", "r3", "r3s", "sp", "R1", "R2", "R3", "R3s", "spirals")))))
        print(f"  {fn:11s} (r0={r0}): {st}", flush=True)

    print("\n=== 2C. S2_dt_heavy ON 5 ADVERSARIAL KILLERS ===", flush=True)
    def de_bruijn(k):
        a = [0] * (2 * k); seq = []
        def db(t, p):
            if t > k:
                if k % p == 0: seq.extend(a[1:p + 1])
            else:
                a[t] = a[t - p]; db(t + 1, p)
                for j in range(a[t - p] + 1, 2): a[t] = j; db(t + 1, t)
        db(1, 1); return seq
    db2 = de_bruijn(4)
    f0, f1 = [0], [0, 1]
    while len(f1) < 6765: f0, f1 = f1, f1 + f0
    fib20 = bytes(f1 + [2])
    killers = [
        ("3-phase K=128", bytes([0, 1]*128 + [2] + [0, 0, 1]*128 + [2] + [0, 1, 1]*128 + [2])),
        ("AltRing k=128", bytes([0, 1]*128 + [1, 2]*128 + [2, 0]*128)),
        ("Low-r Padded-Ring K=64", bytes([0 if x==0 else 1 for x in db2 for _ in range(64)] + [1 if x==0 else 2 for x in db2 for _ in range(64)] + [2 if x==0 else 0 for x in db2 for _ in range(64)])),
        ("SymMidRun K=128", bytes([1]*128 + [0] + [1]*128 + [2])),
        ("Fib_20+$", fib20),
    ]
    for name, T in killers:
        n = len(T); sa = list(range(n)); rk = list(T); k = 1
        while k < n:
            sa.sort(key=lambda i: (rk[i], rk[(i + k) % n]))
            tmp = [0] * n
            for j in range(1, n):
                a, b = sa[j - 1], sa[j]
                tmp[b] = tmp[a] + (1 if (rk[a], rk[(a + k) % n]) != (rk[b], rk[(b + k) % n]) else 0)
            rk = tmp
            if rk[sa[-1]] == n - 1: break
            k <<= 1
        L = bytes(T[(i - 1) % n] for i in sa); p = sa.index(0)
        dec, st = ttf.fold_train_track(L, p, schedule="S2_dt_heavy")
        assert dec == T
        r0 = st["r0"]
        print(f"  [{name:22s}] n={n:5d} r0={r0:2d}: {st}", flush=True)

    print("\n=== 3. THICK-MORSE ON block-0.L ===", flush=True)
    blk0, data0 = blocks[0]
    strips, thin_by_byte, m_ret = tma.extract_thick_and_thin(data0, blk0["primary_index"], limit=16)
    c = tma.census_thick_relationships(strips)
    ns = c["ns"]
    print(f"  block-0.L (W=16): ns={ns}, r1={c['r1_exact']}({100*c['r1_exact']/ns:.1f}%), "
          f"flush+strict={c['flush_incl']+c['strict_int']}({100*(c['flush_incl']+c['strict_int'])/ns:.1f}%), "
          f"m2={c['m2_full_tile']}, m3={c['m3_full_tile']}", flush=True)


if __name__ == "__main__":
    main()
