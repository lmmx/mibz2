#!/usr/bin/env python3
"""Cycle 2 quantitative measurements on the 6 admitted blocks (data/admitted-bwt).

Computes:
1. N-8: Amended 3-part N-0 (novel hex / repeated hex / non-hex) under two
   tokenizations, exact empirical verification of MONODROMY M-7 (width-1 rows
   per repeated SHA1 / hex occurrence), and copy-sector / r-index ceilings.
2. N-9: MONODROMY NEEDS M-4 (type-A/B valid flips, endpoint coverage on both
   Stream R hex w1 and Stream N non-hex w1, optimal disjoint-span flip count
   nu'/n, per-row nu'(x) distribution for Stream R and Stream N, |U|, and
   orbit cut-point gaps), plus witness verification for Record 101 Thm 3.
3. N-10: SA[x] smoothness (|SA[x+1] - SA[x]| on width-1 runs) for learned
   piecewise-linear / PGM-index / compressed-sensing viability.
"""

from __future__ import annotations

import csv
import json
import os
import re
import subprocess
from collections import Counter
from pathlib import Path

HEX_BYTES = set(b"0123456789abcdefABCDEF")
HEX_TOK_ANY = re.compile(rb"[0-9a-fA-F]+")
HEX_TOK_6 = re.compile(rb"[0-9a-fA-F]{6,}")
SHA1_RE = re.compile(rb"(?<![0-9a-fA-F])[0-9a-fA-F]{40}(?![0-9a-fA-F])")
GUID_RE = re.compile(
    rb"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}"
)


def load_admitted_blocks() -> list[tuple[str, int, bytes]]:
    env_dir = os.environ.get("ADMITTED_BWT_DIR")
    if env_dir and (Path(env_dir) / "manifest.json").exists():
        base = Path(env_dir)
        manifest = json.loads((base / "manifest.json").read_text())
        return [
            (e["file"], e["primary_index"], (base / e["file"]).read_bytes())
            for e in manifest["blocks"]
        ]
    manifest = json.loads(
        subprocess.check_output(
            ["git", "show", "origin/data/admitted-bwt:admitted-bwt/manifest.json"]
        )
    )
    return [
        (
            e["file"],
            e["primary_index"],
            subprocess.check_output(
                ["git", "show", f"origin/data/admitted-bwt:admitted-bwt/{e['file']}"]
            ),
        )
        for e in manifest["blocks"]
    ]


def load_census_costs() -> dict[tuple[int, int, int], float]:
    census_path = Path("docs/journal/2026-11-24-wikidata-block-census/census.tsv")
    feat_path = Path("docs/journal/2026-11-24-wikidata-block-census/features.tsv")
    c_by_seq = {}
    with census_path.open() as f:
        for row in csv.DictReader(f, delimiter="\t"):
            c_by_seq[int(row["seq"])] = row
    costs: dict[tuple[int, int, int], list[float]] = {}
    with feat_path.open() as f:
        for frow in csv.DictReader(f, delimiter="\t"):
            crow = c_by_seq[int(frow["seq"])]
            key = (int(crow["n"]), int(crow["p"]), int(crow["runs"]))
            m = int(crow["m"])
            tot = (float(frow["emit_ns_per_row"]) + float(frow["construct_ns_per_row"])) * m
            costs.setdefault(key, []).append(tot)
    return {k: sum(v) / len(v) for k, v in costs.items()}


def compute_lf(L: bytes) -> list[int]:
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 256
    acc = 0
    for b in range(256):
        C[b] = acc
        acc += counts[b]
    seen = [0] * 256
    lf = [0] * n
    for i, b in enumerate(L):
        lf[i] = C[b] + seen[b]
        seen[b] += 1
    return lf


def orbit_maps(L: bytes, p: int, lf: list[int]) -> tuple[bytes, list[int], list[int]]:
    """Return (T, row_of_text_pos, lf_step_of_row).

    lf_step_of_row[x] = k in [0, n) such that LF^k(p) = x (Record 101's pos(x)).
    T[j] = L[Psi^{j+1}(p)] = L[row_of_text_pos[j]].
    """
    n = len(L)
    lf_pos = [0] * n
    x = p
    for step in range(n):
        lf_pos[x] = step
        x = lf[x]
    assert x == p, "Expected single-cycle column"
    # T[j] corresponds to row x with lf_pos[x] = (n - 1 - j) mod n
    T = bytearray(n)
    row_of_text_pos = [0] * n
    for row in range(n):
        j = (n - 1 - lf_pos[row]) % n
        T[j] = L[row]
        row_of_text_pos[j] = row
    return bytes(T), row_of_text_pos, lf_pos


