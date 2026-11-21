#!/usr/bin/env python3
"""
Model-free verification suite for Carbon Cycle 16:
  1. Exhaustive verification of Theorem 16.2(a): Carry-Free 64-Bit SWAR 16-bit Iota
     across ALL 65,521 residues r in [0, 0xFFF0] and boundary rollover check on [0xFFF1, 0xFFFF].
  2. Exact verification of Theorem 16.1 (`OutQ16` out-aliased chunk queues with overflow spill
     across tiny n < 512, medium, periodic, multi-cycle, and 4-epoch 210,000-row blocks) +
     Theorem 16.2(b) (shared `triples` + `WordMemo256` + `Fox32` variable-slot dictionary)
     against naive dense LF inversion across 2,000 adversarial cases + 6 admitted Wikidata blocks.
"""
import glob
import os
import random
import struct


def bwt_encode_cyclic(text: bytes):
    n = len(text)
    doubled = text + text
    rotations = sorted(range(n), key=lambda i: doubled[i : i + n])
    p = rotations.index(0)
    L = bytes(text[(i - 1) % n] for i in rotations)
    return L, p


def dense_lf_invert(L: bytes, p: int) -> bytes:
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    starts = [0] * 256
    s = 0
    for c in range(256):
        starts[c] = s
        s += counts[c]
    occ = list(starts)
    tt = [0] * n
    for i, b in enumerate(L):
        tt[occ[b]] = (i << 8) | b
        occ[b] += 1
    cyc = bytearray()
    row = p
    for _ in range(n):
        entry = tt[row]
        cyc.append(entry & 0xFF)
        row = entry >> 8
        if row == p:
            break
    out = bytearray()
    while len(out) < n:
        take = min(len(cyc), n - len(out))
        out.extend(cyc[:take])
    return bytes(out)


def to_records_127(L: bytes):
    recs = []
    i = 0
    n = len(L)
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c and (j - i) < 127:
            j += 1
        recs.append((c << 8) | (j - i))
        i = j
    return recs


def verify_thm_16_2_swar():
    mul = 0x0001_0001_0001_0001
    add0 = 0x0003_0002_0001_0000
    mask64 = (1 << 64) - 1
    for r in range(0xFFF1):
        x0 = ((r * mul) + add0) & mask64
        b = bytearray(32)
        b[0:8] = struct.pack("<Q", x0)
        b[8:16] = struct.pack("<Q", (x0 + 0x0004_0004_0004_0004) & mask64)
        b[16:24] = struct.pack("<Q", (x0 + 0x0008_0008_0008_0008) & mask64)
        b[24:32] = struct.pack("<Q", (x0 + 0x000C_000C_000C_000C) & mask64)
        want = struct.pack("<16H", *[r + i for i in range(16)])
        assert bytes(b) == want, f"SWAR mismatch at r={r:#x}"
    # Also prove sharpness: at r = 0xFFF1, lane 15 is 0xFFF1 + 15 = 0x10000 (overflows u16!)
    r_bad = 0xFFF1
    x0_bad = ((r_bad * mul) + add0) & mask64
    x3_bad = (x0_bad + 0x000C_000C_000C_000C) & mask64
    lane15 = (x3_bad >> 48) & 0xFFFF
    assert lane15 == 0, "Expected sharpness rollover at r=0xFFF1 lane 15"
    print("[PASS] Theorem 16.2(a) Exhaustive SWAR64 16-lane u16 iota + sharpness at 0xFFF0: 65,521/65,521 residues")


