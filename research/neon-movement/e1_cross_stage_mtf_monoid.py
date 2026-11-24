#!/usr/bin/env python3
"""E1 (Principle 2 — Upstream Cross-Stage Enrichment):
Enriched MTF-Occurrence-Gap Semidirect Monoid
    M_MTF = (N^Sigma x Z_min^Sigma x Z_max^Sigma) rtimes S_Sigma

Demonstrates:
1. Exact associative monoid lifting of bzip2's MTF + RLE2 token stream so that
   chunk-level MTF permutation, per-symbol occurrence prefix sums, exact inter-chunk
   row gaps, and Open run records IN are acquired in ONE parallel O(log G) prefix
   scan with ZERO materialisation of the n-byte column L.
2. Active MTF support width K_g census across 50-symbol Huffman groups and
   1024-token chunks on novel random hex + all 6 admitted Wikidata blocks.
3. Exact NEEDS M-22 measurement (MONODROMY record 115): per-symbol gap stream
   sizes (1-byte gap with 255-escape + 16-entry absolute anchor) on full L and
   on Open's retained rows M (L_thresh=16), plus byte-exact inverse-BWT decode
   directly from the zero-L gap-encoded inverse table on all 8/8 blocks.
"""

from __future__ import annotations

import json
import math
import os
import sys
from dataclasses import dataclass
from typing import Any

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common


# ---------------------------------------------------------------------------
# 1. Exact bzip2 forward MTF + RLE2 (deterministic from L) & verification
# ---------------------------------------------------------------------------

def encode_bzip2_mtf_rle2(L: bytes) -> tuple[list[int], list[int]]:
    """Convert BWT column L to exact bzip2 RLE2+MTF token stream.

    Token alphabet (matching libbzip2 / src/mtf.rs):
      0 = RUNA, 1 = RUNB, k+1 (for k >= 1) = nonzero MTF depth k.
    Returns (tokens, seq_to_unseq).
    """
    seq_to_unseq = sorted(set(L))
    m = len(seq_to_unseq)
    unseq_to_seq = [0] * 256
    for idx, b in enumerate(seq_to_unseq):
        unseq_to_seq[b] = idx

    mtf = list(range(m))
    tokens: list[int] = []

    def emit_zero_run(zlen: int) -> None:
        rem = zlen
        while rem > 0:
            if rem & 1:
                tokens.append(0)  # RUNA
                rem = (rem - 1) >> 1
            else:
                tokens.append(1)  # RUNB
                rem = (rem - 2) >> 1

    zrun = 0
    for b in L:
        s = unseq_to_seq[b]
        if mtf[0] == s:
            zrun += 1
        else:
            if zrun > 0:
                emit_zero_run(zrun)
                zrun = 0
            k = mtf.index(s)
            # Move to front
            mtf.pop(k)
            mtf.insert(0, s)
            tokens.append(k + 1)
    if zrun > 0:
        emit_zero_run(zrun)
    return tokens, seq_to_unseq


# ---------------------------------------------------------------------------
# 2. Enriched MTF-Occurrence-Gap Semidirect Monoid M_MTF
# ---------------------------------------------------------------------------

INF = 1 << 30
NEG_INF = -(1 << 30)


@dataclass
class MtfMonoidElem:
    """Element of M_MTF = (N^m x Z_min^m x Z_max^m) rtimes S_m.

    Convention:
      sigma[p] = initial slot index that occupies MTF position p at chunk exit.
      delta[i] = total emitted bytes of initial slot i in this chunk.
      first[i] = first local row offset of initial slot i (INF if delta[i]==0).
      last[i]  = last local row offset of initial slot i (NEG_INF if delta[i]==0).
      length   = total bytes emitted by this chunk.
      k_active = 1 + max MTF position touched (all slots >= k_active are fixed!).
    """
    m: int
    sigma: list[int]
    delta: list[int]
    first: list[int]
    last: list[int]
    length: int
    k_active: int


def monoid_identity(m: int) -> MtfMonoidElem:
    return MtfMonoidElem(
        m=m,
        sigma=list(range(m)),
        delta=[0] * m,
        first=[INF] * m,
        last=[NEG_INF] * m,
        length=0,
        k_active=0,
    )


