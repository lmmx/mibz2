#!/usr/bin/env python3
"""Quantitative measurements for Cycle 1 directed E2 screens (N-3 through N-7).

Runs on the 6 admitted Wikidata blocks (data/admitted-bwt) plus a Tribonacci
Pisot substitution toy witness for N-4.
"""

from __future__ import annotations

import json
import math
import os
import re
import subprocess
from collections import Counter
from pathlib import Path

GUID_RE = re.compile(
    rb"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}"
)
SHA1_RE = re.compile(rb"(?<![0-9a-fA-F])[0-9a-fA-F]{40}(?![0-9a-fA-F])")


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


def invert_bwt(L: bytes, p: int, lf: list[int]) -> tuple[bytes, list[int]]:
    n = len(L)
    psi = [0] * n
    for i, j in enumerate(lf):
        psi[j] = i
    T = bytearray(n)
    row_of_pos = [0] * n
    x = p
    for j in range(n):
        x = psi[x]
        T[j] = L[x]
        row_of_pos[j] = x
    return bytes(T), row_of_pos


def cycle_lengths(perm: list[int]) -> list[int]:
    n = len(perm)
    vis = bytearray(n)
    lens = []
    for i in range(n):
        if not vis[i]:
            c = 0
            x = i
            while not vis[x]:
                vis[x] = 1
                x = perm[x]
                c += 1
            lens.append(c)
    lens.sort(reverse=True)
    return lens


def factor_complexity(data: bytes, ks: list[int]) -> dict[int, int]:
    out = {}
    n = len(data)
    for k in ks:
        seen = set()
        for i in range(n - k + 1):
            seen.add(data[i : i + k])
        out[k] = len(seen)
    return out


def tribonacci_toy() -> None:
    """Verify N-4 escape on Tribonacci Pisot substitution (a->ab, b->ac, c->a)."""
    s = "a"
    for _ in range(11):
        s = "".join({"a": "ab", "b": "ac", "c": "a"}[ch] for ch in s)
    # Use prefix of length 274 (Tribonacci number T_11) and compute cyclic BWT
    T = s.encode()
    n = len(T)
    rots = sorted(range(n), key=lambda i: T[i:] + T[:i])
    L = bytes(T[(i - 1) % n] for i in rots)
    runs = 1 + sum(1 for i in range(1, n) if L[i] != L[i - 1])
    h1 = 0
    i = 0
    while i < n:
        j = i + 1
        while j < n and L[j] == L[i]:
            j += 1
        if j - i == 1:
            h1 += 1
        i = j
    p_k = factor_complexity(T, [1, 2, 4, 8, 12, 16])
    print("=== N-4 TOY: Tribonacci Pisot word ===")
    print(f"  n={n}, BWT runs r={runs}, h1={h1}")
    print(f"  factor complexity p(k): {p_k} (theory: 2k+1 for infinite Tribonacci)")


