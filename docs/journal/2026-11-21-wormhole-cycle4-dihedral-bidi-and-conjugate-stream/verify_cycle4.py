#!/usr/bin/env python3
"""Cycle 4 Verifier: Dihedral Zero-Copy Meet-in-the-Middle (Bidi-Open) & Conjugate u16-Stream Open (CSW-Open).

Verifies:
  1. Theorem 4.1 (Dihedral Time-Reversal Zero-Copy Meet-in-the-Middle Orbit Walk, Bidi-Open):
     - Forward Psi-ray from (mark, at_f=0) and backward LF-ray from (mark, at_b=n) partition any
       marked cycle of retained length m_cyc >= 1 into ceil(m_cyc/2) + floor(m_cyc/2) disjoint
       steps meeting at j_f == j_b, with at_f == at_b (zero staging buffers, zero extra copies)
       on all primitive blocks, and exact memmove+repeat closure when d < n.
     - Verified byte-exact across 26,495 small + random multi-cycle (L, p, Lambda) instances
       (including m_cyc = 1, 2, 3, odd/even, periodic, and multi-cycle) + all 6 Wikidata blocks.
  2. Theorem 4.2 (Conjugate Skyscraper First-Return g = P o pi_Q^-1 & u16-Stream Vec16 Wiring, CSW-Open):
     - Exact arrow reversal edges_bwd[src] = (code, dst) <=> edges_fwd[dst] = (code, src) built
       from a single 2B/run thin_stream with 0 B 256-way queues, branchless single-band fast-path
       rate census (93.4%-97.1% of thin runs!), and right-aligned 16B downward emission with
       zero front-padding shift on primitive blocks.
     - Verified byte-exact across all 26,495 small + multi-cycle instances + all 6 Wikidata blocks.
"""

from __future__ import annotations

import hashlib
import json
import os
import pathlib


def plain_invert(L: bytes, p: int) -> bytes:
    n = len(L)
    counts = [0] * 256
    for c in L:
        counts[c] += 1
    ftab = [0] * 256
    acc = 0
    for a in range(256):
        ftab[a] = acc
        acc += counts[a]
    psi = [0] * n
    for i, c in enumerate(L):
        k = ftab[c]
        psi[k] = i
        ftab[c] = k + 1
    out = bytearray(n)
    cur = psi[p]
    for t in range(n):
        out[t] = L[cur]
        cur = psi[cur]
    return bytes(out)


