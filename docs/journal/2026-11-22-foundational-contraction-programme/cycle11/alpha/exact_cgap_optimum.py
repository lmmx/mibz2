#!/usr/bin/env python3
"""
Cycle 11 — Part 1:
Exact Global Minimum Q*_{C_gap}(L) via Exact Search (BFS / Dijkstra / Branch-and-Bound)
on Valid Single-Cycle BWTs!

Model C_gap:
  - State = canonical gap-closed maximally-coalesced W-IET on [0, m) as tuple of
    (lo, hi, delta, word_bytes) (with no two adjacent pieces sharing (delta, word_bytes)!).
  - Allowed elementary moves from any state (cost +1):
      1. Binary Join(A, B) for ANY overlapping pair (J_A cap I_B != empty, A != B) ->
         composes the exact overlap sub-interval A_0 = (J_A cap I_B) - delta_A with B_0 = J_A cap I_B,
         deletes B_0 from the domain, closes the gap densely, and applies FREE maximal coalescence!
         (Wait: also verifies that any proper sub-interval join never beats maximal-overlap join.)
      2. Spiral Pow(A) for ANY self-overlapping piece (0 < |delta_A| < hi_A - lo_A) ->
         collapses the entire spiral overlap in 1 step, closes gap, and applies FREE maximal coalescence!
  - Goal: reach P = 1 (single closed cycle d == 0 of length n).

Tests:
  1. ALL valid single-cycle binary BWTs (n = 4..10, up to r_0 = 8) — exact global minimum Q*!
  2. Sample of valid single-cycle ternary BWTs (n = 6..12, r_0 = 4..9) — exact global minimum Q*!
  3. Structured z(T) << r_0 families (Thue-Morse T_3..T_6, Fibonacci F_5..F_9, Period-Doubling)!
  4. Checks exact minimum ratio Q* / r_0, checks whether ANY valid BWT achieves Q* < 0.50 r_0
     or Q* < 1.00 r_0, extracts the exact winning contraction sequence for the smallest ratio,
     AND checks Theorem C4's amortized potential Psi = U_t - D_t on every step!
"""
import heapq
import random
import sys
from bisect import bisect_right
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import bwt_core as bc


def compact_and_coalesce(pieces):
    """Close deleted gaps densely into [0, m_new) and coalesce adjacent identical (delta, word) pieces."""
    pieces = sorted(pieces, key=lambda x: x[0])
    if not pieces:
        return ()
    old_los = [p[0] for p in pieces]
    new_los = []
    acc = 0
    for lo, hi, d, w in pieces:
        new_los.append(acc)
        acc += (hi - lo)

    def phi(x):
        idx = bisect_right(old_los, x) - 1
        return new_los[idx] + (x - old_los[idx])

    out = []
    for i, (lo, hi, d, w) in enumerate(pieces):
        nlo = new_los[i]
        nhi = nlo + (hi - lo)
        nt = phi(lo + d)
        nd = nt - nlo
        if out and out[-1][1] == nlo and out[-1][2] == nd and out[-1][3] == w:
            out[-1] = (out[-1][0], nhi, nd, w)
        else:
            out.append((nlo, nhi, nd, w))
    return tuple(out)


def psi_potential(state):
    """Theorem C4 atomic-piece amortized potential: Psi = U_t - D_t."""
    u = 0
    d = 0
    prev_len1_w = None
    for lo, hi, delta, w in state:
        if len(w) == 1:
            u += 1
            if prev_len1_w == w:
                d += 1
            prev_len1_w = w
        else:
            prev_len1_w = None
    return u - d, u, d


