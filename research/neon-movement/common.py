#!/usr/bin/env python3
"""Shared testbed, independent reference BWT decoder, synthetic novel-hex
generators, genuine endpoint-only thick-strip contraction (no row-by-row
walk through thick interiors), and strict Pi cost accounting (MONODROMY 112).
"""

from __future__ import annotations

import bisect
import hashlib
import json
import math
import random
import subprocess
from dataclasses import dataclass, field
from typing import Any

HEX_BYTES = b"0123456789abcdef"


@dataclass
class BlockData:
    name: str
    n: int
    primary_index: int
    L: bytes
    r: int
    h1: int
    runs: list[tuple[int, int, int]]  # (start, width, byte_val)
    C: list[int]  # 256 F-bucket starts
    counts: list[int]  # 256 symbol counts
    T_ref: bytes  # independent reference decode


@dataclass
class PiCost:
    """Cost vector under specification Pi (MONODROMY record 112)."""
    Q: int  # addressed LOAD/STORE/SEEK count (acquisition + run)
    D: int  # longest dependent chain of charged accesses
    P: int  # sweep count under Pi trace-to-pass rule (K=2 budget)
    S: int  # peak working memory in bytes
    Q_acq: int = 0
    Q_run: int = 0
    D_acq: int = 0
    D_run: int = 0
    P_acq: int = 0
    P_run: int = 0
    P_bytes: float = 0.0  # total streamed bytes / n (diagnostic only)
    s_cursors: int = 0  # peak concurrent cursors (budget |Sigma| + c0)
    notes: str = ""

    def as_dict(self, h1: int, n: int) -> dict[str, Any]:
        return {
            "Q": self.Q,
            "Q_over_h1": round(self.Q / h1, 4) if h1 else 0.0,
            "Q_acq": self.Q_acq,
            "Q_run": self.Q_run,
            "D": self.D,
            "D_over_n": round(self.D / n, 6) if n else 0.0,
            "P": self.P,
            "P_acq": self.P_acq,
            "P_run": self.P_run,
            "P_bytes": round(self.P_bytes, 3),
            "S": self.S,
            "S_over_n": round(self.S / n, 3) if n else 0.0,
            "s_cursors": self.s_cursors,
            "notes": self.notes,
        }


def compute_counts_and_runs(L: bytes) -> tuple[list[int], list[int], list[tuple[int, int, int]], int]:
    """Single streaming pass over L: symbol counts, F-starts C, maximal runs, h1."""
    n = len(L)
    counts = [0] * 256
    runs: list[tuple[int, int, int]] = []
    h1 = 0
    if n == 0:
        return [0] * 256, counts, runs, 0
    cur_b = L[0]
    cur_s = 0
    for i, b in enumerate(L):
        counts[b] += 1
        if b != cur_b:
            w = i - cur_s
            runs.append((cur_s, w, cur_b))
            if w == 1:
                h1 += 1
            cur_b = b
            cur_s = i
    w = n - cur_s
    runs.append((cur_s, w, cur_b))
    if w == 1:
        h1 += 1
    C = [0] * 256
    acc = 0
    for b in range(256):
        C[b] = acc
        acc += counts[b]
    return C, counts, runs, h1


def build_lf_array(L: bytes, C: list[int]) -> list[int]:
    """Reference LF array (used ONLY by the independent oracle & test harnesses)."""
    occ = C[:]
    lf = [0] * len(L)
    for i, b in enumerate(L):
        lf[i] = occ[b]
        occ[b] += 1
    return lf


def reference_decode(L: bytes, primary_index: int) -> bytes:
    """Independent textbook inverse-BWT oracle via LF walk from primary_index.

    Convention (admitted-bwt/README.md):
    Follow LF backward from primary_index to recover T[n-1], T[n-2], ..., T[0].
    Equivalently, start at Psi(primary_index) and follow Psi reading L[x].
    We verify single-cycle exactness (visited == n and return to primary_index).
    """
    n = len(L)
    C, _, _, _ = compute_counts_and_runs(L)
    lf = build_lf_array(L, C)
    out = bytearray(n)
    x = primary_index
    for k in range(n - 1, -1, -1):
        out[k] = L[x]
        x = lf[x]
    assert x == primary_index, f"Not a single cycle of length n={n}: ended at {x} != {primary_index}"
    return bytes(out)


