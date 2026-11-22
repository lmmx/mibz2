#!/usr/bin/env python3
"""
Cycle 2 Audit Task 1 (fast O(n log n) SA, zero OOM!): Adversarial Audit of Gamma's
IET-Schur + Thick-Doubling Core-RV (/tmp/fcp-c/).
"""
import bisect
import hashlib
import json
import os
import sys
import time
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-c")
import explore_iet_schur as g_exp
import test_unified_schur as g_uni
import audit_dichotomy_and_costs as g_aud
import verify_solver_and_ptrack as g_ver


def bwt_encode_fast(T):
    """O(n log^2 n) cyclic suffix-array doubling (O(n) memory!)."""
    n = len(T)
    rank = list(T)
    sa = list(range(n))
    k = 1
    while k < n:
        sa.sort(key=lambda i: (rank[i], rank[(i + k) % n]))
        tmp = [0] * n
        r = 0
        tmp[sa[0]] = 0
        for idx in range(1, n):
            a, b = sa[idx - 1], sa[idx]
            if (rank[a], rank[(a + k) % n]) != (rank[b], rank[(b + k) % n]):
                r += 1
            tmp[b] = r
        rank = tmp
        if r == n - 1:
            break
        k <<= 1
    L = [T[(i - 1) % n] for i in sa]
    p = sa.index(0)
    return L, p


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


def test_part1_edge_cases_and_theorems():
    print("=== 1.1 THEOREMS C1-C5 & EDGE CASES ===", flush=True)

    # (A) Periodic valid BWTs: T = "abab", "abcabc", "aaaa"
    for name, T in [("periodic abab", [0, 1, 0, 1]), ("periodic abcabc", [0, 1, 2, 0, 1, 2]), ("unary aaaa", [0, 0, 0, 0])]:
        L, p = bwt_encode_fast(T)
        I0 = g_ver.encode_I0(L, p)
        got = g_ver.decode_standalone_from_I0(*I0, core_limit=2)
        print(f"  [{name:16s}] T={T} (len={len(T)}) -> decode_standalone_from_I0 returned {got} (len={len(got)}) | EXACT={got == T}", flush=True)

    # (B) Multi-cycle arbitrary L with closed cycle inside eliminated bucket & delta==0
    for L_test in [[1, 0, 2, 2], [1, 0, 2, 1, 2, 2], [0, 1, 2, 0, 1, 2], [2, 1, 0, 2, 1, 0]]:
        errs = []
        for p_try in range(len(L_test)):
            try:
                I0 = g_ver.encode_I0(L_test, p_try)
                got = g_ver.decode_standalone_from_I0(*I0, core_limit=2)
                ref, clen, _, _, _ = g_exp.decode_lf_reference(L_test, p_try)
                if got != ref:
                    errs.append((p_try, f"mismatch {got} vs {ref}"))
            except Exception as ex:
                errs.append((p_try, f"EXCEPTION {type(ex).__name__}: {ex}"))
        print(f"  Multi-cycle L={L_test}: errors={errs}", flush=True)

    # (C) Sub-Interval 1-Piece Schur Delta_q check on SymDB-Ring(k=4)
    db2 = de_bruijn(2, 4)
    T_ring = [0 if x == 0 else 1 for x in db2] + [1 if x == 0 else 2 for x in db2] + [2 if x == 0 else 0 for x in db2]
    L_ring, _ = bwt_encode_fast(T_ring)
    alpha, C, counts, pbs, slp, r0 = g_exp.build_initial_iet(L_ring)
    g_exp.eliminate_bucket(0, pbs, slp)
    cur = list(pbs[1])
    ext = [pc for b in pbs if b != 1 for pc in pbs[b] if pc[4] == 1]
    q_init_sub = len(cur) + len(ext)
    max_dq = -999
    max_q_sub = q_init_sub
    step_no = 0
    while True:
        s_idx = [i for i, pc in enumerate(cur) if pc[4] == 1]
        if not s_idx:
            break
        q_before = len(cur) + len(ext)
        big_i = max(s_idx, key=lambda i: cur[i][1])
        A, Lb, Db, _, _, Wb = cur.pop(big_i)
        B = A + Lb
        delta = Db - A
        def route(plist):
            out = []
            for pc in plist:
                s, l, d, sb, db, wid = pc
                if db != 1:
                    out.append(pc); continue
                if d < A:
                    tk = min(l, A - d); out.append((s, tk, d, sb, db, wid)); s += tk; d += tk; l -= tk
                if l <= 0: continue
                if d < B:
                    tk = min(l, B - d); u = d
                    st = [(s, tk, u, wid)]
                    while st:
                        ss, sl, su, sw = st.pop()
                        sv = su + sl
                        if delta > 0:
                            kmax = (B - 1 - su) // delta + 1
                            kmin = (B - 1 - (sv - 1)) // delta + 1
                            if kmax == kmin:
                                out.append((ss, sl, su + kmax * delta, sb, 1, 0))
                            else:
                                sp = B - (kmax - 1) * delta; l1 = sp - su
                                if l1 < sl: st.append((ss + l1, sl - l1, sp, sw))
                                out.append((ss, l1, su + kmax * delta, sb, 1, 0))
                        else:
                            stp = -delta
                            kmin = (su - A) // stp + 1
                            kmax = (sv - 1 - A) // stp + 1
                            if kmin == kmax:
                                out.append((ss, sl, su - kmin * stp, sb, 1, 0))
                            else:
                                sp = A + kmin * stp; l1 = sp - su
                                if l1 < sl: st.append((ss + l1, sl - l1, sp, sw))
                                out.append((ss, l1, su - kmin * stp, sb, 1, 0))
                    s += tk; d += tk; l -= tk
                if l <= 0: continue
                out.append((s, l, d, sb, db, wid))
            return out
        cur = route(cur)
        ext = route(ext)
        q_after = len(cur) + len(ext)
        if q_after - q_before > max_dq: max_dq = q_after - q_before
        if q_after > max_q_sub: max_q_sub = q_after
        step_no += 1
    print(f"  Sub-Interval 1-Piece Schur on SymDB-Ring(k=4) bucket 1: q_init={q_init_sub}, max_q={max_q_sub}, max_Delta_q={max_dq:+d} (steps={step_no})", flush=True)


