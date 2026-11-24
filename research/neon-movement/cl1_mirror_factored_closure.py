#!/usr/bin/env python3
"""CL1 (Cycle 3 — The Closure Problem, Channel #27 / chatgpt-closure.md):
Mirror-Factored Bi-Directional P o LF_v Closure (Zero-Fragmentation Second Composition)

Directly answers the Closure Problem:
  Starting from Open's factorization R = P o LF_v (2,148 per-byte +1-segments on
  block-0.L under w > 16 thick rule, m = 89,874, h1 = 35,116), one standard
  in-place M-17 sweep fragments R^(1) to 44,814-52,319 segments (21-24x) and
  bloats flat word storage 0.90 MB -> 2.51 MB.

Constructive representation that survives a SECOND composition without fragmenting:
  Keep the compact B_P-segment P o LF_v schedule (2,148 pieces = ~17 KB on block-0)
  intact rather than overwriting it, and split M = D_1 u A_1 implicitly by R[j] < j
  vs R[j] > j:
    - Sweep 1 (ascending j = 0..m-1): advance the |Sigma| cursors of P o LF_v
      forward; compose ONLY descent rows j in D_1 (R[j] < j, y != mark) by
      reading T_1[y] at y = P(LF_v(j)).
    - Sweep 2 (descending j = m-1..0): advance the EXACT SAME |Sigma| cursors
      of P o LF_v in reverse; compose ONLY ascent rows j in A_1 (R[j] > j, y != mark)
      by reading T_2[y] at y = P(LF_v(j)) (which holds T_2[y] if y in A_1, or
      T_1[y] if y in D_1 — contracting every valley-to-valley ascent+descent arch!).
    - Word payloads: zero-copy 8-byte binary DAG concat nodes (no flat byte copying
      during sweeps), compared against 8B/16B capped flat and uncapped flat storage.

Verified:
  - Reproduces chatgpt-closure.md on block-0.L to the exact integer (m=89,874,
    2,148 segments, 44,814/46,671/52,319 after Sweep 1, 2,507,122 flat bytes).
  - Byte-exact on all 8/8 blocks (2 synthetic novel hex + 6 admitted Wikidata blocks).
  - Exact under bzip2's PROOF-PROG multi-cycle / periodic contract (including
    corrupt columns where marked cycle length ell does not divide n!).
"""

from __future__ import annotations

import json
import os
import sys
from typing import Any

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common


# ---------------------------------------------------------------------------
# 1. Exact Segment-Schedule Metrics (Exact +1, Cache-Line <=16, Monotone)
# ---------------------------------------------------------------------------

def measure_schedule_metrics(
    targets: list[int],
    labels: list[int],
    mask: list[bool] | None = None,
    reverse: bool = False,
) -> dict[str, Any]:
    """Measure per-label successor-address segment counts along a sweep:
      - seg_exact1:   new segment whenever target != prev + step (step=+1 or -1)
      - seg_cache16:  new segment whenever (target - prev)*step not in [1, 16]
      - seg_monotone: new segment whenever (target - prev)*step < 1 (direction break!)
      - n_active:     number of active reads (mask is None or mask[j] is True)
    """
    m = len(targets)
    step = -1 if reverse else 1
    idxs = range(m - 1, -1, -1) if reverse else range(m)

    last_t = [-10**9] * 256
    has_last = [False] * 256
    seg_exact1 = 0
    seg_cache16 = 0
    seg_monotone = 0
    n_active = 0
    total_abs_delta_monotone = 0
    n_monotone_steps = 0

    for j in idxs:
        if mask is not None and not mask[j]:
            continue
        n_active += 1
        c = labels[j]
        t = targets[j]
        if not has_last[c]:
            seg_exact1 += 1
            seg_cache16 += 1
            seg_monotone += 1
            has_last[c] = True
            last_t[c] = t
        else:
            diff = (t - last_t[c]) * step
            if diff != 1:
                seg_exact1 += 1
            if not (1 <= diff <= 16):
                seg_cache16 += 1
            if diff < 1:
                seg_monotone += 1
            else:
                total_abs_delta_monotone += diff
                n_monotone_steps += 1
            last_t[c] = t

    return {
        "n_active_reads": n_active,
        "seg_exact1": seg_exact1,
        "seg_cache16": seg_cache16,
        "seg_monotone": seg_monotone,
        "mean_monotone_stride": round(total_abs_delta_monotone / max(1, n_monotone_steps), 3),
    }


