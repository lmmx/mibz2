#!/usr/bin/env python3
"""
Exact Compositional Redheffer Star Product of Boundary Transfer Maps (BTM-WIET v2)
over ARBITRARY symbol subsets S subset Sigma (zero boundary splits!).

Key Theorem (Target-Bucket Closure):
For ANY symbol subset S subset Sigma, let U_S = union_{c in S} F_c.
Every piece P = ([lo, hi), delta, target_sym, slp_id, mark_off) of Phi_{U_S}
maps 100% into a SINGLE target F-bucket F_{target_sym} with target_sym not in S.
Proof: At the exit step t = tau(x), L[f^{t-1}(x)] = c is constant across the piece,
so f^t([lo, hi)) subset F_c with c not in S. QED.

Consequence:
  1. Merging Phi_{S1} * Phi_{S2} for ANY disjoint S1, S2 subset Sigma produces
     ZERO boundary splits! A piece of Phi_{S1} is internal iff target_sym in S2.
  2. Any binary merge tree over Sigma is exact and associative:
     - byte-order balanced tree
     - frequency-ordered (rarest symbol at root -> tiny root cycle!)
     - bigram-weighted heavy-edge agglomerative tree (absorbs high-traffic edges deep in tree)
"""

import hashlib
import json
import sys
import time
from bisect import bisect_right
from collections import defaultdict


class SLPStore:
    __slots__ = ("kind", "a", "b", "length")

    def __init__(self):
        # 0 = lit(a), 1 = concat(a, b), 2 = power(a, b)
        self.kind = [0] * 256
        self.a = list(range(256))
        self.b = [0] * 256
        self.length = [1] * 256

    def concat(self, u, v):
        idx = len(self.kind)
        self.kind.append(1)
        self.a.append(u)
        self.b.append(v)
        self.length.append(self.length[u] + self.length[v])
        return idx

    def power(self, u, k):
        if k == 1:
            return u
        idx = len(self.kind)
        self.kind.append(2)
        self.a.append(u)
        self.b.append(k)
        self.length.append(self.length[u] * k)
        return idx

    def expand_bytes(self, node_id):
        out = bytearray()
        stack = [(node_id, 1)]
        while stack:
            nid, mult = stack.pop()
            k = self.kind[nid]
            if k == 0:
                if mult == 1:
                    out.append(self.a[nid])
                else:
                    out.extend(bytes([self.a[nid]]) * mult)
            elif k == 1:
                if mult == 1:
                    stack.append((self.b[nid], 1))
                    stack.append((self.a[nid], 1))
                else:
                    sub = self.expand_bytes(nid)
                    out.extend(sub * mult)
            else:
                stack.append((self.a[nid], self.b[nid] * mult))
        return bytes(out)


def combine_mark(m1, len1, m2):
    if m1 >= 0:
        assert m2 < 0
        return m1
    if m2 >= 0:
        return len1 + m2
    return -1


def build_initial_from_L(L_bytes, primary_idx):
    """Acquire W-IET and exact 256x256 bigram transition weights directly from I_0 in 1 pass."""
    n = len(L_bytes)
    if n == 0:
        return 0, [], {}, {}, SLPStore()

    runs = []
    counts = [0] * 256
    i = 0
    while i < n:
        c = L_bytes[i]
        j = i + 1
        while j < n and L_bytes[j] == c:
            j += 1
        runs.append((c, i, j))
        counts[c] += (j - i)
        i = j

    C = [0] * 257
    acc = 0
    buckets = []
    for c in range(256):
        C[c] = acc
        if counts[c] > 0:
            buckets.append((c, acc, acc + counts[c]))
        acc += counts[c]
    C[256] = acc

    slp = SLPStore()
    cur_t = C[:]
    pieces_by_bucket = {c: [] for c, _, _ in buckets}
    # Bigram weights: (f_sym, l_sym) -> total rows
    bigrams = defaultdict(int)

    b_idx = 0
    num_b = len(buckets)
    for c, r_lo, r_hi in runs:
        t_start = cur_t[c]
        cur_t[c] += (r_hi - r_lo)
        delta = t_start - r_lo
        pos = r_lo
        while pos < r_hi:
            while b_idx < num_b and buckets[b_idx][2] <= pos:
                b_idx += 1
            bc, b_lo, b_hi = buckets[b_idx]
            seg_hi = r_hi if r_hi < b_hi else b_hi
            bigrams[(bc, c)] += (seg_hi - pos)
            if pos <= primary_idx < seg_hi:
                if pos < primary_idx:
                    pieces_by_bucket[bc].append((pos, primary_idx, delta, c, c, -1))
                pieces_by_bucket[bc].append((primary_idx, primary_idx + 1, delta, c, c, 0))
                if primary_idx + 1 < seg_hi:
                    pieces_by_bucket[bc].append((primary_idx + 1, seg_hi, delta, c, c, -1))
            else:
                pieces_by_bucket[bc].append((pos, seg_hi, delta, c, c, -1))
            pos = seg_hi

    return n, buckets, pieces_by_bucket, bigrams, slp


