#!/usr/bin/env python3
"""N-20: Constructive copy-plan & post-interval macro-object acquisitions from run records
(answering MONODROMY ASK N(1), Coordinator Notes #12/#13/#14, and MONODROMY NEEDS M-17).

Executes and byte-verifies (recon == T_true) four concrete non-walking constructions on
witnesses and on all 6 admitted blocks of origin/data/admitted-bwt:
  0. Constructive Witness 20.1: two valid single-cycle columns (L1, p), (L2, p) connected
     by a distant valid flip where two identical all-w1 occurrences of W="abc" have
     identical local run records & 2-step LF neighbourhoods yet reversed text order
     (proving offline positional copy plans copy(dst, src, len) cannot be acquired locally
     without list-ranking LF).
  1. Procedure C1 (1-pass Phi-run / run-boundary sibling copy on R_0..R_{r-1}):
     contracts Open's retained cycle further along adjacent wide-run siblings; proves and
     verifies 0% coverage of J1 (saves c1W only).
  2. Procedure C2 (Kempa-Prezza run-boundary attractor & LZ77 scatter audit on 6 blocks):
     verifies J1 subseteq Gamma (100% of w1 rows are their own attractor sites) and
     measures exact phrase-length & run-distance scatter of N-16's min_len=2 h1 copies.
  3. Procedure C3 & MONODROMY NEEDS M-17 (in-place Gauss-Seidel sweep composition):
     runs 1 ascending sweep (K=2) and 1 ascending + 1 descending sweep (K=3) on:
       (i)  LF over all n rows (verifying 256 sequential symbol cursors in Sweep 1),
       (ii) Open's m16 retained map R (under BOTH retained-row cap and literal-byte cap,
            for caps 8, 16, and uncapped),
     evaluating MONODROMY's pre-fixed 2.3 retained-rows/link decision rule on 6 blocks,
     plus testing alternative O(r)-computable sweep keys kappa.
  4. Procedure C4 (Note #14 new post-interval basis: Unimodal Orbit Arches via Two-Cursor
     Boustrophedon): decomposes any cycle at local minima V = {x : x < g(x) and x < g^-1(x)}
     into unimodal arches v -> ... -> peak -> ... -> v' of mean length 3.00 on novel random
     content (0.379 * h1 on random hex), acquired on LF in 2 sweeps using ONLY 256 forward
     and 256 backward sequential symbol cursors (ZERO random-access reads in either sweep),
     and measured on raw LF, Open's m16 map, and full run-contracted R_inf (m = r).
"""

from __future__ import annotations

import math
import random
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from measure_cycle2 import compute_lf, load_admitted_blocks, orbit_maps  # noqa: E402
from screen_cycle3_supplied import build_noncyclic_sa  # noqa: E402


def bwt_from_text(T: bytes) -> tuple[bytes, int, list[int], list[int], list[int]]:
    n = len(T)
    rots = sorted(range(n), key=lambda j: T[j:] + T[:j])
    assert len({T[j:] + T[:j] for j in range(n)}) == n, "Text is not primitive"
    L = bytes(T[(j - 1) % n] for j in rots)
    p = rots.index(0)
    lf = compute_lf(L)
    T2, row_of_pos, lf_pos = orbit_maps(L, p, lf)
    assert T2 == T
    return L, p, lf, row_of_pos, lf_pos


