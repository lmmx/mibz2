#!/usr/bin/env python3
"""Cycle 9 Task 3: Independent Adversarial Audit of Gamma's Origami Theorems O1-O3 & AP4 Text Anatomy."""
import sys
from collections import Counter

sys.path.insert(0, "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-foundational-contraction-programme/cycle8/gamma")
import cycle8_origami_sl2z as orig
import cycle8_ap4_text_anatomy as anat


def main():
    print("=== TASK 3A: ORIGAMI THEOREM O1 AUDIT ON block-0.L & EDGE CASES ===", flush=True)
    blocks = orig.load_blocks()
    blk0, data0 = blocks[0]
    n = len(data0)
    LF, C, cnts = orig.build_lf(data0)
    for k in (0, 1, -1, 2, 100000):
        perm = LF if k == 0 else [(v + k) % n for v in LF]
        st = orig.analyze_permutation(perm, limit=16)
        q0, m16, o1, r1 = orig.compute_sheared_out1_and_r1(perm, limit=16)
        r_s = st["r"]
        cy = st["cycles"]
        mc = st["max_cyc"]
        print(f"  block-0.L k={k:6d}: r={r_s:5d} m16={m16:5d} | out1={100*o1/m16:5.1f}% r1={100*r1/m16:5.1f}% | cycles={cy:3d} max_cyc={mc:6d} ({100*mc/n:4.1f}% n)", flush=True)

    LF_ex = [2, 0, 1]
    st0 = orig.analyze_permutation(LF_ex, limit=16)
    st1 = orig.analyze_permutation([(v + 1) % 3 for v in LF_ex], limit=16)
    print(f"  Wrap-around merge edge case LF=[2,0,1]: k=0 r={st0['r']} -> k=+1 perm=[0,1,2] r={st1['r']} (Delta r = {st1['r']-st0['r']:+d})", flush=True)

    # Also test wrap-around thick-run split edge case (Delta m_16 = +2*16 = +32? Actually splitting 1 thick run
    # of width 32 into two runs of width 16 adds +32 to m_16!):
    LF_th = list(range(32))  # 1 thick run of width 32, m_16 = 0
    perm_th = [(v + 16) % 32 for v in LF_th]  # splits into [0,16)->[16,32) and [16,32)->[0,16) => m_16 = 32!
    st_t0 = orig.analyze_permutation(LF_th, limit=16)
    st_t1 = orig.analyze_permutation(perm_th, limit=16)
    print(f"  Wrap-around thick-split witness (n=32): k=0 r={st_t0['r']}, m16={st_t0['m_16']} -> k=16 r={st_t1['r']}, m16={st_t1['m_16']} (Delta r=+1, Delta m16=+32 SHARP!)", flush=True)

    print("\n=== TASK 3B: AP4 TEXT ANATOMY SPOT-CHECK ON block-0.L & block-210.L ===", flush=True)
    for blk, data in [blocks[0], blocks[4]]:
        nb = len(data)
        pb = blk["primary_index"]
        strips, _, text, row_to_tpos = anat.build_strips_and_decode(data, pb)
        tags = anat.classify_wikidata_text(text)
        par, reason, *_ = anat.run_ap4_max_with_reasons(strips, nb, pb, cap_len=15)
        # Count critical rows by survival reason & text class, and w=1 elimination rate
        w1_tot = w1_elim = 0
        crit_by_reason = Counter()
        crit_by_cls = Counter()
        m_crit = 0
        for u, (lo, hi, d, b, m) in enumerate(strips):
            w = hi - lo
            if w == 1:
                w1_tot += 1
                if par[u] >= 0:
                    w1_elim += 1
            if par[u] < 0 and (w <= 16 or m >= 0):
                crit_by_reason[reason[u]] += w
                m_crit += w
                for row in range(lo, hi):
                    crit_by_cls[anat.CLS_NAMES[tags[row_to_tpos[row]]]] += 1
        print(f"  {blk['file']:11s}: m_crit={m_crit:6d} | w=1 elim={w1_elim}/{w1_tot} ({100*w1_elim/w1_tot:.1f}%) | "
              f"reasons={dict((k, f'{100*v/m_crit:.1f}%') for k, v in crit_by_reason.most_common())}", flush=True)
        print(f"      top classes={dict((k, f'{100*v/m_crit:.1f}%') for k, v in crit_by_cls.most_common(4))}", flush=True)


if __name__ == "__main__":
    main()
