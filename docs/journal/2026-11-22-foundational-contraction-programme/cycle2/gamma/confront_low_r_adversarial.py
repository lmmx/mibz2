#!/usr/bin/env python3
"""
Cycle 2 Mission 1: Adversarial Confrontation of Sub-Interval 1-Piece Schur (Widest-Piece Euclidean Spiral)
against:
  1. Referee #2's `Low-r Padded-Ring` (k=4, |Sigma|=3, r=39 constant, K = 4, 16, 64, 256, 1024)
     under ALL 6 elimination orders of {a, b, c}!
  2. Beta's `MultiTag_M_runs(K)` (r=17 constant, K = 16, 64, 256, 1024) & `MultiTag_MX(K)` & `Alt_ab_ac(K)`!
  3. 3-letter `Symmetric-Middle-Run` T = M^K A M^K Z (r=4 constant, K = 4, 16, 64, 256, 1024)!
  4. Period-3 / Period-4 synthetic non-monotone self-IET stress test!
Measures exact steps, ops, peak piece count max_r, SLP size, and verifies 100% byte-for-byte exactness!
"""
import bisect
import itertools
import sys
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-c")
from explore_iet_schur import build_initial_iet, decode_lf_reference, expand_slp
from test_unified_schur import euclid_accelerate_2iet_bucket


def bwt_encode_fast(T):
    """O(n log n) suffix doubling BWT (exact for arbitrary bytes/ints)."""
    n = len(T)
    s = list(T)
    sa = list(range(n))
    rk = list(s)
    k = 1
    while k < n:
        sa.sort(key=lambda i: (rk[i], rk[(i + k) % n]))
        tmp = [0] * n
        for j in range(1, n):
            a, b = sa[j - 1], sa[j]
            tmp[b] = tmp[a] + (1 if (rk[a], rk[(a + k) % n]) != (rk[b], rk[(b + k) % n]) else 0)
        rk = tmp
        if rk[sa[-1]] == n - 1:
            break
        k <<= 1
    L = [s[(i - 1) % n] for i in sa]
    p = sa.index(0)
    return L, p


def de_bruijn(k, n_order):
    a = [0] * (k * n_order)
    seq = []
    def db(t, p):
        if t > n_order:
            if n_order % p == 0:
                seq.extend(a[1:p + 1])
        else:
            a[t] = a[t - p]
            db(t + 1, p)
            for j in range(a[t - p] + 1, k):
                a[t] = j
                db(t + 1, t)
    db(1, 1)
    return seq


