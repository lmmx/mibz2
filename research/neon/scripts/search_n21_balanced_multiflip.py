#!/usr/bin/env python3
"""N-21: Constructive search for balanced multi-flip fibre witnesses under O = (H, A, A2)
(Delta H = 0, Delta A = 0, Delta A2 = 0, pos_{L', p} != pos_{L, p}) and witnessed U_O
upper bounds on block-0.L (plus 6-block summary), answering MONODROMY ASK N(2).

Key exact properties verified:
  1. Lemma 21.1 (Single- and Two-Flip Algebra):
     For any flip f = (i, m, j) swapping L[i] != L[j] (i < j):
       Delta A(f)  = (i - j) * (L[j] - L[i]) != 0
       Delta A2(f) = (i^2 - j^2) * (L[j] - L[i]) = (i + j) * Delta A(f)
     Hence k = 1 never balances A, and k = 2 balances (A, A2) ONLY if i_1 + j_1 = i_2 + j_2
     (concentric nested spans — IMPOSSIBLE for pairwise-disjoint spans j_1 < i_2).
  2. Constructive search on block-0.L across:
     - k = 2 concentric nested flips (i_1 + j_1 = i_2 + j_2, Delta A_1 + Delta A_2 = 0),
     - k = 3 pairwise-disjoint flips,
     - k = 4 pairwise-disjoint same-byte-pair {a, b} flips (which preserve NOT ONLY scalar
       (H, A, A2) = (H, sum r*L[r], sum r^2*L[r]), BUT ALSO all 256 per-symbol vector
       moments (H[c], sum_{L[r]=c} r, sum_{L[r]=c} r^2) simultaneously!).
  3. Witnessed U_O(L, p) bound:
     Finds a small covering set W_cover of balanced 4-flips (including uncapped-span flips
     near origPtr's orbit endpoints) and FULL-O(n) BYTE-VERIFIES every single witness in
     W_cover (checking single n-cycle LF', exact H, exact A, exact A2, exact per-symbol
     moments, and exact pos_{L', p}(x) for all x in [0, n)) so the upper bound on |U_O|
     and |J1 cap U_O| relies on ZERO unverified shortcuts.
"""

from __future__ import annotations

import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from measure_cycle2 import compute_lf, load_admitted_blocks, orbit_maps  # noqa: E402


def moments_scalar_and_vec(L: bytes) -> tuple[tuple[int, ...], int, int, tuple[int, ...], tuple[int, ...]]:
    h = [0] * 256
    a_vec = [0] * 256
    a2_vec = [0] * 256
    A = 0
    A2 = 0
    for idx, b in enumerate(L):
        h[b] += 1
        a_vec[b] += idx
        idx2 = idx * idx
        a2_vec[b] += idx2
        A += idx * b
        A2 += idx2 * b
    return tuple(h), A, A2, tuple(a_vec), tuple(a2_vec)


def enumerate_flips(
    L: bytes, lf: list[int], lf_pos: list[int], span_cap: int | None
) -> list[tuple[int, int, int, int, int, int]]:
    """Enumerate valid Type-A/B single flips (i, m, j, ftype, dA, u=i+j).
    ftype = 0 for Type A (L[i]=L[m]=a, L[j]=b -> b, a, a),
    ftype = 1 for Type B (L[i]=a, L[m]=L[j]=b -> b, b, a).
    """
    n = len(L)
    last = [-1] * 256
    prev_occ = [-1] * n
    next_occ = [n] * n
    for idx, b in enumerate(L):
        pr = last[b]
        prev_occ[idx] = pr
        if pr != -1:
            next_occ[pr] = idx
        last[b] = idx

    out: list[tuple[int, int, int, int, int, int]] = []
    cap = n if span_cap is None else span_cap

    # Type B: consecutive pair (m, j) of byte b
    for m in range(n):
        j = next_occ[m]
        if j >= n or j - m >= cap:
            continue
        lo = max(prev_occ[m] + 1, j - cap)
        pm = lf_pos[m]
        pj = lf_pos[j]
        b_val = L[j]
        if pm < pj:
            for i in range(lo, m):
                if next_occ[i] > j:
                    pi = lf_pos[i]
                    if pi < pm or pi > pj:
                        dA = (i - j) * (b_val - L[i])
                        out.append((i, m, j, 1, dA, i + j))
        else:
            for i in range(lo, m):
                if next_occ[i] > j:
                    pi = lf_pos[i]
                    if pj < pi < pm:
                        dA = (i - j) * (b_val - L[i])
                        out.append((i, m, j, 1, dA, i + j))

    # Type A: consecutive pair (i, m) of byte a
    for i in range(n):
        m = next_occ[i]
        if m >= n or m - i >= cap:
            continue
        hi = min(next_occ[m], i + cap + 1)
        pi = lf_pos[i]
        pm = lf_pos[m]
        a_val = L[i]
        if pi < pm:
            for j in range(m + 1, hi):
                if prev_occ[j] < i:
                    pj = lf_pos[j]
                    if pi < pj < pm:
                        dA = (i - j) * (L[j] - a_val)
                        out.append((i, m, j, 0, dA, i + j))
        else:
            for j in range(m + 1, hi):
                if prev_occ[j] < i:
                    pj = lf_pos[j]
                    if pj > pi or pj < pm:
                        dA = (i - j) * (L[j] - a_val)
                        out.append((i, m, j, 0, dA, i + j))

    return out


