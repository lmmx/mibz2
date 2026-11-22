#!/usr/bin/env python3
"""
Cycle 3 — Task 1(a) & 1(b):
Exact Gap-Closed Coalescing + Hash-Consed W-IET Contraction (Myhill-Nerode / Quotient Test)
on:
  1. Exhaustive small BWTs (binary n<=8, ternary n<=5) — verifies exactness & searches for
     smallest Q_comp / r_0 ratio under free gap-closing coalescence + free hash-consing!
  2. All 6 authenticated Wikidata blocks (`/tmp/mibz2-blocks/`) under:
     - Gap-Closed Coalescing W-IET Polarity-DAG (compressing coordinates to [0, cur_m) each round
       and coalescing adjacent pieces with identical (compressed_delta, hashcons_word_id)!)
     - Gap-Closed Coalescing Bucket-Schur (eliminating buckets in leverage order, closing gaps
       and coalescing identical adjacent (compressed_delta, hashcons_word_id) pieces!)
  Measures:
     - Q_comp (total piece compositions) vs r_0 vs z(T) = 0.378 r_0
     - Exact number of coalescences (gap-triggered vs composition-triggered)
     - Hash-consed distinct SLP rules |G_dedup| vs r_0 vs z(T)
     - Exact empirical verification of Theorem C3 (Q_comp >= r_0 / 4, and checking tighter 1/2!)
"""
import hashlib
import json
import sys
import time
from bisect import bisect_right
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import measure_all_trees as mt


class HashConsSLP:
    """Exact hash-consed RLSLP: identical (left, right) or (base, exp) share the EXACT same ID!"""
    __slots__ = ("kind", "a", "b", "length", "concat_tbl", "pow_tbl", "dedup_hits", "raw_joins")

    def __init__(self):
        self.kind = [0] * 256
        self.a = list(range(256))
        self.b = [0] * 256
        self.length = [1] * 256
        self.concat_tbl = {}
        self.pow_tbl = {}
        self.dedup_hits = 0
        self.raw_joins = 0

    def concat(self, u, v):
        self.raw_joins += 1
        key = (u, v)
        got = self.concat_tbl.get(key)
        if got is not None:
            self.dedup_hits += 1
            return got
        idx = len(self.kind)
        self.kind.append(1)
        self.a.append(u)
        self.b.append(v)
        self.length.append(self.length[u] + self.length[v])
        self.concat_tbl[key] = idx
        return idx

    def power(self, u, k):
        if k == 1:
            return u
        key = (u, k)
        got = self.pow_tbl.get(key)
        if got is not None:
            self.dedup_hits += 1
            return got
        idx = len(self.kind)
        self.kind.append(2)
        self.a.append(u)
        self.b.append(k)
        self.length.append(self.length[u] * k)
        self.pow_tbl[key] = idx
        return idx

    def expand(self, nid):
        out = bytearray()
        st = [(nid, 1)]
        while st:
            u, mult = st.pop()
            k = self.kind[u]
            if k == 0:
                out.extend(bytes([self.a[u]]) * mult)
            elif k == 1:
                if mult == 1:
                    st.append((self.b[u], 1))
                    st.append((self.a[u], 1))
                else:
                    out.extend(self.expand(u) * mult)
            else:
                st.append((self.a[u], self.b[u] * mult))
        return bytes(out)


def compact_and_coalesce(pieces):
    """
    Close ALL deleted gaps by re-indexing the live domain U_t densely into [0, cur_m)
    in strictly increasing coordinate order, recomputing compressed targets/deltas,
    and COALESCING any adjacent pieces with identical (compressed_delta, slp_id)
    (never coalescing across a primary mark!).

    Wait: since U_t is a union of sorted disjoint intervals [lo_i, hi_i), mapping
    old coordinate x in [lo_i, hi_i) -> new_lo_i + (x - lo_i) is a STRICTLY INCREASING
    bijection phi: U_t -> [0, cur_m)!
    IMPORTANT: Because targets may span MULTIPLE old intervals (which were separated by
    a deleted gap, now closed!), a single piece's target [t_lo, t_hi) in old coords
    lies completely inside U_t — wait: could [t_lo, t_hi) contain a deleted gap?!
    NO! Because f_t([lo, hi)) = [t_lo, t_hi) is a CONTIGUOUS interval of U_t (every point
    of [t_lo, t_hi) is in U_t, since it's the image of a piece in the bijection f_t: U_t -> U_t)!
    Therefore phi maps [t_lo, t_hi) to a SINGLE contiguous interval [phi(t_lo), phi(t_lo) + (hi - lo))!
    """
    if not pieces:
        return [], 0
    # Build prefix map old_lo -> new_lo
    old_los = []
    new_los = []
    acc = 0
    for lo, hi, d, w, m in pieces:
        old_los.append(lo)
        new_los.append(acc)
        acc += (hi - lo)

    def map_pos(x):
        idx = bisect_right(old_los, x) - 1
        return new_los[idx] + (x - old_los[idx])

    comp = []
    coalesced = 0
    for i, (lo, hi, d, w, m) in enumerate(pieces):
        nlo = new_los[i]
        nhi = nlo + (hi - lo)
        nt = map_pos(lo + d)
        nd = nt - nlo
        # Can we coalesce with comp[-1]?
        if comp and comp[-1][1] == nlo and comp[-1][2] == nd and comp[-1][3] == w and comp[-1][4] < 0 and m < 0:
            prev = comp[-1]
            comp[-1] = (prev[0], nhi, nd, w, -1)
            coalesced += 1
        else:
            comp.append((nlo, nhi, nd, w, m))
    return comp, coalesced


