#!/usr/bin/env python3
"""Sense 1 (text-axis): Contraction along the text rather than the rows.

Three exact, byte-verified constructions:
  1A. Recursive Landmark Induction on raw LF (MONODROMY 114 Sense E + NEEDS M-21):
      Y_1 = F-interval I_{c1} (most frequent byte), recursing to k=1,2,3
      (also measuring most frequent pair (c1, c2)).
  1B. Orthogonal Two-Axis Contraction (Row-Axis thick strips FIRST -> Text-Axis
      landmark induction on retained M SECOND), with genuine thick-strip
      acquisition fully charged in (Q, D, P, S).
  1C. Forward-Psi F-interval k-mer sharing (contracting repeated text phrases
      without Neon Witness 20.1's src < dst acyclicity oracle).
"""

from __future__ import annotations

import json
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import (
    BlockData,
    PiCost,
    build_lf_array,
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


def run_landmark_raw_lf(blk: BlockData, max_k: int = 3) -> dict:
    """Sense 1A: Recursive landmark induction on raw LF (NEEDS M-21).

    Level 1: Y_1 = [C[c1], C[c1] + counts[c1]) where c1 = argmax counts[c].
    Wait: in LF backward walk, a row x in I_{c1} has F[x] = c1 (its suffix starts
    with c1). Stepping backward by LF from x until we hit another row in I_{c1}
    (equivalently, until L[cur] == c1, so LF(cur) in I_{c1}!) gives the exact
    first-return map R_1 : Y_1 -> Y_1 and return words W_1!

    Level k -> k+1: let Y_{k+1} = {y in Y_k : second emitted byte (or first
    return-word byte L[y]) == c_{k+1}} where c_{k+1} is the most frequent byte
    among Y_k! Compose return words along R_k until landing in Y_{k+1}.
    Note: primary_index p might not be in Y_k, so we also track the single
    prefix excursion from p to its first hit in Y_k (or include p as 1 extra
    anchor). Including p costs +1 anchor and guarantees 100% exact decode.
    """
    n = blk.n
    L = blk.L
    C = blk.C
    counts = blk.counts
    lf = build_lf_array(L, C)

    # Most frequent byte c1
    c1 = max(range(256), key=lambda b: counts[b])
    y_lo = C[c1]
    y_hi = C[c1] + counts[c1]

    # Also find the most frequent adjacent byte pair (b_prev, b_curr) in text
    # using L[x] and F[x] directly from run records (O(r), zero walk!).
    pair_counts: Counter[tuple[int, int]] = Counter()
    # F[x] is constant on each F-bucket [C[f], C[f]+counts[f])
    f_idx = 0
    while f_idx < 256 and counts[f_idx] == 0:
        f_idx += 1
    f_end = C[f_idx] + counts[f_idx] if f_idx < 256 else 0
    for s, w, b in blk.runs:
        cur = s
        rem = w
        while rem > 0:
            while cur >= f_end:
                f_idx += 1
                while f_idx < 256 and counts[f_idx] == 0:
                    f_idx += 1
                f_end = C[f_idx] + counts[f_idx]
            take = min(rem, f_end - cur)
            pair_counts[(b, f_idx)] += take
            cur += take
            rem -= take
    best_pair, best_pair_cnt = pair_counts.most_common(1)[0]

    # Level 1 build via synchronous rounds (ordered frontier!)
    # Anchors: all rows in Y_1 = [y_lo, y_hi), plus primary_index if outside Y_1.
    p = blk.primary_index
    y1_rows = list(range(y_lo, y_hi))
    p_in_y1 = y_lo <= p < y_hi

    # First-return map R_1 on Y_1 (indexed 0..len(Y_1)-1)
    m1 = len(y1_rows)
    R_cur = [0] * m1
    W_cur: list[bytes] = [b""] * m1
    gaps_1: list[int] = [0] * m1

    # Synchronous round-based build (measures exact D = max_gap and Q_build = n)
    # To keep Python fast while exact, compute each lane's return word; the round
    # depth is strictly max(gaps_1) because all lanes advance 1 LF step per round.
    for i, start in enumerate(y1_rows):
        buf = bytearray()
        cur = start
        steps = 0
        while True:
            b = L[cur]
            buf.append(b)
            cur = lf[cur]
            steps += 1
            if y_lo <= cur < y_hi:
                break
        R_cur[i] = cur - y_lo
        W_cur[i] = bytes(buf)
        gaps_1[i] = steps

    assert sum(gaps_1) == n, f"Level 1 return words must sum to n={n}, got {sum(gaps_1)}"

    # Handle p -> first hit in Y_1 (prefix tail) if p not in Y_1
    p_prefix = bytearray()
    p_entry_y1 = p - y_lo
    if not p_in_y1:
        cur = p
        while not (y_lo <= cur < y_hi):
            p_prefix.append(L[cur])
            cur = lf[cur]
        p_entry_y1 = cur - y_lo

    # Verify byte-exact decode at level k
    def verify_level_decode(R_k: list[int], W_k: list[bytes], start_idx: int, prefix_bytes: bytes) -> bytes:
        # Walk induced map R_k from start_idx collecting all n bytes (cyclic shift by len(prefix_bytes))
        # In LF backward order, starting from p emits T[n-1], T[n-2], ..., T[0].
        # Since p reaches start_idx after len(prefix_bytes) steps, the full backward stream
        # starting at start_idx is the cyclic rotation of backward(T) by len(prefix_bytes)!
        full_back = bytearray()
        idx = start_idx
        steps = 0
        while True:
            full_back.extend(W_k[idx])
            idx = R_k[idx]
            steps += 1
            if idx == start_idx:
                break
        assert len(full_back) == n and steps == len(R_k)
        shift = len(prefix_bytes)
        if shift > 0:
            # Verify prefix_bytes matches the tail of full_back!
            assert full_back[n - shift :] == prefix_bytes
            ordered_back = bytes(full_back[n - shift :]) + bytes(full_back[: n - shift])
        else:
            ordered_back = bytes(full_back)
        T_dec = ordered_back[::-1]
        assert T_dec == blk.T_ref, f"Landmark decode mismatch on {blk.name}!"
        return T_dec

    levels_out: list[dict] = []

    # Record Level 1
    verify_level_decode(R_cur, W_cur, p_entry_y1, bytes(p_prefix))
    s_gaps = sorted(gaps_1)
    q_build_cum = n  # each of the n rows stepped once in Level 1
    d_build_cum = s_gaps[-1]  # max gap = synchronous rounds in Level 1
    levels_out.append({
        "k": 1,
        "landmark_desc": f"byte 0x{c1:02x} ({chr(c1) if 32 <= c1 < 127 else '.'})",
        "Y_size": len(R_cur),
        "Y_over_h1": round(len(R_cur) / blk.h1, 4),
        "gap_mean": round(sum(gaps_1) / len(gaps_1), 2),
        "gap_p99": percentile(s_gaps, 0.99),
        "gap_max": s_gaps[-1],
        "probe_mode": PiCost(
            Q=q_build_cum + len(R_cur),
            D=d_build_cum + len(R_cur),
            P=2,  # 1 build phase (rounds via addressed LF), 1 emit walk
            S=n + len(R_cur) * 8,  # return words (n bytes total) + R_1 table
            Q_acq=q_build_cum,
            Q_run=len(R_cur),
            D_acq=d_build_cum,
            D_run=len(R_cur),
            P_acq=1,
            P_run=1,
            P_bytes=round((n + len(R_cur) * 8) / n, 3),
            s_cursors=256,
            notes="NEEDS M-21 probe mode: Q>=n, D=max_gap+|Y_1|",
        ).as_dict(blk.h1, n),
        "stream_mode": PiCost(
            Q=len(R_cur),
            D=len(R_cur),
            P=s_gaps[-1] + 1,
            S=n + len(R_cur) * 8,
            Q_acq=0,
            Q_run=len(R_cur),
            D_acq=0,
            D_run=len(R_cur),
            P_acq=s_gaps[-1],
            P_run=1,
            P_bytes=round((s_gaps[-1] * blk.r * 6 + n) / n, 3),
            s_cursors=256,
            notes="Streamed wave against IN: Q_acq=0, P=max_gap+1",
        ).as_dict(blk.h1, n),
        "byte_exact": True,
    })

    # Recurse k = 2, 3: pick most frequent first byte of W_cur among current Y_k
    cur_start = p_entry_y1
    cur_prefix = bytearray(p_prefix)
    for k in range(2, max_k + 1):
        # Choose landmark byte on the induced map: most frequent byte at position 0 of W_cur
        # (equivalently, L[y] for y in Y_{k-1} at k=2, i.e., exact pair (c2, c1)!)
        # For k >= 3, use the first byte of the return word that isn't already fixed, or
        # simply the most frequent return-word prefix byte at index k-2 (or index 0 of next word).
        # Specifically: key(i) = W_cur[i][0] at k=2 (gives exact pair (c2, c1)!);
        # at k=3, key(i) = W_cur[i][1] if len > 1 else W_cur[i][0], OR hash/first byte of R_cur's word!
        # Using W_cur[R_cur[i]][0] selects consecutive return-word pairs (still O(1) from Y_{k-1}!).
        if k == 2:
            keys = [w[0] for w in W_cur]
        else:
            # Select by the next symbol after the previous landmark prefix:
            # W_cur[i][0] is already c2 for all i in Y_2; use W_cur[i][1] if len>=2 else -1
            # or (better, always defined and free from Y_2's own words) the LAST byte before
            # hitting Y_2, or W_cur[i][1] (second byte of return word).
            keys = [w[1] if len(w) > 1 else -1 for w in W_cur]
        cnts = Counter(keys)
        # Prefer a non-trivial subset (size >= 2 and < len(R_cur))
        chosen_key = None
        for cand_k, cand_c in cnts.most_common():
            if 2 <= cand_c < len(R_cur):
                chosen_key = cand_k
                break
        if chosen_key is None:
            break

        sub_indices = [i for i, kv in enumerate(keys) if kv == chosen_key]
        sub_rank = [-1] * len(R_cur)
        for new_i, old_i in enumerate(sub_indices):
            sub_rank[old_i] = new_i

        # Build induced map R_next on sub_indices
        m_next = len(sub_indices)
        R_next = [0] * m_next
        W_next: list[bytes] = [b""] * m_next
        step_gaps: list[int] = [0] * m_next  # steps in Y_{k-1}
        byte_gaps: list[int] = [0] * m_next  # bytes in T

        for new_i, old_start in enumerate(sub_indices):
            buf = bytearray()
            cur = old_start
            st = 0
            while True:
                buf.extend(W_cur[cur])
                cur = R_cur[cur]
                st += 1
                if sub_rank[cur] != -1:
                    break
            R_next[new_i] = sub_rank[cur]
            W_next[new_i] = bytes(buf)
            step_gaps[new_i] = st
            byte_gaps[new_i] = len(buf)

        assert sum(byte_gaps) == n

        # Advance cur_start to first hit in sub_indices
        if sub_rank[cur_start] == -1:
            # Find where cur_start lands in sub_indices
            extra = bytearray()
            c_idx = cur_start
            while sub_rank[c_idx] == -1:
                extra.extend(W_cur[c_idx])
                c_idx = R_cur[c_idx]
            cur_prefix.extend(extra)
            cur_start = sub_rank[c_idx]
        else:
            cur_start = sub_rank[cur_start]

        verify_level_decode(R_next, W_next, cur_start, bytes(cur_prefix))

        # Cost update: each node of Y_{k-1} was stepped once in level k's build!
        q_build_cum += len(R_cur)
        d_build_cum += max(step_gaps)
        s_bg = sorted(byte_gaps)
        s_sg = sorted(step_gaps)
        levels_out.append({
            "k": k,
            "landmark_desc": f"subkey 0x{chosen_key:02x} ({chr(chosen_key) if 32 <= chosen_key < 127 else '.'})",
            "Y_size": m_next,
            "Y_over_h1": round(m_next / blk.h1, 4),
            "step_gap_mean": round(sum(step_gaps) / m_next, 2),
            "step_gap_p99": percentile(s_sg, 0.99),
            "step_gap_max": s_sg[-1],
            "byte_gap_mean": round(sum(byte_gaps) / m_next, 2),
            "byte_gap_p99": percentile(s_bg, 0.99),
            "byte_gap_max": s_bg[-1],
            "probe_mode": PiCost(
                Q=q_build_cum + m_next,
                D=d_build_cum + m_next,
                P=k + 1,
                S=n + m_next * 8,
                Q_acq=q_build_cum,
                Q_run=m_next,
                D_acq=d_build_cum,
                D_run=m_next,
                P_acq=k,
                P_run=1,
                P_bytes=round((n + q_build_cum * 4) / n, 3),
                s_cursors=256,
                notes=f"k={k} probe mode: Q_acq grows by |Y_{{k-1}}|, D collapses!",
            ).as_dict(blk.h1, n),
            "byte_exact": True,
        })
        R_cur = R_next
        W_cur = W_next

    # Also measure exact NEEDS M-21 for the most frequent pair (best_pair = (b_prev, b_curr))
    # Rows whose F[x] == b_curr and L[x] == b_prev (exact pair in text!)
    bp_l, bp_f = best_pair
    pair_rows = [x for x in range(C[bp_f], C[bp_f] + counts[bp_f]) if L[x] == bp_l]
    pair_set = set(pair_rows)
    assert len(pair_rows) == best_pair_cnt
    pair_gaps: list[int] = []
    for start in pair_rows:
        cur = lf[start]
        st = 1
        while cur not in pair_set:
            cur = lf[cur]
            st += 1
        pair_gaps.append(st)
    assert sum(pair_gaps) == n
    spg = sorted(pair_gaps)

    return {
        "levels": levels_out,
        "most_freq_pair": {
            "pair_hex": f"({bp_l:02x}, {bp_f:02x})",
            "pair_ascii": f"{chr(bp_l) if 32<=bp_l<127 else '.'}{chr(bp_f) if 32<=bp_f<127 else '.'}",
            "Y_size": len(pair_rows),
            "Y_over_h1": round(len(pair_rows) / blk.h1, 4),
            "gap_mean": round(sum(pair_gaps) / len(pair_gaps), 2),
            "gap_p99": percentile(spg, 0.99),
            "gap_max": spg[-1],
            "probe_Q": n + len(pair_rows),
            "probe_D": spg[-1] + len(pair_rows),
        },
    }


def run_orthogonal_two_axis(blk: BlockData, L_min: int = 16, max_k: int = 3) -> dict:
    """Sense 1B: Orthogonal Row-Axis (thick strips) FIRST, Text-Axis landmark
    induction on the contracted retained map R SECOND!

    All thick-strip acquisition (Q_acq_strips + Q_acq_glue) is explicitly charged.
    """
    rs = build_retained_system_genuine(blk, L_min=L_min)
    m = rs.m
    R_cur = rs.R[:]
    # Represent words as lists of DAG node IDs (O(1) per retained step, NO n-byte copy!)
    W_cur: list[list[int]] = [[wid] for wid in rs.R_word_id]
    # First byte of each retained row is simply L[rs.rows[i]]!
    first_bytes = [blk.L[r_idx] for r_idx in rs.rows]

    cur_start = rs.mark_rank
    cur_prefix_dags: list[int] = []
    q_acq_thick = rs.Q_acq_strips + rs.Q_acq_glue
    q_build_text = 0
    d_build_text = 0

    def verify_retained_landmark_decode(
        R_k: list[int], W_k: list[list[int]], start_i: int, pref_dags: list[int]
    ) -> None:
        full_back = bytearray()
        idx = start_i
        steps = 0
        while True:
            for wid in W_k[idx]:
                expand_dag_word(rs.dag_left, rs.dag_right, wid, full_back)
            idx = R_k[idx]
            steps += 1
            if idx == start_i:
                break
        assert len(full_back) == blk.n and steps == len(R_k)
        pref_bytes = bytearray()
        for wid in pref_dags:
            expand_dag_word(rs.dag_left, rs.dag_right, wid, pref_bytes)
        sh = len(pref_bytes)
        if sh > 0:
            assert full_back[blk.n - sh :] == pref_bytes
            ordered_back = bytes(full_back[blk.n - sh :]) + bytes(full_back[: blk.n - sh])
        else:
            ordered_back = bytes(full_back)
        assert ordered_back[::-1] == blk.T_ref, f"Orthogonal 2-axis mismatch on {blk.name}"

    levels: list[dict] = []
    # Level 0 is baseline Open retained map
    for k in range(1, max_k + 1):
        if k == 1:
            keys = first_bytes
        else:
            # Next symbol: first byte of second retained step in W_cur, or L of R_cur target
            keys = [blk.L[rs.rows[R_cur[i]]] if k == 2 else (W_cur[i][1] if len(W_cur[i]) > 1 else -1) for i in range(len(R_cur))]
            # Actually simpler & strictly valid: use first byte of the *next* retained node W_cur[i][-1] mod 256
            # Better: keep the actual retained-row id of each landmark!
            keys = [rs.dag_left[W_cur[i][1]] if (len(W_cur[i]) > 1 and rs.dag_right[W_cur[i][1]] == -1) else (W_cur[i][0] & 0xF) for i in range(len(R_cur))]

        cnts = Counter(keys)
        chosen_key = None
        for cand_k, cand_c in cnts.most_common():
            if 2 <= cand_c < len(R_cur):
                chosen_key = cand_k
                break
        if chosen_key is None:
            break

        sub_indices = [i for i, kv in enumerate(keys) if kv == chosen_key]
        sub_rank = [-1] * len(R_cur)
        for ni, oi in enumerate(sub_indices):
            sub_rank[oi] = ni

        m_next = len(sub_indices)
        R_next = [0] * m_next
        W_next: list[list[int]] = []
        step_gaps: list[int] = [0] * m_next
        for ni, oi in enumerate(sub_indices):
            dags: list[int] = []
            cur = oi
            st = 0
            while True:
                dags.extend(W_cur[cur])
                cur = R_cur[cur]
                st += 1
                if sub_rank[cur] != -1:
                    break
            R_next[ni] = sub_rank[cur]
            W_next.append(dags)
            step_gaps[ni] = st

        if sub_rank[cur_start] == -1:
            c_idx = cur_start
            while sub_rank[c_idx] == -1:
                cur_prefix_dags.extend(W_cur[c_idx])
                c_idx = R_cur[c_idx]
            cur_start = sub_rank[c_idx]
        else:
            cur_start = sub_rank[cur_start]

        verify_retained_landmark_decode(R_next, W_next, cur_start, cur_prefix_dags)

        q_build_text += len(R_cur)
        d_build_text += max(step_gaps)
        s_sg = sorted(step_gaps)
        total_Q = q_acq_thick + q_build_text + m_next
        total_D = rs.D_acq + d_build_text + m_next
        levels.append({
            "k": k,
            "Y_size": m_next,
            "Y_over_h1": round(m_next / blk.h1, 4),
            "step_gap_mean": round(sum(step_gaps) / m_next, 2),
            "step_gap_max": s_sg[-1],
            "cost": PiCost(
                Q=total_Q,
                D=total_D,
                P=rs.P_acq + k + 1,
                S=rs.S_acq + m * 8 + m_next * 8,
                Q_acq=q_acq_thick + q_build_text,
                Q_run=m_next,
                D_acq=rs.D_acq + d_build_text,
                D_run=m_next,
                P_acq=rs.P_acq + k,
                P_run=1,
                P_bytes=round((blk.r * 6 + q_build_text * 8) / blk.n, 3),
                s_cursors=256,
                notes=f"Orthogonal L_min={L_min} + text landmark k={k}: D collapses from {m} to {total_D}",
            ).as_dict(blk.h1, blk.n),
            "byte_exact": True,
        })
        R_cur = R_next
        W_cur = W_next

    return {
        "L_min": L_min,
        "m_retained": m,
        "Q_acq_thick": q_acq_thick,
        "D_acq_thick": rs.D_acq,
        "levels": levels,
    }


def run_forward_psi_f_intervals(blk: BlockData, ks: tuple[int, ...] = (2, 4, 6, 8)) -> dict:
    """Sense 1C: Forward-Psi F-interval k-mer sharing (text-axis word sharing
    WITHOUT needing Neon Witness 20.1's src < dst acyclicity order).

    Every row x in an F-interval I(w) shares the exact next k forward bytes w
    under Psi. On novel random hex, |I(w)| collapses to 1 past k ~ log_16(n);
    on repeated text (repeated SHA1s/phrases), |I(w)| >= 2 persists.
    """
    n = blk.n
    T = blk.T_ref
    # Build Psi = LF^-1 to check exact forward-k-mer grouping directly from L!
    lf = build_lf_array(blk.L, blk.C)
    psi = [0] * n
    for x, y in enumerate(lf):
        psi[y] = x

    # Identify width-1 rows (lone rows J1)
    is_w1 = bytearray(n)
    for s, w, _ in blk.runs:
        if w == 1:
            is_w1[s] = 1

    # F byte for every row
    F = bytearray(n)
    for b in range(256):
        c0 = blk.C[b]
        cnt = blk.counts[b]
        if cnt:
            F[c0 : c0 + cnt] = bytes([b]) * cnt

    # Byte-exact forward decode via Psi starting at Psi(primary_index)
    # (checks our Psi convention matches T_ref)
    out_psi = bytearray(n)
    cur = psi[blk.primary_index]
    for i in range(n):
        out_psi[i] = blk.L[cur]
        cur = psi[cur]
    assert bytes(out_psi) == T

    # Compute k-mer intervals I(w) in F (since rows are sorted by suffix,
    # identical k-mers form CONTIGUOUS intervals in [0, n)!)
    # For each row x, its forward k-mer is (F[x], F[psi[x]], ..., F[psi^{k-1}[x]]).
    # Build incrementally by doubling / stepping k = 1..max(ks)
    max_k = max(ks)
    # Track group id of each row at step k (same group iff same k-mer prefix in F)
    # At k=1, group is F[x]. At k -> k+1, group is (group[x], F[psi^k[x]])
    grp = list(F)
    psi_pow = list(range(n))  # psi^{k-1}[x]
    results = {}
    for k in range(1, max_k + 1):
        if k > 1:
            psi_pow = [psi[p_x] for p_x in psi_pow]
            # Contiguous intervals in F! Just detect boundaries where grp[x] != grp[x-1]
            # or F[psi_pow[x]] != F[psi_pow[x-1]]
            new_grp = [0] * n
            gid = 0
            prev_a = grp[0]
            prev_b = F[psi_pow[0]]
            for x in range(1, n):
                a = grp[x]
                b = F[psi_pow[x]]
                if a != prev_a or b != prev_b:
                    gid += 1
                    prev_a = a
                    prev_b = b
                new_grp[x] = gid
            grp = new_grp

        if k in ks:
            # Count intervals and how many w1 rows lie in shared (size >= 2) intervals
            n_intervals = grp[-1] + 1
            # Size of each interval and w1 count inside
            shared_w1 = 0
            shared_intervals_with_w1 = 0
            x = 0
            while x < n:
                g = grp[x]
                y = x + 1
                w1_in = is_w1[x]
                while y < n and grp[y] == g:
                    w1_in += is_w1[y]
                    y += 1
                sz = y - x
                if sz >= 2 and w1_in > 0:
                    shared_w1 += w1_in
                    shared_intervals_with_w1 += 1
                x = y
            results[f"k={k}"] = {
                "n_intervals": n_intervals,
                "shared_w1_rows": shared_w1,
                "shared_w1_frac": round(shared_w1 / blk.h1, 4) if blk.h1 else 0.0,
                "shared_intervals_with_w1": shared_intervals_with_w1,
            }
    return {"byte_exact_psi": True, "f_intervals": results}


def main() -> None:
    blocks = [
        make_synthetic_novel_hex_iid(65536),
        make_synthetic_templated_hex(1000, 40),
    ] + load_admitted_blocks()

    all_results = {}
    for blk in blocks:
        print(f"\n=== Sense 1 (text-axis): {blk.name} (n={blk.n}, r={blk.r}, h1={blk.h1}) ===")
        r_1a = run_landmark_raw_lf(blk, max_k=3)
        r_1b = run_orthogonal_two_axis(blk, L_min=16, max_k=3)
        r_1c = run_forward_psi_f_intervals(blk)
        all_results[blk.name] = {
            "n": blk.n,
            "r": blk.r,
            "h1": blk.h1,
            "s1a_raw_landmark": r_1a,
            "s1b_orthogonal_2axis": r_1b,
            "s1c_forward_f_intervals": r_1c,
        }
        for lv in r_1a["levels"]:
            pm = lv["probe_mode"]
            print(
                f"  1A raw LF k={lv['k']}: |Y|={lv['Y_size']:6d} ({lv['Y_over_h1']:.3f}*h1) "
                f"Q={pm['Q']:7d} ({pm['Q_over_h1']:.2f}*h1) D={pm['D']:6d} P={pm['P']}"
            )
        mp = r_1a["most_freq_pair"]
        print(
            f"  1A best pair {mp['pair_ascii']} ({mp['pair_hex']}): |Y|={mp['Y_size']:6d} "
            f"gap mean={mp['gap_mean']} p99={mp['gap_p99']} max={mp['gap_max']} D={mp['probe_D']}"
        )
        for lv in r_1b["levels"]:
            c = lv["cost"]
            print(
                f"  1B ortho  k={lv['k']}: |Y|={lv['Y_size']:6d} ({lv['Y_over_h1']:.3f}*h1) "
                f"Q_tot={c['Q']:7d} ({c['Q_over_h1']:.2f}*h1, Q_run={c['Q_run']}) "
                f"D={c['D']:6d} P={c['P']}"
            )
        f8 = r_1c["f_intervals"]["k=8"]
        f4 = r_1c["f_intervals"]["k=4"]
        print(
            f"  1C F-int  k=4: shared_w1={f4['shared_w1_frac']*100:5.1f}% | "
            f"k=8: shared_w1={f8['shared_w1_frac']*100:5.1f}% (n_int={f8['n_intervals']})"
        )

    out_path = Path(__file__).resolve().parent / "results_s1_text_axis.json"
    out_path.write_text(json.dumps(all_results, indent=2) + "\n")
    print(f"\nSaved {out_path} [ALL BYTE-EXACT]")


if __name__ == "__main__":
    main()
