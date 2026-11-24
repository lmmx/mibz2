#!/usr/bin/env python3
"""Cycle 3 supplied-capability measurements (Outside Review §5.1, §5.2, §5.3).

Runs on the 6 admitted blocks of data/admitted-bwt:
1. N-12 (§5.1): Exact acyclic LZ77 positional copy plans (via O(n) SA/LCP/LPF),
   byte-exact reconstruction check, q_copy = z_copy + z_lit, coverage of h1 and
   m16 thin rows, copy distances (D_H miss proxy), and supplied-capability payoff.
2. N-13 (§5.2): Exact differential suffix array DSA[i] = SA[i] - SA[i-1]:
   distinct values, equal-DSA runs, LF-invariance breaks, H0 entropy bits,
   and numerical evaluation of Kempa Theorem 5.1 workspace/time at optimal tau.
3. N-14 (§5.3): Durand ordered return-word hierarchies on raw T and on
   template-with-holes T_skel: directive size s, hole count e_slots, and
   paid exception width-1 bytes e_w1 vs h1.
"""

from __future__ import annotations

import json
import math
import os
import re
import subprocess
from collections import Counter
from pathlib import Path

HEX_BYTES = set(b"0123456789abcdefABCDEF")
GUID_RE = re.compile(
    rb"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}"
)
SHA1_RE = re.compile(rb"(?<![0-9a-fA-F])[0-9a-fA-F]{40}(?![0-9a-fA-F])")
ENTITY_RE = re.compile(rb"[QPL][0-9]+")
NUM4_RE = re.compile(rb"[0-9]{4,}")


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


def build_sa_isa_t(L: bytes, p: int) -> tuple[bytes, list[int], list[int], list[int]]:
    """Return (T, SA, ISA, LF).

    SA[row] = text position j such that rot(T, j) is at BWT row `row`.
    Note: L[row] = T[(SA[row] - 1) mod n], so the row emitting T[j] is ISA[(j + 1) mod n].
    """
    lf = compute_lf(L)
    n = len(L)
    # Walk LF from p: p has SA[p] = 0, LF(p) has SA = n-1, LF^k(p) has SA = (n - k) mod n.
    sa = [0] * n
    isa = [0] * n
    T = bytearray(n)
    x = p
    for k in range(n):
        pos = (n - k) % n
        sa[x] = pos
        isa[pos] = x
        # L[x] = T[(pos - 1) % n]
        T[(pos - 1) % n] = L[x]
        x = lf[x]
    assert x == p
    return bytes(T), sa, isa, lf


def compute_lcp_kasai(T: bytes, sa: list[int], isa: list[int]) -> list[int]:
    """Non-cyclic LCP[i] = lcp(T[SA[i-1]:], T[SA[i]:]) for i = 1..n-1 (LCP[0] = 0)."""
    n = len(T)
    lcp = [0] * n
    k = 0
    for j in range(n):
        rank = isa[j]
        if rank == 0:
            k = 0
            continue
        prev = sa[rank - 1]
        limit = n - (j if j > prev else prev)
        if k > limit:
            k = limit
        while k < limit and T[j + k] == T[prev + k]:
            k += 1
        lcp[rank] = k
        if k > 0:
            k -= 1
    return lcp


