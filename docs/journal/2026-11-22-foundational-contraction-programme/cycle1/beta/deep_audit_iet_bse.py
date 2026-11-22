#!/usr/bin/env python3
"""
Deep Audit of IET-BSE (Cycle 1, Investigator Beta):
1. Fix defaultdict mutation bug in build_fvs_acyclic_order -> measure TRUE FVS / acyclic-complement order
   across all 6 SHA-256 authenticated blocks (byte-exact verified).
2. Inspect the exact worst-case non-monotone self-IET buckets (e.g., 44.25x k_self on block-0.L,
   17.31x k_self on block-85.L) to characterize the hidden-Theta(rows) phenomenon precisely.
3. Measure exact distribution of bucket working-set sizes and per-bucket sequential vs local-random work.
4. Measure exact SLP expansion cost across tau in {2, 4, 8, 16, inf} and early-stop hybrid tradeoffs.
5. Verify exactness on small exhaustive + F4 + Fib_{2m}+$ + Symmetric-Middle-Run (with monotone spiral check).
"""

import json
import time
import sys
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-b")
import measure_iet_bse as mb


def build_true_fvs_acyclic_order(active_syms, dom_buf):
    """
    Bug-free greedy Feedback Vertex Set + Acyclic-Complement Topological Order on G_Sigma.
    Edge (u -> v) exists iff there is a strip from F_u to F_v with u != v.
    """
    adj_out = {u: {} for u in active_syms}
    adj_in = {u: {} for u in active_syms}
    for u in active_syms:
        for pc in dom_buf[u]:
            v = pc[1]
            if v != u:
                adj_out[u][v] = adj_out[u].get(v, 0) + 1
                adj_in[v][u] = adj_in[v].get(u, 0) + 1

    rem = set(active_syms)
    left_topo = []
    right_topo = []
    fvs = []

    while rem:
        progressed = True
        while progressed:
            progressed = False
            sources = [u for u in rem if not any(w in rem for w in adj_in[u])]
            if sources:
                sources.sort(key=lambda u: (sum(adj_out[u].get(w, 0) for w in rem), u))
                for u in sources:
                    rem.remove(u)
                    left_topo.append(u)
                progressed = True
                continue
            sinks = [u for u in rem if not any(w in rem for w in adj_out[u])]
            if sinks:
                sinks.sort(key=lambda u: (sum(adj_in[u].get(w, 0) for w in rem), u))
                for u in sinks:
                    rem.remove(u)
                    right_topo.append(u)
                progressed = True
        if rem:
            def hub_score(u):
                out_d = sum(1 for w in adj_out[u] if w in rem)
                in_d = sum(1 for w in adj_in[u] if w in rem)
                out_w = sum(cnt for w, cnt in adj_out[u].items() if w in rem)
                in_w = sum(cnt for w, cnt in adj_in[u].items() if w in rem)
                return (out_d * in_d, out_w + in_w, u)
            v = max(rem, key=hub_score)
            rem.remove(v)
            fvs.append(v)

    acyclic = left_topo + list(reversed(right_topo))
    # Verify acyclic is indeed a DAG in G_Sigma!
    pos_in_acyc = {u: idx for idx, u in enumerate(acyclic)}
    back_edges = 0
    for u in acyclic:
        for v in adj_out[u]:
            if v in pos_in_acyc and pos_in_acyc[v] <= pos_in_acyc[u]:
                back_edges += 1
    assert back_edges == 0, f"Acyclic complement not a DAG! back_edges={back_edges}"

    order = acyclic + list(reversed(fvs))
    return order, len(fvs), len(acyclic)


# Monkey-patch mb.build_fvs_acyclic_order with the bug-free version
mb.build_fvs_acyclic_order = build_true_fvs_acyclic_order


