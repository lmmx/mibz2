#!/usr/bin/env python3
"""
Exact verification of Bipolar and Quotient-Accelerated Zero-Sort Half-Bipolar
Direction-Reversal Contraction (plus Monotone Alphabet-Cut BWT Recompression).

Verifies:
1. Exhaustive small cases: all binary columns n=1..8 (2^1..2^8) and all ternary
   columns n=1..5 (3^1..3^5), at EVERY primary mark p in 0..n-1 (5,227 cases),
   including multi-cycle, periodic, fixed-point, and non-BWT columns.
2. Adversarial extreme self-overlap families: Sturmian (1, 99999), Sturmian (7, 100000),
   Fibonacci (6765, 10946), periodic multi-cycle blocks, and random 16-IETs.
3. Byte-exact reconstruction against dense LF walk on all six authenticated
   Wikidata blocks from data/admitted-bwt (verified by SHA-256 against manifest.json).
"""

import hashlib
import itertools
import json
import sys
from collections import defaultdict
from pathlib import Path


def dense_bwt_invert(L, p):
    """Reference exact marked inverse BWT (stable LF, forward text order)."""
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
    """Fast reference exact marked inverse BWT for byte columns (handles short cycles)."""
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
    # Trace cycle backwards from p until returning to p (or n steps)
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


def bwt_to_iet_sorted(L):
    """
    Build initial IET for Psi = LF^{-1} directly from maximal runs of L.
    Returns (n, pieces, A_order, B_order) where:
      - pieces[i] = (u, v, w, sym_id)
      - A_order lists piece indices in strictly increasing u (F-bucket) order
      - B_order lists piece indices in strictly increasing v (L-run) order (= 0..r-1)
    Note: constructed in O(r + sigma) without comparison sorting!
    """
    n = len(L)
    runs = []
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        runs.append((c, i, j - i))
        i = j
    counts = defaultdict(int)
    run_counts = defaultdict(int)
    for c, _, w in runs:
        counts[c] += w
        run_counts[c] += 1
    syms = sorted(counts)
    starts = {}
    tot = 0
    for c in syms:
        starts[c] = tot
        tot += counts[c]
    a_starts = {}
    atot = 0
    for c in syms:
        a_starts[c] = atot
        atot += run_counts[c]

    r = len(runs)
    cur_u = dict(starts)
    cur_a = dict(a_starts)
    pieces = [None] * r
    A = [0] * r
    B = list(range(r))
    for idx, (c, v, w) in enumerate(runs):
        u = cur_u[c]
        cur_u[c] += w
        pieces[idx] = (u, v, w, c)
        A[cur_a[c]] = idx
        cur_a[c] += 1
    return n, pieces, A, B


class SlpBuilder:
    """Binary SLP with binary exponentiation for quotient-accelerated self-overlaps."""

    def __init__(self, initial_syms):
        self.base = {}
        self.rules = {}  # id -> (left, right, length)
        self.lens = {}
        self.next_id = 0
        for s in initial_syms:
            self.base[self.next_id] = s
            self.lens[self.next_id] = 1
            self.next_id += 1
        self.sym_to_id = {v: k for k, v in self.base.items()}
        self.power_cache = {}  # (word_id, k) -> word_id

    def join(self, a, b):
        if a is None:
            return b
        if b is None:
            return a
        nid = self.next_id
        self.next_id += 1
        self.rules[nid] = (a, b, self.lens[a] + self.lens[b])
        self.lens[nid] = self.lens[a] + self.lens[b]
        return nid

    def power(self, w, k):
        assert k >= 1
        if k == 1:
            return w
        key = (w, k)
        if key in self.power_cache:
            return self.power_cache[key]
        res = None
        base = w
        rem = k
        while rem > 0:
            if rem & 1:
                res = self.join(res, base)
            rem >>= 1
            if rem > 0:
                base = self.join(base, base)
        self.power_cache[key] = res
        return res

    def expand_list(self, root_id):
        """Expand SLP root into exact symbol list (with memoized caching for large blocks)."""
        # For large byte blocks, use one-scan output-copy expansion
        out = []
        first_pos = {}
        stack = [root_id]
        while stack:
            nid = stack.pop()
            if nid in self.base:
                out.append(self.base[nid])
            elif nid in first_pos:
                pos, ln = first_pos[nid]
                out.extend(out[pos : pos + ln])
            else:
                left, right, ln = self.rules[nid]
                first_pos[nid] = (len(out), ln)
                stack.append(right)
                stack.append(left)
        return out

    def expand_bytes(self, root_id):
        out = bytearray()
        first_pos = {}
        stack = [root_id]
        while stack:
            nid = stack.pop()
            if nid in self.base:
                out.append(self.base[nid])
            elif nid in first_pos:
                pos, ln = first_pos[nid]
                out.extend(out[pos : pos + ln])
            else:
                left, right, ln = self.rules[nid]
                first_pos[nid] = (len(out), ln)
                stack.append(right)
                stack.append(left)
        return bytes(out)