def verify_flip_arc_formula() -> None:
    """Self-test the Record 101 arc separation formula and Thm 3 run-record witness."""
    # Test 1: Record 101 Thm 3 run-record blindness break witness: L = "caab" <-> "cbaa"
    L1 = b"caab"
    L2 = b"cbaa"
    # i=1, m=2, j=3 (Type A: L[1]=L[2]='a', L[3]='b')
    lf1 = compute_lf(L1)
    lf2 = compute_lf(L2)
    assert len(set(orbit_maps(L1, 0, lf1)[2])) == 4
    assert len(set(orbit_maps(L2, 0, lf2)[2])) == 4

    def run_of(L: bytes, idx: int) -> tuple[int, int, int]:
        a = idx
        while a > 0 and L[a - 1] == L[idx]:
            a -= 1
        b = idx + 1
        while b < len(L) and L[b] == L[idx]:
            b += 1
        return (a, b, L[idx])

    assert run_of(L1, 2) == (1, 3, ord("a"))
    assert run_of(L2, 2) == (2, 4, ord("a"))
    print("=== RECORD 101 THM 3 WITNESS VERIFIED ===")
    print("  L='caab' <-> L'='cbaa' (i=1, m=2, j=3): D={2} has i,j not in D,")
    print(f"  yet row 2's full run record changes: {run_of(L1, 2)} -> {run_of(L2, 2)}")

    # Test 2: Exact verification of arc-separation interval (t1, t3] on L="caab" for all o
    for o in range(4):
        _, _, pos1 = orbit_maps(L1, o, lf1)
        _, _, pos2 = orbit_maps(L2, o, lf2)
        t1, t2, t3 = sorted((pos1[1], pos1[2], pos1[3]))
        for x in range(4):
            moved = pos1[x] != pos2[x]
            in_interval = t1 < pos1[x] <= t3
            assert moved == in_interval, f"Arc formula mismatch at o={o}, x={x}"
    print("  Arc-separation formula (pos(x) in (t1, t3]) verified exact on all o in 0..3.")


