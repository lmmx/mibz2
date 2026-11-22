#!/usr/bin/env python3
"""
Cycle 5 Deep Structural Discovery Script:
  1. Thin-Tube Census (w in 2..16) on all 6 authenticated Wikidata blocks:
     - Exact-match 1-to-1 thin tubes: LF(S_t) == S_{t+1} (exact same width w, zero split!)
     - Unsplit inclusion thin tubes: LF(S_t) subset S_{t+1} (w(S_t) <= w(S_{t+1}) <= 16)
     - Distribution of tube length h, tube width w, and tube area A = w * h!
     - How many of Open's m_16 rows sit inside w>=2 tubes of area A >= 16 (and leverage = area / steps = w)!
  2. Test Tube-Collapsed Open IET:
     What if we collapse every exact-match w>=2 thin tube of area >= 16 (or h >= 2) into a
     SINGLE strip (or eliminate it via tube contraction!) — measuring exact remaining m_ret,
     exact tube-contraction work (per-run steps, NOT per-row!), and exact Open IET nodes/steps!
  3. Cross-Byte Co-Shifted Thin Bundles (Mission 2B):
     - Adjacent cross-byte thin pairs (j, j+1) where LF(j) and LF(j+1) are adjacent? (Check!)
     - 2-step co-shifting: adjacent thin rows (j, j+1) where LF^2(j+1) == LF^2(j) + 1!
     - Multi-byte repeating thin patterns in L whose shifted images co-travel in parallel!
"""
import bisect
import hashlib
import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-c")
from cycle5_open_anatomy import load_blocks, compute_maximal_runs_and_lf, simulate_open_endpoint