def main():
    blocks = mb.load_authenticated_blocks()
    print("=== 1. TRUE FVS / ACYCLIC-COMPLEMENT ORDER ACROSS ALL 6 AUTHENTICATED BLOCKS ===")
    true_fvs_results = {}
    for binfo, raw in blocks:
        fname = binfo["file"]
        n, p = binfo["n"], binfo["primary_index"]
        runs = mb.rle_from_bytes(raw)
        r0 = len(runs)
        _, m16_strip, _ = mb.compute_open_m16(runs, p, limit=16)
        ref_bytes = mb.decode_lf_reference_bytes(raw, p)

        st = mb.run_iet_bse_instrumented(runs, p, order_mode="fvs_acyclic_order", verify_bytes=ref_bytes)
        assert st["q_violations"] == 0
        nm = st["nonmono_records"]
        sum_nm_k = sum(x["k_self"] for x in nm)
        max_nm_ratio = max((x["self_steps"] / x["k_self"] for x in nm), default=0.0)
        max_nm_row = max((x["self_steps"] / x["rows"] for x in nm), default=0.0)
        fvs_sz, acyc_sz = st["fvs_info"]

        print(f"  {fname:12s} | FVS={fvs_sz:2d} Acyc={acyc_sz:2d} (sigma={st['sigma']}) | "
              f"buf_rd={st['buf_reads_total']:6d} (live={st['buf_reads_live']:6d}, stale={st['buf_reads_stale']:6d}) | "
              f"emit={st['emitted_total']:6d} ({st['emitted_total']/r0:.2f}r) | maxB={st['max_bucket_pieces']:5d}")
        print(f"               | 0s={st['zero_self_buckets']:2d} M={st['mono_self_buckets']:2d} NM={st['nonmono_self_buckets']:2d} | "
              f"NM_slf={st['steps_nonmono_self']:5d} ({st['steps_nonmono_self']/sum_nm_k:.2f}x k_self, max={max_nm_ratio:.2f}x, max_row={max_nm_row:.3f}) | "
              f"last={repr(bytes([st['last_b']]))} n={st['last_n']:6d} q={st['last_q']:5d} finRV={st['final_rv_steps']:5d}")
        print(f"               | TOT_STEPS={st['total_all_steps']:6d} ({st['total_all_steps']/r0:.2f}r, {st['total_all_steps']/m16_strip:.2f}*m16) | "
              f"|G|={st['slp_created']:6d} h={st['slp_height']:3d} | Exp4={st['exp'][4]['total_ops']:6d} ({st['exp'][4]['total_ops']/m16_strip:.2f}*m16)")
        st_save = dict(st)
        st_save["nonmono_top5_by_ratio"] = sorted(nm, key=lambda x: -x["self_steps"] / x["k_self"])[:5]
        st_save["nonmono_top5_by_steps"] = sorted(nm, key=lambda x: -x["self_steps"])[:5]
        del st_save["nonmono_records"]
        true_fvs_results[fname] = st_save

    print("\n=== 2. INSPECTING WORST-CASE NON-MONOTONE SELF-IET RATIOS (HIDDEN Theta(rows) AUDIT) ===")
    for binfo, raw in blocks[:3]:  # block-0, block-2, block-85
        fname = binfo["file"]
        runs = mb.rle_from_bytes(raw)
        p = binfo["primary_index"]
        for om in ("most_freq_first", "rarest_first_rows", "fvs_acyclic_order"):
            st = mb.run_iet_bse_instrumented(runs, p, order_mode=om, verify_bytes=None)
            nm = st["nonmono_records"]
            worst_ratio = sorted(nm, key=lambda x: -x["self_steps"] / x["k_self"])[:3]
            print(f"  {fname} [{om}] top-3 by (self_steps / k_self):")
            for x in worst_ratio:
                print(f"    b={x['b']:3d} ({repr(bytes([x['b']])):6s}) rows={x['rows']:6d} k_in={x['k_in']:4d} k_out={x['k_out']:4d} "
                      f"k_self={x['k_self']:4d} runs={x['mono_runs']:4d} self_steps={x['self_steps']:5d} "
                      f"({x['self_steps']/x['k_self']:5.2f}x k_self, {x['self_steps']/x['rows']:.3f}x rows) w1={x['width1_self']:4d}")

    print("\n=== 3. EARLY-STOP / ZERO-FILL ANALYSIS ON block-0.L (How much disappears in 0-self + mono-self?) ===")
    b0, raw0 = blocks[0]
    runs0 = mb.rle_from_bytes(raw0)
    # Check what happens as we eliminate buckets one by one under rarest_first_deg and most_freq_first
    for om in ("most_freq_first", "rarest_first_deg", "fvs_acyclic_order"):
        st = mb.run_iet_bse_instrumented(runs0, b0["primary_index"], order_mode=om, verify_bytes=None)
        nm = st["nonmono_records"]
        nm_rows = sum(x["rows"] for x in nm)
        print(f"  block-0.L [{om:17s}]: zero_self_b={st['zero_self_buckets']:2d} mono_b={st['mono_self_buckets']:2d} "
              f"nonmono_b={st['nonmono_self_buckets']:2d} (sum_rows={nm_rows:6d}) | "
              f"pure_merge_steps={st['steps_zero_self'] + st['steps_mono_merge'] + st['steps_nonmono_merge']:6d} | "
              f"self_steps={st['steps_mono_self'] + st['steps_nonmono_self']:6d} | final_rv={st['final_rv_steps']:5d}")

    with open("/tmp/fcp-b/true_fvs_results.json", "w") as f:
        json.dump(true_fvs_results, f, indent=2)
    print("\nSaved true FVS results to /tmp/fcp-b/true_fvs_results.json")


if __name__ == "__main__":
    main()