def subinterval_schur_eliminate_bucket(e, pieces_by_src, slp, mode="widest_self", coalesce=True):
    """
    Eliminate bucket e using Sub-Interval 1-Piece Schur (widest-piece spiral inside F_e),
    optionally coalescing adjacent pieces with identical shift & dst_bucket?
    Wait: can we coalesce adjacent pieces? Only if they ALSO have identical word_id!
    Wait — two adjacent pieces with identical shift AND identical word_id have the exact same
    output string AND exact same translation, so coalescing them is 100% exact!
    Let's track BOTH raw (no coalesce) and exact-word-coalesced piece counts!
    Also handles eliminating ALL remaining pieces of the final bucket down to 1 single piece
    (stopping at 1 row, e.g., p) when asked, OR eliminating bucket e completely into K = S \ {e}!
    """
    out_all = pieces_by_src.pop(e)
    in_pieces = []
    for b, plist in list(pieces_by_src.items()):
        keep = []
        for p in plist:
            if p[4] == e:
                in_pieces.append(p)
            else:
                keep.append(p)
        pieces_by_src[b] = keep

    # Run Sub-Interval 1-Piece Schur on F_e!
    cur_pieces = list(out_all)
    ext_in = list(in_pieces)

    def slp_cat(l, r):
        idx = len(slp)
        slp.append(('cat', l, r))
        return idx

    def slp_rep(b, k):
        if k == 1:
            return b
        idx = len(slp)
        slp.append(('rep', b, k))
        return idx

    steps = 0
    total_ops = 0
    max_cur_pieces = len(cur_pieces)
    max_ext_pieces = len(ext_in)

    while cur_pieces:
        self_idxs = [i for i, p in enumerate(cur_pieces) if p[4] == e]
        if not self_idxs:
            break
        if mode == "widest_self":
            big_i = max(self_idxs, key=lambda i: (cur_pieces[i][1], -cur_pieces[i][0]))
        elif mode == "widest_any":
            big_i = max(range(len(cur_pieces)), key=lambda i: (cur_pieces[i][1], -cur_pieces[i][0]))
        else:
            raise ValueError(mode)

        A, L_big, D_big, _, db_big, W_big = cur_pieces.pop(big_i)
        B = A + L_big
        delta = D_big - A
        steps += 1
        if steps > 50000:
            return {'aborted': True, 'steps': steps, 'ops': total_ops, 'max_cur': max_cur_pieces}

        def route_list(plist):
            nonlocal total_ops
            out_list = []
            for p in plist:
                s, l, d, sb, db, wid = p
                if db != e:
                    out_list.append(p)
                    continue
                if d < A:
                    take = min(l, A - d)
                    out_list.append((s, take, d, sb, db, wid))
                    s += take
                    d += take
                    l -= take
                if l <= 0:
                    continue
                if d < B:
                    take = min(l, B - d)
                    u = d
                    if db_big != e:
                        # Single exit hop!
                        total_ops += 1
                        nw = slp_cat(W_big, wid)
                        out_list.append((s, take, D_big + (u - A), sb, db_big, nw))
                    else:
                        if delta == 0:
                            raise RuntimeError("Entered fixed point from outside")
                        sub_stack = [(s, take, u, wid)]
                        while sub_stack:
                            ss, sl, su, sw = sub_stack.pop()
                            total_ops += 1
                            sv = su + sl
                            if delta > 0:
                                k_max = (B - 1 - su) // delta + 1
                                k_min = (B - 1 - (sv - 1)) // delta + 1
                                if k_max == k_min:
                                    nw = slp_cat(slp_rep(W_big, k_max), sw)
                                    out_list.append((ss, sl, su + k_max * delta, sb, e, nw))
                                else:
                                    split_x = B - (k_max - 1) * delta
                                    l1 = split_x - su
                                    if l1 < sl:
                                        sub_stack.append((ss + l1, sl - l1, split_x, sw))
                                    nw = slp_cat(slp_rep(W_big, k_max), sw)
                                    out_list.append((ss, l1, su + k_max * delta, sb, e, nw))
                            else:
                                step = -delta
                                k_min = (su - A) // step + 1
                                k_max = (sv - 1 - A) // step + 1
                                if k_min == k_max:
                                    nw = slp_cat(slp_rep(W_big, k_min), sw)
                                    out_list.append((ss, sl, su - k_min * step, sb, e, nw))
                                else:
                                    split_x = A + k_min * step
                                    l1 = split_x - su
                                    if l1 < sl:
                                        sub_stack.append((ss + l1, sl - l1, split_x, sw))
                                    nw = slp_cat(slp_rep(W_big, k_min), sw)
                                    out_list.append((ss, l1, su - k_min * step, sb, e, nw))
                    s += take
                    d += take
                    l -= take
                if l <= 0:
                    continue
                out_list.append((s, l, d, sb, db, wid))
            return out_list

        cur_pieces = route_list(cur_pieces)
        ext_in = route_list(ext_in)
        if len(cur_pieces) > max_cur_pieces:
            max_cur_pieces = len(cur_pieces)
        if len(ext_in) > max_ext_pieces:
            max_ext_pieces = len(ext_in)

    # Final merge of remaining exit pieces (0 self-loops!)
    cur_pieces.sort(key=lambda x: x[0])
    cur_starts = [x[0] for x in cur_pieces]
    for s, l, d, sb, db, wid in ext_in:
        rem = l
        cs, cd = s, d
        while rem > 0:
            total_ops += 1
            oi = bisect.bisect_right(cur_starts, cd) - 1
            os_, ol_, od_, _, odb_, owid_ = cur_pieces[oi]
            off = cd - os_
            take = min(rem, ol_ - off)
            pieces_by_src[sb].append((cs, take, od_ + off, sb, odb_, slp_cat(owid_, wid)))
            cs += take
            cd += take
            rem -= take

    for b in pieces_by_src:
        pieces_by_src[b].sort(key=lambda x: x[0])

    return {
        'aborted': False,
        'steps': steps,
        'ops': total_ops,
        'max_cur': max_cur_pieces,
        'max_ext': max_ext_pieces,
        'r_after': sum(len(v) for v in pieces_by_src.values()),
    }