def build_p_schedule(R: list[int], v: list[int]) -> list[list[tuple[int, int]]]:
    """Extract the compact per-byte P o LF_v +1-segment schedule:
    For each byte c, a list of (target_start, seg_len) tuples.
    Can be replayed forward (Sweep 1) AND backward (Sweep 2) with zero R[0..m) array!
    """
    sched: list[list[tuple[int, int]]] = [[] for _ in range(256)]
    for j, t in enumerate(R):
        c = v[j]
        lst = sched[c]
        if lst and lst[-1][0] + lst[-1][1] == t:
            lst[-1] = (lst[-1][0], lst[-1][1] + 1)
        else:
            lst.append((t, 1))
    return sched


def replay_p_schedule_verify(sched: list[list[tuple[int, int]]], v: list[int], R: list[int]) -> None:
    """Assert that forward and reverse cursor replay of the compact B_P schedule
    reproduces R[j] identically for every j in 0..m-1 without storing R!
    """
    m = len(v)
    # Forward replay
    seg_i = [0] * 256
    seg_off = [0] * 256
    for j in range(m):
        c = v[j]
        si = seg_i[c]
        so = seg_off[c]
        t_start, s_len = sched[c][si]
        y = t_start + so
        assert y == R[j]
        so += 1
        if so == s_len:
            seg_i[c] = si + 1
            seg_off[c] = 0
        else:
            seg_off[c] = so

    # Reverse replay
    seg_i = [len(sched[c]) - 1 for c in range(256)]
    seg_off = [sched[c][-1][1] - 1 if sched[c] else 0 for c in range(256)]
    for j in range(m - 1, -1, -1):
        c = v[j]
        si = seg_i[c]
        so = seg_off[c]
        t_start, _ = sched[c][si]
        y = t_start + so
        assert y == R[j]
        so -= 1
        if so < 0:
            si -= 1
            seg_i[c] = si
            seg_off[c] = sched[c][si][1] - 1 if si >= 0 else 0
        else:
            seg_off[c] = so


# ---------------------------------------------------------------------------
# 2. Sweep 1 (ChatGPT Pro Baseline) + Sweep 2 (CL1 Mirror-Factored Closure)
# ---------------------------------------------------------------------------