def gap_closed_polarity_dag(L_bytes, primary_idx, use_coalesce=True):
    """
    Gap-Closed Coalescing + Hash-Consed W-IET Polarity-DAG!
    Every round:
      1. Spiral-compress self-overlaps in dense [0, cur_m) coordinates.
      2. Eliminate majority direction (delta < 0 vs delta > 0) in 1 acyclic shot.
      3. Compact coordinates (close all deleted gaps!) and COALESCE adjacent pieces
         with identical (compressed_delta, hashcons_word_id)!
    """
    n = len(L_bytes)
    if n == 0:
        return b"", {}

    # Build initial maximal runs directly (NOT bucket-split!)
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
    r0 = len(runs)

    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += counts[c]
    cur_t = C[:]

    pieces = []
    for c, lo, hi in runs:
        d = cur_t[c] - lo
        cur_t[c] += (hi - lo)
        if lo <= primary_idx < hi:
            if lo < primary_idx:
                pieces.append((lo, primary_idx, d, c, -1))
            pieces.append((primary_idx, primary_idx + 1, d, c, 0))
            if primary_idx + 1 < hi:
                pieces.append((primary_idx + 1, hi, d, c, -1))
        else:
            pieces.append((lo, hi, d, c, -1))

    slp = HashConsSLP()
    stats = defaultdict(int)
    stats["r0"] = r0
    primary_cycle = None
    rounds = 0

    while primary_cycle is None:
        # Step 0: Spiral compress
        step0 = []
        did_sp = False
        w_down = 0
        w_up = 0
        for lo, hi, d, w, m in pieces:
            if d == 0:
                if m >= 0:
                    primary_cycle = (w, m)
                stats["self_cycles"] += 1
                continue
            L = hi - lo
            if d > 0 and d < L:
                assert m < 0
                q, rem = divmod(L, d)
                stats["spirals"] += 1
                did_sp = True
                if rem > 0:
                    step0.append((lo, lo + rem, (q + 1) * d, slp.power(w, q + 1), -1))
                if d - rem > 0:
                    step0.append((lo + rem, lo + d, q * d, slp.power(w, q), -1))
            elif d < 0 and (-d) < L:
                assert m < 0
                s = -d
                q, rem = divmod(L, s)
                stats["spirals"] += 1
                did_sp = True
                if s - rem > 0:
                    step0.append((hi - s, hi - rem, -q * s, slp.power(w, q), -1))
                if rem > 0:
                    step0.append((hi - rem, hi, -(q + 1) * s, slp.power(w, q + 1), -1))
            else:
                if d < 0:
                    w_down += L
                else:
                    w_up += L
                step0.append((lo, hi, d, w, m))

        if primary_cycle is not None:
            break
        if did_sp:
            step0.sort(key=lambda x: x[0])
            if use_coalesce:
                pieces, c_cnt = compact_and_coalesce(step0)
                stats["coalesced"] += c_cnt
            else:
                pieces = step0
            continue

        rounds += 1
        del_down = (w_down >= w_up)
        tagged = [(lo, hi, d, w, m, (d < 0) if del_down else (d > 0)) for lo, hi, d, w, m in step0]
        starts = [p[0] for p in tagged]

        # Wavefront / stack resolution of retained pieces through deleted DAG
        next_pieces = []
        stack = []
        for lo, hi, d, w, m, is_del in tagged:
            if not is_del:
                stack.append((lo, hi, d, w, m))

        while stack:
            lo, hi, d, w, m = stack.pop()
            t_lo, t_hi = lo + d, hi + d
            idx = bisect_right(starts, t_lo) - 1
            pos = t_lo
            w_len = slp.length[w]
            while pos < t_hi:
                p_lo, p_hi, p_d, p_w, p_m, p_del = tagged[idx]
                ov = t_hi if t_hi < p_hi else p_hi
                slo, shi = pos - d, ov - d
                if not p_del:
                    next_pieces.append((slo, shi, d, w, m))
                else:
                    stats["Q_comp"] += 1
                    nw = slp.concat(w, p_w)
                    nm = mt.combine_mark(m, w_len, p_m)
                    stack.append((slo, shi, d + p_d, nw, nm))
                pos = ov
                idx += 1

        next_pieces.sort(key=lambda x: x[0])
        if use_coalesce:
            pieces, c_cnt = compact_and_coalesce(next_pieces)
            stats["coalesced"] += c_cnt
            if rounds <= 5:
                stats[f"P_after_r{rounds}"] = len(pieces)
        else:
            pieces = next_pieces

    c_slp, mark_off = primary_cycle
    cyc = slp.expand(c_slp)
    q = len(cyc)
    rot = cyc[mark_off:] + cyc[:mark_off]
    text = ((rot * (n // q + 1))[:n])[::-1]

    # Count distinct reachable hash-consed SLP nodes from root!
    vis = set()
    st = [c_slp]
    while st:
        u = st.pop()
        if u in vis:
            continue
        vis.add(u)
        if slp.kind[u] == 1:
            st.append(slp.a[u]); st.append(slp.b[u])
        elif slp.kind[u] == 2:
            st.append(slp.a[u])

    stats["rounds"] = rounds
    stats["G_created_dedup"] = len(slp.kind) - 256
    stats["G_reach_dedup"] = sum(1 for u in vis if slp.kind[u] != 0)
    stats["dedup_hits"] = slp.dedup_hits
    stats["raw_joins"] = slp.raw_joins
    return text, dict(stats)


def main():
    print("=== 1. Exhaustive Small Check of Gap-Closed Coalescing + Hash-Consed W-IET ===")
    min_q_ratio = 999.0
    min_g_ratio = 999.0
    q_below_r0 = 0
    g_below_r0 = 0
    tot = 0
    min_q_wit = None
    min_g_wit = None

    for base, max_n in [(2, 8), (3, 5)]:
        for n in range(2, max_n + 1):
            for code in range(base ** n):
                T = bytes((code // (base ** i)) % base for i in range(n))
                L, p = mt.bwt_encode(T)
                dec, st = gap_closed_polarity_dag(L, p, use_coalesce=True)
                assert dec == T, f"EXACTNESS FAIL on {T}"
                r0 = st["r0"]
                if r0 >= 2:
                    tot += 1
                    qtot = st.get("Q_comp", 0) + st.get("spirals", 0)
                    gd = st["G_reach_dedup"]
                    if qtot < r0:
                        q_below_r0 += 1
                    if gd < r0:
                        g_below_r0 += 1
                    if qtot / r0 < min_q_ratio:
                        min_q_ratio = qtot / r0
                        min_q_wit = (list(T), r0, qtot, st.get("coalesced", 0), gd)
                    if gd / r0 < min_g_ratio:
                        min_g_ratio = gd / r0
                        min_g_wit = (list(T), r0, qtot, st.get("coalesced", 0), gd)
    print(f"  EXHAUSTIVE PASS: {tot} non-trivial binary(n<=8)+ternary(n<=5) BWTs [verified]")
    print(f"    (Q_comp+spirals) < r_0: {q_below_r0}/{tot} | min ratio = {min_q_ratio:.3f} witness={min_q_wit}")
    print(f"    G_reach_dedup    < r_0: {g_below_r0}/{tot} | min ratio = {min_g_ratio:.3f} witness={min_g_wit}")

    print("\n=== 2. ALL 6 AUTHENTICATED WIKIDATA BLOCKS: Gap-Closed Coalescence + Hash-Consing vs z(T) ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    zs = {"block-0.L": 20083, "block-2.L": 21950, "block-85.L": 30001,
          "block-143.L": 40664, "block-210.L": 46451, "block-207.L": 33608}

    for entry in manifest["blocks"]:
        with open(f"/tmp/mibz2-blocks/{entry['file']}", "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        p = entry["primary_index"]
        ref = mt.ref_ibwt(L, p)

        dec_no, st_no = gap_closed_polarity_dag(L, p, use_coalesce=False)
        dec_co, st_co = gap_closed_polarity_dag(L, p, use_coalesce=True)
        assert dec_no == ref and dec_co == ref
        r0 = st_co["r0"]
        z = zs[entry["file"]]
        print(f"  {entry['file']:11s} r0={r0:6d} z={z:5d}({z/r0:.3f}r) | "
              f"NoCoal: Q={st_no['Q_comp']:6d}({st_no['Q_comp']/r0:4.2f}r) G_dedup={st_no['G_reach_dedup']:6d}({st_no['G_reach_dedup']/r0:.3f}r) | "
              f"GapCoal: Q={st_co['Q_comp']:6d}({st_co['Q_comp']/r0:4.2f}r) coal={st_co.get('coalesced',0):5d} "
              f"G_dedup={st_co['G_reach_dedup']:6d}({st_co['G_reach_dedup']/r0:.3f}r, {st_co['G_reach_dedup']/z:.2f}z) "
              f"dedup_hits={st_co['dedup_hits']:6d} [EXACT]", flush=True)


if __name__ == "__main__":
    main()
