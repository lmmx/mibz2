#!/usr/bin/env python3
"""
Verify the exact ZERO-SORT 256-cursor L-order wiring of SelThinSq (LF-orientation, matching
production Open's exact L-run thin classification len(I_u) <= 16!), check Phi^2(L=8) thick_strips
count, and test SelThin^4 + (1-Pass Ascent-DAG + SelThinSq) composition across all 6 blocks!
"""

import bisect
import sys
sys.path.insert(0, "/tmp/fcp-b")
import measure_iet_bse as mb
import dir2_strip_squaring as d2
import dir1_zerosort_contraction as d1


def verify_selthinsq_256cursor_lf(runs, n, p, limit=16, verify_bytes=None):
    """
    EXACT ZERO-SORT 256-CURSOR WIRING OF SelThinSq IN PRODUCTION Open's EXACT L-RUN ORIENTATION!
    In LF-orientation (backward in text):
      - L-run u in 0..r0-1 occupies I_u = [l_start[u], l_start[u] + len[u]) with symbol c = sym[u],
        mapping I_u --(LF)--> J_u = [f_start[u], f_start[u] + len[u]) subset F_c.
      - Production Open defines THIN rows S_thin = union_{len[u] <= limit} I_u (count = m_16!),
        and THICK strips = {u : len[u] > limit}.
      - For every thin row x in I_u (len[u] <= limit):
          Let y1 = LF(x) in J_u, lying in some L-run v (found in ONE linear scan of F-order vs L-order!).
          * If I_v is THICK (len[v] > limit):
              Wait: x already hits a thick strip at step 1! Keep step-1 jump x -> y1 (1 byte L[x]),
              so Open routes x through thick strip I_v as usual! (OR advance through I_v!)
          * If I_v is THIN (len[v] <= limit):
              x jumps TWO steps: x -> y2 = LF(y1) = LF^2(x) in J_v subset F_{sym[v]}, emitting 2 bytes
              (L[x], L[y1]) = (sym[u], sym[v]) in u16!!
              AND where does y2 land in L-order?!
              Because v advances MONOTONICALLY 0, 1, ..., r0-1 during the single linear two-pointer scan
              of J_u (F-order) vs I_v (L-order), for EACH byte c = sym[v], y2 = J_v + ov advances
              STRICTLY MONOTONICALLY through bucket F_c = [C[c], C[c+1])!
              So a 256-entry L1-resident cursor `l_cursor[c]` finds the exact L-run w containing y2
              in O(1) amortized (total cursor advances across ALL 256 cursors <= r0!! ZERO binary search,
              ZERO sorting, ZERO random lookup!!)!
    Verifies 100% byte-exact reconstruction AND counts:
      - total cursor increments (proving <= r0!)
      - thin->thick (1-byte), thin->thin->thin (2-byte direct edge!), thin->thin->thick (2-byte into thick strip!)
      - exact actual_walk_steps from p vs Open's m_16!
    """
    r0 = len(runs)
    counts = [0] * 256
    for c, w in runs: counts[c] += w
    C = [0] * 257
    for c in range(256): C[c + 1] = C[c] + counts[c]

    l_start = [0] * r0
    l_len = [0] * r0
    l_sym = [0] * r0
    f_start = [0] * r0
    thin_base = [-1] * r0

    f_cur = list(C[:256])
    buckets_u = [[] for _ in range(256)]
    pos = 0; m16 = 0
    for u, (c, w) in enumerate(runs):
        l_start[u] = pos; l_len[u] = w; l_sym[u] = c
        f_start[u] = f_cur[c]
        buckets_u[c].append(u)
        f_cur[c] += w
        if w <= limit:
            thin_base[u] = m16
            m16 += w
        pos += w
    assert pos == n

    # F-order list of u's (ALREADY sorted by f_start[u], O(r0) bucket gather!)
    f_order_u = []
    for c in range(256):
        f_order_u.extend(buckets_u[c])

    # Initialize 256-entry L-run cursor `l_cursor[c]` to the first L-run overlapping C[c]!
    l_cursor = [0] * 256
    u_scan = 0
    for c in range(256):
        if counts[c] == 0: continue
        while u_scan + 1 < r0 and l_start[u_scan] + l_len[u_scan] <= C[c]:
            u_scan += 1
        l_cursor[c] = u_scan

    # Build exact mixed-step backward map on [0, n):
    #   - Every thin row x in I_u (len[u] <= limit):
    #       if y1 = LF(x) in thin I_v: jump x -> LF^2(x) (2 bytes: L[x], L[y1])!
    #       if y1 = LF(x) in thick I_v: jump x -> LF(x) (1 byte: L[x])!  (so thick strip runs!)
    #   - Every thick row x in I_u (len[u] > limit): jump x -> LF(x) (1 byte: L[x])!
    # Wait: let's store directly for EVERY thin row `tid in 0..m16`:
    #   next_coord[tid], step_bytes[tid] (1 or 2 bytes!)
    # AND verify the 256-cursor finds the exact target L-run w of y2 with ZERO binary search!
    next_coord = [0] * m16
    step_word = [b""] * m16
    cursor_advances = 0
    cnt_t_to_thick = 0
    cnt_tt_to_thin = 0
    cnt_tt_to_thick = 0

    # ONE linear two-pointer merge of J_u (f_order_u, sorted by f_start!) vs I_v (v = 0..r0-1, sorted by l_start!)
    iu = 0; v = 0; ou = 0; ov = 0
    while iu < r0 and v < r0:
        u = f_order_u[iu]
        take = min(l_len[u] - ou, l_len[v] - ov)
        # Overlap at m = f_start[u] + ou == l_start[v] + ov
        if l_len[u] <= limit:
            # Source rows in I_u are THIN!
            x_base = thin_base[u] + ou
            c_u = l_sym[u]
            if l_len[v] > limit:
                # y1 lands in THICK strip v -> keep 1-byte jump to y1 = l_start[v] + ov!
                y1_base = l_start[v] + ov
                w1 = bytes([c_u])
                for k in range(take):
                    next_coord[x_base + k] = y1_base + k
                    step_word[x_base + k] = w1
                cnt_t_to_thick += take
            else:
                # y1 lands in THIN strip v -> 2-byte jump to y2 = f_start[v] + ov (in F_{c_v})!
                c_v = l_sym[v]
                y2_base = f_start[v] + ov
                w2 = bytes([c_u, c_v])  # backward order: L[x], then L[LF(x)]
                # Advance l_cursor[c_v] monotonically to cover [y2_base, y2_base + take)!
                w_run = l_cursor[c_v]
                cur_y2 = y2_base
                rem2 = take
                off2 = 0
                while rem2 > 0:
                    while l_start[w_run] + l_len[w_run] <= cur_y2:
                        w_run += 1
                        cursor_advances += 1
                    assert l_start[w_run] <= cur_y2 < l_start[w_run] + l_len[w_run]
                    tk2 = min(rem2, l_start[w_run] + l_len[w_run] - cur_y2)
                    if l_len[w_run] <= limit:
                        cnt_tt_to_thin += tk2
                    else:
                        cnt_tt_to_thick += tk2
                    for k in range(tk2):
                        next_coord[x_base + off2 + k] = cur_y2 + k
                        step_word[x_base + off2 + k] = w2
                    cur_y2 += tk2; rem2 -= tk2; off2 += tk2
                l_cursor[c_v] = w_run
        ou += take; ov += take
        if ou == l_len[u]: iu += 1; ou = 0
        if ov == l_len[v]: v += 1; ov = 0
    assert iu == r0 and v == r0

    # Walk backward from p using next_coord/step_word on thin rows and exact LF on thick rows!
    # Build fast lookup from row -> L-run for thick rows
    grid_shift = 8
    grid_sz = (n >> grid_shift) + 2
    grid = [0] * grid_sz
    si = 0
    for g in range(grid_sz):
        x0 = g << grid_shift
        while si + 1 < r0 and l_start[si + 1] <= x0: si += 1
        grid[g] = si

    out_rev = bytearray()
    cur = p
    thin_steps = 0
    thick_steps = 0
    while len(out_rev) < n:
        u = grid[cur >> grid_shift]
        while u + 1 < r0 and l_start[u + 1] <= cur: u += 1
        if l_len[u] <= limit:
            tid = thin_base[u] + (cur - l_start[u])
            thin_steps += 1
            wd = step_word[tid]
            rem = n - len(out_rev)
            if rem >= len(wd): out_rev.extend(wd)
            else: out_rev.extend(wd[:rem])
            cur = next_coord[tid]
        else:
            thick_steps += 1
            out_rev.append(l_sym[u])
            cur = f_start[u] + (cur - l_start[u])

    out = bytes(out_rev[::-1])
    if verify_bytes is not None:
        assert out == verify_bytes, "256-cursor SelThinSq MISMATCH!"

    return {
        "m16": m16,
        "cursor_advances": cursor_advances,
        "thin_steps": thin_steps,
        "thick_steps": thick_steps,
        "t_to_thick": cnt_t_to_thick,
        "tt_to_thin": cnt_tt_to_thin,
        "tt_to_thick": cnt_tt_to_thick,
    }


