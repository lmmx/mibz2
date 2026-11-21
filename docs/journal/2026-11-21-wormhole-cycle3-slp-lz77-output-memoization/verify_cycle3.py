#!/usr/bin/env python3
"""Cycle 3 Verifier & Census: AHT/Zorich Euclidean Pairing + Rytter-Gasieniec Output-Memoized SLP (OM-Fox).

Verifies:
  1. Theorem 3.1 (Zorich / Agol-Hass-Thurston Multiplicative Periodic Pairing):
     - Exact O(log q) binary-doubling SLP construction when a strip/gap pair repeats adjacently
       (prev0[0]=x, prev1[0]=y with next0[y]=x or next1[x]=y), verified across 26,135 small
       (L, p, Lambda) cases + 30 6-block decodes.
  2. Theorem 3.2 (Compact Inline-16 Split + Zero-Spelling Output-Memoized SLP Decoding):
     - Short words (2 <= len <= 16) expand directly into compact inline16 (0 slots wasted on
       >16B words); long words (len > 16) store (slp_root, tail_byte) and expand during emit
       via Rytter (2003) / Gasieniec et al. (2005) first-occurrence memoization directly inside
       out[0..at), eliminating 100% of self.spelling (0 B across all Lambda in {1, 2, 4, 8, 16}!).
     - Verified byte-exact on all 26,135 small cases (primitive, periodic, multi-cycle, and
       arbitrary non-BWT columns) and all 6 authenticated Wikidata blocks at Lambda in {16,8,4,2,1}.
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


def om_fox_decode(L: bytes, p: int, limit: int, aht_accel: bool = True):
    """Exact OM-Fox decoder: AHT/Zorich pairing + Compact Inline16 + Zero-Spelling Output-Memoized SLP."""
    n = len(L)
    all_c = [0] * 256
    thin_c = [0] * 256
    thick_c = [0] * 256
    strips = []
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

    # SLP rules: [left, right, len, first_out_pos]
    # Note: Endpoint::eliminate joins (a.word, b.word) in LF order, and Endpoint::append reverses
    # only if needed? Wait: in src/open/endpoint.rs, join(a.word, b.word) ALREADY builds the
    # reversed (text-order!) tree!! Let's check endpoint.rs:133 & 164-165:
    # join(a.word, b.word) has left=a.word (downstream in LF = EARLIER in text!) and right=b.word!
    # Yes!! In endpoint.rs, side 0 is codomain (downstream LF = earlier in text), side 1 is domain
    # (upstream LF = later in text), so join(a.word, b.word) is ALREADY IN FORWARD TEXT ORDER!
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

    def pow_word(base_w: int, q: int) -> int:
        """Binary exponentiation of base_w^q in O(log q) balanced SLP rules!"""
        if base_w == 0 or q == 0:
            return 0
        res = 0
        cur = base_w
        e = q
        while e > 0:
            if e & 1:
                res = join(res, cur)
            e >>= 1
            if e > 0:
                cur = join(cur, cur)
        return res

    rem = n
    steps = 0
    aht_quotients = 0
    while rem > 0:
        nx = prev0[0]
        ny = prev1[0]
        aw = n_width[nx]
        bw = n_width[ny]
        if nx == ny:
            rem -= aw
            steps += 1
            p0, q0 = prev0[nx], next0[nx]
            next0[p0], prev0[q0] = q0, p0
            p1, q1 = prev1[nx], next1[nx]
            next1[p1], prev1[q1] = q1, p1
            continue

        if aw <= bw:
            # Check AHT/Zorich periodic condition: if prev0[nx] == ny, cutting nx and inserting
            # after ny leaves nx at prev0[0] and ny at prev1[0] for q = bw // aw iterations!
            if aht_accel and prev0[nx] == ny and bw >= 2 * aw:
                q = bw // aw
                aht_quotients += 1
                steps += 1
                rem -= q * aw
                # Over q iterations: n_word[nx] <- (...((w_x + w_y) + w_y)...) = w_x + (w_y)^q
                n_word[nx] = join(n_word[nx], pow_word(n_word[ny], q))
                if q * aw == bw:
                    p0, q0 = prev0[ny], next0[ny]
                    next0[p0], prev0[q0] = q0, p0
                    p1, q1 = prev1[ny], next1[ny]
                    next1[p1], prev1[q1] = q1, p1
                else:
                    n_width[ny] = bw - q * aw
                continue

            rem -= aw
            steps += 1
            wj = join(n_word[nx], n_word[ny])
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
                n_width[ny] -= aw
        else:
            # bw < aw: symmetric AHT/Zorich periodic check: if prev1[ny] == nx, ny stays at prev1[0]!
            if aht_accel and prev1[ny] == nx and aw >= 2 * bw:
                q = aw // bw
                aht_quotients += 1
                steps += 1
                rem -= q * bw
                # Over q iterations: n_word[ny] <- (w_x)^q + w_y
                n_word[ny] = join(pow_word(n_word[nx], q), n_word[ny])
                n_width[nx] = aw - q * bw
                if n_width[nx] == 0:
                    p0, q0 = prev0[nx], next0[nx]
                    next0[p0], prev0[q0] = q0, p0
                    p1, q1 = prev1[nx], next1[nx]
                    next1[p1], prev1[q1] = q1, p1
                continue

            rem -= bw
            steps += 1
            wj = join(n_word[nx], n_word[ny])
            p1, q1 = prev1[ny], next1[ny]
            next1[p1], prev1[q1] = q1, p1
            qn = next1[nx]
            prev1[ny], next1[ny] = nx, qn
            next1[nx], prev1[qn] = ny, ny
            n_word[ny] = wj
            n_width[nx] -= bw

    bottom = [0] * num_nodes
    nid = next1[0]
    x = 0
    while nid != 0:
        bottom[nid] = x
        x += n_width[nid]
        nid = next1[nid]
    assert x == m

    # Split dictionary:
    # - Codes 0..255: 1-byte literals in inline16[0..256]
    # - Short multi-byte words (2 <= step_len <= 16): expanded directly into inline16[256 + s_idx]!
    #   ZERO self.spelling buffer!
    # - Long words (step_len > 16): stored in long_handles = [(wd, tail_byte, step_len)]!
    #   ZERO inline16 slot wasted! ZERO self.spelling buffer!
    inline16 = [bytes([c]) + b"\x00" * 15 for c in range(256)]
    inline_lens = [1] * 256
    long_handles = []  # (wd, tail_byte, step_len)

    def expand_short_16(wd: int, tail_a: int) -> tuple[bytes, int]:
        buf = bytearray()
        stk = [wd]
        while stk:
            cur = stk.pop()
            if cur <= 256:
                buf.append(cur - 1)
            else:
                rl = rules[cur - 257]
                stk.append(rl[1])
                stk.append(rl[0])
        buf.append(tail_a)
        ln = len(buf)
        assert 2 <= ln <= 16
        return bytes(buf) + b"\x00" * (16 - ln), ln

    # First pass over bands to build compact short_words & long_handles
    band_list = []  # (dst, span, is_long, idx_or_code, q_a_slice)
    nid = next0[0]
    in_node = 0
    k = 0
    for a in range(256):
        end = k + thin_c[a]
        q_idx = 0
        q_a = queues[a]
        while k < end:
            nw = n_width[nid]
            span = min(end - k, nw - in_node)
            dst = bottom[nid] + in_node
            wd = n_word[nid]
            if wd == 0:
                band_list.append((dst, span, False, a, q_a[q_idx : q_idx + span]))
            else:
                step_len = wlen(wd) + 1
                if step_len <= 16:
                    slot_bytes, ln = expand_short_16(wd, a)
                    scode = len(inline16)
                    inline16.append(slot_bytes)
                    inline_lens.append(ln)
                    band_list.append((dst, span, False, scode, q_a[q_idx : q_idx + span]))
                else:
                    lidx = len(long_handles)
                    long_handles.append((wd, a, step_len))
                    band_list.append((dst, span, True, lidx, q_a[q_idx : q_idx + span]))
            q_idx += span
            in_node += span
            k += span
            if in_node == nw:
                nid = next0[nid]
                in_node = 0

    short_end = len(inline16)  # codes [0 .. short_end) are <=16B inline; [short_end ..) are long SLP!
    next_j = [0] * m
    edge_code = [0] * m
    for dst, span, is_long, idx_val, src_slice in band_list:
        code = (short_end + idx_val) if is_long else idx_val
        for d, src_j in enumerate(src_slice):
            next_j[dst + d] = src_j
            edge_code[dst + d] = code

    # Zero-spelling Output-Memoized Emit (Rytter 2003 / Gasieniec et al. 2005)!
    out = bytearray(n + 16)
    at = 0
    j = mark
    stk = []
    out_lz_copies = 0
    out_lz_bytes = 0

    for _ in range(m):
        code = edge_code[j]
        j = next_j[j]
        if code < short_end:
            ln = inline_lens[code]
            out[at : at + 16] = inline16[code]
            at += ln
        else:
            wd, tail_a, step_len = long_handles[code - short_end]
            if at + step_len <= n:
                # Fast un-clamped exact Rytter-Gasieniec output-memoized expansion!
                stk.clear()
                stk.append((wd, False))
                while stk:
                    cur, post = stk.pop()
                    if cur <= 256:
                        out[at] = cur - 1
                        at += 1
                    else:
                        rl = rules[cur - 257]
                        if post:
                            # Both children just finished writing [at - rl[2] .. at)!
                            rl[3] = at - rl[2]
                        elif rl[3] != -1:
                            # Already materialized in out[rl[3] .. rl[3] + rl[2])!
                            src_pos = rl[3]
                            r_len = rl[2]
                            out[at : at + r_len] = out[src_pos : src_pos + r_len]
                            at += r_len
                            out_lz_copies += 1
                            out_lz_bytes += r_len
                        else:
                            stk.append((cur, True))
                            stk.append((rl[1], False))
                            stk.append((rl[0], False))
                out[at] = tail_a
                at += 1
            else:
                # Clamped cold tail for non-BWT truncated multi-cycle tests
                stk.clear()
                stk.append(wd)
                while stk and at < n:
                    cur = stk.pop()
                    if cur <= 256:
                        out[at] = cur - 1
                        at += 1
                    else:
                        rl = rules[cur - 257]
                        if rl[3] != -1:
                            take = min(rl[2], n - at)
                            out[at : at + take] = out[rl[3] : rl[3] + take]
                            at += take
                        else:
                            stk.append(rl[1])
                            stk.append(rl[0])
                if at < n:
                    out[at] = tail_a
                    at += 1
        if j == mark or at >= n:
            break

    valid_len = min(at, n)
    res = bytearray(out[:valid_len])
    while len(res) < n:
        take = min(len(res), n - len(res))
        res.extend(res[:take])

    stats = {
        "m": m,
        "H": H,
        "steps": steps,
        "aht_quotients": aht_quotients,
        "rules": len(rules),
        "short_codes": short_end,
        "long_codes": len(long_handles),
        "total_codes": short_end + len(long_handles),
        "inline16_kib": round((16 * short_end) / 1024.0, 1),
        "long_handles_kib": round((8 * len(long_handles)) / 1024.0, 1),
        "slp_rules_kib": round((12 * len(rules)) / 1024.0, 1),  # left(u32), right_len_packed, first_out(u32) = 12B!
        "spelling_kib": 0.0,
        "out_lz_copies": out_lz_copies,
        "out_lz_bytes": out_lz_bytes,
    }
    return bytes(res), stats


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
                        got, _ = om_fox_decode(L, p, lim, aht_accel=True)
                        assert got == ref, f"OM-Fox mismatch at L={L}, p={p}, lim={lim}"
                        cases += 1
    # Also test random non-BWT multi-cycle columns up to n=513 at all limits!
    seed = 12383217
    for n in (1, 31, 127, 128, 129, 385):
        for alpha in (1, 2, 11, 256):
            col = bytearray(n)
            for i in range(n):
                seed = (seed * 1664525 + 1013904223) & 0xFFFFFFFF
                col[i] = ((seed >> 8) % alpha)
            L = bytes(col)
            for p in (0, n // 2, n - 1):
                ref = plain_invert(L, p)
                for lim in (0, 1, 2, 4, 16):
                    got, _ = om_fox_decode(L, p, lim, aht_accel=True)
                    assert got == ref, f"Random multi-cycle mismatch n={n}, alpha={alpha}, p={p}, lim={lim}"
                    cases += 1
    return cases


def main():
    small_cases = run_small_exhaustive()
    print(f"[G1 EXHAUSTIVE + MULTI-CYCLE] PASS: {small_cases} (L, p, Lambda) instances verified byte-exact (0 B spelling!)")

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

        lims_out = {}
        for lim in (16, 8, 4, 2, 1):
            _, st_unaccel = om_fox_decode(L, p, lim, aht_accel=False)
            got, st = om_fox_decode(L, p, lim, aht_accel=True)
            assert got == ref, f"Mismatch on {fname} at limit={lim}"
            st["unaccel_steps"] = st_unaccel["steps"]
            lims_out[str(lim)] = st

        results["blocks"].append({"file": fname, "n": n, "r": entry["r"], "limits": lims_out})
        s16 = lims_out["16"]
        s8 = lims_out["8"]
        s4 = lims_out["4"]
        s1 = lims_out["1"]
        print(
            f"[6-BLOCK EXACT] {fname:11s} | "
            f"L16: steps {s16['unaccel_steps']:5d}->{s16['steps']:5d}, inline16={s16['inline16_kib']:4.1f}K (vs Open 75-118K), spell=0K | "
            f"L8: inline16={s8['inline16_kib']:4.1f}K | "
            f"L4: inline16={s4['inline16_kib']:4.1f}K | "
            f"L1: inline16={s1['inline16_kib']:4.1f}K (spell=0K vs 548K!), out_LZ={s1['out_lz_bytes']//1024:3d}KB"
        )

    out_path = pathlib.Path(__file__).resolve().parent / "cycle3_results.json"
    out_path.write_text(json.dumps(results, indent=2) + "\n")
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