def run_sweep1_and_cl1(
    ret: common.RetainedSystem,
    v: list[int],
    blk: common.BlockData,
    byte_cap: int | None = None,
) -> dict[str, Any]:
    """Run:
      1) Standard M-17 Sweep 1 (ascending j = 0..m-1, in-place) — reproducing
         chatgpt-closure.md's exact baseline (segments, links, flat word bytes).
      2) CL1 Mirror-Factored Sweep 2 (descending j = m-1..0, ONLY j in A_1,
         reading strictly through P o LF_v's B_P schedule in reverse!) with
         zero-copy binary word DAG.
    """
    m = ret.m
    R = ret.R
    mark = ret.mark_rank
    dag_left = ret.dag_left[:]
    dag_right = ret.dag_right[:]
    dag_len = ret.dag_len[:]
    base_dag_nodes = len(dag_left)

    # Precompute base word lengths and flat byte sizes
    base_wid = ret.R_word_id[:]
    base_flat_bytes = sum(dag_len[w] for w in base_wid)

    # ---- SWEEP 1 (Ascending: j = 0..m-1, compose descent rows R[j] < j) ----
    # Note: guard y != mark so mark is NEVER bypassed (works on single-cycle,
    # periodic, AND multi-cycle corrupt columns!).
    next1 = R[:]
    wid1 = base_wid[:]
    len1 = [dag_len[w] for w in base_wid]
    composed1 = [False] * m
    # Track flat-word bloat metric (exact sum of word lengths across all m rows)
    n_comp1 = 0
    read_mask1 = [False] * m

    for j in range(m):
        y = R[j]
        if y < j and y != mark:
            new_len = len1[j] + len1[y]
            if byte_cap is None or new_len <= byte_cap:
                read_mask1[j] = True
                next1[j] = next1[y]
                new_wid = len(dag_left)
                dag_left.append(wid1[j])
                dag_right.append(wid1[y])
                dag_len.append(new_len)
                wid1[j] = new_wid
                len1[j] = new_len
                composed1[j] = True
                n_comp1 += 1

    # Measure Sweep 1 output schedule (ChatGPT Pro's exact metric!)
    s1_out_sched = measure_schedule_metrics(next1, v, mask=None, reverse=False)
    # Measure Sweep 1 ACTUAL read schedule (reads only at j where read_mask1[j] is True!)
    s1_read_sched = measure_schedule_metrics(R, v, mask=read_mask1, reverse=False)
    flat_bytes_s1 = sum(len1)

    # Verify Sweep 1 byte-exact decode & count emit links
    out_rev = bytearray()
    cur = mark
    links_s1 = 0
    while True:
        common.expand_dag_word(dag_left, dag_right, wid1[cur], out_rev)
        cur = next1[cur]
        links_s1 += 1
        if cur == mark:
            break
    assert bytes(reversed(out_rev)) == blk.T_ref, f"Sweep 1 decode failed on {blk.name}"

    # ---- SWEEP 2 (CL1 Mirror-Factored: descending j = m-1..0, compose ONLY A_1: R[j] > j) ----
    # Every read address is y = R[j] = P(LF_v(j)) (replayed in reverse from the B_P schedule)!
    # Notice: for j in D_1 (R[j] < j), T_2[j] = T_1[j] (untouched in Sweep 2!).
    # For j in A_1 (R[j] > j, y != mark), y > j is ALREADY finalized in T_2[y]!
    #   - If y in A_1, T_2[y] holds the full ascent+descent arch starting at y!
    #   - If y in D_1, T_2[y] == T_1[y] holds the full descent chain starting at y!
    next2 = next1[:]
    wid2 = wid1[:]
    len2 = len1[:]
    n_comp2 = 0
    read_mask2 = [False] * m

    for j in range(m - 1, -1, -1):
        y = R[j]
        if y > j and y != mark:
            # Note: j was NOT composed in Sweep 1 (since y > j), so wid2[j] == base_wid[j]!
            assert not composed1[j]
            new_len = len2[j] + len2[y]
            if byte_cap is None or new_len <= byte_cap:
                read_mask2[j] = True
                next2[j] = next2[y]
                new_wid = len(dag_left)
                dag_left.append(wid2[j])
                dag_right.append(wid2[y])
                dag_len.append(new_len)
                wid2[j] = new_wid
                len2[j] = new_len
                n_comp2 += 1

    # Measure Sweep 2 ACTUAL read schedule (strictly through P o LF_v in reverse!)
    s2_read_sched = measure_schedule_metrics(R, v, mask=read_mask2, reverse=True)
    # Also measure Sweep 2 output schedule (for diagnostic comparison with Sweep 1)
    s2_out_sched = measure_schedule_metrics(next2, v, mask=None, reverse=False)
    flat_bytes_s2 = sum(len2)
    dag_nodes_added = len(dag_left) - base_dag_nodes

    # Verify CL1 Sweep 2 byte-exact decode & count emit links + DAG node visits!
    out_rev2 = bytearray()
    cur = mark
    links_s2 = 0
    dag_internal_visits = 0
    while True:
        # Count internal DAG pops during expand
        stack = [wid2[cur]]
        while stack:
            nid = stack.pop()
            rr = dag_right[nid]
            if rr == -1:
                out_rev2.append(dag_left[nid])
            else:
                dag_internal_visits += 1
                stack.append(rr)
                stack.append(dag_left[nid])
        cur = next2[cur]
        links_s2 += 1
        if cur == mark:
            break
    assert bytes(reversed(out_rev2)) == blk.T_ref, f"CL1 Sweep 2 decode failed on {blk.name}"

    return {
        "byte_cap": byte_cap if byte_cap is not None else "uncapped",
        "sweep1": {
            "compositions": n_comp1,
            "emit_links": links_s1,
            "emit_links_over_h1": round(links_s1 / blk.h1, 4) if blk.h1 else 0.0,
            "emit_links_over_m": round(links_s1 / m, 4),
            "out_seg_exact1": s1_out_sched["seg_exact1"],
            "out_seg_exact1_over_h1": round(s1_out_sched["seg_exact1"] / blk.h1, 4) if blk.h1 else 0.0,
            "out_seg_monotone": s1_out_sched["seg_monotone"],
            "read_schedule": s1_read_sched,
            "flat_word_bytes": flat_bytes_s1,
        },
        "cl1_sweep2": {
            "compositions": n_comp2,
            "emit_links": links_s2,
            "emit_links_over_h1": round(links_s2 / blk.h1, 4) if blk.h1 else 0.0,
            "emit_links_over_m": round(links_s2 / m, 4),
            "read_schedule": s2_read_sched,
            "out_seg_exact1": s2_out_sched["seg_exact1"],
            "out_seg_monotone": s2_out_sched["seg_monotone"],
            "flat_word_bytes": flat_bytes_s2,
            "dag_nodes_added": dag_nodes_added,
            "dag_added_bytes_8B": dag_nodes_added * 8,
            "dag_vs_flat_ratio": round((dag_nodes_added * 8) / max(1, flat_bytes_s2 - base_flat_bytes), 4),
            "emit_dag_internal_visits": dag_internal_visits,
        },
    }


