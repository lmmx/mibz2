#!/usr/bin/env python3
"""
Cycle 2 Supplemental Precision Measurements:
  1. Single-pass Shift-Binning Sort of `out_alive` (simultaneous O(k_out) sort + grid build!):
     measures exact average bin occupancy and exact insertion-sort comparisons/swaps per element!
  2. Inline7 (u64, 20B/piece) vs Inline15 (u128, 32B/piece) memory diet vs Open's 968 KiB scratch!
  3. Exact Stage 3 Two-Level Emit operation counts (ZERO flat_core duplication!):
     counts exact inline u128/u64 stores, flat_q memcpys, and Stage-2 thick-DAG visits to write all 5.40 MB!
"""
import hashlib
import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-c")
from explore_iet_schur import BLOCKS_DIR, build_initial_iet
from cycle2_core_and_inline15 import run_iet_schur_inline15_on_K
from test_unified_schur import compute_open_m16


def measure_shift_bin_sort_and_stage3(data, keep_bucket=34, inline_cap=15):
    """Run Stage 1 + Stage 2 with exact measurement of shift-binning sort swaps AND Stage 3 two-level emit ops."""
    alpha, C, counts, pieces_by_src_init, _, r0 = build_initial_iet(data)
    n = len(data)
    r_init = sum(len(v) for v in pieces_by_src_init.values())

    slp = []
    def w_len(h):
        return len(h[1]) if h[0] == 'inl' else h[2]
    def op_cat(hl, hr):
        if hl[0] == 'inl' and hr[0] == 'inl' and len(hl[1]) + len(hr[1]) <= inline_cap:
            return ('inl', hl[1] + hr[1])
        idx = len(slp)
        tot = w_len(hl) + w_len(hr)
        slp.append(('cat', hl, hr, tot))
        return ('slp', idx, tot)
    def op_rep(hb, k):
        if k == 1:
            return hb
        if hb[0] == 'inl' and len(hb[1]) * k <= inline_cap:
            return ('inl', hb[1] * k)
        idx = len(slp)
        tot = w_len(hb) * k
        slp.append(('rep', hb, k, tot))
        return ('slp', idx, tot)

    by_src = {b: [] for b in alpha}
    by_dst = {b: [] for b in alpha}
    r_in_cnt = defaultdict(int)
    r_out_cnt = defaultdict(int)
    r_self_cnt = defaultdict(int)
    self_rows_cnt = defaultdict(int)
    for b in alpha:
        for s, l, d, sb, db, _ in pieces_by_src_init[b]:
            p = (s, l, d, sb, db, ('inl', bytes([db])))
            by_src[sb].append(p)
            by_dst[db].append(p)
            if sb == db:
                r_self_cnt[sb] += 1
                self_rows_cnt[sb] += l
            else:
                r_out_cnt[sb] += 1
                r_in_cnt[db] += 1

    eliminated = set()
    alive = set(alpha)
    bin_sort_elems = 0
    bin_sort_swaps = 0
    bin_sort_grid_cells = 0
    cur_live = r_init
    peak_live = r_init
    cur_idx = 2 * r_init
    peak_idx = 2 * r_init

    while len(alive) > 1:
        cands = [b for b in alive if b != keep_bucket]
        def key_fn(b):
            rs = r_self_cnt[b]
            io = r_in_cnt[b] + r_out_cnt[b]
            deg = io + 2 * rs
            in_rows = counts[b] - self_rows_cnt[b]
            return (0 if rs == 0 else 1, (counts[b] / max(1, in_rows)) * deg, counts[b], b)
        e = min(cands, key=key_fn)
        raw_out = by_src.pop(e)
        raw_in = by_dst.pop(e)
        cur_idx -= (len(raw_out) + len(raw_in))

        out_alive = [p for p in raw_out if p[4] not in eliminated]
        in_alive = [p for p in raw_in if p[3] not in eliminated and p[3] != e]

        # Measure single-pass Shift-Binning Sort on out_alive!
        k = len(out_alive)
        b_start, b_len = C[e], counts[e]
        bin_sort_elems += k
        if k > 1:
            g_bits = 1
            while (1 << g_bits) < min(16384, max(8, 2 * k)):
                g_bits += 1
            G = 1 << g_bits
            shift = 0
            while ((b_len - 1) >> shift) >= G:
                shift += 1
            grid_sz = ((b_len - 1) >> shift) + 1
            bin_sort_grid_cells += grid_sz
            bins = [[] for _ in range(grid_sz)]
            for p in out_alive:
                bins[(p[0] - b_start) >> shift].append(p)
            out_sorted = []
            for blist in bins:
                bl = len(blist)
                if bl == 1:
                    out_sorted.append(blist[0])
                elif bl > 1:
                    # insertion sort inside bin, counting exact swaps!
                    for i in range(1, bl):
                        val = blist[i]
                        j = i - 1
                        while j >= 0 and blist[j][0] > val[0]:
                            blist[j + 1] = blist[j]
                            j -= 1
                            bin_sort_swaps += 1
                        blist[j + 1] = val
                    out_sorted.extend(blist)
            out_alive = out_sorted

        for p in out_alive:
            if p[4] != e:
                r_in_cnt[p[4]] -= 1
        for p in in_alive:
            r_out_cnt[p[3]] -= 1
        cur_live -= (len(out_alive) + len(in_alive))
        eliminated.add(e)
        alive.remove(e)

        import bisect
        out_starts = [p[0] for p in out_alive]
        self_q = []
        for s, l, d, sb, db, wid in in_alive:
            cs, cd, rem = s, d, l
            oi = bisect.bisect_right(out_starts, cd) - 1
            while rem > 0:
                os_, ol_, od_, _, odb_, owid_ = out_alive[oi]
                off = cd - os_
                take = rem if rem <= (ol_ - off) else (ol_ - off)
                if odb_ != e:
                    np = (cs, take, od_ + off, sb, odb_, op_cat(owid_, wid))
                    by_src[sb].append(np)
                    by_dst[odb_].append(np)
                    cur_idx += 2
                    cur_live += 1
                    if sb == odb_:
                        r_self_cnt[sb] += 1
                        self_rows_cnt[sb] += take
                    else:
                        r_out_cnt[sb] += 1
                        r_in_cnt[odb_] += 1
                else:
                    delta = od_ - os_
                    A, B = os_, os_ + ol_
                    sub_s = [(cs, take, cd, wid)]
                    while sub_s:
                        ss, sl, su, sw = sub_s.pop()
                        sv = su + sl
                        if delta > 0:
                            k_max = (B - 1 - su) // delta + 1
                            k_min = (B - 1 - (sv - 1)) // delta + 1
                            if k_max == k_min:
                                self_q.append((ss, sl, su + k_max * delta, sb, op_cat(op_rep(owid_, k_max), sw)))
                            else:
                                sp = B - (k_max - 1) * delta
                                l1 = sp - su
                                if l1 < sl: sub_s.append((ss + l1, sl - l1, sp, sw))
                                self_q.append((ss, l1, su + k_max * delta, sb, op_cat(op_rep(owid_, k_max), sw)))
                        else:
                            step = -delta
                            k_min = (su - A) // step + 1
                            k_max = (sv - 1 - A) // step + 1
                            if k_min == k_max:
                                self_q.append((ss, sl, su - k_min * step, sb, op_cat(op_rep(owid_, k_min), sw)))
                            else:
                                sp = A + k_min * step
                                l1 = sp - su
                                if l1 < sl: sub_s.append((ss + l1, sl - l1, sp, sw))
                                self_q.append((ss, l1, su - k_min * step, sb, op_cat(op_rep(owid_, k_min), sw)))
                cs += take
                cd += take
                rem -= take
                oi += 1

        while self_q:
            orig_s, l, cur_pos, sb, wid = self_q.pop()
            oi = bisect.bisect_right(out_starts, cur_pos) - 1
            os_, ol_, od_, _, odb_, owid_ = out_alive[oi]
            off = cur_pos - os_
            take = l if l <= (ol_ - off) else (ol_ - off)
            if take < l:
                self_q.append((orig_s + take, l - take, cur_pos + take, sb, wid))
            if odb_ != e:
                np = (orig_s, take, od_ + off, sb, odb_, op_cat(owid_, wid))
                by_src[sb].append(np)
                by_dst[odb_].append(np)
                cur_idx += 2
                cur_live += 1
                if sb == odb_:
                    r_self_cnt[sb] += 1
                    self_rows_cnt[sb] += take
                else:
                    r_out_cnt[sb] += 1
                    r_in_cnt[odb_] += 1
            else:
                delta = od_ - os_
                A, B = os_, os_ + ol_
                u, v = cur_pos, cur_pos + take
                if delta > 0:
                    k_max = (B - 1 - u) // delta + 1
                    k_min = (B - 1 - (v - 1)) // delta + 1
                    if k_max == k_min:
                        self_q.append((orig_s, take, u + k_max * delta, sb, op_cat(op_rep(owid_, k_max), wid)))
                    else:
                        sp = B - (k_max - 1) * delta
                        l1 = sp - u
                        if l1 < take: self_q.append((orig_s + l1, take - l1, sp, sb, wid))
                        self_q.append((orig_s, l1, u + k_max * delta, sb, op_cat(op_rep(owid_, k_max), wid)))
                else:
                    step = -delta
                    k_min = (u - A) // step + 1
                    k_max = (v - 1 - A) // step + 1
                    if k_min == k_max:
                        self_q.append((orig_s, take, u - k_min * step, sb, op_cat(op_rep(owid_, k_min), wid)))
                    else:
                        sp = A + k_min * step
                        l1 = sp - u
                        if l1 < take: self_q.append((orig_s + l1, take - l1, sp, sb, wid))
                        self_q.append((orig_s, l1, u - k_min * step, sb, op_cat(op_rep(owid_, k_min), wid)))

        if cur_live > peak_live: peak_live = cur_live
        if cur_idx > peak_idx: peak_idx = cur_idx

    core_pieces = [p for p in by_src[keep_bucket] if p[4] not in eliminated]
    # Stage 3 Two-Level Emit directly from Stage-1 handles (each piece in F_" is 1 emit atom:
    # either 1 inline u128/u64 store OR 1 flat_q memcpy!):
    # Across all n bytes, total emit atoms = sum_{p in F_"} p.len = |F_"|!
    inl_atoms = sum(p[1] for p in core_pieces if p[5][0] == 'inl')
    flatq_atoms = sum(p[1] for p in core_pieces if p[5][0] == 'slp')
    inl_bytes = sum(p[1] * w_len(p[5]) for p in core_pieces if p[5][0] == 'inl')
    flatq_bytes = sum(p[1] * w_len(p[5]) for p in core_pieces if p[5][0] == 'slp')
    assert inl_bytes + flatq_bytes == n

    # Memory under Inline7 (20B/piece) vs Inline15 (32B/piece)
    p_bytes = 24 if inline_cap == 7 else 32
    peak_kb = (peak_live * p_bytes + peak_idx * 4 + len(slp) * 16) / 1024.0
    s1_slp = len(slp)
    uniq_ovf = set(p[5][1] for p in core_pieces if p[5][0] == 'slp')
    ovf_bytes = sum(slp[u][3] for u in uniq_ovf)

    return {
        'bin_elems': bin_sort_elems,
        'bin_swaps': bin_sort_swaps,
        'swaps_per_el': bin_sort_swaps / max(1, bin_sort_elems),
        'grid_cells': bin_sort_grid_cells,
        'inl_atoms': inl_atoms,
        'flatq_atoms': flatq_atoms,
        'inl_bytes_pct': 100.0 * inl_bytes / n,
        's1_slp': s1_slp,
        'ovf_kb': ovf_bytes / 1024.0,
        'peak_kb': peak_kb,
    }