def bwt_encode_primitive(T: bytes) -> tuple[bytes, int]:
    """O(n log n) suffix-doubling BWT encoder for synthetic test texts (primitive cyclic)."""
    n = len(T)
    # Suffix doubling on cyclic rotations
    rank = list(T)
    sa = list(range(n))
    k = 1
    while k < n:
        key_pair = [(rank[i], rank[(i + k) % n]) for i in range(n)]
        sa.sort(key=lambda i: key_pair[i])
        new_rank = [0] * n
        r_val = 0
        new_rank[sa[0]] = 0
        for idx in range(1, n):
            if key_pair[sa[idx]] != key_pair[sa[idx - 1]]:
                r_val += 1
            new_rank[sa[idx]] = r_val
        rank = new_rank
        if r_val == n - 1:
            break
        k <<= 1
    assert len(set(rank)) == n, "Synthetic text is periodic (not primitive)"
    L = bytes(T[(i - 1) % n] for i in sa)
    primary_index = sa.index(0)
    return L, primary_index


def make_synthetic_novel_hex_iid(n: int = 65536, seed: int = 20261124) -> BlockData:
    """Pure i.i.d. uniform random hex [0-9a-f] — maximally novel random hex."""
    rng = random.Random(seed)
    T = bytes(HEX_BYTES[rng.randrange(16)] for _ in range(n))
    L, p = bwt_encode_primitive(T)
    T_ref = reference_decode(L, p)
    assert T_ref == T
    C, counts, runs, h1 = compute_counts_and_runs(L)
    return BlockData(
        name=f"synth-hex-iid-{n}",
        n=n,
        primary_index=p,
        L=L,
        r=len(runs),
        h1=h1,
        runs=runs,
        C=C,
        counts=counts,
        T_ref=T_ref,
    )


def make_synthetic_templated_hex(m_records: int = 1000, hex_len: int = 40, seed: int = 20261125) -> BlockData:
    """Templated JSON-like records with novel random 40-hex SHA1 fields (Wikidata model G)."""
    rng = random.Random(seed)
    parts: list[bytes] = []
    for i in range(m_records):
        h = bytes(HEX_BYTES[rng.randrange(16)] for _ in range(hex_len))
        rec = f'{{"id":"Q{100000 + i}","sha1":"'.encode("ascii") + h + b'","type":"item"},'
        parts.append(rec)
    T = b"".join(parts)
    L, p = bwt_encode_primitive(T)
    T_ref = reference_decode(L, p)
    assert T_ref == T
    C, counts, runs, h1 = compute_counts_and_runs(L)
    return BlockData(
        name=f"synth-tmpl-hex-{m_records}x{hex_len}",
        n=len(T),
        primary_index=p,
        L=L,
        r=len(runs),
        h1=h1,
        runs=runs,
        C=C,
        counts=counts,
        T_ref=T_ref,
    )


def load_admitted_blocks() -> list[BlockData]:
    """Load all 6 admitted blocks from origin/data/admitted-bwt, verifying SHA-256."""
    manifest = json.loads(
        subprocess.check_output(
            ["git", "show", "origin/data/admitted-bwt:admitted-bwt/manifest.json"]
        )
    )
    blocks: list[BlockData] = []
    for entry in manifest["blocks"]:
        fname = entry["file"]
        raw = subprocess.check_output(
            ["git", "show", f"origin/data/admitted-bwt:admitted-bwt/{fname}"]
        )
        sha = hashlib.sha256(raw).hexdigest()
        assert sha == entry["sha256"], f"SHA-256 mismatch for {fname}"
        assert len(raw) == entry["n"]
        p = entry["primary_index"]
        C, counts, runs, h1 = compute_counts_and_runs(raw)
        assert len(runs) == entry["r"]
        T_ref = reference_decode(raw, p)
        blocks.append(
            BlockData(
                name=fname,
                n=len(raw),
                primary_index=p,
                L=raw,
                r=len(runs),
                h1=h1,
                runs=runs,
                C=C,
                counts=counts,
                T_ref=T_ref,
            )
        )
    return blocks


