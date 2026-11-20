#!/usr/bin/env python3
"""Gate G1 Exhaustive, Adversarial, and 6-Block Verifier for:
  1. Iterated Unsplit-Piece Functional In-Forest Contraction (UFC-Full) with
     Flat Round-1 literal spelling + Quotient Self-Overlap Acceleration (for extreme Sturmian cases).
  2. UFC-1 + Open (1 round of Flat UFC followed by Open endpoint elimination on the
     reduced IET T_K, retaining only thin pieces w <= 16).

Verifies:
- Exhaustive small cases: all binary columns n <= 8 (510 columns) and all ternary
  columns n <= 5 (363 columns), at every primary mark p in 0..n-1 (5,227 cases each for
  UFC-Full and UFC-1+Open = 10,454 total small cases, including multi-cycle, periodic,
  fixed-point, and early U-cycle closures).
- Adversarial extreme self-overlap cases: Sturmian(1, 99999), Sturmian(99999, 1),
  Fibonacci(6765, 10946), periodic blocks, single-symbol blocks.
- Six authenticated admitted Wikidata BWT blocks (5,399,692 bytes total) byte-exact
  against a dense LF reference walk, reporting exact Flat-R1 + SLP-R2..K statistics.
"""
import json, os, time
from bisect import bisect_right
from itertools import product

DATA_DIR = "/home/antigravity/.gemini/jetski/brain/v1/default/jetski_remote_agents_3p/7e9ef9b7-4d8c-42f5-9d61-045e6541ce3d/scratch/admitted-bwt"


def dense_reference(L: bytes, p: int) -> bytes:
    n = len(L)
    cnt = [0] * 256
    for b in L:
        cnt[b] += 1
    C = [0] * 256
    t = 0
    for c in range(256):
        C[c] = t
        t += cnt[c]
    cur = C[:]
    LF = [0] * n
    for i, b in enumerate(L):
        LF[i] = cur[b]
        cur[b] += 1
    cyc_rev = bytearray()
    x = p
    while True:
        cyc_rev.append(L[x])
        x = LF[x]
        if x == p or len(cyc_rev) == n:
            break
    cyc = bytes(reversed(cyc_rev))
    if len(cyc) == n:
        return cyc
    reps = (n + len(cyc) - 1) // len(cyc)
    return (cyc * reps)[:n]