def solve_final_single_bucket_subinterval(pieces, slp, p_row):
    """
    Collapse the FINAL surviving bucket F_{a*} ALL THE WAY DOWN to 1 single piece on {p_row}
    using Sub-Interval 1-Piece Schur (never eliminating p_row itself)!
    Specifically, split any piece containing p_row into [s, p_row), [p_row, p_row+1), [p_row+1, s+l),
    protect [p_row, p_row+1), and eliminate ALL other pieces widest-first!
    """
    # Isolate [p_row, p_row+1) as a separate protected piece!
    cur_pieces = []
    ext_in = []  # just the 1 protected piece [p_row, p_row+1)!
    e_sym = pieces[0][3]
    for s, l, d, sb, db, wid in pieces:
        if s <= p_row < s + l:
            if p_row > s:
                cur_pieces.append((s, p_row - s, d, sb, db, wid))
            ext_in.append((p_row, 1, d + (p_row - s), sb, db, wid))
            if p_row + 1 < s + l:
                cur_pieces.append((p_row + 1, s + l - (p_row + 1), d + (p_row + 1 - s), sb, db, wid))
        else:
            cur_pieces.append((s, l, d, sb, db, wid))

    def slp_cat(l, r):
        idx = len(slp)
        slp.append(('cat', l, r))
        return idx

    def slp_rep(b, k):
        if k == 1:
            return b
        idx = len(slp)
        slp.append(('rep', b, k))
        return idx

    steps = 0
    total_ops = 0
    max_cur = len(cur_pieces)

    while cur_pieces:
        # Pick widest piece in cur_pieces (all map within F_{a*} except those landing on p_row!)
        # Note: a piece in cur_pieces cannot have p_row in its domain; its image CAN hit p_row!
        big_i = max(range(len(cur_pieces)), key=lambda i: (cur_pieces[i][1], -cur_pieces[i][0]))
        A, L_big, D_big, _, _, W_big = cur_pieces.pop(big_i)
        B = A + L_big
        delta = D_big - A
        steps += 1
        if steps > 50000:
            return None, steps, total_ops, max_cur

        def route_list(plist):
            nonlocal total_ops
            out_list = []
            for s, l, d, sb, db, wid in plist:
                if d < A:
                    take = min(l, A - d)
                    out_list.append((s, take, d, sb, db, wid))
                    s += take
                    d += take
                    l -= take
                if l <= 0:
                    continue
                if d < B:
                    take = min(l, B - d)
                    u = d
                    if delta == 0:
                        raise RuntimeError("Fixed point")
                    sub_stack = [(s, take, u, wid)]
                    while sub_stack:
                        ss, sl, su, sw = sub_stack.pop()
                        total_ops += 1
                        sv = su + sl
                        if delta > 0:
                            k_max = (B - 1 - su) // delta + 1
                            k_min = (B - 1 - (sv - 1)) // delta + 1
                            if k_max == k_min:
                                nw = slp_cat(slp_rep(W_big, k_max), sw)
                                out_list.append((ss, sl, su + k_max * delta, sb, db, nw))
                            else:
                                split_x = B - (k_max - 1) * delta
                                l1 = split_x - su
                                if l1 < sl:
                                    sub_stack.append((ss + l1, sl - l1, split_x, sw))
                                nw = slp_cat(slp_rep(W_big, k_max), sw)
                                out_list.append((ss, l1, su + k_max * delta, sb, db, nw))
                        else:
                            step = -delta
                            k_min = (su - A) // step + 1
                            k_max = (sv - 1 - A) // step + 1
                            if k_min == k_max:
                                nw = slp_cat(slp_rep(W_big, k_min), sw)
                                out_list.append((ss, sl, su - k_min * step, sb, db, nw))
                            else:
                                split_x = A + k_min * step
                                l1 = split_x - su
                                if l1 < sl:
                                    sub_stack.append((ss + l1, sl - l1, split_x, sw))
                                nw = slp_cat(slp_rep(W_big, k_min), sw)
                                out_list.append((ss, l1, su - k_min * step, sb, db, nw))
                    s += take
                    d += take
                    l -= take
                if l <= 0:
                    continue
                out_list.append((s, l, d, sb, db, wid))
            return out_list

        cur_pieces = route_list(cur_pieces)
        ext_in = route_list(ext_in)
        if len(cur_pieces) > max_cur:
            max_cur = len(cur_pieces)

    assert len(ext_in) == 1 and ext_in[0][0] == p_row and ext_in[0][2] == p_row
    return ext_in[0][5], steps, total_ops, max_cur


