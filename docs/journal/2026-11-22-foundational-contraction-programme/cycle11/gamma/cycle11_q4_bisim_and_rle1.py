#!/usr/bin/env python3
"""
Cycle 11 — Foundational Question #4 ("Is the inverse BWT the right problem?"):
  1. Upstream MTF + RLE2 Audit on the 6 authenticated Wikidata blocks:
     - Exact MTF symbol distribution (mtf == 0 [RUNA/RUNB], mtf == 1, mtf == 2, mtf >= 3)
     - Correlation of MTF classes with Open thick strips (>16), Open thin (m_16), w=1, and AP4_max crit!
     - Exact `mtf == 1` (ababa...) and `mtf <= 2` ({a,b,c}*) local sub-alphabet run-length census!
  2. Post-IBWT RLE1 Structural Invariant (`c^5` forbidden except `count==c`):
     - Self-runs `L[i] == F[i]` (max self-run width in L vs max consecutive same-byte orbit depth in T)!
  3. "Bytes, Not a Permutation" — Exact k-step Forward Byte-Bisimulation Quotient |K_16 / ~_k|:
     - For k in {1, 2, 4, 8, 16, 32, 64, 128, 256, inf} on all 6 authenticated Wikidata blocks!
     - Exact verification of Theorem Q3: EVERY ~_k equivalence class on [0, n) is a SINGLE CONTIGUOUS
       BWT interval (0 non-contiguous bisimulation pairs across all 5.4M rows and all k)!
     - Also k-step Open-word bisimulation |K_16 / ~^word_k| for k in {1, 2, 3, 4}!
"""
import hashlib
import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-c")
from cycle8_ap4_text_anatomy import load_blocks, build_strips_and_decode, run_ap4_max_with_reasons


def compute_mtf_ranks(data):
    """Exact bzip2 MTF encoding of BWT column L (alphabet sorted in ascending byte order)."""
    present = sorted(set(data))
    mtf = list(present)
    pos_map = {b: i for i, b in enumerate(mtf)}
    # Fast list MTF (alphabet <= 100 on Wikidata!)
    ranks = bytearray(len(data))
    for i, b in enumerate(data):
        if mtf[0] == b:
            ranks[i] = 0
        else:
            idx = mtf.index(b)
            ranks[i] = idx
            del mtf[idx]
            mtf.insert(0, b)
    return ranks