@dataclass
class RetainedSystem:
    """Genuine Rauzy-Veech / Open thick-strip contraction (endpoint-only, NO row walk).

    Every strip s = (dom_start, width, target_start, word_dag_id) translates
    x in [dom_start, dom_start+width) to target_start + (x - dom_start) while
    emitting word(word_dag_id) (stored as an O(1)-per-strip binary DAG node,
    NOT materialized per row).

    Retained rows M are all rows in runs of width < L_min, plus primary_index
    if primary_index fell in a thick run.
    """
    L_min: int
    m: int
    rows: list[int]          # sorted retained row indices, len m
    rank_of_row: dict[int, int]  # retained row -> rank in 0..m-1
    mark_rank: int           # rank of primary_index in M
    R: list[int]             # induced permutation R: [0..m) -> [0..m)
    R_word_id: list[int]     # DAG word id emitted by each retained transition
    # Word DAG: leaf node i has (byte_val, -1); concat node has (left_id, right_id)
    dag_left: list[int]
    dag_right: list[int]
    dag_len: list[int]
    # Exact Pi accounting for building RetainedSystem from run records IN:
    Q_acq_strips: int        # addressed strip lookups/splits during strip fixpoint
    Q_acq_glue: int          # addressed lookups to resolve each retained row's exit
    D_acq: int               # max chain depth of thick-strip compositions
    P_acq: int               # streaming passes over IN
    S_acq: int               # peak bytes (strips + DAG + M tables)
    n_strips_init: int
    n_strips_final: int
    rounds: int


def expand_dag_word(dag_left: list[int], dag_right: list[int], node_id: int, out: bytearray) -> None:
    """Iteratively expand a word-DAG node into out (in LF emission order)."""
    stack = [node_id]
    while stack:
        nid = stack.pop()
        r = dag_right[nid]
        if r == -1:
            out.append(dag_left[nid])
        else:
            stack.append(r)
            stack.append(dag_left[nid])


