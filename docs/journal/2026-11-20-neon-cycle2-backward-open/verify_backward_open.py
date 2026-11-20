#!/usr/bin/env python3
"""
Exact verification of Queue-Free Backward Open (g = P o LF_{L[S]}).

Eliminates Open's queues[0..256] (360-817 KiB) and scattered edge writes by
evaluating g = P o LF_v directly in the backward (LF) direction, writing
edges[0..m] strictly sequentially (j = 0..m-1) and emitting out[0..n]
right-to-left (with exact short-cycle period extraction & forward repeat).

Verifies:
1. Exhaustive small cases: all binary columns n=1..8 and all ternary columns
   n=1..5, at EVERY primary mark p in 0..n-1 and LIMIT in {1, 2, 4, 16}
   (20,908 total cases, including multi-cycle, fixed-point, and non-dividing
   short periods!).
2. Byte-exact output against dense LF walk on all six authenticated Wikidata
   blocks from data/admitted-bwt (verified by SHA-256 against manifest.json).
"""

import hashlib
import itertools
import json
import sys
from collections import defaultdict
from pathlib import Path


def dense_bwt_invert(L, p):
    n = len(L)
    counts = defaultdict(int)
    for c in L:
        counts[c] += 1
    syms = sorted(counts)
    starts = {}
    tot = 0
    for c in syms:
        starts[c] = tot
        tot += counts[c]
    LF = [0] * n
    cur = dict(starts)
    for i, c in enumerate(L):
        LF[i] = cur[c]
        cur[c] += 1
    Psi = [0] * n
    for i, j in enumerate(LF):
        Psi[j] = i
    F = [None] * n
    for c in syms:
        for k in range(starts[c], starts[c] + counts[c]):
            F[k] = c
    out = []
    x = p
    for _ in range(n):
        out.append(F[x])
        x = Psi[x]
    return out


def dense_invert_bytes(L: bytes, p: int) -> bytes:
    n = len(L)
    C = [0] * 256
    for c in L:
        C[c] += 1
    starts = [0] * 256
    tot = 0
    for c in range(256):
        starts[c] = tot
        tot += C[c]
    LF = [0] * n
    cur = starts[:]
    for i, c in enumerate(L):
        LF[i] = cur[c]
        cur[c] += 1
    cyc = bytearray()
    x = p
    for _ in range(n):
        cyc.append(L[x])
        x = LF[x]
        if x == p:
            break
    cyc.reverse()
    if len(cyc) == n:
        return bytes(cyc)
    out = bytearray()
    while len(out) < n:
        take = min(len(cyc), n - len(out))
        out.extend(cyc[:take])
    return bytes(out)


def records_of(L):
    recs = []
    for b in L:
        if recs and recs[-1][0] == b and recs[-1][1] < 127:
            recs[-1] = (b, recs[-1][1] + 1)
        else:
            recs.append((b, 1))
    return recs


def each_run(recs, p, limit):
    pos = 0
    i = 0
    nrec = len(recs)
    while i < nrec:
        c, w = recs[i]
        i += 1
        while i < nrec and recs[i][0] == c:
            w += recs[i][1]
            i += 1
        end = pos + w
        if w > limit and pos <= p < end:
            if pos < p:
                yield (c, pos, p - pos, False)
            yield (c, p, 1, True)
            if p + 1 < end:
                yield (c, p + 1, end - p - 1, False)
        else:
            yield (c, pos, w, w <= limit)
        pos = end