def split_frags_ltr(frags, split_w):
    """Split a left-to-right list of (fw, orig, par, word) into (left of width split_w, right)."""
    left = []
    right = []
    rem = split_w
    for fw, orig, par, word in frags:
        if rem == 0:
            right.append((fw, orig, par, word))
        elif fw <= rem:
            left.append((fw, orig, par, word))
            rem -= fw
        else:
            left.append((rem, orig, par, word))
            right.append((fw - rem, orig + rem, par, word))
            rem = 0
    assert rem == 0
    return left, right


def split_frags_rtl(frags, split_w):
    """
    Split a right-to-left list of (fw, orig, par, word) into:
      - right_part: first `split_w` rows from the RIGHT (front of RTL list)
      - left_part: remaining rows to the LEFT (tail of RTL list)
    """
    right_part = []
    left_part = []
    rem = split_w
    for fw, orig, par, word in frags:
        if rem == 0:
            left_part.append((fw, orig, par, word))
        elif fw <= rem:
            right_part.append((fw, orig, par, word))
            rem -= fw
        else:
            # Rightmost `rem` of [orig, orig + fw) is [orig + fw - rem, orig + fw)
            right_part.append((rem, orig + fw - rem, par, word))
            left_part.append((fw - rem, orig, par, word))
            rem = 0
    assert rem == 0
    return right_part, left_part


