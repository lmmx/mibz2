#!/usr/bin/env python3
"""Discharge MONODROMY NEEDS M-8: i9-10900K memory model H and Open dependent misses.

Machine facts (bench/moonshot/calibration.json, PROOF-PROGRAMME.md §5.10):
- CPU: Intel Core i9-10900K (Comet Lake)
- Cache line: 64 B
- L1d: 32 KiB per core (512 lines)
- L2: 256 KiB per core (4,096 lines)
- L3: 20 MiB (20,480 KiB) shared -> 2,048 KiB (32,768 lines) per worker at W = 10

Simulates Open's exact two-ended emit_bidirectional (src/open.rs:919) on the 6
admitted blocks of data/admitted-bwt:
- edges: Vec<u32> (4 bytes per retained row = 16 entries per 64B cache line)
- Two-ended walk: front = mark moving forward (Psi-return), back = before moving
  backward (LF-return), alternating edges[front] and edges[back] until they meet.
- Tracks exact LRU cache misses on edges[] at:
  * L1d (32 KiB, 512 lines)
  * L2 (256 KiB, 4,096 lines) — primary dependent-miss metric D_L2(Open, c)
  * L3/worker at W=10 (2,048 KiB, 32,768 lines) both cold-start (edges only) and
    with co-resident out[n] streaming writes (900 KiB) sharing the 2 MiB slice.
"""

from __future__ import annotations

import json
import os
import subprocess
from collections import OrderedDict
from pathlib import Path


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


def open_retained_cycle(L: bytes, p: int, limit: int = 16) -> tuple[list[int], int, int]:
    """Build exact Open retained-rank cycle (rank in [0, m), mark_rank, h1).

    Retained rows S = all rows in maximal runs of width <= limit, plus p if in a thick run.
    Returns (order_psi, mark_rank, h1) where order_psi[k] is the retained rank visited
    at step k starting from mark_rank walking forward (Psi first-return).
    """
    n = len(L)
    lf = compute_lf(L)
    is_retained = bytearray(n)
    h1 = 0
    i = 0
    while i < n:
        j = i + 1
        b = L[i]
        while j < n and L[j] == b:
            j += 1
        w = j - i
        if w == 1:
            h1 += 1
        if w <= limit:
            for k in range(i, j):
                is_retained[k] = 1
        elif i <= p < j:
            is_retained[p] = 1
        i = j

    rank_of = [-1] * n
    m = 0
    for idx in range(n):
        if is_retained[idx]:
            rank_of[idx] = m
            m += 1
    mark_rank = rank_of[p]

    # Walk LF from p to collect retained rows in LF-return order, then reverse for Psi order
    lf_ret = []
    x = p
    for _ in range(n):
        if is_retained[x]:
            lf_ret.append(rank_of[x])
        x = lf[x]
    assert x == p and len(lf_ret) == m
    # Forward (Psi) order starting at mark_rank: lf_ret[0] = mark_rank, then lf_ret[-1], lf_ret[-2], ...
    psi_ret = [lf_ret[0]] + list(reversed(lf_ret[1:]))
    return psi_ret, mark_rank, h1


def lru_misses(seq: list[int], capacity_lines: int) -> int:
    cache: OrderedDict[int, None] = OrderedDict()
    misses = 0
    for line in seq:
        if line in cache:
            cache.move_to_end(line)
        else:
            misses += 1
            cache[line] = None
            if len(cache) > capacity_lines:
                cache.popitem(last=False)
    return misses