def compute_lpf(sa: list[int], lcp: list[int]) -> tuple[list[int], list[int]]:
    """Compute (lpf_len, lpf_src) via exact PSV/NSV monotone stacks over (SA, LCP)."""
    n = len(sa)
    lpf_len = [0] * n
    lpf_src = [-1] * n

    # Pass 1: PSV (nearest i' < i with SA[i'] < SA[i])
    # Stack holds (sa_val, min_lcp_from_that_entry_to_top)
    stack_sa: list[int] = []
    stack_lcp: list[int] = []
    for i in range(n):
        s = sa[i]
        cur_min = lcp[i]
        while stack_sa and stack_sa[-1] >= s:
            stack_sa.pop()
            v = stack_lcp.pop()
            if v < cur_min:
                cur_min = v
        if stack_sa:
            if cur_min > lpf_len[s]:
                lpf_len[s] = cur_min
                lpf_src[s] = stack_sa[-1]
        stack_sa.append(s)
        stack_lcp.append(cur_min)

    # Pass 2: NSV (nearest i' > i with SA[i'] < SA[i])
    stack_sa.clear()
    stack_lcp.clear()
    for i in range(n - 1, -1, -1):
        s = sa[i]
        cur_min = lcp[i + 1] if i + 1 < n else 0
        while stack_sa and stack_sa[-1] >= s:
            stack_sa.pop()
            v = stack_lcp.pop()
            if v < cur_min:
                cur_min = v
        if stack_sa:
            if cur_min > lpf_len[s]:
                lpf_len[s] = cur_min
                lpf_src[s] = stack_sa[-1]
        stack_sa.append(s)
        stack_lcp.append(cur_min)

    return lpf_len, lpf_src


