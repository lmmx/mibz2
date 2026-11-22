#!/usr/bin/env python3
"""Final 5-second surgical checks for Cycle 2:
1. Stage 2 Thick-Doubling Theta(n) scaling on AltRing (r0=10 constant) + Sub-Interval 1-Piece Schur
   Delta_q > 0 check & worst-case steps on 4-IETs / Fib / Tribonacci.
2. Apples-to-apples k-layer interval DAG (k=2, 3, 4, Full-DAG) WITH self-run fallback on block-0.L
   down to n <= 4096 vs BDMR and Open!
"""
import hashlib
import json
import os
import random
import sys

sys.path.insert(0, "/tmp/fcp-c")
import explore_iet_schur as g_exp
import test_unified_schur as g_uni
import audit_dichotomy_and_costs as g_aud

sys.path.insert(0, "/tmp/fcp-ref2")
from cycle2_audit_gamma import bwt_encode_fast
from cycle2_audit_ref1_sat import run_k_layer_dag_step

sys.path.insert(0, "/tmp/fcp-ref")
import p1_potential_lp as p1


def check_stage2_and_subint():
    print("=== 1. STAGE 2 THICK-DOUBLING Theta(n) BLOWUP ON AltRing (r0=10 CONSTANT!) ===")
    for k in [16, 64, 256, 1024]:
        T = [0, 1] * k + [1, 2] * k + [2, 0] * k
        L, p = bwt_encode_fast(T)
        st1 = g_aud.run_stage1_instrumented(L, keep_bucket=2)
        st2 = g_aud.endpoint_core_rv_on_bucket(st1['final_pieces'], st1['slp'], limit=16)
        print(
            f"  AltRing k={k:4d} n={len(T):5d} r0={st1['r0']:2d} | "
            f"St1(SlfLkp={st1['self_loop_lookups']:2d}) | "
            f"St2(F_q={len(st1['final_pieces']):2d}, ThkRds={st2['thick_rounds']:2d}, "
            f"ThkOps={st2['thick_ops']:5d} ({st2['thick_ops']/len(T):.3f}n, {st2['thick_ops']/st1['r0']:6.1f}r0), "
            f"thick_final={st2['thick_final_pieces']:4d}, m_core={st2['m_core']:2d})"
        )

    print("\n=== 2. SUB-INTERVAL 1-PIECE SCHUR: CHECK Delta_q > 0 ON RANDOM 3-IETs / SMALL BWTs ===")
    # Check if eliminating widest self-piece P_big = [A, B) EVER gives Delta_q >= 0 (splits a piece straddling [A, B))!
    found_pos_dq = None
    for n in range(4, 10):
        for T in p1.gen_lyndon(n, 3):
            if len(set(T)) < 3:
                continue
            L, _ = bwt_encode_fast(T)
            alpha, C, counts, pbs, slp, r0 = g_exp.build_initial_iet(L)
            for e0 in (0, 1, 2):
                _, _, _, pbs2, slp2, _ = g_exp.build_initial_iet(L)
                g_exp.eliminate_bucket(e0, pbs2, slp2)
                for e1 in (0, 1, 2):
                    if e1 == e0:
                        continue
                    cur = list(pbs2[e1])
                    ext = [pc for b in pbs2 if b != e1 for pc in pbs2[b] if pc[4] == e1]
                    while True:
                        s_idx = [i for i, pc in enumerate(cur) if pc[4] == e1]
                        if not s_idx:
                            break
                        qb = len(cur) + len(ext)
                        big_i = max(s_idx, key=lambda i: cur[i][1])
                        A, Lb, Db, _, _, _ = cur.pop(big_i)
                        B = A + Lb
                        delta = Db - A
                        if delta == 0:
                            break
                        def rt(plist):
                            out = []
                            for pc in plist:
                                s, l, d, sb, db, wid = pc
                                if db != e1:
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
                                            kmx = (B - 1 - su) // delta + 1; kmn = (B - 1 - (sv - 1)) // delta + 1
                                            if kmx == kmn: out.append((ss, sl, su + kmx * delta, sb, e1, 0))
                                            else:
                                                sp = B - (kmx - 1) * delta; l1 = sp - su
                                                if l1 < sl: st.append((ss + l1, sl - l1, sp, sw))
                                                out.append((ss, l1, su + kmx * delta, sb, e1, 0))
                                        else:
                                            stp = -delta
                                            kmn = (su - A) // stp + 1; kmx = (sv - 1 - A) // stp + 1
                                            if kmn == kmx: out.append((ss, sl, su - kmn * stp, sb, e1, 0))
                                            else:
                                                sp = A + kmn * stp; l1 = sp - su
                                                if l1 < sl: st.append((ss + l1, sl - l1, sp, sw))
                                                out.append((ss, l1, su - kmn * stp, sb, e1, 0))
                                    s += tk; d += tk; l -= tk
                                if l <= 0: continue
                                out.append((s, l, d, sb, db, wid))
                            return out
                        cur = rt(cur); ext = rt(ext)
                        qa = len(cur) + len(ext)
                        if qa - qb > 0:
                            found_pos_dq = (n, T, e0, e1, qb, qa, qa - qb)
                            break
                    if found_pos_dq:
                        break
                if found_pos_dq:
                    break
            if found_pos_dq:
                break
        if found_pos_dq:
            break
    print(f"  Sub-Interval 1-Piece Schur Delta_q > 0 witness (ternary Lyndon n<=9): {found_pos_dq}")

    # Also test a synthetic worst-case 1-step witness: P_big = [A, B) of width 2 inside [0, 10) straddled by piece of width 6!
    # Check if random 5-piece IETs of total width 10^6 EVER blow up Sub-Interval 1-Piece Schur!
    rng = random.Random(42)
    max_steps = 0
    max_ops = 0
    max_q_ratio = 0.0
    for trial in range(100):
        # Build random 4-IET on [0, N) with N = 100000 and 1 width-1 exit hole!
        N = 100000
        cuts = sorted(rng.sample(range(1, N), 4))
        lens = [cuts[0], cuts[1] - cuts[0], cuts[2] - cuts[1], cuts[3] - cuts[2], N - cuts[3]]
        perm = list(range(5))
        rng.shuffle(perm)
        src_offs = [0]
        for l in lens[:-1]: src_offs.append(src_offs[-1] + l)
        dst_offs = [0] * 5
        acc = 0
        for p_i in perm:
            dst_offs[p_i] = acc
            acc += lens[p_i]
        # Replace position N-1 with exit to bucket 99, and entry from bucket 99 to where N-1 mapped!
        slp = [('lit', i) for i in range(10)]
        pcs = []
        in_p = []
        for i in range(5):
            if i == 4 and lens[i] > 1:
                pcs.append((src_offs[i], lens[i] - 1, dst_offs[i], 0, 0, i))
                pcs.append((N - 1, 1, 0, 0, 99, i))
                in_p.append((0, 1, dst_offs[i] + lens[i] - 1, 99, 0, 5))
            elif i == 4:
                pcs.append((N - 1, 1, 0, 0, 99, i))
                in_p.append((0, 1, dst_offs[i], 99, 0, 5))
            else:
                pcs.append((src_offs[i], lens[i], dst_offs[i], 0, 0, i))
        # Guard against delta == 0
        if any(pc[4] == 0 and pc[2] == pc[0] for pc in pcs):
            continue
        try:
            _, stp, ops = g_uni.euclid_accelerate_2iet_bucket(pcs, in_p, slp)
            if stp > max_steps: max_steps = stp
            if ops > max_ops: max_ops = ops
        except AssertionError:
            pass
    print(f"  Random 5-piece IETs (N=100,000, 100 trials) in SubInt-1Pc: max_steps={max_steps}, max_ops={max_ops}")