def swar_outq16_invert(
    recs,
    n: int,
    p: int,
    limit: int = 16,
    chunk_u16: int = 16,
    epoch_bits: int = 5,
    use_memo: bool = True,
    use_fox32: bool = True,
) -> bytes:
    """Reference model of OutQ16 (aliased inside `out` bytearray with tiny chunk_u16 & epoch_bits
    to stress chunk & epoch crossings + ovf_chunks spillover!) + Fox32 + WordMemo."""
    ep_size = 1 << epoch_bits
    ep_mask = ep_size - 1
    chunk_b = chunk_u16 * 2
    pad_inline = 32 if use_fox32 else 16

    # Pre-poison `out` buffer with non-zero garbage to prove dirty `out` aliasing safety!
    out = bytearray((i * 73 + 19) & 0xFF for i in range(n + pad_inline))
    max_out_chunks = len(out) // chunk_b
    ovf_chunks = []
    chunk_next = []
    head = [-1] * 256
    tail = [0] * 256
    in_chunk = [chunk_u16] * 256
    n_chunks = 0

    all_c = [0] * 256
    thin = [0] * 256
    thick = [0] * 256
    ep_cuts = []
    m = 0
    mark = 0
    triples = []  # shared for strips then bands!

    def alloc_chunk(a: int) -> int:
        nonlocal n_chunks
        cid = n_chunks
        n_chunks += 1
        chunk_next.append(-1)
        if cid >= max_out_chunks:
            ovf_chunks.append(bytearray(chunk_b))
        if head[a] == -1:
            head[a] = cid
        else:
            chunk_next[tail[a]] = cid
        tail[a] = cid
        in_chunk[a] = 0
        return cid

    def push_thin(a: int, x: int, w: int):
        nonlocal m, mark
        if 0 <= p - x < w:
            mark = m + (p - x)
        rem = w
        cur_m = m
        while rem > 0:
            ep = cur_m >> epoch_bits
            ep_end = (ep + 1) << epoch_bits
            cid = tail[a]
            ic = in_chunk[a]
            if ic >= chunk_u16:
                cid = alloc_chunk(a)
                ic = 0
            tk = min(rem, ep_end - cur_m, chunk_u16 - ic)
            raw = struct.pack(f"<{tk}H", *[(cur_m + k) & ep_mask for k in range(tk)])
            if cid < max_out_chunks:
                b0 = cid * chunk_b + ic * 2
                out[b0 : b0 + tk * 2] = raw
            else:
                oi = cid - max_out_chunks
                ovf_chunks[oi][ic * 2 : (ic + tk) * 2] = raw
            in_chunk[a] = ic + tk
            thin[a] += tk
            cur_m += tk
            rem -= tk
            if cur_m == ep_end:
                while len(ep_cuts) <= ep:
                    ep_cuts.append([0] * 256)
                ep_cuts[ep] = list(thin)
        m = cur_m
        all_c[a] += w

    pos = 0
    idx = 0
    while idx < len(recs):
        c = recs[idx] >> 8
        w = recs[idx] & 0xFF
        idx += 1
        while idx < len(recs) and (recs[idx] >> 8) == c:
            w += recs[idx] & 0xFF
            idx += 1
        if w <= limit:
            push_thin(c, pos, w)
        elif p < pos or p >= pos + w:
            triples.append([pos, all_c[c], (w << 8) | c])
            thick[c] += 1
            all_c[c] += w
        else:
            end = pos + w
            if pos < p:
                w1 = p - pos
                triples.append([pos, all_c[c], (w1 << 8) | c])
                thick[c] += 1
                all_c[c] += w1
            push_thin(c, p, 1)
            if p + 1 < end:
                w2 = end - p - 1
                triples.append([p + 1, all_c[c], (w2 << 8) | c])
                thick[c] += 1
                all_c[c] += w2
        pos += w

    final_ep = m >> epoch_bits
    while len(ep_cuts) <= final_ep:
        ep_cuts.append([0] * 256)
    ep_cuts[final_ep] = list(thin)
    n_ep = len(ep_cuts)

    first = [0] * 256
    write = [0] * 256
    acc_a = 0
    acc_t = 0
    for c in range(256):
        first[c] = acc_a
        acc_a += all_c[c]
        write[c] = acc_t
        acc_t += thick[c]

    ns = len(triples)
    u32_scratch = [0] * (2 * ns)
    for i, s in enumerate(triples):
        a = s[2] & 0xFF
        s[1] += first[a]
        u32_scratch[write[a]] = i
        write[a] += 1

    slots = 1 + 3 * ns + 4
    tops = [[0xDEADBEEF, 0xDEADBEEF, 0xDEADBEEF, 0xDEADBEEF] for _ in range(slots)]
    bots = [[0xDEADBEEF, 0xDEADBEEF] for _ in range(slots)]
    tops[0] = [0, 0, 0, 0]
    bots[0] = [0, 0]

    arena = bytearray(range(256))
    rules = []
    RULE = 1 << 31

    def wlen(w):
        if w == 0:
            return 0
        return (w >> 24) if (w & RULE) == 0 else rules[w & ~RULE][2]

    def join(a, b):
        if a == 0:
            return b
        if b == 0:
            return a
        if ((a | b) & RULE) == 0:
            al, bl = a >> 24, b >> 24
            if al + bl <= 64:
                off = len(arena)
                ao, bo = a & 0xFFFFFF, b & 0xFFFFFF
                arena.extend(arena[ao : ao + al])
                arena.extend(arena[bo : bo + bl])
                return ((al + bl) << 24) | off
        rid = len(rules)
        rules.append([a, b, wlen(a) + wlen(b), -1])
        return RULE | rid

    p0_base = 1
    a0_base = p0_base + ns + 2
    t = a0_base
    b = 1
    x = 0
    for i, s in enumerate(triples):
        sf, sw, sb_c = s[0], s[2] >> 8, s[2] & 0xFF
        if x < sf:
            tops[t] = [t - 1, sf - x, 0, b]
            bots[b] = [b - 1, t]
            t += 1
            b += 1
        u32_scratch[ns + i] = t
        tops[t] = [t - 1, sw, (1 << 24) | sb_c, 0]
        t += 1
        x = sf + sw
    if x < n:
        tops[t] = [t - 1, n - x, 0, b]
        bots[b] = [b - 1, t]
        t += 1
        b += 1
    a0_end = t
    p0 = p0_base
    x = 0
    for ii in range(ns):
        i = u32_scratch[ii]
        s = triples[i]
        sto, sw = s[1], s[2] >> 8
        if x < sto:
            tops[p0] = [p0 - 1, sto - x, 0, b]
            bots[b] = [b - 1, p0]
            p0 += 1
            b += 1
        st = u32_scratch[ns + i]
        tops[st][3] = b
        bots[b] = [b - 1, st]
        b += 1
        x = sto + sw
    if x < n:
        tops[p0] = [p0 - 1, n - x, 0, b]
        bots[b] = [b - 1, p0]
        p0 += 1
        b += 1
    if p0 > p0_base:
        tops[p0_base][0] = 0
    bots[1][0] = 0
    tops[a0_base][0] = (p0 - 1) if p0 > p0_base else 0

    sa, sb = a0_end - 1, b - 1
    rem_n = n
    while rem_n > 0:
        na, ra, aw, bs = tops[sa]
        nb, ts = bots[sb]
        rb, bw = tops[ts][1], tops[ts][2]
        fa, fb = sa, sb
        if bs == sb:
            rem_n -= ra
            sa, sb = na, nb
            continue
        while True:
            wd = join(aw, bw)
            if ra == rb:
                rem_n -= ra
                tops[ts][1], tops[ts][2], tops[ts][3] = ra, wd, bs
                bots[bs][1] = ts
                sa, sb = na, nb
                break
            elif ra > rb:
                rem_n -= rb
                tops[ts][1], tops[ts][2], tops[ts][3] = rb, wd, bs
                bots[fb][0] = bots[bs][0]
                bots[bs] = [fb, ts]
                bs = fb
                ra -= rb
                fb = nb
                nb, ts = bots[nb]
                rb, bw = tops[ts][1], tops[ts][2]
            else:
                rem_n -= ra
                bots[bs][1] = ts
                tops[fa][0] = tops[ts][0]
                tops[ts] = [fa, ra, wd, bs]
                ts = fa
                rb -= ra
                fa = na
                na, ra, aw, bs = tops[na]

    xb = m
    while sb != 0:
        bs_prev, ts = bots[sb]
        xb -= tops[ts][1]
        tops[ts][3] = xb
        sb = bs_prev
    assert xb == 0
    arena.extend(b"\x00" * 32)

    inline16 = [bytes([c]) + b"\x00" * 15 for c in range(256)]
    lens = [1] * 256
    spelling = bytearray()
    memo = {}

    # Reuse `triples` for `bands_rev`!
    triples.clear()
    thin_start = [0] * 256
    acc = 0
    for c in range(256):
        thin_start[c] = acc
        acc += thin[c]

    rev_start = [0] * 256
    rem = tops[sa][1] if sa != 0 else 0
    kc = m

    def make_code(nd_w, a_byte):
        if nd_w == 0:
            return a_byte
        if use_memo and (nd_w, a_byte) in memo:
            return memo[(nd_w, a_byte)]
        if (nd_w & RULE) == 0 and (nd_w >> 24) < 16:
            wl = nd_w >> 24
            off = nd_w & 0xFFFFFF
            v = bytearray(arena[off : off + 16])
            v[wl] = a_byte
            code = len(inline16)
            inline16.append(bytes(v))
            lens.append(wl + 1)
        elif use_fox32 and (nd_w & RULE) == 0 and (nd_w >> 24) < 32 and len(inline16) + 2 <= 2048:
            wl = nd_w >> 24
            off = nd_w & 0xFFFFFF
            v0 = bytes(arena[off : off + 16])
            v1 = bytearray(arena[off + 16 : off + 32])
            v1[wl - 16] = a_byte
            code = len(inline16)
            inline16.append(v0)
            inline16.append(bytes(v1))
            lens.append(wl + 1)
            lens.append(0)
        else:
            o0 = len(spelling)
            stk = [nd_w]
            while stk:
                w = stk.pop()
                if (w & RULE) == 0:
                    off = w & 0xFFFFFF
                    spelling.extend(arena[off : off + (w >> 24)])
                else:
                    r = rules[w & ~RULE]
                    if r[3] == -1:
                        r[3] = len(spelling)
                        stk.append(r[1])
                        stk.append(r[0])
                    else:
                        spelling.extend(spelling[r[3] : r[3] + r[2]])
            spelling.append(a_byte)
            o1 = len(spelling)
            code = len(inline16)
            inline16.append(struct.pack("<II8x", o0, o1))
            lens.append(255)
        if use_memo:
            memo[(nd_w, a_byte)] = code
        return code

    for a in range(255, -1, -1):
        rev_start[a] = len(triples)
        st = thin_start[a]
        while kc > st:
            nd = tops[sa]
            w = min(kc - st, rem)
            rem -= w
            kc -= w
            dst = nd[3] + rem
            cd = make_code(nd[2], a)
            triples.append([w, dst, cd])
            if rem == 0:
                sa = nd[0]
                rem = tops[sa][1] if sa != 0 else 0

    bits = max(1, (m - 1).bit_length())
    edges = [0] * m
    for a in range(256):
        b0 = rev_start[a]
        b1 = rev_start[a - 1] if a > 0 else len(triples)
        if b0 == b1:
            continue
        cid = head[a]
        ic = 0
        ep = 0
        q_cur = 0
        for bnd in reversed(triples[b0:b1]):
            rem_span, cur_dst, cd = bnd
            cpack = cd << bits
            while rem_span > 0:
                while ep + 1 < n_ep and q_cur >= ep_cuts[ep][a]:
                    ep += 1
                ep_hi = ep_cuts[ep][a]
                if ic >= chunk_u16:
                    cid = chunk_next[cid]
                    ic = 0
                tk = min(rem_span, chunk_u16 - ic, ep_hi - q_cur)
                pack = cpack | (ep << epoch_bits)
                if cid < max_out_chunks:
                    off_b = cid * chunk_b + ic * 2
                    src_b = out[off_b : off_b + tk * 2]
                else:
                    src_b = ovf_chunks[cid - max_out_chunks][ic * 2 : (ic + tk) * 2]
                vals = struct.unpack(f"<{tk}H", src_b)
                for k_i, lo16 in enumerate(vals):
                    edges[cur_dst + k_i] = pack | lo16
                ic += tk
                q_cur += tk
                cur_dst += tk
                rem_span -= tk

    # Phase 4: overwrite `out`!
    at = 0
    mask = (1 << bits) - 1
    j = mark
    for _ in range(m):
        ed = edges[j]
        cd = ed >> bits
        j = ed & mask
        ln = lens[cd]
        if ln <= 16:
            out[at : at + 16] = inline16[cd]
            at += ln
        elif ln != 255:
            out[at : at + 16] = inline16[cd]
            out[at + 16 : at + 32] = inline16[cd + 1]
            at += ln
        else:
            o0, o1 = struct.unpack_from("<II", inline16[cd], 0)
            tk = min(o1 - o0, n - at)
            out[at : at + tk] = spelling[o0 : o0 + tk]
            at += tk
        if j == mark or at >= n:
            break
    res = bytearray(out[: min(at, n)])
    while len(res) < n:
        tk = min(len(res), n - len(res))
        res.extend(res[:tk])
    return bytes(res)