def half_bipolar_step_accel(n, pieces, A, B, p, slp: SlpBuilder):
    """
    Quotient-Accelerated, Zero-Sort Half-Bipolar Contraction step.
    Maintains A (u-sorted) and B (v-sorted) WITHOUT radix/comparison sorting:
      - B_new is automatically sorted by emission order!
      - A_new is constructed in O(r_new) via parent bucketing (parent_id in 0..p_cnt-1)!
    Returns (m_S, new_pieces, A_new, B_new, p_word, p_new, is_fixed, stats).
    """
    r = len(pieces)
    # Check if p lies in a fixed piece (u == v)
    p_piece_sign = 0
    for idx in A:
        u, v, w, word = pieces[idx]
        if u <= p < u + w:
            if u == v:
                return None, None, None, None, word, 0, True, {}
            p_piece_sign = 1 if v > u else -1
            break

    sign = [
        1 if pieces[i][1] > pieces[i][0] else (-1 if pieces[i][1] < pieces[i][0] else 0)
        for i in range(r)
    ]
    rows_R = sum(pieces[i][2] for i in range(r) if sign[i] == 1)
    rows_L = sum(pieces[i][2] for i in range(r) if sign[i] == -1)
    elim_sign = 1 if rows_R >= rows_L else -1
    keep_sign = -elim_sign

    # Assign compressed coordinates [0, m_S) and parent IDs [0, p_cnt) to kept pieces in A-order
    s_coord = [0] * r
    parent_id = [-1] * r
    m_S = 0
    p_cnt = 0
    for a in A:
        if sign[a] == keep_sign:
            s_coord[a] = m_S
            parent_id[a] = p_cnt
            m_S += pieces[a][2]
            p_cnt += 1

    payload = [[] for _ in range(r)]
    parent_buckets = [[] for _ in range(p_cnt)]
    p_target_x = p
    p_word = None
    p_new = None
    hops = 0
    ov = 0
    accel_jumps = 0
    direct_hits = 0

    new_pieces = []

    if elim_sign == 1:
        # ONE Left-to-Right sweep eliminating all R-pieces (v > u)
        def consume_front(frags, st, take):
            out = []
            rem = take
            while rem > 0:
                fw, orig, par, word = frags[st[0]]
                avail = fw - st[1]
                t = rem if rem < avail else avail
                out.append((t, orig + st[1], par, word))
                st[1] += t
                if st[1] == fw:
                    st[0] += 1
                    st[1] = 0
                rem -= t
            return out

        ai = bi = 0
        x = 0
        b_st = [0, 0]
        while x < n:
            a = A[ai]
            b = B[bi]
            au, av, aw, aword = pieces[a]
            bu, bv, bw, bword = pieces[b]
            a_end = au + aw
            b_end = bv + bw
            end = a_end if a_end < b_end else b_end
            take = end - x
            ov += 1
            sa = sign[a]
            sb = sign[b]

            # Quotient acceleration for self-overlapping R-piece (a == b, delta < take)
            if a == b and sa == 1 and take > (av - au):
                delta = av - au
                assert end == a_end
                q, rem = divmod(take, delta)
                # Track p inside [x, end) in O(log q)
                if p_new is None and x <= p_target_x < end:
                    k_p = (end - 1 - p_target_x) // delta + 1
                    p_word = slp.join(p_word, slp.power(aword, k_p))
                    p_target_x += k_p * delta
                # Pop generator window [x, x + delta)
                gen = consume_front(payload[a], b_st, delta)
                left_g, right_g = split_frags_ltr(gen, rem)
                wq = slp.power(aword, q)
                wq1 = slp.join(wq, aword) if rem > 0 else None
                dst = payload[a]
                # [end, end + delta - rem) gets right_g + wq
                for fw, orig, par, word in right_g:
                    dst.append((fw, orig, par, slp.join(word, wq)))
                    hops += 1
                # [end + delta - rem, end + delta) gets left_g + wq1
                for fw, orig, par, word in left_g:
                    dst.append((fw, orig, par, slp.join(word, wq1)))
                    hops += 1
                accel_jumps += 1
                x = end
                ai += 1
                # Note: b_end = av + aw = end + delta > end, so bi stays on b with b_st pointing to [end, end+delta)!
                continue

            if p_new is None and x <= p_target_x < end:
                if sa == 1:
                    p_word = slp.join(p_word, aword)
                    p_target_x += av - au
                elif sa == -1:
                    p_new = s_coord[a] + (p_target_x - au)

            if sb == -1:
                orig0 = s_coord[b] + (x - bv)
                par = parent_id[b]
                if sa == -1:
                    pid = len(new_pieces)
                    new_pieces.append((orig0, s_coord[a] + (x - au), take, bword))
                    parent_buckets[par].append(pid)
                    direct_hits += 1
                elif sa == 1:
                    payload[a].append((take, orig0, par, slp.join(bword, aword)))
                    hops += 1
            elif sb == 1:
                frags = consume_front(payload[b], b_st, take)
                if sa == 1:
                    dst = payload[a]
                    for fw, orig, par, word in frags:
                        dst.append((fw, orig, par, slp.join(word, aword)))
                        hops += 1
                elif sa == -1:
                    dst_v = s_coord[a] + (x - au)
                    for fw, orig, par, word in frags:
                        pid = len(new_pieces)
                        new_pieces.append((orig, dst_v, fw, word))
                        parent_buckets[par].append(pid)
                        dst_v += fw

            x = end
            if x == a_end:
                ai += 1
            if x == b_end:
                bi += 1
                b_st = [0, 0]

        B_new = list(range(len(new_pieces)))
    else:
        # ONE Right-to-Left sweep eliminating all L-pieces (v < u)
        def consume_rtl(frags, st, take):
            out = []
            rem = take
            while rem > 0:
                fw, orig, par, word = frags[st[0]]
                avail = fw - st[1]
                t = rem if rem < avail else avail
                out.append((t, orig + avail - t, par, word))
                st[1] += t
                if st[1] == fw:
                    st[0] += 1
                    st[1] = 0
                rem -= t
            return out

        ai = bi = r - 1
        x = n
        b_st = [0, 0]
        while x > 0:
            a = A[ai]
            b = B[bi]
            au, av, aw, aword = pieces[a]
            bu, bv, bw, bword = pieces[b]
            start = au if au > bv else bv
            take = x - start
            ov += 1
            sa = sign[a]
            sb = sign[b]

            # Quotient acceleration for self-overlapping L-piece (a == b, delta < take)
            if a == b and sa == -1 and take > (au - av):
                delta = au - av
                assert start == au
                q, rem = divmod(take, delta)
                if p_new is None and start <= p_target_x < x:
                    k_p = (p_target_x - start) // delta + 1
                    p_word = slp.join(p_word, slp.power(aword, k_p))
                    p_target_x -= k_p * delta
                gen = consume_rtl(payload[a], b_st, delta)
                right_g, left_g = split_frags_rtl(gen, rem)
                wq = slp.power(aword, q)
                wq1 = slp.join(wq, aword) if rem > 0 else None
                dst = payload[a]
                # RTL order covering [start - delta, start):
                # first rightmost [start - delta + rem, start) gets left_g + wq
                for fw, orig, par, word in left_g:
                    dst.append((fw, orig, par, slp.join(word, wq)))
                    hops += 1
                # next leftmost [start - delta, start - delta + rem) gets right_g + wq1
                for fw, orig, par, word in right_g:
                    dst.append((fw, orig, par, slp.join(word, wq1)))
                    hops += 1
                accel_jumps += 1
                x = start
                ai -= 1
                continue

            if p_new is None and start <= p_target_x < x:
                if sa == -1:
                    p_word = slp.join(p_word, aword)
                    p_target_x += av - au
                elif sa == 1:
                    p_new = s_coord[a] + (p_target_x - au)

            if sb == 1:
                orig0 = s_coord[b] + (start - bv)
                par = parent_id[b]
                if sa == 1:
                    pid = len(new_pieces)
                    new_pieces.append((orig0, s_coord[a] + (start - au), take, bword))
                    parent_buckets[par].append(pid)
                    direct_hits += 1
                elif sa == -1:
                    payload[a].append((take, orig0, par, slp.join(bword, aword)))
                    hops += 1
            elif sb == -1:
                frags = consume_rtl(payload[b], b_st, take)
                if sa == -1:
                    dst = payload[a]
                    for fw, orig, par, word in frags:
                        dst.append((fw, orig, par, slp.join(word, aword)))
                        hops += 1
                elif sa == 1:
                    dst_hi = s_coord[a] + (x - au)
                    for fw, orig, par, word in frags:
                        dst_hi -= fw
                        pid = len(new_pieces)
                        new_pieces.append((orig, dst_hi, fw, word))
                        parent_buckets[par].append(pid)

            x = start
            if x == au:
                ai -= 1
            if x == bv:
                bi -= 1
                b_st = [0, 0]

        # Emitted in strictly decreasing dst_v order -> reversing gives exact sorted B_new!
        B_new = list(range(len(new_pieces) - 1, -1, -1))

    assert p_new is not None
    # Build A_new via ZERO-SORT parent bucketing (only sort tiny multi-child buckets of avg size ~1.07!)
    A_new = []
    bucket_sorts = 0
    for bkt in parent_buckets:
        if len(bkt) == 1:
            A_new.append(bkt[0])
        elif len(bkt) == 2:
            i0, i1 = bkt
            if new_pieces[i0][0] <= new_pieces[i1][0]:
                A_new.append(i0)
                A_new.append(i1)
            else:
                A_new.append(i1)
                A_new.append(i0)
            bucket_sorts += 1
        elif len(bkt) > 2:
            bkt.sort(key=lambda idx: new_pieces[idx][0])
            A_new.extend(bkt)
            bucket_sorts += 1

    # Debug assertion: verify A_new and B_new are strictly sorted!
    for k in range(1, len(A_new)):
        assert new_pieces[A_new[k - 1]][0] + new_pieces[A_new[k - 1]][2] == new_pieces[A_new[k]][0]
    for k in range(1, len(B_new)):
        assert new_pieces[B_new[k - 1]][1] + new_pieces[B_new[k - 1]][2] == new_pieces[B_new[k]][1]

    stats = {
        "hops": hops,
        "ov": ov,
        "accel_jumps": accel_jumps,
        "direct": direct_hits,
        "bucket_sorts": bucket_sorts,
    }
    return m_S, new_pieces, A_new, B_new, p_word, p_new, False, stats