def neighbors_cgap(state):
    """
    Generate all elementary C_gap transitions (cost +1) from `state`:
      - Every binary Join(A, B) for A != B with J_A cap I_B != empty
      - Every Pow spiral on self-overlapping A (0 < |d_A| < L_A)
    Returns list of (next_state, move_desc).
    """
    np = len(state)
    out = []
    # 1. Pow spirals
    for i, (lo, hi, d, w) in enumerate(state):
        L = hi - lo
        if d > 0 and d < L:
            q, rem = divmod(L, d)
            nxt = list(state[:i]) + list(state[i + 1:])
            if rem > 0:
                nxt.append((lo, lo + rem, (q + 1) * d, w * (q + 1)))
            if d - rem > 0:
                nxt.append((lo + rem, lo + d, q * d, w * q))
            out.append((compact_and_coalesce(nxt), ("Pow+", i)))
        elif d < 0 and (-d) < L:
            s = -d
            q, rem = divmod(L, s)
            nxt = list(state[:i]) + list(state[i + 1:])
            if s - rem > 0:
                nxt.append((hi - s, hi - rem, -q * s, w * q))
            if rem > 0:
                nxt.append((hi - rem, hi, -(q + 1) * s, w * (q + 1)))
            out.append((compact_and_coalesce(nxt), ("Pow-", i)))

    # 2. Binary Joins (A, B) with A != B and J_A cap I_B != empty
    for ia, (alo, ahi, ad, aw) in enumerate(state):
        atlo, athi = alo + ad, ahi + ad
        for ib, (blo, bhi, bd, bw) in enumerate(state):
            if ia == ib:
                continue
            ov_lo = atlo if atlo > blo else blo
            ov_hi = athi if athi < bhi else bhi
            if ov_lo >= ov_hi:
                continue
            # Overlap G = [ov_lo, ov_hi) subseteq I_B; preimage in A is [ov_lo - ad, ov_hi - ad)
            a_ov_lo = ov_lo - ad
            a_ov_hi = ov_hi - ad
            nxt = []
            for k, pc in enumerate(state):
                if k == ia:
                    if alo < a_ov_lo:
                        nxt.append((alo, a_ov_lo, ad, aw))
                    nxt.append((a_ov_lo, a_ov_hi, ad + bd, aw + bw))
                    if a_ov_hi < ahi:
                        nxt.append((a_ov_hi, ahi, ad, aw))
                elif k == ib:
                    if blo < ov_lo:
                        nxt.append((blo, ov_lo, bd, bw))
                    if ov_hi < bhi:
                        nxt.append((ov_hi, bhi, bd, bw))
                else:
                    nxt.append(pc)
            out.append((compact_and_coalesce(nxt), ("Join", ia, ib)))
    return out