def test_part2_stress_gamma_subinterval():
    print("\n=== 1.2 STRESS-TESTING GAMMA STAGE 1 + SUB-INTERVAL 1-PIECE SCHUR ON ADVERSARIAL LOW-r FAMILIES ===", flush=True)
    db2 = de_bruijn(2, 4)

    print("\n--- Family 1: Low-r Padded-Ring (k=4, |Sigma|=3, r=39 CONSTANT) ---", flush=True)
    for K in [4, 16, 64, 256, 1024]:
        T = (
            [0 if x == 0 else 1 for x in db2 for _ in range(K)]
            + [1 if x == 0 else 2 for x in db2 for _ in range(K)]
            + [2 if x == 0 else 0 for x in db2 for _ in range(K)]
        )
        L, p = bwt_encode_fast(T)
        st1 = g_aud.run_stage1_instrumented(L, keep_bucket=2)
        alpha, C, counts, pbs, slp, r0 = g_exp.build_initial_iet(L)
        g_exp.eliminate_bucket(0, pbs, slp, use_spiral=True)
        in_p = [pc for b in pbs if b != 1 for pc in pbs[b] if pc[4] == 1]
        final_ext, sub_steps, sub_ops = g_uni.euclid_accelerate_2iet_bucket(pbs[1], in_p, slp)
        print(
            f"  K={K:4d} n={len(T):5d} r0={r0:2d} | "
            f"Std-Schur(1stMrg={st1['first_hop_merges']:3d}, SlfLkp={st1['self_loop_lookups']:5d}, Sp={st1['spiral_jumps']:4d}) | "
            f"SubInt-1Pc(steps={sub_steps:4d}, ops={sub_ops:5d}, out_q={len(final_ext):3d})",
            flush=True,
        )

    print("\n--- Family 2: AltRing (ab)^k (bc)^k (ca)^k (|Sigma|=3, r=10 CONSTANT) ---", flush=True)
    for k in [4, 16, 64, 256, 1024]:
        T = [0, 1] * k + [1, 2] * k + [2, 0] * k
        L, p = bwt_encode_fast(T)
        st1 = g_aud.run_stage1_instrumented(L, keep_bucket=2)
        alpha, C, counts, pbs, slp, r0 = g_exp.build_initial_iet(L)
        g_exp.eliminate_bucket(0, pbs, slp, use_spiral=True)
        in_p = [pc for b in pbs if b != 1 for pc in pbs[b] if pc[4] == 1]
        final_ext, sub_steps, sub_ops = g_uni.euclid_accelerate_2iet_bucket(pbs[1], in_p, slp)
        print(
            f"  k={k:4d} n={len(T):5d} r0={r0:2d} | "
            f"Std-Schur(1stMrg={st1['first_hop_merges']:3d}, SlfLkp={st1['self_loop_lookups']:5d}, Sp={st1['spiral_jumps']:4d}) | "
            f"SubInt-1Pc(steps={sub_steps:4d}, ops={sub_ops:5d}, out_q={len(final_ext):3d})",
            flush=True,
        )

    print("\n--- Family 3: Symmetric-Middle-Run M^K A M^K Z (r=4 CONSTANT) ---", flush=True)
    for K in [4, 16, 64, 256, 1024]:
        T = [1] * K + [0] + [1] * K + [2]
        L, p = bwt_encode_fast(T)
        st1 = g_aud.run_stage1_instrumented(L, keep_bucket=2)
        alpha, C, counts, pbs, slp, r0 = g_exp.build_initial_iet(L)
        g_exp.eliminate_bucket(0, pbs, slp, use_spiral=True)
        in_p = [pc for b in pbs if b != 1 for pc in pbs[b] if pc[4] == 1]
        final_ext, sub_steps, sub_ops = g_uni.euclid_accelerate_2iet_bucket(pbs[1], in_p, slp)
        print(
            f"  K={K:4d} n={len(T):5d} r0={r0:2d} | "
            f"Std-Schur(1stMrg={st1['first_hop_merges']:3d}, SlfLkp={st1['self_loop_lookups']:5d}, Sp={st1['spiral_jumps']:4d}) | "
            f"SubInt-1Pc(steps={sub_steps:4d}, ops={sub_ops:5d}, out_q={len(final_ext):3d})",
            flush=True,
        )

    print("\n--- Family 4: Non-Monotone Multi-Phase Period Family & Thick-Doubling Stage 2 Check ---", flush=True)
    for K in [8, 32, 128, 512]:
        T = [0, 1] * K + [2] + [0, 0, 1] * K + [2] + [0, 1, 1] * K + [2]
        L, p = bwt_encode_fast(T)
        st1 = g_aud.run_stage1_instrumented(L, keep_bucket=2)
        alpha, C, counts, pbs, slp, r0 = g_exp.build_initial_iet(L)
        g_exp.eliminate_bucket(0, pbs, slp, use_spiral=True)
        in_p = [pc for b in pbs if b != 1 for pc in pbs[b] if pc[4] == 1]
        final_ext, sub_steps, sub_ops = g_uni.euclid_accelerate_2iet_bucket(pbs[1], in_p, slp)
        print(
            f"  3-phase K={K:3d} n={len(T):5d} r0={r0:2d} | "
            f"Std-Schur(SlfLkp={st1['self_loop_lookups']:5d}, Sp={st1['spiral_jumps']:4d}) | "
            f"SubInt-1Pc(steps={sub_steps:4d}, ops={sub_ops:5d}, out_q={len(final_ext):3d})",
            flush=True,
        )

    # Check Stage 2 Thick-Doubling on Low-r Padded-Ring K=256 and AltRing k=256!
    for label, T in [
        ("Low-r Padded-Ring K=256", [0 if x == 0 else 1 for x in db2 for _ in range(256)] + [1 if x == 0 else 2 for x in db2 for _ in range(256)] + [2 if x == 0 else 0 for x in db2 for _ in range(256)]),
        ("AltRing k=256", [0, 1] * 256 + [1, 2] * 256 + [2, 0] * 256),
    ]:
        L, p = bwt_encode_fast(T)
        st1 = g_aud.run_stage1_instrumented(L, keep_bucket=2)
        st2 = g_aud.endpoint_core_rv_on_bucket(st1['final_pieces'], st1['slp'], limit=16)
        print(
            f"  Stage2 [{label:24s}] n={len(T):5d} r0={st1['r0']:2d} | "
            f"St1(SlfLkp={st1['self_loop_lookups']:5d}) | "
            f"St2(F_q={len(st1['final_pieces']):3d}, ThkRds={st2['thick_rounds']:2d}, ThkOps={st2['thick_ops']:5d}, "
            f"ThnMrg={st2['thin_merge_ops']:4d}, m_core={st2['m_core']:5d})",
            flush=True,
        )


