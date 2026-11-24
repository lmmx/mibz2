#!/usr/bin/env python3
"""N-18: Complete-observation fibres (small exhaustive n) & 6-block nu_hat threshold distribution.

Part 1 (Small exhaustive single-cycle BWT columns):
  Enumerates all valid single-cycle columns L in {0..sigma-1}^n (verified against the
  primitive necklace count M(n, sigma)) with fixed origin o = 0, and groups them by:
    (i-a) raw bytes L|_D alone (no histogram),
    (i-b) global histogram H + raw bytes L|_D,
    (ii)  global histogram H + first positional moment A = sum_i i * code(L[i]),
    (ii+) global histogram H + (A, A2) where A2 = sum_i i^2 * code(L[i]),
    (iii) global histogram H + run records with LF translations (s_d, e_d, L[d], LF(d)-d) on D,
    (iii-raw) run records with LF translations on D without H.
  Reports exact fibre counts, ambiguous fibres (>=2 distinct pos_0 maps), max fibre size,
  conditional entropy H(pos_0 | obs), exact fibre-fixed row sets U_O (and J1 cap U_O),
  concrete minimal witness pairs, and exhaustive verification of the sharp 2|D| / 1|D|
  disjoint-span flip blindness coefficients.

Part 2 (6 admitted blocks of origin/data/admitted-bwt):
  Reuses measure_cycle2.py's valid Type-A/B flip enumeration and optimal disjoint-span
  scheduling to compute the observation-relative threshold distribution
      #{x in J1 : nu_hat(x) <= q}
  and critical budgets q_{1%}, q_{5%}, q_{10%}, q_{25%}, q_{50%} bearing directly on
  MONODROMY's Theorem N' (record 107).
"""

from __future__ import annotations

import itertools
import math
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from measure_cycle2 import compute_lf, load_admitted_blocks, orbit_maps  # noqa: E402

HEX_BYTES = set(b"0123456789abcdefABCDEF")


def mobius(k: int) -> int:
    mu = 1
    d = 2
    x = k
    while d * d <= x:
        if x % d == 0:
            x //= d
            if x % d == 0:
                return 0
            mu = -mu
        d += 1
    if x > 1:
        mu = -mu
    return mu