def run_part_a(blocks: list[tuple[str, int, bytes]], costs: dict) -> None:
    print("\n=== N-8 PART A: AMENDED 3-PART N-0 (NOVEL HEX / REPEATED HEX / NON-HEX) & M-7 TEST ===")
    # Tokenization 1: maximal [0-9a-fA-F]+ (any length >= 1)
    # Tokenization 2: maximal [0-9a-fA-F]{6,} (identifier-length hex tokens; len < 6 classed separately)
    for label, regex in [
        ("Def1: maximal [0-9a-fA-F]+ (all lengths)", HEX_TOK_ANY),
        ("Def2: hex tokens len>=6 ([0-9a-fA-F]{6,}); short 1-5 hex split out", HEX_TOK_6),
    ]:
        print(f"\n--- {label} ---")
        hdr = (
            f"{'block':<12} {'h1':>7} {'novel_hex':>10} {'rep_hex':>9} "
            f"{'short_hex':>10} {'non_hex':>9} | {'nov%':>6} {'rep%':>6} {'nonh%':>6} {'cost_ms':>8}"
        )
        print(hdr)
        print("-" * len(hdr))
        cw_nov = cw_rep = cw_short = cw_non = cw_tot = 0.0
        for fname, p, L in blocks:
            n = len(L)
            lf = compute_lf(L)
            T, row_of_text_pos, _ = orbit_maps(L, p, lf)
            is_w1 = bytearray(n)
            r = 0
            i = 0
            while i < n:
                j = i + 1
                b = L[i]
                while j < n and L[j] == b:
                    j += 1
                r += 1
                if j - i == 1:
                    is_w1[i] = 1
                i = j
            matches = list(regex.finditer(T))
            tok_counts = Counter(m.group(0) for m in matches)
            # tag each text pos: 0=not in regex match, 1=novel match, 2=repeated match
            t_tag = bytearray(n)
            for m in matches:
                code = 1 if tok_counts[m.group(0)] == 1 else 2
                for k in range(m.start(), m.end()):
                    t_tag[k] = code
            nov = rep = short_h = non_h = 0
            for pos in range(n):
                row = row_of_text_pos[pos]
                if not is_w1[row]:
                    continue
                if L[row] not in HEX_BYTES:
                    non_h += 1
                else:
                    tt = t_tag[pos]
                    if tt == 1:
                        nov += 1
                    elif tt == 2:
                        rep += 1
                    else:
                        short_h += 1
            h1 = nov + rep + short_h + non_h
            cms = costs[(n, p, r)] / 1e6
            cw_nov += cms * (nov / h1)
            cw_rep += cms * (rep / h1)
            cw_short += cms * (short_h / h1)
            cw_non += cms * (non_h / h1)
            cw_tot += cms
            print(
                f"{fname:<12} {h1:>7} {nov:>10} {rep:>9} {short_h:>10} {non_h:>9} | "
                f"{100*nov/h1:>5.1f}% {100*rep/h1:>5.1f}% {100*non_h/h1:>5.1f}% {cms:>8.3f}"
            )
        print(
            f"Cost-weighted 6-block aggregate: novel_hex={100*cw_nov/cw_tot:.2f}%, "
            f"rep_hex={100*cw_rep/cw_tot:.2f}%, short_hex={100*cw_short/cw_tot:.2f}%, "
            f"non_hex={100*cw_non/cw_tot:.2f}%"
        )

    print("\n--- Empirical test of MONODROMY M-7 (repeats-are-wide) on 40-hex SHA1 tokens ---")
    print(f"{'block':<12} {'rep_SHA1_occ':>12} {'rep_SHA1_bytes':>14} {'rep_SHA1_w1':>12} {'w1/occ':>8} {'w1/h1':>7} | {'nov_SHA1_occ':>12} {'nov_w1/occ':>10}")
    for fname, p, L in blocks:
        n = len(L)
        lf = compute_lf(L)
        T, row_of_text_pos, _ = orbit_maps(L, p, lf)
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
        matches = list(SHA1_RE.finditer(T))
        cnt = Counter(m.group(0) for m in matches)
        rep_occ = nov_occ = rep_w1 = nov_w1 = 0
        for m in matches:
            w1_in = sum(1 for k in range(m.start(), m.end()) if is_w1[row_of_text_pos[k]])
            if cnt[m.group(0)] >= 2:
                rep_occ += 1
                rep_w1 += w1_in
            else:
                nov_occ += 1
                nov_w1 += w1_in
        h1 = sum(is_w1)
        w1_per_rep = rep_w1 / rep_occ if rep_occ else 0.0
        w1_per_nov = nov_w1 / nov_occ if nov_occ else 0.0
        print(
            f"{fname:<12} {rep_occ:>12} {rep_occ*40:>14} {rep_w1:>12} {w1_per_rep:>8.2f} "
            f"{100*rep_w1/h1:>6.2f}% | {nov_occ:>12} {w1_per_nov:>10.2f}"
        )