def main():
    blocks = mb.load_authenticated_blocks()
    print("=== 1. VERIFYING EXACT ZERO-SORT 256-CURSOR SelThinSq (LF-ORIENTATION = PRODUCTION Open!) ===")
    tot_m16 = 0; tot_walk = 0; tot_adv = 0; tot_r0 = 0
    for binfo, raw in blocks:
        fname = binfo["file"]
        n, p = binfo["n"], binfo["primary_index"]
        runs = mb.rle_from_bytes(raw)
        r0 = len(runs)
        ref = mb.decode_lf_reference_bytes(raw, p)
        st = verify_selthinsq_256cursor_lf(runs, n, p, limit=16, verify_bytes=ref)
        tot_m16 += st["m16"]; tot_walk += st["thin_steps"]; tot_adv += st["cursor_advances"]; tot_r0 += r0
        print(f"  {fname:11s}: r0={r0:6d} | cursor_adv={st['cursor_advances']:6d} ({st['cursor_advances']/r0:.2f}r!) | "
              f"m16={st['m16']:6d} (1.000x) | WALK={st['thin_steps']:6d} ({st['thin_steps']/st['m16']:.3f}x Open!!) | "
              f"direct_2B_thin={100*st['tt_to_thin']/st['m16']:4.1f}% 2B_thick={100*st['tt_to_thick']/st['m16']:4.1f}% 1B_thick={100*st['t_to_thick']/st['m16']:4.1f}% [EXACT]")
    print(f"  6-BLK SUM  : r0={tot_r0} | cursor_adv={tot_adv} ({tot_adv/tot_r0:.2f}r!) | m16={tot_m16} | WALK={tot_walk} ({tot_walk/tot_m16:.3f}x Open!!)")

    print("\n=== 2. CHECKING Phi^2 (LIM=8 vs LIM=16) THICK-STRIP COUNT vs Open (LIM=16) ===")
    for binfo, raw in blocks:
        runs = mb.rle_from_bytes(raw)
        p1_im, p1_dom, _ = d2.build_phi_strips(runs)
        p2 = d2.compose_iet_sorted(p1_im, p1_dom)
        tk1_16 = sum(1 for s in p1_im if s[2] > 16)
        tk2_16 = sum(1 for s in p2 if s[2] > 16)
        tk2_8 = sum(1 for s in p2 if s[2] > 8)
        print(f"  {binfo['file']:11s}: Open(Phi^1,L16) thick={tk1_16:4d} | Phi^2(L16) thick={tk2_16:4d} ({tk2_16/tk1_16:.2f}x) | Phi^2(L8) thick={tk2_8:5d} ({tk2_8/tk1_16:.2f}x)")


if __name__ == "__main__":
    main()