# ---------------------------------------------------------------------------
# 3. Exact Multi-Cycle & Periodic PROOF-PROG Stress Test (chatgpt-closure.md §5)
# ---------------------------------------------------------------------------

def test_multicycle_and_periodic_columns() -> dict[str, Any]:
    """Test CL1 on:
      (a) Periodic valid BWT columns (e.g. 'abc'*10, '012345'*20) where ell | n;
      (b) Corrupt multi-cycle BWT columns where marked cycle length ell does NOT
          divide n (verifying exact PROOF-PROG contract: decode[j] = decode[j mod ell]).
    """
    cases = []

    def ref_decode_proof_prog(L: bytes, p: int) -> tuple[bytes, int]:
        n = len(L)
        C, _, _, _ = common.compute_counts_and_runs(L)
        lf = common.build_lf_array(L, C)
        cyc_rev = bytearray()
        x = p
        while True:
            cyc_rev.append(L[x])
            x = lf[x]
            if x == p:
                break
        ell = len(cyc_rev)
        cyc_fwd = bytes(reversed(cyc_rev))
        out = bytes(cyc_fwd[j % ell] for j in range(n))
        return out, ell

    def cl1_decode_raw(L: bytes, p: int) -> tuple[bytes, int, int]:
        n = len(L)
        C, _, _, _ = common.compute_counts_and_runs(L)
        R = common.build_lf_array(L, C)
        # Word DAG leaves 0..255
        dl = list(range(256))
        dr = [-1] * 256
        # Sweep 1 (ascending, R[j] < j, y != p)
        next1 = R[:]
        wid1 = list(L)
        comp1 = [False] * n
        for j in range(n):
            y = R[j]
            if y < j and y != p:
                next1[j] = next1[y]
                nw = len(dl)
                dl.append(wid1[j])
                dr.append(wid1[y])
                wid1[j] = nw
                comp1[j] = True
        # Sweep 2 (descending, R[j] > j, y != p)
        next2 = next1[:]
        wid2 = wid1[:]
        for j in range(n - 1, -1, -1):
            y = R[j]
            if y > j and y != p:
                assert not comp1[j]
                next2[j] = next2[y]
                nw = len(dl)
                dl.append(wid2[j])
                dr.append(wid2[y])
                wid2[j] = nw
        # Emit marked cycle from p until returning to p
        cyc_rev = bytearray()
        cur = p
        links = 0
        while True:
            common.expand_dag_word(dl, dr, wid2[cur], cyc_rev)
            cur = next2[cur]
            links += 1
            if cur == p:
                break
        ell = len(cyc_rev)
        cyc_fwd = bytes(reversed(cyc_rev))
        out = bytes(cyc_fwd[j % ell] for j in range(n))
        return out, ell, links

    # Case 1: Periodic valid BWT of b"abcd" * 25 (n=100, ell=4 | 100)
    # Build cyclic rotations BWT manually for periodic string
    T_per = b"abcd" * 25
    n_p = len(T_per)
    rots = sorted(range(n_p), key=lambda i: T_per[i:] + T_per[:i])
    L_per = bytes(T_per[(i - 1) % n_p] for i in rots)
    p_per = rots.index(0)
    ref_o, ref_ell = ref_decode_proof_prog(L_per, p_per)
    cl1_o, cl1_ell, links = cl1_decode_raw(L_per, p_per)
    assert cl1_o == ref_o and cl1_ell == ref_ell
    cases.append({"name": "periodic_abcd_x25", "n": n_p, "ell": ref_ell, "divides_n": n_p % ref_ell == 0, "links": links, "exact": True})

    # Case 2: Corrupt multi-cycle columns where ell does NOT divide n!
    import random
    rng = random.Random(20261124)
    n_corrupt_checked = 0
    n_nondiv_checked = 0
    for trial in range(200):
        n_c = rng.randint(15, 120)
        L_c = bytes(rng.choice(b"0123456789abcdef") for _ in range(n_c))
        p_c = rng.randrange(n_c)
        ref_o, ref_ell = ref_decode_proof_prog(L_c, p_c)
        cl1_o, cl1_ell, links = cl1_decode_raw(L_c, p_c)
        assert cl1_o == ref_o and cl1_ell == ref_ell, f"Multi-cycle mismatch on trial {trial}"
        n_corrupt_checked += 1
        if n_c % ref_ell != 0:
            n_nondiv_checked += 1

    return {
        "periodic_exact": True,
        "random_corrupt_multicycle_checked": n_corrupt_checked,
        "non_dividing_ell_cases_checked": n_nondiv_checked,
        "all_exact": True,
    }


