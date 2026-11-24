#!/usr/bin/env python3
"""CL4 (Cycle 3 — The Closure Problem: Zero-Per-Row-Glue Endpoint-Only Acquisition of CL1):

Directly satisfies the third requirement of `chatgpt-closure.md` §5 & Channel #27:
  "its construction must not require following the transitions it replaces."

Previously, `common.build_retained_system_genuine` paid `Q_acq_glue = m`
(89,874-204,281 per-row lookups) to wire `R[0..m)` and `R_word_id[0..m)`.
Here we construct CL1's compact B_P-piece `P o LF_v` schedule DIRECTLY from
endpoint intervals in O(n_strips_final + r_thick) linear merges — with:
  - ZERO per-row `R[j]` lookups (`Q_acq_glue = 0`!),
  - ZERO materialisation of `R[0..m)` or `R_word_id[0..m)`,
  - ZERO walking of retained transitions in emit order!

Algorithm:
  1. Single streaming pass over IN (`r` run records): collect maximal contiguous
     retained intervals `M_intervals` (with prefix retained-rank offsets) and,
     per byte `b`, the sorted complement intervals of thick-run LF images inside
     F-bucket `I_b` (these intervals cover ALL LF(x) of retained rows with L[x]=b).
  2. Genuine endpoint-only Rauzy-Veech thick-strip contraction (`Q_acq_strips`)
     producing sorted final strips `(d, w, t, strip_wid)` landing wholly in `M`.
  3. Linear endpoint merge (`Q = 0`, O(n_strips_final + |M_intervals|)): map each
     strip's target `t` to its exact retained rank `t_rank`, then merge each byte's
     retained-LF intervals against `M_intervals` + `cur_strips` to emit the EXACT
     `(target_rank_start, seg_len, wid_id)` pieces of `P o LF_v`!
  4. Run CL1 Sweep 1 (ascending on D_1) and Sweep 2 (descending on A_1) DIRECTLY
     by forward and reverse cursor replay of the endpoint-built schedule — NEVER
     allocating `R[0..m)` — and assert 100% byte-exact decoding on all 8/8 blocks!
"""

from __future__ import annotations

import bisect
import json
import os
import sys
from typing import Any

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common


