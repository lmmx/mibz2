#!/usr/bin/env python3
"""
Carbon Kitchen Sink — Cycle 2 Gate G1 Exhaustive Verifier (verify_cycle2.py).

Verifies byte-exactness against dense LF on all binary n=1..8 (3,586 pairs),
all ternary n=1..5 (1,641 pairs), and all 6 authenticated Wikidata blocks for:
  1. OIBS-H (Hysteresis-gated Online In-Place Bundle Shortcutting on Open's edges,
     with zero writes to edges on the hot path and zero construct changes), tested
     across hysteresis thresholds h_trig in {1, 2, 4} and limits in {1, 2, 16}.
  2. LRB-Open (Direct Pre-Column MTF+RLE2 Stream Fusion into Open via Cycle 1 Theorem 3,
     eliminating Runs.runs and CAP=127 splitting/re-merging).
  3. LRB-OIBS (Combined MTF->Open stream fusion + Hysteresis-4 OIBS emit).
"""

from __future__ import annotations
import itertools
import json
import pathlib
import sys
import time

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent / "2026-11-20-carbon-kitchen-sink"))
from verify_ttsr import dense_lf_invert
from probe_cycle2 import build_open_edges_exact

BLOCKS_DIR = pathlib.Path("/home/antigravity/admitted-bwt")


def encode_exact_bzip2_syms(L: bytes) -> tuple[list[int], list[int], int]:
    """
    Encode raw column L into exact libbzip2 MTF+RLE2 Symbols:
      returns (seq_to_unseq, syms_with_eob, eob).
    Symbols: 0 = RUNA, 1 = RUNB, 2..alpha = MTF index 1..alpha-1 (+1), alpha+1 = EOB.
    """
    present = sorted(set(L))
    alpha = len(present)
    eob = alpha + 1
    yy = list(present)
    syms = []

    n = len(L)
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        w = j - i
        idx = yy.index(c)
        if idx == 0:
            assert i == 0
            rem = w
        else:
            syms.append(idx + 1)
            yy.pop(idx)
            yy.insert(0, c)
            rem = w - 1
        # Bijective base-2: RUNA (0) = 1*weight, RUNB (1) = 2*weight
        while rem > 0:
            rem -= 1
            syms.append(rem & 1)
            rem >>= 1
        i = j

    syms.append(eob)
    return present, syms, eob