def test_part3_wikidata_hidden_costs():
    print("\n=== 1.3 FULL HIDDEN-COST AUDIT OF GAMMA ON AUTHENTICATED WIKIDATA BLOCKS ===", flush=True)
    with open(os.path.join(g_exp.BLOCKS_DIR, "manifest.json")) as f:
        manifest = json.load(f)

    for blk in manifest["blocks"]:
        data = open(os.path.join(g_exp.BLOCKS_DIR, blk["file"]), "rb").read()
        assert hashlib.sha256(data).hexdigest() == blk["sha256"]
        n = len(data)
        p = blk["primary_index"]
        m16, r0, _ = g_uni.compute_open_m16(data, 16)
        I0 = g_ver.encode_I0(data, p, cap=127)
        R_127 = len(I0[2])

        t0 = time.time()
        st1 = g_aud.run_stage1_instrumented(data, keep_bucket=34)
        slp = st1['slp']
        slp_after_st1 = len(slp)
        st2 = g_aud.endpoint_core_rv_on_bucket(st1['final_pieces'], slp, limit=16)
        slp_after_st2 = len(slp)
        dt = time.time() - t0

        gamma_tot_ops = st1['first_hop_merges'] + st1['self_loop_lookups'] + st2['thick_ops'] + st2['thin_merge_ops']
        tot_buf_scanned = st1['src_scanned'] + st1['dst_scanned']
        tot_stale = st1['src_stale'] + st1['dst_stale']
        tot_appends = 2 * (st1['r_init'] + st1['new_pieces'])
        st1_sort_el = st1['sort_elems']
        st2_sort_el = (st2['thick_rounds'] + 2) * st2['thick_final_pieces'] + st2['wired_thin_pieces']

        wt = st2['wired_thin']
        wlen = [0] * len(slp)
        tree_sz = [0] * len(slp)
        for i, nd in enumerate(slp):
            if nd[0] == 'lit':
                wlen[i] = 1
                tree_sz[i] = 1
            elif nd[0] == 'cat':
                wlen[i] = wlen[nd[1]] + wlen[nd[2]]
                tree_sz[i] = 1 + tree_sz[nd[1]] + tree_sz[nd[2]]
            else:
                wlen[i] = wlen[nd[1]] * nd[2]
                tree_sz[i] = 1 + tree_sz[nd[1]] * nd[2]

        used_root_wids = set(x[5] for x in wt)
        reachable = set()
        stack = list(used_root_wids)
        while stack:
            u = stack.pop()
            if u in reachable:
                continue
            reachable.add(u)
            nd = slp[u]
            if nd[0] == 'cat':
                stack.append(nd[1]); stack.append(nd[2])
            elif nd[0] == 'rep':
                stack.append(nd[1])

        flat_core_bytes = sum(wlen[u] for u in used_root_wids)
        naive_expand_visits = sum(x[1] * tree_sz[x[5]] for x in wt)
        m_core = st2['m_core']
        wt_pieces = st2['wired_thin_pieces']

        # Compute i9 calibration budget (Open emit = 4.8-5.8 ns/retained row, Open build = 0.72-0.90 ns/block row)!
        open_build_us = (0.72 * n / 1000.0, 0.90 * n / 1000.0)
        open_emit_us = (4.8 * m16 / 1000.0, 5.8 * m16 / 1000.0)
        gamma_emit_walk_saved_us = (4.8 * (m16 - m_core) / 1000.0, 5.8 * (m16 - m_core) / 1000.0)

        print(f"\n--- {blk['file']} (n={n}, R_127={R_127}, r0={r0}, m16={m16}, py_t={dt:.2f}s) ---", flush=True)
        print(f"  Gamma Headline TotOps = {gamma_tot_ops:7d} ({gamma_tot_ops/r0:.2f} r0) "
              f"[1stMrg={st1['first_hop_merges']}, SlfLkp={st1['self_loop_lookups']}, ThkOps={st2['thick_ops']}, ThnMrg={st2['thin_merge_ops']}]", flush=True)
        print(f"  HIDDEN 1 (Tombstones/Appends): appends={tot_appends:7d} ({tot_appends/r0:.2f} r0), "
              f"scanned={tot_buf_scanned:7d} ({tot_buf_scanned/r0:.2f} r0), stale={tot_stale:7d} ({100*tot_stale/tot_buf_scanned:.1f}%)", flush=True)
        print(f"  HIDDEN 2 (Sorting): St1_SortEl={st1_sort_el:7d} ({st1_sort_el/r0:.2f} r0), "
              f"St2_SortEl~={st2_sort_el:6d} ({st2['thick_rounds']} rds, thick_final={st2['thick_final_pieces']})", flush=True)
        print(f"  HIDDEN 3 (Core/Wiring): m_core={m_core:6d} ({m_core/m16:.3f}x Open m16), "
              f"wt_pieces={wt_pieces:6d} (avg_w={m_core/max(1,wt_pieces):.2f}), "
              f"edges+wid={m_core*8/1024:.1f} KiB", flush=True)
        print(f"  HIDDEN 4 (SLP/FlatCore): slp_tot={slp_after_st2:7d} (st1={slp_after_st1}, st2={slp_after_st2-slp_after_st1}), "
              f"memoOps={len(reachable):7d} ({len(reachable)/r0:.2f} r0), "
              f"flat_q={len(used_root_wids):6d}, flat_core={flat_core_bytes/1024:.1f} KiB, "
              f"naive_rec_visits={naive_expand_visits/n:.2f}n", flush=True)
        print(f"  i9 BAR (4.8-5.8ns emit, 0.72-0.90ns build): Open_build={open_build_us[0]:.0f}-{open_build_us[1]:.0f}us, "
              f"Open_emit={open_emit_us[0]:.0f}-{open_emit_us[1]:.0f}us, "
              f"Gamma_walk_savings={gamma_emit_walk_saved_us[0]:.0f}-{gamma_emit_walk_saved_us[1]:.0f}us", flush=True)


if __name__ == "__main__":
    test_part1_edge_cases_and_theorems()
    test_part2_stress_gamma_subinterval()
    test_part3_wikidata_hidden_costs()
