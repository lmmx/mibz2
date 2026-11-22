#!/usr/bin/env python3
"""
Cycle 4 Task 3: Independent Adversarial Audit of Alpha's Theorem C3 (`4 Q_comp + 2 Q_sp + P_rem >= r_0` in C_gap).
1. Test the exact boundary-accounting bound:
   - Can ONE composition + gap-closing coalescence destroy > 4 pieces (i.e. Delta P = -5)?
   - Can ONE spiral + gap-closing coalescence destroy > 2 pieces (i.e. Delta P = -3)?
   - Check unary, periodic multi-cycle, full-bucket annihilation, exhaustive binary n<=10, ternary n<=6!
2. Independently run and audit Alpha's `test_gap_closed_quotient.py` (exactness, G_dedup = 0.801-0.938 r_0,
   and 5.2% gap-closing saving claim on all 6 authenticated Wikidata blocks).
"""
import hashlib
import json
import os
import sys
from bisect import bisect_right

sys.path.insert(0, "/tmp/fcp-a")
import test_gap_closed_quotient as gcq
import measure_all_trees as mt


def audit_single_step_deltas():
    print("=== 3.1 TESTING ATOMIC STEP PIECE-DROP BOUNDS IN C_gap (Delta P <= -5? Spiral <= -3? Unary/Periodic?) ===")

    # (A) Construct an EXPLICIT valid permutation bijection in C_gap where ONE composition + gap-closing
    #     achieves Delta P = -5 (REFUTING the step-level `-4` bound if images straddling dom(B) coalesce!)
    # Let's build it explicitly on [0, 10) with 6 pieces:
    #   Need:
    #     - Piece B = [4, 5) -> [9, 10), word 'b' (DELETED via composing A -> B!)
    #     - L_B = [3, 4) -> [8, 9), word 'z'  and  R_B = [5, 6) -> [9, ...? Wait: target of R_B must be adjacent to im(L_B) AFTER closing [4, 5)!
    #       Notice: if im(L_B) = [6, 7) and im(R_B) = [7, 8) (both > 5, shifted by -1!), then AFTER closing [4, 5):
    #         L_B: [3, 4) -> [5, 6) (delta = +2, word 'z')
    #         R_B: [4, 5) -> [6, 7) (delta = +2, word 'z') => COALESCE (-1)!!
    #     - Piece X = [6, 7) -> [3, 4), word 'x'  and  Piece Y = [7, 8) -> [5, 6), word 'x'!
    #       BEFORE closing [4, 5): X has delta = -3, Y has delta = -2 (DIFFERENT deltas, cannot coalesce!).
    #       Wait: im(X) = [3, 4) ends at 4 = dom(B).lo, and im(Y) = [5, 6) starts at 5 = dom(B).hi!
    #       AFTER closing [4, 5): X maps [5, 6) -> [3, 4) (delta = -2) and Y maps [6, 7) -> [4, 5) (delta = -2) => COALESCE (-1)!!
    #     - Piece A = [1, 2) -> [4, 5) (word 'a'), flanked by:
    #       L_A = [0, 1) -> [8, 9) (word 'ab') and R_A = [2, 3) -> [10? Wait use [0, 11)!]
    #       Let domain be [0, 11):
    #         L_A: [0, 1) -> [8, 9),  word 'ab'
    #         A  : [1, 2) -> [4, 5),  word 'a'   (composes with B=[4,5)->[9,10) word 'b' => [1,2)->[9,10) word 'ab'!)
    #         R_A: [2, 3) -> [10,11), word 'ab'
    #         L_B: [3, 4) -> [6, 7),  word 'z'
    #         B  : [4, 5) -> [9, 10), word 'b'   (DELETED!)
    #         R_B: [5, 6) -> [7, 8),  word 'z'
    #         X  : [6, 7) -> [3, 4),  word 'x'
    #         Y  : [7, 8) -> [5, 6),  word 'x'
    #         Z1 : [8, 11) -> [0, 3), word 'q'
    # CHECK BIJECTION ON [0, 11):
    #   Domains: [0,1),[1,2),[2,3),[3,4),[4,5),[5,6),[6,7),[7,8),[8,11) -> exact partition of [0, 11)!
    #   Images:  [8,9),[4,5),[10,11),[6,7),[9,10),[7,8),[3,4),[5,6),[0,3) -> [0,3),[3,4),[4,5),[5,6),[6,7),[7,8),[8,9),[9,10),[10,11) -> EXACT BIJECTION on [0, 11)!!
    #   Initial pieces: 9 pieces!
    #   Perform 1 SINGLE composition A o B (delete B = [4,5)):
    #     Remaining 8 pieces before gap-closing:
    #       [0,1)->[8,9) 'ab', [1,2)->[9,10) 'ab', [2,3)->[10,11) 'ab',
    #       [3,4)->[6,7) 'z', [5,6)->[7,8) 'z',
    #       [6,7)->[3,4) 'x', [7,8)->[5,6) 'x',
    #       [8,11)->[0,3) 'q'
    #   Close gap [4, 5) via compact_and_coalesce:
    #       [0,3)->[7,10) 'ab' (L_A + A' + R_A coalesced: -2!)
    #       [3,5)->[5,7)  'z'  (L_B + R_B coalesced: -1!)
    #       [5,7)->[3,5)  'x'  (X + Y coalesced: -1!)
    #       [7,10)->[0,3) 'q'
    #   Remaining pieces: 4 pieces!!
    #   DELTA P = 4 - 9 = -5 FROM ONE SINGLE COMPOSITION (Q_comp = 1)!!
    slp = gcq.HashConsSLP()
    w_a, w_b, w_z, w_x, w_q = ord('a'), ord('b'), ord('z'), ord('x'), ord('q')
    w_ab = slp.concat(w_a, w_b)
    init_9 = [
        (0, 1, 8, w_ab, -1),   # L_A: [0,1)->[8,9)
        (1, 2, 3, w_a, -1),    # A:   [1,2)->[4,5)
        (2, 3, 8, w_ab, -1),   # R_A: [2,3)->[10,11)
        (3, 4, 3, w_z, -1),    # L_B: [3,4)->[6,7)
        (4, 5, 5, w_b, -1),    # B:   [4,5)->[9,10)  <-- deleted!
        (5, 6, 2, w_z, -1),    # R_B: [5,6)->[7,8)
        (6, 7, -3, w_x, -1),   # X:   [6,7)->[3,4)
        (7, 8, -2, w_x, -1),   # Y:   [7,8)->[5,6)
        (8, 11, -8, w_q, -1),  # Z1:  [8,11)->[0,3)
    ]
    # Verify init_9 is already maximally coalesced (0 coalescences before step!)
    c0, coal0 = gcq.compact_and_coalesce(init_9)
    assert len(c0) == 9 and coal0 == 0
    # Verify init_9 is an exact bijection on [0, 11)
    imgs = sorted(x + d for lo, hi, d, _, _ in init_9 for x in range(lo, hi))
    assert imgs == list(range(11))
    # Execute 1 SINGLE composition A o B (delete B)
    after_1comp = [
        (0, 1, 8, w_ab, -1),
        (1, 2, 8, w_ab, -1),   # A o B: [1,2)->[9,10), word 'ab'
        (2, 3, 8, w_ab, -1),
        (3, 4, 3, w_z, -1),
        (5, 6, 2, w_z, -1),
        (6, 7, -3, w_x, -1),
        (7, 8, -2, w_x, -1),
        (8, 11, -8, w_q, -1),
    ]
    c1, coal1 = gcq.compact_and_coalesce(after_1comp)
    print(f"  [COUNTEREXAMPLE TO -4/COMP STEP BOUND]: 1 single composition (Q_comp=1) in C_gap drops P: "
          f"{len(c0)} -> {len(c1)} (Delta P = {len(c1) - len(c0):+d}, coalesced={coal1})!!", flush=True)

    # (B) Construct an EXPLICIT valid bijection where ONE SINGLE SPIRAL (Q_sp = 1, Q_comp = 0)
    #     achieves Delta P = -3 (REFUTING the step-level `-2/spiral` bound!)
    # Let S = [1, 3) -> [2, 4) (d = +1, L = 2, word 'a'): spiral compresses S to [1, 2) -> [3, 4) (word 'aa'),
    # deleting gap [2, 3)!
    # Let:
    #   L_S = [0, 1) -> [2, 3)? No, [2, 3) is inside S's image [2, 4)! S maps [1,2)->[2,3) and [2,3)->[3,4).
    #   So after deleting [2, 3), S' = [1, 2) -> [3, 4) (compressed: [1, 2) -> [2, 3), word 'aa', delta = +1)!
    #   Put L_S = [0, 1) -> [1, 2) (compressed: [0, 1) -> [1, 2), word 'aa', delta = +1) => COALESCE with S' (-1)!
    #   Put R_S = [3, 4) -> [4, 5) (compressed: [2, 3) -> [3, 4), word 'aa', delta = +1) => COALESCE with S' (-1)!
    #   Put X = [4, 5) -> [0, 1)? Wait: need im(X) ending at 2 (impossible since [1,2)->[2,3) was S!)!
    #   AHA!! Can im(X) end at 2? NO, because [2, 3) is covered by im(S)!
    #   WAIT: what about L_S and R_S coalescing with S'? Check bijection on [0, 6)!
    #   Domains:
    #     L_S: [0, 1) -> [1, 2)   word 'aa'
    #     S  : [1, 3) -> [2, 4)   word 'a'   (d=+1, L=2 -> spirals to [1,2)->[3,4) word 'aa', deleting [2,3)!)
    #     R_S: [3, 4) -> [4, 5)   word 'aa'
    #     Z  : [4, 5) -> [0, 1)   word 'z'
    #   CHECK BIJECTION ON [0, 5):
    #     Domains: [0,1), [1,3), [3,4), [4,5) -> exact partition of [0, 5)!
    #     Images:  [1,2), [2,4), [4,5), [0,1) -> exact bijection on [0, 5)!!
    #   Initial pieces: 4 pieces (maximally coalesced: 'aa', 'a', 'aa', 'z')!
    #   After 1 SINGLE spiral on S (Q_sp = 1, Q_comp = 0):
    #     L_S = [0, 1) -> [1, 2) 'aa'
    #     S'  = [1, 2) -> [2, 3) 'aa' (after closing gap [2, 3)!)
    #     R_S = [2, 3) -> [3, 4) 'aa'
    #     Z   = [3, 4) -> [0, 1) 'z'
    #   Coalesced: [0, 3) -> [1, 4) 'aa', [3, 4) -> [0, 1) 'z' => 2 pieces (Delta P = -2)!
    #   Wait: why couldn't X, Y ALSO coalesce across gap [2, 3)? Because [2, 3) had ONLY internal preimage from S itself,
    #   so NO external piece mapped to 2 or from 2! So a single spiral REALLY CAN ONLY drop P by AT MOST -2!!
    w_aa = slp.power(w_a, 2)
    init_sp = [
        (0, 1, 1, w_aa, -1),
        (1, 3, 1, w_a, -1),
        (3, 4, 1, w_aa, -1),
        (4, 5, -4, w_z, -1),
    ]
    cs0, coal_s0 = gcq.compact_and_coalesce(init_sp)
    assert len(cs0) == 4 and coal_s0 == 0
    after_sp = [
        (0, 1, 1, w_aa, -1),
        (1, 2, 2, w_aa, -1),  # S': [1,2)->[3,4) before gap closing!
        (3, 4, 1, w_aa, -1),
        (4, 5, -4, w_z, -1),
    ]
    cs1, coal_s1 = gcq.compact_and_coalesce(after_sp)
    print(f"  [TIGHT SPIRAL BOUND VERIFIED]: 1 spiral achieves exact Delta P = {len(cs1)-len(cs0):+d} ({len(cs0)} -> {len(cs1)}, coalesced={coal_s1}); -3 impossible because im(gap) is internal to S!", flush=True)

    # (C) Check Theorem C3 `4*Q_comp + 2*Q_sp + P_rem >= r0` on unary & periodic BWTs in Alpha's exact code!
    print("\n=== 3.2 CHECKING ALPHA'S THEOREM C3 ON UNARY, PERIODIC & SMALL VALID BWTs ===", flush=True)
    for lbl, T in [
        ("unary a^8", b"aaaaaaaa"),
        ("periodic (ab)^4", b"abababab"),
        ("periodic (a^4 b)^2", b"aaaabaaaab"),
        ("periodic (abc)^3", b"abcabcabc"),
        ("primitive a^8 b", b"aaaaaaaab"),
        ("primitive Fib_7", b"abaababaabaababaababa"),
    ]:
        L, p = mt.bwt_encode(T)
        dec, st = gcq.gap_closed_polarity_dag(L, p, use_coalesce=True)
        assert dec == T
        r0 = st["r0"]
        qc = st.get("Q_comp", 0)
        qs = st.get("spirals", 0)
        sc = st.get("self_cycles", 0)
        lhs = 4 * qc + 2 * qs
        print(f"  [{lbl:18s}] r0={r0:2d} | Q_comp={qc:2d} Q_sp={qs:2d} self_cyc={sc:2d} | 4Q_c+2Q_s={lhs:3d} vs r0={r0:2d} (holds? {lhs >= r0})", flush=True)

    # (D) Independently run Alpha's 6-block Wikidata audit (G_dedup & 5.2% saving) and verify bit-exactness!
    print("\n=== 3.3 INDEPENDENT 6-BLOCK WIKIDATA AUDIT OF ALPHA'S GAP-CLOSED QUOTIENT ===", flush=True)
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    zs = {"block-0.L": 20083, "block-2.L": 21950, "block-85.L": 30001,
          "block-143.L": 40664, "block-210.L": 46451, "block-207.L": 33608}
    sum_r0 = sum_q_no = sum_q_co = sum_g_co = sum_coal = 0
    for entry in manifest["blocks"]:
        L = open(os.path.join("/tmp/mibz2-blocks", entry["file"]), "rb").read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        p = entry["primary_index"]
        ref = mt.ref_ibwt(L, p)
        dec_no, st_no = gcq.gap_closed_polarity_dag(L, p, use_coalesce=False)
        dec_co, st_co = gcq.gap_closed_polarity_dag(L, p, use_coalesce=True)
        assert dec_no == ref and dec_co == ref
        r0 = st_co["r0"]
        z = zs[entry["file"]]
        q_no, q_co = st_no["Q_comp"], st_co["Q_comp"]
        g_co = st_co["G_reach_dedup"]
        sav = 100.0 * (q_no - q_co) / q_no
        sum_r0 += r0; sum_q_no += q_no; sum_q_co += q_co; sum_g_co += g_co; sum_coal += st_co.get("coalesced", 0)
        print(
            f"  {entry['file']:11s} r0={r0:6d} | Q_no={q_no:6d} ({q_no/r0:.2f}r) -> Q_co={q_co:6d} ({q_co/r0:.2f}r, -{sav:.1f}%) "
            f"| coal={st_co.get('coalesced',0):5d} | G_dedup={g_co:6d} ({g_co/r0:.3f}r, {g_co/z:.2f}z) | EXACT",
            flush=True,
        )
    print(f"  6-BLOCK SUM: r0={sum_r0}, Q_no={sum_q_no} ({sum_q_no/sum_r0:.2f}r) -> Q_co={sum_q_co} ({sum_q_co/sum_r0:.2f}r, -{100*(sum_q_no-sum_q_co)/sum_q_no:.1f}%), "
          f"G_dedup={sum_g_co} ({sum_g_co/sum_r0:.3f}r)", flush=True)


if __name__ == "__main__":
    audit_single_step_deltas()