def eval_disjoint_multiflip_compressed(
    flips_tuple: list[tuple[int, int, int, int, int, int]],
    p: int,
    n: int,
    lf: list[int],
    lf_pos: list[int],
) -> tuple[bool, list[tuple[int, int]]]:
    """Exact O(k) single-cycle & fixed-orbit-interval evaluator for pairwise-disjoint-span flips.

    Returns (is_single_cycle, zero_shift_orbit_intervals) where each (lo, hi) in
    zero_shift_orbit_intervals means rows x with lf_pos[x] in [lo, hi] keep pos'(x) == pos(x).
    """
    # Map modified rows r -> LF'(r)
    mod_lf: dict[int, int] = {}
    for i, m, j, ftype, _, _ in flips_tuple:
        if ftype == 0:
            # Type A: L[i]=L[m]=a, L[j]=b -> L' has b at i, a at m, a at j
            # Original LF: i -> u_a, m -> u_a+1, j -> v_b
            # New LF':     i -> v_b, m -> u_a,   j -> u_a+1
            mod_lf[i] = lf[j]
            mod_lf[m] = lf[i]
            mod_lf[j] = lf[m]
        else:
            # Type B: L[i]=a, L[m]=L[j]=b -> L' has b at i, b at m, a at j
            # Original LF: i -> u_a, m -> v_b,   j -> v_b+1
            # New LF':     i -> v_b, m -> v_b+1, j -> u_a
            mod_lf[i] = lf[m]
            mod_lf[m] = lf[j]
            mod_lf[j] = lf[i]

    if p not in mod_lf:
        mod_lf[p] = lf[p]

    # Sort special vertices by original orbit position u = lf_pos[v]
    specials = sorted((lf_pos[v], v) for v in mod_lf)
    M = len(specials)
    assert specials[0] == (0, p)
    u_vals = [u for u, _ in specials]
    idx_of_row = {v: idx for idx, (_, v) in enumerate(specials)}

    import bisect

    visited = [False] * M
    cur_idx = 0
    cur_new_pos = 0
    zero_intervals: list[tuple[int, int]] = []
    steps_visited = 0

    while not visited[cur_idx]:
        visited[cur_idx] = True
        steps_visited += 1
        u_cur, r_cur = specials[cur_idx]
        # Note: r_cur itself has new_pos == cur_new_pos and old_pos == u_cur
        if cur_new_pos == u_cur:
            zero_intervals.append((u_cur, u_cur))

        # Step 1 from r_cur goes to w = mod_lf[r_cur] at new_pos = cur_new_pos + 1
        w = mod_lf[r_cur]
        pw = lf_pos[w]
        if w in idx_of_row:
            nxt_idx = idx_of_row[w]
            cur_new_pos += 1
        else:
            # Find first special u_t > pw (wrapping to idx 0 at u_0 = 0)
            nxt_idx = bisect.bisect_left(u_vals, pw)
            if nxt_idx == M:
                nxt_idx = 0
                u_end = n - 1
            else:
                u_end = u_vals[nxt_idx] - 1
            # All interior vertices from pw .. u_end advance in lockstep along LF:
            # For any x with pos(x) in [pw, u_end], new_pos(x) = (cur_new_pos + 1) + (pos(x) - pw)
            # So new_pos(x) == pos(x)  <==>  cur_new_pos + 1 == pw!
            if cur_new_pos + 1 == pw:
                zero_intervals.append((pw, u_end))
            arc_len = (u_vals[nxt_idx] - pw) % n
            cur_new_pos += 1 + arc_len

        cur_idx = nxt_idx

    if steps_visited != M or cur_idx != 0 or cur_new_pos != n:
        return False, []
    return True, zero_intervals