def main() -> None:
    tribonacci_toy()
    blocks = load_admitted_blocks()

    print("\n=== N-4 VIABILITY: Factor complexity p(k) on Full T vs Stream N (no GUID, no SHA1) ===")
    ks = [1, 2, 4, 8, 12, 16]
    for fname, p, L in blocks:
        lf = compute_lf(L)
        T, row_of_pos = invert_bwt(L, p, lf)
        T_no_guid = GUID_RE.sub(b"\x01", T)
        T_no_gs = SHA1_RE.sub(b"\x02", T_no_guid)
        pf = factor_complexity(T, ks)
        pn1 = factor_complexity(T_no_guid, ks)
        pn2 = factor_complexity(T_no_gs, ks)
        print(f"  {fname} (len T={len(T)}, T_noGUID={len(T_no_guid)}, T_noGUID_noSHA1={len(T_no_gs)}):")
        print(f"    Full T:          " + ", ".join(f"p({k})={pf[k]:,}" for k in ks))
        print(f"    No-GUID (N1):    " + ", ".join(f"p({k})={pn1[k]:,}" for k in ks) + f"  [p(16)/16 = {pn1[16]/16:,.1f}]")
        print(f"    No-GUID/SHA1(N2):" + ", ".join(f"p({k})={pn2[k]:,}" for k in ks) + f"  [p(16)/16 = {pn2[16]/16:,.1f}]")

    print("\n=== N-5 VIABILITY: Veech / SL(2,Z) cylinder decompositions on 6 admitted blocks ===")
    for fname, p, L in blocks:
        n = len(L)
        lf = compute_lf(L)
        # Directions:
        # (1, 1): u o LF (i -> (LF[i] + 1) % n)
        # (1,-1): u^-1 o LF (i -> (LF[i] - 1) % n)
        # (2, 1): u o LF^2
        # (1, 2): u^2 o LF
        d11 = [(lf[i] + 1) % n for i in range(n)]
        d1m1 = [(lf[i] - 1) % n for i in range(n)]
        d21 = [(lf[lf[i]] + 1) % n for i in range(n)]
        d12 = [(lf[i] + 2) % n for i in range(n)]

        c11 = cycle_lengths(d11)
        c1m1 = cycle_lengths(d1m1)
        c21 = cycle_lengths(d21)
        c12 = cycle_lengths(d12)

        # Verify Theorem V1: translation pieces of d11 and d1m1 have IDENTICAL count & h1 as LF
        def piece_stats(perm: list[int]) -> tuple[int, int]:
            pieces = 0
            w1 = 0
            i = 0
            while i < n:
                j = i + 1
                while j < n and (perm[j] - perm[j - 1]) % n == 1:
                    j += 1
                pieces += 1
                if j - i == 1:
                    w1 += 1
                i = j
            return pieces, w1

        r_lf, h1_lf = piece_stats(lf)
        r_11, h1_11 = piece_stats(d11)
        r_1m1, h1_1m1 = piece_stats(d1m1)
        assert (r_lf, h1_lf) == (r_11, h1_11) == (r_1m1, h1_1m1), "Theorem V1 mismatch!"

        print(
            f"  {fname}: pieces/h1 invariant across (1,0),(1,1),(1,-1) = ({r_lf}, {h1_lf}); "
            f"cyls (1,1)={len(c11)} (top3={c11[:3]}), "
            f"(1,-1)={len(c1m1)} (top3={c1m1[:3]}), "
            f"(2,1)={len(c21)}, (1,2)={len(c12)}, "
            f"sym((1,1)==(1,-1))={c11 == c1m1}"
        )

    print("\n=== N-6 MORSE & N-7 AHT: Functional-graph w1 neighborhoods & AHT self-overlaps ===")
    for fname, p, L in blocks:
        n = len(L)
        lf = compute_lf(L)
        psi = [0] * n
        for i, j in enumerate(lf):
            psi[j] = i

        runs = []
        is_w1 = bytearray(n)
        is_thick = bytearray(n)
        i = 0
        while i < n:
            j = i + 1
            b = L[i]
            while j < n and L[j] == b:
                j += 1
            w = j - i
            runs.append((i, j, b))
            if w == 1:
                is_w1[i] = 1
            elif w > 16:
                for k in range(i, j):
                    is_thick[k] = 1
            i = j

        # N-6: classify each width-1 run by (predecessor is w1?, successor is w1?)
        morse_cls = Counter()
        for a, b, _ in runs:
            if b - a == 1:
                prev_w1 = bool(is_w1[psi[a]])
                next_w1 = bool(is_w1[lf[a]])
                morse_cls[(prev_w1, next_w1)] += 1
        h1 = sum(morse_cls.values())

        # N-7: AHT self-overlapping runs (R_i intersects LF(R_i)) and contraction quotient q
        overlap_runs = 0
        overlap_rows = 0
        max_q = 0
        q_ge_2_runs = 0
        fixed_pts = 0
        for a, b, _ in runs:
            w = b - a
            fa = lf[a]
            fb = fa + w
            # overlap of [a, b) and [fa, fb)
            lo = max(a, fa)
            hi = min(b, fb)
            if lo < hi:
                overlap_runs += 1
                overlap_rows += w
                shift = abs(fa - a)
                if shift == 0:
                    fixed_pts += w
                else:
                    q = w // shift
                    if q > max_q:
                        max_q = q
                    if q >= 2:
                        q_ge_2_runs += 1

        print(
            f"  {fname}: r={len(runs)}, h1={h1} ({100*h1/len(runs):.1f}% of pairings) | "
            f"Morse w1-chains: iso(F,F)={100*morse_cls[(False,False)]/h1:.1f}%, "
            f"entry(F,T)={100*morse_cls[(False,True)]/h1:.1f}%, "
            f"exit(T,F)={100*morse_cls[(True,False)]/h1:.1f}%, "
            f"mid(T,T)={100*morse_cls[(True,True)]/h1:.1f}% | "
            f"AHT self-overlap runs={overlap_runs} ({overlap_rows} rows, "
            f"q>=2 runs={q_ge_2_runs}, max_q={max_q}, fixed={fixed_pts})"
        )

    print("\n=== COPY-SECTOR / REPETITION CEILING ON NON-GUID WIDTH-1 ROWS (for Cycle 2 leads) ===")
    for fname, p, L in blocks:
        n = len(L)
        lf = compute_lf(L)
        T, row_of_pos = invert_bwt(L, p, lf)
        is_w1 = bytearray(n)
        i = 0
        while i < n:
            j = i + 1
            while j < n and L[j] == L[i]:
                j += 1
            if j - i == 1:
                is_w1[i] = 1
            i = j
        is_guid = bytearray(n)
        for mobj in GUID_RE.finditer(T):
            for k in range(mobj.start(), mobj.end()):
                is_guid[k] = 1
        # Check for each non-GUID width-1 text position whether its 8-byte or 16-byte window repeats in T
        for W in (8, 16):
            cnt = Counter(T[i : i + W] for i in range(n - W + 1))
            non_guid_w1 = 0
            rep_non_guid_w1 = 0
            for pos in range(n - W + 1):
                if is_w1[row_of_pos[pos]] and not is_guid[pos]:
                    non_guid_w1 += 1
                    if cnt[T[pos : pos + W]] >= 2:
                        rep_non_guid_w1 += 1
            print(
                f"  {fname} (window W={W}): non-GUID w1={non_guid_w1:,}, "
                f"in repeated W-gram={rep_non_guid_w1:,} ({100*rep_non_guid_w1/non_guid_w1:.1f}%)"
            )


if __name__ == "__main__":
    main()
