#!/usr/bin/env python3
"""Adversarial verifier for MONODROMY records 96, 98, 99, 100 and main-theorem.md.

Checks:
1. Break 1: Theorem G1 (c1 >= h1 on every Column) fails on multi-cycle columns.
2. Break 2a: Mirror-rule row attribution in c1_split crosses N/R boundaries.
3. Break 2b: Open's c1 vs LF walk's c1 across the 232 census blocks under Reading A vs B.
"""

from __future__ import annotations

import csv
import itertools
from pathlib import Path


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


def maximal_runs(L: bytes) -> list[tuple[int, int, int]]:
    runs = []
    n = len(L)
    i = 0
    while i < n:
        j = i + 1
        while j < n and L[j] == L[i]:
            j += 1
        runs.append((i, j, L[i]))
        i = j
    return runs


def marked_cycle(lf: list[int], p: int) -> list[int]:
    cyc = [p]
    x = lf[p]
    while x != p:
        cyc.append(x)
        x = lf[x]
    return cyc


def decode_spec(L: bytes, p: int) -> bytes:
    lf = compute_lf(L)
    n = len(L)
    psi = [0] * n
    for i, j in enumerate(lf):
        psi[j] = i
    out = bytearray(n)
    x = p
    for j in range(n):
        x = psi[x]
        out[j] = L[x]
    return bytes(out)


def check_break1() -> None:
    print("=== BREAK 1: G1 on multi-cycle Column ===")
    for name, L, p in [
        ("minimal-c1=0", b"aab", 0),
        ("minimal-n=2", b"ab", 0),
        ("large-gap", b"aaabcdef", 0),
    ]:
        lf = compute_lf(L)
        runs = maximal_runs(L)
        h1 = sum(1 for a, b, _ in runs if b - a == 1)
        cyc = set(marked_cycle(lf, p))
        h1_marked = sum(1 for a, b, _ in runs if b - a == 1 and a in cyc)
        out = decode_spec(L, p)
        # Check whether the marked cycle touches only runs of width >= 2:
        run_of = {}
        for a, b, _ in runs:
            for x in range(a, b):
                run_of[x] = (a, b)
        c1_min_chain = sum(1 for x in cyc if run_of[x][1] - run_of[x][0] == 1)
        print(
            f"  {name}: L={L.decode()!r}, p={p}, LF={lf}, marked_cycle={sorted(cyc)}, "
            f"decode={out.decode()!r}, h1={h1}, h1_marked={h1_marked}, c1_chain={c1_min_chain}"
        )
        assert c1_min_chain < h1, "Expected strict violation of c1 >= h1"

    # Exhaustive check over all binary columns n <= 6, all p:
    total = 0
    violations = 0
    zero_c1_violations = 0
    for n in range(1, 7):
        for bits in itertools.product(b"ab", repeat=n):
            L = bytes(bits)
            lf = compute_lf(L)
            runs = maximal_runs(L)
            h1 = sum(1 for a, b, _ in runs if b - a == 1)
            for p in range(n):
                total += 1
                cyc = set(marked_cycle(lf, p))
                h1_marked = sum(1 for a, b, _ in runs if b - a == 1 and a in cyc)
                if h1_marked < h1:
                    violations += 1
                    if h1_marked == 0:
                        zero_c1_violations += 1
    print(
        f"  Exhaustive binary n=1..6 (all p): {violations}/{total} ({100*violations/total:.1f}%) "
        f"have h1_marked < h1; {zero_c1_violations} have c1=0 < h1."
    )


def check_break2_mirror_split() -> None:
    print("\n=== BREAK 2a: Mirror rule subject row crosses N/R split ===")
    # Single-cycle column with hex (R) and non-hex (N) width-1 runs:
    # L = b"g0g0" -> check single cycle:
    # Let's find smallest single-cycle column where a width-1 N-run maps via LF to an R-row.
    for L in [b"g0g", b"0g0", b"ag0", b"g0x", b"gg0x", b"0gx0", b"x0g"]:
        lf = compute_lf(L)
        if len(marked_cycle(lf, 0)) == len(L):
            runs = maximal_runs(L)
            hex_set = set(b"0123456789abcdefABCDEF")
            for a, b, ch in runs:
                if b - a == 1:
                    dest = lf[a]
                    if (ch in hex_set) != (L[dest] in hex_set):
                        print(
                            f"  Witness L={L.decode()!r} (single cycle, LF={lf}): "
                            f"width-1 run x={a} (L[x]={chr(ch)!r}, stream={'R' if ch in hex_set else 'N'}) "
                            f"has mirror subject LF(x)={dest} (L[LF(x)]={chr(L[dest])!r}, "
                            f"stream={'R' if L[dest] in hex_set else 'N'})"
                        )
                        return


def check_break2_census_open() -> None:
    print("\n=== BREAK 2b: Open retained rows m vs h1 on 232 distinct census blocks ===")
    census_path = Path("docs/journal/2026-11-24-wikidata-block-census/census.tsv")
    seen = {}
    with census_path.open() as f:
        for row in csv.DictReader(f, delimiter="\t"):
            bid = row["id"]
            if bid in seen:
                continue
            m = int(row["m"])
            if m < 65536:
                continue  # match #805's 232 distinct two-ended blocks
            seen[bid] = (m, int(row["runs_1"]), int(row["runs"]))
    ratios = sorted(m / h1 for m, h1, _ in seen.values())
    diffs = sorted(m - h1 for m, h1, _ in seen.values())
    print(f"  Distinct two-ended blocks: {len(seen)}")
    print(
        f"  m / h1: min={ratios[0]:.3f}, median={ratios[len(ratios)//2]:.3f}, max={ratios[-1]:.3f}"
    )
    print(
        f"  m - h1 (width 2..16 retained rows): min={diffs[0]}, "
        f"median={diffs[len(diffs)//2]}, max={diffs[-1]}"
    )


if __name__ == "__main__":
    check_break1()
    check_break2_mirror_split()
    check_break2_census_open()
