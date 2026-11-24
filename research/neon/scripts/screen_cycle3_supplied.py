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


def _sais(s: list[int], alphabet_size: int) -> list[int]:
    """Exact O(n) SA-IS suffix array for integer sequence s with unique sentinel s[-1] = 0."""
    n = len(s)
    if n <= 1:
        return list(range(n))
    if n == 2:
        return [1, 0] if s[1] < s[0] else [0, 1]

    # Classify suffixes: True = S-type, False = L-type
    is_s = bytearray(n)
    is_s[n - 1] = 1
    for i in range(n - 2, -1, -1):
        if s[i] < s[i + 1]:
            is_s[i] = 1
        elif s[i] == s[i + 1]:
            is_s[i] = is_s[i + 1]

    # Bucket sizes
    counts = [0] * alphabet_size
    for c in s:
        counts[c] += 1

    def bucket_heads() -> list[int]:
        heads = [0] * alphabet_size
        acc = 0
        for c in range(alphabet_size):
            heads[c] = acc
            acc += counts[c]
        return heads

    def bucket_tails() -> list[int]:
        tails = [0] * alphabet_size
        acc = 0
        for c in range(alphabet_size):
            acc += counts[c]
            tails[c] = acc - 1
        return tails

    def induce(sa: list[int]) -> None:
        # Induce L-type from left to right
        heads = bucket_heads()
        for i in range(n):
            j = sa[i] - 1
            if j >= 0 and not is_s[j]:
                c = s[j]
                sa[heads[c]] = j
                heads[c] += 1
        # Induce S-type from right to left
        tails = bucket_tails()
        for i in range(n - 1, -1, -1):
            j = sa[i] - 1
            if j >= 0 and is_s[j]:
                c = s[j]
                sa[tails[c]] = j
                tails[c] -= 1

    # Place LMS suffixes at bucket tails
    sa = [-1] * n
    tails = bucket_tails()
    lms_pos: list[int] = []
    for i in range(1, n):
        if is_s[i] and not is_s[i - 1]:
            lms_pos.append(i)
            c = s[i]
            sa[tails[c]] = i
            tails[c] -= 1

    induce(sa)

    # Name LMS substrings
    is_lms = bytearray(n)
    for i in lms_pos:
        is_lms[i] = 1

    lms_names = [-1] * n
    cur_name = 0
    prev = -1
    for i in sa:
        if i > 0 and is_lms[i]:
            if prev != -1:
                # Compare LMS substrings starting at prev and i
                diff = False
                k = 0
                while True:
                    if s[prev + k] != s[i + k] or is_s[prev + k] != is_s[i + k]:
                        diff = True
                        break
                    if k > 0 and (is_lms[prev + k] or is_lms[i + k]):
                        if is_lms[prev + k] != is_lms[i + k]:
                            diff = True
                        break
                    k += 1
                if diff:
                    cur_name += 1
            lms_names[i] = cur_name
            prev = i

    num_lms = len(lms_pos)
    if cur_name + 1 < num_lms:
        reduced = [lms_names[i] for i in lms_pos]
        sa_lms = _sais(reduced, cur_name + 1)
        ordered_lms = [lms_pos[i] for i in sa_lms]
    else:
        ordered_lms = [0] * num_lms
        for i in lms_pos:
            ordered_lms[lms_names[i]] = i

    # Final induced sort from sorted LMS suffixes
    sa = [-1] * n
    tails = bucket_tails()
    for i in range(num_lms - 1, -1, -1):
        p = ordered_lms[i]
        c = s[p]
        sa[tails[c]] = p
        tails[c] -= 1
    induce(sa)
    return sa


def build_noncyclic_sa(T: bytes) -> tuple[list[int], list[int]]:
    """Build exact non-cyclic (SA, ISA) of T with end-of-string sentinel < all bytes."""
    n = len(T)
    s = [b + 1 for b in T]
    s.append(0)
    sa_full = _sais(s, 257)
    assert sa_full[0] == n
    sa_nc = sa_full[1:]
    isa_nc = [0] * n
    for rank, pos in enumerate(sa_nc):
        isa_nc[pos] = rank
    # O(n) exactness certificate: (T[a], isa_nc[a+1]) < (T[b], isa_nc[b+1]) for all adjacent ranks
    for i in range(1, n):
        a = sa_nc[i - 1]
        b = sa_nc[i]
        ta = T[a]
        tb = T[b]
        if ta != tb:
            assert ta < tb, f"Non-cyclic SA order violation at rank {i}"
        else:
            ra = isa_nc[a + 1] if a + 1 < n else -1
            rb = isa_nc[b + 1] if b + 1 < n else -1
            assert ra < rb, f"Non-cyclic SA tie-break violation at rank {i}"
    return sa_nc, isa_nc


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
    """Compute (lpf_len, lpf_src) via exact PSV/NSV monotone stacks over non-cyclic (SA, LCP)."""
    n = len(sa)
    lpf_len = [0] * n
    lpf_src = [-1] * n

    # Pass 1: PSV (nearest i' < i with SA[i'] < SA[i])
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


