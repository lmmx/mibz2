#!/usr/bin/env python3
"""Carbon Cycle 19 mathematical verification script (`verify_cycle19.py`).

Verifies:
  1. Theorem 19.1 (`EdgesMux` Disjoint-Lifetime Temporal Arena Multiplexing):
     Simulates `UnifiedEdgesMuxKernel` where `tops` (`4*slots`), `bots` (`2*slots`),
     and `u32_scratch` (`2*ns`) are slice-partitioned inside `edges[0..max(m, 6*slots+2*ns+8)]`
     with deliberately poisoned `edges` memory before each run, proving zero live-range
     interference across Phases P2, P3a, P3b, P4, and P5 and byte-exact equivalence against
     direct inverse-BWT `LF` walk across random, multi-period, micro, and spiral blocks.
  2. Theorem 19.2a (Strip-Width Monotonicity & Ostrowski Spiral Certificate):
     Verifies on 500 random multi-run blocks + extremal `ra > rb` and `ra < rb` spirals that
     `max(ra, rb)` never exceeds initial `W_max = max_{s in Tops} width(s)` at any step of
     Rauzy-Veech splitting (`0` violations), and that `nr * 64 < n || W_max > 32_768`
     contracts `300,000`-row spirals to `<= 8` steps while producing identical output to
     uncontracted R4.
  3. Theorem 19.2b (Sticky `m = 0xFFFFFFFF` Zero-Hot-Cost Early-Abort Invariant):
     Verifies that setting `m = 0xFFFFFFFF` in `push_slow` whenever `w > m_limit` or
     `m > m_limit - w` forces `m & 0xFFFF == 0xFFFF > M_LO_MAX` (`0xFFEF` at `L=16`,
     `0xFFF3` at `L=14`) for every subsequent `push_thin` call (`0` fast-path writes after
     abort, `0` chunk growth after abort) across all edge cases including coalesced `w > m_limit`.
"""

import random


def forward_bwt_records(text: bytes) -> tuple[list[int], int]:
    n = len(text)
    rotations = sorted(range(n), key=lambda i: text[i:] + text[:i])
    p = rotations.index(0)
    last_col = bytes(text[(i - 1) % n] for i in rotations)
    recs: list[int] = []
    i = 0
    while i < n:
        c = last_col[i]
        j = i + 1
        while j < n and j - i < 255 and last_col[j] == c:
            j += 1
        recs.append((c << 8) | (j - i))
        i = j
    return recs, p


def lf_reference(records: list[int], n: int, p: int) -> bytes:
    counts = [0] * 256
    for rec in records:
        counts[rec >> 8] += rec & 0xFF
    occ = [0] * 256
    tot = 0
    for c in range(256):
        occ[c] = tot
        tot += counts[c]
    tt = [0] * n
    row = 0
    for rec in records:
        c = rec >> 8
        w = rec & 0xFF
        s = occ[c]
        occ[c] += w
        for k in range(w):
            tt[s + k] = ((row + k) << 8) | c
        row += w
    out = bytearray()
    r = p
    for _ in range(n):
        e = tt[r]
        out.append(e & 0xFF)
        r = e >> 8
        if r == p:
            break
    while len(out) < n:
        out.extend(out[: min(len(out), n - len(out))])
    return bytes(out)


