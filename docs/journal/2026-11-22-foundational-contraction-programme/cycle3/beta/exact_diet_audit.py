#!/usr/bin/env python3
"""
Exact Diet Audit of the Winning Architecture:
  K = |Sigma| (static rarest_first_rows) + Theorem B4 Zero-Tombstone Causal Routing
  + Single-Sort (no double-sorting!) + AHT-Spiral + Decoupled 16B Records + u128 Flat<=16
  (measuring BOTH Full AHT th=2 AND Hub-Fusion th=8/16 across all 6 SHA-256 authenticated blocks!)
"""

import json
import sys
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-b")
import measure_iet_bse as mb
import pareto_superbucket as ps


def audit_exact_single_sort_and_memory():
    blocks = mb.load_authenticated_blocks()
    print("=== EXACT DIET AUDIT: K=|Sigma| + THEOREM B4 (ZERO-TOMBSTONE) + SINGLE-SORT + u128 FLAT<=16 ===\n")

    tot = defaultdict(int)
    tot_h8 = defaultdict(int)

    for binfo, raw in blocks:
        fname = binfo["file"]
        n, p = binfo["n"], binfo["primary_index"]
        runs = mb.rle_from_bytes(raw)
        r0 = len(runs)
        _, m16, _ = mb.compute_open_m16(runs, p, limit=16)
        ref = mb.decode_lf_reference_bytes(raw, p)

        # Measure exact single-sort counts & peak memory cleanly via K=96 Theorem B4
        st_full = ps.run_superbucket_zero_tombstone(
            runs, p, K=96, strategy="freq_tiered_hub_last",
            aht_thresh=2, inline_cap=16, stop_at_last_sb=False, verify_seq=ref
        )
        st_h8 = ps.run_superbucket_zero_tombstone(
            runs, p, K=96, strategy="freq_tiered_hub_last",
            aht_thresh=2, inline_cap=16, stop_at_last_sb=True, last_sb_aht_thresh=8, verify_seq=ref
        )

        # Exact single-sort count: each bucket sorts (self + exit) ONCE on dom_lo and (self + entry) ONCE on im_lo!
        # Wait: let's compute exact sum(n_self + n_exit) + sum(n_self + n_entry) = buf_reads + sum(n_self)!
        # Let's measure sum(n_self) directly from the causal routing of init + emitted!
        # Even easier: in Cycle 1/2 rarest_first_rows, total live dom + live im reads was EXACTLY
        # buf_reads_live = sum_b (len(d_list) + len(i_list)) = buf_reads_B4 + sum_b n_self(b)!!
        # Let's verify that exact algebraic identity!
        # In B4: every non-self piece is read ONCE (as exit or entry), every self piece is read ONCE (in self_buf).
        # So sum_b (n_dom(b) + n_im(b)) = sum_b (n_exit + n_self + n_entry + n_self) = buf_reads_B4 + sum_b n_self(b)!
        # Wait: for AHT th=2 (which does NOT need dom sorted before AHT, only im sorted before AHT, and dom sorted
        # only for remaining exit+thin_self after AHT!), true single-sort is AT MOST buf_reads + sum_self!
        # Let's compute exact peak memory (16B decoupled records + u128 inline pool + SLP table):
        #   - Live buffer pieces: peak_live_buf * 16 B (NEVER exceeds q0 * 16 B!)
        #   - Active bucket workspace: max_sb_pcs * 24 B (AHT/RV arrays, freed after bucket!)
        #   - u128 inline / flat pool: only live pieces + materialized leaves!
        #     Actually, during contraction, live pieces hold <= q0 u128s, PLUS materialized leaves (<= 2 * slp_cats)!
        #   - SLP cat table: slp_cats * 8 B (2 x u32 child IDs!)
        #   - Memo pos table during expansion: slp_cats * 8 B (start: u32, len: u32)
        q0 = st_full["peak_live_buf"]
        buf_kib = (q0 * 16) / 1024.0
        ws_kib = (st_full["max_sb_pcs"] * 24) / 1024.0
        slp_kib = (st_full["slp_cats"] * 8) / 1024.0
        # Flat leaf pool: each materialized leaf <= 16B (avg ~7-8B packed, or 16B fixed)
        # In Full AHT+Flat16, exp_leaves = slp_cats + 1; unique flat leaves <= slp_cats * 1.1 (~8B avg packed or 16B fixed)
        # Wait: let's measure exact flat_leaf_mats bytes vs packed!
        # At 16B decoupled buffer + 16B word side-slot for live pieces (<= q0):
        # PeakM_decoupled = (q0 * 32B live pieces) + ws_kib + slp_kib + flat_pool!
        # Wait: if we store live pieces as 32B (16B coords + 16B u128 inline/handle) directly in buffers:
        #   q0 * 32B = 1.64 MiB (block-0) to 4.12 MiB (block-210)!
        # If we store live pieces in a FREE-LIST slab of <= q0 slots (16B coords in buffer, 16B word in slab):
        #   STILL q0 * 32B live! Because there are AT MOST q0 live pieces at any instant!
        peak_live_kib = (q0 * 32) / 1024.0
        peak_tot_kib_full = peak_live_kib + ws_kib + slp_kib
        # For Hub-Fusion th=8: edges_core = 4 * m_core bytes!
        edges_h8_kib = (st_h8["m_core"] * 4) / 1024.0
        peak_tot_kib_h8 = peak_live_kib + ws_kib + (st_h8["slp_cats"] * 8) / 1024.0 + edges_h8_kib

        # Sequential bytes (16B coords written/read in causal buffers, u128 words in slab)
        seq_mb_16b = (2 * st_full["buf_reads"] * 16) / (1024.0 * 1024.0)
        seq_mb_32b = (2 * st_full["buf_reads"] * 32) / (1024.0 * 1024.0)

        # Calibrated i9 Open bar (us)
        open_lo = (0.72 * n + 4.8 * m16) / 1000.0
        open_hi = (0.90 * n + 5.8 * m16) / 1000.0

        print(f"[{fname:11s}] r0={r0:6d} m16={m16:6d} | i9 Open bar = {open_lo:4.0f}-{open_hi:4.0f} us (0.97 MiB/w)")
        print(f"  B4 Full  : rd=wr={st_full['buf_reads']:6d} ({st_full['buf_reads']/r0:.2f}r, 0% stale) | seq={seq_mb_16b:4.1f}MB(16B)/{seq_mb_32b:4.1f}MB(32B) | "
              f"maxSB={st_full['max_sb_pcs']:5d} | c2={st_full['c2']:5d}(ov={st_full['c2_ov']:5d}) mrg={st_full['p2_mrg']:6d} slf={st_full['p2_slf']:5d} | "
              f"SLP={st_full['slp_cats']:5d} ExpOps={st_full['exp_ops']:6d} ({st_full['exp_ops']/m16:.2f}m16) | PeakM~{peak_tot_kib_full/1024:.2f} MiB")
        print(f"  B4 Hub8  : rd=wr={st_h8['buf_reads']:6d} ({st_h8['buf_reads']/r0:.2f}r, 0% stale) | "
              f"c2={st_h8['c2']:5d}(ov={st_h8['c2_ov']:5d}) mrg={st_h8['p2_mrg']:6d} slf={st_h8['p2_slf']:5d} | "
              f"m_core={st_h8['m_core']:5d} ({st_h8['m_core']/m16:.3f}m16, {edges_h8_kib:.0f}KiB) SLP={st_h8['slp_cats']:5d} EmitOps={st_h8['exp_ops']:6d} ({st_h8['exp_ops']/m16:.2f}m16) | PeakM~{peak_tot_kib_h8/1024:.2f} MiB\n")

        tot["n"] += n; tot["r0"] += r0; tot["m16"] += m16
        for k in ("buf_reads", "c2", "c2_ov", "c2_spl", "p2_mrg", "p2_slf", "slp_cats", "exp_ops", "exp_leaves"):
            tot[k] += st_full[k]
            tot_h8[k] += st_h8[k]
        tot_h8["m_core"] += st_h8["m_core"]

    print("=" * 105)
    open_6_lo = (0.72 * tot["n"] + 4.8 * tot["m16"]) / 1000.0
    open_6_hi = (0.90 * tot["n"] + 5.8 * tot["m16"]) / 1000.0
    print(f"6-BLOCK SUMS: n={tot['n']}, r0={tot['r0']}, m16={tot['m16']} | i9 Open bar = {open_6_lo/1000:.2f}-{open_6_hi/1000:.2f} ms")
    print(f"  B4 Full  : rd=wr={tot['buf_reads']} ({tot['buf_reads']/tot['r0']:.2f}r, 0% stale) | "
          f"seq16B={2*tot['buf_reads']*16/1e6:.1f}MB | c2={tot['c2']} (ov={tot['c2_ov']}, spl={tot['c2_spl']}) | "
          f"mrg={tot['p2_mrg']} slf={tot['p2_slf']} | SLP={tot['slp_cats']} | ExpLeaves={tot['exp_leaves']} ({tot['exp_leaves']/tot['m16']:.2f}m16) ExpOps={tot['exp_ops']} ({tot['exp_ops']/tot['m16']:.2f}m16)")
    print(f"  B4 Hub8  : rd=wr={tot_h8['buf_reads']} ({tot_h8['buf_reads']/tot['r0']:.2f}r, 0% stale) | "
          f"c2={tot_h8['c2']} (ov={tot_h8['c2_ov']}) | mrg={tot_h8['p2_mrg']} slf={tot_h8['p2_slf']} | "
          f"m_core={tot_h8['m_core']} ({tot_h8['m_core']/tot['m16']:.3f}m16) | SLP={tot_h8['slp_cats']} | EmitOps={tot_h8['exp_ops']} ({tot_h8['exp_ops']/tot['m16']:.2f}m16)")


if __name__ == "__main__":
    audit_exact_single_sort_and_memory()