def lrb_open_construct_from_syms(seq_to_unseq: list[int], syms: list[int], eob: int, p: int, limit: int = 16):
    """
    Direct MTF+RLE2 -> Open construct (LRB-Open):
    Exploits Exact MTF-Run Bijection: every s >= 2 (before eob) starts a guaranteed
    DISTINCT maximal run (no c == last_c check, no CAP=127 split, no records buffer!).
    """
    yy = list(seq_to_unseq)
    all_c = [0] * 256
    thin_c = [0] * 256
    thick_c = [0] * 256
    queues = [[] for _ in range(256)]
    strips = []

    pos = 0
    m = 0
    mark = 0

    def flush_maximal_run(c: int, width: int):
        nonlocal pos, m, mark
        end = pos + width
        if width > limit and pos <= p < end:
            if pos < p:
                w1 = p - pos
                strips.append([pos, all_c[c], w1, c])
                thick_c[c] += 1
                all_c[c] += w1
            mark = m
            queues[c].append(m)
            thin_c[c] += 1
            m += 1
            all_c[c] += 1
            if p + 1 < end:
                w2 = end - (p + 1)
                strips.append([p + 1, all_c[c], w2, c])
                thick_c[c] += 1
                all_c[c] += w2
        elif width <= limit:
            if pos <= p < end:
                mark = m + (p - pos)
            queues[c].extend(range(m, m + width))
            thin_c[c] += width
            m += width
            all_c[c] += width
        else:
            strips.append([pos, all_c[c], width, c])
            thick_c[c] += 1
            all_c[c] += width
        pos = end

    # State: have_open_run tracks whether a literal has started a run waiting for RLE2 weight
    have_open = False
    cur_c = yy[0]
    run_extra = 0
    weight = 1

    for s in syms:
        if s <= 1:
            run_extra += (s + 1) * weight
            weight <<= 1
            continue
        # Flush previous run if any!
        if have_open:
            flush_maximal_run(cur_c, 1 + run_extra)
            run_extra = 0
            weight = 1
        elif run_extra > 0:
            # Initial RUNA/RUNB run of yy[0] before first literal!
            flush_maximal_run(yy[0], run_extra)
            run_extra = 0
            weight = 1
        if s == eob:
            break
        nn = s - 1
        cur_c = yy.pop(nn)
        yy.insert(0, cur_c)
        have_open = True

    n = pos
    # Run exact same endpoint + wiring as Open!
    # Verify queues & strips match build_open_edges_exact via full emit check below
    first = [0] * 256
    write_p = [0] * 256
    ta = 0
    tt = 0
    for a in range(256):
        first[a] = ta
        ta += all_c[a]
        write_p[a] = tt
        tt += thick_c[a]

    H = len(strips)
    image_order = [0] * H
    for si, st in enumerate(strips):
        a = st[3]
        st[1] += first[a]
        image_order[write_p[a]] = si
        write_p[a] += 1

    node_w = [0]
    node_word = [0]
    for st in strips:
        node_w.append(st[2])
        node_word.append(st[3] + 1)

    order0, order1, term0, term1 = [], [], [], []
    x = 0
    for si, st in enumerate(strips):
        if x < st[0]:
            gid = len(node_w)
            node_w.append(st[0] - x)
            node_word.append(0)
            order0.append(gid)
            term1.append(gid)
        order0.append(si + 1)
        x = st[0] + st[2]
    if x < n:
        gid = len(node_w)
        node_w.append(n - x)
        node_word.append(0)
        order0.append(gid)
        term1.append(gid)

    x = 0
    for si in image_order:
        st = strips[si]
        if x < st[1]:
            gid = len(node_w)
            node_w.append(st[1] - x)
            node_word.append(0)
            order1.append(gid)
            term0.append(gid)
        order1.append(si + 1)
        x = st[1] + st[2]
    if x < n:
        gid = len(node_w)
        node_w.append(n - x)
        node_word.append(0)
        order1.append(gid)
        term0.append(gid)

    K = len(node_w) - 1
    prev0, next0, prev1, next1 = [0] * (K + 1), [0] * (K + 1), [0] * (K + 1), [0] * (K + 1)
    prv = 0
    for nid in term0 + order0:
        next0[prv] = nid; prev0[nid] = prv; prv = nid
    next0[prv] = 0; prev0[0] = prv
    prv = 0
    for nid in term1 + order1:
        next1[prv] = nid; prev1[nid] = prv; prv = nid
    next1[prv] = 0; prev1[0] = prv

    rules_l, rules_r, rules_len, rules_first = [], [], [], []
    def tlen(w): return (1 if w != 0 else 0) if w <= 256 else rules_len[w - 257]
    def join(l, rt):
        if l == 0: return rt
        if rt == 0: return l
        rid = 257 + len(rules_l)
        rules_l.append(l); rules_r.append(rt); rules_len.append(tlen(l) + tlen(rt)); rules_first.append(-1)
        return rid
    def cut(ap, an, uid):
        pr, nx = ap[uid], an[uid]; an[pr] = nx; ap[nx] = pr
    def ins(ap, an, at, uid):
        nx = an[at]; ap[uid] = at; an[uid] = nx; an[at] = uid; ap[nx] = uid

    rem = n
    while rem > 0:
        xn, yn = prev0[0], prev1[0]
        wx, wy = node_w[xn], node_w[yn]
        wc = wx if wx < wy else wy
        rem -= wc
        if xn == yn:
            cut(prev0, next0, xn); cut(prev1, next1, xn); continue
        comb = join(node_word[xn], node_word[yn])
        if wx <= wy:
            cut(prev0, next0, xn); ins(prev0, next0, yn, xn); node_word[xn] = comb
            if wx == wy: cut(prev0, next0, yn); cut(prev1, next1, yn)
            else: node_w[yn] = wy - wc
        else:
            cut(prev1, next1, yn); ins(prev1, next1, xn, yn); node_word[yn] = comb; node_w[xn] = wx - wc

    bottom = [0] * (K + 1)
    nid = next1[0]; bx = 0
    while nid != 0:
        bottom[nid] = bx; bx += node_w[nid]; nid = next1[nid]

    spelling = bytearray(); offsets = [0]
    def app_word(wt):
        stk = [wt]
        while stk:
            tid = stk.pop()
            if tid <= 256: spelling.append(tid - 1)
            else:
                ri = tid - 257; fp = rules_first[ri]
                if fp == -1:
                    rules_first[ri] = len(spelling); stk.append(rules_r[ri]); stk.append(rules_l[ri])
                else:
                    ln = rules_len[ri]; spelling.extend(spelling[fp:fp + ln])

    bits = max(1, (m - 1).bit_length())
    # Ensure m < (1 << bits) so (edge + 1) never overflows next into code!
    if m == (1 << bits):
        bits += 1
    edges = [0] * m
    nid = next0[0]; in_node = 0; k = 0
    for a in range(256):
        end_a = k + thin_c[a]
        qa = queues[a]; q_from = 0
        while k < end_a:
            nw = node_w[nid]
            bw = min(end_a - k, nw - in_node)
            dst = bottom[nid] + in_node
            if node_word[nid] == 0:
                code = a
            else:
                app_word(node_word[nid]); spelling.append(a); offsets.append(len(spelling)); code = 254 + len(offsets)
            pack = code << bits
            for u in range(bw):
                edges[dst + u] = pack | qa[q_from + u]
            q_from += bw; in_node += bw; k += bw
            if in_node == nw:
                nid = next0[nid]; in_node = 0

    return n, m, mark, bits, edges, bytes(spelling), offsets