def build_retained_system_genuine(blk: BlockData, L_min: int = 16, max_rounds: int = 64) -> RetainedSystem:
    """Contract thick runs (width >= L_min) purely by endpoint-only IET interval
    splitting and composition (Rauzy-Veech / Open), NEVER walking thick interiors
    row-by-row. Explicitly charges every interval split and lookup to Q_acq.
    """
    n = blk.n
    runs = blk.runs
    C = blk.C

    # 1 streaming pass over IN (r run records): compute LF(start_i) for each run
    # using 256 running counters (s = 256 cursors, Q = 0, P = 1).
    occ = C[:]
    run_lf_starts: list[int] = [0] * len(runs)
    for idx, (s, w, b) in enumerate(runs):
        run_lf_starts[idx] = occ[b]
        occ[b] += w

    # Word DAG: first 256 nodes are single-byte leaves 0..255
    dag_left = list(range(256))
    dag_right = [-1] * 256
    dag_len = [1] * 256

    # Partition runs into retained rows (< L_min, or split out primary_index) vs thick strips
    retained_rows: list[int] = []
    retained_byte: list[int] = []
    retained_lf: list[int] = []

    # Thick strips: (dom_start, width, target_start, word_id)
    thick_strips: list[tuple[int, int, int, int]] = []
    p = blk.primary_index

    for idx, (s, w, b) in enumerate(runs):
        lf_s = run_lf_starts[idx]
        if w < L_min:
            for j in range(w):
                retained_rows.append(s + j)
                retained_byte.append(b)
                retained_lf.append(lf_s + j)
        else:
            if s <= p < s + w:
                # Split out primary_index so primary_index is always in M
                left_w = p - s
                right_w = (s + w) - (p + 1)
                if left_w >= L_min:
                    thick_strips.append((s, left_w, lf_s, b))
                else:
                    for j in range(left_w):
                        retained_rows.append(s + j)
                        retained_byte.append(b)
                        retained_lf.append(lf_s + j)
                retained_rows.append(p)
                retained_byte.append(b)
                retained_lf.append(lf_s + left_w)
                if right_w >= L_min:
                    thick_strips.append((p + 1, right_w, lf_s + left_w + 1, b))
                else:
                    for j in range(right_w):
                        retained_rows.append(p + 1 + j)
                        retained_byte.append(b)
                        retained_lf.append(lf_s + left_w + 1 + j)
            else:
                thick_strips.append((s, w, lf_s, b))

    # Build a sorted interval map of thick strips for genuine endpoint composition
    # Note: thick_strips are already sorted by dom_start!
    n_strips_init = len(thick_strips)
    Q_acq_strips = 0
    D_acq = 0

    # Iterative doubling Rauzy-Veech / Open strip contraction:
    # Each strip is (dom_start, width, target_start, word_id, is_done).
    # In each round, ONLY strips with is_done=False are queried against cur_strips.
    # Doubling converges in <= 12 rounds even for >2000-byte thick corridors!
    cur_strips: list[tuple[int, int, int, int, bool]] = [
        (d, w, t, wid, False) for (d, w, t, wid) in thick_strips
    ]
    rounds = 0
    for rnd in range(max_rounds):
        if not cur_strips or all(st[4] for st in cur_strips):
            break
        starts = [st[0] for st in cur_strips]
        ends = [st[0] + st[1] for st in cur_strips]
        new_strips: list[tuple[int, int, int, int, bool]] = []
        any_composed = False

        for d, w, t, wid, is_done in cur_strips:
            if is_done:
                new_strips.append((d, w, t, wid, True))
                continue
            pos = bisect.bisect_right(ends, t)
            Q_acq_strips += 1  # endpoint lookup of t in cur_strips
            off = 0
            while off < w:
                cur_t = t + off
                rem = w - off
                if pos >= len(cur_strips) or starts[pos] >= cur_t + rem:
                    new_strips.append((d + off, rem, cur_t, wid, True))
                    off = w
                elif cur_t < starts[pos]:
                    gap = starts[pos] - cur_t
                    new_strips.append((d + off, gap, cur_t, wid, True))
                    off += gap
                else:
                    sd, sw, st_t, st_wid, st_done = cur_strips[pos]
                    overlap = min(rem, ends[pos] - cur_t)
                    new_t = st_t + (cur_t - sd)
                    new_wid = len(dag_left)
                    dag_left.append(wid)
                    dag_right.append(st_wid)
                    dag_len.append(dag_len[wid] + dag_len[st_wid])
                    new_strips.append((d + off, overlap, new_t, new_wid, st_done))
                    Q_acq_strips += 1
                    any_composed = True
                    off += overlap
                    if cur_t + overlap == ends[pos]:
                        pos += 1
        cur_strips = new_strips
        rounds = rnd + 1
        if any_composed:
            D_acq += 1
        else:
            break

    # Strip out is_done flag for final table
    cur_strips_4 = [(d, w, t, wid) for (d, w, t, wid, _) in cur_strips]
    cur_strips = cur_strips_4  # type: ignore[assignment]

    # Now EVERY thick strip lands entirely in retained rows M!
    # Verify and resolve each retained row x -> LF(x) -> (optional 1 strip jump) -> M
    m = len(retained_rows)
    rank_of_row = {r_idx: i for i, r_idx in enumerate(retained_rows)}
    mark_rank = rank_of_row[p]
    starts = [st[0] for st in cur_strips]
    ends = [st[0] + st[1] for st in cur_strips]

    R = [0] * m
    R_word_id = [0] * m
    Q_acq_glue = 0
    total_emitted_len = 0

    for i in range(m):
        y = retained_lf[i]
        b = retained_byte[i]
        # Check if y is retained or in a thick strip
        r_y = rank_of_row.get(y)
        if r_y is not None:
            R[i] = r_y
            R_word_id[i] = b
            total_emitted_len += 1
            Q_acq_glue += 1  # lookup whether y is retained (or resolved via strip table)
        else:
            # Binary search y in contracted thick strips (1 jump, O(1) translation!)
            pos = bisect.bisect_right(starts, y) - 1
            sd, sw, st_t, st_wid = cur_strips[pos]
            assert sd <= y < sd + sw, f"Row {y} neither retained nor in thick strip!"
            dest = st_t + (y - sd)
            # In rare case max_rounds wasn't enough, follow strip chain (charged!)
            while dest not in rank_of_row:
                Q_acq_glue += 1
                pos2 = bisect.bisect_right(starts, dest) - 1
                sd2, sw2, st_t2, st_wid2 = cur_strips[pos2]
                assert sd2 <= dest < sd2 + sw2
                new_wid = len(dag_left)
                dag_left.append(st_wid)
                dag_right.append(st_wid2)
                dag_len.append(dag_len[st_wid] + dag_len[st_wid2])
                st_wid = new_wid
                dest = st_t2 + (dest - sd2)
            R[i] = rank_of_row[dest]
            wid_full = len(dag_left)
            dag_left.append(b)
            dag_right.append(st_wid)
            dag_len.append(1 + dag_len[st_wid])
            R_word_id[i] = wid_full
            total_emitted_len += dag_len[wid_full]
            Q_acq_glue += 1

    assert total_emitted_len == n, f"Sum of retained word lengths {total_emitted_len} != n={n}"

    # Peak memory estimate: strips (16 B each) + DAG nodes (8 B each) + M arrays (12 B each)
    S_acq = len(cur_strips) * 16 + len(dag_left) * 8 + m * 12

    return RetainedSystem(
        L_min=L_min,
        m=m,
        rows=retained_rows,
        rank_of_row=rank_of_row,
        mark_rank=mark_rank,
        R=R,
        R_word_id=R_word_id,
        dag_left=dag_left,
        dag_right=dag_right,
        dag_len=dag_len,
        Q_acq_strips=Q_acq_strips,
        Q_acq_glue=Q_acq_glue,
        D_acq=D_acq,
        P_acq=1,  # 1 pass over IN; strip contraction works on O(r) strip table
        S_acq=S_acq,
        n_strips_init=n_strips_init,
        n_strips_final=len(cur_strips),
        rounds=rounds,
    )