def backward_open_invert(L, p, limit=16):
    """
    Queue-free Backward Open: computes g(j) = P(LF_{L[S]}(j)) directly.
    No queues[0..256]; edges[0..m] written strictly sequentially j = 0..m-1.
    """
    n = len(L)
    recs = records_of(L)

    # Pass 1: count symbols and extract ONLY thick strips (zero queues!)
    all_cnt = [0] * 256
    thin_cnt = [0] * 256
    thick_cnt = [0] * 256
    strips = []  # (from, to_rel, width, byte)
    m = 0
    mark = 0
    for c, x, w, keep in each_run(recs, p, limit):
        if keep:
            if x <= p < x + w:
                mark = m + (p - x)
            thin_cnt[c] += w
            m += w
        else:
            strips.append([x, all_cnt[c], w, c])
            thick_cnt[c] += 1
        all_cnt[c] += w

    first = [0] * 256
    write = [0] * 256
    tot = ttot = 0
    for a in range(256):
        first[a] = tot
        tot += all_cnt[a]
        write[a] = ttot
        ttot += thick_cnt[a]

    image_order = [0] * len(strips)
    for i, s in enumerate(strips):
        a = s[3]
        s[1] += first[a]
        image_order[write[a]] = i
        write[a] += 1

    # Pass 2: Endpoint induction (identical to src/open/endpoint.rs, H strips only)
    # nodes[0] is sentinel; node fields: [prev0, prev1, next0, next1, width, word]
    nodes = [[0, 0, 0, 0, 0, 0]]
    rules = []  # (left, right, length) -> id = 257 + idx

    def add_node(width, word):
        nid = len(nodes)
        nodes.append([0, 0, 0, 0, width, word])
        return nid

    for s in strips:
        add_node(s[2], s[3] + 1)

    ord0, ord1, term0, term1 = [], [], [], []
    x = 0
    for i, s in enumerate(strips):
        if x < s[0]:
            nid = add_node(s[0] - x, 0)
            ord0.append(nid)
            term1.append(nid)
        ord0.append(i + 1)
        x = s[0] + s[2]
    if x < n:
        nid = add_node(n - x, 0)
        ord0.append(nid)
        term1.append(nid)

    x = 0
    for i in image_order:
        s = strips[i]
        if x < s[1]:
            nid = add_node(s[1] - x, 0)
            ord1.append(nid)
            term0.append(nid)
        ord1.append(i + 1)
        x = s[1] + s[2]
    if x < n:
        nid = add_node(n - x, 0)
        ord1.append(nid)
        term0.append(nid)

    for side, seq in [(0, term0 + ord0), (1, term1 + ord1)]:
        prev = 0
        for nid in seq:
            nodes[prev][2 + side] = nid
            nodes[nid][side] = prev
            prev = nid
        nodes[prev][2 + side] = 0
        nodes[0][side] = prev

    def cut(nid, side):
        pr = nodes[nid][side]
        nx = nodes[nid][2 + side]
        nodes[pr][2 + side] = nx
        nodes[nx][side] = pr

    def insert(at, nid, side):
        nx = nodes[at][2 + side]
        nodes[nid][side] = at
        nodes[nid][2 + side] = nx
        nodes[at][2 + side] = nid
        nodes[nx][side] = nid

    def join(lw, rw):
        if lw == 0:
            return rw
        if rw == 0:
            return lw
        la = 1 if lw <= 256 else rules[lw - 257][2]
        lb = 1 if rw <= 256 else rules[rw - 257][2]
        rid = 257 + len(rules)
        rules.append((lw, rw, la + lb))
        return rid

    rem = n
    while rem > 0:
        ix = nodes[0][0]
        iy = nodes[0][1]
        wx = nodes[ix][4]
        wy = nodes[iy][4]
        w = wx if wx < wy else wy
        rem -= w
        if ix == iy:
            cut(ix, 0)
            cut(ix, 1)
            continue
        word = join(nodes[ix][5], nodes[iy][5])
        if wx <= wy:
            cut(ix, 0)
            insert(iy, ix, 0)
            nodes[ix][5] = word
            if wx == wy:
                cut(iy, 0)
                cut(iy, 1)
            else:
                nodes[iy][4] -= w
        else:
            cut(iy, 1)
            insert(ix, iy, 1)
            nodes[iy][5] = word
            nodes[ix][4] -= w

    # Extract bottom coordinates and bands (exact same forward word rev(w_k) + a!)
    bottom = [0] * len(nodes)
    nid = nodes[0][3]
    x = 0
    while nid != 0:
        bottom[nid] = x
        x += nodes[nid][4]
        nid = nodes[nid][3]
    assert x == m

    spelling = bytearray()
    offsets = [0]
    first_occ = {}  # rule_id -> (start, len)

    def spell_word(wid):
        st = [wid]
        while st:
            cur = st.pop()
            if cur <= 256:
                spelling.append(cur - 1)
            elif cur in first_occ:
                s0, ln = first_occ[cur]
                spelling.extend(spelling[s0 : s0 + ln])
            else:
                lw, rw, ln = rules[cur - 257]
                first_occ[cur] = (len(spelling), ln)
                st.append(rw)
                st.append(lw)

    bands = []  # (end, delta, code)
    cursors = [0] * 256
    nid = nodes[0][2]
    in_node = 0
    k = 0
    for a in range(256):
        cursors[a] = len(bands)
        end = k + thin_cnt[a]
        while k < end:
            nd = nodes[nid]
            w = min(end - k, nd[4] - in_node)
            dst = bottom[nid] + in_node
            if nd[5] == 0:
                code = a
            else:
                spell_word(nd[5])
                spelling.append(a)
                offsets.append(len(spelling))
                code = 254 + len(offsets)
            bands.append((k + w, dst - k, code))
            in_node += w
            k += w
            if in_node == nd[4]:
                nid = nd[2]
                in_node = 0

    # Pass 3: DIRECT SEQUENTIAL FILL of edges[0..m] (g(j) = P(LF_v(j))) via L1-resident band cache!
    # ZERO queues! ZERO scattered writes into edges!
    edges_next = [0] * m
    edges_code = [0] * m
    rank = [0] * 256
    tot = 0
    for a in range(256):
        rank[a] = tot
        tot += thin_cnt[a]

    # Active band cache per symbol (L1-resident: 256 entries)
    b_idx = cursors[:]
    b_end = [0] * 256
    b_delta = [0] * 256
    b_code = [0] * 256
    for a in range(256):
        if thin_cnt[a] > 0:
            be, bd, bc = bands[b_idx[a]]
            b_end[a] = be
            b_delta[a] = bd
            b_code[a] = bc

    j = 0
    band_splits = 0
    thin_runs = 0
    for c, _, w, keep in each_run(recs, p, limit):
        if not keep:
            continue
        thin_runs += 1
        rk = rank[c]
        rem_w = w
        if rk + rem_w <= b_end[c]:
            # Fast path (95%+ of thin runs!): strictly inside current band!
            dst = rk + b_delta[c]
            bc = b_code[c]
            for s in range(rem_w):
                edges_next[j + s] = dst + s
                edges_code[j + s] = bc
            j += rem_w
            rk += rem_w
            if rk == b_end[c] and rk < (rank[c + 1] if c + 1 < 256 else m):
                pass  # lazy reload on next hit or immediate reload below
            rank[c] = rk
            if rk == b_end[c]:
                b_idx[c] += 1
                if b_idx[c] < (cursors[c + 1] if c + 1 < 256 else len(bands)):
                    b_end[c], b_delta[c], b_code[c] = bands[b_idx[c]]
        else:
            band_splits += 1
            while rem_w > 0:
                span = min(rem_w, b_end[c] - rk)
                dst = rk + b_delta[c]
                bc = b_code[c]
                for s in range(span):
                    edges_next[j + s] = dst + s
                    edges_code[j + s] = bc
                j += span
                rk += span
                rem_w -= span
                if rk == b_end[c]:
                    b_idx[c] += 1
                    if b_idx[c] < (cursors[c + 1] if c + 1 < 256 else len(bands)):
                        b_end[c], b_delta[c], b_code[c] = bands[b_idx[c]]
            rank[c] = rk
    assert j == m

    # Pass 4: Backward emission walk from `mark`, writing right-to-left out[n..0]
    out = bytearray(n)
    pos = n
    cur = mark
    for _ in range(m):
        code = edges_code[cur]
        cur = edges_next[cur]
        if code < 256:
            pos -= 1
            out[pos] = code
        else:
            wi = code - 256
            s0, s1 = offsets[wi], offsets[wi + 1]
            ln = s1 - s0
            pos -= ln
            out[pos : pos + ln] = spelling[s0:s1]
        if cur == mark or pos == 0:
            break

    if pos > 0:
        # Short cycle: exact period word U sits in out[pos..n] in forward order!
        l_cyc = n - pos
        unit = bytes(out[pos:n])
        wpos = 0
        while wpos < n:
            take = min(l_cyc, n - wpos)
            out[wpos : wpos + take] = unit[:take]
            wpos += take

    return bytes(out), thin_runs, band_splits