def monoid_mul(a: MtfMonoidElem, b: MtfMonoidElem) -> MtfMonoidElem:
    """Associative composition a (x) b in M_MTF."""
    m = a.m
    k = max(a.k_active, b.k_active)
    sigma = list(range(m))
    delta = a.delta[:]
    first = a.first[:]
    last = a.last[:]
    # Only slots in [0, k) can move or have nonzero activity!
    sa = a.sigma
    sb = b.sigma
    len_a = a.length
    for p in range(k):
        orig = sa[sb[p]]
        sigma[p] = orig
    # Map b's initial slots q -> a's initial slots orig = sa[q]
    for q in range(b.k_active):
        orig = sa[q]
        db = b.delta[q]
        if db:
            delta[orig] += db
            fb = len_a + b.first[q]
            if fb < first[orig]:
                first[orig] = fb
            lb = len_a + b.last[q]
            if lb > last[orig]:
                last[orig] = lb
    return MtfMonoidElem(
        m=m,
        sigma=sigma,
        delta=delta,
        first=first,
        last=last,
        length=len_a + b.length,
        k_active=k,
    )


def partition_tokens_aligned(tokens: list[int], target_chunk: int) -> list[tuple[int, int]]:
    """Partition token stream into chunks of ~target_chunk tokens without splitting
    a RUNA/RUNB run across chunk boundaries (only 1-token boundary lookahead)."""
    n_tok = len(tokens)
    chunks: list[tuple[int, int]] = []
    i = 0
    while i < n_tok:
        j = min(i + target_chunk, n_tok)
        # Advance past any trailing RUNA/RUNB tokens so each run stays intact
        while j < n_tok and tokens[j] <= 1:
            j += 1
        chunks.append((i, j))
        i = j
    return chunks


def eval_chunk_monoid(tokens: list[int], start: int, end: int, m: int) -> MtfMonoidElem:
    """Compute M_MTF element for tokens[start:end] independently (no prior state!)."""
    sigma = list(range(m))
    delta = [0] * m
    first = [INF] * m
    last = [NEG_INF] * m
    pos = 0
    k_active = 0
    i = start
    while i < end:
        tok = tokens[i]
        if tok <= 1:
            # RUNA/RUNB run of slot currently at sigma[0]
            run = 0
            weight = 1
            while i < end and tokens[i] <= 1:
                run += (tokens[i] + 1) * weight
                weight <<= 1
                i += 1
            slot0 = sigma[0]
            if k_active < 1:
                k_active = 1
            if first[slot0] == INF:
                first[slot0] = pos
            last[slot0] = pos + run - 1
            delta[slot0] += run
            pos += run
        else:
            k = tok - 1
            if k + 1 > k_active:
                k_active = k + 1
            slot = sigma[k]
            # Move slot to front
            sigma.pop(k)
            sigma.insert(0, slot)
            if first[slot] == INF:
                first[slot] = pos
            last[slot] = pos
            delta[slot] += 1
            pos += 1
            i += 1
    return MtfMonoidElem(
        m=m,
        sigma=sigma,
        delta=delta,
        first=first,
        last=last,
        length=pos,
        k_active=k_active,
    )


def verify_monoid_associativity(elems: list[MtfMonoidElem]) -> None:
    """Explicitly assert (A*B)*C == A*(B*C) on sample triples."""
    for idx in range(0, min(len(elems) - 2, 20), 3):
        a, b, c = elems[idx], elems[idx + 1], elems[idx + 2]
        lhs = monoid_mul(monoid_mul(a, b), c)
        rhs = monoid_mul(a, monoid_mul(b, c))
        assert lhs.sigma == rhs.sigma
        assert lhs.delta == rhs.delta
        assert lhs.first == rhs.first
        assert lhs.last == rhs.last
        assert lhs.length == rhs.length