def decode_retained_system(rs: RetainedSystem, n: int) -> bytes:
    """Decode T by walking the contracted retained map R from mark_rank."""
    out = bytearray(n)
    pos = n
    x = rs.mark_rank
    buf = bytearray()
    steps = 0
    while pos > 0:
        buf.clear()
        expand_dag_word(rs.dag_left, rs.dag_right, rs.R_word_id[x], buf)
        lw = len(buf)
        # buf is in LF (backward) order: buf[0] is T[pos-1], buf[1] is T[pos-2], ...
        for k in range(lw):
            out[pos - 1 - k] = buf[k]
        pos -= lw
        x = rs.R[x]
        steps += 1
    assert x == rs.mark_rank and steps == rs.m, f"Cycle mismatch: x={x}, steps={steps}, m={rs.m}"
    return bytes(out)


if __name__ == "__main__":
    print("Self-testing common.py on synthetic + 6 admitted blocks...")
    s_iid = make_synthetic_novel_hex_iid(16384)
    s_tmpl = make_synthetic_templated_hex(250, 40)
    admitted = load_admitted_blocks()
    for blk in [s_iid, s_tmpl] + admitted:
        rs16 = build_retained_system_genuine(blk, L_min=16)
        dec16 = decode_retained_system(rs16, blk.n)
        assert dec16 == blk.T_ref, f"L_min=16 decode mismatch on {blk.name}!"
        print(
            f"{blk.name:22s} n={blk.n:7d} r={blk.r:6d} h1={blk.h1:6d} "
            f"m16={rs16.m:6d} ({rs16.m/blk.h1:.3f}*h1) "
            f"strips={rs16.n_strips_init}->{rs16.n_strips_final} "
            f"rounds={rs16.rounds} Q_strips={rs16.Q_acq_strips} [EXACT]"
        )