def emit_oibs_hysteresis(
    n: int,
    m: int,
    mark: int,
    bits: int,
    edges_in: list[int],
    spelling: bytes,
    offsets: list[int],
    h_trig: int = 4,
    k_min: int = 4,
) -> tuple[bytes, dict]:
    """
    Hysteresis-gated Read-Only Hot-Loop Online In-Place Bundle Shortcutting (OIBS-H):
      - Hot loop makes ZERO writes to `edges`!
      - Padded sentinel slots at index 0 and m+1 eliminate bounds checks on j-1 and j+1!
      - Branchless `streak` counter triggers cold sheet-mode ONLY when `(e & SHORTCUT_BIT) or streak == h_trig`!
    """
    # Ensure m < (1 << bits) so e +/- 1 cannot wrap mask into code when next is in [0, m)
    assert m <= (1 << bits)
    mask = (1 << bits) - 1
    SHORTCUT_BIT = 1 << 31
    VISITED_LEADER = 0x7FFFFFFE

    # Pad edges by +1 on left and right with sentinel 0x7FFFFFFD (cannot match e +/- 1 or SHORTCUT_BIT)
    SENTINEL = 0x7FFFFFFD
    padded = [SENTINEL] + list(edges_in) + [SENTINEL]
    # Shift indices by +1? Or keep helper access at idx + 1 (in C: `uint32_t *edges = buf + 1;`)!
    sc_next = []
    sc_pos = []
    sc_len = []

    out = bytearray()
    j = mark
    streak = 0
    cold_entries = 0
    shortcuts_taken = 0
    shortcuts_written = 0
    hops = 0

    for _ in range(m):
        e = padded[j + 1]
        hops += 1
        # Check cold condition: shortcut hit OR streak reached h_trig!
        if not ((e & SHORTCUT_BIT) or streak >= h_trig):
            # HOT PATH: ZERO writes to padded[], branchless streak update!
            nxt = e & mask
            # Note: if m == (1 << bits), guard nxt+1 < m; if m < (1 << bits), padded[j+2] == e+1 implies
            # either same code & nxt+1 <= mask (< m if m == (1<<bits)? Wait: if m < (1<<bits), nxt+1 could equal m,
            # which no valid edge has as low bits! Wait: every valid edge in padded[1..m] has low bits in 0..m-1!
            # So if nxt == m - 1, e + 1 has low bits m (if m < 1<<bits) or 0 (if m == 1<<bits)!
            # Low bits m CANNOT exist in any valid edge! Only if m == (1<<bits) could low bits wrap to 0 with code+1!
            # Since we enforced m < (1<<bits) when constructing, e + 1 NEVER false-matches ANY valid edge!!
            coh = (padded[j + 2] == e + 1) or (padded[j] == e - 1)
            streak = (streak + 1) if coh else 0

            code = e >> bits
            j = nxt
            if code < 256:
                out.append(code)
            else:
                w = code - 256
                s0, s1 = offsets[w], offsets[w + 1]
                out.extend(spelling[s0:s0 + min(s1 - s0, n - len(out))])
            if j == mark or len(out) >= n:
                break
            continue

        cold_entries += 1
        if e & SHORTCUT_BIT:
            sid = e & 0x7FFFFFFF
            s_p = sc_pos[sid]
            s_l = sc_len[sid]
            j = sc_next[sid]
            streak = 0
            shortcuts_taken += 1
            take = min(s_l, n - len(out))
            # Copy from earlier output (note: source [s_p, s_p+take) was written strictly earlier, disjoint from len(out)!)
            out.extend(out[s_p:s_p + take])
            if j == mark or len(out) >= n:
                break
            continue

        # Cold path: streak >= h_trig and e is a normal edge!
        streak = 0
        # Find maximal coherent interval [-L_span, +R_span] around j (up to +-15), never crossing `mark`!
        L_span = 0
        while L_span < 15 and (j - (L_span + 1)) != mark and padded[j + 1 - (L_span + 1)] == e - (L_span + 1):
            L_span += 1
        R_span = 0
        while R_span < 15 and (j + (R_span + 1)) != mark and padded[j + 1 + (R_span + 1)] == e + (R_span + 1):
            R_span += 1

        if L_span == 0 and R_span == 0:
            # Hysteresis streak ended right at j: emit j normally and continue!
            code = e >> bits
            j = e & mask
            if code < 256:
                out.append(code)
            else:
                w = code - 256
                s0, s1 = offsets[w], offsets[w + 1]
                out.extend(spelling[s0:s0 + min(s1 - s0, n - len(out))])
            if j == mark or len(out) >= n:
                break
            continue

        start_j = j
        start_out = len(out)
        cur_j = j
        cur_L, cur_R = L_span, R_span
        sheet_steps = 0

        while True:
            ce = padded[cur_j + 1]
            if (ce & SHORTCUT_BIT) or ce == VISITED_LEADER:
                break
            if sheet_steps > 0:
                hops += 1
                nl = 0
                while nl < cur_L and (cur_j - (nl + 1)) != mark and padded[cur_j + 1 - (nl + 1)] == ce - (nl + 1):
                    nl += 1
                nr = 0
                while nr < cur_R and (cur_j + (nr + 1)) != mark and padded[cur_j + 1 + (nr + 1)] == ce + (nr + 1):
                    nr += 1
                if nl == 0 and nr == 0:
                    break
                if sheet_steps >= k_min and (nl < cur_L or nr < cur_R):
                    blen = len(out) - start_out
                    for d in list(range(-cur_L, -nl)) + list(range(nr + 1, cur_R + 1)):
                        sj = start_j + d
                        if sj != mark and padded[sj + 1] != VISITED_LEADER and not (padded[sj + 1] & SHORTCUT_BIT):
                            sid = len(sc_next)
                            sc_next.append(cur_j + d)
                            sc_pos.append(start_out)
                            sc_len.append(blen)
                            padded[sj + 1] = SHORTCUT_BIT | sid
                            shortcuts_written += 1
                cur_L, cur_R = nl, nr

            padded[cur_j + 1] = VISITED_LEADER
            ccode = ce >> bits
            cur_j = ce & mask
            if ccode < 256:
                out.append(ccode)
            else:
                w = ccode - 256
                s0, s1 = offsets[w], offsets[w + 1]
                out.extend(spelling[s0:s0 + min(s1 - s0, n - len(out))])
            sheet_steps += 1
            if cur_j == mark or len(out) >= n:
                break

        if sheet_steps >= k_min and (cur_L > 0 or cur_R > 0):
            blen = len(out) - start_out
            for d in list(range(-cur_L, 0)) + list(range(1, cur_R + 1)):
                sj = start_j + d
                if sj != mark and padded[sj + 1] != VISITED_LEADER and not (padded[sj + 1] & SHORTCUT_BIT):
                    sid = len(sc_next)
                    sc_next.append(cur_j + d)
                    sc_pos.append(start_out)
                    sc_len.append(blen)
                    padded[sj + 1] = SHORTCUT_BIT | sid
                    shortcuts_written += 1

        j = cur_j
        if j == mark or len(out) >= n:
            break

    del out[n:]
    plen = len(out)
    assert plen > 0
    while len(out) < n:
        take = min(plen, n - len(out))
        out.extend(out[:take])

    return bytes(out), {
        "m": m,
        "hops": hops,
        "hops_ratio": round(hops / m, 4),
        "cold_entries": cold_entries,
        "cold_pct_hops": round(100.0 * cold_entries / max(1, hops), 2),
        "shortcuts_written": shortcuts_written,
        "shortcuts_taken": shortcuts_taken,
    }