def prefix_scan_monoid(elems: list[MtfMonoidElem], m: int) -> tuple[list[MtfMonoidElem], MtfMonoidElem]:
    """Exclusive prefix scan in M_MTF (associative; models O(log G) Blelloch scan).
    Returns (prefixes[0..G-1], total_product).
    """
    prefixes: list[MtfMonoidElem] = []
    acc = monoid_identity(m)
    for e in elems:
        prefixes.append(acc)
        acc = monoid_mul(acc, e)
    return prefixes, acc


# ---------------------------------------------------------------------------
# 3. Zero-L Parallel Direct Emission of (a) Open IN runs & (b) Gap-Encoded Psi
# ---------------------------------------------------------------------------

def emit_gap_streams_and_runs_zero_L(
    tokens: list[int],
    seq_to_unseq: list[int],
    chunks: list[tuple[int, int]],
    prefixes: list[MtfMonoidElem],
    total: MtfMonoidElem,
    anchor_stride: int = 16,
) -> tuple[list[tuple[int, int, int, int]], list[list[int]], list[bytearray], list[int], dict[str, int]]:
    """Directly emit Open's run records IN AND MONODROMY 115's per-symbol gap streams
    from the MTF tokens + M_MTF prefix states, with ZERO materialisation of L[0..n-1].

    Returns:
      runs_with_rank: list of (start_row, width, byte_val, occ_before)
      anchors[s]: list of absolute row anchors every anchor_stride occurrences
      gap_streams[s]: bytearray of 1-byte gaps (1..254) with 255 + u32 escape
      counts_seq[s]: exact counts per seq symbol s
      stats: escape counts, raw vs varint byte sizes
    """
    m = len(seq_to_unseq)
    counts_seq = total.delta[:]
    # Per-symbol output buffers (each chunk knows its exact starting occ & prev_row,
    # so chunks can emit independently in parallel and concatenate per symbol).
    anchors: list[list[int]] = [[] for _ in range(m)]
    gap_streams: list[bytearray] = [bytearray() for _ in range(m)]
    runs_with_rank: list[tuple[int, int, int, int]] = []

    n_gaps_total = 0
    n_gaps_escaped = 0
    n_gaps_one = 0
    varint_extra_bytes = 0

    # For Open run merging across chunk boundaries (if last symbol of chunk g-1 == first of g)
    for g, (start, end) in enumerate(chunks):
        pref = prefixes[g]
        # Initial MTF state for chunk g: state[p] = seq_symbol at position p
        state = pref.sigma[:]
        occ = pref.delta[:]
        prev_row = pref.last[:]
        row = pref.length

        def record_occ_range(sym: int, r_start: int, width: int) -> None:
            nonlocal n_gaps_total, n_gaps_escaped, n_gaps_one, varint_extra_bytes
            # Merge into Open runs_with_rank
            if runs_with_rank and runs_with_rank[-1][2] == seq_to_unseq[sym]:
                ps, pw, pb, po = runs_with_rank[-1]
                runs_with_rank[-1] = (ps, pw + width, pb, po)
            else:
                runs_with_rank.append((r_start, width, seq_to_unseq[sym], occ[sym]))

            gs = gap_streams[sym]
            anc = anchors[sym]
            o = occ[sym]
            pr = prev_row[sym]
            for j in range(width):
                cur_r = r_start + j
                if (o % anchor_stride) == 0:
                    anc.append(cur_r)
                else:
                    gap = cur_r - pr
                    assert gap >= 1
                    n_gaps_total += 1
                    if gap == 1:
                        n_gaps_one += 1
                    if gap <= 254:
                        gs.append(gap)
                    else:
                        n_gaps_escaped += 1
                        gs.append(255)
                        gs.extend(gap.to_bytes(4, "little"))
                        # Also compute LEB128 varint extra bytes over 1-byte header
                        rem = gap - 255
                        vb = 1
                        while rem >= 128:
                            rem >>= 7
                            vb += 1
                        varint_extra_bytes += vb
                pr = cur_r
                o += 1
            occ[sym] = o
            prev_row[sym] = pr

        i = start
        while i < end:
            tok = tokens[i]
            if tok <= 1:
                run = 0
                weight = 1
                while i < end and tokens[i] <= 1:
                    run += (tokens[i] + 1) * weight
                    weight <<= 1
                    i += 1
                sym = state[0]
                record_occ_range(sym, row, run)
                row += run
            else:
                k = tok - 1
                sym = state[k]
                state.pop(k)
                state.insert(0, sym)
                record_occ_range(sym, row, 1)
                row += 1
                i += 1

    stats = {
        "n_gaps_total": n_gaps_total,
        "n_gaps_one": n_gaps_one,
        "n_gaps_escaped": n_gaps_escaped,
        "anchor_bytes": sum(len(a) * 4 for a in anchors),
        "gap_stream_bytes_u32esc": sum(len(gs) for gs in gap_streams),
        "gap_stream_bytes_varint": (
            (n_gaps_total - n_gaps_escaped) + n_gaps_escaped + varint_extra_bytes
        ),
    }
    return runs_with_rank, anchors, gap_streams, counts_seq, stats