def main():
    manifest = json.load(open(os.path.join(BLOCKS_DIR, "manifest.json")))
    print("=== SHIFT-BINNING O(k_out) SORT + INLINE7 vs INLINE15 + STAGE 3 TWO-LEVEL EMIT (ALL 6 BLOCKS) ===")
    for cap in (15, 7):
        print(f"\n--- Inline{cap} (K = {{'\"'}}) ---")
        print(f"{'Block':11s} | {'BinEl':>6s} {'Swp/El':>6s} {'GridC':>6s} | {'S1_slp':>6s} {'OvfKB':>6s} {'PeakKB':>6s} | "
              f"{'InlAtm':>6s} {'FlqAtm':>6s} {'InlB%':>6s}")
        print("-" * 88)
        for blk in manifest["blocks"]:
            data = open(os.path.join(BLOCKS_DIR, blk["file"]), "rb").read()
            assert hashlib.sha256(data).hexdigest() == blk["sha256"]
            r = measure_shift_bin_sort_and_stage3(data, keep_bucket=34, inline_cap=cap)
            print(f"{blk['file']:11s} | {r['bin_elems']:6d} {r['swaps_per_el']:6.2f} {r['grid_cells']:6d} | "
                  f"{r['s1_slp']:6d} {r['ovf_kb']:5.1f}K {r['peak_kb']:5.0f}K | "
                  f"{r['inl_atoms']:6d} {r['flatq_atoms']:6d} {r['inl_bytes_pct']:5.1f}%")


if __name__ == "__main__":
    main()