def primitive_necklaces(n: int, sigma: int) -> int:
    total = 0
    for d in range(1, n + 1):
        if n % d == 0:
            total += mobius(d) * (sigma ** (n // d))
    assert total % n == 0
    return total // n


def lf_and_pos0(L: tuple[int, ...], sigma: int) -> tuple[tuple[int, ...], tuple[int, ...]] | None:
    """Return (LF, pos_0) if L has a single n-cycle under LF, else None."""
    n = len(L)
    counts = [0] * sigma
    for b in L:
        counts[b] += 1
    C = [0] * sigma
    acc = 0
    for b in range(sigma):
        C[b] = acc
        acc += counts[b]
    seen = [0] * sigma
    lf = [0] * n
    for i, b in enumerate(L):
        lf[i] = C[b] + seen[b]
        seen[b] += 1
    pos = [-1] * n
    x = 0
    for step in range(n):
        if pos[x] != -1:
            return None
        pos[x] = step
        x = lf[x]
    if x != 0:
        return None
    return tuple(lf), tuple(pos)


def w1_mask(L: tuple[int, ...]) -> tuple[int, ...]:
    n = len(L)
    out = []
    for i in range(n):
        left_same = i > 0 and L[i - 1] == L[i]
        right_same = i + 1 < n and L[i + 1] == L[i]
        if not left_same and not right_same:
            out.append(i)
    return tuple(out)


def run_record_with_lf(L: tuple[int, ...], lf: tuple[int, ...], d: int) -> tuple[int, int, int, int]:
    """Full run record (s_d, e_d, L[d], LF(d) - d) for row d."""
    n = len(L)
    b = L[d]
    s = d
    while s > 0 and L[s - 1] == b:
        s -= 1
    e = d + 1
    while e < n and L[e] == b:
        e += 1
    return (s, e, b, lf[d] - d)


def hist_of(L: tuple[int, ...], sigma: int) -> tuple[int, ...]:
    h = [0] * sigma
    for b in L:
        h[b] += 1
    return tuple(h)


def enumerate_valid_columns(n: int, sigma: int):
    """Return list of (L, lf, pos0, hist, j1) for all valid single-cycle columns."""
    valid = []
    for L in itertools.product(range(sigma), repeat=n):
        res = lf_and_pos0(L, sigma)
        if res is not None:
            lf, pos0 = res
            valid.append((L, lf, pos0, hist_of(L, sigma), w1_mask(L)))
    expected = primitive_necklaces(n, sigma)
    assert len(valid) == expected, f"Expected M({n},{sigma})={expected}, got {len(valid)}"
    return valid


def analyze_model(valid, n: int, key_fn):
    """Group valid columns by key_fn(L, lf, pos0, hist) and compute exact fibre ambiguity metrics."""
    fibres: dict = defaultdict(list)
    for item in valid:
        L, lf, pos0, hist, j1 = item
        fibres[key_fn(L, lf, pos0, hist)].append(item)

    V = len(valid)
    num_fibres = len(fibres)
    ambig_fibres = 0
    ambig_cols = 0
    max_pos_fibre = 1
    cond_entropy = 0.0
    sum_u_frac = 0.0
    sum_j1_u_frac = 0.0
    j1_nonempty = 0
    witness = None

    for key, members in fibres.items():
        # Distinct pos_0 maps in this fibre
        pos_groups: dict[tuple[int, ...], list] = defaultdict(list)
        for item in members:
            pos_groups[item[2]].append(item)
        k_pos = len(pos_groups)
        m_cnt = len(members)
        if k_pos > max_pos_fibre:
            max_pos_fibre = k_pos
        if k_pos >= 2:
            ambig_fibres += 1
            ambig_cols += m_cnt
            cond_entropy += (m_cnt / V) * math.log2(k_pos)
            if witness is None:
                p_list = list(pos_groups.values())
                witness = (key, p_list[0][0], p_list[1][0])

        # Exact U_O for this fibre: rows x whose pos_0(x) is identical across ALL members
        distinct_pos = list(pos_groups.keys())
        fixed_rows = set()
        for x in range(n):
            v0 = distinct_pos[0][x]
            if all(p[x] == v0 for p in distinct_pos[1:]):
                fixed_rows.add(x)
        sum_u_frac += m_cnt * (len(fixed_rows) / n)
        for item in members:
            j1 = item[4]
            if j1:
                j1_nonempty += 1
                sum_j1_u_frac += sum(1 for x in j1 if x in fixed_rows) / len(j1)

    return {
        "V": V,
        "fibres": num_fibres,
        "ambig_fibres": ambig_fibres,
        "ambig_cols_pct": 100.0 * ambig_cols / V,
        "max_pos_fibre": max_pos_fibre,
        "H_pos_given_obs": cond_entropy,
        "mean_U_frac": sum_u_frac / V,
        "mean_J1_U_frac": (sum_j1_u_frac / j1_nonempty) if j1_nonempty else 0.0,
        "witness": witness,
    }


def fmt_L(L: tuple[int, ...]) -> str:
    return "".join(chr(ord("a") + c) for c in L)


def verify_sharp_blindness_coefficients() -> None:
    """Exhaustively verify sharp disjoint-span flip blindness coefficients (1|D| raw/pointwise, 2|D| full run+LF)."""
    # 1. Verify round-2 witness bacacb <-> cacabb for D={2}
    L1 = tuple(ord(c) - ord("a") for c in "bacacb")
    L2 = tuple(ord(c) - ord("a") for c in "cacabb")
    lf1, pos1 = lf_and_pos0(L1, 3)
    lf2, pos2 = lf_and_pos0(L2, 3)
    assert lf1 == (2, 0, 4, 1, 5, 3) and lf2 == (4, 0, 5, 1, 2, 3)
    rr1 = run_record_with_lf(L1, lf1, 2)
    rr2 = run_record_with_lf(L2, lf2, 2)
    assert rr1 == (2, 3, 2, 2) and rr2 == (2, 3, 2, 3) and rr1 != rr2

    # 2. Exhaustively check sigma=3, n=4..7: max pairwise-disjoint valid flips affecting:
    #    (a) raw byte L[d] alone: <= 1
    #    (b) pointwise (L[d], LF(d)-d): <= 1
    #    (c) full run record with LF (s_d, e_d, L[d], LF(d)-d): <= 2
    max_raw = 0
    max_ptlf = 0
    max_full = 0
    witness_2 = None
    for n in range(4, 8):
        valid = enumerate_valid_columns(n, 3)
        for L, lf, pos0, _, _ in valid:
            # Find all valid Type-A/B flips (i, m, j) on L
            flips = []
            for i in range(n):
                for j in range(i + 2, n):
                    a, b = L[i], L[j]
                    if a == b:
                        continue
                    mid = [k for k in range(i + 1, j) if L[k] == a or L[k] == b]
                    if len(mid) != 1:
                        continue
                    m = mid[0]
                    L2_list = list(L)
                    L2_list[i], L2_list[j] = L2_list[j], L2_list[i]
                    res2 = lf_and_pos0(tuple(L2_list), 3)
                    if res2 is not None:
                        flips.append((i, m, j, tuple(L2_list), res2[0]))

            if not flips:
                continue
            for d in range(n):
                rr0 = run_record_with_lf(L, lf, d)
                # Collect flips affecting each observation type
                aff_raw = []
                aff_ptlf = []
                aff_full = []
                for i, m, j, L2_t, lf2_t in flips:
                    if L2_t[d] != L[d]:
                        aff_raw.append((j, i))
                    if L2_t[d] != L[d] or (lf2_t[d] - d) != (lf[d] - d):
                        aff_ptlf.append((j, i))
                    if run_record_with_lf(L2_t, lf2_t, d) != rr0:
                        aff_full.append((j, i))

                def max_disjoint(spans):
                    spans.sort()
                    cnt = 0
                    last_j = -1
                    chosen = []
                    for j_s, i_s in spans:
                        if i_s > last_j:
                            cnt += 1
                            last_j = j_s
                            chosen.append((i_s, j_s))
                    return cnt, chosen

                c_raw, _ = max_disjoint(aff_raw)
                c_ptlf, _ = max_disjoint(aff_ptlf)
                c_full, ch_full = max_disjoint(aff_full)
                if c_raw > max_raw:
                    max_raw = c_raw
                if c_ptlf > max_ptlf:
                    max_ptlf = c_ptlf
                if c_full > max_full:
                    max_full = c_full
                if c_full == 2 and witness_2 is None:
                    witness_2 = (n, fmt_L(L), d, ch_full)

    assert max_raw <= 1, f"Expected <=1 for raw byte, got {max_raw}"
    assert max_ptlf <= 1, f"Expected <=1 for pointwise (L[d], LF(d)-d), got {max_ptlf}"
    assert max_full <= 2, f"Expected <=2 for full run+LF, got {max_full}"
    print("=== SHARP BLINDNESS COEFFICIENTS EXHAUSTIVELY VERIFIED (sigma=3, n=4..7) ===")
    print(f"  round-2 witness bacacb <-> cacabb at D={{2}}: rr1={rr1} != rr2={rr2} (separated)")
    print(f"  max disjoint-span flips affecting a single d in D:")
    print(f"    raw byte L[d]:                     max = {max_raw} (bound <= 1 -> |P| + |D| > nu')")
    print(f"    pointwise (L[d], LF(d)-d):         max = {max_ptlf} (bound <= 1 -> |P| + |D| > nu')")
    print(f"    full run + LF (s, e, L[d], LF-d):  max = {max_full} (bound <= 2 -> |P| + 2|D| > nu', sharp; witness={witness_2})")


def run_part1_exhaustive_fibres() -> None:
    print("\n=== N-18 PART 1: COMPLETE-OBSERVATION FIBRES (SMALL EXHAUSTIVE SINGLE-CYCLE COLUMNS, o=0) ===")
    # Verify round-2 (H, A) witness bca vs cab at n=3, sigma=3
    L_bca = (1, 2, 0)
    L_cab = (2, 0, 1)
    lf_b, pos_b = lf_and_pos0(L_bca, 3)
    lf_c, pos_c = lf_and_pos0(L_cab, 3)
    assert hist_of(L_bca, 3) == hist_of(L_cab, 3) == (1, 1, 1)
    assert sum(i * c for i, c in enumerate(L_bca)) == sum(i * c for i, c in enumerate(L_cab)) == 2
    assert lf_b == (1, 2, 0) and lf_c == (2, 0, 1) and pos_b != pos_c
    print("  Verified (H, A) witness: bca vs cab (H=(1,1,1), A=2, pos_0=(0,1,2) vs (0,2,1))")

    # Test grids: sigma=3 for n=3..10, sigma=4 for n=3..8, sigma=2 for n=4..12
    configs = [
        (3, list(range(3, 11))),
        (4, list(range(3, 9))),
        (2, [4, 6, 8, 10, 12]),
    ]

    for sigma, ns in configs:
        print(f"\n--- Alphabet sigma = {sigma} ---")
        for n in ns:
            valid = enumerate_valid_columns(n, sigma)
            D1 = (0,)
            D2 = (0, n // 2) if n >= 2 else (0,)
            Dhalf = tuple(range(0, n, 2))  # ceil(n/2) evenly spaced probes

            models = [
                ("(i-a) raw L|_D1 (D={0}, no H)", lambda L, lf, p, h, D=D1: tuple(L[d] for d in D)),
                ("(i-b) H + raw L|_D1 (D={0})", lambda L, lf, p, h, D=D1: (h, tuple(L[d] for d in D))),
                ("(i-c) H + raw L|_Dhalf (|D|=ceil(n/2))", lambda L, lf, p, h, D=Dhalf: (h, tuple(L[d] for d in D))),
                ("(ii)  H + first moment A", lambda L, lf, p, h: (h, sum(i * c for i, c in enumerate(L)))),
                ("(ii+) H + moments (A, A2)", lambda L, lf, p, h: (h, sum(i * c for i, c in enumerate(L)), sum(i * i * c for i, c in enumerate(L)))),
                ("(iii-a) H + run+LF on D1 ({0})", lambda L, lf, p, h, D=D1: (h, tuple(run_record_with_lf(L, lf, d) for d in D))),
                ("(iii-b) H + run+LF on D2 ({0,n//2})", lambda L, lf, p, h, D=D2: (h, tuple(run_record_with_lf(L, lf, d) for d in D))),
                ("(iii-c) H + run+LF on Dhalf (|D|=ceil(n/2))", lambda L, lf, p, h, D=Dhalf: (h, tuple(run_record_with_lf(L, lf, d) for d in D))),
            ]

            print(f"\n  n={n} (V={len(valid):,} valid single-cycle columns = M({n},{sigma})):")
            print(
                f"    {'Model':<44} {'fibres':>7} {'ambigF':>7} {'ambig%':>7} "
                f"{'maxF':>6} {'H(pos|O)':>8} {'E[|U|/n]':>8} {'E[|J1nU|/|J1|]':>14}"
            )
            for mname, kfn in models:
                res = analyze_model(valid, n, kfn)
                print(
                    f"    {mname:<44} {res['fibres']:>7,} {res['ambig_fibres']:>7,} "
                    f"{res['ambig_cols_pct']:>6.1f}% {res['max_pos_fibre']:>6,} "
                    f"{res['H_pos_given_obs']:>8.3f} {res['mean_U_frac']:>8.3f} "
                    f"{res['mean_J1_U_frac']:>14.3f}"
                )
                if n in (3, 4, 5, 6) and res["witness"] is not None and mname in (
                    "(i-b) H + raw L|_D1 (D={0})",
                    "(ii)  H + first moment A",
                    "(iii-a) H + run+LF on D1 ({0})",
                    "(iii-b) H + run+LF on D2 ({0,n//2})",
                    "(iii-c) H + run+LF on Dhalf (|D|=ceil(n/2))",
                ):
                    _, w1, w2 = res["witness"]
                    print(
                        f"       witness: L1={fmt_L(w1[0])} (pos0={w1[2]}) vs "
                        f"L2={fmt_L(w2[0])} (pos0={w2[2]})"
                    )


def run_part2_six_block_thresholds() -> None:
    print("\n=== N-18 PART 2: 6-BLOCK OBSERVATION-RELATIVE U THRESHOLD DISTRIBUTION #{x in J1 : nu_hat(x) <= q} ===")
    SPAN_CAP = 256
    Q_GRID = [0, 1, 10, 50, 100, 250, 500, 1000, 2000, 4000, 6000, 8000, 10000, 12000, 15000, 20000]
    blocks = load_admitted_blocks()

    for fname, p, L in blocks:
        n = len(L)
        lf = compute_lf(L)
        _, _, lf_pos = orbit_maps(L, p, lf)

        last = [-1] * 256
        prev_occ = [-1] * n
        next_occ = [n] * n
        for idx, b in enumerate(L):
            pr = last[b]
            prev_occ[idx] = pr
            if pr != -1:
                next_occ[pr] = idx
            last[b] = idx

        is_w1 = bytearray(n)
        i = 0
        while i < n:
            j = i + 1
            b = L[i]
            while j < n and L[j] == b:
                j += 1
            if j - i == 1:
                is_w1[i] = 1
            i = j

        flips: list[tuple[int, int, int, int]] = []
        min_t1 = n
        max_t3 = -1

        # Type B
        for m in range(n):
            j = next_occ[m]
            if j >= n or j - m >= SPAN_CAP:
                continue
            lo = max(prev_occ[m] + 1, j - SPAN_CAP)
            pm = lf_pos[m]
            pj = lf_pos[j]
            if pm < pj:
                for idx in range(lo, m):
                    if next_occ[idx] > j:
                        pi = lf_pos[idx]
                        if pi < pm or pi > pj:
                            t1, t3 = (pi, pj) if pi < pm else (pm, pi)
                            flips.append((j, idx, t1, t3))
                            if t1 < min_t1:
                                min_t1 = t1
                            if t3 > max_t3:
                                max_t3 = t3
            else:
                for idx in range(lo, m):
                    if next_occ[idx] > j:
                        pi = lf_pos[idx]
                        if pj < pi < pm:
                            flips.append((j, idx, pj, pm))
                            if pj < min_t1:
                                min_t1 = pj
                            if pm > max_t3:
                                max_t3 = pm

        # Type A
        for idx in range(n):
            m = next_occ[idx]
            if m >= n or m - idx >= SPAN_CAP:
                continue
            hi = min(next_occ[m], idx + SPAN_CAP + 1)
            pi = lf_pos[idx]
            pm = lf_pos[m]
            if pi < pm:
                for j in range(m + 1, hi):
                    if prev_occ[j] < idx:
                        pj = lf_pos[j]
                        if pi < pj < pm:
                            flips.append((j, idx, pi, pm))
                            if pi < min_t1:
                                min_t1 = pi
                            if pm > max_t3:
                                max_t3 = pm
            else:
                for j in range(m + 1, hi):
                    if prev_occ[j] < idx:
                        pj = lf_pos[j]
                        if pj > pi or pj < pm:
                            t1, t3 = (pm, pj) if pj > pi else (pj, pi)
                            flips.append((j, idx, t1, t3))
                            if t1 < min_t1:
                                min_t1 = t1
                            if t3 > max_t3:
                                max_t3 = t3

        flips.sort()
        disj_flips = []
        last_j = -1
        for j, idx, t1, t3 in flips:
            if idx > last_j:
                disj_flips.append((idx, j, t1, t3))
                last_j = j

        diff = [0] * (n + 2)
        for _, _, t1, t3 in disj_flips:
            diff[t1 + 1] += 1
            diff[t3 + 1] -= 1
        nu_at_pos = [0] * n
        cur = 0
        for pos in range(n):
            cur += diff[pos]
            nu_at_pos[pos] = cur

        nu_j1 = sorted(nu_at_pos[lf_pos[idx]] for idx in range(n) if is_w1[idx])
        nu_hex = sorted(nu_at_pos[lf_pos[idx]] for idx in range(n) if is_w1[idx] and L[idx] in HEX_BYTES)
        nu_non = sorted(nu_at_pos[lf_pos[idx]] for idx in range(n) if is_w1[idx] and L[idx] not in HEX_BYTES)
        h1 = len(nu_j1)
        u_all_j1 = sum(1 for idx in range(n) if is_w1[idx] and (lf_pos[idx] <= min_t1 or lf_pos[idx] > max_t3))
        u_all_total = (min_t1 + 1) + (n - 1 - max_t3)

        import bisect

        print(f"\n  {fname}: h1={h1:,}, |F_disj|={len(disj_flips):,}, |U_all_flips|={u_all_total} (in J1: {u_all_j1})")
        q_pcts = [
            (pct, nu_j1[min(h1 - 1, int(math.floor(pct * h1)))])
            for pct in (0.01, 0.05, 0.10, 0.25, 0.50, 0.75, 0.90)
        ]
        print(
            "    Critical budgets q_p (max q with #{J1: nu_hat<=q} <= p*h1): "
            + ", ".join(f"q_{int(100*pct)}%={val:,}" for pct, val in q_pcts)
        )
        print(
            f"    Stream split q_50%: hex_w1={nu_hex[len(nu_hex)//2]:,}, nonhex_w1={nu_non[len(nu_non)//2]:,} | "
            f"q_10%: hex_w1={nu_hex[len(nu_hex)//10]:,}, nonhex_w1={nu_non[len(nu_non)//10]:,}"
        )
        row_parts = []
        for q in Q_GRID:
            cnt = bisect.bisect_right(nu_j1, q)
            row_parts.append(f"q<={q}:{cnt}({100*cnt/h1:.2f}%)")
        print("    Grid: " + " | ".join(row_parts[:8]))
        print("          " + " | ".join(row_parts[8:]))


if __name__ == "__main__":
    verify_sharp_blindness_coefficients()
    run_part1_exhaustive_fibres()
    run_part2_six_block_thresholds()