def decode_from_gap_streams_zero_L(
    n: int,
    primary_index: int,
    seq_to_unseq: list[int],
    counts_seq: list[int],
    anchors: list[list[int]],
    gap_streams: list[bytearray],
    anchor_stride: int = 16,
) -> tuple[bytes, dict[str, int]]:
    """Byte-exact forward-Psi inverse BWT directly from the zero-L per-symbol gap streams!

    Notice:
      - L[0..n-1] is NEVER read or stored.
      - The 4n-byte tt array is NEVER built or stored.
      - For each symbol bucket s, anchors[s][q] gives Psi(C_seq[s] + 16*q) directly,
        and decoding the r = y % 16 gaps from gap_streams[s] gives Psi(y) = next_row!
      - To locate gap_streams[s] for block q in O(1), we keep a tiny block-offset
        directory block_off[s][q] (only n/16 entries -> 4n/16 = 0.25n bytes, or packed
        with anchor in 8 bytes per 16 rows = 0.5n bytes!).
    """
    m = len(seq_to_unseq)
    # Build C_seq (F-bucket starts in seq order) and block byte-offsets in gap_streams
    C_seq = [0] * (m + 1)
    acc = 0
    for s in range(m):
        C_seq[s] = acc
        acc += counts_seq[s]
    C_seq[m] = acc
    assert acc == n

    # Precompute byte offset of each 16-entry gap block (O(n/16) entries)
    block_off: list[list[int]] = []
    for s in range(m):
        gs = gap_streams[s]
        cnt = counts_seq[s]
        n_blk = (cnt + anchor_stride - 1) // anchor_stride
        offs = [0] * n_blk
        p = 0
        for b_idx in range(n_blk):
            offs[b_idx] = p
            # Skip up to (anchor_stride - 1) gaps in this block
            in_blk = min(anchor_stride, cnt - b_idx * anchor_stride) - 1
            for _ in range(in_blk):
                if gs[p] == 255:
                    p += 5
                else:
                    p += 1
        assert p == len(gs)
        block_off.append(offs)

    # F-bucket lookup by binary search on C_seq (m <= 256, O(log m) register ops)
    import bisect

    out = bytearray(n)
    # Forward Psi walk: start at row = Psi(primary_index)? Wait:
    # Let's match common.reference_decode convention:
    #   reference_decode: x starts at primary_index; out[n-1] = L[primary_index], x = LF(x)...
    #   Equivalently: F[primary_index] = T[0], F[Psi(primary_index)] = T[1], ..., F[Psi^{n-1}(p)] = T[n-1]!
    x = primary_index
    total_gap_steps = 0
    total_escapes_traversed = 0
    for t in range(n):
        s = bisect.bisect_right(C_seq, x) - 1
        out[t] = seq_to_unseq[s]
        local_idx = x - C_seq[s]
        q, rem = divmod(local_idx, anchor_stride)
        row_val = anchors[s][q]
        if rem > 0:
            gs = gap_streams[s]
            p = block_off[s][q]
            for _ in range(rem):
                g = gs[p]
                if g == 255:
                    g = int.from_bytes(gs[p + 1 : p + 5], "little")
                    p += 5
                    total_escapes_traversed += 1
                else:
                    p += 1
                row_val += g
            total_gap_steps += rem
        x = row_val
    assert x == primary_index, f"Cycle mismatch: ended at {x} != {primary_index}"
    return bytes(out), {
        "total_gap_steps": total_gap_steps,
        "mean_gap_steps_per_row": round(total_gap_steps / n, 3),
        "total_escapes_traversed": total_escapes_traversed,
        "block_dir_bytes": sum(len(bo) * 4 for bo in block_off),
    }