def ufc_round1_flat(L: bytes, p: int):
    """Exact Round 1 of Unsplit-Piece Functional In-Forest Contraction in F-order
    with flat literal byte spelling (zero SLP rules created in Round 1!).
    Returns either:
      ("CLOSED", cyc_bytes, mark_shift, stats) if marked cycle closed wholly inside U, OR
      ("NEXT", new_n, new_p, mark_shift, pieces, lit_words, stats)
    where each piece is (from, to, w, lit_id) and lit_words[lit_id] is a bytes object.
    """
    n = len(L)
    # Pass 1: maximal runs in L-order
    l_from = []
    l_sym = bytearray()
    sym_rcnt = [0] * 256
    p_lrun = -1
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        if i <= p < j:
            p_lrun = len(l_from)
        l_from.append(i)
        l_sym.append(c)
        sym_rcnt[c] += 1
        i = j
    r = len(l_from)
    l_from.append(n)

    if r == 1:
        return ("CLOSED", bytes([l_sym[0]]), 0, {"r": 1, "K": 0, "arcs": 0, "hops": 0, "lit_bytes": 1})

    # Pass 2: assign F-order run index fi for each L-run k (zero random reads!)
    sym_roff = [0] * 256
    t = 0
    for c in range(256):
        sym_roff[c] = t
        t += sym_rcnt[c]
    sym_rend = sym_roff[1:] + [r]

    f_of_l = [0] * r
    f_w = [0] * r
    cur_off = sym_roff[:]
    for k in range(r):
        c = l_sym[k]
        fi = cur_off[c]
        cur_off[c] = fi + 1
        f_of_l[k] = fi
        f_w[fi] = l_from[k + 1] - l_from[k]
    p_fi = f_of_l[p_lrun]
    p_in_run = p - l_from[p_lrun]

    # Pass 3: linear 2-pointer merge (F-order fi vs L-order v) -> forest[fi] and raw_arcs
    succ = [-1] * r
    delta = [0] * r
    f_sym = bytearray(r)
    k_id_of_fi = [-1] * r
    k_new_from = []
    new_n = 0
    raw_arcs = []  # (k_id, sub_off, ow, v_fi, in_v, sym_c)

    v = 0
    t0 = 0
    sym_c = 0
    for fi in range(r):
        while fi >= sym_rend[sym_c]:
            sym_c += 1
        f_sym[fi] = sym_c
        w = f_w[fi]
        t1 = t0 + w
        while l_from[v + 1] <= t0:
            v += 1
        if l_from[v + 1] >= t1:
            succ[fi] = f_of_l[v]
            delta[fi] = t0 - l_from[v]
        else:
            kid = len(k_new_from)
            k_id_of_fi[fi] = kid
            k_new_from.append(new_n)
            new_n += w
            vv = v
            while vv < r and l_from[vv] < t1:
                ov0 = t0 if t0 > l_from[vv] else l_from[vv]
                ov1 = t1 if t1 < l_from[vv + 1] else l_from[vv + 1]
                raw_arcs.append((kid, ov0 - t0, ov1 - ov0, f_of_l[vv], ov0 - l_from[vv], sym_c))
                vv += 1
            v = vv - 1
        t0 = t1

    # Check if p_fi rides U into K or closes a pure cycle in U!
    mark_shift = 0
    if succ[p_fi] != -1:
        cur = p_fi
        seen = {}
        path_bytes = bytearray()
        acc_d = 0
        while succ[cur] != -1 and cur not in seen:
            seen[cur] = len(path_bytes)
            path_bytes.append(f_sym[cur])
            acc_d += delta[cur]
            cur = succ[cur]
        if succ[cur] != -1:
            # Closed pure cycle inside U containing p_fi!
            # Note: since U is functional and p_fi is on a cycle, cur MUST be p_fi!
            assert cur == p_fi
            # path_bytes = [L[p], L[LF(p)], ...] in backward LF order -> reverse for forward period!
            return ("CLOSED", bytes(reversed(path_bytes)), 0, {
                "r": r, "K": len(k_new_from), "arcs": 0, "hops": len(path_bytes), "lit_bytes": len(path_bytes)
            })
        # Otherwise cur is a splitter in K!
        mark_shift = len(path_bytes)
        new_p = k_new_from[k_id_of_fi[cur]] + acc_d + p_in_run
    else:
        new_p = k_new_from[k_id_of_fi[p_fi]] + p_in_run

    # Pass 4: trace all raw_arcs through forest[0..r] into flat literal byte strings!
    pieces = []
    lit_words = []
    total_hops = 0
    total_lit_bytes = 0
    buf = bytearray()
    for kid, sub_off, ow, v_fi, in_v, sym_c in raw_arcs:
        sub_from = k_new_from[kid] + sub_off
        if succ[v_fi] == -1:
            sub_to = k_new_from[k_id_of_fi[v_fi]] + in_v
            lid = len(lit_words)
            lit_words.append(bytes([sym_c]))
            total_lit_bytes += 1
            pieces.append((sub_from, sub_to, ow, lid))
        else:
            cur = v_fi
            acc_d = 0
            buf.clear()
            buf.append(sym_c)
            while succ[cur] != -1:
                buf.append(f_sym[cur])
                acc_d += delta[cur]
                cur = succ[cur]
                total_hops += 1
            rt_kid = k_id_of_fi[cur]
            sub_to = k_new_from[rt_kid] + acc_d + in_v
            buf.reverse()
            lid = len(lit_words)
            lit_words.append(bytes(buf))
            total_lit_bytes += len(buf)
            pieces.append((sub_from, sub_to, ow, lid))

    return ("NEXT", new_n, new_p, mark_shift, pieces, lit_words, {
        "r": r, "K": len(k_new_from), "arcs": len(pieces),
        "hops": total_hops, "lit_bytes": total_lit_bytes,
    })


