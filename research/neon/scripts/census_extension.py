#!/usr/bin/env python3
"""Self-contained 232-block census extension of N-0, M-4 (incl. N-18), and M-8 (N-22).

Input:
  A directory of BWT blocks in the `data/admitted-bwt` format:
    - `manifest.json` with `{"blocks": [{"file": "block-0.L", "primary_index": 264045, ...}, ...]}`
    - raw `.L` files (one byte per BWT row, before RLE1).

Output:
  Deterministic TSV (header + one row per block in manifest order) to stdout or `-o OUT.tsv`,
  reporting for each block:
    - Block geometry:       block, n, p, r, m16
    - N-0 / N-2 (sigma_hex): h1, h1R, h1N, h1R_share, h1N_share, pglue_w1_switches, all_switches
    - N-8 (3-part novelty): novel_hex, repeated_hex, non_hex, novel_hex_share, repeated_hex_share,
                            non_hex_share, novel_hex6, repeated_hex6, short_hex_1_5,
                            novel_hex6_share, repeated_hex6_share, short_hex_1_5_share,
                            rep_sha1_occ, rep_sha1_w1, nov_sha1_occ
    - M-4 / N-9 & N-18:     valid_flips_256, f_disj, u_all_flips, u_all_flips_j1,
                            hex_w1_ep_cov, nonhex_w1_ep_cov,
                            nu_q01, nu_q05, nu_q10, nu_q25, nu_q50, nu_q75, nu_q90,
                            nu_hex_p10, nu_hex_p50, nu_hex_p90,
                            nu_nonhex_p10, nu_nonhex_p50, nu_nonhex_p90
    - M-8 / N-15 (i9 H):    d_l2, d_l2_over_m16, d_l2_over_h1, d_l1d, d_l3_cold, d_l3_cores_10w

Usage:
  python3 research/neon/scripts/census_extension.py /path/to/bwt_dir [-o out.tsv] [--jobs 10]
  python3 research/neon/scripts/census_extension.py --verify-admitted [/path/to/admitted_dir]
"""

from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import math
import re
import subprocess
import sys
import tempfile
from collections import Counter, OrderedDict
from pathlib import Path

HEX_BYTES = frozenset(b"0123456789abcdefABCDEF")
HEX_TOK_ANY = re.compile(rb"[0-9a-fA-F]+")
HEX_TOK_6 = re.compile(rb"[0-9a-fA-F]{6,}")
SHA1_RE = re.compile(rb"(?<![0-9a-fA-F])[0-9a-fA-F]{40}(?![0-9a-fA-F])")

SPAN_CAP = 256
LIMIT_OPEN = 16

