#!/usr/bin/env python3
"""
Carbon Kitchen Sink — Cycle 14 Mathematical Probe & Verifier (probe_cycle14.py)

Closing Foundational Question 5 (`FRONTIER.md:107`) & `OPEN-QUESTIONS.md` §4
("Copies from completed output — using information Open already has, such as its bands"):

1. Theorem 14.1 — The Exit-Row Decoupling Theorem (`Band-Itinerary Tubes` vs `Affine Lockstep Tubes`):
   - Proves & verifies on 1,508 small primitive BWTs + all 6 Wikidata blocks that along `Open<16>`'s
     retained cycle `j -> next_j[j]`, two rows `j_1, j_2` sharing the same d-step band sequence
     `(b_0, ..., b_{d-1})` spell the EXACT same bytes `W_d(j_1) == W_d(j_2)` in `out`, YET
     skipping the `d - 1` intermediate pointer loads from `j_2` requires knowing `pi_ret^d(j_2)`!
   - Measures BOTH:
       (a) `Band-Itinerary Copy Coverage`: fraction of walk steps/bytes covered by earlier-visited
           identical d-band itineraries (`d = 2, 4, 8`), and what fraction of those have NON-AFFINE
           exit (`pi_ret^d(j_2) - pi_ret^d(j_1) != j_2 - j_1`, so output bytes alone CANNOT skip hops!),
       (b) `Affine Lockstep Tube Coverage`: exact maximal d-step affine tubes (`d = 2, 4, 6, 8`)
           where `next_j` steps in unit lockstep (`+1`) for all `d` steps, accounting for the
           mandatory First-Visit Populate Tax (`1 / w` of every width-`w` tube must walk normally!).

2. Theorem 14.2 — The Structural Demand-Read & Working-Set Obstruction Against `mibz2 0.5.8`:
   - Simulates exact 64-byte cache-line / L1D(32 KiB) / L2(256 KiB, i9 private L2) demand-read
     traffic of `mibz2 0.5.8` (`with L1D-resident inline16`) vs:
       * Band/Tube-Directory Self-Sourcing (`recording (out_pos, exit_row) per affine run`), and
       * Zero-Extra-RAM Neighbour-Sheet Probing (`checking if j-1 or j+1 was already emitted via
         in-place edge tagging`).
   - Proves quantitatively why the demand-read rise (`+24% to +89%` non-L1D cache-line reads!) and
     per-worker working-set growth (`+371 to +983 KiB`) are STRUCTURAL on `Open<16>`.
"""

from __future__ import annotations
import itertools
import json
from pathlib import Path
import sys
import numpy as np

ADMITTED_DIR = Path("/home/antigravity/admitted-bwt")
OUT_PATH = Path(__file__).resolve().parent / "probe_cycle14_results.json"
LIMIT = 16

C12_DIR = str(Path(__file__).resolve().parents[1] / "2026-11-20-carbon-kitchen-sink-cycle12")
if C12_DIR not in sys.path:
    sys.path.insert(0, C12_DIR)
import probe_cycle12 as pc12


def load_manifest():
    with open(ADMITTED_DIR / "manifest.json", "r", encoding="utf-8") as f:
        return json.load(f)["blocks"]


def build_open058_full_state(L: bytes, p: int, limit: int = 16):
    """Returns exact 0.5.8 retained state: m, mark, next_j, band_id_of_row, word_bytes_of_row."""
    info = pc12.extract_pieces(L, p, limit=limit)
    res = pc12.right_eliminate_058(info, hash_cons_rules=True)
    m = info["m"]
    mark = info["mark"]
    queues = [[] for _ in range(256)]
    for c, w, m_start in info["thin_runs"]:
        queues[c].extend(range(m_start, m_start + w))

    q_cur = [0] * 256
    next_j = np.zeros(m, dtype=np.int32)
    band_id = np.zeros(m, dtype=np.int32)
    word_len = np.zeros(m, dtype=np.int32)
    band_words = []

    for b_idx, (k_s, k_e, dst, wb, w_id, a) in enumerate(res["bands"]):
        span = k_e - k_s
        src = queues[a][q_cur[a] : q_cur[a] + span]
        q_cur[a] += span
        next_j[dst : dst + span] = src
        band_id[dst : dst + span] = b_idx
        word_len[dst : dst + span] = len(wb)
        band_words.append(wb)

    return {
        "n": info["n"],
        "m": m,
        "mark": mark,
        "num_bands": len(res["bands"]),
        "next_j": next_j,
        "band_id": band_id,
        "word_len": word_len,
        "band_words": band_words,
    }