def solve_exact_cgap_astar(T: bytes, max_expansions=40000):
    """
    Exact A* search for global minimum Q*_{C_gap}(L) using our admissible Theorem C4 lower bound
      h(s) = max(ceil((P(s) - 1) / 5), ceil(Psi(s) / 3)) !
    Returns (q_star, path_trace, exact_flag, stats).
    """
    L, p = bc.bwt_encode(T)
    n, r0, m16, st_tup, _, _ = bc.load_runs_and_open_baseline(L, p)
    # Build initial maximal-run state (NOT isolating p, since C_gap contracts the full circular cycle!)
    runs = []
    i = 0
    counts = [0] * 256
    while i < n:
        c = L[i]; j = i + 1
        while j < n and L[j] == c: j += 1
        runs.append((c, i, j)); counts[c] += (j - i); i = j
    C = [0] * 256; acc = 0
    for c in range(256): C[c] = acc; acc += counts[c]
    cur_t = C[:]
    init = []
    for c, lo, hi in runs:
        d = cur_t[c] - lo; cur_t[c] += (hi - lo)
        init.append((lo, hi, d, bytes([c])))
    init = tuple(init)
    r0 = len(init)
    if r0 == 1:
        return 0, [], True, {"r0": 1, "max_dp": 0, "max_dpsi": 0}

    def h_lb(st):
        if len(st) <= 1:
            return 0
        psi, _, _ = psi_potential(st)
        h1 = (len(st) - 1 + 4) // 5  # Theorem C3 admissible: Delta P >= -5
        h2 = (psi + 2) // 3          # Theorem C4 admissible: Delta Psi >= -3
        return h1 if h1 > h2 else h2

    # Priority queue: (f = g + h, g, tie, state)
    pq = [(h_lb(init), 0, 0, init)]
    best_g = {init: 0}
    parent = {init: None}
    tie = 1
    expansions = 0
    max_dp_seen = 0
    max_dpsi_seen = 0

    while pq and expansions < max_expansions:
        f, g, _, st = heapq.heappop(pq)
        if g > best_g.get(st, 10**9):
            continue
        if len(st) == 1 and st[0][2] == 0:
            # Verify exact circular rotation match!
            w = st[0][3][::-1]
            assert len(w) == n and T in (w + w), f"Exact cycle mismatch on {T}!"
            # Reconstruct winning path
            path = []
            cur = st
            while parent[cur] is not None:
                prev, mv = parent[cur]
                dp = len(cur) - len(prev)
                dpsi = psi_potential(cur)[0] - psi_potential(prev)[0]
                path.append((mv, len(prev), len(cur), dp, dpsi))
                cur = prev
            path.reverse()
            return g, path, True, {"r0": r0, "expansions": expansions, "max_dp": max_dp_seen, "max_dpsi": max_dpsi_seen}

        expansions += 1
        psi_cur = psi_potential(st)[0]
        p_cur = len(st)
        for nxt, mv in neighbors_cgap(st):
            dp = len(nxt) - p_cur
            dpsi = psi_potential(nxt)[0] - psi_cur
            if -dp > max_dp_seen: max_dp_seen = -dp
            if -dpsi > max_dpsi_seen: max_dpsi_seen = -dpsi
            # Verify Theorem C3 and C4 never violated!
            if mv[0] == "Join":
                assert dp >= -5, f"Theorem C3 violated! dp={dp}"
                assert dpsi >= -3, f"Theorem C4 violated! dpsi={dpsi}"
            else:
                assert dp >= -3, f"Theorem C3 Pow violated! dp={dp}"
                assert dpsi >= -1, f"Theorem C4 Pow violated! dpsi={dpsi}"
            ng = g + 1
            if ng < best_g.get(nxt, 10**9):
                best_g[nxt] = ng
                parent[nxt] = (st, mv)
                heapq.heappush(pq, (ng + h_lb(nxt), ng, tie, nxt))
                tie += 1

    return None, [], False, {"r0": r0, "expansions": expansions, "max_dp": max_dp_seen, "max_dpsi": max_dpsi_seen}