def main():
    t0 = time.time()
    small_checks = 0
    for n in range(1, 9):
        for bits_seq in itertools.product((0, 1), repeat=n):
            L = bytes(bits_seq)
            seq_u, syms, eob = encode_exact_bzip2_syms(L)
            for p in range(n):
                ref = dense_lf_invert(L, p)
                for lim in (1, 2, 16):
                    n2, m, mark, b_bits, edges, spelling, offsets = lrb_open_construct_from_syms(seq_u, syms, eob, p, limit=lim)
                    assert n2 == n
                    for ht, km in ((1, 2), (2, 2), (4, 4)):
                        got, _ = emit_oibs_hysteresis(n, m, mark, b_bits, edges, spelling, offsets, h_trig=ht, k_min=km)
                        assert got == ref, f"Binary mismatch n={n} p={p} lim={lim} ht={ht}"
                        small_checks += 1
    print(f"PASS exhaustive binary n=1..8 (LRB-Open + OIBS-H x 9 configs): {small_checks} / {small_checks} byte-exact")

    ter_checks = 0
    for n in range(1, 6):
        for trits in itertools.product((0, 1, 2), repeat=n):
            L = bytes(trits)
            seq_u, syms, eob = encode_exact_bzip2_syms(L)
            for p in range(n):
                ref = dense_lf_invert(L, p)
                for lim in (1, 16):
                    n2, m, mark, b_bits, edges, spelling, offsets = lrb_open_construct_from_syms(seq_u, syms, eob, p, limit=lim)
                    for ht, km in ((1, 2), (4, 4)):
                        got, _ = emit_oibs_hysteresis(n, m, mark, b_bits, edges, spelling, offsets, h_trig=ht, k_min=km)
                        assert got == ref, f"Ternary mismatch n={n} p={p} lim={lim} ht={ht}"
                        ter_checks += 1
    print(f"PASS exhaustive ternary n=1..5 (LRB-Open + OIBS-H x 4 configs): {ter_checks} / {ter_checks} byte-exact")

    manifest = json.loads((BLOCKS_DIR / "manifest.json").read_text())
    for item in manifest["blocks"]:
        fname = item["file"]
        p = item["primary_index"]
        L = (BLOCKS_DIR / fname).read_bytes()
        ref = dense_lf_invert(L, p)
        seq_u, syms, eob = encode_exact_bzip2_syms(L)
        n2, m, mark, b_bits, edges, spelling, offsets = lrb_open_construct_from_syms(seq_u, syms, eob, p, limit=16)
        got, stats = emit_oibs_hysteresis(n2, m, mark, b_bits, edges, spelling, offsets, h_trig=4, k_min=4)
        assert got == ref, f"Block mismatch on {fname}!"
        print(f"PASS {fname}: byte-exact | {json.dumps(stats)}")

    print(f"\nALL {small_checks + ter_checks} SMALL CONFIGS + 6 BLOCKS PASSED IN {time.time() - t0:.2f}s")


if __name__ == "__main__":
    main()