TSV_COLUMNS = [
    "block",
    "n",
    "p",
    "r",
    "m16",
    "h1",
    "h1R",
    "h1N",
    "h1R_share",
    "h1N_share",
    "pglue_w1_switches",
    "all_switches",
    "novel_hex",
    "repeated_hex",
    "non_hex",
    "novel_hex_share",
    "repeated_hex_share",
    "non_hex_share",
    "novel_hex6",
    "repeated_hex6",
    "short_hex_1_5",
    "novel_hex6_share",
    "repeated_hex6_share",
    "short_hex_1_5_share",
    "rep_sha1_occ",
    "rep_sha1_w1",
    "nov_sha1_occ",
    "valid_flips_256",
    "f_disj",
    "u_all_flips",
    "u_all_flips_j1",
    "hex_w1_ep_cov",
    "nonhex_w1_ep_cov",
    "nu_q01",
    "nu_q05",
    "nu_q10",
    "nu_q25",
    "nu_q50",
    "nu_q75",
    "nu_q90",
    "nu_hex_p10",
    "nu_hex_p50",
    "nu_hex_p90",
    "nu_nonhex_p10",
    "nu_nonhex_p50",
    "nu_nonhex_p90",
    "d_l2",
    "d_l2_over_m16",
    "d_l2_over_h1",
    "d_l1d",
    "d_l3_cold",
    "d_l3_cores_10w",
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


def orbit_maps(L: bytes, p: int, lf: list[int]) -> tuple[bytes, list[int], list[int]]:
    n = len(L)
    lf_pos = [-1] * n
    x = p
    for step in range(n):
        if lf_pos[x] != -1:
            raise ValueError(f"Column is not a single cycle (revisited row {x} at step {step})")
        lf_pos[x] = step
        x = lf[x]
    if x != p:
        raise ValueError("Column LF walk did not return to primary_index after n steps")

    T = bytearray(n)
    row_of_pos = [0] * n
    for row in range(n):
        j = (n - 1 - lf_pos[row]) % n
        T[j] = L[row]
        row_of_pos[j] = row
    return bytes(T), row_of_pos, lf_pos


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


def process_one_block(task: tuple[str, Path, int, int | None, int | None, str | None]) -> dict[str, object]:
    fname, fpath, p, expected_n, expected_r, expected_sha = task
    L = fpath.read_bytes()
    n = len(L)
    if expected_n is not None and n != expected_n:
        raise ValueError(f"{fname}: len(L)={n} != manifest n={expected_n}")
    if expected_sha is not None:
        actual_sha = hashlib.sha256(L).hexdigest()
        if actual_sha != expected_sha:
            raise ValueError(f"{fname}: sha256={actual_sha} != manifest sha256={expected_sha}")
    if not (0 <= p < n):
        raise ValueError(f"{fname}: primary_index={p} out of range [0, {n})")

    # 1. Run scan: r, h1, h1R, h1N, is_w1, is_retained (LIMIT=16)
    is_w1 = bytearray(n)
    is_retained = bytearray(n)
    r = 0
    h1 = 0
    h1R = 0
    i = 0
    while i < n:
        j = i + 1
        b = L[i]
        while j < n and L[j] == b:
            j += 1
        w = j - i
        r += 1
        if w == 1:
            h1 += 1
            is_w1[i] = 1
            if b in HEX_BYTES:
                h1R += 1
        if w <= LIMIT_OPEN:
            for k in range(i, j):
                is_retained[k] = 1
        elif i <= p < j:
            is_retained[p] = 1
        i = j

    if expected_r is not None and r != expected_r:
        raise ValueError(f"{fname}: r={r} != manifest r={expected_r}")
    h1N = h1 - h1R

    # 2. LF & Orbit maps
    lf = compute_lf(L)
    T, row_of_pos, lf_pos = orbit_maps(L, p, lf)

    # 3. N-2 Pglue boundary switch counts
    w1_stream_seq: list[int] = []
    all_switches = 0
    prev_r = (is_w1[row_of_pos[0]] == 1) and (L[row_of_pos[0]] in HEX_BYTES)
    for pos in range(n):
        row = row_of_pos[pos]
        cur_r = (is_w1[row] == 1) and (L[row] in HEX_BYTES)
        if pos > 0 and cur_r != prev_r:
            all_switches += 1
        prev_r = cur_r
        if is_w1[row]:
            w1_stream_seq.append(1 if (L[row] in HEX_BYTES) else 0)
    first_r = (is_w1[row_of_pos[0]] == 1) and (L[row_of_pos[0]] in HEX_BYTES)
    if prev_r != first_r:
        all_switches += 1
    pglue_w1_switches = (
        sum(
            1
            for k in range(len(w1_stream_seq))
            if w1_stream_seq[k] != w1_stream_seq[(k + 1) % len(w1_stream_seq)]
        )
        if w1_stream_seq
        else 0
    )

    # 4. N-8 Amended 3-part N-0 (Def1: [0-9a-fA-F]+; Def2: [0-9a-fA-F]{6,}) & M-7 SHA1 check
    def split_hex_novelty(regex: re.Pattern[bytes]) -> tuple[int, int, int, int]:
        matches = list(regex.finditer(T))
        tok_counts = Counter(mobj.group(0) for mobj in matches)
        t_tag = bytearray(n)
        for mobj in matches:
            code = 1 if tok_counts[mobj.group(0)] == 1 else 2
            for k in range(mobj.start(), mobj.end()):
                t_tag[k] = code
        nov = rep = short_h = non_h = 0
        for pos in range(n):
            row = row_of_pos[pos]
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
        return nov, rep, short_h, non_h

    nov1, rep1, short1, non1 = split_hex_novelty(HEX_TOK_ANY)
    assert short1 == 0 and nov1 + rep1 + non1 == h1
    nov6, rep6, short6, non6 = split_hex_novelty(HEX_TOK_6)
    assert non6 == non1 and nov6 + rep6 + short6 + non6 == h1

    sha_matches = list(SHA1_RE.finditer(T))
    sha_cnt = Counter(mobj.group(0) for mobj in sha_matches)
    rep_sha1_occ = rep_sha1_w1 = nov_sha1_occ = 0
    for mobj in sha_matches:
        w1_in = sum(1 for k in range(mobj.start(), mobj.end()) if is_w1[row_of_pos[k]])
        if sha_cnt[mobj.group(0)] >= 2:
            rep_sha1_occ += 1
            rep_sha1_w1 += w1_in
        else:
            nov_sha1_occ += 1

    # 5. M-4 (N-9) & N-18: Valid Type-A/B flips (SPAN_CAP = 256), F_disj, and nu_hat quantiles
    last = [-1] * 256
    prev_occ = [-1] * n
    next_occ = [n] * n
    for idx, b in enumerate(L):
        pr = last[b]
        prev_occ[idx] = pr
        if pr != -1:
            next_occ[pr] = idx
        last[b] = idx

    is_endpoint = bytearray(n)
    flips: list[tuple[int, int, int, int]] = []
    min_t1 = n
    max_t3 = -1

    # Type B
    for m_idx in range(n):
        j = next_occ[m_idx]
        if j >= n or j - m_idx >= SPAN_CAP:
            continue
        lo = max(prev_occ[m_idx] + 1, j - SPAN_CAP)
        pm = lf_pos[m_idx]
        pj = lf_pos[j]
        if pm < pj:
            for idx in range(lo, m_idx):
                if next_occ[idx] > j:
                    pi = lf_pos[idx]
                    if pi < pm or pi > pj:
                        is_endpoint[idx] = 1
                        is_endpoint[j] = 1
                        t1, t3 = (pi, pj) if pi < pm else (pm, pi)
                        flips.append((j, idx, t1, t3))
                        if t1 < min_t1:
                            min_t1 = t1
                        if t3 > max_t3:
                            max_t3 = t3
        else:
            for idx in range(lo, m_idx):
                if next_occ[idx] > j:
                    pi = lf_pos[idx]
                    if pj < pi < pm:
                        is_endpoint[idx] = 1
                        is_endpoint[j] = 1
                        flips.append((j, idx, pj, pm))
                        if pj < min_t1:
                            min_t1 = pj
                        if pm > max_t3:
                            max_t3 = pm

    # Type A
    for idx in range(n):
        m_idx = next_occ[idx]
        if m_idx >= n or m_idx - idx >= SPAN_CAP:
            continue
        hi = min(next_occ[m_idx], idx + SPAN_CAP + 1)
        pi = lf_pos[idx]
        pm = lf_pos[m_idx]
        if pi < pm:
            for j in range(m_idx + 1, hi):
                if prev_occ[j] < idx:
                    pj = lf_pos[j]
                    if pi < pj < pm:
                        is_endpoint[idx] = 1
                        is_endpoint[j] = 1
                        flips.append((j, idx, pi, pm))
                        if pi < min_t1:
                            min_t1 = pi
                        if pm > max_t3:
                            max_t3 = pm
        else:
            for j in range(m_idx + 1, hi):
                if prev_occ[j] < idx:
                    pj = lf_pos[j]
                    if pj > pi or pj < pm:
                        is_endpoint[idx] = 1
                        is_endpoint[j] = 1
                        t1, t3 = (pm, pj) if pj > pi else (pj, pi)
                        flips.append((j, idx, t1, t3))
                        if t1 < min_t1:
                            min_t1 = t1
                        if t3 > max_t3:
                            max_t3 = t3

    h1_hex_ep = sum(1 for idx in range(n) if is_w1[idx] and L[idx] in HEX_BYTES and is_endpoint[idx])
    h1_non_ep = sum(1 for idx in range(n) if is_w1[idx] and L[idx] not in HEX_BYTES and is_endpoint[idx])

    flips.sort()
    disj_flips: list[tuple[int, int, int, int]] = []
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

    if flips:
        u_all_flips = (min_t1 + 1) + (n - 1 - max_t3)
        u_all_flips_j1 = sum(
            1 for idx in range(n) if is_w1[idx] and (lf_pos[idx] <= min_t1 or lf_pos[idx] > max_t3)
        )
    else:
        u_all_flips = n
        u_all_flips_j1 = h1

    def q_budget(arr: list[int], pct: float) -> int:
        if not arr:
            return 0
        return arr[min(len(arr) - 1, int(math.floor(pct * len(arr))))]

    def p_decile(arr: list[int], num: int) -> int:
        if not arr:
            return 0
        return arr[(num * len(arr)) // 10]

    # 6. M-8 (N-15): Exact i9-10900K LRU cache-miss simulation on Open emit_bidirectional
    rank_of = [-1] * n
    m16 = 0
    for idx in range(n):
        if is_retained[idx]:
            rank_of[idx] = m16
            m16 += 1

    lf_ret: list[int] = []
    x = p
    for _ in range(n):
        if is_retained[x]:
            lf_ret.append(rank_of[x])
        x = lf[x]
    assert x == p and len(lf_ret) == m16
    psi_ret = [lf_ret[0]] + list(reversed(lf_ret[1:]))

    access_lines: list[int] = []
    lo = 0
    hi = m16 - 1
    while lo <= hi:
        access_lines.append(psi_ret[lo] >> 4)
        if lo == hi:
            break
        access_lines.append(psi_ret[hi] >> 4)
        lo += 1
        hi -= 1

    d_l1d = lru_misses(access_lines, 512)
    d_l2 = lru_misses(access_lines, 4096)
    d_l3_cold = lru_misses(access_lines, 32768)

    cache_co: OrderedDict[int, None] = OrderedDict()
    d_l3_cores_10w = 0
    out_line_lo = 1_000_000
    out_line_hi = 1_000_000 + (n // 64)
    bytes_per_step = n / m16 if m16 else 0.0
    acc_lo = acc_hi = 0.0
    for step_idx, eline in enumerate(access_lines):
        if eline in cache_co:
            cache_co.move_to_end(eline)
        else:
            d_l3_cores_10w += 1
            cache_co[eline] = None
            if len(cache_co) > 32768:
                cache_co.popitem(last=False)
        if step_idx % 2 == 0:
            acc_lo += bytes_per_step
            while acc_lo >= 64.0:
                acc_lo -= 64.0
                cache_co[out_line_lo] = None
                cache_co.move_to_end(out_line_lo)
                out_line_lo += 1
                if len(cache_co) > 32768:
                    cache_co.popitem(last=False)
        else:
            acc_hi += bytes_per_step
            while acc_hi >= 64.0:
                acc_hi -= 64.0
                cache_co[out_line_hi] = None
                cache_co.move_to_end(out_line_hi)
                out_line_hi -= 1
                if len(cache_co) > 32768:
                    cache_co.popitem(last=False)

    h1_denom = h1 if h1 > 0 else 1
    m16_denom = m16 if m16 > 0 else 1

    return {
        "block": fname,
        "n": n,
        "p": p,
        "r": r,
        "m16": m16,
        "h1": h1,
        "h1R": h1R,
        "h1N": h1N,
        "h1R_share": f"{h1R / h1_denom:.6f}",
        "h1N_share": f"{h1N / h1_denom:.6f}",
        "pglue_w1_switches": pglue_w1_switches,
        "all_switches": all_switches,
        "novel_hex": nov1,
        "repeated_hex": rep1,
        "non_hex": non1,
        "novel_hex_share": f"{nov1 / h1_denom:.6f}",
        "repeated_hex_share": f"{rep1 / h1_denom:.6f}",
        "non_hex_share": f"{non1 / h1_denom:.6f}",
        "novel_hex6": nov6,
        "repeated_hex6": rep6,
        "short_hex_1_5": short6,
        "novel_hex6_share": f"{nov6 / h1_denom:.6f}",
        "repeated_hex6_share": f"{rep6 / h1_denom:.6f}",
        "short_hex_1_5_share": f"{short6 / h1_denom:.6f}",
        "rep_sha1_occ": rep_sha1_occ,
        "rep_sha1_w1": rep_sha1_w1,
        "nov_sha1_occ": nov_sha1_occ,
        "valid_flips_256": len(flips),
        "f_disj": len(disj_flips),
        "u_all_flips": u_all_flips,
        "u_all_flips_j1": u_all_flips_j1,
        "hex_w1_ep_cov": f"{h1_hex_ep / (h1R if h1R else 1):.6f}",
        "nonhex_w1_ep_cov": f"{h1_non_ep / (h1N if h1N else 1):.6f}",
        "nu_q01": q_budget(nu_j1, 0.01),
        "nu_q05": q_budget(nu_j1, 0.05),
        "nu_q10": q_budget(nu_j1, 0.10),
        "nu_q25": q_budget(nu_j1, 0.25),
        "nu_q50": q_budget(nu_j1, 0.50),
        "nu_q75": q_budget(nu_j1, 0.75),
        "nu_q90": q_budget(nu_j1, 0.90),
        "nu_hex_p10": p_decile(nu_hex, 1),
        "nu_hex_p50": p_decile(nu_hex, 5),
        "nu_hex_p90": p_decile(nu_hex, 9),
        "nu_nonhex_p10": p_decile(nu_non, 1),
        "nu_nonhex_p50": p_decile(nu_non, 5),
        "nu_nonhex_p90": p_decile(nu_non, 9),
        "d_l2": d_l2,
        "d_l2_over_m16": f"{d_l2 / m16_denom:.6f}",
        "d_l2_over_h1": f"{d_l2 / h1_denom:.6f}",
        "d_l1d": d_l1d,
        "d_l3_cold": d_l3_cold,
        "d_l3_cores_10w": d_l3_cores_10w,
    }


def load_tasks_from_dir(bwt_dir: Path) -> list[tuple[str, Path, int, int | None, int | None, str | None]]:
    manifest_path = bwt_dir / "manifest.json"
    if not manifest_path.exists():
        raise FileNotFoundError(f"Missing manifest.json in {bwt_dir}")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    blocks = manifest.get("blocks")
    if not isinstance(blocks, list):
        raise ValueError(f"Expected 'blocks' list in {manifest_path}")

    tasks = []
    for entry in blocks:
        fname = entry["file"]
        p_val = entry.get("primary_index")
        if p_val is None:
            p_val = entry.get("orig_ptr", entry.get("p"))
        if p_val is None:
            raise KeyError(f"Entry {fname} missing primary_index / orig_ptr / p")
        fpath = bwt_dir / fname
        if not fpath.exists():
            raise FileNotFoundError(f"Missing block file {fpath}")
        tasks.append(
            (
                fname,
                fpath,
                int(p_val),
                int(entry["n"]) if "n" in entry else None,
                int(entry["r"]) if "r" in entry else None,
                str(entry["sha256"]) if "sha256" in entry else None,
            )
        )
    return tasks


def run_directory(bwt_dir: Path, jobs: int = 1) -> list[dict[str, object]]:
    tasks = load_tasks_from_dir(bwt_dir)
    if jobs <= 1 or len(tasks) <= 1:
        return [process_one_block(t) for t in tasks]
    with concurrent.futures.ProcessPoolExecutor(max_workers=jobs) as ex:
        return list(ex.map(process_one_block, tasks))


def format_tsv(rows: list[dict[str, object]]) -> str:
    lines = ["\t".join(TSV_COLUMNS)]
    for row in rows:
        lines.append("\t".join(str(row[col]) for col in TSV_COLUMNS))
    return "\n".join(lines) + "\n"


# Published 6-block ground truth across N-2/N-0, N-8, N-9, N-18, and N-15
PUBLISHED_SIX_BLOCKS: dict[str, dict[str, object]] = {
    "block-0.L": {
        "n": 899931, "p": 264045, "r": 51184, "m16": 89874,
        "h1": 35116, "h1R": 29638, "h1N": 5478, "pglue_w1_switches": 8948,
        "novel_hex": 23456, "repeated_hex": 6182, "non_hex": 5478,
        "novel_hex6": 15781, "repeated_hex6": 3865, "short_hex_1_5": 9992,
        "rep_sha1_occ": 794, "rep_sha1_w1": 3, "nov_sha1_occ": 47,
        "valid_flips_256": 242940, "f_disj": 16206, "u_all_flips": 354, "u_all_flips_j1": 2,
        "nu_q01": 206, "nu_q05": 778, "nu_q10": 1379, "nu_q25": 3397, "nu_q50": 6903, "nu_q75": 8845, "nu_q90": 9892,
        "nu_hex_p10": 1372, "nu_hex_p50": 6903, "nu_hex_p90": 9889,
        "nu_nonhex_p10": 1435, "nu_nonhex_p50": 6918, "nu_nonhex_p90": 9912,
        "d_l1d": 58547, "d_l2": 14630, "d_l3_cold": 5618, "d_l3_cores_10w": 5618,
    },
    "block-2.L": {
        "n": 900000, "p": 320807, "r": 54532, "m16": 112070,
        "h1": 34262, "h1R": 28254, "h1N": 6008, "pglue_w1_switches": 8746,
        "novel_hex": 22357, "repeated_hex": 5897, "non_hex": 6008,
        "novel_hex6": 15397, "repeated_hex6": 3101, "short_hex_1_5": 9756,
        "rep_sha1_occ": 765, "rep_sha1_w1": 0, "nov_sha1_occ": 71,
        "valid_flips_256": 276262, "f_disj": 20168, "u_all_flips": 116, "u_all_flips_j1": 1,
        "nu_q01": 235, "nu_q05": 1159, "nu_q10": 2034, "nu_q25": 5630, "nu_q50": 8761, "nu_q75": 10433, "nu_q90": 10980,
        "nu_hex_p10": 1959, "nu_hex_p50": 8753, "nu_hex_p90": 10982,
        "nu_nonhex_p10": 2910, "nu_nonhex_p50": 8984, "nu_nonhex_p90": 10974,
        "d_l1d": 72721, "d_l2": 27293, "d_l3_cold": 7005, "d_l3_cores_10w": 7005,
    },
    "block-85.L": {
        "n": 899935, "p": 325088, "r": 79375, "m16": 156404,
        "h1": 55504, "h1R": 45874, "h1N": 9630, "pglue_w1_switches": 13798,
        "novel_hex": 35546, "repeated_hex": 10328, "non_hex": 9630,
        "novel_hex6": 23673, "repeated_hex6": 4766, "short_hex_1_5": 17435,
        "rep_sha1_occ": 1276, "rep_sha1_w1": 0, "nov_sha1_occ": 22,
        "valid_flips_256": 446389, "f_disj": 26491, "u_all_flips": 1, "u_all_flips_j1": 1,
        "nu_q01": 315, "nu_q05": 1589, "nu_q10": 3195, "nu_q25": 6187, "nu_q50": 9988, "nu_q75": 12402, "nu_q90": 13348,
        "nu_hex_p10": 3206, "nu_hex_p50": 10054, "nu_hex_p90": 13348,
        "nu_nonhex_p10": 3142, "nu_nonhex_p50": 9645, "nu_nonhex_p90": 13345,
        "d_l1d": 90017, "d_l2": 42608, "d_l3_cold": 9776, "d_l3_cores_10w": 9776,
    },
    "block-143.L": {
        "n": 899869, "p": 802868, "r": 108335, "m16": 196841,
        "h1": 72578, "h1R": 58922, "h1N": 13656, "pglue_w1_switches": 18100,
        "novel_hex": 44509, "repeated_hex": 14413, "non_hex": 13656,
        "novel_hex6": 31491, "repeated_hex6": 5322, "short_hex_1_5": 22109,
        "rep_sha1_occ": 356, "rep_sha1_w1": 0, "nov_sha1_occ": 167,
        "valid_flips_256": 522707, "f_disj": 33011, "u_all_flips": 30, "u_all_flips_j1": 1,
        "nu_q01": 482, "nu_q05": 2097, "nu_q10": 4412, "nu_q25": 8971, "nu_q50": 14120, "nu_q75": 17137, "nu_q90": 17979,
        "nu_hex_p10": 4478, "nu_hex_p50": 14082, "nu_hex_p90": 17977,
        "nu_nonhex_p10": 4278, "nu_nonhex_p50": 14314, "nu_nonhex_p90": 17991,
        "d_l1d": 151662, "d_l2": 86903, "d_l3_cold": 12303, "d_l3_cores_10w": 12303,
    },
    "block-210.L": {
        "n": 899959, "p": 791429, "r": 128732, "m16": 204281,
        "h1": 97054, "h1R": 82630, "h1N": 14424, "pglue_w1_switches": 21750,
        "novel_hex": 66164, "repeated_hex": 16466, "non_hex": 14424,
        "novel_hex6": 46579, "repeated_hex6": 6622, "short_hex_1_5": 29429,
        "rep_sha1_occ": 207, "rep_sha1_w1": 0, "nov_sha1_occ": 132,
        "valid_flips_256": 606726, "f_disj": 33407, "u_all_flips": 4, "u_all_flips_j1": 1,
        "nu_q01": 652, "nu_q05": 2107, "nu_q10": 3651, "nu_q25": 8304, "nu_q50": 12974, "nu_q75": 14011, "nu_q90": 14631,
        "nu_hex_p10": 3648, "nu_hex_p50": 12972, "nu_hex_p90": 14635,
        "nu_nonhex_p10": 3695, "nu_nonhex_p50": 13009, "nu_nonhex_p90": 14605,
        "d_l1d": 159851, "d_l2": 95695, "d_l3_cold": 12768, "d_l3_cores_10w": 12768,
    },
    "block-207.L": {
        "n": 899998, "p": 205180, "r": 87111, "m16": 135921,
        "h1": 62407, "h1R": 49791, "h1N": 12616, "pglue_w1_switches": 15012,
        "novel_hex": 39311, "repeated_hex": 10480, "non_hex": 12616,
        "novel_hex6": 27284, "repeated_hex6": 3688, "short_hex_1_5": 18819,
        "rep_sha1_occ": 247, "rep_sha1_w1": 0, "nov_sha1_occ": 144,
        "valid_flips_256": 403035, "f_disj": 24062, "u_all_flips": 65, "u_all_flips_j1": 1,
        "nu_q01": 403, "nu_q05": 1460, "nu_q10": 2595, "nu_q25": 5625, "nu_q50": 9684, "nu_q75": 12479, "nu_q90": 13139,
        "nu_hex_p10": 2543, "nu_hex_p50": 9841, "nu_hex_p90": 13142,
        "nu_nonhex_p10": 2901, "nu_nonhex_p50": 8771, "nu_nonhex_p90": 13101,
        "d_l1d": 97343, "d_l2": 41813, "d_l3_cold": 8496, "d_l3_cores_10w": 8496,
    },
}


def verify_against_published(rows: list[dict[str, object]]) -> None:
    if len(rows) != 6:
        raise AssertionError(f"Expected 6 admitted blocks, got {len(rows)}")
    checked_fields = 0
    for row in rows:
        bname = str(row["block"])
        if bname not in PUBLISHED_SIX_BLOCKS:
            raise AssertionError(f"Unexpected block {bname}")
        exp = PUBLISHED_SIX_BLOCKS[bname]
        for k, v_exp in exp.items():
            v_got = row[k]
            if v_got != v_exp:
                raise AssertionError(f"{bname} field {k!r}: got {v_got!r} != published {v_exp!r}")
            checked_fields += 1
    sys.stderr.write(
        f"[verify-admitted] PASS: 6/6 blocks matched 100% of {checked_fields} published "
        "exact figures (N-0/N-2, N-8, N-9, N-18, N-15).\n"
    )


def materialize_admitted_from_git(dst_dir: Path) -> None:
    manifest_raw = subprocess.check_output(
        ["git", "show", "origin/data/admitted-bwt:admitted-bwt/manifest.json"]
    )
    (dst_dir / "manifest.json").write_bytes(manifest_raw)
    manifest = json.loads(manifest_raw)
    for entry in manifest["blocks"]:
        fname = entry["file"]
        blob = subprocess.check_output(
            ["git", "show", f"origin/data/admitted-bwt:admitted-bwt/{fname}"]
        )
        (dst_dir / fname).write_bytes(blob)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="232-block census extension of N-0, M-4 (incl. N-18), and M-8."
    )
    parser.add_argument(
        "bwt_dir",
        nargs="?",
        type=Path,
        help="Directory containing manifest.json and raw .L BWT block files.",
    )
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        default=None,
        help="Output TSV path (defaults to stdout).",
    )
    parser.add_argument(
        "-j",
        "--jobs",
        type=int,
        default=1,
        help="Number of parallel worker processes (default: 1).",
    )
    parser.add_argument(
        "--verify-admitted",
        action="store_true",
        help="Verify exact reproduction against all published 6-block numbers (stages origin/data/admitted-bwt if bwt_dir omitted).",
    )
    args = parser.parse_args(argv)

    if args.bwt_dir is None:
        if not args.verify_admitted:
            parser.error("the following arguments are required: bwt_dir (or pass --verify-admitted)")
        with tempfile.TemporaryDirectory(prefix="admitted_bwt_") as tmp:
            tmp_path = Path(tmp)
            materialize_admitted_from_git(tmp_path)
            rows = run_directory(tmp_path, jobs=args.jobs)
    else:
        rows = run_directory(args.bwt_dir, jobs=args.jobs)

    if args.verify_admitted:
        verify_against_published(rows)

    tsv_text = format_tsv(rows)
    if args.output is not None:
        args.output.write_text(tsv_text, encoding="utf-8")
    else:
        sys.stdout.write(tsv_text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