def run_part_b_and_c(blocks: list[tuple[str, int, bytes]]) -> None:
    print("\n=== N-9: NEEDS M-4 (SWAP-INVARIANCE VALID FLIPS ON STREAM R & STREAM N) ===")
    SPAN_CAP = 256  # exact lower bound on valid flips and nu', exact upper bound on |U|
    for fname, p, L in blocks:
        n = len(L)
        lf = compute_lf(L)
        _, _, lf_pos = orbit_maps(L, p, lf)

        # Build prev_occ and next_occ for each row in O(n)
        last = [-1] * 256
        prev_occ = [-1] * n
        next_occ = [n] * n
        for idx, b in enumerate(L):
            pr = last[b]
            prev_occ[idx] = pr
            if pr != -1:
                next_occ[pr] = idx
            last[b] = idx

        # Width-1 mask
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

        # Enumerate valid Type-A and Type-B flips with j - i <= SPAN_CAP
        # Record for each flip: (j, i, t1, t3)
        is_endpoint = bytearray(n)
        flips: list[tuple[int, int, int, int]] = []
        cut_positions_mask = bytearray(n)
        min_t1 = n
        max_t3 = -1

        # Type B: consecutive pair (m, j) of byte b, i in (max(prev_b, j - SPAN_CAP - 1), m)
        # with next_occ[i] > j, valid iff cyclic order i -> m -> j in lf_pos
        for m in range(n):
            j = next_occ[m]
            if j >= n or j - m >= SPAN_CAP:
                continue
            prev_b = prev_occ[m]
            lo = max(prev_b + 1, j - SPAN_CAP)
            pm = lf_pos[m]
            pj = lf_pos[j]
            # Cyclic order i -> m -> j holds iff:
            # if pm < pj: pi < pm or pi > pj
            # if pj < pm: pj < pi < pm
            if pm < pj:
                for idx in range(lo, m):
                    if next_occ[idx] > j:
                        pi = lf_pos[idx]
                        if pi < pm or pi > pj:
                            is_endpoint[idx] = 1
                            is_endpoint[j] = 1
                            if pi < pm:
                                t1, t3 = pi, pj
                            else:
                                t1, t3 = pm, pi
                            flips.append((j, idx, t1, t3))
                            cut_positions_mask[pi] = 1
                            cut_positions_mask[pm] = 1
                            cut_positions_mask[pj] = 1
                            if t1 < min_t1:
                                min_t1 = t1
                            if t3 > max_t3:
                                max_t3 = t3
            else:
                for idx in range(lo, m):
                    if next_occ[idx] > j:
                        pi = lf_pos[idx]
                        if pj < pi < pm:
                            is_endpoint[idx] = 1
                            is_endpoint[j] = 1
                            t1, t3 = pj, pm
                            flips.append((j, idx, t1, t3))
                            cut_positions_mask[pi] = 1
                            cut_positions_mask[pm] = 1
                            cut_positions_mask[pj] = 1
                            if t1 < min_t1:
                                min_t1 = t1
                            if t3 > max_t3:
                                max_t3 = t3

        # Type A: consecutive pair (i, m) of byte a, j in (m, min(next_a, i + SPAN_CAP + 1))
        # with prev_occ[j] < i, valid iff cyclic order i -> j -> m in lf_pos
        for idx in range(n):
            m = next_occ[idx]
            if m >= n or m - idx >= SPAN_CAP:
                continue
            next_a = next_occ[m]
            hi = min(next_a, idx + SPAN_CAP + 1)
            pi = lf_pos[idx]
            pm = lf_pos[m]
            # Cyclic order i -> j -> m holds iff:
            # if pi < pm: pi < pj < pm
            # if pm < pi: pj > pi or pj < pm
            if pi < pm:
                for j in range(m + 1, hi):
                    if prev_occ[j] < idx:
                        pj = lf_pos[j]
                        if pi < pj < pm:
                            is_endpoint[idx] = 1
                            is_endpoint[j] = 1
                            t1, t3 = pi, pm
                            flips.append((j, idx, t1, t3))
                            cut_positions_mask[pi] = 1
                            cut_positions_mask[pm] = 1
                            cut_positions_mask[pj] = 1
                            if t1 < min_t1:
                                min_t1 = t1
                            if t3 > max_t3:
                                max_t3 = t3
            else:
                for j in range(m + 1, hi):
                    if prev_occ[j] < idx:
                        pj = lf_pos[j]
                        if pj > pi or pj < pm:
                            is_endpoint[idx] = 1
                            is_endpoint[j] = 1
                            if pj > pi:
                                t1, t3 = pm, pj
                            else:
                                t1, t3 = pj, pi
                            flips.append((j, idx, t1, t3))
                            cut_positions_mask[pi] = 1
                            cut_positions_mask[pm] = 1
                            cut_positions_mask[pj] = 1
                            if t1 < min_t1:
                                min_t1 = t1
                            if t3 > max_t3:
                                max_t3 = t3

        # (i) Endpoint coverage on width-1 hex (Stream R) AND width-1 non-hex (Stream N)
        h1_hex = h1_hex_ep = h1_non = h1_non_ep = 0
        for idx in range(n):
            if is_w1[idx]:
                if L[idx] in HEX_BYTES:
                    h1_hex += 1
                    if is_endpoint[idx]:
                        h1_hex_ep += 1
                else:
                    h1_non += 1
                    if is_endpoint[idx]:
                        h1_non_ep += 1

        # (ii) Optimal greedy interval scheduling for pairwise-disjoint column spans [i, j]
        flips.sort()  # sorts by j ascending
        disj_flips = []
        last_j = -1
        for j, idx, t1, t3 in flips:
            if idx > last_j:
                disj_flips.append((idx, j, t1, t3))
                last_j = j

        # Per-row nu'(x) from disj_flips via prefix-sum difference array over orbit positions
        diff = [0] * (n + 2)
        for _, _, t1, t3 in disj_flips:
            diff[t1 + 1] += 1
            diff[t3 + 1] -= 1
        nu_at_pos = [0] * n
        cur = 0
        for pos in range(n):
            cur += diff[pos]
            nu_at_pos[pos] = cur

        nu_hex = sorted(nu_at_pos[lf_pos[idx]] for idx in range(n) if is_w1[idx] and L[idx] in HEX_BYTES)
        nu_non = sorted(nu_at_pos[lf_pos[idx]] for idx in range(n) if is_w1[idx] and L[idx] not in HEX_BYTES)

        # (iii) |U|: rows in origPtr's arc for EVERY valid flip found
        # A row x is in origPtr's arc iff lf_pos[x] <= min_t1 or lf_pos[x] > max_t3
        U_size = (min_t1 + 1) + (n - 1 - max_t3)

        # (iv) Orbit cut-point gap distribution across all valid flips
        cuts = [pos for pos in range(n) if cut_positions_mask[pos]]
        gaps = sorted(
            (cuts[(k + 1) % len(cuts)] - cuts[k]) % n for k in range(len(cuts))
        )
        g_med = gaps[len(gaps) // 2]
        g_p90 = gaps[9 * len(gaps) // 10]
        g_max = gaps[-1]

        print(f"  {fname} (valid flips span<={SPAN_CAP}: {len(flips):,}):")
        print(
            f"    (i)  endpoint coverage: hex_w1={h1_hex_ep}/{h1_hex} ({100*h1_hex_ep/h1_hex:.2f}%), "
            f"nonhex_w1={h1_non_ep}/{h1_non} ({100*h1_non_ep/h1_non:.2f}%)"
        )
        print(
            f"    (ii) disjoint-span flips |F_disj|={len(disj_flips):,} (|F_disj|/n={len(disj_flips)/n:.4f}); "
            f"per-row nu'(x): hex_w1 p10/med/p90={nu_hex[len(nu_hex)//10]:,}/{nu_hex[len(nu_hex)//2]:,}/{nu_hex[9*len(nu_hex)//10]:,}; "
            f"nonhex_w1 p10/med/p90={nu_non[len(nu_non)//10]:,}/{nu_non[len(nu_non)//2]:,}/{nu_non[9*len(nu_non)//10]:,}"
        )
        print(
            f"    (iii) |U| <= {U_size} rows (min_t1={min_t1}, n-1-max_t3={n-1-max_t3})"
        )
        print(
            f"    (iv) cut points={len(cuts):,} ({100*len(cuts)/n:.1f}% of n); "
            f"orbit gaps: median={g_med}, p90={g_p90}, max={g_max}"
        )

    print("\n=== N-10 VIABILITY: SA[x] JUMP DISTRIBUTION ON ADJACENT ROWS (LEARNED / PGM-INDEX) ===")
    for fname, p, L in blocks:
        n = len(L)
        lf = compute_lf(L)
        _, _, lf_pos = orbit_maps(L, p, lf)
        # SA[x] = (n - 1 - lf_pos[x]) % n
        sa = [(n - 1 - lf_pos[x]) % n for x in range(n)]
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
        jumps_all_w1 = sorted(abs(sa[x + 1] - sa[x]) for x in range(n - 1) if is_w1[x])
        jumps_nonhex_w1 = sorted(
            abs(sa[x + 1] - sa[x]) for x in range(n - 1) if is_w1[x] and L[x] not in HEX_BYTES
        )
        small_64 = sum(1 for d in jumps_all_w1 if d <= 64)
        print(
            f"  {fname}: w1 |SA[x+1]-SA[x]| median={jumps_all_w1[len(jumps_all_w1)//2]:,} "
            f"(mean/n={sum(jumps_all_w1)/len(jumps_all_w1)/n:.3f}, <=64: {100*small_64/len(jumps_all_w1):.2f}%); "
            f"nonhex_w1 median={jumps_nonhex_w1[len(jumps_nonhex_w1)//2]:,} "
            f"(mean/n={sum(jumps_nonhex_w1)/len(jumps_nonhex_w1)/n:.3f})"
        )


if __name__ == "__main__":
    verify_flip_arc_formula()
    blocks = load_admitted_blocks()
    costs = load_census_costs()
    run_part_a(blocks, costs)
    run_part_b_and_c(blocks)