def analyze_thin_tubes(data, full_runs, limit=16):
    """
    Analyze exact-match thin tubes (LF(S_t) == S_{t+1}, w in 2..=limit) and
    inclusion thin chains (LF(S_t) subset S_{t+1}, both w in 2..=limit)!
    Also checks exact cross-byte co-shifting (1-step & 2-step LF^2 adjacency).
    """
    n = len(data)
    num_r = len(full_runs)
    run_starts = [r[0] for r in full_runs]

    # Map start_pos -> run_idx for O(1) exact-match check!
    start_to_idx = {r[0]: idx for idx, r in enumerate(full_runs)}

    # 1. Exact-match 1-to-1 thin edges: run i (2 <= w <= limit) -> run j where dst(i) == pos(j) AND w(i) == w(j)!
    exact_next = [-1] * num_r
    exact_prev = [-1] * num_r
    # Also relaxed inclusion: dst(i) >= pos(j) and dst(i)+w(i) <= pos(j)+w(j) with 2 <= w(j) <= limit
    incl_next = [-1] * num_r
    incl_in_deg = [0] * num_r

    for i, (pos, w, b, dst) in enumerate(full_runs):
        if 2 <= w <= limit:
            # Exact match?
            j = start_to_idx.get(dst, -1)
            if j != -1 and full_runs[j][1] == w:
                exact_next[i] = j
                exact_prev[j] = i
            # Inclusion into any thin run (w <= limit)?
            ti = bisect.bisect_right(run_starts, dst) - 1
            t_pos, t_w, _, _ = full_runs[ti]
            if t_w <= limit and dst + w <= t_pos + t_w:
                incl_next[i] = ti
                incl_in_deg[ti] += 1

    # Decompose exact_next into maximal paths/cycles (exact-match thin tubes!)
    visited = [False] * num_r
    tubes = []  # list of (width, length_h, list_of_run_idxs, is_cycle)
    for i in range(num_r):
        w = full_runs[i][1]
        if not (2 <= w <= limit) or visited[i]:
            continue
        if exact_prev[i] == -1:
            chain = []
            cur = i
            while cur != -1 and not visited[cur]:
                visited[cur] = True
                chain.append(cur)
                cur = exact_next[cur]
            tubes.append((w, len(chain), chain, False))
    # Any remaining unvisited are closed exact-match cycles
    for i in range(num_r):
        w = full_runs[i][1]
        if not (2 <= w <= limit) or visited[i]:
            continue
        chain = []
        cur = i
        while cur != -1 and not visited[cur]:
            visited[cur] = True
            chain.append(cur)
            cur = exact_next[cur]
        tubes.append((w, len(chain), chain, True))

    # Measure tube statistics for h >= 2 and for area w*h >= 16!
    t_h2_cnt = 0
    t_h2_strips = 0
    t_h2_rows = 0
    t_h2_saved_rows = 0  # keeping 1 anchor strip per tube saves (h-1)*w rows!
    t_area16_cnt = 0
    t_area16_strips = 0
    t_area16_rows = 0
    t_area16_saved = 0
    t_area32_cnt = 0
    t_area32_strips = 0
    t_area32_saved = 0
    max_h = 0
    max_area = 0
    area_hist = defaultdict(int)

    # Also build mask of runs inside exact tubes with saved_rows = (h-1)*w >= 16 (or h>=2)
    in_exact_tube_h2 = [False] * num_r
    in_exact_tube_a16 = [False] * num_r

    for w, h, chain, is_cyc in tubes:
        area = w * h
        saved = w * (h - 1) if h >= 2 else 0
        if h > max_h:
            max_h = h
        if area > max_area:
            max_area = area
        if h >= 2:
            t_h2_cnt += 1
            t_h2_strips += h
            t_h2_rows += area
            t_h2_saved_rows += saved
            for r_id in chain[:-1]:  # eliminate first h-1 strips of tube, keep last strip (or eliminate all h!)
                in_exact_tube_h2[r_id] = True
        if saved >= 16:
            t_area16_cnt += 1
            t_area16_strips += h
            t_area16_rows += area
            t_area16_saved += saved
            for r_id in chain[:-1]:
                in_exact_tube_a16[r_id] = True
        if saved >= 32:
            t_area32_cnt += 1
            t_area32_strips += h
            t_area32_saved += saved

    # Also measure RELAXED inclusion chains (where LF(S_t) subset S_{t+1} with S_{t+1} thin <= 16,
    # even if S_{t+1} is WIDER than S_t, e.g., merging a w=3 branch into a w=8 tube!):
    # Count how many thin strips (w in 2..16) are strictly interior to an inclusion tree of depth >= 1
    # (i.e. can be contracted into their target thin run without splitting it!)
    # Note: eliminating a strip S_i whose image lands strictly inside a thin run S_j saves w(S_i) rows,
    # and if S_i has NO incoming thin strip of equal width it's a leaf of the inclusion forest!
    incl_edges = sum(1 for x in incl_next if x != -1)
    incl_rows = sum(full_runs[i][1] for i in range(num_r) if incl_next[i] != -1)

    # 2. Cross-Byte Co-Shifted Thin Bundles (1-step & 2-step LF^2 adjacency!)
    # Build LF on all n rows (just for exact diagnostic measurement!)
    LF = [0] * n
    for pos, w, b, dst in full_runs:
        for off in range(w):
            LF[pos + off] = dst + off

    is_thin = bytearray(n)
    for pos, w, _, _ in full_runs:
        if w <= limit:
            is_thin[pos:pos + w] = b'\x01' * w

    cross_lf1_adj = 0
    cross_lf2_adj = 0
    cross_lf1_small_shift = 0  # |LF(j+1) - LF(j)| <= 16 when L[j] != L[j+1]
    for j in range(n - 1):
        if is_thin[j] and is_thin[j + 1] and data[j] != data[j + 1]:
            d1 = LF[j + 1] - LF[j]
            if d1 == 1:
                cross_lf1_adj += 1
            if -16 <= d1 <= 16:
                cross_lf1_small_shift += 1
            if LF[LF[j + 1]] == LF[LF[j]] + 1:
                cross_lf2_adj += 1

    # Also check: among ALL adjacent thin runs (w <= 16), how many gaps between thick strips exist,
    # and what is their mean length?
    m_16 = sum(r[1] for r in full_runs if r[1] <= limit)

    return {
        'm_16': m_16,
        't_h2_cnt': t_h2_cnt,
        't_h2_strips': t_h2_strips,
        't_h2_rows': t_h2_rows,
        't_h2_saved': t_h2_saved_rows,
        't_a16_cnt': t_area16_cnt,
        't_a16_strips': t_area16_strips,
        't_a16_saved': t_area16_saved,
        't_a32_cnt': t_area32_cnt,
        't_a32_strips': t_area32_strips,
        't_a32_saved': t_area32_saved,
        'max_h': max_h,
        'max_area': max_area,
        'incl_edges': incl_edges,
        'incl_rows': incl_rows,
        'cross_lf1_adj': cross_lf1_adj,
        'cross_lf1_small': cross_lf1_small_shift,
        'cross_lf2_adj': cross_lf2_adj,
        'tubes': tubes,
    }