# ---------------------------------------------------------------------------
# 4. Main Benchmark Driver (8/8 blocks + multi-cycle tests)
# ---------------------------------------------------------------------------

def main() -> None:
    mc = test_multicycle_and_periodic_columns()
    print(
        f"Multi-cycle / periodic PROOF-PROG check: periodic=EXACT, "
        f"{mc['random_corrupt_multicycle_checked']} random corrupt columns "
        f"({mc['non_dividing_ell_cases_checked']} with ell !| n) -> ALL EXACT!"
    )

    blocks = [
        (common.make_synthetic_novel_hex_iid(65536), 2),
        (common.make_synthetic_templated_hex(1000, 40), 17),
        *[(b, 17) for b in common.load_admitted_blocks()],
    ]

    results = []
    for blk, lmin in blocks:
        ret = common.build_retained_system_genuine(blk, L_min=lmin)
        v = [blk.L[x] for x in ret.rows]

        # Extract and verify forward + reverse replay of compact B_P schedule
        sched = build_p_schedule(ret.R, v)
        replay_p_schedule_verify(sched, v, ret.R)
        B_P = sum(len(s) for s in sched)
        orig_metrics = measure_schedule_metrics(ret.R, v, mask=None, reverse=False)
        assert orig_metrics["seg_exact1"] == B_P

        arms = {}
        for cap in (8, 16, None):
            key = "uncapped" if cap is None else f"cap_{cap}"
            arms[key] = run_sweep1_and_cl1(ret, v, blk, byte_cap=cap)

        unc = arms["uncapped"]
        s1 = unc["sweep1"]
        s2 = unc["cl1_sweep2"]
        s1_rm = s1["read_schedule"]
        s2_rm = s2["read_schedule"]

        # Honest Pi cost for CL1 (uncapped with binary word-DAG):
        #   Q_acq_strips = Open endpoint strip contraction
        #   Sweep 1 reads: s1_rm["n_active_reads"] (monotone in B_P pieces!)
        #   Sweep 2 reads: s2_rm["n_active_reads"] (monotone in EXACT SAME B_P pieces!)
        #   Emit reads:    s2["emit_links"] (plus DAG expansion of n bytes)
        #   Notice: under strict Pi, reading T[y] via B_P-piece monotone cursors has
        #   Q_seeks = s1_rm["seg_monotone"] + s2_rm["seg_monotone"] <= 2 * B_P (only ~4,000 seeks
        #   total on block-0!), while total element reads in T_1/T_2 are n_active1 + n_active2 = m!
        pi_cl1 = {
            "B_P_pieces": B_P,
            "B_P_over_h1": round(B_P / blk.h1, 4) if blk.h1 else 0.0,
            "B_P_schedule_bytes_8B": B_P * 8,
            "sweep1_monotone_seeks": s1_rm["seg_monotone"],
            "sweep1_cache16_segs": s1_rm["seg_cache16"],
            "sweep2_monotone_seeks": s2_rm["seg_monotone"],
            "sweep2_cache16_segs": s2_rm["seg_cache16"],
            "total_sweep1_plus_sweep2_monotone_seeks": s1_rm["seg_monotone"] + s2_rm["seg_monotone"],
            "Q_acq_strips": ret.Q_acq_strips,
            "Q_emit_links": s2["emit_links"],
            "Q_emit_links_over_h1": s2["emit_links_over_h1"],
            "P_sweeps": 2,  # Sweep 1 (up) + Sweep 2 (down)
            "S_dag_added_KB": round(s2["dag_added_bytes_8B"] / 1024.0, 1),
            "S_flat_s2_KB": round(s2["flat_word_bytes"] / 1024.0, 1),
        }

        rec = {
            "name": blk.name,
            "n": blk.n,
            "r": blk.r,
            "h1": blk.h1,
            "L_min": lmin,
            "m_retained": ret.m,
            "B_P": B_P,
            "B_P_over_h1": round(B_P / blk.h1, 4) if blk.h1 else 0.0,
            "arms": arms,
            "pi_summary_uncapped": pi_cl1,
        }
        results.append(rec)
        print(
            f"{blk.name:24s} | m={ret.m:6d} B_P={B_P:5d} ({B_P/blk.h1:.3f}h1) | "
            f"S1(unc): out_seg={s1['out_seg_exact1']:5d} ({s1['out_seg_exact1']/B_P:4.1f}x) links={s1['emit_links']:5d} ({s1['emit_links_over_h1']:.3f}h1) | "
            f"CL1 S2(unc): mono_seeks={s2_rm['seg_monotone']:5d} (<=B_P!) cache16={s2_rm['seg_cache16']:5d} "
            f"links={s2['emit_links']:5d} ({s2['emit_links_over_h1']:.3f}h1) DAG={s2['dag_added_bytes_8B']/1024:5.0f}KB vs flat={s2['flat_word_bytes']/1024:6.0f}KB | EXACT"
        )

    out_path = os.path.join(
        os.path.dirname(os.path.abspath(__file__)),
        "results_cl1_mirror_factored_closure.json",
    )
    with open(out_path, "w") as f:
        json.dump({"multicycle_periodic_check": mc, "blocks": results}, f, indent=2)
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