def main() -> None:
    blocks = load_admitted_blocks()
    print("=== NEEDS M-8: i9-10900K MEMORY MODEL H & OPEN EMIT DEPENDENT MISSES ===")
    print("Machine: i9-10900K | line=64B (16 u32 edges/line) | L1d=32KiB (512L) | L2=256KiB (4096L) | L3/10W=2048KiB (32768L)")
    hdr = (
        f"{'block':<12} {'m16':>7} {'edges_KB':>8} {'lines':>6} | "
        f"{'D_L1d':>7} ({'L1%':>5}) | {'D_L2':>7} ({'L2%':>5}) | "
        f"{'D_L3_cold':>9} | {'D_L3_co':>7} ({'L3%':>5}) | {'D_L2/h1':>7}"
    )
    print(hdr)
    print("-" * len(hdr))

    for fname, p, L in blocks:
        n = len(L)
        psi_ret, mark_rank, h1 = open_retained_cycle(L, p, limit=16)
        m = len(psi_ret)
        edges_kb = (m * 4) / 1024.0
        unique_lines = (m + 15) // 16

        # Exact emit_bidirectional access order on edges[] (src/open.rs:935-960):
        # front walks psi_ret[0], psi_ret[1], ...; back walks psi_ret[m-1], psi_ret[m-2], ...
        access_lines = []
        lo = 0
        hi = m - 1
        while lo <= hi:
            access_lines.append(psi_ret[lo] >> 4)  # 16 u32 entries per 64B line
            if lo == hi:
                break
            access_lines.append(psi_ret[hi] >> 4)
            lo += 1
            hi -= 1
        assert len(access_lines) == m

        # Cold-start LRU on edges[] alone
        m_l1 = lru_misses(access_lines, 512)
        m_l2 = lru_misses(access_lines, 4096)
        m_l3_cold = lru_misses(access_lines, 32768)

        # Co-resident L3 (2,048 KiB = 32,768 lines) at W=10 where sequential out[0..n)
        # writes ~n/m bytes per emit step (distinct line tags >= 1_000_000) sharing L3.
        # Approximate steady-state L3 capacity left for edges[] alongside out[] (900 KiB = 14,063 lines)
        # + inline/spelling (~64 KiB = 1,024 lines): 32,768 - 15,087 = 17,681 lines (~1,105 KiB).
        # Because edges[] is only 359-817 KiB (5,618-12,768 lines) < 17,681 lines,
        # edges[] stays L3-resident after cold fill at W=10 (matching Record 95)!
        # Also simulate an explicit interleaved out-write stream into the 32,768-line L3:
        cache: OrderedDict[int, None] = OrderedDict()
        m_l3_co = 0
        out_line_lo = 1_000_000
        out_line_hi = 1_000_000 + (n // 64)
        bytes_per_step = n / m
        acc_lo = acc_hi = 0.0
        for step_idx, eline in enumerate(access_lines):
            if eline in cache:
                cache.move_to_end(eline)
            else:
                m_l3_co += 1
                cache[eline] = None
                if len(cache) > 32768:
                    cache.popitem(last=False)
            # Interleaved sequential output line touch every ~64 output bytes
            if step_idx % 2 == 0:
                acc_lo += bytes_per_step
                while acc_lo >= 64.0:
                    acc_lo -= 64.0
                    cache[out_line_lo] = None
                    cache.move_to_end(out_line_lo)
                    out_line_lo += 1
                    if len(cache) > 32768:
                        cache.popitem(last=False)
            else:
                acc_hi += bytes_per_step
                while acc_hi >= 64.0:
                    acc_hi -= 64.0
                    cache[out_line_hi] = None
                    cache.move_to_end(out_line_hi)
                    out_line_hi -= 1
                    if len(cache) > 32768:
                        cache.popitem(last=False)

        print(
            f"{fname:<12} {m:>7,} {edges_kb:>7.1f}K {unique_lines:>6,} | "
            f"{m_l1:>7,} ({100*m_l1/m:>4.1f}%) | {m_l2:>7,} ({100*m_l2/m:>4.1f}%) | "
            f"{m_l3_cold:>9,} | {m_l3_co:>7,} ({100*m_l3_co/m:>4.1f}%) | {m_l2/h1:>7.2f}"
        )


if __name__ == "__main__":
    main()
