#!/usr/bin/env python3
"""
Cycle 6 Task 1: Independent Adversarial Audit of ALL THREE Cycle 5 Breakthroughs:
  1. Alpha's Theorem K1/K2/K3 (M1..M4, /tmp/fcp-a/test_k1_peeling.py & bench_m1_k1_native.c):
     - Try to break Theorem K1 (can R1 create a new R1 edge? pure R1 cycles, periodic, unary?)
     - Try to break Theorem K2 (M2 Church-Rosser confluence across orders on arbitrary/periodic/multi-cycle/BWT!)
     - Verify -52.7% m_16 reduction on block-0.L & block-210.L and audit bench_m1_k1_native.c omissions!
  2. Beta's Theorem B5 (256-Cursor Zero-Sort SelThinSq, /tmp/fcp-b/verify_zerosort_selthinsq.py):
     - Exhaustive binary n<=8 + ternary n<=6 (incl. unary, periodic, odd/even, multi-cycle) + 5 adversarial killers!
     - Check l_cursor[c] strict monotonicity & total advances <= r0!
     - Check exact Open u32 edges packing (bits=17/18 -> only 14/15 bits left!), distinct thin bigrams vs 16,383,
       and wire_xor breakage!
  3. Gamma's Tube-Open V6 (/tmp/fcp-c/cycle5_tube_contraction_exact.py):
     - Test thin-tube cycles, self-loops, periodic (abc)^k, unary, AltRing, 3-phase, Low-r Padded-Ring!
     - Spot-check -30.7% walk reduction & exact Open `wide` overflow (`Open_dict + uniq_tube_words` vs `2^(32-bits)-1`)
       on block-0.L & block-210.L!
"""
import hashlib
import itertools
import json
import os
import random
import sys
import time
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import test_k1_peeling as k1p
import measure_all_trees as mt

sys.path.insert(0, "/tmp/fcp-b")
import verify_zerosort_selthinsq as b5v
import escape_padded_ring as epr
import measure_iet_bse as mb

sys.path.insert(0, "/tmp/fcp-c")
import cycle5_tube_contraction_exact as tc5
import cycle5_open_anatomy as c5a