def main():
    blocks = load_blocks()

    print("=== 1. EXACT-MATCH THIN TUBES (w in 2..16, LF(S_t) == S_{t+1}) ACROSS ALL 6 WIKIDATA BLOCKS ===")
    print(f"{'Block':11s} {'m_16':>7s} | {'Tubes':>6s} {'Strps':>6s} {'Sav(h>=2)':>9s} {'%m16':>5s} {'AvgLev':>6s} | "
          f"{'A>=16#':>6s} {'Strps':>6s} {'Sav>=16':>8s} {'%m16':>5s} | {'A>=32#':>6s} {'Sav>=32':>8s} {'%m16':>5s} | {'MaxH':>4s} {'MaxA':>5s}")
    print("-" * 134)

    tot = defaultdict(int)
    per_blk = []
    for blk, data in blocks:
        full_runs, C, counts = compute_maximal_runs_and_lf(data)
        res = analyze_thin_tubes(data, full_runs, limit=16)
        per_blk.append((blk, data, full_runs, res))
        m = res['m_16']
        avg_lev = res['t_h2_saved'] / max(1, res['t_h2_strips'] - res['t_h2_cnt'])
        print(f"{blk['file']:11s} {m:7d} | {res['t_h2_cnt']:6d} {res['t_h2_strips']:6d} {res['t_h2_saved']:9d} {100*res['t_h2_saved']/m:4.1f}% {avg_lev:5.2f}x | "
              f"{res['t_a16_cnt']:6d} {res['t_a16_strips']:6d} {res['t_a16_saved']:8d} {100*res['t_a16_saved']/m:4.1f}% | "
              f"{res['t_a32_cnt']:6d} {res['t_a32_saved']:8d} {100*res['t_a32_saved']/m:4.1f}% | {res['max_h']:4d} {res['max_area']:5d}")
        for k in ('m_16', 't_h2_cnt', 't_h2_strips', 't_h2_rows', 't_h2_saved',
                  't_a16_cnt', 't_a16_strips', 't_a16_saved', 't_a32_cnt', 't_a32_strips', 't_a32_saved',
                  'incl_edges', 'incl_rows', 'cross_lf1_adj', 'cross_lf1_small', 'cross_lf2_adj'):
            tot[k] += res[k]

    print("-" * 134)
    tm = tot['m_16']
    avg_lev = tot['t_h2_saved'] / max(1, tot['t_h2_strips'] - tot['t_h2_cnt'])
    print(f"{'6-BLK SUM':11s} {tm:7d} | {tot['t_h2_cnt']:6d} {tot['t_h2_strips']:6d} {tot['t_h2_saved']:9d} {100*tot['t_h2_saved']/tm:4.1f}% {avg_lev:5.2f}x | "
          f"{tot['t_a16_cnt']:6d} {tot['t_a16_strips']:6d} {tot['t_a16_saved']:8d} {100*tot['t_a16_saved']/tm:4.1f}% | "
          f"{tot['t_a32_cnt']:6d} {tot['t_a32_saved']:8d} {100*tot['t_a32_saved']/tm:4.1f}% |")

    print(f"\n=== 2. RELAXED INCLUSION (LF(S_t) subset thin S_{{t+1}}) & CROSS-BYTE CO-SHIFTING (6-BLK SUM) ===")
    print(f"  Relaxed inclusion (2<=w<=16 -> thin): {tot['incl_edges']} strips, {tot['incl_rows']} rows ({100*tot['incl_rows']/tm:.1f}% of m_16)")
    print(f"  Cross-byte 1-step adjacent (LF(j+1)==LF(j)+1):   {tot['cross_lf1_adj']} pairs ({100*tot['cross_lf1_adj']/tm:.3f}% of m_16)")
    print(f"  Cross-byte 1-step |delta|<=16:                   {tot['cross_lf1_small']} pairs ({100*tot['cross_lf1_small']/tm:.3f}% of m_16)")
    print(f"  Cross-byte 2-step adjacent (LF^2(j+1)==LF^2+1):  {tot['cross_lf2_adj']} pairs ({100*tot['cross_lf2_adj']/tm:.3f}% of m_16)")

    print("\n=== 3. TUBE LENGTH & WIDTH BREAKDOWN OF EXACT-MATCH THIN TUBES (6-BLK SUM) ===")
    by_h = defaultdict(lambda: [0, 0, 0])  # bucket -> [tubes, strips_eliminated, saved_rows]
    for _, _, _, res in per_blk:
        for w, h, chain, is_cyc in res['tubes']:
            if h < 2:
                continue
            saved = w * (h - 1)
            bk = "h=2" if h == 2 else ("h=3..4" if h <= 4 else ("h=5..8" if h <= 8 else ("h=9..16" if h <= 16 else ("h=17..32" if h <= 32 else "h>=33"))))
            by_h[bk][0] += 1
            by_h[bk][1] += (h - 1)
            by_h[bk][2] += saved
    for bk in ("h=2", "h=3..4", "h=5..8", "h=9..16", "h=17..32", "h>=33"):
        tc, se, sr = by_h[bk]
        print(f"  {bk:8s}: {tc:6d} tubes, {se:6d} elim strips, {sr:7d} saved rows ({100*sr/tm:4.1f}% of m_16, avg {sr/max(1,se):4.2f} rows/strip, {sr/max(1,tc):5.1f} rows/tube)")


if __name__ == "__main__":
    main()