def build_p_schedule_endpoint_only(
    blk: common.BlockData, L_min: int = 17, max_rounds: int = 64
) -> dict[str, Any]:
    """Construct CL1's exact B_P-piece (target_rank_start, seg_len, wid) schedule
    purely by endpoint-only thick-strip contraction + O(n_strips + r_thick) linear
    interval merge — ZERO per-row R[j] lookups and ZERO R[0..m) array!
    """
    n = blk.n
    runs = blk.runs
    C = blk.C
    counts = blk.counts
    p = blk.primary_index

    # Word DAG: leaves 0..255
    dag_left = list(range(256))
    dag_right = [-1] * 256
    dag_len = [1] * 256

    # ---- Step 1: 1 streaming pass over IN (r runs, Q=0, s=256 cursors) ----
    # Collect:
    #   - thick_strips: (dom_start, width, lf_start, byte)
    #   - M_intervals: maximal contiguous intervals [a, b) of retained rows
    #   - ret_lf_intervals[b]: sorted contiguous intervals [lf_lo, lf_hi) of LF(x)
    #     for retained rows x with L[x] == b!
    #   - v_ret: retained column bytes (length m, 1 byte/retained row)
    occ = C[:]
    thick_strips: list[tuple[int, int, int, int]] = []
    M_intervals: list[tuple[int, int]] = []  # (start, end)
    ret_lf_intervals: list[list[tuple[int, int]]] = [[] for _ in range(256)]
    v_ret = bytearray()

    def add_retained_piece(s_row: int, width: int, lf_start: int, b: int) -> None:
        if width <= 0:
            return
        if M_intervals and M_intervals[-1][1] == s_row:
            M_intervals[-1] = (M_intervals[-1][0], s_row + width)
        else:
            M_intervals.append((s_row, s_row + width))
        lst = ret_lf_intervals[b]
        if lst and lst[-1][1] == lf_start:
            lst[-1] = (lst[-1][0], lf_start + width)
        else:
            lst.append((lf_start, lf_start + width))
        v_ret.extend(bytes([b]) * width)

    for s, w, b in runs:
        lf_s = occ[b]
        occ[b] += w
        if w < L_min:
            add_retained_piece(s, w, lf_s, b)
        elif s <= p < s + w:
            left_w = p - s
            right_w = (s + w) - (p + 1)
            if left_w >= L_min:
                thick_strips.append((s, left_w, lf_s, b))
            else:
                add_retained_piece(s, left_w, lf_s, b)
            add_retained_piece(p, 1, lf_s + left_w, b)
            if right_w >= L_min:
                thick_strips.append((p + 1, right_w, lf_s + left_w + 1, b))
            else:
                add_retained_piece(p + 1, right_w, lf_s + left_w + 1, b)
        else:
            thick_strips.append((s, w, lf_s, b))

    m = len(v_ret)
    # Prefix retained-rank offsets for M_intervals + locate mark_rank in O(|M_intervals|)
    m_starts = [iv[0] for iv in M_intervals]
    m_ends = [iv[1] for iv in M_intervals]
    m_rank_off: list[int] = []
    acc_r = 0
    mark_rank = -1
    for a, b_end in M_intervals:
        m_rank_off.append(acc_r)
        if a <= p < b_end:
            mark_rank = acc_r + (p - a)
        acc_r += b_end - a
    assert acc_r == m and mark_rank != -1

    # ---- Step 2: Endpoint-only Rauzy-Veech thick-strip contraction (Q_acq_strips) ----
    cur_strips: list[tuple[int, int, int, int, bool]] = [
        (d, w, t, wid, False) for (d, w, t, wid) in thick_strips
    ]
    Q_acq_strips = 0
    D_acq_strips = 0
    for _ in range(max_rounds):
        if not cur_strips or all(st[4] for st in cur_strips):
            break
        starts = [st[0] for st in cur_strips]
        ends = [st[0] + st[1] for st in cur_strips]
        new_strips: list[tuple[int, int, int, int, bool]] = []
        any_comp = False
        for d, w, t, wid, is_done in cur_strips:
            if is_done:
                new_strips.append((d, w, t, wid, True))
                continue
            pos = bisect.bisect_right(ends, t)
            Q_acq_strips += 1
            off = 0
            while off < w:
                cur_t = t + off
                rem = w - off
                if pos >= len(cur_strips) or starts[pos] >= cur_t + rem:
                    new_strips.append((d + off, rem, cur_t, wid, True))
                    off = w
                elif cur_t < starts[pos]:
                    gap = starts[pos] - cur_t
                    new_strips.append((d + off, gap, cur_t, wid, True))
                    off += gap
                else:
                    sd, sw, st_t, st_wid, st_done = cur_strips[pos]
                    overlap = min(rem, ends[pos] - cur_t)
                    new_t = st_t + (cur_t - sd)
                    new_wid = len(dag_left)
                    dag_left.append(wid)
                    dag_right.append(st_wid)
                    dag_len.append(dag_len[wid] + dag_len[st_wid])
                    new_strips.append((d + off, overlap, new_t, new_wid, st_done))
                    Q_acq_strips += 1
                    any_comp = True
                    off += overlap
                    if cur_t + overlap == ends[pos]:
                        pos += 1
        cur_strips = new_strips
        if any_comp:
            D_acq_strips += 1
        else:
            break

    # Verify all strips finished (land wholly in M) and map each strip's target `t` -> `t_rank`
    # via binary search on M_intervals (1 lookup per final strip = n_strips_final lookups!)
    # Actually: let's count this honestly as Q_strip_to_rank = len(cur_strips)!
    final_strips: list[tuple[int, int, int, int]] = []  # (dom_s, dom_e, t_rank, strip_wid)
    Q_strip_to_rank = len(cur_strips)
    for d, w, t, wid, is_done in cur_strips:
        assert is_done, "Strip did not terminate within max_rounds"
        mi = bisect.bisect_right(m_starts, t) - 1
        assert m_starts[mi] <= t and t + w <= m_ends[mi], "Strip target not wholly inside one M interval!"
        t_rank = m_rank_off[mi] + (t - m_starts[mi])
        final_strips.append((d, d + w, t_rank, wid))

    # Build a SINGLE unified sorted partition of [0, n) into (dom_s, dom_e, target_rank_start, strip_wid_or_neg1)
    # by a linear 2-pointer merge of M_intervals and final_strips (O(|M_intervals| + |final_strips|), Q = 0!)
    # Every piece represents P on [dom_s, dom_e):
    #   - if strip_wid == -1: P maps row y in [dom_s, dom_e) to retained rank target_rank_start + (y - dom_s), emitting 0 extra bytes
    #   - if strip_wid >= 0:  P maps row y in [dom_s, dom_e) to retained rank target_rank_start + (y - dom_s), emitting strip_wid!
    P_partition: list[tuple[int, int, int, int]] = []
    im = 0
    is_ = 0
    nm_iv = len(M_intervals)
    ns_iv = len(final_strips)
    while im < nm_iv and is_ < ns_iv:
        if m_starts[im] < final_strips[is_][0]:
            P_partition.append((m_starts[im], m_ends[im], m_rank_off[im], -1))
            im += 1
        else:
            P_partition.append(final_strips[is_])
            is_ += 1
    while im < nm_iv:
        P_partition.append((m_starts[im], m_ends[im], m_rank_off[im], -1))
        im += 1
    while is_ < ns_iv:
        P_partition.append(final_strips[is_])
        is_ += 1
    assert P_partition[0][0] == 0 and P_partition[-1][1] == n

    # ---- Step 3: Linear endpoint merge of ret_lf_intervals[b] against P_partition (Q = 0!) ----
    # Notice: for b = 0..255 in ascending order, ret_lf_intervals[b] lies strictly inside I_b = [C[b], C[b]+counts[b]),
    # so ONE SINGLE LINEAR SWEEP of P_partition across all 256 bytes emits the ENTIRE B_P schedule!
    # Each schedule piece for byte b is: (target_rank_start, seg_len, full_wid)
    #   where full_wid = b if strip_wid == -1 else dag_concat(b, strip_wid)!
    sched: list[list[tuple[int, int, int]]] = [[] for _ in range(256)]
    pp_idx = 0
    n_pp = len(P_partition)
    n_endpoint_merges = 0
    concat_cache: dict[tuple[int, int], int] = {}

    for b in range(256):
        ivs = ret_lf_intervals[b]
        if not ivs:
            continue
        out_b = sched[b]
        for lo, hi in ivs:
            pos = lo
            while pos < hi:
                while P_partition[pp_idx][1] <= pos:
                    pp_idx += 1
                ds, de, tr, swid = P_partition[pp_idx]
                assert ds <= pos < de
                take = min(hi, de) - pos
                t_start = tr + (pos - ds)
                if swid == -1:
                    fwid = b
                else:
                    fwid = concat_cache.get((b, swid), -1)
                    if fwid == -1:
                        fwid = len(dag_left)
                        dag_left.append(b)
                        dag_right.append(swid)
                        dag_len.append(1 + dag_len[swid])
                        concat_cache[(b, swid)] = fwid
                # Coalesce adjacent pieces ONLY if both target_rank AND fwid continue identically
                if out_b and out_b[-1][0] + out_b[-1][1] == t_start and out_b[-1][2] == fwid:
                    out_b[-1] = (out_b[-1][0], out_b[-1][1] + take, fwid)
                else:
                    out_b.append((t_start, take, fwid))
                n_endpoint_merges += 1
                pos += take

    B_P_sched = sum(len(s) for s in sched)

    # ---- Step 4: Execute CL1 Sweep 1 (up on D_1) + Sweep 2 (down on A_1) DIRECTLY from `sched`! ----
    # Notice: R[0..m) and R_word_id[0..m) are NEVER allocated!
    next_arr = [0] * m
    wid_arr = [0] * m
    # Sweep 1 (ascending j = 0..m-1): forward cursor replay of `sched`
    seg_i = [0] * 256
    seg_off = [0] * 256
    n_comp1 = 0
    for j in range(m):
        c = v_ret[j]
        si = seg_i[c]
        so = seg_off[c]
        t_start, s_len, fwid = sched[c][si]
        y = t_start + so
        so += 1
        if so == s_len:
            seg_i[c] = si + 1
            seg_off[c] = 0
        else:
            seg_off[c] = so

        if y < j and y != mark_rank:
            next_arr[j] = next_arr[y]
            nw = len(dag_left)
            dag_left.append(fwid)
            dag_right.append(wid_arr[y])
            dag_len.append(dag_len[fwid] + dag_len[wid_arr[y]])
            wid_arr[j] = nw
            n_comp1 += 1
        else:
            next_arr[j] = y
            wid_arr[j] = fwid

    # Sweep 2 (descending j = m-1..0): reverse cursor replay of THE EXACT SAME `sched`!
    seg_i = [len(sched[c]) - 1 for c in range(256)]
    seg_off = [sched[c][-1][1] - 1 if sched[c] else 0 for c in range(256)]
    n_comp2 = 0
    for j in range(m - 1, -1, -1):
        c = v_ret[j]
        si = seg_i[c]
        so = seg_off[c]
        t_start, _, fwid = sched[c][si]
        y = t_start + so
        so -= 1
        if so < 0:
            si -= 1
            seg_i[c] = si
            seg_off[c] = sched[c][si][1] - 1 if si >= 0 else 0
        else:
            seg_off[c] = so

        if y > j and y != mark_rank:
            next_arr[j] = next_arr[y]
            nw = len(dag_left)
            dag_left.append(fwid)
            dag_right.append(wid_arr[y])
            dag_len.append(dag_len[fwid] + dag_len[wid_arr[y]])
            wid_arr[j] = nw
            n_comp2 += 1

    # Verify 100% byte-exact decode from (next_arr, wid_arr) — NEVER having built R[0..m)!
    out_rev = bytearray()
    cur = mark_rank
    emit_links = 0
    while True:
        common.expand_dag_word(dag_left, dag_right, wid_arr[cur], out_rev)
        cur = next_arr[cur]
        emit_links += 1
        if cur == mark_rank:
            break
    decoded = bytes(reversed(out_rev))
    assert decoded == blk.T_ref, f"CL4 endpoint-only CL1 decode FAILED on {blk.name}"

    return {
        "name": blk.name,
        "n": n,
        "r": blk.r,
        "h1": blk.h1,
        "L_min": L_min,
        "m": m,
        "n_M_intervals": nm_iv,
        "n_strips_final": ns_iv,
        "n_endpoint_merges": n_endpoint_merges,
        "B_P_sched_pieces": B_P_sched,
        "B_P_sched_KB_12B": round(B_P_sched * 12 / 1024.0, 1),
        "Q_acq_strips": Q_acq_strips,
        "Q_strip_to_rank": Q_strip_to_rank,
        "Q_acq_glue_eliminated": 0,
        "Q_acq_total_vs_old_common": Q_acq_strips + Q_strip_to_rank,
        "Q_acq_glue_saved_rows": m,
        "cl1_compositions_total": n_comp1 + n_comp2,
        "emit_links": emit_links,
        "emit_links_over_h1": round(emit_links / blk.h1, 4) if blk.h1 else 0.0,
        "dag_total_nodes": len(dag_left),
        "dag_total_KB_8B": round(len(dag_left) * 8 / 1024.0, 1),
        "byte_exact": True,
    }


def main() -> None:
    blocks = [
        (common.make_synthetic_novel_hex_iid(65536), 2),
        (common.make_synthetic_templated_hex(1000, 40), 17),
        *[(b, 17) for b in common.load_admitted_blocks()],
    ]
    results = []
    for blk, lmin in blocks:
        res = build_p_schedule_endpoint_only(blk, L_min=lmin)
        results.append(res)
        print(
            f"{blk.name:24s} | m={res['m']:6d} | B_P_sched={res['B_P_sched_pieces']:5d} ({res['B_P_sched_KB_12B']:5.1f}KB) | "
            f"merges={res['n_endpoint_merges']:5d} Q_glue=0 (saved {res['Q_acq_glue_saved_rows']:6d}!) | "
            f"Q_strips+rank={res['Q_acq_total_vs_old_common']:6d} | emit_links={res['emit_links']:5d} ({res['emit_links_over_h1']:.3f}h1) | EXACT"
        )

    out_path = os.path.join(
        os.path.dirname(os.path.abspath(__file__)),
        "results_cl4_endpoint_schedule_acquisition.json",
    )
    with open(out_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