def build_common_endpoint(L: bytes, p: int, limit: int):
    """Run Pass 1 (recording 2B thin_stream + thick strips) and Endpoint elimination."""
    n = len(L)
    all_c = [0] * 256
    thin_c = [0] * 256
    thick_c = [0] * 256
    strips = []
    thin_stream = []  # (byte, width) pairs — 2 bytes per thin piece!
    queues = [[] for _ in range(256)]
    m = 0
    mark = 0
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        w = j - i
        pos = i
        i = j

        def piece(x: int, pw: int, keep: bool):
            nonlocal m, mark
            if keep:
                if x <= p < x + pw:
                    mark = m + (p - x)
                thin_stream.append((c, pw))
                queues[c].extend(range(m, m + pw))
                thin_c[c] += pw
                m += pw
            else:
                strips.append([x, all_c[c], pw, c])
                thick_c[c] += 1
            all_c[c] += pw

        if w <= limit:
            piece(pos, w, True)
        elif pos <= p < pos + w:
            if pos < p:
                piece(pos, p - pos, False)
            piece(p, 1, True)
            if p + 1 < pos + w:
                piece(p + 1, pos + w - p - 1, False)
        else:
            piece(pos, w, False)

    first = [0] * 256
    acc = 0
    for a in range(256):
        first[a] = acc
        acc += all_c[a]
    write = [0] * 256
    acc = 0
    for a in range(256):
        write[a] = acc
        acc += thick_c[a]
    H = len(strips)
    image_order = [0] * H
    for idx, s in enumerate(strips):
        a = s[3]
        s[1] += first[a]
        image_order[write[a]] = idx
        write[a] += 1

    max_nodes = 1 + 3 * H + 4
    prev0 = [0] * max_nodes
    prev1 = [0] * max_nodes
    next0 = [0] * max_nodes
    next1 = [0] * max_nodes
    n_width = [0] * max_nodes
    n_word = [0] * max_nodes
    num_nodes = 1

    def add_node(w: int, word: int):
        nonlocal num_nodes
        nid = num_nodes
        num_nodes += 1
        n_width[nid] = w
        n_word[nid] = word
        return nid

    order0, order1, term0, term1 = [], [], [], []
    for s in strips:
        add_node(s[2], s[3] + 1)
    x = 0
    for idx, s in enumerate(strips):
        if x < s[0]:
            nid = add_node(s[0] - x, 0)
            order0.append(nid)
            term1.append(nid)
        order0.append(idx + 1)
        x = s[0] + s[2]
    if x < n:
        nid = add_node(n - x, 0)
        order0.append(nid)
        term1.append(nid)

    x = 0
    for idx in image_order:
        s = strips[idx]
        if x < s[1]:
            nid = add_node(s[1] - x, 0)
            order1.append(nid)
            term0.append(nid)
        order1.append(idx + 1)
        x = s[1] + s[2]
    if x < n:
        nid = add_node(n - x, 0)
        order1.append(nid)
        term0.append(nid)

    prev = 0
    for nid in term0 + order0:
        next0[prev] = nid
        prev0[nid] = prev
        prev = nid
    next0[prev] = 0
    prev0[0] = prev

    prev = 0
    for nid in term1 + order1:
        next1[prev] = nid
        prev1[nid] = prev
        prev = nid
    next1[prev] = 0
    prev1[0] = prev

    rules = []

    def wlen(w: int) -> int:
        if w <= 256:
            return 1 if w != 0 else 0
        return rules[w - 257][2]

    def join(lw: int, rw: int) -> int:
        if lw == 0:
            return rw
        if rw == 0:
            return lw
        rid = len(rules) + 257
        rules.append([lw, rw, wlen(lw) + wlen(rw), -1])
        return rid

    rem = n
    while rem > 0:
        nx = prev0[0]
        ny = prev1[0]
        aw = n_width[nx]
        bw = n_width[ny]
        w = aw if aw < bw else bw
        rem -= w
        if nx == ny:
            p0, q0 = prev0[nx], next0[nx]
            next0[p0], prev0[q0] = q0, p0
            p1, q1 = prev1[nx], next1[nx]
            next1[p1], prev1[q1] = q1, p1
            continue
        wj = join(n_word[nx], n_word[ny])
        if aw <= bw:
            p0, q0 = prev0[nx], next0[nx]
            next0[p0], prev0[q0] = q0, p0
            qn = next0[ny]
            prev0[nx], next0[nx] = ny, qn
            next0[ny], prev0[qn] = nx, nx
            n_word[nx] = wj
            if aw == bw:
                p0, q0 = prev0[ny], next0[ny]
                next0[p0], prev0[q0] = q0, p0
                p1, q1 = prev1[ny], next1[ny]
                next1[p1], prev1[q1] = q1, p1
            else:
                n_width[ny] -= w
        else:
            p1, q1 = prev1[ny], next1[ny]
            next1[p1], prev1[q1] = q1, p1
            qn = next1[nx]
            prev1[ny], next1[ny] = nx, qn
            next1[nx], prev1[qn] = ny, ny
            n_word[ny] = wj
            n_width[nx] -= w

    bottom = [0] * num_nodes
    nid = next1[0]
    x = 0
    while nid != 0:
        bottom[nid] = x
        x += n_width[nid]
        nid = next1[nid]
    assert x == m

    spelling = bytearray()
    offsets = [0]
    stk = []

    def append_slp(wd: int):
        stk.clear()
        stk.append(wd)
        while stk:
            cur = stk.pop()
            if cur <= 256:
                spelling.append(cur - 1)
            else:
                rl = rules[cur - 257]
                if rl[3] == -1:
                    rl[3] = len(spelling)
                    stk.append(rl[1])
                    stk.append(rl[0])
                else:
                    st = rl[3]
                    spelling.extend(spelling[st : st + rl[2]])

    bands = []  # (end, delta, code)
    cursors = [0] * 257
    nid = next0[0]
    in_node = 0
    k = 0
    nwords = 0
    for a in range(256):
        cursors[a] = len(bands)
        end = k + thin_c[a]
        while k < end:
            nw = n_width[nid]
            width = min(end - k, nw - in_node)
            dst = bottom[nid] + in_node
            wd = n_word[nid]
            if wd == 0:
                code = a
            else:
                nwords += 1
                append_slp(wd)
                spelling.append(a)
                offsets.append(len(spelling))
                code = 255 + nwords
            bands.append((k + width, (dst - k) & 0xFFFFFFFF, code))
            in_node += width
            k += width
            if in_node == nw:
                nid = next0[nid]
                in_node = 0
    cursors[256] = len(bands)

    return {
        "n": n,
        "m": m,
        "mark": mark,
        "thin_c": thin_c,
        "thin_stream": thin_stream,
        "queues": queues,
        "bands": bands,
        "cursors": cursors,
        "nwords": nwords,
        "spelling": bytes(spelling),
        "offsets": offsets,
    }