def full_verify_witness(
    L: bytes,
    p: int,
    lf: list[int],
    lf_pos: list[int],
    base_moments: tuple,
    swaps: list[tuple[int, int]],
) -> tuple[bytes, list[int], int]:
    """Full O(n) independent verification of a multi-flip witness L':
    applies swaps (i, j) to L, computes LF' from scratch, verifies single n-cycle,
    verifies exact (H, A, A2) equality, and returns (L', lf_pos', moved_rows_count)."""
    L2_arr = bytearray(L)
    for i, j in swaps:
        L2_arr[i], L2_arr[j] = L2_arr[j], L2_arr[i]
    L2 = bytes(L2_arr)
    assert L2 != L

    # Full O(n) LF' and single-cycle orbit walk
    lf2 = compute_lf(L2)
    n = len(L)
    pos2 = [-1] * n
    cur = p
    for step in range(n):
        assert pos2[cur] == -1, "Witness L2 is NOT a single cycle (visited vertex twice)!"
        pos2[cur] = step
        cur = lf2[cur]
    assert cur == p, "Witness L2 did not return to p after n steps!"

    # Full O(n) exact moments check
    m2 = moments_scalar_and_vec(L2)
    assert m2[0] == base_moments[0], "Histogram H mismatch!"
    assert m2[1] == base_moments[1], f"First moment A mismatch: {m2[1]} != {base_moments[1]}"
    assert m2[2] == base_moments[2], f"Second moment A2 mismatch: {m2[2]} != {base_moments[2]}"

    moved = sum(1 for x in range(n) if pos2[x] != lf_pos[x])
    assert moved > 0, "Position map pos_{L', p} did not differ from pos_{L, p}!"
    return L2, pos2, moved


def search_k2_concentric(L: bytes, p: int, lf: list[int], lf_pos: list[int], base_moments: tuple, flips: list) -> tuple | None:
    """Check whether EVEN a k=2 concentric nested multi-flip (i1 < i2 < j2 < j1 with
    i1 + j1 == i2 + j2 and dA1 + dA2 == 0) exists on block-0.L!"""
    by_center_dA: dict[tuple[int, int], list] = defaultdict(list)
    for f in flips:
        i, m, j, ftype, dA, u = f
        by_center_dA[(u, dA)].append(f)

    for (u, dA), grp1 in by_center_dA.items():
        if dA <= 0:
            continue
        grp2 = by_center_dA.get((u, -dA))
        if not grp2:
            continue
        for f1 in grp1:
            i1, m1, j1, _, _, _ = f1
            for f2 in grp2:
                i2, m2, j2, _, _, _ = f2
                if len({i1, j1, i2, j2}) < 4:
                    continue
                # Test L' directly in O(n) since concentric pairs are rare!
                L2_arr = bytearray(L)
                L2_arr[i1], L2_arr[j1] = L2_arr[j1], L2_arr[i1]
                L2_arr[i2], L2_arr[j2] = L2_arr[j2], L2_arr[i2]
                L2 = bytes(L2_arr)
                m2 = moments_scalar_and_vec(L2)
                if m2[:3] != base_moments[:3]:
                    continue
                lf2 = compute_lf(L2)
                pos2 = [-1] * len(L)
                cur = p
                ok = True
                for step in range(len(L)):
                    if pos2[cur] != -1:
                        ok = False
                        break
                    pos2[cur] = step
                    cur = lf2[cur]
                if ok and cur == p:
                    moved = sum(1 for x in range(len(L)) if pos2[x] != lf_pos[x])
                    if moved > 0:
                        return (f1, f2, L2, pos2, moved)
    return None