def test_n12_witness() -> None:
    """Automated regression test for the N-16 cyclic-vs-suffix LPF witness (L=baaaa, p=2)."""
    L = b"baaaa"
    p = 2
    T, sa_cyc, isa_cyc, lf = build_sa_isa_t(L, p)
    assert lf == [4, 0, 1, 2, 3]
    assert T == b"aabaa"
    assert sa_cyc == [3, 4, 0, 1, 2]

    # Confirm the old bug on cyclic order:
    lcp_buggy = compute_lcp_kasai(T, sa_cyc, isa_cyc)
    lpf_buggy, _ = compute_lpf(sa_cyc, lcp_buggy)
    assert lpf_buggy == [0, 1, 0, 1, 1]

    # Confirm the repaired non-cyclic SA/LCP/LPF:
    sa_nc, isa_nc = build_noncyclic_sa(T)
    assert sa_nc == [4, 3, 0, 1, 2]
    lcp_true = compute_lcp_kasai(T, sa_nc, isa_nc)
    lpf_true, src_true = compute_lpf(sa_nc, lcp_true)
    assert lpf_true == [0, 1, 0, 2, 1], f"Expected [0,1,0,2,1], got {lpf_true}"
    assert src_true[3] == 0

    # Also cross-check against brute-force LPF on edge cases and pseudo-random strings
    import random

    rng = random.Random(20261124)
    cases = [b"a", b"aa", b"aaaa", b"aabaa", b"ababab", b"banana", b"mississippi"]
    for _ in range(40):
        ln = rng.randint(2, 40)
        alph = rng.choice([b"ab", b"abc", b"abcd"])
        cases.append(bytes(rng.choice(alph) for _ in range(ln)))
    for text in cases:
        n = len(text)
        sa_nc, isa_nc = build_noncyclic_sa(text)
        expected_sa = sorted(range(n), key=lambda idx: text[idx:])
        assert sa_nc == expected_sa, f"SA mismatch on {text!r}"
        lcp = compute_lcp_kasai(text, sa_nc, isa_nc)
        lpf_len, lpf_src = compute_lpf(sa_nc, lcp)
        for j in range(n):
            best = 0
            for sr in range(j):
                k = 0
                while j + k < n and text[sr + k] == text[j + k]:
                    k += 1
                if k > best:
                    best = k
            assert lpf_len[j] == best, f"LPF mismatch on {text!r} at j={j}: {lpf_len[j]} != {best}"
            if best > 0:
                sr = lpf_src[j]
                assert 0 <= sr < j and text[sr : sr + best] == text[j : j + best]


def run_n12_lz77(
    fname: str,
    T: bytes,
    isa_cyc: list[int],
    is_w1: bytearray,
    is_thin: bytearray,
) -> None:
    n = len(T)
    sa_nc, isa_nc = build_noncyclic_sa(T)
    lcp = compute_lcp_kasai(T, sa_nc, isa_nc)
    lpf_len, lpf_src = compute_lpf(sa_nc, lcp)

    # Emit row for T[j] is isa_cyc[(j + 1) % n] (BWT row order)
    w1_at_pos = bytearray(n)
    thin_at_pos = bytearray(n)
    for j in range(n):
        row = isa_cyc[(j + 1) % n]
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


def test_n13_counterexample() -> None:
    """Verify the round-2 N-13 arithmetic-DSA counterexample: T = [0,2,4,6,8,7,5,3,1]."""
    T = bytes([0, 2, 4, 6, 8, 7, 5, 3, 1])
    n = len(T)
    # Build exact cyclic BWT of T
    rots = sorted(range(n), key=lambda j: T[j:] + T[:j])
    L = bytes(T[(j - 1) % n] for j in rots)
    p = rots.index(0)
    assert L == bytes([1, 3, 0, 5, 2, 7, 4, 8, 6])
    assert p == 0
    T2, sa_cyc, _, lf = build_sa_isa_t(L, p)
    assert T2 == T
    assert sa_cyc == [0, 8, 1, 7, 2, 6, 3, 5, 4]
    # Check O(1) closed-form arithmetic formula: SA[2j] = j, SA[2j+1] = 8 - j
    for idx, val in enumerate(sa_cyc):
        expected = idx // 2 if idx % 2 == 0 else (n - 1) - (idx // 2)
        assert val == expected
    dsa = [sa_cyc[0] - sa_cyc[-1]] + [sa_cyc[i] - sa_cyc[i - 1] for i in range(1, n)]
    assert dsa == [-4, 8, -7, 6, -5, 4, -3, 2, -1]
    assert len(set(dsa)) == n


def main() -> None:
    import sys

    args = set(sys.argv[1:])
    test_n12_witness()
    test_n13_counterexample()
    print("=== SELF-TESTS PASSED: N-12 witness (L=baaaa, p=2 -> LPF=[0,1,0,2,1]) & N-13 arithmetic counterexample ===")
    if "--tests-only" in args:
        return

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

    print("\n=== N-12 / N-16 (§5.1 REPAIRED): EXACT NON-CYCLIC ACYCLIC LZ77 COPY PLANS (BYTE-VERIFIED) ===")
    for fname, L, T, sa, isa, lf, r, is_w1, is_thin in prepped:
        run_n12_lz77(fname, T, isa, is_w1, is_thin)

    if "--only-n12" in args:
        return

    print("\n=== N-13 (§5.2): COMPRESSED DSA (SA[i]-SA[i-1]) & KEMPA THM 5.1 AUDIT ===")
    for fname, L, T, sa, isa, lf, r, is_w1, is_thin in prepped:
        run_n13_dsa(fname, L, sa, lf, r, is_w1, is_thin)

    print("\n=== N-14 (§5.3): DURAND ORDERED RETURN-WORD HIERARCHIES WITH PAID EXCEPTIONS ===")
    for fname, L, T, sa, isa, lf, r, is_w1, is_thin in prepped:
        run_n14_durand(fname, T, isa, is_w1)


if __name__ == "__main__":
    main()