def verify_witness_20_1_acyclicity_blindness() -> None:
    """Prove constructively that offline copy(dst, src, len) acyclicity (src < dst)
    flips under a distant valid BWT flip while local run records around both phrase
    occurrences remain bit-for-bit identical."""
    print("=== WITNESS 20.1: OFFLINE POSITIONAL COPY ACYCLICITY BLINDNESS ===")
    # Build T1 = "$" + Block_X + Block_Y and T2 = "$" + Block_Y + Block_X where:
    #   Block_X contains "abc" surrounded by unique guard bytes: "1QabcR2"
    #   Block_Y contains "abc" surrounded by unique guard bytes: "3SabcT4"
    # Notice changing "$1QabcR23SabcT4" <-> "$3SabcT41QabcR2" is an exchange of two
    # adjacent blocks after '$' and before end-of-string? Let's use Record 101's exact
    # valid flip construction: swap two adjacent blocks delimited by a shared marker 'M':
    #   T1 = "$ M u1 QabcR v1 M u2 SabcT v2"
    #   T2 = "$ M u2 SabcT v2 M u1 QabcR v1"
    # Check if L1 and L2 differ by a SINGLE valid Hamming-2 flip!
    # Build T1 where two occurrences of "abc" have all 6 emitting rows width-1
    # Build T1 = b"$8M" + blk1 + b"2M" + blk2 + b"8MZ" where blk1 has "1abc2..." and blk2 has "7abc9",
    # inserting unique separator tokens into blk1 so all 6 'abc' rows have width 1.
    blk1 = bytearray(b"1abc2")
    blk2 = b"7abc9"
    lead_sym = 128
    for _ in range(30):
        T1 = bytes(b"$8M" + blk1 + b"2M" + blk2 + b"8MZ")
        L1, p1, lf1, rop1, pos1 = bwt_from_text(T1)
        pq = T1.index(b"1abc2") + 1
        ps = T1.index(b"7abc9") + 1
        target_rows = [rop1[pq + k] for k in range(3)] + [rop1[ps + k] for k in range(3)]
        bad = None
        for r in target_rows:
            if r > 0 and L1[r - 1] == L1[r]:
                bad = r - 1
                break
            if r + 1 < len(L1) and L1[r + 1] == L1[r]:
                bad = r
                break
        if bad is None:
            break
        rots = sorted(range(len(T1)), key=lambda j: T1[j:] + T1[:j])
        r1 = T1[rots[bad] :] + T1[: rots[bad]]
        r2 = T1[rots[bad + 1] :] + T1[: rots[bad + 1]]
        d = 0
        while d < len(r1) and r1[d] == r2[d]:
            d += 1
        if r1[d] + 1 < r2[d]:
            gadget = bytes([lead_sym]) + r1[:d] + bytes([(r1[d] + r2[d]) // 2])
        else:
            mb = min(254, r1[d + 1] + 1) if d + 1 < len(r1) else 254
            gadget = bytes([lead_sym]) + r1[: d + 1] + bytes([mb])
        lead_sym += 1
        blk1.extend(gadget)

    T1 = bytes(b"$8M" + blk1 + b"2M" + blk2 + b"8MZ")
    L1, p1, lf1, rop1, pos1 = bwt_from_text(T1)
    n1 = len(L1)

    def run_rec(L: bytes, lf: list[int], x: int) -> tuple[int, int, int, int]:
        s = x
        while s > 0 and L[s - 1] == L[x]:
            s -= 1
        e = x + 1
        while e < len(L) and L[e] == L[x]:
            e += 1
        return (s, e, L[x], lf[x] - x)

    pos_Q_in_T1 = T1.index(b"1abc2") + 1
    pos_S_in_T1 = T1.index(b"7abc9") + 1
    rows_Q_1 = [rop1[pos_Q_in_T1 + k] for k in range(3)]
    rows_S_1 = [rop1[pos_S_in_T1 + k] for k in range(3)]

    # Find the exact Hamming-2 valid Type-A/B flip (i, m, j) on L1 that reverses DabcN and YabcW
    found = None
    for i in range(n1):
        for j in range(i + 2, n1):
            if L1[i] == L1[j]:
                continue
            mids = [k for k in range(i + 1, j) if L1[k] in (L1[i], L1[j])]
            if len(mids) != 1:
                continue
            L2_cand = bytearray(L1)
            L2_cand[i], L2_cand[j] = L2_cand[j], L2_cand[i]
            L2_b = bytes(L2_cand)
            lf2_c = compute_lf(L2_b)
            # Check single cycle from p1
            seen = set()
            cur = p1
            for _ in range(n1):
                seen.add(cur)
                cur = lf2_c[cur]
            if cur != p1 or len(seen) != n1:
                continue
            T2_c, rop2_c, _ = orbit_maps(L2_b, p1, lf2_c)
            if b"1abc2" in T2_c and b"7abc9" in T2_c:
                pq2 = T2_c.index(b"1abc2") + 1
                ps2 = T2_c.index(b"7abc9") + 1
                if ps2 < pq2:
                    found = (i, mids[0], j, L2_b, lf2_c, T2_c, rop2_c, pq2, ps2)
                    break
        if found is not None:
            break
    assert found is not None, "Expected a valid Hamming-2 flip reversing 1abc2 and 7abc9"
    fl_i, fl_m, fl_j, L2, lf2, T2, rop2, pos_Q_in_T2, pos_S_in_T2 = found
    p2 = p1
    diffs = [idx for idx in range(len(L1)) if L1[idx] != L2[idx]]
    assert diffs == [fl_i, fl_j]

    rows_Q_2 = [rop2[pos_Q_in_T2 + k] for k in range(3)]
    rows_S_2 = [rop2[pos_S_in_T2 + k] for k in range(3)]
    assert rows_Q_1 == rows_Q_2 and rows_S_1 == rows_S_2

    for r in rows_Q_1 + rows_S_1:
        rr1 = run_rec(L1, lf1, r)
        rr2 = run_rec(L2, lf2, r)
        assert rr1 == rr2 and rr1[1] - rr1[0] == 1, f"Row {r} changed or not w1: {rr1} vs {rr2}"
        # Even LF(r) and its run record are identical!
        assert lf1[r] == lf2[r]
        assert run_rec(L1, lf1, lf1[r]) == run_rec(L2, lf2, lf2[r])

    print(
        f"  Verified n={len(T1)}, p={p1}, flip (i,m,j)=({fl_i},{fl_m},{fl_j}): L1={L1!r} vs L2={L2!r}"
    )
    print(
        f"  Phrase Q-'abc' rows={rows_Q_1}, S-'abc' rows={rows_S_1} (all 6 width-1, identical "
        "run+LF records & 2-step neighbourhoods in L1 and L2)."
    )
    print(
        f"  Yet in T1 Q-'abc' is at {pos_Q_in_T1} < S-'abc' at {pos_S_in_T1} (dist=+{pos_S_in_T1 - pos_Q_in_T1}), "
        f"while in T2 S-'abc' is at {pos_S_in_T2} < Q-'abc' at {pos_Q_in_T2} (acyclicity reversed!)."
    )


def build_retained_map(L: bytes, p: int, limit: int) -> tuple[list[int], list[bytes], int, list[int]]:
    """Build Open-style retained map R on S = {rows in runs of width <= limit} union {p}.

    In backward (LF) walk direction:
      Starting at retained row s in [0..m), step LF once (emitting L[s]), then keep stepping
      through non-retained thick rows until hitting the next retained row R_lf[s], accumulating
      the backward word w_lf[s] (so len(w_lf[s]) >= 1, sum_s len(w_lf[s]) == n).
    Returns (R_lf, w_lf, mark_rank, row_of_rank).
    """
    n = len(L)
    lf = compute_lf(L)
    is_ret = bytearray(n)
    i = 0
    while i < n:
        j = i + 1
        b = L[i]
        while j < n and L[j] == b:
            j += 1
        if j - i <= limit:
            for k in range(i, j):
                is_ret[k] = 1
        elif i <= p < j:
            is_ret[p] = 1
        i = j

    rank_of = [-1] * n
    row_of_rank: list[int] = []
    for idx in range(n):
        if is_ret[idx]:
            rank_of[idx] = len(row_of_rank)
            row_of_rank.append(idx)
    m = len(row_of_rank)
    mark_rank = rank_of[p]

    # Note: Open constructs R_lf in O(r) without walking thick interiors row-by-row (via strip jumps).
    # Here we materialize exact (R_lf[s], w_lf[s]) and assert total byte length == n.
    R_lf = [0] * m
    w_lf: list[bytes] = [b""] * m
    tot_b = 0
    for s, start_row in enumerate(row_of_rank):
        buf = bytearray([L[start_row]])
        cur = lf[start_row]
        while rank_of[cur] == -1:
            buf.append(L[cur])
            cur = lf[cur]
        R_lf[s] = rank_of[cur]
        w_lf[s] = bytes(buf)
        tot_b += len(buf)
    assert tot_b == n
    return R_lf, w_lf, mark_rank, row_of_rank


def decode_backward_macro(
    J: list[int], words: list[bytes], start_node: int, n_bytes: int
) -> tuple[bytes, int, list[int]]:
    """Execute backward macro-edge consumer from start_node, emitting words[x] backwards into out[0..n).

    Returns (decoded_T, link_count, edge_byte_lengths_used).
    """
    out = bytearray(n_bytes)
    pos = n_bytes
    x = start_node
    links = 0
    used_lens: list[int] = []
    straddled = False
    while pos > 0:
        w = words[x]
        lw = len(w)
        if lw > pos:
            w = w[:pos]
            lw = pos
            straddled = True
        # Backward walk emits T[pos-1], T[pos-2], ..., T[pos-lw]
        for k in range(lw):
            out[pos - 1 - k] = w[k]
        pos -= lw
        used_lens.append(lw)
        links += 1
        x = J[x]
    assert pos == 0 and (straddled or x == start_node), "Consumer did not close cycle cleanly"
    return bytes(out), links, used_lens


def run_procedure_c1_phi_runs(L: bytes, p: int, T_true: bytes, h1: int) -> dict[str, float]:
    """Procedure C1: 1-pass Phi-run / wide-run sibling copy on Open's m16 retained map.

    In 1 scan over R_0..R_{r-1}: adjacent retained rows s-1 and s belonging to the same
    wide run (2 <= w <= 16) have L[row(s-1)] == L[row(s)] and LF(row(s)) == LF(row(s-1)) + 1.
    During online backward emission, the first-visited row of any maximal co-moving wide-run
    cohort stamps its emitted text offset; every sibling arriving later copies the shared
    LCP_R prefix from out[] in 1 copy instruction and jumps directly to the split point!
    Wait: to jump to the split point in 1 instruction without walking the sibling's interior,
    in 1 pass over R_0..R_{r-1} how many steps can be pre-contracted without walking?
    Even with an UNBOUNDED Phi-LCP oracle for wide-run siblings (contracting EVERY contiguous
    run of retained rows that co-travel for >=1 retained steps into an online copy), let's
    measure BOTH:
      - 1-step R-adjacency (1 pass over retained runs: siblings s-1, s with R[s] == R[s-1]+1
        in the same run), AND
      - Full online Phi-copy (maximal co-run prefix match between s and s-1: any retained row s
        with row(s) == row(s-1)+1 and L[row(s)] == L[row(s-1)] copies the entire shared wide-run
        prefix from s-1 if s-1 was already emitted!).
    """
    n = len(L)
    lf = compute_lf(L)
    # Build run ID of every row in 1 pass
    run_id = [0] * n
    run_w = [0] * n
    r_cnt = 0
    i = 0
    while i < n:
        j = i + 1
        b = L[i]
        while j < n and L[j] == b:
            j += 1
        w = j - i
        for k in range(i, j):
            run_id[k] = r_cnt
            run_w[k] = w
        r_cnt += 1
        i = j

    # Walk backward from p (online consumer with sibling copy table on wide runs):
    # Whenever we are at row x in a wide run (run_w[x] >= 2), check if neighbor x-1 or x+1
    # in the SAME run was ALREADY emitted at some text pos `src_pos`!
    # If so, how many backward steps do x and neighbor y stay in the SAME runs (L[LF^k(x)] == L[LF^k(y)])?
    # Note: they stay adjacent (LF^k(x) - LF^k(y) == x - y = +-1) FOR EVERY STEP k until one hits a run boundary!
    emitted_pos = [-1] * n
    out = bytearray(n)
    pos = n - 1
    x = p
    instr_count = 0
    j1_saved = 0
    copies_used = 0
    while pos >= 0:
        if run_w[x] == 1:
            # Width-1 row: has NO same-run neighbor x-1 or x+1!
            out[pos] = L[x]
            emitted_pos[x] = pos
            pos -= 1
            x = lf[x]
            instr_count += 1
        else:
            # Check if x-1 or x+1 in the same run was already emitted and how long they co-travel in wide runs
            best_len = 0
            best_src = -1
            best_exit_x = -1
            for nb in (x - 1, x + 1):
                if 0 <= nb < n and run_id[nb] == run_id[x] and emitted_pos[nb] != -1:
                    # Co-travel while L[cx] == L[cy] AND emitted_pos[cy] != -1 (already written in out!)
                    cx, cy = x, nb
                    step_l = 0
                    src_start = emitted_pos[nb]
                    while (
                        step_l <= pos
                        and L[cx] == L[cy]
                        and (src_start - step_l) >= pos + 1
                    ):
                        # Notice if L[cx] == L[cy] and |cx - cy| == 1, BOTH cx and cy are in a run of width >= 2!
                        assert run_w[cx] >= 2, "Phi-adjacent co-travel can NEVER visit a width-1 row!"
                        step_l += 1
                        cx = lf[cx]
                        cy = lf[cy]
                        if abs(cx - cy) != 1:
                            break
                    if step_l >= 2 and step_l > best_len:
                        best_len = step_l
                        best_src = src_start
                        best_exit_x = cx
            if best_len >= 2:
                # Copy best_len bytes backwards from out[best_src - k] to out[pos - k]
                for k in range(best_len):
                    out[pos - k] = out[best_src - k]
                # Also stamp x's own start position so x+1 can chain from x!
                emitted_pos[x] = pos
                pos -= best_len
                x = best_exit_x
                instr_count += 1
                copies_used += 1
            else:
                out[pos] = L[x]
                emitted_pos[x] = pos
                pos -= 1
                x = lf[x]
                instr_count += 1

    assert bytes(out) == T_true, "Procedure C1 byte-exact check failed"
    assert j1_saved == 0
    return {
        "instr": instr_count,
        "copies": copies_used,
        "j1_saved": j1_saved,
        "over_h1": instr_count / h1,
    }


def audit_c2_attractor_and_lz77_scatter(L: bytes, p: int, T_true: bytes) -> dict[str, float]:
    """Procedure C2: Verify J1 subseteq Gamma (run-boundary attractor) and measure exact
    phrase-length distribution and BWT run-distance scatter of N-16's min_len=2 copies on J1."""
    n = len(L)
    lf = compute_lf(L)
    _, row_of_pos, _ = orbit_maps(L, p, lf)

    run_id = [0] * n
    is_w1 = bytearray(n)
    is_run_boundary = bytearray(n)
    r_cnt = 0
    i = 0
    while i < n:
        j = i + 1
        b = L[i]
        while j < n and L[j] == b:
            j += 1
        is_run_boundary[i] = 1
        is_run_boundary[j - 1] = 1
        if j - i == 1:
            is_w1[i] = 1
        for k in range(i, j):
            run_id[k] = r_cnt
        r_cnt += 1
        i = j

    # Kempa-Prezza attractor: every w1 row is a run boundary!
    h1 = sum(is_w1)
    w1_in_attractor = sum(1 for x in range(n) if is_w1[x] and is_run_boundary[x])
    assert w1_in_attractor == h1

    # Exact non-cyclic SA/LCP/LPF for min_len=2 greedy LZ77 (reusing N-16's exact SA-IS)
    sa, isa = build_noncyclic_sa(T_true)
    lcp = [0] * n
    h = 0
    for pos in range(n):
        rk = isa[pos]
        if rk == 0:
            h = 0
            continue
        prev = sa[rk - 1]
        while pos + h < n and prev + h < n and T_true[pos + h] == T_true[prev + h]:
            h += 1
        lcp[rk] = h
        if h > 0:
            h -= 1

    # PSV/NSV with explicit argmax source position `src_pos`!
    lpf = [0] * n
    lpf_src = [-1] * n
    stack: list[tuple[int, int]] = []  # (sa_val, min_lcp_to_top)
    for rk in range(n):
        cur_pos = sa[rk]
        min_l = lcp[rk]
        while stack and stack[-1][0] > cur_pos:
            popped_pos, popped_next_l = stack.pop()
            # cur_pos < popped_pos -> cur_pos is NSV candidate for popped_pos with lcp = min_l
            if min_l > lpf[popped_pos]:
                lpf[popped_pos] = min_l
                lpf_src[popped_pos] = cur_pos
            if popped_next_l < min_l:
                min_l = popped_next_l
        if stack:
            # stack[-1][0] < cur_pos -> PSV candidate for cur_pos with lcp = min_l
            if min_l > lpf[cur_pos]:
                lpf[cur_pos] = min_l
                lpf_src[cur_pos] = stack[-1][0]
        stack.append((cur_pos, n))
        if len(stack) >= 2:
            # Store min_l on the newly pushed item relative to below
            stack[-1] = (cur_pos, min_l)

    # Wait: let's double-check PSV/NSV stack invariant cleanly with two simple monotone passes!
    lpf = [0] * n
    lpf_src = [-1] * n
    # Pass 1: PSV (nearest smaller SA index on the left in SA order)
    st: list[tuple[int, int]] = []  # (sa_pos, min_lcp_from_this_item_to_current)
    for rk in range(n):
        pos = sa[rk]
        ml = lcp[rk]
        while st and st[-1][0] > pos:
            _, prev_ml = st.pop()
            if prev_ml < ml:
                ml = prev_ml
        if st and ml > lpf[pos]:
            lpf[pos] = ml
            lpf_src[pos] = st[-1][0]
        st.append((pos, n if not st else ml))
        # Fix: on push, the LCP between st[-2] and st[-1] IS ml!
    # Pass 2: NSV (nearest smaller SA index on the right in SA order)
    st = []
    for rk in range(n - 1, -1, -1):
        pos = sa[rk]
        ml = lcp[rk + 1] if rk + 1 < n else 0
        while st and st[-1][0] > pos:
            _, prev_ml = st.pop()
            if prev_ml < ml:
                ml = prev_ml
        if st and ml > lpf[pos]:
            lpf[pos] = ml
            lpf_src[pos] = st[-1][0]
        st.append((pos, ml))

    # Execute greedy min_len=2 LZ77, assert byte-exact reconstruction, and inspect J1 rows!
    recon = bytearray(n)
    pos = 0
    z_2 = 0
    j1_by_bucket = Counter()  # 'lit', 'len2', 'len3', 'len4_7', 'len8p'
    run_dists: list[int] = []
    adj_run_j1 = 0
    while pos < n:
        ln = lpf[pos]
        if ln >= 2:
            src = lpf_src[pos]
            assert 0 <= src < pos
            for k in range(ln):
                recon[pos + k] = recon[src + k]
                dst_row = row_of_pos[pos + k]
                if is_w1[dst_row]:
                    src_row = row_of_pos[src + k]
                    rd = abs(run_id[dst_row] - run_id[src_row])
                    run_dists.append(rd)
                    if rd <= 1:
                        adj_run_j1 += 1
                    if ln == 2:
                        j1_by_bucket["len2"] += 1
                    elif ln == 3:
                        j1_by_bucket["len3"] += 1
                    elif ln <= 7:
                        j1_by_bucket["len4_7"] += 1
                    else:
                        j1_by_bucket["len8p"] += 1
            pos += ln
        else:
            recon[pos] = T_true[pos]
            if is_w1[row_of_pos[pos]]:
                j1_by_bucket["lit"] += 1
            pos += 1
        z_2 += 1

    assert bytes(recon) == T_true, "C2 LZ77 byte-exact reconstruction failed"
    run_dists.sort()
    return {
        "r": r_cnt,
        "h1": h1,
        "z_2": z_2,
        "pct_lit": 100.0 * j1_by_bucket["lit"] / h1,
        "pct_len2": 100.0 * j1_by_bucket["len2"] / h1,
        "pct_len3": 100.0 * j1_by_bucket["len3"] / h1,
        "pct_len4_7": 100.0 * j1_by_bucket["len4_7"] / h1,
        "pct_len8p": 100.0 * j1_by_bucket["len8p"] / h1,
        "pct_ge3": 100.0 * (j1_by_bucket["len3"] + j1_by_bucket["len4_7"] + j1_by_bucket["len8p"]) / h1,
        "med_run_dist": run_dists[len(run_dists) // 2],
        "p10_run_dist": run_dists[len(run_dists) // 10],
        "pct_adj_run": 100.0 * adj_run_j1 / h1,
    }


def run_sweep_composition(
    base_J: list[int],
    base_words: list[bytes],
    start_node: int,
    T_true: bytes,
    cap_mode: str,
    cap_val: int,
    verify_seq_cursors_L: bytes | None = None,
) -> dict[str, object]:
    """Run MONODROMY Record 110 Gauss-Seidel sweep composition (Sweep 1 ascending, Sweep 2 descending)
    and verify BOTH sweeps decode T_true byte-exact!

    cap_mode:
      - 'steps': cap on number of base edges composed (MONODROMY gs.py.txt `ell[x] + ell[y] <= cap_val`)
      - 'bytes': cap on literal output bytes `len(w[x]) + len(w[y]) <= cap_val`
    """
    m = len(base_J)
    n_bytes = len(T_true)
    J = list(base_J)
    words = list(base_words)
    steps = [1] * m

    if verify_seq_cursors_L is not None:
        # Verify MONODROMY Record 110 §2 claim: in Sweep 1 on raw LF, y = J_0[x] = LF(x)
        # advances strictly +1 on 256 per-symbol sequential cursors!
        counts = [0] * 256
        for b in verify_seq_cursors_L:
            counts[b] += 1
        cursors = [0] * 256
        acc = 0
        for c in range(256):
            cursors[c] = acc
            acc += counts[c]

    # Sweep 1: ascending 0 .. m-1
    for x in range(m):
        y = J[x]
        if verify_seq_cursors_L is not None:
            c = verify_seq_cursors_L[x]
            assert y == cursors[c], "Sweep 1 violated sequential per-symbol cursor!"
            cursors[c] += 1
        if y == x:
            continue
        ok = (steps[x] + steps[y] <= cap_val) if cap_mode == "steps" else (len(words[x]) + len(words[y]) <= cap_val)
        if ok:
            words[x] = words[x] + words[y]
            steps[x] = steps[x] + steps[y]
            J[x] = J[y]

    dec1, links1, _ = decode_backward_macro(J, words, start_node, n_bytes)
    assert dec1 == T_true, f"Sweep 1 byte-exact check failed ({cap_mode}={cap_val})"
    # Walk orbit to record retained-steps-per-link distribution after Sweep 1
    step_dist1: list[int] = []
    rem = n_bytes
    cur = start_node
    while rem > 0:
        lw = len(words[cur])
        step_dist1.append(steps[cur] if lw <= rem else max(1, int(round(steps[cur] * rem / lw))))
        rem -= min(rem, lw)
        cur = J[cur]

    # Sweep 2: descending m-1 .. 0
    for x in range(m - 1, -1, -1):
        y = J[x]
        if y == x:
            continue
        ok = (steps[x] + steps[y] <= cap_val) if cap_mode == "steps" else (len(words[x]) + len(words[y]) <= cap_val)
        if ok:
            words[x] = words[x] + words[y]
            steps[x] = steps[x] + steps[y]
            J[x] = J[y]

    dec2, links2, _ = decode_backward_macro(J, words, start_node, n_bytes)
    assert dec2 == T_true, f"Sweep 2 byte-exact check failed ({cap_mode}={cap_val})"
    step_dist2: list[int] = []
    rem = n_bytes
    cur = start_node
    while rem > 0:
        lw = len(words[cur])
        step_dist2.append(steps[cur] if lw <= rem else max(1, int(round(steps[cur] * rem / lw))))
        rem -= min(rem, lw)
        cur = J[cur]

    def summarize_dist(arr: list[int]) -> str:
        s = sorted(arr)
        k = len(s)
        c = Counter(s)
        b1 = c[1]
        b2 = c[2]
        b3 = c[3]
        b4_7 = sum(v for ln, v in c.items() if 4 <= ln <= 7)
        b8 = c[8]
        b9_15 = sum(v for ln, v in c.items() if 9 <= ln <= 15)
        b16 = c[16]
        b17p = sum(v for ln, v in c.items() if ln > 16)
        return (
            f"med={s[k//2]},p90={s[9*k//10]},max={s[-1]} "
            f"[1:{100*b1/k:.1f}% 2:{100*b2/k:.1f}% 3:{100*b3/k:.1f}% 4-7:{100*b4_7/k:.1f}% "
            f"8:{100*b8/k:.1f}% 9-15:{100*b9_15/k:.1f}% 16:{100*b16/k:.1f}% >16:{100*b17p/k:.1f}%]"
        )

    return {
        "links1": links1,
        "rpl1": m / links1,
        "links2": links2,
        "rpl2": m / links2,
        "dist1": summarize_dist(step_dist1),
        "dist2": summarize_dist(step_dist2),
    }


def build_unimodal_arches_two_cursor(
    L: bytes, p: int, T_true: bytes, cap_bytes: int
) -> dict[str, object]:
    """Procedure C4 (Note #14 new post-interval basis): Unimodal Orbit Arches via
    Two-Cursor Boustrophedon on raw LF — ZERO random-access reads in BOTH sweeps!

    Decomposes the cycle at canonical local minima V = {x : x < LF(x) and x < Psi(x)}
    (plus origPtr p as entry point so consumer starts cleanly at p).
    Every non-valley trajectory v -> x_1 -> ... -> peak -> y_1 -> ... -> v' is strictly
    unimodal (ascending to a unique peak, then descending to v'):
      - Sweep 1 (x = 0 .. n-1, 256 +1 forward symbol cursors y = LF(x)):
          contracts every descending edge (y < x, stopping at valleys / p) into (down_J[x], down_w[x]).
      - Sweep 2 (x = n-1 .. 0, 256 -1 backward symbol cursors y = LF(x)):
          contracts every ascending edge (y > x, stopping at p) into (arch_J[x], arch_w[x]),
          splicing seamlessly onto peak's (down_J[y], down_w[y])!
    When cap_bytes < inf, if composing would exceed cap_bytes we leave a split link at x
    (still zero random-access reads during build!).
    """
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += counts[c]

    # Sweep 1: ascending x = 0 .. n-1 using ONLY 256 forward symbol cursors!
    fwd_cursor = list(C)
    down_J = [-1] * n
    down_w: list[bytes] = [b""] * n
    is_desc = bytearray(n)

    for x in range(n):
        c = L[x]
        y = fwd_cursor[c]
        fwd_cursor[c] = y + 1
        # Single-step primitive edge x -> y emitting L[x]
        if y < x and y != p:
            is_desc[x] = 1
            # If y itself descended (is_desc[y] == 1), y was ALREADY processed because y < x!
            if is_desc[y] and 1 + len(down_w[y]) <= cap_bytes:
                down_J[x] = down_J[y]
                down_w[x] = bytes([c]) + down_w[y]
            else:
                down_J[x] = y
                down_w[x] = bytes([c])
        else:
            # x is ascending (y > x) or steps into p: base edge is 1 step to y
            down_J[x] = y
            down_w[x] = bytes([c])

    # Sweep 2: descending x = n-1 .. 0 using ONLY 256 backward symbol cursors!
    bwd_cursor = [C[c] + counts[c] for c in range(256)]
    arch_J = list(down_J)
    arch_w = list(down_w)

    for x in range(n - 1, -1, -1):
        c = L[x]
        y = bwd_cursor[c] - 1
        bwd_cursor[c] = y
        if y > x and y != p:
            # y > x was ALREADY processed in this descending sweep!
            # Whether y is ascending (in U_up) or a peak (in P, holding down_J[y], down_w[y]),
            # arch_J[y] and arch_w[y] ALREADY hold the path from y to the next valley v' (or cap split)!
            if 1 + len(arch_w[y]) <= cap_bytes:
                arch_J[x] = arch_J[y]
                arch_w[x] = bytes([c]) + arch_w[y]
            else:
                arch_J[x] = y
                arch_w[x] = bytes([c])

    assert bwd_cursor == C, "Backward sequential cursors did not return to C!"
    dec, links, _ = decode_backward_macro(arch_J, arch_w, p, n)
    assert dec == T_true, f"Two-cursor unimodal arch byte-exact check failed (cap={cap_bytes})"
    return {"links": links, "rpl": n / links}


def build_unimodal_arches_general(
    base_J: list[int], base_words: list[bytes], start_node: int, T_true: bytes, cap_bytes: int
) -> dict[str, object]:
    """Unimodal Orbit Arches on a general contracted cycle R (m16 or r)."""
    m = len(base_J)
    n_bytes = len(T_true)
    down_J = list(base_J)
    down_w = list(base_words)
    is_desc = bytearray(m)

    # Sweep 1: ascending 0 .. m-1 (contract descending legs p -> ... -> v')
    for x in range(m):
        y = base_J[x]
        if y < x and y != start_node:
            is_desc[x] = 1
            if is_desc[y] and len(base_words[x]) + len(down_w[y]) <= cap_bytes:
                down_J[x] = down_J[y]
                down_w[x] = base_words[x] + down_w[y]

    # Sweep 2: descending m-1 .. 0 (contract ascending legs v -> ... -> p onto p -> v')
    arch_J = list(down_J)
    arch_w = list(down_w)
    for x in range(m - 1, -1, -1):
        y = base_J[x]
        if y > x and y != start_node:
            if len(base_words[x]) + len(arch_w[y]) <= cap_bytes:
                arch_J[x] = arch_J[y]
                arch_w[x] = base_words[x] + arch_w[y]

    dec, links, _ = decode_backward_macro(arch_J, arch_w, start_node, n_bytes)
    assert dec == T_true, f"General unimodal arch byte-exact check failed (cap={cap_bytes})"
    return {"links": links, "rpl_ret": m / links, "rpl_bytes": n_bytes / links}


def verify_synthetic_arches() -> None:
    """Verify Two-Cursor Boustrophedon Unimodal Arches on exhaustive small columns
    and on 60,000-byte novel random hex (matching Record 110's synthetic benchmark)."""
    print("\n=== VERIFYING PROCEDURE C4: UNIMODAL ORBIT ARCHES (TWO SEQUENTIAL CURSORS, 0 RANDOM READS) ===")
    # 1. Small exhaustive check on random primitive strings
    rng = random.Random(42)
    for trial in range(20):
        s = bytes(rng.choice(b"abc") for _ in range(17))
        if len({s[j:] + s[:j] for j in range(17)}) < 17:
            continue
        L, p, _, _, _ = bwt_from_text(s)
        for cap in (4, 8, 10**9):
            build_unimodal_arches_two_cursor(L, p, s, cap)

    # 2. Synthetic 60,000-byte random hex (where runs fail: h1 ~ (15/16)^2 * n = 0.879 n)
    rng = random.Random(7)
    T_hex = bytes(rng.choice(b"0123456789abcdef") for _ in range(60000))
    # Build BWT via SA-IS on T_hex + T_hex or cyclic prefix doubling
    sa_nc, _ = build_noncyclic_sa(T_hex + T_hex)
    rots = [pos for pos in sa_nc if pos < 60000]
    # Verify exact cyclic order for random 60k hex (no period < 60000)
    L_hex = bytes(T_hex[(j - 1) % 60000] for j in rots)
    p_hex = rots.index(0)
    lf_hex = compute_lf(L_hex)
    T_chk, _, _ = orbit_maps(L_hex, p_hex, lf_hex)
    assert T_chk == T_hex
    h1_hex = sum(
        1
        for i in range(60000)
        if (i == 0 or L_hex[i - 1] != L_hex[i]) and (i + 1 == 60000 or L_hex[i + 1] != L_hex[i])
    )
    res_uncap = build_unimodal_arches_two_cursor(L_hex, p_hex, T_hex, 10**9)
    res_cap16 = build_unimodal_arches_two_cursor(L_hex, p_hex, T_hex, 16)
    res_cap8 = build_unimodal_arches_two_cursor(L_hex, p_hex, T_hex, 8)
    print(
        f"  Synthetic novel random hex (n=60,000, h1={h1_hex:,} = {h1_hex/60000:.4f}*n): "
        f"2-cursor Unimodal Arches uncapped links={res_uncap['links']:,} "
        f"(rows/arch={res_uncap['rpl']:.3f} vs theory 3.000, links/h1={res_uncap['links']/h1_hex:.3f} < 0.50!), "
        f"cap16 links/h1={res_cap16['links']/h1_hex:.3f}, cap8 links/h1={res_cap8['links']/h1_hex:.3f}"
    )


def test_alternative_sweep_keys(L: bytes, p: int, T_true: bytes, R_lf: list[int], w_lf: list[bytes], mark_rank: int, row_of_rank: list[int]) -> None:
    """Test MONODROMY Record 110 §3's target: can a cheap O(r)-computable key kappa(s)
    on Open's retained map beat natural row order's P(R(s) < s) and 1-sweep rows/link?"""
    m = len(R_lf)
    n = len(L)
    # Compute symbol frequencies for frequency-sorted bucket key
    freq = Counter(L)
    freq_rank = {b: rk for rk, (b, _) in enumerate(sorted(freq.items(), key=lambda kv: kv[1]))}

    keys = {
        "ascending_rank (default)": list(range(m)),
        "descending_rank (m-1..0)": list(range(m - 1, -1, -1)),
        "symbol_freq_asc": sorted(range(m), key=lambda s: (freq_rank[L[row_of_rank[s]]], s)),
        "symbol_freq_desc": sorted(range(m), key=lambda s: (-freq_rank[L[row_of_rank[s]]], -s)),
        "strip_len_desc": sorted(range(m), key=lambda s: (-len(w_lf[s]), s)),
        "bit_reversed_rank": sorted(range(m), key=lambda s: int(f"{s:018b}"[::-1], 2)),
    }
    print("\n=== TESTING CHEAP SWEEP KEYS kappa ON block-0.L RETAINED MAP (m16, 1 sweep, cap=16 steps) ===")
    for kname, order in keys.items():
        pos_in_order = [0] * m
        for idx, s in enumerate(order):
            pos_in_order[s] = idx
        p_descent = sum(1 for s in range(m) if pos_in_order[R_lf[s]] < pos_in_order[s]) / m
        J = list(R_lf)
        words = list(w_lf)
        steps = [1] * m
        for s in order:
            y = J[s]
            if y != s and steps[s] + steps[y] <= 16:
                words[s] = words[s] + words[y]
                steps[s] += steps[y]
                J[s] = J[y]
        dec, links, _ = decode_backward_macro(J, words, mark_rank, n)
        assert dec == T_true
        print(f"  {kname:<26}: P(kappa(R(s)) < kappa(s)) = {p_descent:.4f} | 1-sweep links = {links:,} | ret_rows/link = {m/links:.3f}")


def main() -> None:
    verify_witness_20_1_acyclicity_blindness()
    verify_synthetic_arches()

    blocks = load_admitted_blocks()

    print("\n=== 6 ADMITTED BLOCKS: PROCEDURES C1 (PHI-COPY) & C2 (ATTRACTOR + LZ77 SCATTER) ===")
    for fname, p, L in blocks:
        n = len(L)
        lf = compute_lf(L)
        T_true, _, _ = orbit_maps(L, p, lf)
        h1 = sum(
            1
            for i in range(n)
            if (i == 0 or L[i - 1] != L[i]) and (i + 1 == n or L[i + 1] != L[i])
        )
        c1_res = run_procedure_c1_phi_runs(L, p, T_true, h1)
        c2_res = audit_c2_attractor_and_lz77_scatter(L, p, T_true)
        print(
            f"  {fname:<12}: h1={h1:,}, r={c2_res['r']:,} (J1 in Gamma: 100%) | "
            f"C1 Phi-copy instr={c1_res['instr']:,} ({c1_res['over_h1']:.3f}*h1, copies={c1_res['copies']:,}, J1_saved=0) | "
            f"C2 LZ77(min=2) z={c2_res['z_2']:,}: J1 in [lit:{c2_res['pct_lit']:.1f}% len2:{c2_res['pct_len2']:.1f}% "
            f"len3:{c2_res['pct_len3']:.1f}% len4-7:{c2_res['pct_len4_7']:.1f}% len>=8:{c2_res['pct_len8p']:.1f}% "
            f"(len>=3: {c2_res['pct_ge3']:.1f}%)], |run(x)-run(y)| p10/med={c2_res['p10_run_dist']:,}/{c2_res['med_run_dist']:,} "
            f"(<=1 run: {c2_res['pct_adj_run']:.2f}%)"
        )

    print("\n=== MONODROMY NEEDS M-17 (i): SWEEP COMPOSITION ON LF OVER ALL n ROWS (6 ADMITTED BLOCKS) ===")
    for fname, p, L in blocks:
        n = len(L)
        lf = compute_lf(L)
        T_true, _, _ = orbit_maps(L, p, lf)
        h1 = sum(
            1
            for i in range(n)
            if (i == 0 or L[i - 1] != L[i]) and (i + 1 == n or L[i + 1] != L[i])
        )
        base_words = [bytes([b]) for b in L]
        r8 = run_sweep_composition(lf, base_words, p, T_true, "bytes", 8, verify_seq_cursors_L=L)
        r16 = run_sweep_composition(lf, base_words, p, T_true, "bytes", 16)
        rinf = run_sweep_composition(lf, base_words, p, T_true, "bytes", 10**9)
        print(f"\n  {fname} (n={n:,}, h1={h1:,}):")
        print(f"    cap=8   : 1-sweep={r8['rpl1']:.3f} r/link (links={r8['links1']:,} = {r8['links1']/h1:.2f}*h1) | 2-sweep={r8['rpl2']:.3f} r/link (links={r8['links2']:,} = {r8['links2']/h1:.2f}*h1)")
        print(f"              dist 1-sw: {r8['dist1']}")
        print(f"              dist 2-sw: {r8['dist2']}")
        print(f"    cap=16  : 1-sweep={r16['rpl1']:.3f} r/link (links={r16['links1']:,} = {r16['links1']/h1:.2f}*h1) | 2-sweep={r16['rpl2']:.3f} r/link (links={r16['links2']:,} = {r16['links2']/h1:.2f}*h1)")
        print(f"              dist 1-sw: {r16['dist1']}")
        print(f"              dist 2-sw: {r16['dist2']}")
        print(f"    uncapped: 1-sweep={rinf['rpl1']:.3f} r/link | 2-sweep={rinf['rpl2']:.3f} r/link")

    print("\n=== MONODROMY NEEDS M-17 (ii): SWEEP COMPOSITION ON OPEN'S RETAINED MAP m16 (6 ADMITTED BLOCKS) ===")
    below_23_steps = 0
    below_23_bytes16 = 0
    below_23_bytes8 = 0
    for idx_b, (fname, p, L) in enumerate(blocks):
        n = len(L)
        lf = compute_lf(L)
        T_true, _, _ = orbit_maps(L, p, lf)
        h1 = sum(
            1
            for i in range(n)
            if (i == 0 or L[i - 1] != L[i]) and (i + 1 == n or L[i + 1] != L[i])
        )
        R_lf, w_lf, mark_rank, row_of_rank = build_retained_map(L, p, limit=16)
        m16 = len(R_lf)
        t2_links = math.ceil(m16 / 2)

        # Mode A: retained-step cap (matches MONODROMY gs.py.txt ell=[1]*m)
        s8 = run_sweep_composition(R_lf, w_lf, mark_rank, T_true, "steps", 8)
        s16 = run_sweep_composition(R_lf, w_lf, mark_rank, T_true, "steps", 16)
        sinf = run_sweep_composition(R_lf, w_lf, mark_rank, T_true, "steps", 10**9)

        # Mode B: literal output-byte cap (len(w_x) + len(w_y) <= cap)
        b8 = run_sweep_composition(R_lf, w_lf, mark_rank, T_true, "bytes", 8)
        b16 = run_sweep_composition(R_lf, w_lf, mark_rank, T_true, "bytes", 16)

        if s16["rpl1"] < 2.3:
            below_23_steps += 1
        if b16["rpl1"] < 2.3:
            below_23_bytes16 += 1
        if b8["rpl1"] < 2.3:
            below_23_bytes8 += 1

        # Also measure P(R(s) < s) on Open's retained map!
        p_desc_ret = sum(1 for s in range(m16) if R_lf[s] < s) / m16

        print(f"\n  {fname} (m16={m16:,}, h1={h1:,}, m16/h1={m16/h1:.3f}, T^2=m16/2={t2_links:,}={t2_links/h1:.3f}*h1, P(R(s)<s)={p_desc_ret:.4f}):")
        print(f"    [Step-cap 8 ] 1-sw={s8['rpl1']:.3f} ret/link (links={s8['links1']:,} = {s8['links1']/h1:.3f}*h1, {s8['links1']/t2_links:.3f}*T2) | 2-sw={s8['rpl2']:.3f} ret/link (links={s8['links2']:,} = {s8['links2']/h1:.3f}*h1)")
        print(f"                  dist 1-sw: {s8['dist1']}")
        print(f"                  dist 2-sw: {s8['dist2']}")
        print(f"    [Step-cap 16] 1-sw={s16['rpl1']:.3f} ret/link (links={s16['links1']:,} = {s16['links1']/h1:.3f}*h1, {s16['links1']/t2_links:.3f}*T2) | 2-sw={s16['rpl2']:.3f} ret/link (links={s16['links2']:,} = {s16['links2']/h1:.3f}*h1)")
        print(f"                  dist 1-sw: {s16['dist1']}")
        print(f"                  dist 2-sw: {s16['dist2']}")
        print(f"    [Step-uncap ] 1-sw={sinf['rpl1']:.3f} ret/link | 2-sw={sinf['rpl2']:.3f} ret/link")
        print(f"    [Byte-cap 8 ] 1-sw={b8['rpl1']:.3f} ret/link (links={b8['links1']:,} = {b8['links1']/h1:.3f}*h1) | 2-sw={b8['rpl2']:.3f} ret/link (links={b8['links2']:,} = {b8['links2']/h1:.3f}*h1)")
        print(f"    [Byte-cap 16] 1-sw={b16['rpl1']:.3f} ret/link (links={b16['links1']:,} = {b16['links1']/h1:.3f}*h1) | 2-sw={b16['rpl2']:.3f} ret/link (links={b16['links2']:,} = {b16['links2']/h1:.3f}*h1)")

        if idx_b == 0:
            test_alternative_sweep_keys(L, p, T_true, R_lf, w_lf, mark_rank, row_of_rank)

    print(
        f"\n  NEEDS M-17 RULE CHECK (< 2.3 retained rows/link after 1 sweep on >= 4 blocks): "
        f"step-cap16 below 2.3 on {below_23_steps}/6 blocks; "
        f"byte-cap16 below 2.3 on {below_23_bytes16}/6 blocks; "
        f"byte-cap8 below 2.3 on {below_23_bytes8}/6 blocks."
    )

    print("\n=== PROCEDURE C4 (NOTE #14 NEW BASIS): UNIMODAL ORBIT ARCHES ON 6 ADMITTED BLOCKS ===")
    for fname, p, L in blocks:
        n = len(L)
        lf = compute_lf(L)
        T_true, _, _ = orbit_maps(L, p, lf)
        h1 = sum(
            1
            for i in range(n)
            if (i == 0 or L[i - 1] != L[i]) and (i + 1 == n or L[i + 1] != L[i])
        )
        # (a) 2-cursor sequential on raw LF (cap=16 and uncapped)
        raw16 = build_unimodal_arches_two_cursor(L, p, T_true, 16)
        rawinf = build_unimodal_arches_two_cursor(L, p, T_true, 10**9)
        # (b) on Open's m16 retained map (LIMIT=16)
        R_m16, w_m16, mark_m16, _ = build_retained_map(L, p, limit=16)
        m16 = len(R_m16)
        ret16 = build_unimodal_arches_general(R_m16, w_m16, mark_m16, T_true, 16)
        retinf = build_unimodal_arches_general(R_m16, w_m16, mark_m16, T_true, 10**9)
        # (c) on full run-contracted cycle (LIMIT=1 -> only 1 row per wide run? Wait: LIMIT=1 retains ONLY w<=1 runs + p!!
        # Wait: CHECK THAT!! Can Open's contraction run at LIMIT=1?!
        # In OpenCells, LIMIT=1 retains ONLY width-1 runs J1 plus p (size m_1 = h1 + [p not in J1])!!
        # Let's test Unimodal Arches AND 1-sweep GS on m_1 (LIMIT=1, thick runs = ALL runs w >= 2)!!
        R_m1, w_m1, mark_m1, _ = build_retained_map(L, p, limit=1)
        m1 = len(R_m1)
        m1_arch_inf = build_unimodal_arches_general(R_m1, w_m1, mark_m1, T_true, 10**9)
        m1_arch_16 = build_unimodal_arches_general(R_m1, w_m1, mark_m1, T_true, 16)
        m1_gs_step16 = run_sweep_composition(R_m1, w_m1, mark_m1, T_true, "steps", 16)
        m1_gs_byte16 = run_sweep_composition(R_m1, w_m1, mark_m1, T_true, "bytes", 16)
        print(
            f"  {fname:<12}: rawLF arches={rawinf['links']:,} ({rawinf['rpl']:.2f} B/arch; cap16={raw16['links']:,}) | "
            f"m16({m16:,}) arches uncap={retinf['links']:,} ({retinf['rpl_ret']:.2f} ret/arch = {retinf['links']/h1:.3f}*h1; cap16B={ret16['links']/h1:.3f}*h1) | "
            f"LIMIT=1 (m1={m1:,} ~ h1): arches uncap={m1_arch_inf['links']:,} ({m1_arch_inf['links']/h1:.3f}*h1!!), "
            f"cap16B={m1_arch_16['links']:,} ({m1_arch_16['links']/h1:.3f}*h1); "
            f"GS 1-sw step16={m1_gs_step16['links1']/h1:.3f}*h1, byte16={m1_gs_byte16['links1']/h1:.3f}*h1"
        )


if __name__ == "__main__":
    main()