def run_full_subinterval_schur_with_order(L, p, elim_order):
    """Eliminate buckets in exact order elim_order[:-1], then collapse final bucket elim_order[-1] to 1 SLP root!"""
    alpha, C, counts, pieces_by_src, slp, r0 = build_initial_iet(L)
    # Track p into final bucket (exact standalone tracking!)
    cur_p = p
    cur_pb = next(c for c in alpha if C[c] <= p < C[c] + counts[c])
    wlen = [1] * len(slp)

    def ensure_wlen():
        while len(wlen) < len(slp):
            node = slp[len(wlen)]
            if node[0] == 'lit':
                wlen.append(1)
            elif node[0] == 'cat':
                wlen.append(wlen[node[1]] + wlen[node[2]])
            else:
                wlen.append(wlen[node[1]] * node[2])

    shift_bytes = 0
    tot_steps = 0
    tot_ops = 0
    max_r = sum(len(v) for v in pieces_by_src.values())

    for e in elim_order[:-1]:
        if e not in pieces_by_src:
            continue
        # Track cur_p if in e (before popping e!)
        if cur_pb == e:
            out_e = sorted(pieces_by_src[e], key=lambda x: x[0])
            starts_e = [x[0] for x in out_e]
            ensure_wlen()
            while True:
                oi = bisect.bisect_right(starts_e, cur_p) - 1
                os_, ol_, od_, _, odb_, owid_ = out_e[oi]
                off = cur_p - os_
                if odb_ != e:
                    cur_p = od_ + off
                    cur_pb = odb_
                    shift_bytes += wlen[owid_]
                    break
                else:
                    delta = od_ - os_
                    A, B = os_, os_ + ol_
                    if delta > 0:
                        k = (B - 1 - cur_p) // delta + 1
                        cur_p += k * delta
                    else:
                        step = -delta
                        k = (cur_p - A) // step + 1
                        cur_p -= k * step
                    shift_bytes += k * wlen[owid_]

        st = subinterval_schur_eliminate_bucket(e, pieces_by_src, slp)
        if st['aborted']:
            return None, {'aborted': True, 'steps': tot_steps + st['steps'], 'ops': tot_ops + st['ops'], 'max_r': max_r}
        tot_steps += st['steps']
        tot_ops += st['ops']
        max_r = max(max_r, st['max_cur'], st['r_after'])

    final_b = elim_order[-1]
    assert cur_pb == final_b
    root_wid, f_steps, f_ops, f_max = solve_final_single_bucket_subinterval(pieces_by_src[final_b], slp, cur_p)
    if root_wid is None:
        return None, {'aborted': True, 'steps': tot_steps + f_steps, 'ops': tot_ops + f_ops, 'max_r': max(max_r, f_max)}
    tot_steps += f_steps
    tot_ops += f_ops
    max_r = max(max_r, f_max)

    out = []
    expand_slp(slp, root_wid, out)
    sh = shift_bytes % len(out)
    out = out[sh:] + out[:sh] if sh else out
    return out, {
        'aborted': False,
        'r0': r0,
        'steps': tot_steps,
        'ops': tot_ops,
        'max_r': max_r,
        'slp_nodes': len(slp),
    }