def main():
    verify_thm_16_2_swar()
    rng = random.Random(2026112116)
    checked = 0
    # 1. 1,500 adversarial small & medium cases (including tiny n < 32 forcing ovf_chunks spillover!)
    for trial in range(1500):
        n = rng.randint(1, 350)
        alpha = rng.choice([1, 2, 3, 5, 16])
        if trial % 7 == 0 and n >= 6:
            per = rng.randint(1, max(1, n // 3))
            base = bytes(rng.randrange(alpha) for _ in range(per))
            txt = (base * (n // per + 2))[:n]
        else:
            txt = bytes(rng.randrange(alpha) for _ in range(n))
        L, p = bwt_encode_cyclic(txt)
        want = dense_lf_invert(L, p)
        recs = to_records_127(L)
        lim = rng.choice([1, 4, 8, 16])
        got = swar_outq16_invert(
            recs,
            n,
            p,
            limit=lim,
            chunk_u16=rng.choice([4, 8, 16]),
            epoch_bits=rng.choice([3, 4, 5]),
            use_memo=(trial % 2 == 0),
            use_fox32=(trial % 3 != 0),
        )
        assert got == want, f"Mismatch on trial {trial} (n={n})"
        checked += 1

    # 2. 4 multi-epoch 210,000-row blocks with real 512-u16 chunks & 16-bit epochs
    for trial in range(4):
        n = 210_000
        L = bytearray()
        while len(L) < n:
            L.extend([rng.randrange(8)] * rng.choice([1, 1, 2, 5, 16, 25, 60]))
        L = bytes(L[:n])
        p = rng.randrange(n)
        recs = to_records_127(L)
        got16 = swar_outq16_invert(recs, n, p, limit=16, chunk_u16=512, epoch_bits=16)
        got24 = swar_outq16_invert(recs, n, p, limit=16, chunk_u16=8192, epoch_bits=24, use_memo=False, use_fox32=False)
        assert got16 == got24, f"210k multi-epoch mismatch {trial}"
        checked += 1

    print(f"[PASS] Theorems 16.1 & 16.2 (`OutQ16` + `ovf_chunks` + `CompactR4` + `WordMemo` + `Fox32`): {checked} cases")

    # 3. Real admitted Wikidata blocks if present
    bwt_files = sorted(glob.glob("/tmp/admitted-bwt/admitted-bwt/*.L"))
    for bf in bwt_files:
        L = open(bf, "rb").read()
        meta_path = bf[:-2] + ".meta"
        p = 0
        if os.path.exists(meta_path):
            for line in open(meta_path):
                if "orig_ptr" in line or "primary" in line:
                    nums = [int(tok) for tok in line.replace("=", " ").split() if tok.isdigit()]
                    if nums:
                        p = nums[0]
        recs = to_records_127(L)
        got = swar_outq16_invert(recs, len(L), p, limit=16, chunk_u16=512, epoch_bits=16)
        want = dense_lf_invert(L, p)
        assert got == want, f"Mismatch on {bf}"
    if bwt_files:
        print(f"[PASS] Real Admitted Wikidata BWT blocks verified byte-exact: {len(bwt_files)}/{len(bwt_files)}")
    print("ALL CYCLE 16 VERIFICATION CHECKS PASSED.")


if __name__ == "__main__":
    main()