# ---------------------------------------------------------------------------
# 4. Retained-Row Gap Streams (NEEDS M-22 second half: Open's retained rows M)
# ---------------------------------------------------------------------------

def measure_retained_gap_streams(blk: common.BlockData, limit: int = 16, anchor_stride: int = 16) -> dict[str, Any]:
    """Measure MONODROMY 115 NEEDS M-22 on Open's retained rows M (at LIMIT=16)."""
    ret = common.build_retained_system_genuine(blk, L_min=limit)
    # Build per-symbol sorted retained-row indices (each retained row j has domain row x and byte L[x])
    by_sym: list[list[int]] = [[] for _ in range(256)]
    for j in range(ret.m):
        x = ret.rows[j]
        by_sym[blk.L[x]].append(x)

    n_gaps = 0
    n_esc = 0
    anchor_bytes = 0
    gap_bytes_u32 = 0
    varint_extra = 0
    for c in range(256):
        xs = by_sym[c]
        if not xs:
            continue
        for idx, x in enumerate(xs):
            if (idx % anchor_stride) == 0:
                anchor_bytes += 4
            else:
                g = x - xs[idx - 1]
                n_gaps += 1
                if g <= 254:
                    gap_bytes_u32 += 1
                else:
                    n_esc += 1
                    gap_bytes_u32 += 5
                    rem = g - 255
                    vb = 1
                    while rem >= 128:
                        rem >>= 7
                        vb += 1
                    varint_extra += vb

    # Open's retained edge table baseline: 4 bytes/retained row for target + word bytes
    open_edge_bytes = 4 * ret.m
    # Directory bytes for O(1) anchor-block jump: 4 bytes per anchor (or 2-byte relative offset)
    dir_bytes = anchor_bytes  # 4B per 16 entries
    tot_u32 = anchor_bytes + gap_bytes_u32 + dir_bytes
    tot_varint = anchor_bytes + (n_gaps - n_esc) + n_esc + varint_extra + dir_bytes
    return {
        "m_retained": ret.m,
        "m_over_h1": round(ret.m / blk.h1, 4) if blk.h1 else 0.0,
        "n_gaps": n_gaps,
        "n_escaped": n_esc,
        "escaped_frac": round(n_esc / n_gaps, 4) if n_gaps else 0.0,
        "anchor_bytes": anchor_bytes,
        "gap_bytes_u32": gap_bytes_u32,
        "dir_bytes": dir_bytes,
        "total_bytes_u32esc": tot_u32,
        "total_bytes_varint": tot_varint,
        "open_edge_bytes_4m": open_edge_bytes,
        "ratio_vs_4m_u32esc": round(tot_u32 / open_edge_bytes, 4) if open_edge_bytes else 0.0,
        "ratio_vs_4m_varint": round(tot_varint / open_edge_bytes, 4) if open_edge_bytes else 0.0,
        "bytes_per_retained_row_u32esc": round(tot_u32 / ret.m, 3) if ret.m else 0.0,
        "bytes_per_retained_row_varint": round(tot_varint / ret.m, 3) if ret.m else 0.0,
    }


# ---------------------------------------------------------------------------
# 5. Benchmark Driver (8/8 byte-exact)
# ---------------------------------------------------------------------------

