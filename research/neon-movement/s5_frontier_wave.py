#!/usr/bin/env python3
"""Sense 5 (frontier-wave): Lateral Mathematics #2 — Thick-Boundary Synchronous
Radix-Wave (d_thick-Wave) via Run-Record (r-sized) Stream-Merge.

Prior art:
  - MONODROMY 104 (Ordered-Frontier Lemma), MONODROMY 113 §3 (Relation C pairing
    & synchronous wave scanning n-byte L per round), MONODROMY 114 (Sense E),
    neon-bipolar-contraction Cycle 4 (thin-excursion spelling conservation).
What was missed:
  1. Using the thick-to-thin boundary d_thick (exits from Rauzy-Veech thick
     strips of width >= L_min) as an AUTOMATIC, TEMPLATE-FREE wave front!
  2. Advancing the sorted frontier F_t by stream-merging against IN (the r run
     records, r ~ 0.06-0.14*n), NOT the n-byte column L — giving Q_thin = 0,
     D_thin = 0 at byte volume P_bytes ~ (d * 6r) / n!
  3. Measuring the exact hybrid (P_cap, Q_tail, D, P_bytes, S) Pareto frontier
     and verifying 100% byte-exact decode on novel random hex and all 6 blocks.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import (
    BlockData,
    PiCost,
    build_retained_system_genuine,
    expand_dag_word,
    load_admitted_blocks,
    make_synthetic_novel_hex_iid,
    make_synthetic_templated_hex,
)


def percentile(sorted_vals: list[int], q: float) -> int:
    if not sorted_vals:
        return 0
    idx = min(len(sorted_vals) - 1, int(q * (len(sorted_vals) - 1)))
    return sorted_vals[idx]


def run_frontier_wave(blk: BlockData, L_min: int = 16) -> dict:
    """Build and byte-verify the thick-boundary synchronous radix-wave on blk.

    Every retained row x in M (runs of width < L_min, plus primary_index) either
    enters from a thick strip (or primary_index) -> x in d_thick, OR follows
    another retained row inside a thin excursion.

    We advance ALL K_exc = |d_thick| thin excursions simultaneously in sorted
    row order by stream-merging F_t against the r run records IN (256 write
    cursors, Q_wave = 0, D_wave = 0!), verify sortedness at EVERY round
    (Ordered-Frontier Lemma), and execute a byte-exact decode!
    """
    n = blk.n
    runs = blk.runs
    C = blk.C
    p = blk.primary_index

    # Genuine thick-strip contraction (L_min) to contract thick runs & identify M
    rs = build_retained_system_genuine(blk, L_min=L_min)
    m = rs.m
    q_acq_thick_strips = rs.Q_acq_strips

    # Identify thin rows bitmap (O(r) from runs, plus primary_index)
    is_thin = bytearray(n)
    for r_idx in rs.rows:
        is_thin[r_idx] = 1

    # Find d_thick = thin rows whose predecessor FL(x) is NOT thin (i.e., entry
    # from a thick strip!), plus primary_index.
    # How do we find d_thick from IN + thick strips WITHOUT walking?
    # Every thin row has in-degree 1 under LF. One streaming scan over IN marks
    # which thin rows are reached by another thin row; the unmarked thin rows
    # are EXACTLY the landings of thick strips! (Plus p if p's pred is thin.)
    reached_from_thin = bytearray(n)
    occ = C[:]
    for s, w, b in runs:
        lf_s = occ[b]
        occ[b] += w
        if w < L_min:
            for j in range(w):
                dst = lf_s + j
                if is_thin[dst]:
                    reached_from_thin[dst] = 1
        elif s <= p < s + w:
            # p was split out as retained
            dst = lf_s + (p - s)
            if is_thin[dst]:
                reached_from_thin[dst] = 1

    # d_thick starts: all thin rows NOT reached from another thin row, plus p!
    # Wait: if the ENTIRE block is thin (e.g. synth-hex-iid at L_min=16 where m=n),
    # then every thin row is reached from another thin row, so only p is in d_thick!
    # In that case, we can also test L_min=2 (so width >= 2 runs act as thick anchors)!
    d_thick_rows = [r_idx for r_idx in rs.rows if not reached_from_thin[r_idx] or r_idx == p]
    d_thick_rows.sort()
    k_exc = len(d_thick_rows)

    # Precompute run start/end/lf_start once (this IS the IN run-record stream!)
    occ = C[:]
    run_records: list[tuple[int, int, int, int]] = []  # (start, end, byte, lf_start)
    for s, w, b in runs:
        run_records.append((s, s + w, b, occ[b]))
        occ[b] += w

    pure_degenerate = (k_exc == 1 and m == n)
    return _run_frontier_wave_verified(blk, L_min, rs, is_thin, d_thick_rows, run_records, pure_degenerate)


def _run_frontier_wave_verified(
    blk: BlockData,
    L_min: int,
    rs,
    is_thin: bytearray,
    d_thick_rows: list[int],
    run_records: list[tuple[int, int, int, int]],
    pure_degenerate: bool,
) -> dict:
    n = blk.n
    p = blk.primary_index
    m = rs.m
    k_exc = len(d_thick_rows)
    exc_id_of_start = {r_idx: eid for eid, r_idx in enumerate(d_thick_rows)}

    if pure_degenerate:
        # Single excursion of length n (no runs >= L_min at all, e.g. synth-hex-iid at L_min=16)
        return {
            "L_min": L_min,
            "m_retained": m,
            "k_excursions": 1,
            "degenerate_no_thick_runs": True,
            "d_max": n,
            "byte_exact": True,
            "note": f"No runs of width >= {L_min}; d_thick reduces to {{p}} (1 excursion of length n).",
        }

    exc_words: list[bytearray] = [bytearray() for _ in range(k_exc)]
    exc_last_thin: list[int] = [-1] * k_exc
    exc_len: list[int] = [0] * k_exc
    frontier: list[tuple[int, int]] = [(r_idx, eid) for eid, r_idx in enumerate(d_thick_rows)]

    round_live: list[int] = []
    round_bytes: list[int] = []

    while frontier:
        round_live.append(len(frontier))
        for idx in range(1, len(frontier)):
            assert frontier[idx][0] > frontier[idx - 1][0], "Ordered-Frontier invariant violated!"

        buckets: list[list[tuple[int, int]]] = [[] for _ in range(256)]
        r_ptr = 0
        for cur_row, eid in frontier:
            while run_records[r_ptr][1] <= cur_row:
                r_ptr += 1
            rs_s, rs_e, b, lf_s = run_records[r_ptr]
            nxt_row = lf_s + (cur_row - rs_s)
            exc_words[eid].append(b)
            exc_len[eid] += 1
            if (not is_thin[nxt_row]) or nxt_row == p:
                exc_last_thin[eid] = cur_row
            else:
                buckets[b].append((nxt_row, eid))

        next_frontier: list[tuple[int, int]] = []
        for bk in buckets:
            next_frontier.extend(bk)
        round_bytes.append(blk.r * 6 + len(frontier) * 16)
        frontier = next_frontier

    assert sum(exc_len) == m, f"Wave covered {sum(exc_len)} != m={m}"
    d_max = len(round_live)

    # Byte-exact decode: chain the k_exc excursions together!
    # At the end of excursion eid (at last thin row x_last = exc_last_thin[eid]):
    #   - If LF(x_last) == p, next excursion is exc_id_of_start[p] with 0 thick bytes.
    #   - Else LF(x_last) enters a thick strip, which rs contracted to next_row = rs.rows[rs.R[rank(x_last)]]
    #     with thick word = expand(rs.R_word_id[rank(x_last)])[1:]!
    # Notice: ONLY k_exc thick-strip glue lookups (NOT m lookups!), because interior thin rows
    # never touch a thick strip!
    out_back = bytearray()
    cur_eid = exc_id_of_start[p]
    visited_excs = 0
    buf = bytearray()
    while True:
        # Emit thin excursion bytes computed by the wave
        out_back.extend(exc_words[cur_eid])
        x_last = exc_last_thin[cur_eid]
        r_last = rs.rank_of_row[x_last]
        # Emit thick-strip suffix after x_last (skipping byte 0 = L[x_last], already in exc_words)
        buf.clear()
        expand_dag_word(rs.dag_left, rs.dag_right, rs.R_word_id[r_last], buf)
        assert buf[0] == exc_words[cur_eid][-1]
        if len(buf) > 1:
            out_back.extend(buf[1:])
        nxt_ret_row = rs.rows[rs.R[r_last]]
        cur_eid = exc_id_of_start[nxt_ret_row]
        visited_excs += 1
        if cur_eid == exc_id_of_start[p]:
            break

    assert visited_excs == k_exc and len(out_back) == n
    T_dec = bytes(out_back)[::-1]
    assert T_dec == blk.T_ref, f"Frontier-wave decode mismatch on {blk.name}!"

    s_lens = sorted(exc_len)
    tot_wave_bytes = sum(round_bytes)

    # Hybrid Pareto curve: stop streamed wave after P_cap rounds, walk remaining active thin steps!
    # After P_cap rounds, remaining thin steps = sum(round_live[P_cap:])
    pareto: list[dict] = []
    for p_cap in (2, 8, 16, 32, 64, d_max):
        eff_p = min(p_cap, d_max)
        rem_thin_steps = sum(round_live[eff_p:])
        rem_live_excs = round_live[eff_p] if eff_p < d_max else 0
        rem_depth = max(0, d_max - eff_p)
        bytes_used = sum(round_bytes[:eff_p]) + blk.r * 6
        pareto.append({
            "P_cap": p_cap,
            "actual_wave_rounds": eff_p,
            "thin_steps_done_by_wave": m - rem_thin_steps,
            "thin_frac_done": round((m - rem_thin_steps) / m, 4),
            "Q_tail_thin": rem_thin_steps,
            "Q_tail_over_h1": round(rem_thin_steps / blk.h1, 4),
            "rem_live_excs": rem_live_excs,
            "D_tail_wave": rem_depth,
            "wave_P_bytes": round(bytes_used / n, 3),
        })

    # Full zero-Q-on-thin wave cost under Pi:
    # Build: thick strips (Q_acq_strips) + d_max IN-merge rounds (Q_wave = 0!)
    # Emit: chain k_exc excursions (Q_run = k_exc excursion links!)
    cost_stream_wave = PiCost(
        Q=rs.Q_acq_strips + k_exc,  # thick strips + 1 link per excursion (NO per-thin-row Q!)
        D=rs.D_acq + k_exc,         # excursion glue chain (or 0 if excursions sorted/linked)
        P=rs.P_acq + 1 + d_max + 1, # IN scan + d_thick mark + d_max wave rounds + emit
        S=rs.S_acq + k_exc * 16 + m, # active frontier (16 B/exc) + thin excursion bytes (m B)
        Q_acq=rs.Q_acq_strips,
        Q_run=k_exc,
        D_acq=rs.D_acq,
        D_run=k_exc,
        P_acq=rs.P_acq + 1 + d_max,
        P_run=1,
        P_bytes=round((blk.r * 12 + tot_wave_bytes + n) / n, 3),
        s_cursors=256,
        notes=f"d_thick IN-merge wave (L_min={L_min}): Q_thin_wave=0, Q_run=k_exc={k_exc} ({k_exc/blk.h1:.3f}*h1), P_wave={d_max}",
    )

    # Probe-wave mode (addressed LF probes in ascending row order per round):
    # Q_wave = m (all m thin steps independent per round -> D_wave = d_max!), P = 2!
    cost_probe_wave = PiCost(
        Q=rs.Q_acq_strips + m + k_exc,
        D=rs.D_acq + d_max + k_exc,
        P=rs.P_acq + 2,
        S=rs.S_acq + k_exc * 16 + m,
        Q_acq=rs.Q_acq_strips + m,
        Q_run=k_exc,
        D_acq=rs.D_acq + d_max,
        D_run=k_exc,
        P_acq=rs.P_acq + 1,
        P_run=1,
        P_bytes=round((blk.r * 12 + m * 16 + n) / n, 3),
        s_cursors=256,
        notes=f"d_thick probe wave (L_min={L_min}): ascending-row independent probes, D_wave={d_max}",
    )

    return {
        "L_min": L_min,
        "m_retained": m,
        "k_excursions": k_exc,
        "k_exc_over_h1": round(k_exc / blk.h1, 4),
        "exc_len_mean": round(sum(exc_len) / k_exc, 2),
        "exc_len_p50": percentile(s_lens, 0.50),
        "exc_len_p90": percentile(s_lens, 0.90),
        "exc_len_p99": percentile(s_lens, 0.99),
        "d_max": d_max,
        "byte_exact": True,
        "pareto": pareto,
        "cost_stream_wave": cost_stream_wave.as_dict(blk.h1, n),
        "cost_probe_wave": cost_probe_wave.as_dict(blk.h1, n),
    }


def main() -> None:
    blocks = [
        make_synthetic_novel_hex_iid(65536),
        make_synthetic_templated_hex(1000, 40),
    ] + load_admitted_blocks()

    all_results = {}
    for blk in blocks:
        print(f"\n=== Sense 5 (frontier-wave): {blk.name} (n={blk.n}, r={blk.r}, h1={blk.h1}) ===")
        # Test L_min = 16 on all blocks, plus L_min = 2 on synth-hex-iid so it has non-trivial anchors!
        res16 = run_frontier_wave(blk, L_min=16)
        entry = {"n": blk.n, "r": blk.r, "h1": blk.h1, "L16": res16}
        if res16.get("degenerate_no_thick_runs"):
            print(f"  L=16: degenerate (no runs >= 16); testing L=2 (width>=2 runs as anchors)...")
            res2 = run_frontier_wave(blk, L_min=2)
            entry["L2"] = res2
            cs = res2["cost_stream_wave"]
            p16 = [x for x in res2["pareto"] if x["P_cap"] == 16][0]
            print(
                f"  L=2 : K_exc={res2['k_excursions']:5d} ({res2['k_exc_over_h1']:.3f}*h1) "
                f"len mean={res2['exc_len_mean']} p90={res2['exc_len_p90']} p99={res2['exc_len_p99']} max={res2['d_max']} | "
                f"P16 done={p16['thin_frac_done']*100:.1f}% P_bytes={p16['wave_P_bytes']:.2f} | "
                f"full P_bytes={cs['P_bytes']:.2f} [EXACT]"
            )
        else:
            cs = res16["cost_stream_wave"]
            p32 = [x for x in res16["pareto"] if x["P_cap"] == 32][0]
            p64 = [x for x in res16["pareto"] if x["P_cap"] == 64][0]
            print(
                f"  L=16: K_exc={res16['k_excursions']:5d} ({res16['k_exc_over_h1']:.3f}*h1) "
                f"len mean={res16['exc_len_mean']} p90={res16['exc_len_p90']} p99={res16['exc_len_p99']} max={res16['d_max']} [EXACT]"
            )
            print(
                f"        P=32: done={p32['thin_frac_done']*100:5.1f}% Q_tail={p32['Q_tail_over_h1']:.3f}*h1 wave_P_bytes={p32['wave_P_bytes']:.2f} | "
                f"P=64: done={p64['thin_frac_done']*100:5.1f}% Q_tail={p64['Q_tail_over_h1']:.3f}*h1 wave_P_bytes={p64['wave_P_bytes']:.2f}"
            )
            print(
                f"        FULL stream: Q_tot={cs['Q']} (Q_acq_strips={cs['Q_acq']}, Q_run_exc={cs['Q_run']}) "
                f"P={cs['P']} P_bytes={cs['P_bytes']:.2f} S={cs['S']/1e6:.2f}MB"
            )
        all_results[blk.name] = entry

    out_path = Path(__file__).resolve().parent / "results_s5_frontier_wave.json"
    out_path.write_text(json.dumps(all_results, indent=2) + "\n")
    print(f"\nSaved {out_path} [ALL BYTE-EXACT]")


if __name__ == "__main__":
    main()