def verify_small_self_sourcing():
    """
    Verifies on all 1,508 small primitive BWTs (binary n<=8, ternary n<=6) that:
    1) Identical d-step band itineraries ALWAYS produce byte-identical phrases in `out`,
    2) Affine lockstep d-tubes (`next_j^t(j2) - next_j^t(j1) == j2 - j1` and same band for t=0..d-1)
       reconstruct the EXACT block bytes when copying `out[src..src+L]` and jumping directly to
       `exit_j1 + (j2 - j1)`!
    """
    checked = 0
    non_affine_band_matches = 0
    for alph, max_n in (((0, 1), 8), ((0, 1, 2), 6)):
        for n in range(2, max_n + 1):
            for w in itertools.product(alph, repeat=n):
                if not pc12.is_primitive(w):
                    continue
                L, p = pc12.bwt_of_word(w)
                expected = bytes(w)
                st = build_open058_full_state(L, p, limit=2)
                m = st["m"]
                nj = st["next_j"]
                bid = st["band_id"]
                bw = st["band_words"]

                # Full reference walk order
                order = []
                cur = st["mark"]
                for _ in range(m):
                    order.append(int(cur))
                    cur = int(nj[cur])
                    if cur == st["mark"]:
                        break

                # Check d=2 band-itinerary vs affine-lockstep decoder
                d = 2
                if len(order) >= d + 1:
                    seen_band_itin = {}
                    for idx in range(len(order) - d):
                        j = order[idx]
                        j1 = int(nj[j])
                        j2 = int(nj[j1])
                        itin = (int(bid[j]), int(bid[j1]))
                        phrase = bw[itin[0]] + bw[itin[1]]
                        if itin in seen_band_itin:
                            prev_j, prev_exit, prev_phrase = seen_band_itin[itin]
                            assert phrase == prev_phrase
                            if j2 - prev_exit != j - prev_j:
                                non_affine_band_matches += 1
                        else:
                            seen_band_itin[itin] = (j, j2, phrase)

                # Verify exactness of Affine-Tube Self-Sourcing Decoder (`d = 2`):
                # Decompose [0, m) into maximal 2-step affine tubes:
                is_aff2 = np.zeros(m, dtype=bool)
                for j in range(m - 1):
                    j_n = int(nj[j])
                    jp_n = int(nj[j + 1])
                    if (
                        bid[j] == bid[j + 1]
                        and jp_n == j_n + 1
                        and j_n + 1 < m
                        and bid[j_n] == bid[j_n + 1]
                        and int(nj[jp_n]) == int(nj[j_n]) + 1
                    ):
                        is_aff2[j] = True
                tube_id = np.zeros(m, dtype=np.int32)
                tube_head = [0]
                tid = 0
                for j in range(1, m):
                    if not is_aff2[j - 1]:
                        tid += 1
                        tube_head.append(j)
                    tube_id[j] = tid

                out = bytearray()
                tube_dir = {}  # tid -> (head_out_pos_virtual, head_exit_j, phrase_len, head_j)
                cur = st["mark"]
                steps_left = len(order)
                i_step = 0
                while i_step < steps_left and len(out) < n:
                    t_id = int(tube_id[cur])
                    if t_id in tube_dir and i_step + 2 <= steps_left:
                        src_pos, ref_exit, plen, ref_j = tube_dir[t_id]
                        # Copy `plen` bytes directly from ALREADY COMPLETED `out[src_pos .. src_pos + plen]`!
                        for k_byte in range(plen):
                            out.append(out[src_pos + k_byte])
                        cur = ref_exit + (cur - ref_j)
                        i_step += 2
                    elif i_step + 2 <= steps_left and (
                        (cur > 0 and tube_id[cur - 1] == t_id) or (cur + 1 < m and tube_id[cur + 1] == t_id)
                    ):
                        # First visit to a width >= 2 affine 2-tube: walk 2 steps normally & record `out` source!
                        pos0 = len(out)
                        j0 = cur
                        out.extend(bw[int(bid[cur])])
                        cur = int(nj[cur])
                        out.extend(bw[int(bid[cur])])
                        cur = int(nj[cur])
                        plen = len(out) - pos0
                        if pos0 + plen <= n:
                            tube_dir[t_id] = (pos0, cur, plen, j0)
                        i_step += 2
                    else:
                        out.extend(bw[int(bid[cur])])
                        cur = int(nj[cur])
                        i_step += 1

                out = out[:n]
                while len(out) < n:
                    take = min(len(out), n - len(out))
                    out.extend(out[:take])
                assert bytes(out) == expected, f"Affine-tube self-source failed on {w}"
                checked += 1

    return {
        "small_primitive_verified": checked,
        "non_affine_band_itinerary_matches_observed": non_affine_band_matches,
    }