def run_block(blk: common.BlockData) -> dict[str, Any]:
    tokens, seq_to_unseq = encode_bzip2_mtf_rle2(blk.L)
    m = len(seq_to_unseq)

    # (A) 50-symbol Huffman-group active-support census K_g
    chunks_50 = partition_tokens_aligned(tokens, target_chunk=50)
    elems_50 = [eval_chunk_monoid(tokens, s, e, m) for (s, e) in chunks_50]
    verify_monoid_associativity(elems_50)
    k50 = [e.k_active for e in elems_50]
    k50_sorted = sorted(k50)
    k50_mean = sum(k50) / len(k50)
    k50_med = k50_sorted[len(k50_sorted) // 2]
    k50_p95 = k50_sorted[int(0.95 * (len(k50_sorted) - 1))]
    k50_le16_frac = sum(1 for v in k50 if v <= 16) / len(k50)
    k50_le32_frac = sum(1 for v in k50 if v <= 32) / len(k50)

    # (B) 1024-token chunk parallel monoid scan + zero-L direct IN & gap emission
    chunks_1k = partition_tokens_aligned(tokens, target_chunk=1024)
    elems_1k = [eval_chunk_monoid(tokens, s, e, m) for (s, e) in chunks_1k]
    verify_monoid_associativity(elems_1k)
    prefixes_1k, total_1k = prefix_scan_monoid(elems_1k, m)
    k1k_mean = sum(e.k_active for e in elems_1k) / len(elems_1k)

    # Verify total_1k matches exact block length & symbol counts (zero-L!)
    assert total_1k.length == blk.n
    for s, c in enumerate(seq_to_unseq):
        assert total_1k.delta[s] == blk.counts[c]

    runs_zero_L, anchors, gap_streams, counts_seq, gstats = emit_gap_streams_and_runs_zero_L(
        tokens, seq_to_unseq, chunks_1k, prefixes_1k, total_1k, anchor_stride=16
    )

    # Assert zero-L Open runs match exact L runs 1-for-1!
    assert len(runs_zero_L) == blk.r, f"run count mismatch: {len(runs_zero_L)} != {blk.r}"
    for (s1, w1, b1, _), (s2, w2, b2) in zip(runs_zero_L, blk.runs):
        assert (s1, w1, b1) == (s2, w2, b2)

    # Byte-exact inverse BWT directly from zero-L gap streams!
    decoded, dec_stats = decode_from_gap_streams_zero_L(
        blk.n, blk.primary_index, seq_to_unseq, counts_seq, anchors, gap_streams, anchor_stride=16
    )
    assert decoded == blk.T_ref, f"Byte-exact decode FAILED on {blk.name}"

    # Full-L NEEDS M-22 size breakdown
    total_gap_u32_with_dir = (
        gstats["anchor_bytes"] + gstats["gap_stream_bytes_u32esc"] + dec_stats["block_dir_bytes"]
    )
    total_gap_u32_nodir = gstats["anchor_bytes"] + gstats["gap_stream_bytes_u32esc"]
    total_gap_varint_with_dir = (
        gstats["anchor_bytes"] + gstats["gap_stream_bytes_varint"] + dec_stats["block_dir_bytes"]
    )
    total_gap_varint_nodir = gstats["anchor_bytes"] + gstats["gap_stream_bytes_varint"]

    # Retained-row NEEDS M-22 measurement
    ret_gap = measure_retained_gap_streams(blk, limit=16, anchor_stride=16)

    # Pi cost accounting for full zero-L M_MTF + gap-stream decode
    # Build: 2 streamed passes over token stream (len(tokens) << n!) + O(log G) scan of G chunks
    # Emit: Q = n addressed accesses (each lands in ~1.2-1.8n bytes instead of 4n tt), D = n
    n_chunks_1k = len(chunks_1k)
    d_build_parallel = 1024 + 2 * max(1, math.ceil(math.log2(max(2, n_chunks_1k))))
    pi_full = common.PiCost(
        Q=blk.n,
        D=blk.n + d_build_parallel,
        P=2,  # pass 1: token->chunk monoids; pass 2: token->gap streams (L never written/read!)
        S=total_gap_u32_with_dir + n_chunks_1k * (m * 16),
        Q_acq=0,
        Q_run=blk.n,
        D_acq=d_build_parallel,
        D_run=blk.n,
        P_acq=2,
        P_run=0,
        P_bytes=(2 * len(tokens) * 2 + total_gap_u32_with_dir) / blk.n,
        s_cursors=m + 2,
        notes="Zero-L M_MTF parallel build + gap-stream Psi walk (no 4n tt, no n-byte L)",
    )

    return {
        "name": blk.name,
        "n": blk.n,
        "r": blk.r,
        "h1": blk.h1,
        "alphabet_active": m,
        "n_mtf_rle2_tokens": len(tokens),
        "tokens_over_n": round(len(tokens) / blk.n, 4),
        "byte_exact": True,
        "huff50_groups": {
            "n_groups": len(chunks_50),
            "K_g_mean": round(k50_mean, 2),
            "K_g_median": k50_med,
            "K_g_p95": k50_p95,
            "frac_K_le_16": round(k50_le16_frac, 4),
            "frac_K_le_32": round(k50_le32_frac, 4),
        },
        "chunk1k_parallel": {
            "n_chunks": n_chunks_1k,
            "K_g_mean": round(k1k_mean, 2),
            "parallel_build_depth_D_acq": d_build_parallel,
            "depth_collapse_vs_serial_n": round(blk.n / d_build_parallel, 1),
        },
        "needs_m22_full_L": {
            "n_gaps": gstats["n_gaps_total"],
            "n_gaps_one": gstats["n_gaps_one"],
            "frac_gap_eq_1": round(gstats["n_gaps_one"] / gstats["n_gaps_total"], 4),
            "n_escaped": gstats["n_gaps_escaped"],
            "escaped_frac": round(gstats["n_gaps_escaped"] / gstats["n_gaps_total"], 4),
            "anchor_bytes": gstats["anchor_bytes"],
            "block_dir_bytes": dec_stats["block_dir_bytes"],
            "gap_bytes_u32esc": gstats["gap_stream_bytes_u32esc"],
            "gap_bytes_varint": gstats["gap_stream_bytes_varint"],
            "total_nodir_u32esc": total_gap_u32_nodir,
            "total_withdir_u32esc": total_gap_u32_with_dir,
            "total_nodir_varint": total_gap_varint_nodir,
            "total_withdir_varint": total_gap_varint_with_dir,
            "ratio_vs_4n_nodir_u32esc": round(total_gap_u32_nodir / (4 * blk.n), 4),
            "ratio_vs_4n_withdir_u32esc": round(total_gap_u32_with_dir / (4 * blk.n), 4),
            "ratio_vs_4n_withdir_varint": round(total_gap_varint_with_dir / (4 * blk.n), 4),
            "bytes_per_row_withdir_u32esc": round(total_gap_u32_with_dir / blk.n, 3),
            "bytes_per_row_withdir_varint": round(total_gap_varint_with_dir / blk.n, 3),
            "mean_inline_gap_steps": dec_stats["mean_gap_steps_per_row"],
        },
        "needs_m22_retained_M16": ret_gap,
        "pi_cost_full": pi_full.as_dict(blk.h1, blk.n),
    }


def main() -> None:
    blocks = [
        common.make_synthetic_novel_hex_iid(65536),
        common.make_synthetic_templated_hex(1000, 40),
        *common.load_admitted_blocks(),
    ]
    results = []
    for blk in blocks:
        res = run_block(blk)
        results.append(res)
        fL = res["needs_m22_full_L"]
        rM = res["needs_m22_retained_M16"]
        h50 = res["huff50_groups"]
        print(
            f"{blk.name:24s} | tok/n={res['tokens_over_n']:.3f} | "
            f"K50_med={h50['K_g_median']:2d} (<=16:{h50['frac_K_le_16']*100:4.1f}%) | "
            f"M22_full: {fL['bytes_per_row_withdir_u32esc']:.2f}B/row ({fL['ratio_vs_4n_withdir_u32esc']:.3f}x 4n, esc={fL['escaped_frac']*100:.2f}%) | "
            f"M22_M16: {rM['bytes_per_retained_row_u32esc']:.2f}B/m ({rM['ratio_vs_4m_u32esc']:.3f}x 4m, esc={rM['escaped_frac']*100:.1f}%) | EXACT"
        )

    out_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "results_e1_cross_stage_mtf_monoid.json")
    with open(out_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