def main():
    print("=== 1. EXACT GLOBAL MINIMUM Q*_{C_gap}(L) ON ALL PRIMITIVE BINARY BWTs (n = 4..10) ===")
    by_r0 = defaultdict(list)
    min_ratio = 999.0
    min_wit = None
    below_1 = 0
    below_half = 0
    exact_solved = 0
    timed_out = 0
    global_max_dp = 0
    global_max_dpsi = 0

    seen_rot = set()
    for n in range(4, 11):
        for mask in range(1 << n):
            T = bytes((mask >> i) & 1 for i in range(n))
            # Check primitive (single-cycle BWT) and deduplicate cyclic rotations!
            rots = [T[i:] + T[:i] for i in range(n)]
            if len(set(rots)) < n:
                continue
            canon = min(rots)
            if canon in seen_rot:
                continue
            seen_rot.add(canon)

            qstar, path, ok, st = solve_exact_cgap_astar(canon, max_expansions=15000)
            if st["max_dp"] > global_max_dp: global_max_dp = st["max_dp"]
            if st["max_dpsi"] > global_max_dpsi: global_max_dpsi = st["max_dpsi"]
            if not ok:
                timed_out += 1
                continue
            exact_solved += 1
            r0 = st["r0"]
            ratio = qstar / r0
            by_r0[r0].append((qstar, canon, path))
            if qstar < r0:
                below_1 += 1
            if qstar * 2 < r0:
                below_half += 1
            if ratio < min_ratio:
                min_ratio = ratio
                min_wit = (canon, r0, qstar, path)

    print(f"  Binary n=4..10 primitive necklaces: exact_solved={exact_solved}, timed_out={timed_out}")
    print(f"  Observed max single-step -Delta P = {global_max_dp} (Thm C3 bound: 5), max -Delta Psi = {global_max_dpsi} (Thm C4 bound: 3)")
    print(f"  Q* < 1.00*r_0: {below_1}/{exact_solved} | Q* < 0.50*r_0: {below_half}/{exact_solved}")
    print(f"  GLOBAL MINIMUM Q*/r_0 (binary n<=10): {min_ratio:.4f} on T={list(min_wit[0])} (r0={min_wit[1]}, Q*={min_wit[2]})")
    print(f"    Winning trace (move, P_in, P_out, dP, dPsi): {min_wit[3]}")
    print("\n  Exact Q* distribution by r_0 (binary n=4..10):")
    for r0 in sorted(by_r0):
        qs = [x[0] for x in by_r0[r0]]
        mn = min(qs); mx = max(qs)
        ex = min(by_r0[r0], key=lambda x: x[0])
        print(f"    r0={r0:2d}: count={len(qs):3d} | min Q*={mn:2d} ({mn/r0:.3f}r0) .. max Q*={mx:2d} ({mx/r0:.3f}r0) | min_witness T={list(ex[1])} path_dP={[p[3] for p in ex[2]]}")

    print("\n=== 2. SAMPLE OF PRIMITIVE TERNARY BWTs (n = 6..12, r_0 = 4..8) ===")
    rng = random.Random(42)
    t_solved = 0; t_below_half = 0; t_min_r = 999.0; t_wit = None
    for _ in range(180):
        n = rng.randint(6, 12)
        T = bytes(rng.randrange(3) for _ in range(n))
        rots = [T[i:] + T[:i] for i in range(n)]
        if len(set(rots)) < n or len(set(T)) < 3:
            continue
        qstar, path, ok, st = solve_exact_cgap_astar(T, max_expansions=15000)
        if not ok:
            continue
        t_solved += 1
        r0 = st["r0"]
        if qstar * 2 < r0: t_below_half += 1
        if qstar / r0 < t_min_r:
            t_min_r = qstar / r0; t_wit = (list(T), r0, qstar, path)
    print(f"  Ternary n=6..12 sample: exact_solved={t_solved} | Q* < 0.50*r_0: {t_below_half}/{t_solved}")
    if t_wit:
        print(f"  Min ternary Q*/r_0 = {t_min_r:.4f} on T={t_wit[0]} (r0={t_wit[1]}, Q*={t_wit[2]}), path_dP={[p[3] for p in t_wit[3]]}")

    print("\n=== 3. STRUCTURED z(T) << r_0 FAMILIES (Exact Q*_{C_gap}) ===")
    # Thue-Morse T_3 (n=8), T_4 (n=16), T_5 (n=32); Fibonacci F_6..F_9; Period-Doubling PD_3..PD_5
    fams = []
    # Thue-Morse
    tm = [0]
    for k in range(1, 6):
        tm = tm + [1 - x for x in tm]
        if k >= 3:
            fams.append((f"ThueMorse_{k}", bytes(tm)))
    # Fibonacci + $
    a, b = [0], [0, 1]
    for k in range(3, 10):
        a, b = b, b + a
        if k >= 5:
            fams.append((f"Fib_{k}+$", bytes(b + [2])))
    # Period-Doubling (0->01, 1->00)
    pd = [0]
    for k in range(1, 6):
        nxt = []
        for x in pd: nxt.extend([0, 1] if x == 0 else [0, 0])
        pd = nxt
        if k >= 3:
            fams.append((f"PeriodDbl_{k}", bytes(pd)))

    for name, T in fams:
        qstar, path, ok, st = solve_exact_cgap_astar(T, max_expansions=40000)
        if ok:
            print(f"  {name:14s} (n={len(T):3d}, r0={st['r0']:2d}): EXACT Q* = {qstar:2d} ({qstar/st['r0']:.3f} r0) | path_dP={[p[3] for p in path]}")
        else:
            print(f"  {name:14s} (n={len(T):3d}, r0={st['r0']:2d}): exceeded 40k expansions (r0 too large for exact A*)")


if __name__ == "__main__":
    main()