def word_bytes(st: dict, code: int) -> bytes:
    if code < 256:
        return bytes([code])
    w = code - 256
    return st["spelling"][st["offsets"][w] : st["offsets"][w + 1]]


def decode_chain4a_bidi(st: dict) -> tuple[bytes, dict]:
    """Chain 4A: Bidi-Open Zero-Copy Meet-in-the-Middle Orbit Walk."""
    n = st["n"]
    m = st["m"]
    mark = st["mark"]
    bands = st["bands"]
    cursors = st["cursors"]
    queues = st["queues"]

    edges_fwd = [(0, 0)] * m  # dst -> (code, src)
    edges_bwd = [(0, 0)] * m  # src -> (code, dst)
    k = 0
    for a in range(256):
        from_idx = 0
        qa = queues[a]
        for bi in range(cursors[a], cursors[a + 1]):
            b_end, b_delta, b_code = bands[bi]
            span = b_end - k
            dst = (k + b_delta) & 0xFFFFFFFF
            for d in range(span):
                src = qa[from_idx + d]
                edges_fwd[dst + d] = (b_code, src)
                edges_bwd[src] = (b_code, dst + d)
            from_idx += span
            k = b_end

    # Build left-aligned and right-aligned 16B inline slots
    ncodes = 256 + st["nwords"]
    inl_left = [b"\x00" * 16] * ncodes
    inl_right = [b"\x00" * 16] * ncodes
    lens = [0] * ncodes
    for c in range(ncodes):
        wb = word_bytes(st, c)
        if len(wb) <= 16:
            lens[c] = len(wb)
            inl_left[c] = wb + b"\x00" * (16 - len(wb))
            inl_right[c] = b"\x00" * (16 - len(wb)) + wb
        else:
            lens[c] = 255

    out = bytearray(n + 16)
    at_f = 0
    at_b = n
    j_f = mark
    j_b = mark
    steps_f = 0
    steps_b = 0

    for _ in range(m):
        # Forward step: use 16B unconditional vector copy while safe gap > 32
        code_f, nxt_f = edges_fwd[j_f]
        j_f = nxt_f
        steps_f += 1
        ln_f = lens[code_f]
        if ln_f != 255 and (at_b - at_f) >= 32:
            out[at_f : at_f + 16] = inl_left[code_f]
            at_f += ln_f
        else:
            wb = word_bytes(st, code_f)
            take = min(len(wb), at_b - at_f)
            out[at_f : at_f + take] = wb[:take]
            at_f += take
        if j_f == j_b or at_f >= at_b:
            break

        # Backward step: use 16B right-aligned unconditional copy while safe gap > 32
        code_b, nxt_b = edges_bwd[j_b]
        j_b = nxt_b
        steps_b += 1
        ln_b = lens[code_b]
        if ln_b != 255 and (at_b - at_f) >= 32:
            out[at_b - 16 : at_b] = inl_right[code_b]
            at_b -= ln_b
        else:
            wb = word_bytes(st, code_b)
            take = min(len(wb), at_b - at_f)
            out[at_b - take : at_b] = wb[len(wb) - take :]
            at_b -= take
        if j_f == j_b or at_f >= at_b:
            break

    # If primitive single-cycle (d == n): at_f == at_b identically, ZERO memmove!
    zero_copy_exact = (at_f == at_b)
    if at_f < at_b:
        tail_len = n - at_b
        out[at_f : at_f + tail_len] = out[at_b:n]
        d = at_f + tail_len
    else:
        d = n
    res = bytearray(out[:d])
    while len(res) < n:
        take = min(len(res), n - len(res))
        res.extend(res[:take])

    return bytes(res), {
        "steps_f": steps_f,
        "steps_b": steps_b,
        "zero_copy_primitive": zero_copy_exact,
    }