def ufc_full_reconstruct(L: bytes, p: int):
    """Flat-Round-1 + Iterated UFC (with Quotient Self-Overlap Acceleration) to 1 piece!"""
    n = len(L)
    r1 = ufc_round1_flat(L, p)
    if r1[0] == "CLOSED":
        _, cyc, sh, st1 = r1
        L_cyc = len(cyc)
        sh %= L_cyc
        rot = cyc[sh:] + cyc[:sh]
        res = (rot * ((n + L_cyc - 1) // L_cyc))[:n] if L_cyc < n else rot
        return res, {"rounds": 1, "r1": st1, "slp_rules": 0, "exp_lits": 1, "exp_copies": 0, "q_accels": 0}

    _, cur_n, cur_p, mark_shift, pieces, lit_words, st1 = r1
    n_lits = len(lit_words)
    # Word IDs: 0..n_lits-1 = flat literal words from Round 1; >= n_lits = SLP rules!
    rules = []  # (left, right, byte_len)

    def wlen(wid):
        return len(lit_words[wid]) if wid < n_lits else rules[wid - n_lits][2]

    def join(left, right):
        if left == -1: return right
        if right == -1: return left
        rid = n_lits + len(rules)
        rules.append((left, right, wlen(left) + wlen(right)))
        return rid

    def pow_word(wid, q):
        # Binary exponentiation for W^q in O(log q) rules!
        res_w = -1
        base_w = wid
        while q > 0:
            if q & 1:
                res_w = join(res_w, base_w)
            q >>= 1
            if q > 0:
                base_w = join(base_w, base_w)
        return res_w

    rounds = 1
    q_accels = 0
    closed_word = -1

    while True:
        rounds += 1
        r = len(pieces)
        froms = [x[0] for x in pieces]
        p_idx = bisect_right(froms, cur_p) - 1

        if r == 1:
            closed_word = pieces[0][3]
            break

        # Quotient Self-Overlap Acceleration check:
        # If any piece k maps [fr, fr+w) to [t0, t0+w) with 0 < |t0 - fr| < w and w >= 2 * |t0 - fr|,
        # we can contract its self-overlapping interior in O(log q) before the forest step!
        # Specifically: if t0 < fr and fr - t0 = d < w (backward shift inside piece k):
        # Rows [fr + d, fr + w) map into [fr, fr + w - d) inside piece k itself!
        # Wait: even simpler, if w >= 2*d for a self-overlapping piece k, split piece k into its
        # tail (exiting k) and accelerated self-loop prefix — OR run exact quotient reduction on piece k!
        accel_done = False
        for k in range(r):
            fr, t0, w, wid = pieces[k]
            if fr < t0 < fr + w:
                d = t0 - fr
                if w >= 2 * d:
                    # Forward self-overlap [fr, fr+w) -> [fr+d, fr+w+d):
                    # Sub-interval [fr, fr+w-d) maps to [fr+d, fr+w) INSIDE piece k!
                    # Any x in [fr, fr+d) hops q or q+1 times inside k until landing in [fr+w, fr+w+d)!
                    # Specifically, eliminating [fr+d, fr+w) leaves only [fr, fr+d) (width d)!
                    # Write w = q*d + rem (q >= 2, 0 <= rem < d).
                    # For offset u in [0, rem): takes q+1 steps -> lands at fr + w + u (exit [fr+w, fr+w+rem)), word W^{q+1}!
                    # For offset u in [rem, d): takes q steps -> lands at fr + w - d + u... wait:
                    # x = fr + u -> after t steps: fr + u + t*d. First >= fr + w:
                    #   if u < rem: fr + u + q*d < fr + w, so needs q+1 steps -> lands at fr + u + (q+1)*d = fr + w + d - (rem - u)!
                    #   if u >= rem: fr + u + q*d = fr + w + (u - rem) >= fr + w, so takes q steps!
                    q, rem = divmod(w, d)
                    # Check if cur_p is in eliminated [fr+d, fr+w)!
                    if fr + d <= cur_p < fr + w:
                        steps_to_exit = (fr + w - cur_p + d - 1) // d
                        mark_shift += steps_to_exit * wlen(wid)
                        cur_p = cur_p + steps_to_exit * d
                    # Delete [fr+d, fr+w) (size w - d) from domain and codomain!
                    cut0 = fr + d
                    cut_len = w - d
                    w_q = pow_word(wid, q)
                    w_q1 = join(w_q, wid) if rem > 0 else -1
                    # Map any coordinate y outside [cut0, cut0+cut_len) to compressed space
                    def comp_coord(y):
                        return y if y < cut0 else y - cut_len
                    new_pieces = []
                    for i2, (f2, t2, w2, wid2) in enumerate(pieces):
                        if i2 == k:
                            # Emit [fr, fr+rem) -> [fr+w+d-rem, fr+w+d) with W^{q+1},
                            # and [fr+rem, fr+d) -> [fr+w, fr+w+d-rem) with W^q!
                            if rem > 0:
                                new_pieces.append((comp_coord(fr), comp_coord(fr + w + d - rem), rem, w_q1))
                            new_pieces.append((comp_coord(fr + rem), comp_coord(fr + w), d - rem, w_q))
                        else:
                            # Note: f2 and t2 cannot strictly overlap [cut0, cut0+cut_len) because
                            # domain [fr, fr+w) and codomain [fr+d, fr+w+d) are disjoint from other pieces!
                            # Wait: t2 IS disjoint from [fr+d, fr+w+d) which CONTAINS [cut0, cut0+cut_len)!
                            # AND f2 IS disjoint from [fr, fr+w) which CONTAINS [cut0, cut0+cut_len)!
                            new_pieces.append((comp_coord(f2), comp_coord(t2), w2, wid2))
                    cur_p = comp_coord(cur_p)
                    cur_n -= cut_len
                    pieces = new_pieces
                    q_accels += 1
                    accel_done = True
                    break
            elif t0 < fr < t0 + w:
                d = fr - t0
                if w >= 2 * d:
                    # Backward self-overlap [fr, fr+w) -> [fr-d, fr+w-d):
                    # Sub-interval [fr+d, fr+w) maps to [fr, fr+w-d) INSIDE piece k!
                    # Eliminating [fr, fr+w-d) (size w - d) leaves ONLY [fr+w-d, fr+w) (width d)!
                    # Check: domain [fr, fr+w) contains [fr, fr+w-d); codomain [fr-d, fr+w-d) contains [fr, fr+w-d)!
                    # For x in [fr+w-d, fr+w), write x = fr + w - d + u (0 <= u < d):
                    # After t steps: x - t*d = fr + w - (t+1)*d + u. First < fr:
                    # Write w = q*d + rem (q >= 2, 0 <= rem < d).
                    # At t = q: fr + rem - d + u:
                    #   if u < d - rem: < fr! Lands at fr - d + rem + u (in [fr-d+rem, fr)), q steps!
                    #   if u >= d - rem: >= fr (in [fr, fr+rem)), takes 1 more step (q+1) -> lands at fr - d + (u - (d - rem)) (in [fr-d, fr-d+rem))!
                    q, rem = divmod(w, d)
                    cut0 = fr
                    cut_len = w - d
                    if cut0 <= cur_p < cut0 + cut_len:
                        steps_to_exit = (cur_p - fr) // d + 1
                        mark_shift += steps_to_exit * wlen(wid)
                        cur_p = cur_p - steps_to_exit * d
                    w_q = pow_word(wid, q)
                    w_q1 = join(w_q, wid) if rem > 0 else -1
                    def comp_coord(y):
                        return y if y < cut0 else y - cut_len
                    new_pieces = []
                    for i2, (f2, t2, w2, wid2) in enumerate(pieces):
                        if i2 == k:
                            # [fr+w-d, fr+w-rem) (width d-rem) -> [fr-d+rem, fr) with W^q
                            # [fr+w-rem, fr+w) (width rem) -> [fr-d, fr-d+rem) with W^{q+1}
                            new_pieces.append((comp_coord(fr + w - d), comp_coord(fr - d + rem), d - rem, w_q))
                            if rem > 0:
                                new_pieces.append((comp_coord(fr + w - rem), comp_coord(fr - d), rem, w_q1))
                        else:
                            new_pieces.append((comp_coord(f2), comp_coord(t2), w2, wid2))
                    cur_p = comp_coord(cur_p)
                    cur_n -= cut_len
                    pieces = new_pieces
                    q_accels += 1
                    accel_done = True
                    break
        if accel_done:
            continue

        # Standard UFC forest step
        succ = [-1] * r
        delta = [0] * r
        for k in range(r):
            fr, t0, w, wid = pieces[k]
            t1 = t0 + w
            v = bisect_right(froms, t0) - 1
            if froms[v] + pieces[v][2] >= t1:
                succ[k] = v
                delta[k] = t0 - froms[v]

        state = [0] * r
        root = [-1] * r
        acc_delta = [0] * r
        acc_word = [-1] * r
        for k in range(r):
            if succ[k] == -1:
                state[k] = 2
                root[k] = k
                acc_delta[k] = 0
                acc_word[k] = -1

        for u in range(r):
            if state[u] != 0:
                continue
            path = []
            c = u
            while state[c] == 0:
                state[c] = 1
                path.append(c)
                c = succ[c]
            if state[c] == 1:
                ci = path.index(c)
                cyc = path[ci:]
                for node in path:
                    state[node] = 3
                if p_idx in cyc:
                    pi = cyc.index(p_idx)
                    w_tot = -1
                    L_c = len(cyc)
                    for step in range(L_c):
                        node = cyc[(pi + step) % L_c]
                        w_tot = join(pieces[node][3], w_tot)
                    closed_word = w_tot
                    break
                continue
            elif state[c] == 3:
                for node in path:
                    state[node] = 3
                continue
            else:
                rt = root[c]
                ad = acc_delta[c]
                aw = acc_word[c]
                for node in reversed(path):
                    ad = delta[node] + ad
                    aw = join(aw, pieces[node][3])
                    root[node] = rt
                    acc_delta[node] = ad
                    acc_word[node] = aw
                    state[node] = 2

        if closed_word != -1:
            break

        if succ[p_idx] != -1:
            assert state[p_idx] == 2
            mark_shift += wlen(acc_word[p_idx])
            in_p = cur_p - pieces[p_idx][0]
            rt_p = root[p_idx]
            cur_p = pieces[rt_p][0] + acc_delta[p_idx] + in_p
            p_idx = rt_p

        K_new_from = [-1] * r
        new_n = 0
        for k in range(r):
            if succ[k] == -1:
                K_new_from[k] = new_n
                new_n += pieces[k][2]
        new_p = K_new_from[p_idx] + (cur_p - pieces[p_idx][0])

        next_pieces = []
        for k in range(r):
            if succ[k] != -1:
                continue
            fr, t0, w, wid_k = pieces[k]
            t1 = t0 + w
            base_from = K_new_from[k]
            v = bisect_right(froms, t0) - 1
            while v < r and froms[v] < t1:
                ov0 = max(t0, froms[v])
                ov1 = min(t1, froms[v] + pieces[v][2])
                ow = ov1 - ov0
                sub_from = base_from + (ov0 - t0)
                in_v = ov0 - froms[v]
                if succ[v] == -1:
                    sub_to = K_new_from[v] + in_v
                    sub_wid = wid_k
                else:
                    rt = root[v]
                    sub_to = K_new_from[rt] + acc_delta[v] + in_v
                    sub_wid = join(acc_word[v], wid_k)
                next_pieces.append((sub_from, sub_to, ow, sub_wid))
                v += 1

        pieces = next_pieces
        cur_n = new_n
        cur_p = new_p

    # Expand SLP (memoized first-occurrence copy)
    L_cyc = wlen(closed_word)
    first_pos = [-1] * len(rules)
    out_cyc = bytearray()
    stack = [closed_word]
    exp_lits = 0
    exp_copies = 0
    while stack:
        wid = stack.pop()
        if wid < n_lits:
            out_cyc.extend(lit_words[wid])
            exp_lits += 1
        else:
            ri = wid - n_lits
            fp = first_pos[ri]
            if fp == -1:
                first_pos[ri] = len(out_cyc)
                l, rr, _ = rules[ri]
                stack.append(rr)
                stack.append(l)
            else:
                ln = rules[ri][2]
                out_cyc.extend(out_cyc[fp:fp + ln])
                exp_copies += 1

    assert len(out_cyc) == L_cyc
    sh = mark_shift % L_cyc
    cyc_bytes = bytes(out_cyc[sh:]) + bytes(out_cyc[:sh])
    res = (cyc_bytes * ((n + L_cyc - 1) // L_cyc))[:n] if L_cyc < n else cyc_bytes
    return res, {
        "rounds": rounds, "r1": st1, "slp_rules": len(rules),
        "exp_lits": exp_lits, "exp_copies": exp_copies, "q_accels": q_accels,
    }


def ufc1_open_reconstruct(L: bytes, p: int, LIMIT: int = 16):
    """Exact UFC-1 + Open: 1 round of Flat UFC followed by Open first-return elimination
    on thick pieces (w > LIMIT) of the reduced IET T_K, and walk on thin pieces (w <= LIMIT)."""
    n = len(L)
    r1 = ufc_round1_flat(L, p)
    if r1[0] == "CLOSED":
        _, cyc, sh, st1 = r1
        L_cyc = len(cyc)
        sh %= L_cyc
        rot = cyc[sh:] + cyc[:sh]
        res = (rot * ((n + L_cyc - 1) // L_cyc))[:n] if L_cyc < n else rot
        return res, {"m1": 0, "strips": 0, "r1": st1}

    _, new_n, new_p, mark_shift, pieces, lit_words, st1 = r1
    # Partition pieces into thick strips (w > LIMIT, splitting around new_p if inside) and thin pieces (w <= LIMIT)
    thins = []   # (thin_idx_start, to, w, lid)
    strips = []  # (from, to, w, lid)
    m1 = 0
    mark = 0
    for fr, to, w, lid in pieces:
        end = fr + w
        if w > LIMIT and fr <= new_p < end:
            if fr < new_p:
                strips.append((fr, to, new_p - fr, lid))
            mark = m1
            thins.append((m1, to + (new_p - fr), 1, lid))
            m1 += 1
            if new_p + 1 < end:
                strips.append((new_p + 1, to + (new_p + 1 - fr), end - new_p - 1, lid))
        elif w <= LIMIT:
            if fr <= new_p < end:
                mark = m1 + (new_p - fr)
            thins.append((m1, to, w, lid))
            m1 += w
        else:
            strips.append((fr, to, w, lid))

    # Open endpoint elimination on thick strips over [0, new_n)
    ns = len(strips)
    s_ord = sorted(range(ns), key=lambda i: strips[i][1])
    nn = 1
    prev0 = [0]; next0 = [0]; prev1 = [0]; next1 = [0]; wid = [0]; word = [0]

    def add_node(w_val, wd_val):
        nonlocal nn
        idx = nn; nn += 1
        prev0.append(0); next0.append(0); prev1.append(0); next1.append(0)
        wid.append(w_val); word.append(wd_val)
        return idx

    n_lits = len(lit_words)
    for fr, to, w, lid in strips:
        add_node(w, lid + 1)

    ord0 = []; term1 = []; x = 0
    for idx, (fr, to, w, lid) in enumerate(strips):
        if x < fr:
            nid = add_node(fr - x, 0); ord0.append(nid); term1.append(nid)
        ord0.append(idx + 1); x = fr + w
    if x < new_n:
        nid = add_node(new_n - x, 0); ord0.append(nid); term1.append(nid)

    ord1 = []; term0 = []; x = 0
    for idx in s_ord:
        fr, to, w, lid = strips[idx]
        if x < to:
            nid = add_node(to - x, 0); ord1.append(nid); term0.append(nid)
        ord1.append(idx + 1); x = to + w
    if x < new_n:
        nid = add_node(new_n - x, 0); ord1.append(nid); term0.append(nid)

    for side, tarr, oarr in [(0, term0, ord0), (1, term1, ord1)]:
        pr = prev0 if side == 0 else prev1
        nx = next0 if side == 0 else next1
        p0 = 0
        for nid in tarr:
            nx[p0] = nid; pr[nid] = p0; p0 = nid
        for nid in oarr:
            nx[p0] = nid; pr[nid] = p0; p0 = nid
        nx[p0] = 0; pr[0] = p0

    rules = []  # (left, right)
    rem = new_n
    while rem > 0:
        ix = prev0[0]; iy = prev1[0]
        wa = wid[ix]; wb = wid[iy]
        w = wa if wa < wb else wb
        rem -= w
        if ix == iy:
            next0[prev0[ix]] = next0[ix]; prev0[next0[ix]] = prev0[ix]
            next1[prev1[ix]] = next1[ix]; prev1[next1[ix]] = prev1[ix]
            continue
        if word[ix] == 0:
            wd = word[iy]
        elif word[iy] == 0:
            wd = word[ix]
        else:
            rules.append((word[ix], word[iy]))
            wd = n_lits + len(rules)
        if wa <= wb:
            next0[prev0[ix]] = next0[ix]; prev0[next0[ix]] = prev0[ix]
            nx = next0[iy]; prev0[ix] = iy; next0[ix] = nx; next0[iy] = ix; prev0[nx] = ix
            word[ix] = wd
            if wa == wb:
                next0[prev0[iy]] = next0[iy]; prev0[next0[iy]] = prev0[iy]
                next1[prev1[iy]] = next1[iy]; prev1[next1[iy]] = prev1[iy]
            else:
                wid[iy] -= w
        else:
            next1[prev1[iy]] = next1[iy]; prev1[next1[iy]] = prev1[iy]
            nx = next1[ix]; prev1[iy] = ix; next1[iy] = nx; next1[ix] = iy; prev1[nx] = iy
            word[iy] = wd; wid[ix] -= w

    bottom = [0] * nn
    cur = next1[0]; x = 0
    while cur != 0:
        bottom[cur] = x; x += wid[cur]; cur = next1[cur]
    assert x == m1

    memo_rule = {}
    def expand_ep_word(wd):
        if wd == 0:
            return b""
        if wd <= n_lits:
            return lit_words[wd - 1]
        if wd in memo_rule:
            return memo_rule[wd]
        out_b = bytearray()
        stk = [wd]
        while stk:
            u = stk.pop()
            if u <= n_lits:
                out_b.extend(lit_words[u - 1])
            elif u in memo_rule:
                out_b.extend(memo_rule[u])
            else:
                l_w, r_w = rules[u - n_lits - 1]
                stk.append(r_w); stk.append(l_w)
        res_b = bytes(out_b)
        memo_rule[wd] = res_b
        return res_b

    bands = []  # (end, delta, bytes)
    cur = next0[0]; k = 0
    while cur != 0:
        bands.append((k + wid[cur], bottom[cur] - k, expand_ep_word(word[cur])))
        k += wid[cur]; cur = next0[cur]

    # Wire edges[0..m1] in image order of thins
    t_ord = sorted(range(len(thins)), key=lambda i: thins[i][1])
    edges_nxt = [0] * m1
    edges_bytes = [b""] * m1
    si = 0; thick_before = 0; bi = 0
    for tid in t_ord:
        src, to, rw, lid = thins[tid]
        while si < ns and strips[s_ord[si]][1] < to:
            thick_before += strips[s_ord[si]][2]; si += 1
        y = to - thick_before
        lw = lit_words[lid]
        while rw > 0:
            while bands[bi][0] <= y:
                bi += 1
            bend, bdelta, bbytes = bands[bi]
            take = rw if rw < (bend - y) else (bend - y)
            dst0 = y + bdelta
            comb = bbytes + lw if bbytes else lw
            for u in range(take):
                edges_nxt[dst0 + u] = src + u
                edges_bytes[dst0 + u] = comb
            y += take; src += take; rw -= take

    # Walk marked cycle from mark (handles short periods / multi-cycles cleanly!)
    out_cyc = bytearray()
    j = mark
    while True:
        out_cyc.extend(edges_bytes[j])
        j = edges_nxt[j]
        if j == mark or len(out_cyc) >= n:
            break
    L_cyc = len(out_cyc)
    sh = mark_shift % L_cyc
    cyc_bytes = bytes(out_cyc[sh:]) + bytes(out_cyc[:sh])
    res = (cyc_bytes * ((n + L_cyc - 1) // L_cyc))[:n] if L_cyc < n else cyc_bytes[:n]
    return res, {"m1": m1, "strips": ns, "r1": st1}


def run_small_exhaustive():
    t0 = time.time()
    cases = 0
    q_cases = 0
    for n in range(1, 9):
        for bits in product((0, 1), repeat=n):
            L = bytes(bits)
            for p in range(n):
                ref = dense_reference(L, p)
                got_f, st_f = ufc_full_reconstruct(L, p)
                assert got_f == ref, f"FAIL UFC-Full bin n={n} L={list(L)} p={p}"
                got_o, _ = ufc1_open_reconstruct(L, p, LIMIT=2)
                assert got_o == ref, f"FAIL UFC-1+Open bin n={n} L={list(L)} p={p}"
                cases += 1
                if st_f["q_accels"] > 0:
                    q_cases += 1
    for n in range(1, 6):
        for syms in product((0, 1, 2), repeat=n):
            L = bytes(syms)
            for p in range(n):
                ref = dense_reference(L, p)
                got_f, st_f = ufc_full_reconstruct(L, p)
                assert got_f == ref, f"FAIL UFC-Full tern n={n} L={list(L)} p={p}"
                got_o, _ = ufc1_open_reconstruct(L, p, LIMIT=2)
                assert got_o == ref, f"FAIL UFC-1+Open tern n={n} L={list(L)} p={p}"
                cases += 1
                if st_f["q_accels"] > 0:
                    q_cases += 1
    print(f"SMALL EXHAUSTIVE PASS: {cases:,} UFC-Full + {cases:,} UFC-1+Open = {2*cases:,} / {2*cases:,} cases ({q_cases} quotient-accelerated) in {time.time()-t0:.2f}s")


def run_adversarial():
    tests = [
        ("Sturmian(1,99999)", bytes([1]) + bytes([0]) * 99999, 0),
        ("Sturmian(99999,1)", bytes([0]) * 99999 + bytes([1]), 50000),
        ("Sturmian(7,99993)", bytes([1]) * 7 + bytes([0]) * 99993, 12345),
        ("Fibonacci(6765,10946)", bytes([1]) * 6765 + bytes([0]) * 10946, 4321),
        ("AllZero(10000)", bytes([0]) * 10000, 333),
        ("Periodic(abc x 1000)", b"c" * 1000 + b"a" * 1000 + b"b" * 1000, 17),
        ("MultiCycle(010101)", b"\x00\x01\x00\x01\x00\x01", 3),
    ]
    print("\n=== ADVERSARIAL TESTS (UFC-Full & UFC-1+Open) ===")
    for name, L, p in tests:
        ref = dense_reference(L, p)
        got_f, st = ufc_full_reconstruct(L, p)
        got_o, sto = ufc1_open_reconstruct(L, p, LIMIT=16)
        assert got_f == ref and got_o == ref, f"FAIL adversarial {name}!"
        print(f"  {name:22s}: n={len(L):6d} rds={st['rounds']:2d} q_acc={st['q_accels']:2d} slp={st['slp_rules']:3d} | UFC1+Open(m1={sto['m1']:3d}, str={sto['strips']:2d}) -> PASS")


def run_six_blocks():
    manifest = json.load(open(os.path.join(DATA_DIR, "manifest.json")))
    print("\n=== SIX AUTHENTICATED WIKIDATA BLOCKS (UFC-Full & UFC-1+Open) ===")
    tot_b = 0
    for b in manifest["blocks"]:
        L = open(os.path.join(DATA_DIR, b["file"]), "rb").read()
        ref = dense_reference(L, b["primary_index"])
        got_f, st = ufc_full_reconstruct(L, b["primary_index"])
        got_o, sto = ufc1_open_reconstruct(L, b["primary_index"], LIMIT=16)
        assert got_f == ref and got_o == ref, f"MISMATCH on {b['file']}!"
        tot_b += len(ref)
        r = b["r"]
        r1 = st["r1"]
        print(
            f"  {b['file']:11s}: r={r:6d} | R1(K={r1['K']:4d}, arcs={r1['arcs']:5d}, hops={r1['hops']:6d}, lit={r1['lit_bytes']//1024:3d} KiB) | "
            f"UFC-Full(rds={st['rounds']:2d}, slp={st['slp_rules']:5d} [{st['slp_rules']/r:.2f}r], mean_cp={len(ref)/(st['exp_lits']+st['exp_copies']):.1f}B) | "
            f"UFC1+Open(m1={sto['m1']:5d}, strips={sto['strips']:4d}) -> PASS"
        )
    print(f"TOTAL VERIFIED BYTES: {tot_b:,} / {tot_b:,} (100% BYTE-EXACT, BOTH ARCHITECTURES)")


if __name__ == "__main__":
    run_small_exhaustive()
    run_adversarial()
    run_six_blocks()