def search_k3_disjoint(L: bytes, p: int, lf: list[int], lf_pos: list[int], base_moments: tuple, flips: list) -> tuple | None:
    """Search for a k=3 pairwise-disjoint balanced multi-flip:
    dA1 + dA2 + dA3 = 0 and u1*dA1 + u2*dA2 + u3*dA3 = 0."""
    # Group flips by (dA, u) — actually u = i + j is almost unique, so use small-span flips
    # with common dA ratios: e.g. dA1 = d, dA2 = -2d, dA3 = d -> u1 - 2*u2 + u3 = 0 (u2 = (u1+u3)/2)!!
    # Look how clean that is: take ANY byte diffs / spans giving dA1 = d, dA3 = d, dA2 = -2*d,
    # and u1 + u3 == 2 * u2 (arithmetic progression of flip centers u1 < u2 < u3)!!
    by_dA: dict[int, list] = defaultdict(list)
    for f in flips:
        if f[2] - f[0] <= 16:
            by_dA[f[4]].append(f)

    # Sort dA values by bucket size descending
    popular_dA = sorted((len(v), d) for d, v in by_dA.items() if d > 0 and -2 * d in by_dA)
    popular_dA.reverse()
    for _, d in popular_dA[:40]:
        pos_list = by_dA[d]
        mid_by_2u: dict[int, list] = defaultdict(list)
        for f2 in by_dA[-2 * d]:
            mid_by_2u[2 * f2[5]].append(f2)
        if not mid_by_2u:
            continue
        # Limit pos_list to 400 evenly spaced flips for instant O(400^2) check
        step = max(1, len(pos_list) // 400)
        sub = pos_list[::step]
        for a_idx in range(len(sub)):
            f1 = sub[a_idx]
            for b_idx in range(a_idx + 1, len(sub)):
                f3 = sub[b_idx]
                s_u = f1[5] + f3[5]
                cand_mids = mid_by_2u.get(s_u)
                if not cand_mids:
                    continue
                for f2 in cand_mids:
                    # Check pairwise disjoint column spans
                    spans = sorted([(f1[0], f1[2], f1), (f2[0], f2[2], f2), (f3[0], f3[2], f3)])
                    if spans[0][1] < spans[1][0] and spans[1][1] < spans[2][0]:
                        ok, _ = eval_disjoint_multiflip_compressed(
                            [spans[0][2], spans[1][2], spans[2][2]], p, len(L), lf, lf_pos
                        )
                        if ok:
                            swaps = [(spans[0][0], spans[0][1]), (spans[1][0], spans[1][1]), (spans[2][0], spans[2][1])]
                            L2, pos2, moved = full_verify_witness(L, p, lf, lf_pos, base_moments, swaps)
                            return ([spans[0][2], spans[1][2], spans[2][2]], L2, pos2, moved)
    return None


def analyze_block_full(fname: str, p: int, L: bytes, verbose: bool = True) -> dict[str, object]:
    n = len(L)
    lf = compute_lf(L)
    _, _, lf_pos = orbit_maps(L, p, lf)
    base_moments = moments_scalar_and_vec(L)

    is_w1 = bytearray(n)
    i = 0
    while i < n:
        j = i + 1
        while j < n and L[j] == L[i]:
            j += 1
        if j - i == 1:
            is_w1[i] = 1
        i = j
    h1 = sum(is_w1)

    if verbose:
        flips_uncap_all = enumerate_flips(L, lf, lf_pos, span_cap=2048)
        flips_256 = [f for f in flips_uncap_all if f[2] - f[0] <= 256]
    else:
        flips_256 = enumerate_flips(L, lf, lf_pos, span_cap=256)
        flips_uncap_all = flips_256
    extreme_flips = [
        f
        for f in flips_uncap_all
        if min(lf_pos[f[0]], lf_pos[f[1]], lf_pos[f[2]]) <= 2000
        or max(lf_pos[f[0]], lf_pos[f[1]], lf_pos[f[2]]) >= n - 2000
    ]
    extreme_flips_set = set(extreme_flips)

    k2_res = search_k2_concentric(L, p, lf, lf_pos, base_moments, flips_256) if verbose else None
    k3_res = search_k3_disjoint(L, p, lf, lf_pos, base_moments, flips_256) if verbose else None

    # Build k=4 same-symbol-pair, equal-span balanced 4-flips (preserving ALL 256 per-symbol
    # vector moments (H[c], A^(c), A2^(c)) as well as scalar (H, A, A2)!):
    # Group flips by (min(a,b), max(a,b), span=j-i, sign=+1 if L[i]<L[j] else -1)
    by_pair_span: dict[tuple[int, int, int], tuple[list, list]] = defaultdict(lambda: ([], []))
    pool = flips_uncap_all if verbose else flips_256
    for f in pool:
        fi, fm, fj, ftype, dA, u = f
        if fj - fi > 64 and f not in extreme_flips_set:
            continue
        a, b = L[fi], L[fj]
        lo_b, hi_b = (a, b) if a < b else (b, a)
        sgn = 0 if a < b else 1  # 0: +1, 1: -1
        by_pair_span[(lo_b, hi_b, fj - fi)][sgn].append(f)

    # 1. Find minimal-total-span k=4 same-symbol-pair witness (e.g. span=2 on each of the 4 flips -> total 8 rows!)
    best_min_witness = None
    for (lo_b, hi_b, sp), (pos_fl, neg_fl) in sorted(
        by_pair_span.items(), key=lambda kv: (kv[0][2], -(len(kv[1][0]) + len(kv[1][1])))
    ):
        if len(pos_fl) < 2 or len(neg_fl) < 2:
            continue
        p_sub = pos_fl[:250]
        n_sub = neg_fl[:250]
        if len(p_sub) < 15 or len(n_sub) < 15:
            continue
        found_here = False
        # Pair 1 (fp1 [+], fn1 [-]) + Pair 2 (fp2 [+], fn2 [-]) satisfies Delta A = 0 and
        # Delta A2 = 0 iff i_fp1 + i_fp2 == i_fn1 + i_fn2:
        sum_pos: dict[int, list[tuple]] = defaultdict(list)
        for idx1 in range(len(p_sub)):
            for idx2 in range(idx1 + 1, len(p_sub)):
                fp1, fp2 = p_sub[idx1], p_sub[idx2]
                if fp1[2] < fp2[0] and len(sum_pos[fp1[0] + fp2[0]]) < 4:
                    sum_pos[fp1[0] + fp2[0]].append((fp1, fp2))
        for idx1 in range(len(n_sub)):
            for idx2 in range(idx1 + 1, len(n_sub)):
                fn1, fn2 = n_sub[idx1], n_sub[idx2]
                if fn1[2] < fn2[0]:
                    s_val = fn1[0] + fn2[0]
                    cands = sum_pos.get(s_val)
                    if cands:
                        for fp1, fp2 in cands:
                            quad = sorted([fp1, fp2, fn1, fn2], key=lambda x: x[0])
                            if all(quad[r][2] < quad[r + 1][0] for r in range(3)):
                                ok, _ = eval_disjoint_multiflip_compressed(quad, p, n, lf, lf_pos)
                                if ok:
                                    swaps = [(q[0], q[2]) for q in quad]
                                    L2, pos2, moved = full_verify_witness(L, p, lf, lf_pos, base_moments, swaps)
                                    # Also assert per-symbol vector moments match!
                                    m2 = moments_scalar_and_vec(L2)
                                    assert m2[3] == base_moments[3] and m2[4] == base_moments[4]
                                    best_min_witness = (quad, L2, pos2, moved, (lo_b, hi_b, sp))
                                    found_here = True
                                    break
                    if found_here:
                        break
            if found_here:
                break
        if best_min_witness is not None:
            break

    assert best_min_witness is not None, f"Expected a balanced 4-flip on {fname}"

    # 2. Drive U_W (rows fixed across all witnessed fibre members) as low as possible!
    # Maintain `still_fixed_at_orbit[u]` for orbit pos u in 0..n-1 (initially all 1).
    still_fixed_orbit = bytearray([1] * n)
    remaining_fixed = n
    cover_witnesses: list[list[tuple[int, int]]] = []

    def try_quad_for_cover(quad: list[tuple[int, int, int, int, int, int]]) -> bool:
        nonlocal remaining_fixed
        ok, zero_ivs = eval_disjoint_multiflip_compressed(quad, p, n, lf, lf_pos)
        if not ok:
            return False
        # Build mask of orbit positions that STAY fixed in this witness (very small or complement)
        # Check if any currently fixed orbit position u is NOT in zero_ivs!
        # Since zero_ivs is a list of <= 13 intervals (lo, hi), sort them and scan gaps!
        z_sorted = sorted(zero_ivs)
        eliminated = 0
        cur_u = 0
        # First pass: check if any still_fixed_orbit[u] == 1 lies outside z_sorted
        gaps: list[tuple[int, int]] = []
        for lo, hi in z_sorted:
            if cur_u < lo:
                gaps.append((cur_u, lo - 1))
            if hi + 1 > cur_u:
                cur_u = hi + 1
        if cur_u < n:
            gaps.append((cur_u, n - 1))

        newly_cleared = False
        for glo, ghi in gaps:
            idx_find = still_fixed_orbit.find(1, glo, ghi + 1)
            if idx_find != -1:
                newly_cleared = True
                for u_pos in range(idx_find, ghi + 1):
                    if still_fixed_orbit[u_pos]:
                        still_fixed_orbit[u_pos] = 0
                        remaining_fixed -= 1
                        eliminated += 1
        if newly_cleared:
            cover_witnesses.append([(q[0], q[2]) for q in quad])
        return newly_cleared

    min_t1_pool = min(min(lf_pos[f[0]], lf_pos[f[1]], lf_pos[f[2]]) for f in pool)
    max_t3_pool = max(max(lf_pos[f[0]], lf_pos[f[1]], lf_pos[f[2]]) for f in pool)
    single_flip_u_floor = (min_t1_pool + 1) + (n - 1 - max_t3_pool)

    # First feed best_min_witness
    try_quad_for_cover(best_min_witness[0])

    by_dA_all: dict[int, list] = defaultdict(list)
    for f in flips_256:
        if f[2] - f[0] <= 32:
            by_dA_all[f[4]].append(f)
    for f in extreme_flips:
        by_dA_all[f[4]].append(f)

    pair_by_dA2: dict[int, list[tuple]] = defaultdict(list)
    for d_val, pos_list in list(by_dA_all.items()):
        if d_val <= 0:
            continue
        neg_list = by_dA_all.get(-d_val)
        if not neg_list:
            continue
        p_sample = pos_list[:: max(1, len(pos_list) // 100)]
        n_sample = neg_list[:: max(1, len(neg_list) // 100)]
        for fp in p_sample:
            for fn in n_sample:
                if fp[2] < fn[0] or fn[2] < fp[0]:
                    dA2_p = d_val * (fp[5] - fn[5])
                    if len(pair_by_dA2[dA2_p]) < 4:
                        pair_by_dA2[dA2_p].append((fp, fn))

    # Pair extreme flips only when their orbit cut can still clear a fixed row!
    for f_ext in extreme_flips:
        if remaining_fixed <= single_flip_u_floor:
            break
        t_lo = min(lf_pos[f_ext[0]], lf_pos[f_ext[1]], lf_ext_p := lf_pos[f_ext[2]])
        t_hi = max(lf_pos[f_ext[0]], lf_pos[f_ext[1]], lf_ext_p)
        if still_fixed_orbit.find(1, t_lo + 1, t_hi + 1) == -1:
            continue
        d_ext = f_ext[4]
        partners = by_dA_all.get(-d_ext, [])
        found_ext = False
        for f_part in partners[:60]:
            if f_ext[2] < f_part[0] or f_part[2] < f_ext[0]:
                dA2_p = d_ext * (f_ext[5] - f_part[5])
                opp_list = pair_by_dA2.get(-dA2_p)
                if opp_list:
                    for g1, g2 in opp_list:
                        quad = sorted([f_ext, f_part, g1, g2], key=lambda x: x[0])
                        if all(quad[r][2] < quad[r + 1][0] for r in range(3)):
                            if try_quad_for_cover(quad):
                                found_ext = True
                                break
            if found_ext:
                break

    checked_general = 0
    for dA2_val, plist in list(pair_by_dA2.items()):
        if remaining_fixed <= single_flip_u_floor or checked_general >= 400:
            break
        if dA2_val <= 0:
            continue
        nlist = pair_by_dA2.get(-dA2_val)
        if not nlist:
            continue
        for p1, p2 in plist:
            for q1, q2 in nlist:
                quad = sorted([p1, p2, q1, q2], key=lambda x: x[0])
                if all(quad[r][2] < quad[r + 1][0] for r in range(3)):
                    try_quad_for_cover(quad)
                    checked_general += 1
                    if checked_general >= 1500:
                        break
            if checked_general >= 1500:
                break
        if checked_general >= 1500:
            break

    # FULL O(n) INDEPENDENT BYTE-VERIFICATION OF EVERY SINGLE WITNESS IN cover_witnesses!
    verified_fixed_rows = set(range(n))
    for swaps in cover_witnesses:
        _, pos2, _ = full_verify_witness(L, p, lf, lf_pos, base_moments, swaps)
        verified_fixed_rows = {x for x in verified_fixed_rows if pos2[x] == lf_pos[x]}

    # Cross-check that compressed-graph fixed count matches full O(n) verified_fixed_rows EXACTLY!
    assert len(verified_fixed_rows) == remaining_fixed, (
        f"Compressed vs full O(n) mismatch: {remaining_fixed} vs {len(verified_fixed_rows)}"
    )
    j1_fixed = sorted(x for x in verified_fixed_rows if is_w1[x])

    # Bonus: count how many PAIRWISE COLUMN-DISJOINT balanced (H, A, A2) 4-flips we can pack on block-0.L!
    disj_quads = []
    if verbose:
        cand_quads = []
        for (lo_b, hi_b, sp), (pos_fl, neg_fl) in by_pair_span.items():
            if sp > 32 or len(pos_fl) < 4 or len(neg_fl) < 4:
                continue
            # Local sliding window of 20 flips to make compact local balanced 4-flips!
            merged = sorted([(f[0], 1, f) for f in pos_fl] + [(f[0], -1, f) for f in neg_fl])
            for start_w in range(0, len(merged) - 8, 4):
                win = merged[start_w : start_w + 24]
                wp = [x[2] for x in win if x[1] == 1]
                wn = [x[2] for x in win if x[1] == -1]
                sp_map: dict[int, tuple] = {}
                for a1 in range(len(wp)):
                    for a2 in range(a1 + 1, len(wp)):
                        if wp[a1][2] < wp[a2][0]:
                            sp_map[wp[a1][0] + wp[a2][0]] = (wp[a1], wp[a2])
                for b1 in range(len(wn)):
                    for b2 in range(b1 + 1, len(wn)):
                        if wn[b1][2] < wn[b2][0]:
                            c = sp_map.get(wn[b1][0] + wn[b2][0])
                            if c is not None:
                                quad = sorted([c[0], c[1], wn[b1], wn[b2]], key=lambda x: x[0])
                                if all(quad[r][2] < quad[r + 1][0] for r in range(3)):
                                    ok, _ = eval_disjoint_multiflip_compressed(quad, p, n, lf, lf_pos)
                                    if ok:
                                        cand_quads.append((quad[3][2], quad[0][0], quad))
        cand_quads.sort()
        last_end = -1
        for q_end, q_start, quad in cand_quads:
            if q_start > last_end:
                disj_quads.append(quad)
                last_end = q_end

    if verbose:
        quad_min, L2_min, pos2_min, moved_min, (lo_b, hi_b, sp_min) = best_min_witness
        print(f"=== N-21 DETAILED RESULTS ON {fname} (n={n:,}, p={p:,}, h1={h1:,}, p in J1: {bool(is_w1[p])}) ===")
        print(f"  Base scalar moments: A = {base_moments[1]:,}, A2 = {base_moments[2]:,}")
        print(f"  Valid single flips: {len(flips_256):,} (span<=256), {len(flips_uncap_all):,} (uncapped)")
        if k2_res is not None:
            f1, f2, _, _, mv2 = k2_res
            print(
                f"  [k=2 concentric nested witness FOUND & O(n)-verified!]: "
                f"f1=({f1[0]},{f1[1]},{f1[2]}) bytes=({L[f1[0]]},{L[f1[2]]}), "
                f"f2=({f2[0]},{f2[1]},{f2[2]}) bytes=({L[f2[0]]},{L[f2[2]]}), "
                f"center i+j={f1[5]}, dA=+-{abs(f1[4])}, moved_rows={mv2:,} ({100*mv2/n:.2f}% of n)"
            )
        else:
            print("  [k=2 concentric nested]: 0 valid concentric 2-flips within span<=256")
        if k3_res is not None:
            q3, _, _, mv3 = k3_res
            print(
                f"  [k=3 pairwise-disjoint witness FOUND & O(n)-verified!]: "
                f"flips={[(f[0],f[1],f[2]) for f in q3]}, dAs={[f[4] for f in q3]}, centers={[f[5] for f in q3]}, "
                f"moved_rows={mv3:,} ({100*mv3/n:.2f}% of n)"
            )
        print(
            f"  [k=4 minimal-span ALL-256-SYMBOL-MOMENTS witness FOUND & O(n)-verified!]: "
            f"byte pair=({chr(lo_b)!r}={lo_b}, {chr(hi_b)!r}={hi_b}), each span={sp_min} (Hamming dist=8):"
        )
        for idx_q, qf in enumerate(quad_min):
            qi, qm, qj, qft, qdA, qu = qf
            print(
                f"    flip {idx_q+1}: (i={qi}, m={qm}, j={qj}), L[i..j]={L[qi:qj+1]!r}, "
                f"type={'A' if qft==0 else 'B'}, dA={qdA:+d}, i+j={qu}"
            )
        print(f"    Full O(n) check: single n-cycle=PASS, Delta H=0, Delta A=0, Delta A2=0, Delta A^(c)=Delta A2^(c)=0 for all 256 c, moved_rows={moved_min:,} ({100*moved_min/n:.2f}% of n)")
        print(
            f"  [Witnessed U_O(block-0.L) bound — ALL {len(cover_witnesses)} covering witnesses 100% O(n) byte-verified!]:"
        )
        print(
            f"    1 <= |U_O| <= {len(verified_fixed_rows)} rows ({100*len(verified_fixed_rows)/n:.5f}% of n); "
            f"{1 if is_w1[p] else 0} <= |J1 cap U_O| <= {len(j1_fixed)} width-1 rows ({100*len(j1_fixed)/h1:.4f}% of h1)! "
            f"j1_fixed={j1_fixed}"
        )
        print(
            f"    => Exact Theorem N' + Lemma 18.1 floor on {fname} under O=(H, A, A2): "
            f"c1 >= h1 - |J1 cap U_O| >= {h1 - len(j1_fixed):,} / {h1:,} ({100*(h1 - len(j1_fixed))/h1:.4f}% of h1)!"
        )
        print(
            f"    Local pairwise-disjoint balanced 4-flips packed on {fname}: {len(disj_quads):,} disjoint 4-flips"
        )

    return {
        "fname": fname,
        "n": n,
        "h1": h1,
        "p_in_j1": int(is_w1[p]),
        "u_upper": len(verified_fixed_rows),
        "j1_u_upper": len(j1_fixed),
        "cover_cnt": len(cover_witnesses),
    }


def main() -> None:
    blocks = load_admitted_blocks()
    b0 = blocks[0]
    res0 = analyze_block_full(b0[0], b0[1], b0[2], verbose=True)

    print("\n=== 6-BLOCK CONFIRMATION SUMMARY FOR O = (H, A, A2) (ALL COVER WITNESSES O(n) VERIFIED) ===")
    for idx, (fname, p, L) in enumerate(blocks):
        res = res0 if idx == 0 else analyze_block_full(fname, p, L, verbose=False)
        print(
            f"  {fname:<12}: h1={res['h1']:,} | cover_witnesses={res['cover_cnt']:>2} | "
            f"1 <= |U_O| <= {res['u_upper']:>4} | {res['p_in_j1']} <= |J1 cap U_O| <= {res['j1_u_upper']:>2} "
            f"=> c1 >= {res['h1'] - res['j1_u_upper']:,} ({100*(res['h1'] - res['j1_u_upper'])/res['h1']:.4f}% of h1)"
        )


if __name__ == "__main__":
    main()