def decode_chain4b_csw(st: dict) -> tuple[bytes, dict]:
    """Chain 4B: CSW-Open (u16 thin_stream + 16-wide sequential iota wiring + right-aligned emit)."""
    n = st["n"]
    m = st["m"]
    mark = st["mark"]
    thin_c = st["thin_c"]
    bands = st["bands"]
    cursors = st["cursors"]
    thin_stream = st["thin_stream"]

    # Initialize ActiveBand[256] per symbol
    k_starts = [0] * 256
    acc = 0
    for a in range(256):
        k_starts[a] = acc
        acc += thin_c[a]

    ab_cur_k = list(k_starts)
    ab_bi = [cursors[a] for a in range(256)]
    ab_end = [0] * 256
    ab_delta = [0] * 256
    ab_code = [0] * 256
    for a in range(256):
        if thin_c[a] > 0:
            be, bd, bc = bands[ab_bi[a]]
            ab_end[a], ab_delta[a], ab_code[a] = be, bd, bc

    # Sequential edges_bwd[0 .. m + 16) filled via 16-wide branchless iota stamp!
    edges_bwd = [(0, 0)] * (m + 16)
    j = 0
    fast_16_hits = 0
    split_runs = 0
    for a, w in thin_stream:
        z = ab_cur_k[a]
        if w <= 16 and z + w <= ab_end[a]:
            # Fast path (>93% of thin runs!): ONE unconditional 16-entry iota write!
            fast_16_hits += 1
            dst0 = (z + ab_delta[a]) & 0xFFFFFFFF
            cd = ab_code[a]
            for i in range(16):
                edges_bwd[j + i] = (cd, (dst0 + i) & 0xFFFFFFFF)
            j += w
            z += w
            ab_cur_k[a] = z
            if z == ab_end[a] and ab_bi[a] + 1 < cursors[a + 1]:
                ab_bi[a] += 1
                be, bd, bc = bands[ab_bi[a]]
                ab_end[a], ab_delta[a], ab_code[a] = be, bd, bc
        else:
            split_runs += 1
            end_z = z + w
            while z < end_z:
                span = min(end_z, ab_end[a]) - z
                dst0 = (z + ab_delta[a]) & 0xFFFFFFFF
                cd = ab_code[a]
                for d in range(span):
                    edges_bwd[j + d] = (cd, (dst0 + d) & 0xFFFFFFFF)
                j += span
                z += span
                if z == ab_end[a] and ab_bi[a] + 1 < cursors[a + 1]:
                    ab_bi[a] += 1
                    be, bd, bc = bands[ab_bi[a]]
                    ab_end[a], ab_delta[a], ab_code[a] = be, bd, bc
            ab_cur_k[a] = end_z
    assert j == m

    # Build right-aligned 16B slots for downward emission
    ncodes = 256 + st["nwords"]
    inl_right = [b"\x00" * 16] * ncodes
    lens = [0] * ncodes
    for c in range(ncodes):
        wb = word_bytes(st, c)
        if len(wb) <= 16:
            lens[c] = len(wb)
            inl_right[c] = b"\x00" * (16 - len(wb)) + wb
        else:
            lens[c] = 255

    # Emit right-to-left directly inside out[0..n) with NO front-padding shift!
    out = bytearray(n)
    at = n
    cur_j = mark
    for _ in range(m):
        code, nxt_j = edges_bwd[cur_j]
        cur_j = nxt_j
        ln = lens[code]
        if ln != 255 and at >= 16:
            out[at - 16 : at] = inl_right[code]
            at -= ln
        else:
            wb = word_bytes(st, code)
            take = min(len(wb), at)
            out[at - take : at] = wb[len(wb) - take :]
            at -= take
        if cur_j == mark or at == 0:
            break

    if at > 0:
        d = n - at
        res = bytearray(out[at:n])
        while len(res) < n:
            take = min(len(res), n - len(res))
            res.extend(res[:take])
        return bytes(res), {
            "thin_runs": len(thin_stream),
            "fast_16_pct": round(100.0 * fast_16_hits / max(1, len(thin_stream)), 2),
            "thin_stream_kib": round(2 * len(thin_stream) / 1024.0, 1),
        }

    return bytes(out), {
        "thin_runs": len(thin_stream),
        "fast_16_pct": round(100.0 * fast_16_hits / max(1, len(thin_stream)), 2),
        "thin_stream_kib": round(2 * len(thin_stream) / 1024.0, 1),
    }