def resolve_subset(sym_set, raw_pieces, slp, stats):
    """
    Compute exact Boundary Transfer Map Phi_{sym_set} from raw_pieces
    (each piece: (lo, hi, delta, target_sym, slp_id, mark_off)).
    Returns: (out_pieces, primary_cycle_or_None)
    where primary_cycle_or_None is (slp_id, mark_off) if the cycle containing
    the primary mark closes strictly inside sym_set!
    """
    # Step 1: Collapse immediate self-overlapping internal pieces (spiral division)
    collapsed = []
    primary_cycle = None
    primary_in_internal = None  # domain pos of marked piece if internal

    for lo, hi, d, tsym, w, m in raw_pieces:
        is_int = (tsym in sym_set)
        if not is_int:
            collapsed.append((lo, hi, d, tsym, w, m, False))
            continue
        if d == 0:
            stats["self_cycles"] += 1
            if m >= 0:
                primary_cycle = (w, m)
            continue
        L = hi - lo
        if d > 0 and d < L:
            assert m < 0
            stats["spirals"] += 1
            q, rem = divmod(L, d)
            if rem > 0:
                collapsed.append((lo, lo + rem, (q + 1) * d, tsym, slp.power(w, q + 1), -1, True))
            if d - rem > 0:
                collapsed.append((lo + rem, lo + d, q * d, tsym, slp.power(w, q), -1, True))
        elif d < 0 and (-d) < L:
            assert m < 0
            stats["spirals"] += 1
            s = -d
            q, rem = divmod(L, s)
            if s - rem > 0:
                collapsed.append((hi - s, hi - rem, -q * s, tsym, slp.power(w, q), -1, True))
            if rem > 0:
                collapsed.append((hi - rem, hi, -(q + 1) * s, tsym, slp.power(w, q + 1), -1, True))
        else:
            collapsed.append((lo, hi, d, tsym, w, m, True))

    collapsed.sort(key=lambda x: x[0])
    starts = [p[0] for p in collapsed]

    # Compute In(S) = domain(collapsed) \ target(internal pieces)
    int_targets = sorted((lo + d, hi + d) for lo, hi, d, tsym, w, m, is_int in collapsed if is_int)
    merged_t = []
    for a, b in int_targets:
        if merged_t and merged_t[-1][1] == a:
            merged_t[-1] = (merged_t[-1][0], b)
        else:
            merged_t.append((a, b))

    entry_work = []
    mt_idx = 0
    num_mt = len(merged_t)
    primary_piece_pos = None

    for lo, hi, d, tsym, w, m, is_int in collapsed:
        if m >= 0:
            primary_piece_pos = lo
        pos = lo
        while pos < hi:
            while mt_idx < num_mt and merged_t[mt_idx][1] <= pos:
                mt_idx += 1
            if mt_idx >= num_mt or merged_t[mt_idx][0] >= hi:
                entry_work.append((pos, hi, d, tsym, w, m, is_int))
                pos = hi
            else:
                ta, tb = merged_t[mt_idx]
                if pos < ta:
                    entry_work.append((pos, ta, d, tsym, w, m, is_int))
                    pos = ta
                pos = hi if hi < tb else tb

    out_pieces = []
    saw_primary_exit = False

    stack = list(reversed(entry_work))
    while stack:
        lo, hi, d, tsym, w, m, is_int = stack.pop()
        if not is_int:
            out_pieces.append((lo, hi, d, tsym, w, m))
            if m >= 0:
                saw_primary_exit = True
            continue
        stats["bounces"] += 1
        t_lo, t_hi = lo + d, hi + d
        idx = bisect_right(starts, t_lo) - 1
        pos = t_lo
        w_len = slp.length[w]
        nxt = []
        while pos < t_hi:
            stats["lookups"] += 1
            p_lo, p_hi, p_d, p_tsym, p_w, p_m, p_int = collapsed[idx]
            ov_hi = t_hi if t_hi < p_hi else p_hi
            sub_lo = pos - d
            sub_hi = ov_hi - d
            nxt.append((
                sub_lo, sub_hi, d + p_d, p_tsym,
                slp.concat(w, p_w),
                combine_mark(m, w_len, p_m),
                p_int
            ))
            stats["slp_joins"] += 1
            pos = ov_hi
            idx += 1
        for item in reversed(nxt):
            stack.append(item)

    # Did the primary mark close inside a cycle strictly within sym_set?
    if primary_piece_pos is not None and not saw_primary_exit and primary_cycle is None:
        # Trace the single point primary_piece_pos around its closed cycle in collapsed!
        # Wait: this takes EXACTLY the number of pieces along the cycle (at most cut_size)!
        x = primary_piece_pos
        idx = bisect_right(starts, x) - 1
        _, _, d0, _, w_acc, m_acc, is_int0 = collapsed[idx]
        assert is_int0 and m_acc >= 0
        x = x + d0
        steps = 1
        while x != primary_piece_pos:
            stats["root_cycle_steps"] += 1
            idx = bisect_right(starts, x) - 1
            p_lo, p_hi, p_d, _, p_w, p_m, p_int = collapsed[idx]
            assert p_lo <= x < p_hi and p_int
            w_acc = slp.concat(w_acc, p_w)
            stats["slp_joins"] += 1
            x = x + p_d
            steps += 1
        primary_cycle = (w_acc, m_acc)

    out_pieces.sort(key=lambda x: x[0])
    return out_pieces, primary_cycle