def main():
    # 1. Exhaustive small cases across 4 limits!
    cases = 0
    for sigma, max_n in [(2, 8), (3, 5)]:
        for n in range(1, max_n + 1):
            for L_tup in itertools.product(range(sigma), repeat=n):
                L_bytes = bytes(L_tup)
                for p in range(n):
                    ref = bytes(dense_bwt_invert(L_tup, p))
                    for lim in (1, 2, 4, 16):
                        got, _, _ = backward_open_invert(L_bytes, p, limit=lim)
                        assert got == ref, f"MISMATCH L={L_tup} p={p} lim={lim}: got={list(got)} ref={list(ref)}"
                        cases += 1
    print(f"[G1-SMALL] PASS: {cases} exhaustive cases (binary n<=8, ternary n<=5, all primary marks, LIMIT in {{1,2,4,16}})")

    if len(sys.argv) < 2:
        return
    bdir = Path(sys.argv[1])
    manifest = json.loads((bdir / "manifest.json").read_text())
    tot_bytes = 0
    for blk in manifest["blocks"]:
        L = (bdir / blk["file"]).read_bytes()
        assert hashlib.sha256(L).hexdigest() == blk["sha256"]
        p = blk["primary_index"]
        ref = dense_invert_bytes(L, p)
        got, truns, bsplits = backward_open_invert(L, p, limit=16)
        assert got == ref, f"MISMATCH on {blk['file']}"
        tot_bytes += len(L)
        print(
            f"[G1-REAL]  PASS: {blk['file']:12s} n={len(L)} thin_runs={truns:6d} "
            f"band_splits={bsplits:4d} ({bsplits/truns:.2%})"
        )
    print(f"[G1-REAL]  ALL 6 BLOCKS BYTE-EXACT: {tot_bytes:,} / {tot_bytes:,} bytes verified!")


if __name__ == "__main__":
    main()