def run_n12_lz77(
    fname: str,
    T: bytes,
    sa: list[int],
    isa: list[int],
    is_w1: bytearray,
    is_thin: bytearray,
) -> None:
    n = len(T)
    lcp = compute_lcp_kasai(T, sa, isa)
    lpf_len, lpf_src = compute_lpf(sa, lcp)

    # Emit row for T[j] is isa[(j + 1) % n]
    w1_at_pos = bytearray(n)
    thin_at_pos = bytearray(n)
    for j in range(n):
        row = isa[(j + 1) % n]
        if is_w1[row]:
            w1_at_pos[j] = 1
        if is_thin[row]:
            thin_at_pos[j] = 1
    h1 = sum(w1_at_pos)
    m16 = sum(thin_at_pos)

    print(f"\n  {fname} (n={n:,}, m16={m16:,}, h1={h1:,}):")
    for min_len in (2, 4, 8, 16):
        z_copy = 0
        z_lit = 0
        copied_bytes = 0
        w1_in_copy = 0
        w1_copy_interior = 0
        thin_in_copy = 0
        thin_copy_interior = 0
        dists = []
        recon = bytearray()
        j = 0
        while j < n:
            ln = lpf_len[j]
            sr = lpf_src[j]
            if ln >= min_len and sr >= 0 and sr < j:
                # Verify byte-exact copy and reconstruct
                # Handle overlapping copies (RLE) identically to LZ77
                z_copy += 1
                copied_bytes += ln
                dists.append(j - sr)
                for off in range(ln):
                    recon.append(recon[sr + off])
                    if w1_at_pos[j + off]:
                        w1_in_copy += 1
                        if off > 0:
                            w1_copy_interior += 1
                    if thin_at_pos[j + off]:
                        thin_in_copy += 1
                        if off > 0:
                            thin_copy_interior += 1
                j += ln
            else:
                z_lit += 1
                recon.append(T[j])
                j += 1
        assert bytes(recon) == T, f"LZ77 reconstruction failed on {fname} min_len={min_len}"
        z = z_copy + z_lit
        dists.sort()
        med_dist = dists[len(dists) // 2] if dists else 0
        # Uncovered h1 / m16 rows + copy descriptors = effective q_copy
        w1_uncov = h1 - w1_in_copy
        thin_uncov = m16 - thin_in_copy
        print(
            f"    min_len={min_len:>2}: z={z:>7,} (copy={z_copy:>6,}, lit={z_lit:>6,}; "
            f"z/h1={z/h1:.2f}, z/m16={z/m16:.2f}) | "
            f"h1_copied={100*w1_in_copy/h1:>5.1f}% (int={100*w1_copy_interior/h1:>5.1f}%, uncov={w1_uncov:>6,}) | "
            f"m16_copied={100*thin_in_copy/m16:>5.1f}% (uncov={thin_uncov:>6,}) | "
            f"med_dist={med_dist:>7,}"
        )


def shannon_h0(vals: list[int]) -> float:
    n = len(vals)
    if n == 0:
        return 0.0
    cnt = Counter(vals)
    h = 0.0
    for c in cnt.values():
        p = c / n
        h -= p * math.log2(p)
    return h


def run_n13_dsa(
    fname: str,
    L: bytes,
    sa: list[int],
    lf: list[int],
    r: int,
    is_w1: bytearray,
    is_thin: bytearray,
) -> None:
    n = len(L)
    dsa = [0] * n
    for i in range(1, n):
        dsa[i] = sa[i] - sa[i - 1]
    dsa[0] = sa[0] - sa[n - 1]

    # Exact LF-invariance breaks: DSA[LF(i)] != DSA[i]
    lf_breaks = sum(1 for i in range(n) if dsa[lf[i]] != dsa[i])
    dsa_runs = 1 + sum(1 for i in range(1, n) if dsa[i] != dsa[i - 1])

    dsa_all_dist = len(set(dsa))
    dsa_w1 = [dsa[i] for i in range(n) if is_w1[i]]
    dsa_nonhex_w1 = [dsa[i] for i in range(n) if is_w1[i] and L[i] not in HEX_BYTES]
    dsa_thin = [dsa[i] for i in range(n) if is_thin[i]]

    h1 = len(dsa_w1)
    h1n = len(dsa_nonhex_w1)
    m16 = len(dsa_thin)

    dist_w1 = len(set(dsa_w1))
    dist_nonhex_w1 = len(set(dsa_nonhex_w1))
    dist_thin = len(set(dsa_thin))

    h0_w1 = shannon_h0(dsa_w1)
    h0_nonhex = shannon_h0(dsa_nonhex_w1)

    # Kempa Thm 5.1 optimal tau = sqrt(n / r)
    tau = math.sqrt(n / r)
    kempa_ws_words = (n / tau) + (r * tau)
    kempa_time_ops = (n / tau) * (math.log2(r * tau) + math.log2(n / tau)) + (tau * tau) * r * math.log2(r * tau)

    print(
        f"  {fname}: r={r:>6,}, lf_breaks={lf_breaks:>6,} ({lf_breaks/r:.2f}r), dsa_runs={dsa_runs:>7,} | "
        f"w1: distinct={dist_w1:>6,}/{h1:>6,} ({100*dist_w1/h1:.1f}%), H0={h0_w1:.2f}b; "
        f"nonhex_w1: distinct={dist_nonhex_w1:>5,}/{h1n:>5,} ({100*dist_nonhex_w1/h1n:.1f}%), H0={h0_nonhex:.2f}b; "
        f"m16: distinct={dist_thin:>6,}/{m16:>6,} ({100*dist_thin/m16:.1f}%) | "
        f"Kempa5.1(tau={tau:.1f}): ws={kempa_ws_words/1e3:.0f}k words, ops={kempa_time_ops/1e6:.1f}M"
    )


def run_n14_durand(
    fname: str,
    T: bytes,
    isa: list[int],
    is_w1: bytearray,
) -> None:
    n = len(T)
    w1_at_pos = bytearray(n)
    for j in range(n):
        if is_w1[isa[(j + 1) % n]]:
            w1_at_pos[j] = 1
    h1 = sum(w1_at_pos)

    # H1: Raw Durand return words split on delimiter b'","' (or b',')
    delim = b","
    parts = T.split(delim)
    cnt_raw = Counter(parts)
    s_raw = len(cnt_raw)
    # Count h1 bytes inside unique (count == 1) vs repeated (count >= 2) raw return words
    w1_in_unique_raw = 0
    pos = 0
    for seg in parts:
        ln = len(seg)
        if cnt_raw[seg] == 1:
            w1_in_unique_raw += sum(w1_at_pos[pos : pos + ln])
        pos += ln + 1

    # H2: Template-with-holes return words (replace GUID, SHA1, Q/P/L-id, Num4+ by typed holes)
    hole_mask = bytearray(n)  # 1 if byte belongs to a typed hole
    hole_slots = 0
    for rx in (GUID_RE, SHA1_RE, ENTITY_RE, NUM4_RE):
        for m in rx.finditer(T):
            # Mark hole bytes not already covered by higher-priority hole
            if not any(hole_mask[m.start() : m.end()]):
                hole_slots += 1
                for k in range(m.start(), m.end()):
                    hole_mask[k] = 1

    w1_in_holes = sum(1 for j in range(n) if w1_at_pos[j] and hole_mask[j])

    # Build skeleton return words (collapsing each maximal hole run to b'\x01')
    skel_segs = []
    seg_ranges = []
    pos = 0
    for seg in parts:
        ln = len(seg)
        buf = bytearray()
        in_h = False
        for off in range(ln):
            if hole_mask[pos + off]:
                if not in_h:
                    buf.append(1)
                    in_h = True
            else:
                buf.append(seg[off])
                in_h = False
        skel_segs.append(bytes(buf))
        seg_ranges.append((pos, pos + ln))
        pos += ln + 1

    cnt_skel = Counter(skel_segs)
    s_skel = len(cnt_skel)
    # Remaining non-hole w1 bytes that STILL sit in unique (count==1) skeleton templates
    w1_in_unique_skel_nonhole = 0
    for sk, (a, b) in zip(skel_segs, seg_ranges):
        if cnt_skel[sk] == 1:
            for k in range(a, b):
                if w1_at_pos[k] and not hole_mask[k]:
                    w1_in_unique_skel_nonhole += 1

    paid_exceptions_w1 = w1_in_holes + w1_in_unique_skel_nonhole
    print(
        f"  {fname}: h1={h1:>6,} | "
        f"Raw Durand(','): N_occ={len(parts):>6,}, s_raw={s_raw:>6,}, w1_in_unique={w1_in_unique_raw:>6,} ({100*w1_in_unique_raw/h1:.1f}%) | "
        f"Template+holes: s_skel={s_skel:>5,}, e_slots={hole_slots:>6,}, "
        f"w1_in_holes={w1_in_holes:>6,} ({100*w1_in_holes/h1:.1f}%), "
        f"w1_uniq_nonhole={w1_in_unique_skel_nonhole:>6,} ({100*w1_in_unique_skel_nonhole/h1:.1f}%), "
        f"TOTAL paid e_w1={paid_exceptions_w1:>6,} ({100*paid_exceptions_w1/h1:.1f}% of h1)"
    )


def main() -> None:
    blocks = load_admitted_blocks()
    prepped = []
    for fname, p, L in blocks:
        n = len(L)
        T, sa, isa, lf = build_sa_isa_t(L, p)
        is_w1 = bytearray(n)
        is_thin = bytearray(n)
        r = 0
        i = 0
        while i < n:
            j = i + 1
            b = L[i]
            while j < n and L[j] == b:
                j += 1
            r += 1
            w = j - i
            if w == 1:
                is_w1[i] = 1
            if w <= 16:
                for k in range(i, j):
                    is_thin[k] = 1
            elif i <= p < j:
                is_thin[p] = 1
            i = j
        prepped.append((fname, L, T, sa, isa, lf, r, is_w1, is_thin))

    print("=== N-12 (§5.1): EXACT ACYCLIC LZ77 POSITIONAL COPY PLANS (BYTE-VERIFIED) ===")
    for fname, L, T, sa, isa, lf, r, is_w1, is_thin in prepped:
        run_n12_lz77(fname, T, sa, isa, is_w1, is_thin)

    print("\n=== N-13 (§5.2): COMPRESSED DSA (SA[i]-SA[i-1]) & KEMPA THM 5.1 AUDIT ===")
    for fname, L, T, sa, isa, lf, r, is_w1, is_thin in prepped:
        run_n13_dsa(fname, L, sa, lf, r, is_w1, is_thin)

    print("\n=== N-14 (§5.3): DURAND ORDERED RETURN-WORD HIERARCHIES WITH PAID EXCEPTIONS ===")
    for fname, L, T, sa, isa, lf, r, is_w1, is_thin in prepped:
        run_n14_durand(fname, T, isa, is_w1)


if __name__ == "__main__":
    main()