def decode_btm_v2(L_bytes, primary_idx, mode="balanced", verbose=False):
    n = len(L_bytes)
    if n == 0:
        return b"", {}

    n, buckets, pieces_by_bucket, bigrams, slp = build_initial_from_L(L_bytes, primary_idx)
    stats = defaultdict(int)
    stats["r0"] = sum(len(v) for v in pieces_by_bucket.values())
    stats["num_buckets"] = len(buckets)

    # Leaf level: compute Phi_{F_c} for each symbol c independently
    # Each node is (sym_frozenset, out_pieces, bucket_count)
    nodes = {}
    primary_cycle = None
    for c, b_lo, b_hi in buckets:
        s = frozenset([c])
        out_p, pcyc = resolve_subset(s, pieces_by_bucket[c], slp, stats)
        if pcyc is not None:
            primary_cycle = pcyc
        stats["leaf_pieces"] += len(out_p)
        nodes[c] = (s, out_p, b_hi - b_lo)

    level_log = []

    def do_merge(node1, node2, depth=0):
        nonlocal primary_cycle
        s1, p1, cnt1 = node1
        s2, p2, cnt2 = node2
        s = s1 | s2
        b0 = stats["bounces"]
        l0 = stats["lookups"]
        out_p, pcyc = resolve_subset(s, p1 + p2, slp, stats)
        if pcyc is not None:
            primary_cycle = pcyc
        level_log.append((depth, len(s), len(p1) + len(p2), len(out_p),
                          stats["bounces"] - b0, stats["lookups"] - l0))
        return (s, out_p, cnt1 + cnt2)

    if len(nodes) == 1:
        root = next(iter(nodes.values()))
    elif mode in ("balanced", "rarest_root", "freq_desc"):
        ordered = [nodes[c] for c, _, _ in buckets]
        if mode == "rarest_root":
            # Sort by count descending, pull rarest symbol to root, balanced on rest
            ordered.sort(key=lambda x: x[2], reverse=True)
            rarest = ordered.pop()
            def rec(arr, d=1):
                if len(arr) == 1:
                    return arr[0]
                mid = len(arr) // 2
                return do_merge(rec(arr[:mid], d + 1), rec(arr[mid:], d + 1), d)
            rest = rec(ordered, 1)
            root = do_merge(rest, rarest, 0)
        elif mode == "freq_desc":
            # Merge two most frequent? No, linear chain from most frequent to rarest,
            # or Huffman-like min-count? Let's test balanced sorted by freq!
            ordered.sort(key=lambda x: x[2], reverse=True)
            def rec(arr, d=1):
                if len(arr) == 1:
                    return arr[0]
                mid = len(arr) // 2
                return do_merge(rec(arr[:mid], d + 1), rec(arr[mid:], d + 1), d)
            root = rec(ordered, 0)
        else:
            def rec(arr, d=1):
                if len(arr) == 1:
                    return arr[0]
                mid = len(arr) // 2
                return do_merge(rec(arr[:mid], d + 1), rec(arr[mid:], d + 1), d)
            root = rec(ordered, 0)
    elif mode == "heavy_edge":
        # Greedy agglomerative merge: at each step merge the pair (u, v) maximizing
        # cross-bigram weight w(u <-> v) / (|p_u| + |p_v|), absorbing heaviest transitions early!
        active = {i: nodes[c] for i, (c, _, _) in enumerate(buckets)}
        # Build symmetric weight matrix between active cluster IDs
        sym_to_cid = {c: i for i, (c, _, _) in enumerate(buckets)}
        w_adj = defaultdict(lambda: defaultdict(int))
        for (a, b), wt in bigrams.items():
            if a != b:
                ca, cb = sym_to_cid[a], sym_to_cid[b]
                w_adj[ca][cb] += wt
                w_adj[cb][ca] += wt
        next_cid = len(active)
        step = 0
        while len(active) > 1:
            # Pick pair maximizing score = w(u, v) / (len(p_u) + len(p_v) + 1)
            # Wait: also prefer small clusters or pure max w / min_size (normalized cut)?
            best_pair = None
            best_score = -1.0
            cids = list(active.keys())
            for u in cids:
                nu = active[u]
                for v, wt in w_adj[u].items():
                    if v > u and v in active:
                        nv = active[v]
                        score = wt / (len(nu[1]) + len(nv[1]) + 1.0)
                        if score > best_score:
                            best_score = score
                            best_pair = (u, v)
            if best_pair is None:
                best_pair = (cids[0], cids[1])
            u, v = best_pair
            merged = do_merge(active.pop(u), active.pop(v), step)
            cid = next_cid
            next_cid += 1
            active[cid] = merged
            # Update w_adj
            new_row = defaultdict(int)
            for nb, wt in w_adj[u].items():
                if nb != v and nb in active:
                    new_row[nb] += wt
            for nb, wt in w_adj[v].items():
                if nb != u and nb in active:
                    new_row[nb] += wt
            del w_adj[u]
            del w_adj[v]
            for nb, wt in new_row.items():
                w_adj[cid][nb] = wt
                w_adj[nb][cid] = wt
                w_adj[nb].pop(u, None)
                w_adj[nb].pop(v, None)
            step += 1
        root = next(iter(active.values()))
    else:
        raise ValueError(mode)

    assert len(root[1]) == 0
    assert primary_cycle is not None

    c_slp, mark_off = primary_cycle
    cyc = slp.expand_bytes(c_slp)
    q = len(cyc)
    rot = cyc[mark_off:] + cyc[:mark_off]
    lf_seq = (rot * (n // q + 1))[:n]
    text = lf_seq[::-1]

    stats["slp_total_nodes"] = len(slp.kind) - 256
    stats["sum_tree_pieces"] = stats["leaf_pieces"] + sum(x[3] for x in level_log)
    stats["max_node_pieces"] = max((x[3] for x in level_log), default=stats["leaf_pieces"])
    if verbose:
        # Aggregate by depth
        by_d = defaultdict(lambda: [0, 0, 0, 0, 0])
        for d, ns, pin, pout, bn, lk in level_log:
            by_d[d][0] += 1
            by_d[d][1] += pin
            by_d[d][2] += pout
            by_d[d][3] += bn
            by_d[d][4] += lk
        for d in sorted(by_d):
            cnt, pin, pout, bn, lk = by_d[d]
            print(f"    depth={d:2d}: merges={cnt:3d} in_P={pin:7d} out_P={pout:7d} "
                  f"bounces={bn:7d} lookups={lk:7d}")
    return text, dict(stats)


def bwt_encode(text):
    n = len(text)
    rots = sorted(range(n), key=lambda i: text[i:] + text[:i])
    L = bytes(text[(i - 1) % n] for i in rots)
    return L, rots.index(0)


def ref_ibwt(L_bytes, p):
    n = len(L_bytes)
    if n == 0:
        return b""
    counts = [0] * 256
    for c in L_bytes:
        counts[c] += 1
    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += counts[c]
    LF = [0] * n
    cur = C[:]
    for i, c in enumerate(L_bytes):
        LF[i] = cur[c]
        cur[c] += 1
    out = bytearray(n)
    x = p
    for k in range(n - 1, -1, -1):
        out[k] = L_bytes[x]
        x = LF[x]
    return bytes(out)


def main():
    print("=== 1. Exhaustive Small Verification (v2 arbitrary subset star) ===")
    tot = 0
    for n in range(1, 9):
        for mask in range(1 << n):
            T = bytes((mask >> i) & 1 for i in range(n))
            L, p = bwt_encode(T)
            pts = range(n) if n <= 6 else [p]
            for pt in pts:
                ref = ref_ibwt(L, pt)
                dec, _ = decode_btm_v2(L, pt, mode="balanced")
                assert dec == ref
                tot += 1
    for n in range(1, 6):
        for code in range(3 ** n):
            x = code
            T = bytes((x // (3 ** i)) % 3 for i in range(n))
            L, p = bwt_encode(T)
            for pt in range(n):
                ref = ref_ibwt(L, pt)
                dec, _ = decode_btm_v2(L, pt, mode="rarest_root")
                assert dec == ref
                tot += 1
            # Also arbitrary L
            for pt in set([0, n // 2, n - 1]):
                ref = ref_ibwt(T, pt)
                dec, _ = decode_btm_v2(T, pt, mode="heavy_edge")
                assert dec == ref
                tot += 1
    print(f"  EXHAUSTIVE PASS: {tot} cases (binary n<=8, ternary n<=5, all primary marks, arb multi-cycle) [verified]")

    print("\n=== 2. Adversarial Witnesses ===")
    for K in [4, 16, 64, 256]:
        M_cnt = 64
        low_tags = [bytes([65 + (i // 8), 65 + (i % 8)]) for i in range(M_cnt)]
        high_tags = [bytes([83 + (i // 8), 83 + (i % 8)]) for i in range(M_cnt)]
        T = b"".join(low_tags[i] + b"M" * K + high_tags[i] for i in range(M_cnt))
        L, p = bwt_encode(T)
        dec, st = decode_btm_v2(L, p, mode="rarest_root")
        assert dec == T
        print(f"  Referee-SymMidRun K={K:3d} n={len(T):5d} r0={st['r0']:3d}: "
              f"bounces={st.get('bounces',0):4d} lookups={st.get('lookups',0):4d} "
              f"root_steps={st.get('root_cycle_steps',0):3d} slp={st['slp_total_nodes']:4d} spirals={st.get('spirals',0)}")

    for m in range(3, 10):
        a_f, b_f = [0], [0, 1]
        for _ in range(2 * m - 2):
            a_f, b_f = b_f, b_f + a_f
        T = bytes(b_f + [2])
        L, p = bwt_encode(T)
        dec, st = decode_btm_v2(L, p, mode="rarest_root")
        assert dec == T
        print(f"  Fib_{2*m:2d}+$ n={len(T):5d} r0={st['r0']:2d}: "
              f"bounces={st.get('bounces',0):4d} lookups={st.get('lookups',0):4d} "
              f"root_steps={st.get('root_cycle_steps',0):2d} slp={st['slp_total_nodes']:4d} spirals={st.get('spirals',0)}")

    print("\n=== 3. Wikidata block-0.L (and all 6 blocks) ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    for idx, entry in enumerate(manifest["blocks"]):
        path = f"/tmp/mibz2-blocks/{entry['file']}"
        with open(path, "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        p = entry["primary_index"]
        n = entry["n"]
        ref = ref_ibwt(L, p)

        modes = ["balanced", "rarest_root", "heavy_edge"] if idx == 0 else ["rarest_root", "heavy_edge"]
        for mode in modes:
            t0 = time.perf_counter()
            dec, st = decode_btm_v2(L, p, mode=mode, verbose=(idx == 0 and mode == "balanced"))
            dt = time.perf_counter() - t0
            assert dec == ref, f"MISMATCH on {entry['file']} mode={mode}"
            r0 = st["r0"]
            lk = st.get("lookups", 0)
            bn = st.get("bounces", 0)
            rs = st.get("root_cycle_steps", 0)
            print(f"  {entry['file']:11s} [{mode:11s}] r0={r0:6d} | "
                  f"bounces={bn:8d} ({bn/r0:5.2f}r) lookups={lk:8d} ({lk/r0:5.2f}r) "
                  f"root={rs:6d} slp={st['slp_total_nodes']:8d} "
                  f"sum_P={st['sum_tree_pieces']:7d} ({st['sum_tree_pieces']/r0:4.2f}r) "
                  f"t={dt:.2f}s [EXACT]", flush=True)


if __name__ == "__main__":
    main()