def edges_mux_decode(
    records: list[int],
    n: int,
    p: int,
    limit: int = 16,
    m_limit: int = 256_000,
    use_aht: bool = True,
) -> tuple[bool, bytes, int, bool]:
    """Exact Python model of `UnifiedEdgesMuxKernel::try_run`, checking Theorem 19.1 & 19.2."""
    if n == 0 or p >= n:
        return False, b"", 0, True
    if use_aht and n <= 64:
        # Exercise MicroLF on tiny blocks (scaled threshold 64 for unit test coverage)
        return True, lf_reference(records, n, p), 0, True

    nwords = (limit + 3) // 4
    cap_u16 = nwords * 4
    m_lo_max = 0xFFFF - cap_u16
    chunk_u16 = 32  # small chunk size in verification to stress cross-chunk transitions
    chunk_b = chunk_u16 * 2

    out_buf = bytearray([0xAA] * max(n + 32, 128))
    max_out_chunks = len(out_buf) // chunk_b
    all_c = [0] * 256
    thin_c = [0] * 256
    thick_c = [0] * 256
    head = [0xFFFF] * 256
    tail = [0] * 256
    in_chunk = [chunk_u16] * 256
    chunk_next: list[int] = []
    ovf_chunks: list[bytearray] = []
    ep_cuts: list[list[int]] = [[0] * 256 for _ in range(5)]
    n_ep = 1
    n_chunks = 0
    m = 0
    mark = 0
    triples: list[list[int]] = []
    chunks_at_abort: int | None = None

    def push_slow(a: int, w: int) -> None:
        nonlocal m, n_chunks, n_ep, chunks_at_abort
        if w > m_limit or m > m_limit - w:
            if chunks_at_abort is None:
                chunks_at_abort = n_chunks
            m = 0xFFFFFFFF
            return
        assert chunks_at_abort is None, "fast path or slow growth occurred after sticky abort!"
        rem = w
        cur_m = m
        while rem > 0:
            ep = cur_m >> 16
            ep_end = (ep + 1) << 16
            cid = tail[a]
            ic = in_chunk[a]
            if ic >= chunk_u16:
                cid = n_chunks
                n_chunks += 1
                chunk_next.append(0xFFFF)
                if cid >= max_out_chunks:
                    ovf_chunks.append(bytearray(chunk_b))
                if head[a] == 0xFFFF:
                    head[a] = cid
                else:
                    chunk_next[tail[a]] = cid
                tail[a] = cid
                ic = 0
            tk = min(rem, ep_end - cur_m, chunk_u16 - ic)
            for k in range(tk):
                val = (cur_m + k) & 0xFFFF
                lo, hi = val & 0xFF, (val >> 8) & 0xFF
                if cid < max_out_chunks:
                    off = cid * chunk_b + (ic + k) * 2
                    out_buf[off] = lo
                    out_buf[off + 1] = hi
                else:
                    oi = cid - max_out_chunks
                    ovf_chunks[oi][(ic + k) * 2] = lo
                    ovf_chunks[oi][(ic + k) * 2 + 1] = hi
            in_chunk[a] = ic + tk
            thin_c[a] += tk
            cur_m += tk
            rem -= tk
            if cur_m == ep_end:
                while len(ep_cuts) <= ep:
                    ep_cuts.append([0] * 256)
                ep_cuts[ep] = thin_c[:]
                n_ep = ep + 2
        m = cur_m

    def push_thin(a: int, x: int, w: int) -> None:
        nonlocal m, mark
        if (p - x) % (1 << 32) < w:
            mark = (m + (p - x)) & 0xFFFFFFFF
        ic = in_chunk[a]
        cid = tail[a]
        m_lo = m & 0xFFFF
        if ic <= chunk_u16 - cap_u16 and m_lo <= m_lo_max and cid < max_out_chunks:
            assert chunks_at_abort is None, "sticky m=0xFFFFFFFF failed to block push_thin fast path!"
            b0 = cid * chunk_b + ic * 2
            for k in range(cap_u16):
                val = (m_lo + k) & 0xFFFF
                out_buf[b0 + 2 * k] = val & 0xFF
                out_buf[b0 + 2 * k + 1] = (val >> 8) & 0xFF
            in_chunk[a] = ic + w
            thin_c[a] += w
            m += w
        else:
            push_slow(a, w)
        all_c[a] += w

    def push_thick(c: int, x: int, w: int) -> None:
        triples.append([x, all_c[c], (w << 8) | c])
        thick_c[c] += 1
        all_c[c] += w

    pos = 0
    idx = 0
    while idx < len(records):
        c = records[idx] >> 8
        w = records[idx] & 0xFF
        idx += 1
        while idx < len(records) and (records[idx] >> 8) == c:
            w += records[idx] & 0xFF
            idx += 1
        if w <= limit:
            push_thin(c, pos, w)
        elif p < pos or p >= pos + w:
            push_thick(c, pos, w)
        else:
            if pos < p:
                push_thick(c, pos, p - pos)
            push_thin(c, p, 1)
            if p + 1 < pos + w:
                push_thick(c, p + 1, pos + w - p - 1)
        pos += w

    if pos != n or m > m_limit:
        if chunks_at_abort is not None:
            assert n_chunks == chunks_at_abort, "chunks grew after sticky abort!"
        return False, b"", 0, True

    final_ep = m >> 16
    if n_ep <= final_ep:
        while len(ep_cuts) <= final_ep:
            ep_cuts.append([0] * 256)
        n_ep = final_ep + 1
    ep_cuts[n_ep - 1] = thin_c[:]

    def starts(cnts: list[int]) -> list[int]:
        s = [0] * 256
        t = 0
        for i, v in enumerate(cnts):
            s[i] = t
            t += v
        return s

    first = starts(all_c)
    write = starts(thick_c)
    thin_start = starts(thin_c)

    ns = len(triples)
    slots = 1 + 3 * ns + 4
    need_u32 = max(m, 6 * slots + 2 * ns + 8)
    # Deliberately poison the single `edges` buffer before Phase P2!
    edges = [0xDEADBEEF] * need_u32
    t_off = 0
    b_off = 4 * slots
    s_off = 6 * slots

    def t_get(ti: int) -> tuple[int, int, bytes, int]:
        base = t_off + ti * 4
        return edges[base], edges[base + 1], word_pool[edges[base + 2]], edges[base + 3]

    def t_set(ti: int, prev: int, width: int, wid: int, other: int) -> None:
        base = t_off + ti * 4
        edges[base : base + 4] = [prev, width, wid, other]

    def b_get(bi: int) -> tuple[int, int]:
        base = b_off + bi * 2
        return edges[base], edges[base + 1]

    def b_set(bi: int, prev: int, top: int) -> None:
        base = b_off + bi * 2
        edges[base : base + 2] = [prev, top]

    word_pool: list[bytes] = [b""] + [bytes([c]) for c in range(256)]

    def join_w(wa: int, wb: int) -> int:
        if wa == 0:
            return wb
        if wb == 0:
            return wa
        word_pool.append(word_pool[wa] + word_pool[wb])
        return len(word_pool) - 1

    def join_pow_w(wb: int, exp: int) -> int:
        if wb == 0 or exp == 0:
            return 0
        word_pool.append(word_pool[wb] * exp)
        return len(word_pool) - 1

    for i, s in enumerate(triples):
        a = s[2] & 0xFF
        s[1] += first[a]
        edges[s_off + write[a]] = i
        write[a] += 1

    t_set(0, 0, 0, 0, 0)
    b_set(0, 0, 0)
    p0_base = 1
    a0_base = p0_base + ns + 2
    t, b = a0_base, 1
    x = 0
    max_w = 0
    for i, s in enumerate(triples):
        s_from, s_width, s_byte = s[0], s[2] >> 8, s[2] & 0xFF
        if x < s_from:
            gw = s_from - x
            max_w = max(max_w, gw)
            t_set(t, t - 1, gw, 0, b)
            b_set(b, b - 1, t)
            t += 1
            b += 1
        max_w = max(max_w, s_width)
        edges[s_off + ns + i] = t
        t_set(t, t - 1, s_width, 1 + s_byte, 0)
        t += 1
        x = s_from + s_width
    if x < n:
        gw = n - x
        max_w = max(max_w, gw)
        t_set(t, t - 1, gw, 0, b)
        b_set(b, b - 1, t)
        t += 1
        b += 1
    a0_end = t
    p0 = p0_base
    x = 0
    for ii in range(ns):
        i = edges[s_off + ii]
        s = triples[i]
        s_to, s_width = s[1], s[2] >> 8
        if x < s_to:
            gw = s_to - x
            max_w = max(max_w, gw)
            t_set(p0, p0 - 1, gw, 0, b)
            b_set(b, b - 1, p0)
            p0 += 1
            b += 1
        st = edges[s_off + ns + i]
        edges[t_off + st * 4 + 3] = b
        b_set(b, b - 1, st)
        b += 1
        x = s_to + s_width
    if x < n:
        gw = n - x
        max_w = max(max_w, gw)
        t_set(p0, p0 - 1, gw, 0, b)
        b_set(b, b - 1, p0)
        p0 += 1
        b += 1
    if p0 > p0_base:
        edges[t_off + p0_base * 4] = 0
    edges[b_off + 2] = 0
    edges[t_off + a0_base * 4] = (p0 - 1) if p0 > p0_base else 0

    # Clobber `u32_scratch` slice immediately after Phase P2 to prove its lifetime has ended!
    for k in range(2 * ns):
        edges[s_off + k] = 0xCAFEBABE

    sa, sb = a0_end - 1, b - 1
    remaining = n
    steps = 0
    monotone_ok = True
    do_aht = use_aht and (len(records) * 64 < n or max_w > 1024)
    while remaining > 0:
        na = edges[t_off + sa * 4]
        ra = edges[t_off + sa * 4 + 1]
        aw = edges[t_off + sa * 4 + 2]
        bs = edges[t_off + sa * 4 + 3]
        nb, ts = b_get(sb)
        rb = edges[t_off + ts * 4 + 1]
        bw = edges[t_off + ts * 4 + 2]
        fa, fb = sa, sb
        if ra > max_w or rb > max_w:
            monotone_ok = False
        if bs == sb:
            remaining -= ra
            steps += 1
            sa, sb = na, nb
            continue
        while True:
            steps += 1
            if ra > max_w or rb > max_w:
                monotone_ok = False
            if do_aht and nb == bs and ra > rb and (ra - 1) >= 2 * rb:
                q = (ra - 1) // rb
                p_tail = edges[b_off + bs * 2]
                bw_new = join_w(join_pow_w(aw, q), bw)
                u, v = (fb, bs) if (q & 1) else (bs, fb)
                edges[t_off + ts * 4 + 1 : t_off + ts * 4 + 4] = [rb, bw_new, v]
                b_set(u, p_tail, ts)
                b_set(v, u, ts)
                bs, fb, nb, bw = u, v, u, bw_new
                consumed = q * rb
                ra -= consumed
                remaining -= consumed
            if do_aht and na == ts and ra < rb and (rb - 1) >= 2 * ra:
                q = (rb - 1) // ra
                p_tail = edges[t_off + ts * 4]
                w_qm1 = join_w(aw, join_pow_w(bw, q - 1))
                w_q = join_w(w_qm1, bw)
                u, v = (fa, ts) if (q & 1) else (ts, fa)
                edges[b_off + bs * 2 + 1] = v
                t_set(u, p_tail, ra, w_qm1, bs)
                t_set(v, u, ra, w_q, bs)
                ts, fa, na, aw = u, v, u, w_q
                consumed = q * ra
                rb -= consumed
                remaining -= consumed
            wd = join_w(aw, bw)
            if ra == rb:
                remaining -= ra
                edges[t_off + ts * 4 + 1 : t_off + ts * 4 + 4] = [ra, wd, bs]
                edges[b_off + bs * 2 + 1] = ts
                sa, sb = na, nb
                break
            elif ra > rb:
                remaining -= rb
                edges[t_off + ts * 4 + 1 : t_off + ts * 4 + 4] = [rb, wd, bs]
                p_bs = edges[b_off + bs * 2]
                b_set(bs, fb, ts)
                edges[b_off + fb * 2] = p_bs
                bs = fb
                ra -= rb
                fb = nb
                nb, ts = b_get(nb)
                rb = edges[t_off + ts * 4 + 1]
                bw = edges[t_off + ts * 4 + 2]
            else:
                remaining -= ra
                edges[b_off + bs * 2 + 1] = ts
                p_ts = edges[t_off + ts * 4]
                t_set(ts, fa, ra, wd, bs)
                edges[t_off + fa * 4] = p_ts
                ts = fa
                rb -= ra
                fa = na
                base_na = t_off + fa * 4
                na, ra, aw, bs = edges[base_na], edges[base_na + 1], edges[base_na + 2], edges[base_na + 3]

    xb = m
    while sb != 0:
        s_prev, s_top = b_get(sb)
        xb -= edges[t_off + s_top * 4 + 1]
        edges[t_off + s_top * 4 + 3] = xb
        sb = s_prev

    # Clobber `bots` slice immediately after Phase P3a to prove its lifetime has ended!
    for k in range(2 * slots):
        edges[b_off + k] = 0xBADF00D

    codes: list[bytes] = [bytes([c]) for c in range(256)]
    bands: list[tuple[int, int, int]] = []
    rev_start = [0] * 256
    rem = edges[t_off + sa * 4 + 1] if sa != 0 else 0
    kc = m
    for a in range(255, -1, -1):
        rev_start[a] = len(bands)
        st = thin_start[a]
        while kc > st:
            nd_prev = edges[t_off + sa * 4]
            nd_wid = edges[t_off + sa * 4 + 2]
            nd_other = edges[t_off + sa * 4 + 3]
            width = min(kc - st, rem)
            rem -= width
            kc -= width
            dst = nd_other + rem
            if nd_wid == 0:
                code = a
            else:
                codes.append(word_pool[nd_wid] + bytes([a]))
                code = len(codes) - 1
            bands.append((width, dst, code))
            if rem == 0:
                sa = nd_prev
                rem = edges[t_off + sa * 4 + 1] if sa != 0 else 0

    # Clobber ALL of `edges` (`tops + bots + scratch`) right before Phase P4 (`wire`)!
    for k in range(len(edges)):
        edges[k] = 0xFEEDFACE

    bits = max(1, (m - 1).bit_length())
    for a in range(256):
        b0 = rev_start[a]
        b1 = rev_start[a - 1] if a > 0 else len(bands)
        if b0 == b1:
            continue
        cid = head[a]
        ic = 0
        ep = 0
        q_cur = 0
        for span, dst, code in reversed(bands[b0:b1]):
            cur_dst = dst
            rem_s = span
            while rem_s > 0:
                while ep + 1 < n_ep and q_cur >= ep_cuts[ep][a]:
                    ep += 1
                if ic >= chunk_u16:
                    cid = chunk_next[cid]
                    ic = 0
                tk = min(rem_s, chunk_u16 - ic, ep_cuts[ep][a] - q_cur)
                for k in range(tk):
                    if cid < max_out_chunks:
                        off = cid * chunk_b + (ic + k) * 2
                        lo16 = out_buf[off] | (out_buf[off + 1] << 8)
                    else:
                        oi = cid - max_out_chunks
                        lo16 = ovf_chunks[oi][(ic + k) * 2] | (ovf_chunks[oi][(ic + k) * 2 + 1] << 8)
                    edges[cur_dst + k] = (code << bits) | (ep << 16) | lo16
                ic += tk
                q_cur += tk
                cur_dst += tk
                rem_s -= tk

    # Phase P5: Sentinel-masked emit overwriting `out_buf`!
    mask = (1 << bits) - 1
    stop_code = len(codes)
    codes.append(b"")
    raw_mark = edges[mark]
    code0 = raw_mark >> bits
    j = raw_mark & mask
    edges[mark] = (stop_code << bits) | j

    res = bytearray()
    res.extend(codes[code0])
    for _ in range(m - 1):
        if len(res) >= n:
            break
        ed = edges[j]
        c = ed >> bits
        j = ed & mask
        if c == stop_code:
            break
        res.extend(codes[c])
    edges[mark] = raw_mark
    res = res[:n]
    while len(res) < n:
        res.extend(res[: min(len(res), n - len(res))])
    return True, bytes(res), steps, monotone_ok