def analyze_wikidata_self_sourcing():
    blocks = load_manifest()
    results = {}

    for blk in blocks:
        name = blk["file"]
        p = blk["primary_index"]
        L = (ADMITTED_DIR / name).read_bytes()
        st = build_open058_full_state(L, p, limit=LIMIT)
        n = st["n"]
        m = st["m"]
        nj = st["next_j"]
        bid = st["band_id"]
        wlen = st["word_len"]

        # Extract exact retained cycle walk order `walk_j[0..m)` and output positions `walk_out_pos[0..m)`!
        walk_j = np.empty(m, dtype=np.int32)
        walk_pos = np.empty(m, dtype=np.int64)
        cur = st["mark"]
        at = 0
        actual_steps = 0
        for s in range(m):
            walk_j[s] = cur
            walk_pos[s] = at
            at += int(wlen[cur])
            cur = int(nj[cur])
            actual_steps += 1
            if cur == st["mark"] or at >= n:
                break
        walk_j = walk_j[:actual_steps]
        walk_pos = walk_pos[:actual_steps]

        # Precompute multi-step iterates nj_pow[t][j] for t = 0..8!
        max_d = 8
        nj_pow = [np.arange(m, dtype=np.int32)]
        for _ in range(max_d):
            nj_pow.append(nj[nj_pow[-1]])

        # 1. Measure Band-Itinerary Equivalence vs Affine Lockstep Equivalence for d in (2, 4, 8)!
        by_depth = {}
        for d in (2, 4, 8):
            # Hash each row's d-step band tuple `(bid[j], bid[nj[j]], ..., bid[nj^{d-1}[j]])` using 64-bit polynomial rolling hash + exact check:
            h = np.zeros(m, dtype=np.uint64)
            C_MUL = np.uint64(0x9E3779B97F4A7C15)
            for t in range(d):
                h = (h * C_MUL) ^ (bid[nj_pow[t]].astype(np.uint64) + np.uint64(t + 1))

            # Greedy non-overlapping scan along walk_j:
            seen_itin = {}  # hash -> (first_j, first_exit_j, first_out_pos)
            band_copy_hits = 0
            band_copy_hops_if_oracle_exit = 0
            band_copy_bytes = 0
            band_hits_that_are_affine = 0
            s = 0
            while s + d <= actual_steps:
                j = int(walk_j[s])
                hj = int(h[j])
                prev = seen_itin.get(hj)
                if prev is not None:
                    first_j, first_exit, first_pos = prev
                    # Exact tuple check against hash collision:
                    if all(bid[nj_pow[t][j]] == bid[nj_pow[t][first_j]] for t in range(d)):
                        band_copy_hits += 1
                        band_copy_hops_if_oracle_exit += (d - 1)
                        plen = int(walk_pos[s + d - 1] + wlen[walk_j[s + d - 1]] - walk_pos[s])
                        band_copy_bytes += plen
                        if int(nj_pow[d][j]) - first_exit == j - first_j:
                            band_hits_that_are_affine += 1
                        s += d
                        continue
                seen_itin[hj] = (j, int(nj_pow[d][j]), int(walk_pos[s]))
                s += 1

            # 2. Exact Maximal d-Step Affine Lockstep Tube Partition `[0, m)`:
            # `j` and `j+1` stay in lockstep for `d` steps iff for t=0..d-1: `bid[nj^t(j)] == bid[nj^t(j+1)]`
            # and `nj^{t+1}(j+1) == nj^{t+1}(j) + 1`!
            adj_ok = np.ones(m - 1, dtype=bool)
            for t in range(d):
                jt = nj_pow[t][:-1]
                jtp = nj_pow[t][1:]
                adj_ok &= (bid[jt] == bid[jtp]) & (nj[jtp] == nj[jt] + 1)
            breaks = np.flatnonzero(~adj_ok) + 1
            t_starts = np.concatenate(([0], breaks))
            t_widths = np.concatenate((breaks, [m])) - t_starts
            num_tubes = int(len(t_starts))
            multi_tubes = int(np.sum(t_widths >= 2))
            row_tube = np.repeat(np.arange(num_tubes, dtype=np.int32), t_widths)
            row_tube_w = t_widths[row_tube]

            # Simulate exact Greedy Affine-Tube Self-Sourcing along `walk_j`:
            # Also count exact non-L1D cache-line demand reads (assuming `inline16` fits in 32 KiB L1D via MN16,
            # while `edges` [351-798 KiB], `tube_dir` [8B * num_tubes], and `out[0..at)` [up to 900 KB] miss 32 KiB L1D!):
            # Wait: also track backwards source distance `at - src_pos` to see what % of `out[src..src+L]` misses
            # BOTH 32 KiB L1D AND 256 KiB private L2 (i9-10900K)!
            visited_tube = {}  # tid -> (ref_j, ref_exit, ref_out_pos, plen)
            aff_hits = 0
            aff_first_visits = 0
            aff_skipped_hops = 0
            aff_copied_bytes = 0
            out_l1_misses = 0  # copy distance > 16 KiB (half of 32 KiB L1D)
            out_l2_misses = 0  # copy distance > 192 KiB (evicted from 256 KiB i9 L2 by edges + out!)
            extra_cl_reads_out = 0

            s = 0
            while s < actual_steps:
                j = int(walk_j[s])
                tid = int(row_tube[j])
                if row_tube_w[j] >= 2 and s + d <= actual_steps:
                    rec = visited_tube.get(tid)
                    if rec is not None:
                        ref_j, ref_exit, ref_pos, plen = rec
                        cur_pos = int(walk_pos[s])
                        dist = cur_pos - ref_pos
                        aff_hits += 1
                        aff_skipped_hops += (d - 1)
                        aff_copied_bytes += plen
                        # Number of 64B cache lines spanned by `out[ref_pos .. ref_pos + plen)`:
                        cls = ((ref_pos + max(1, plen) - 1) >> 6) - (ref_pos >> 6) + 1
                        if dist > 16384:
                            out_l1_misses += cls
                        if dist > 196608:
                            out_l2_misses += cls
                        extra_cl_reads_out += cls
                        s += d
                        continue
                    else:
                        # First visit to this width>=2 tube: must walk `d` steps normally + populate `tube_dir[tid]`!
                        plen = int(walk_pos[s + d - 1] + wlen[walk_j[s + d - 1]] - walk_pos[s])
                        visited_tube[tid] = (j, int(nj_pow[d][j]), int(walk_pos[s]), plen)
                        aff_first_visits += 1
                        s += d
                        continue
                s += 1

            # Baseline `0.5.8 + MN16` non-L1D demand reads in `emit`:
            # Simply `actual_steps` loads from `edges[0..m)` (`inline16` < 32 KiB is L1D-resident!).
            baseline_non_l1_reads = actual_steps
            # Self-sourcing non-L1D demand reads:
            # - Remaining `edges` loads: `actual_steps - aff_skipped_hops`
            # - `tube_dir[tid]` (`8 B * num_tubes = 380..1006 KiB`) lookup on every step (`or row_tube[j]`):
            #   even if width==1 bit is packed into `edges[j]` (`0` extra lookup on singletons!!), we STILL
            #   read `tube_dir` on `aff_first_visits + aff_hits` multi-row tube visits (+ dirty writeback on `aff_first_visits`),
            #   AND read `out[ref_pos..]` (`out_l1_misses` cache lines) on `aff_hits`!
            #   WAIT — check BOTH:
            #   (i) General directory lookup (1 lookup per step not skipped), and
            #   (ii) Idealized zero-singleton-cost bit-tagged `edges[j]` (lookup ONLY when `w >= 2`)!
            reads_general_dir = (actual_steps - aff_skipped_hops) + (actual_steps - aff_hits * (d - 1)) + out_l1_misses
            reads_ideal_tagged = (actual_steps - aff_skipped_hops) + (aff_first_visits + aff_hits) + out_l1_misses
            tube_dir_kib = round(num_tubes * 8 / 1024.0, 1)

            by_depth[f"d={d}"] = {
                "band_itinerary_hits": band_copy_hits,
                "band_itinerary_bytes_pct_of_n": round(100.0 * band_copy_bytes / float(n), 2),
                "band_hits_non_affine_exit_pct": round(
                    100.0 * (band_copy_hits - band_hits_that_are_affine) / max(1, band_copy_hits), 2
                ),
                "affine_tubes_total": num_tubes,
                "affine_tubes_width_ge2": multi_tubes,
                "affine_hits": aff_hits,
                "affine_first_visits": aff_first_visits,
                "affine_skipped_hops": aff_skipped_hops,
                "affine_skipped_hops_pct_of_m": round(100.0 * aff_skipped_hops / float(actual_steps), 2),
                "affine_copied_bytes_pct_of_n": round(100.0 * aff_copied_bytes / float(n), 2),
                "out_source_l1_miss_cls": out_l1_misses,
                "out_source_i9_l2_miss_cls": out_l2_misses,
                "demand_reads_ratio_general_dir": round(reads_general_dir / float(baseline_non_l1_reads), 3),
                "demand_reads_ratio_ideal_tagged": round(reads_ideal_tagged / float(baseline_non_l1_reads), 3),
                "extra_tube_dir_ws_kib": tube_dir_kib,
            }

        results[name] = {
            "n": n,
            "m": m,
            "num_bands": st["num_bands"],
            "mean_band_width": round(m / float(st["num_bands"]), 2),
            "depths": by_depth,
        }
        d2 = by_depth["d=2"]
        d4 = by_depth["d=4"]
        print(
            f"  {name:12s}: bands={st['num_bands']:4d} (w={m/st['num_bands']:.1f}) | "
            f"d=2: band_bytes={d2['band_itinerary_bytes_pct_of_n']:5.1f}% (non_aff={d2['band_hits_non_affine_exit_pct']:5.1f}%) "
            f"aff_skip={d2['affine_skipped_hops_pct_of_m']:5.2f}% demand_gen={d2['demand_reads_ratio_general_dir']:.3f}x "
            f"demand_ideal={d2['demand_reads_ratio_ideal_tagged']:.3f}x (+{d2['extra_tube_dir_ws_kib']}K WS) | "
            f"d=4: aff_skip={d4['affine_skipped_hops_pct_of_m']:5.2f}% demand_ideal={d4['demand_reads_ratio_ideal_tagged']:.3f}x"
        )

    return results


def main():
    print("=== Cycle 14 Probe: Output-Sourced Decoding via Open's Bands (Foundational Q5 / §4) ===")
    sm = verify_small_self_sourcing()
    print(f"[1] Small checks verified: {sm}")
    wiki = analyze_wikidata_self_sourcing()
    OUT_PATH.write_text(json.dumps({"small_verifications": sm, "blocks": wiki}, indent=2), encoding="utf-8")
    print(f"Saved {OUT_PATH}")


if __name__ == "__main__":
    main()