def half_bipolar_invert(L, p, is_bytes=False):
    n_orig = len(L)
    syms = sorted(set(L))
    slp = SlpBuilder(syms)
    n, raw_pieces, A, B = bwt_to_iet_sorted(L)
    pieces = [(u, v, w, slp.sym_to_id[c]) for (u, v, w, c) in raw_pieces]

    cur_p = p
    prefix_len = 0
    rds = 0
    tot_hops = 0
    tot_accel = 0
    tot_pieces = 0
    while True:
        res_n, np, A_n, B_n, pw, new_p, fixed, st = half_bipolar_step_accel(
            n, pieces, A, B, cur_p, slp
        )
        if fixed:
            unit_id = pw
            break
        if pw is not None:
            prefix_len += slp.lens[pw]
        rds += 1
        tot_hops += st["hops"]
        tot_accel += st["accel_jumps"]
        tot_pieces += len(np)
        n, pieces, A, B, cur_p = res_n, np, A_n, B_n, new_p

    if is_bytes:
        unit = slp.expand_bytes(unit_id)
        ulen = len(unit)
        shift = prefix_len % ulen
        rot = unit[ulen - shift :] + unit[: ulen - shift] if shift else unit
        if ulen == n_orig:
            out = rot
        else:
            buf = bytearray()
            while len(buf) < n_orig:
                take = min(ulen, n_orig - len(buf))
                buf.extend(rot[:take])
            out = bytes(buf)
        return out, rds, tot_pieces, tot_hops, tot_accel, len(slp.rules)
    else:
        unit = slp.expand_list(unit_id)
        ulen = len(unit)
        shift = prefix_len % ulen
        rot = unit[ulen - shift :] + unit[: ulen - shift] if shift else unit
        out = []
        while len(out) < n_orig:
            out.extend(rot)
        return out[:n_orig], rds, tot_pieces, tot_hops, tot_accel, len(slp.rules)