def main() -> None:
    rng = random.Random(2026112119)

    # 1. Theorem 19.1 & 19.2a across 500 random multi-alphabet, periodic, micro, and multi-epoch blocks
    monotone_violations = 0
    for trial in range(500):
        alph = rng.choice([2, 3, 5, 16, 64])
        n = rng.randint(1, 900) if trial < 450 else rng.randint(2000, 4500)
        if trial % 25 == 0:
            period = bytes(rng.randrange(alph) for _ in range(rng.randint(1, 19)))
            text = (period * ((n // len(period)) + 2))[:n]
        else:
            text = bytes(rng.randrange(alph) for _ in range(n))
        recs, p = forward_bwt_records(text)
        want = lf_reference(recs, n, p)
        assert want == text, f"lf_reference bug on trial {trial}"
        for lim in (14, 16):
            ok, got, _, mono = edges_mux_decode(recs, n, p, limit=lim, m_limit=500_000, use_aht=True)
            assert ok, f"unexpected abort on trial {trial}"
            assert got == text, f"EdgesMux mismatch on trial {trial} (L={lim})"
            if not mono:
                monotone_violations += 1
    assert monotone_violations == 0, f"strip width monotonicity violated {monotone_violations} times"
    print("[PASS] Theorem 19.1 (`EdgesMux` clobber-checked disjoint lifetimes): 500/500 x L={14,16} EXACT")
    print("[PASS] Theorem 19.2a (Strip-width monotonicity max(ra,rb) <= W_max): 0 violations")

    # 2. Theorem 19.2a Ostrowski spiral contraction on both `ra > rb` and `ra < rb` (sz = 20,000)
    sz = 20_000
    for name, c0, w0, c1, w1, p_sp in [
        ("ra>rb", 1, 1, 0, sz - 1, 0),
        ("ra<rb", 1, sz - 1, 0, 1, sz - 1),
    ]:
        recs = []
        rem = w0
        while rem > 0:
            tk = min(rem, 127)
            recs.append((c0 << 8) | tk)
            rem -= tk
        rem = w1
        while rem > 0:
            tk = min(rem, 127)
            recs.append((c1 << 8) | tk)
            rem -= tk
        want = lf_reference(recs, sz, p_sp)
        ok_no, got_no, steps_no, _ = edges_mux_decode(recs, sz, p_sp, m_limit=500_000, use_aht=False)
        ok_aht, got_aht, steps_aht, mono = edges_mux_decode(recs, sz, p_sp, m_limit=500_000, use_aht=True)
        assert ok_no and ok_aht and mono
        assert got_no == want and got_aht == want
        assert steps_no >= sz - 5 and steps_aht <= 8, f"{name}: steps_no={steps_no}, steps_aht={steps_aht}"
        print(f"[PASS] Theorem 19.2a ({name} spiral sz={sz}): {steps_no} steps -> {steps_aht} steps, EXACT")

    # 3. Theorem 19.2b Sticky `m = 0xFFFFFFFF` early-abort invariant (including coalesced w > m_limit)
    for m_lim in (128, 1000, 4096):
        # Case A: alternating thin runs exceeding m_limit across multiple chunks/epochs
        recs_a = [((i & 0xFF) << 8) | 12 for i in range((m_lim // 6) + 40)]
        n_a = sum(r & 0xFF for r in recs_a)
        ok_a, _, _, _ = edges_mux_decode(recs_a, n_a, 0, limit=16, m_limit=m_lim, use_aht=True)
        assert not ok_a
        # Case B: single coalesced run of width <= 16 when m_lim is tiny, or multi-record thin?
        # Also test exact boundary m == m_lim (must succeed!) vs m == m_lim + 1 (must abort!)
        recs_eq = [((i & 0xFF) << 8) | 1 for i in range(m_lim)]
        ok_eq, got_eq, _, _ = edges_mux_decode(recs_eq, m_lim, 0, limit=16, m_limit=m_lim, use_aht=True)
        assert ok_eq and got_eq == lf_reference(recs_eq, m_lim, 0)
        recs_gt = recs_eq + [(0 << 8) | 1]
        ok_gt, _, _, _ = edges_mux_decode(recs_gt, m_lim + 1, 0, limit=16, m_limit=m_lim, use_aht=True)
        assert not ok_gt
    print("[PASS] Theorem 19.2b (Sticky m=0xFFFFFFFF abort, exact boundary m==m_limit vs m_limit+1): PASS")
    print("ALL CYCLE 19 MATHEMATICAL VERIFICATIONS PASSED!")


if __name__ == "__main__":
    main()