def check_klayer_block0_with_selfrun():
    print("\n=== 3. APPLES-TO-APPLES k-LAYER INTERVAL DAG (+ SELF-RUN FALLBACK) ON block-0.L (to n <= 4096) ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    blk0 = manifest["blocks"][0]
    data = open(os.path.join("/tmp/mibz2-blocks", blk0["file"]), "rb").read()
    assert hashlib.sha256(data).hexdigest() == blk0["sha256"]
    n0 = len(data)
    syms0 = sorted(set(data))
    smap = {c: i for i, c in enumerate(syms0)}
    runs0 = []
    i = 0
    while i < n0:
        c = smap[data[i]]
        j = i + 1
        while j < n0 and smap[data[j]] == c:
            j += 1
        runs0.append((c, j - i))
        i = j
    r0 = len(runs0)
    s0 = len(syms0)

    def choose_k_layers(runs, n, s, k, reverse=False):
        counts = [0] * s
        for c, w in runs: counts[c] += w
        layer_of = [0] * s
        acc = 0; cur_l = 0; target = n / k
        for c in range(s):
            layer_of[c] = (k - 1 - cur_l) if reverse else cur_l
            acc += counts[c]
            if cur_l < k - 1 and acc >= (cur_l + 1) * target:
                cur_l += 1
        return layer_of

    def self_run_step_on_runs(runs, n, s):
        # Exact self-run step using Beta's deep_analysis
        C = [0] * (s + 1)
        for c, w in runs: C[c + 1] += w
        for c in range(s): C[c + 1] += C[c]
        # Build L array (only called when stalled, rare!) -> better: use Beta's run-space self_run_step!
        br = [[] for _ in range(s)]
        b = 0; pos = 0
        for c, w in runs:
            rem = w
            while rem > 0:
                while C[b + 1] <= pos: b += 1
                tk = min(rem, C[b + 1] - pos)
                br[b].append((c, tk))
                pos += tk; rem -= tk
        # Wait: call p1.db.self_run_step directly!
        rules = [('lit', c) for c in range(s)]
        res = p1.db.self_run_contraction_step(runs, s, n, 0, 0, [1] * s, list(range(s)), rules, C, br)
        out_runs, _, _, total_del, _, new_s, _ = res
        return out_runs, n - total_del, new_s

    for k_mode in (2, 3, 4, 999999):
        runs = list(runs0)
        n = n0
        s = s0
        rnd = 0
        self_rnds = 0
        tot_r_work = 0
        tot_pieces = 0
        max_r = r0
        while n > 4096 and rnd < 60:
            rev = (rnd % 2 == 1)
            layer_of = ([s - 1 - c if rev else c for c in range(s)] if k_mode >= s else choose_k_layers(runs, n, s, k_mode, reverse=rev))
            runs_next, n_next, s_next, st = run_k_layer_dag_step(runs, n, s, layer_of)
            if (n - n_next) < 0.05 * n:
                # Try opposite direction
                layer_of2 = ([s - 1 - c if not rev else c for c in range(s)] if k_mode >= s else choose_k_layers(runs, n, s, k_mode, reverse=not rev))
                rn2, nn2, sn2, st2 = run_k_layer_dag_step(runs, n, s, layer_of2)
                if nn2 < n_next:
                    runs_next, n_next, s_next, st = rn2, nn2, sn2, st2
            if (n - n_next) < 0.05 * n:
                # Self-run fallback!
                tot_r_work += len(runs)
                tot_pieces += len(runs)
                runs, n, s = self_run_step_on_runs(runs, n, s)
                self_rnds += 1
                rnd += 1
                if len(runs) > max_r: max_r = len(runs)
                continue
            tot_r_work += len(runs)
            tot_pieces += st["tot_pieces"]
            if len(runs_next) > max_r: max_r = len(runs_next)
            runs, n, s = runs_next, n_next, s_next
            rnd += 1
        k_lbl = "Full-DAG" if k_mode > 100 else f"k={k_mode}-Layer"
        print(
            f"  block-0.L [{k_lbl:9s}] -> n={n:5d} in {rnd:2d} rds (self={self_rnds}) | "
            f"max_r={max_r:6d} ({max_r/r0:.2f} r0) | sum_r={tot_r_work:7d} ({tot_r_work/r0:.2f} r0) | "
            f"tot_pieces={tot_pieces:7d} ({tot_pieces/r0:.2f} r0)"
        )


if __name__ == "__main__":
    check_stage2_and_subint()
    check_klayer_block0_with_selfrun()