def audit_1_alpha_k1_k2_k3():
    print("=== 1. AUDITING ALPHA'S THEOREM K1 / K2 / K3 (M1..M4 & bench_m1_k1_native.c) ===", flush=True)

    # (A) Can an R1 contraction (J_u == I_v, u != v) EVER create a NEW R1 edge?!
    # Proof check: When u -> v (J_u == I_v) contracts into u' = (I_u -> J_v):
    #   - u' has domain I_u (UNCHANGED!) and target J_v (UNCHANGED from v's target!).
    #   - Any predecessor w of u still has J_w overlapping I_u (EXACTLY the same overlap as before!).
    #   - Any successor z of v still has I_z overlapping J_v (EXACTLY the same overlap as before!).
    #   -Wait: WHAT IF J_v == I_u (a 2-cycle u <-> v of pure R1 edges, e.g. periodic (ab)^k without isolating p!)?
    #   Then contracting u -> v turns u' into a SELF-LOOP J_{u'} == I_{u'} (not a new u!=v R1 edge)!
    #   WHAT IF isolate_p=False on periodic T = "abab" (L = "bbaa", 2-cycle of R1 edges: [0,2)->[2,4) and [2,4)->[0,2))?!
    #   TEST BOTH isolate_p=True AND isolate_p=False on unary, periodic, multi-cycle!
    for lbl, T in [
        ("unary a^6", b"aaaaaa"),
        ("periodic (ab)^4", b"abababab"),
        ("periodic (abc)^3", b"abcabcabc"),
        ("primitive a^6 b", b"aaaaaab"),
    ]:
        L, p = mt.bwt_encode(T)
        for iso in (True, False):
            for mode in ("M1", "M2"):
                try:
                    strips, ws, cp, st = k1p.peel_k1_algebra(L, p, mode=mode, order_policy="fifo", isolate_p=iso)
                    if iso:
                        k1p.verify_strips_exact(strips, ws, len(T), T, cp)
                    live_cnt = sum(1 for s in strips if s[5])
                    print(f"  [{lbl:16s}] iso={str(iso):5s} {mode}: OK (q0={st['q0']}->q_core={live_cnt}, closed_p={cp is not None})", flush=True)
                except Exception as ex:
                    print(f"  [{lbl:16s}] iso={str(iso):5s} {mode}: FAIL {type(ex).__name__}: {ex}", flush=True)

    # (B) Try to BREAK Theorem K2 (M2 Church-Rosser confluence) on:
    #     - Exhaustive binary n<=8 & ternary n<=6 across 4 peeling orders (fifo, lifo, narrow_first, wide_first)
    #     - AND pure R2 cycles (arbitrary non-BWT / periodic multi-cycle L with isolate_p=False!)!
    #     Wait: THINK! In M2 (R1 + R2), suppose v_1 and v_2 form a PURE R2 2-CYCLE (e.g., I_{v1} == J_{v2}
    #     and I_{v2} == J_{v1}, NO isolate_p)!
    #     If we eliminate v_1 FIRST -> v_2 survives (as a self-loop on I_{v2})!
    #     If we eliminate v_2 FIRST -> v_1 survives (as a self-loop on I_{v1})!
    #     DIFFERENT NORMAL FORMS (I_{v1} vs I_{v2}) => CONFLUENCE FAILS ON PURE R1/R2 CYCLES!!
    #     CHECK THAT RIGHT NOW!!
    L_cyc = b"bbaa"  # (ab)^2: strip 0 = [0,2)->[2,4) 'b', strip 1 = [2,4)->[0,2) 'a' (pure R1/R2 2-cycle!)
    s_fifo, _, _, _ = k1p.peel_k1_algebra(L_cyc, 0, mode="M2", order_policy="fifo", isolate_p=False)
    s_lifo, _, _, _ = k1p.peel_k1_algebra(L_cyc, 0, mode="M2", order_policy="lifo", isolate_p=False)
    dom_fifo = sorted((s[0], s[1]) for s in s_fifo if s[5])
    dom_lifo = sorted((s[0], s[1]) for s in s_lifo if s[5])
    print(f"  [PURE R1/R2 2-CYCLE CONFLUENCE TEST on L='bbaa' iso=False]: fifo_dom={dom_fifo} vs lifo_dom={dom_lifo} -> SAME? {dom_fifo == dom_lifo}", flush=True)

    # EVEN BETTER: Test if a PRIMITIVE valid BWT (with isolate_p=True!) can have a pure R2 cycle NOT containing p!
    # Wait: on a PRIMITIVE valid BWT (single LF-cycle on [0, n)), can ANY subset of strips form a closed R2 cycle
    # NOT containing p?! NO! Because a closed union of strips S with LF(S) == S would be a proper invariant union
    # of cycles of LF (impossible when LF is a single cycle of length n and p in [0, n)!)!
    # WAIT: What about an UNMARKED R2 2-cycle on a MULTI-CYCLE column L (even WITH isolate_p=True, p on another cycle!)?
    L_mc = bytes([1, 0, 2, 2])  # LF: 0->1, 1->0 (2-cycle!), 2->2, 3->3. Put p=2 (on cycle {2}, NOT on {0,1}!)
    s_mc1, _, _, _ = k1p.peel_k1_algebra(L_mc, 2, mode="M2", order_policy="fifo", isolate_p=True)
    s_mc2, _, _, _ = k1p.peel_k1_algebra(L_mc, 2, mode="M2", order_policy="lifo", isolate_p=True)
    d_mc1 = sorted((s[0], s[1]) for s in s_mc1 if s[5])
    d_mc2 = sorted((s[0], s[1]) for s in s_mc2 if s[5])
    print(f"  [MULTI-CYCLE UNMARKED R1/R2 2-CYCLE on L=[1,0,2,2] p=2 iso=True]: fifo_dom={d_mc1} vs lifo_dom={d_mc2} -> SAME? {d_mc1 == d_mc2}", flush=True)

    # Check M2 confluence on ALL primitive binary n<=8 & ternary n<=6 with isolate_p=True across fifo vs lifo vs random!
    conf_checked = 0; conf_fail = None
    for base, nmax in [(2, 8), (3, 6)]:
        for n in range(2, nmax + 1):
            for code in range(base ** n):
                T = bytes((code // (base ** i)) % base for i in range(n))
                if len(set(T)) < 2: continue
                L, p = mt.bwt_encode(T)
                # Check primitive (single LF cycle)
                s1, _, cp1, _ = k1p.peel_k1_algebra(L, p, mode="M2", order_policy="fifo", isolate_p=True)
                s2, _, cp2, _ = k1p.peel_k1_algebra(L, p, mode="M2", order_policy="lifo", isolate_p=True)
                d1 = sorted((s[0], s[1], s[2]) for s in s1 if s[5])
                d2 = sorted((s[0], s[1], s[2]) for s in s2 if s[5])
                conf_checked += 1
                if d1 != d2 and conf_fail is None:
                    # Check if T is primitive!
                    is_prim = all(T != T[k:] + T[:k] for k in range(1, n))
                    conf_fail = (list(T), is_prim, d1, d2)
    print(f"  M2 Confluence check ({conf_checked} binary n<=8 + ternary n<=6 BWTs, fifo vs lifo, iso=True): first_diff={conf_fail}", flush=True)

    # (C) Verify Alpha's -52.7% m_16 reduction on block-0.L & block-210.L for M1 and M2!
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    for blk in manifest["blocks"]:
        if blk["file"] not in ("block-0.L", "block-210.L"): continue
        L = open(os.path.join("/tmp/mibz2-blocks", blk["file"]), "rb").read()
        assert hashlib.sha256(L).hexdigest() == blk["sha256"]
        p = blk["primary_index"]
        ref = mt.ref_ibwt(L, p)
        for mode in ("M1", "M2"):
            strips, ws, cp, st = k1p.peel_k1_algebra(L, p, mode=mode, order_policy="fifo", isolate_p=True)
            k1p.verify_strips_exact(strips, ws, len(L), ref, cp)
            m16 = st["m16"]
            m_th = sum(s[1] - s[0] for s in strips if s[5] and (s[1] - s[0]) <= 16)
            q_core = sum(1 for s in strips if s[5])
            print(f"  {blk['file']:11s} {mode}: r0={st['r0']:6d} -> q_core={q_core:6d} ({q_core/st['r0']:.3f}r) | "
                  f"m_th={m_th:6d} / m16={m16:6d} = {m_th/m16:.3f}x ({100*(m_th-m16)/m16:+.1f}%) | slp_gt16={ws.slp_gt16} [EXACT]", flush=True)


def audit_2_beta_b5_selthinsq():
    print("\n=== 2. AUDITING BETA'S THEOREM B5 (256-CURSOR ZERO-SORT SelThinSq) ===", flush=True)
    # (A) Exhaustive binary n<=8 & ternary n<=6 (ALL arbitrary columns, incl. unary, periodic, odd/even, multi-cycle!)
    checked = 0; max_adv_ratio = 0.0; fail = None
    for base, nmax in [(2, 8), (3, 6)]:
        for n in range(1, nmax + 1):
            for code in range(base ** n):
                L = [(code // (base ** i)) % base for i in range(n)]
                runs = mb.rle_from_bytes(L)
                r0 = len(runs)
                # Reference LF walk of length n (exact bzip2 contract, handles multi-cycle & periodic!)
                mx = max(L) + 1
                cnt = [0] * mx
                for x in L: cnt[x] += 1
                C = [0] * mx; acc = 0
                for c in range(mx): C[c] = acc; acc += cnt[c]
                occ = list(C); LF = [0] * n
                for i, x in enumerate(L): LF[i] = occ[x]; occ[x] += 1
                for p in (range(n) if n <= 5 else [0, n // 2, n - 1]):
                    out_r = []
                    cur = p
                    for _ in range(n):
                        out_r.append(L[cur]); cur = LF[cur]
                    ref = bytes(reversed(out_r))
                    for lim in (2, 4, 16):
                        try:
                            st = b5v.verify_selthinsq_256cursor_lf(runs, n, p, limit=lim, verify_bytes=ref)
                            checked += 1
                            if st["cursor_advances"] / r0 > max_adv_ratio:
                                max_adv_ratio = st["cursor_advances"] / r0
                            if st["cursor_advances"] > r0 and fail is None:
                                fail = ("adv>r0", L, p, lim, st)
                        except Exception as ex:
                            if fail is None:
                                fail = (f"{type(ex).__name__}: {ex}", L, p, lim)
    print(f"  Exhaustive arbitrary binary(n<=8)+ternary(n<=6) x lim in {{2,4,16}}: {checked} tests, max(adv/r0)={max_adv_ratio:.3f}, fail={fail}", flush=True)

    # (B) 5 Adversarial killers under Theorem B5!
    db2 = epr.de_bruijn_binary(4)
    f0, f1 = [0], [0, 1]
    while len(f1) < 6765: f0, f1 = f1, f1 + f0
    fib20 = f1 + [2]
    killers = [
        ("3-phase K=64", [0, 1]*64 + [2] + [0, 0, 1]*64 + [2] + [0, 1, 1]*64 + [2]),
        ("AltRing k=64", [0, 1]*64 + [1, 2]*64 + [2, 0]*64),
        ("Low-r Padded-Ring K=32", [0 if x==0 else 1 for x in db2 for _ in range(32)] + [1 if x==0 else 2 for x in db2 for _ in range(32)] + [2 if x==0 else 0 for x in db2 for _ in range(32)]),
        ("SymMidRun K=64", [1]*64 + [0] + [1]*64 + [2]),
        ("Fib_20+$", fib20),
    ]
    for name, T in killers:
        L, p = epr.bwt_encode_fast(T)
        runs = mb.rle_from_bytes(L)
        r0 = len(runs)
        st = b5v.verify_selthinsq_256cursor_lf(runs, len(T), p, limit=16, verify_bytes=bytes(T))
        print(f"  [{name:22s}] n={len(T):5d} r0={r0:2d} | adv={st['cursor_advances']:2d} ({st['cursor_advances']/r0:.2f}r) | "
              f"m16={st['m16']:4d} -> walk={st['thin_steps']:4d} ({st['thin_steps']/max(1,st['m16']):.3f}x) [EXACT]", flush=True)

    # (C) Check exact Open u32 edges bit-budget & distinct thin bigrams on ALL 6 Wikidata blocks!
    print("\n  --- Open v0.5.12 u32 edges Bit-Budget Check for SelThinSq (All 6 Wikidata Blocks) ---", flush=True)
    blocks = mb.load_authenticated_blocks()
    for binfo, raw in blocks:
        fn = binfo["file"]; n = binfo["n"]; p = binfo["primary_index"]
        runs = mb.rle_from_bytes(raw)
        r0 = len(runs)
        # Count m16, distinct thin->thin bigrams, AND Open's thick-band count!
        counts = [0] * 256
        for c, w in runs: counts[c] += w
        C = [0] * 257
        for c in range(256): C[c + 1] = C[c] + counts[c]
        l_start = [0] * r0; l_len = [0] * r0; l_sym = [0] * r0; f_start = [0] * r0
        f_cur = list(C[:256]); buckets_u = [[] for _ in range(256)]
        pos = 0; m16 = 0
        for u, (c, w) in enumerate(runs):
            l_start[u] = pos; l_len[u] = w; l_sym[u] = c; f_start[u] = f_cur[c]
            buckets_u[c].append(u); f_cur[c] += w; pos += w
            if w <= 16: m16 += w
        f_order = [u for c in range(256) for u in buckets_u[c]]
        uniq_bg = set()
        iu = v = ou = ov = 0
        while iu < r0 and v < r0:
            u = f_order[iu]
            tk = min(l_len[u] - ou, l_len[v] - ov)
            if l_len[u] <= 16 and l_len[v] <= 16:
                uniq_bg.add((l_sym[u], l_sym[v]))
            ou += tk; ov += tk
            if ou == l_len[u]: iu += 1; ou = 0
            if ov == l_len[v]: v += 1; ov = 0
        bits = (m16 - 1).bit_length()
        max_codes = (1 << (32 - bits)) - 1
        print(f"  {fn:11s}: m16={m16:6d} (bits={bits:2d}, free_bits={32-bits:2d} -> max_codes={max_codes:5d}) | "
              f"uniq_thin_bigrams={len(uniq_bg):4d} | direct_u16_fits_in_u32? NO ({bits}+16={bits+16}>32!)", flush=True)


def audit_3_gamma_tube_open_v6():
    print("\n=== 3. AUDITING GAMMA'S TUBE-OPEN (V6) (/tmp/fcp-c/cycle5_tube_contraction_exact.py) ===", flush=True)

    # (A) Adversarial tests: periodic (abc)^k, unary, self-loop thin runs, AltRing, 3-phase, Low-r Padded-Ring!
    db2 = epr.de_bruijn_binary(4)
    adv_cases = [
        ("unary a^6", b"aaaaaa"),
        ("periodic (ab)^4", b"abababab"),
        ("periodic (abc)^4", b"abcabcabcabc"),
        ("thin-cycle (ab)^4 (cd)^4", b"ababababcdcdcdcd"),
        ("AltRing k=4", b"ab"*4 + b"bc"*4 + b"ca"*4),
        ("3-phase K=8", b"ab"*8 + b"c" + b"aab"*8 + b"c" + b"abb"*8 + b"c"),
        ("Low-r Padded-Ring K=8", bytes([65 if x==0 else 66 for x in db2 for _ in range(8)] + [66 if x==0 else 67 for x in db2 for _ in range(8)] + [67 if x==0 else 65 for x in db2 for _ in range(8)])),
    ]
    for name, T in adv_cases:
        L, p = mt.bwt_encode(T)
        full_runs, C, counts = c5a.compute_maximal_runs_and_lf(L)
        ret_rows, row_to_ret, base_next, base_words, base_wlen, thin_runs, p_ret = tc5.build_open_retained_map(L, p, full_runs, limit=16)
        try:
            rec, st = tc5.simulate_tube_open(
                L, p, full_runs, ret_rows, row_to_ret, base_next, base_words, base_wlen, thin_runs, p_ret,
                mode="incl", max_wlen=15, include_thick_tail=True, limit=16
            )
            ok = (rec == T)
            print(f"  [{name:24s}] n={len(T):4d} | exact={str(ok):5s} (len_rec={len(rec):4d}) | m_open={st['m_open']:4d}->m_tube={st['m_tube']:4d} ({st['pct_saved']:4.1f}% sav, tubes={st['tubes_built']})", flush=True)
        except Exception as ex:
            print(f"  [{name:24s}] CRASH {type(ex).__name__}: {ex}", flush=True)

    # (B) Spot-check block-0.L & block-210.L for -30.7% walk reduction AND EXACT Open `wide` overflow!
    print("\n  --- Spot-Check block-0.L & block-210.L: Walk Reduction & Exact Open `wide` Code Budget ---", flush=True)
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    for blk in manifest["blocks"]:
        if blk["file"] not in ("block-0.L", "block-210.L"): continue
        L = open(os.path.join("/tmp/mibz2-blocks", blk["file"]), "rb").read()
        assert hashlib.sha256(L).hexdigest() == blk["sha256"]
        p = blk["primary_index"]
        full_runs, C, counts = c5a.compute_maximal_runs_and_lf(L)
        ret_rows, row_to_ret, base_next, base_words, base_wlen, thin_runs, p_ret = tc5.build_open_retained_map(L, p, full_runs, limit=16)
        rec, st = tc5.simulate_tube_open(
            L, p, full_runs, ret_rows, row_to_ret, base_next, base_words, base_wlen, thin_runs, p_ret,
            mode="incl", max_wlen=15, include_thick_tail=True, limit=16
        )
        ref = mt.ref_ibwt(L, p)
        assert rec == ref
        # Count exact distinct words in base Open (thick words + literals) vs Tube-Open V6 (if deduplicated vs raw!)
        base_uniq_w = len(set(base_words))
        # Count distinct words actually created by simulate_tube_open:
        # Recompute unique new_words via st['new_words'] (raw) and compare with max_codes!
        m = st["m_open"]
        bits = (m - 1).bit_length()
        max_codes = (1 << (32 - bits)) - 1
        raw_total_codes = base_uniq_w + st["new_words"]
        print(
            f"  {blk['file']:11s}: m_open={m:6d} (bits={bits}, max_codes={max_codes:5d}) -> m_tube={st['m_tube']:6d} "
            f"(-{st['pct_saved']:.1f}%) | base_uniq_w={base_uniq_w:5d}, raw_tube_words={st['new_words']:5d} "
            f"(sum={raw_total_codes:5d} vs max_codes={max_codes:5d}, OVERFLOW_RAW? {raw_total_codes > max_codes}) [EXACT]",
            flush=True,
        )


if __name__ == "__main__":
    audit_1_alpha_k1_k2_k3()
    audit_2_beta_b5_selthinsq()
    audit_3_gamma_tube_open_v6()