def run_small_exhaustive() -> int:
    cases = 0
    for base, max_n in ((2, 8), (3, 5)):
        for n in range(1, max_n + 1):
            for z in range(base**n):
                tmp = z
                col = bytearray(n)
                for i in range(n):
                    col[i] = tmp % base
                    tmp //= base
                L = bytes(col)
                for p in range(n):
                    ref = plain_invert(L, p)
                    for lim in (0, 1, 2, 4, 16):
                        st = build_common_endpoint(L, p, lim)
                        got_a, _ = decode_chain4a_bidi(st)
                        assert got_a == ref, f"Bidi mismatch L={L}, p={p}, lim={lim}"
                        got_b, _ = decode_chain4b_csw(st)
                        assert got_b == ref, f"CSW mismatch L={L}, p={p}, lim={lim}"
                        cases += 1
    seed = 12383217
    for n in (1, 31, 127, 128, 129, 385):
        for alpha in (1, 2, 11, 256):
            col = bytearray(n)
            for i in range(n):
                seed = (seed * 1664525 + 1013904223) & 0xFFFFFFFF
                col[i] = (seed >> 8) % alpha
            L = bytes(col)
            for p in (0, n // 2, n - 1):
                ref = plain_invert(L, p)
                for lim in (0, 1, 2, 4, 16):
                    st = build_common_endpoint(L, p, lim)
                    got_a, _ = decode_chain4a_bidi(st)
                    assert got_a == ref
                    got_b, _ = decode_chain4b_csw(st)
                    assert got_b == ref
                    cases += 1
    return cases


def main():
    small_cases = run_small_exhaustive()
    print(f"[G1 EXHAUSTIVE + MULTI-CYCLE] PASS: {small_cases} instances verified byte-exact for BOTH Bidi-Open & CSW-Open!")

    blocks_dir = pathlib.Path(os.environ.get("BLOCKS_DIR", "/home/antigravity/admitted-bwt"))
    manifest = json.loads((blocks_dir / "manifest.json").read_text())
    results = {"small_cases": small_cases, "blocks": []}

    for entry in manifest["blocks"]:
        fname = entry["file"]
        L = (blocks_dir / fname).read_bytes()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        n = entry["n"]
        p = entry["primary_index"]
        ref = plain_invert(L, p)

        st = build_common_endpoint(L, p, 16)
        got_a, info_a = decode_chain4a_bidi(st)
        assert got_a == ref and info_a["zero_copy_primitive"]
        got_b, info_b = decode_chain4b_csw(st)
        assert got_b == ref

        row = {
            "file": fname,
            "n": n,
            "r": entry["r"],
            "m": st["m"],
            **info_a,
            **info_b,
        }
        results["blocks"].append(row)
        print(
            f"[6-BLOCK EXACT] {fname:11s} | m={st['m']:6d} | "
            f"Bidi: steps=({info_a['steps_f']}+{info_a['steps_b']}), zero_copy={info_a['zero_copy_primitive']} | "
            f"CSW: thin_stream={info_b['thin_stream_kib']:5.1f}K (vs 3055-8059K Vec16!), fast16_hit={info_b['fast_16_pct']:5.2f}%"
        )

    out_path = pathlib.Path(__file__).resolve().parent / "cycle4_results.json"
    out_path.write_text(json.dumps(results, indent=2) + "\n")
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