def main():
    print("=== 1. CONFRONTATION: Referee #2 Low-r Padded-Ring (k=4, |Sigma|=3, r=39 CONSTANT) — ALL 6 ORDERS ===")
    db2 = de_bruijn(2, 4)
    for K in [4, 16, 64, 256, 1024]:
        w_ab = b"".join((b"a" if x == 0 else b"b") * K for x in db2)
        w_bc = b"".join((b"b" if x == 0 else b"c") * K for x in db2)
        w_ca = b"".join((b"c" if x == 0 else b"a") * K for x in db2)
        T = list(w_ab + w_bc + w_ca)
        L, p = bwt_encode_fast(T)
        r0 = 1 + sum(1 for i in range(1, len(L)) if L[i] != L[i - 1])
        print(f"\n  K={K:4d}, n={len(T):6d}, r0={r0} (r0/n={r0/len(T):.5f}):")
        for perm in itertools.permutations([ord('a'), ord('b'), ord('c')]):
            out, st = run_full_subinterval_schur_with_order(L, p, list(perm))
            ord_s = "".join(chr(c) for c in perm)
            if st['aborted']:
                print(f"    order={ord_s}: ABORTED steps={st['steps']} ops={st['ops']} max_r={st['max_r']}")
            else:
                assert out == T, f"MISMATCH on Low-r Padded-Ring K={K}, order={ord_s}"
                print(f"    order={ord_s}: steps={st['steps']:4d}, ops={st['ops']:5d} ({st['ops']/r0:5.2f}r0, {st['ops']/len(T):.5f}n), "
                      f"max_r={st['max_r']:4d}, slp={st['slp_nodes']:5d} [EXACT!]")

    print("\n=== 2. CONFRONTATION: Beta MultiTag_M_runs(K), MultiTag_MX(K), Alt_ab_ac(K), SymMidRun(K) ===")
    families = [
        ("SymMidRun(M^K A M^K Z)", lambda K: list(b"M" * K + b"A" + b"M" * K + b"Z")),
        ("Alt_ab_ac", lambda K: list(b"ab" * K + b"ac" * K + b"!")),
        ("MultiTag_M_runs", lambda K: list(b"".join(bytes([65 + i]) + b"M" * K + bytes([80 + i]) for i in range(8)))),
        ("MultiTag_MX", lambda K: list(b"".join(bytes([65 + i]) + b"MX" * K + bytes([80 + i]) for i in range(8)))),
    ]
    for name, gen in families:
        print(f"\n  Family [{name}]:")
        for K in [16, 64, 256, 1024]:
            T = gen(K)
            L, p = bwt_encode_fast(T)
            alpha = sorted(set(L))
            # Test ascending, descending, and freq-ascending (keeping most frequent last)
            counts = defaultdict(int)
            for c in L:
                counts[c] += 1
            orders = [
                ("asc", alpha),
                ("desc", list(reversed(alpha))),
                ("freq_asc", sorted(alpha, key=lambda c: (counts[c], c))),
                ("freq_desc", sorted(alpha, key=lambda c: (-counts[c], c))),
            ]
            line = []
            r0 = 1 + sum(1 for i in range(1, len(L)) if L[i] != L[i - 1])
            for oname, ord_list in orders:
                out, st = run_full_subinterval_schur_with_order(L, p, ord_list)
                assert not st['aborted'] and out == T, f"FAIL {name} K={K} {oname}"
                line.append(f"{oname}:st={st['steps']},ops={st['ops']},mr={st['max_r']},slp={st['slp_nodes']}")
            print(f"    K={K:4d} n={len(T):5d} r0={r0:2d} | " + " | ".join(line))


if __name__ == "__main__":
    main()