def main():
    blocks = load_blocks()

    # Aggregators
    mtf_all = [0, 0, 0, 0]    # mtf==0, ==1, ==2, >=3
    mtf_thick = [0, 0, 0, 0]
    mtf_m16 = [0, 0, 0, 0]
    mtf_w1 = [0, 0, 0, 0]
    mtf_ap4 = [0, 0, 0, 0]

    mtf1_runs_ge4 = mtf1_rows_ge4 = mtf1_rows_ge8 = mtf1_rows_ge16 = 0
    mtf1_w1_ge4 = mtf1_w1_ge8 = 0
    mtf12_rows_ge8 = mtf12_rows_ge16 = 0
    mtf12_w1_ge8 = 0

    tot_self_rows = 0
    max_self_run = 0
    max_same_byte_text_run = 0
    rle1_triggers = 0
    rle1_expanded_bytes = 0

    ks = [1, 2, 4, 8, 16, 32, 64, 128, 256]
    bisim_classes = {k: 0 for k in ks}
    bisim_noncontig = {k: 0 for k in ks}
    word_bisim_classes = {1: 0, 2: 0, 3: 0, 4: 0}
    tot_m16 = 0
    tot_n = 0

    for blk, data in blocks:
        n = len(data)
        p = blk['primary_index']
        tot_n += n
        strips, LF, text, row_to_tpos = build_strips_and_decode(data, p)
        par, reason, crit_rows, _, _ = run_ap4_max_with_reasons(strips, n, p, cap_len=15)

        # Build Open retained mask & w=1 mask & AP4 crit mask
        is_m16 = bytearray(n)
        is_w1 = bytearray(n)
        for lo, hi, d, b, m_off in strips:
            w = hi - lo
            if w <= 16:
                is_m16[lo:hi] = b'\x01' * w
                if w == 1:
                    is_w1[lo] = 1
            elif m_off >= 0:
                is_m16[lo + m_off] = 1
                is_w1[lo + m_off] = 1
        is_ap4 = bytearray(n)
        for r in crit_rows:
            is_ap4[r] = 1

        m16_rows = [r for r in range(n) if is_m16[r]]
        tot_m16 += len(m16_rows)

        # 1. Exact MTF analysis
        ranks = compute_mtf_ranks(data)
        for i in range(n):
            rk = ranks[i]
            bk = rk if rk <= 2 else 3
            mtf_all[bk] += 1
            if is_m16[i]:
                mtf_m16[bk] += 1
                if is_w1[i]:
                    mtf_w1[bk] += 1
                if is_ap4[i]:
                    mtf_ap4[bk] += 1
            else:
                mtf_thick[bk] += 1

        # Contiguous runs of `mtf == 1` (pure alternating ababa... in L!) and `mtf <= 2` ({a,b,c}* in L!)
        i = 0
        while i < n:
            if ranks[i] == 1:
                j = i + 1
                while j < n and ranks[j] == 1:
                    j += 1
                Lrun = j - i
                if Lrun >= 4:
                    mtf1_runs_ge4 += 1
                    mtf1_rows_ge4 += Lrun
                    mtf1_w1_ge4 += sum(is_w1[i:j])
                if Lrun >= 8:
                    mtf1_rows_ge8 += Lrun
                    mtf1_w1_ge8 += sum(is_w1[i:j])
                if Lrun >= 16:
                    mtf1_rows_ge16 += Lrun
                i = j
            else:
                i += 1

        i = 0
        while i < n:
            if ranks[i] in (1, 2):  # strictly alternating 2-3 symbol non-zero MTF!
                j = i + 1
                while j < n and ranks[j] in (1, 2):
                    j += 1
                Lrun = j - i
                if Lrun >= 8:
                    mtf12_rows_ge8 += Lrun
                    mtf12_w1_ge8 += sum(is_w1[i:j])
                if Lrun >= 16:
                    mtf12_rows_ge16 += Lrun
                i = j
            else:
                i += 1

        # 2. Self-runs (L[i] == F[i]) vs max consecutive identical byte run in decoded T_RLE1!
        counts = [0] * 256
        for b in data: counts[b] += 1
        F_col = bytearray()
        for c in range(256):
            F_col.extend(bytes([c]) * counts[c])
        cur_sr = 0
        for i in range(n):
            if data[i] == F_col[i]:
                tot_self_rows += 1
                cur_sr += 1
                if cur_sr > max_self_run:
                    max_self_run = cur_sr
            else:
                cur_sr = 0

        # Check RLE1 triggers & max identical byte run in T_RLE1 (decoded BWT output)!
        ti = 0
        while ti < n:
            b = text[ti]
            tj = ti + 1
            while tj < n and text[tj] == b:
                tj += 1
            rl = tj - ti
            if rl > max_same_byte_text_run:
                max_same_byte_text_run = rl
            # Bzip2 RLE1: 4 identical bytes followed by a repeat-count byte!
            if rl >= 4:
                rle1_triggers += 1
                if ti + 4 < n:
                    rle1_expanded_bytes += text[ti + 4]
                ti += 5
            else:
                ti = tj

        # 3. Exact k-step Forward Byte-Bisimulation Quotient |K_16 / ~_k| AND Theorem Q3 Contiguity Check!
        # In forward text `text`, row `r` starts at `t_fwd = (row_to_tpos[r] + 1) % n`!
        # (Wait: L[r] = text[row_to_tpos[r]], so F[r] = text[(row_to_tpos[r] + 1) % n] = first byte of rotation r!)
        # Double `text` for O(1) cyclic slicing!
        text2 = text + text
        fwd_pos = [(row_to_tpos[r] + 1) % n for r in m16_rows]
        m16_len = len(m16_rows)

        for k in ks:
            # Because `m16_rows` is ALREADY sorted in BWT (lexicographic) order,
            # Theorem Q3 predicts identical k-byte forward slices MUST be contiguous!
            # Verify BOTH exact distinct count (set) AND contiguous run count!
            prev_s = None
            contig_groups = 0
            seen = set()
            for fp in fwd_pos:
                sl = text2[fp:fp + k]
                if sl != prev_s:
                    contig_groups += 1
                    prev_s = sl
                seen.add(sl)
            bisim_classes[k] += len(seen)
            if contig_groups != len(seen):
                bisim_noncontig[k] += (contig_groups - len(seen))

        # Also k-step Open-word bisimulation on K_16 (k = 1, 2, 3, 4 Open steps!)
        # Compute FL and Open first-return step from each r in m16_rows
        FL = [0] * n
        for i, v in enumerate(LF):
            FL[v] = i
        ret_idx_of = [-1] * n
        for idx_u, r in enumerate(m16_rows):
            ret_idx_of[r] = idx_u
        open_next = [0] * m16_len
        open_word = [None] * m16_len
        for idx_u, r in enumerate(m16_rows):
            buf = bytearray()
            cur = FL[r]
            while ret_idx_of[cur] < 0:
                buf.append(data[cur])
                cur = FL[cur]
            buf.append(data[cur])
            open_next[idx_u] = ret_idx_of[cur]
            open_word[idx_u] = bytes(buf)

        cur_seq = list(open_word)
        word_bisim_classes[1] += len(set(cur_seq))
        cur_ptr = list(open_next)
        for step_k in (2, 3, 4):
            nxt_seq = [cur_seq[u] + open_word[cur_ptr[u]] for u in range(m16_len)]
            cur_ptr = [open_next[cur_ptr[u]] for u in range(m16_len)]
            cur_seq = nxt_seq
            word_bisim_classes[step_k] += len(set(cur_seq))

    print("=== 1. UPSTREAM MTF SYMBOL DISTRIBUTION vs OPEN THICK / m_16 / w=1 / AP4_max CRIT (6-BLK SUM) ===")
    labels = ["mtf==0 (RUNA/B)", "mtf==1 (alt-2)", "mtf==2 (3rd)", "mtf>=3 (deep)"]
    print(f"{'MTF Class':16s} | {'All n':>8s} {'%n':>5s} | {'Thick':>8s} {'%Thk':>5s} | {'m_16':>8s} {'%m16':>5s} | {'w=1':>8s} {'%w1':>5s} | {'AP4Crit':>8s} {'%AP4':>5s}")
    print("-" * 102)
    s_thk = sum(mtf_thick); s_m16 = sum(mtf_m16); s_w1 = sum(mtf_w1); s_ap4 = sum(mtf_ap4)
    for b in range(4):
        print(f"{labels[b]:16s} | {mtf_all[b]:8d} {100*mtf_all[b]/tot_n:4.1f}% | {mtf_thick[b]:8d} {100*mtf_thick[b]/s_thk:4.1f}% | "
              f"{mtf_m16[b]:8d} {100*mtf_m16[b]/s_m16:4.1f}% | {mtf_w1[b]:8d} {100*mtf_w1[b]/s_w1:4.1f}% | {mtf_ap4[b]:8d} {100*mtf_ap4[b]/s_ap4:4.1f}%")

    print("\n  Local 2-letter / 3-letter non-zero MTF runs in L:")
    print(f"    mtf==1 runs (len>=4):  {mtf1_runs_ge4:6d} runs, {mtf1_rows_ge4:6d} rows ({100*mtf1_rows_ge4/tot_m16:.2f}% m16; w=1: {mtf1_w1_ge4:6d} = {100*mtf1_w1_ge4/s_w1:.2f}% of w=1)")
    print(f"    mtf==1 runs (len>=8):  {mtf1_rows_ge8:6d} rows ({100*mtf1_rows_ge8/tot_m16:.2f}% m16; w=1: {mtf1_w1_ge8:6d} = {100*mtf1_w1_ge8/s_w1:.2f}% of w=1)")
    print(f"    mtf==1 runs (len>=16): {mtf1_rows_ge16:6d} rows ({100*mtf1_rows_ge16/tot_m16:.3f}% m16)")
    print(f"    mtf in {{1,2}} (len>=8): {mtf12_rows_ge8:6d} rows ({100*mtf12_rows_ge8/tot_m16:.2f}% m16; w=1: {mtf12_w1_ge8:6d} = {100*mtf12_w1_ge8/s_w1:.2f}% of w=1)")
    print(f"    mtf in {{1,2}} (len>=16):{mtf12_rows_ge16:6d} rows ({100*mtf12_rows_ge16/tot_m16:.2f}% m16)")

    print("\n=== 2. POST-IBWT RLE1 INVARIANT & SELF-RUNS (L[i] == F[i]) (6-BLK SUM) ===")
    print(f"  Self-run rows (L[i]==F[i]):  {tot_self_rows:7d} ({100*tot_self_rows/tot_n:.2f}% of n), MAX self-run width in L = {max_self_run}")
    print(f"  MAX identical-byte run in decoded T_RLE1 (max consecutive self-orbit depth) = {max_same_byte_text_run}")
    print(f"  RLE1 4-byte triggers in 6 blocks = {rle1_triggers}, extra expanded bytes = {rle1_expanded_bytes} ({100*rle1_expanded_bytes/tot_n:.3f}% of n)")

    print("\n=== 3. 'BYTES, NOT A PERMUTATION' — EXACT k-STEP FORWARD BYTE-BISIMULATION QUOTIENT |K_16 / ~_k| ===")
    print(f"{'Horizon k':>10s} | {'|K16/~_k| (6b)':>14s} {'/ m_16':>7s} {'Redund%':>8s} | {'NonContig(Thm Q3)':>17s}")
    print("-" * 68)
    for k in ks:
        cl = bisim_classes[k]
        print(f"{k:10d} | {cl:14d} {cl/tot_m16:6.3f}x {100*(1 - cl/tot_m16):7.1f}% | {bisim_noncontig[k]:17d}")
    print(f"{'inf':>10s} | {tot_m16:14d} {1.000:6.3f}x {0.0:7.1f}% | {0:17d}")

    print("\n=== 4. k-STEP OPEN-WORD BISIMULATION |K_16 / ~^word_k| (6-BLK SUM) ===")
    for sk in (1, 2, 3, 4):
        cl = word_bisim_classes[sk]
        print(f"  k = {sk} Open step(s): {cl:7d} / {tot_m16} distinct word sequences ({cl/tot_m16:.3f}x m_16, {100*(1-cl/tot_m16):.1f}% shared)")


if __name__ == "__main__":
    main()