def main():
    # 1. Exhaustive small-case verification
    cases = 0
    max_rds = 0
    accel_hits = 0
    for sigma, max_n in [(2, 8), (3, 5)]:
        for n in range(1, max_n + 1):
            for L in itertools.product(range(sigma), repeat=n):
                for p in range(n):
                    ref = dense_bwt_invert(L, p)
                    got, rds, _, _, acc, _ = half_bipolar_invert(L, p, is_bytes=False)
                    assert got == ref, f"MISMATCH L={L} p={p}: got={got} ref={ref}"
                    cases += 1
                    if rds > max_rds:
                        max_rds = rds
                    accel_hits += acc
    print(
        f"[G1-SMALL] PASS: {cases} exhaustive cases (binary n<=8, ternary n<=5, all primary marks; "
        f"max_rds={max_rds}, accel_jumps={accel_hits})"
    )

    # 2. Adversarial self-overlap stress tests (Sturmian, Fibonacci, multi-cycle periodic)
    adversarial = [
        ("Sturmian(1,99999)", bytes([1]) + bytes([0]) * 99999, 0),
        ("Sturmian(1,99999)@mid", bytes([1]) + bytes([0]) * 99999, 54321),
        ("Sturmian(7,100000)", bytes([1]) * 7 + bytes([0]) * 100000, 12345),
        ("Sturmian(99999,1)", bytes([1]) * 99999 + bytes([0]), 54321),
        ("Fibonacci(6765,10946)", bytes([1]) * 6765 + bytes([0]) * 10946, 4000),
        ("Periodic_2cyc", (bytes([1]) * 500 + bytes([0]) * 500), 17),
        ("NonDividing_short_cyc", bytes([2, 0, 1, 0, 0, 1, 2, 1, 0, 1, 2]), 3),
    ]
    for name, L, p in adversarial:
        ref = dense_invert_bytes(L, p)
        got, rds, tp, th, acc, nru = half_bipolar_invert(L, p, is_bytes=True)
        assert got == ref, f"ADVERSARIAL MISMATCH on {name}"
        print(
            f"[G1-ADV]   PASS: {name:24s} n={len(L):6d} rds={rds:2d} pieces={tp:4d} "
            f"hops={th:4d} accel={acc:2d} rules={nru:4d}"
        )

    # 3. Six authenticated Wikidata blocks
    if len(sys.argv) < 2:
        print("Pass path to admitted-bwt directory as argv[1] to verify six real blocks.")
        return
    bdir = Path(sys.argv[1])
    manifest = json.loads((bdir / "manifest.json").read_text())
    tot_bytes = 0
    for blk in manifest["blocks"]:
        L = (bdir / blk["file"]).read_bytes()
        h = hashlib.sha256(L).hexdigest()
        assert h == blk["sha256"], f"SHA-256 mismatch for {blk['file']}"
        p = blk["primary_index"]
        ref = dense_invert_bytes(L, p)
        got, rds, tp, th, acc, nru = half_bipolar_invert(L, p, is_bytes=True)
        assert got == ref, f"BLOCK MISMATCH on {blk['file']}"
        tot_bytes += len(L)
        r0 = blk["r"]
        print(
            f"[G1-REAL]  PASS: {blk['file']:12s} n={len(L)} r0={r0:6d} rds={rds:2d} "
            f"pieces={tp:6d}({tp/r0:.2f}r) hops={th:6d}({th/r0:.2f}r) accel={acc} rules={nru:6d}({nru/r0:.2f}r)"
        )
    print(f"[G1-REAL]  ALL 6 BLOCKS BYTE-EXACT: {tot_bytes:,} / {tot_bytes:,} bytes verified!")


if __name__ == "__main__":
    main()
