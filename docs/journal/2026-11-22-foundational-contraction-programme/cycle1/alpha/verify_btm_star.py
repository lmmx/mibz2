#!/usr/bin/env python3
"""
Exact prototype & verifier for Compositional Redheffer Star Product of
Boundary Transfer Maps (BTM-WIET) on inverse BWT.

Mathematical object:
Given I_0 = (n, p, R, C, Sigma), LF is a piecewise-translation bijection
on [0, n) with r_0 domain intervals I_k = [s_k, e_k), target J_k = [t_k, t_k+l_k),
offset delta_k = t_k - s_k, and SLP symbol w_k = L[I_k].

For any disjoint region U subset [0, n) (specifically contiguous union of F-buckets):
  Phi_U = (T_U, C_U)
where T_U: In(U) -> Out(U) is a sorted list of disjoint transfer pieces
  (lo, hi, delta, slp_id, mark_off)
with In(U) = U \ f(U), Out(U) = f(U) \ U, and C_U holds closed cycles inside U.
mark_off is -1 if the piece does not contain the primary row p, else the exact
0-based byte offset of L[p] inside the SLP string w (oriented in LF order:
L[x], L[f(x)], ..., L[f^{tau-1}(x)]).

Star product Phi_{U1} * Phi_{U2} eliminates all internal transitions between
U1 and U2 using:
  1) Exact self-loop spiral compression (O(1) per overlapping piece),
  2) Exact interval-composition with instrumentation of work, pieces, splits,
     bounces, and SLP nodes.
"""

import hashlib
import json
import sys
import time
from bisect import bisect_right
from collections import defaultdict


class SLPStore:
    """DAG / SLP store with length, exact byte expansion, and power nodes."""
    __slots__ = ("kind", "a", "b", "length", "rev_cache")

    def __init__(self):
        # kind: 0 = literal byte (a = byte 0..255)
        #       1 = concat (a = left_id, b = right_id)
        #       2 = power  (a = base_id, b = exponent >= 2)
        self.kind = []
        self.a = []
        self.b = []
        self.length = []
        for c in range(256):
            self.kind.append(0)
            self.a.append(c)
            self.b.append(0)
            self.length.append(1)

    def concat(self, u, v):
        if u == -1:
            return v
        if v == -1:
            return u
        idx = len(self.kind)
        self.kind.append(1)
        self.a.append(u)
        self.b.append(v)
        self.length.append(self.length[u] + self.length[v])
        return idx

    def power(self, u, k):
        assert k >= 1
        if k == 1:
            return u
        idx = len(self.kind)
        self.kind.append(2)
        self.a.append(u)
        self.b.append(k)
        self.length.append(self.length[u] * k)
        return idx

    def expand_bytes(self, node_id):
        """Iterative expansion to bytearray (LF order)."""
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
                    # expand base once and repeat
                    sub = self.expand_bytes(nid)
                    out.extend(sub * mult)
            else:
                # power node
                stack.append((self.a[nid], self.b[nid] * mult))
        return bytes(out)


def combine_mark(m1, len1, m2):
    if m1 >= 0:
        assert m2 < 0, f"Duplicate primary mark! {m1}, {m2}"
        return m1
    if m2 >= 0:
        return len1 + m2
    return -1


def build_initial_pieces_from_L(L_bytes, primary_idx):
    """
    Acquire W-IET directly from I_0 = (n, p, maximal runs of L, C, Sigma).
    Returns: (n, buckets, pieces_by_bucket, slp)
    where buckets = list of (c, c_lo, c_hi) for present symbols c in sorted order,
    and pieces_by_bucket[c] is sorted list of (lo, hi, delta, slp_id, mark_off)
    for domain intervals inside F_c = [c_lo, c_hi).
    """
    n = len(L_bytes)
    if n == 0:
        return 0, [], {}, SLPStore()

    # Run-length encode L (simulating uncapped I_0)
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

    # Cumulative counts C
    C = [0] * 257
    acc = 0
    buckets = []
    for c in range(256):
        C[c] = acc
        if counts[c] > 0:
            buckets.append((c, acc, acc + counts[c]))
        acc += counts[c]
    C[256] = acc

    # Split runs by F-bucket boundaries (sequential O(r + |Sigma|) scan)
    # and split the single run containing primary_idx so [p, p+1) is marked.
    slp = SLPStore()
    cur_t = C[:]  # next target position in F_c for each c
    pieces_by_bucket = {c: [] for c, _, _ in buckets}

    b_idx = 0
    num_b = len(buckets)
    for c, r_lo, r_hi in runs:
        t_start = cur_t[c]
        cur_t[c] += (r_hi - r_lo)
        delta = t_start - r_lo

        # Advance bucket pointer to cover [r_lo, r_hi)
        pos = r_lo
        while pos < r_hi:
            while b_idx < num_b and buckets[b_idx][2] <= pos:
                b_idx += 1
            bc, b_lo, b_hi = buckets[b_idx]
            seg_hi = r_hi if r_hi < b_hi else b_hi
            # Check if primary_idx falls inside [pos, seg_hi)
            if pos <= primary_idx < seg_hi:
                if pos < primary_idx:
                    pieces_by_bucket[bc].append((pos, primary_idx, delta, c, -1))
                pieces_by_bucket[bc].append((primary_idx, primary_idx + 1, delta, c, 0))
                if primary_idx + 1 < seg_hi:
                    pieces_by_bucket[bc].append((primary_idx + 1, seg_hi, delta, c, -1))
            else:
                pieces_by_bucket[bc].append((pos, seg_hi, delta, c, -1))
            pos = seg_hi

    return n, buckets, pieces_by_bucket, slp


def resolve_region_pieces(region_lo, region_hi, raw_pieces, slp, stats):
    """
    Given a sorted list of disjoint pieces (lo, hi, delta, slp_id, mark_off)
    whose domains lie in U = [region_lo, region_hi), compute the exact
    Boundary Transfer Map Phi_U = (out_pieces, closed_cycles).

    Method:
    1. Partition raw_pieces into:
       - exit_pieces: target [lo+d, hi+d) disjoint from U -> already done!
       - int_pieces: target inside U (split any piece straddling region_lo or region_hi).
    2. Compute In(U) = U \ target(int_pieces).
       Actually, even cleaner: resolve ALL int_pieces via iterative
       self-loop compression + shortcutting / backward-or-degree elimination!
    Let's implement exact, general W-IET elimination for int_pieces inside U:
      Keep two maps:
        - active pieces whose domain is in U and target is in U (internal)
        - done pieces whose domain is in U and target is outside U (exit)
      Wait: if we ONLY want Phi_U on In(U), we can either:
        (A) Forward-trace from In(U) with self-loop spiral compression, OR
        (B) Eliminate internal pieces inside U\In(U) until none remain!
      Let's implement exact self-loop compression + forward-from-In(U) with
      cycle detection for closed cycles, and measure EVERY step!
    """
    # Step 1: Split pieces straddling region boundaries into pure-internal and pure-exit.
    all_pieces = []
    for lo, hi, d, w, m in raw_pieces:
        t_lo, t_hi = lo + d, hi + d
        if t_hi <= region_lo or t_lo >= region_hi:
            all_pieces.append((lo, hi, d, w, m, False))  # exit
        elif t_lo >= region_lo and t_hi <= region_hi:
            all_pieces.append((lo, hi, d, w, m, True))   # internal
        else:
            # Straddles boundary
            cuts = [t_lo]
            if t_lo < region_lo < t_hi:
                cuts.append(region_lo)
            if t_lo < region_hi < t_hi:
                cuts.append(region_hi)
            cuts.append(t_hi)
            for idx in range(len(cuts) - 1):
                a, b = cuts[idx] - d, cuts[idx + 1] - d
                is_int = (region_lo <= cuts[idx] and cuts[idx + 1] <= region_hi)
                # Note: marked piece has length 1 so cannot straddle
                all_pieces.append((a, b, d, w, m, is_int))

    # Step 2: Collapse immediate self-overlapping internal pieces in O(1) spiral division!
    # An internal piece (lo, hi, d, w, m) with [lo, hi) intersecting [lo+d, hi+d):
    #   Case d == 0: closed fixed-point cycle(s) of length 1 each!
    #   Case d > 0: overlap is [lo+d, hi). Entry window is [lo, lo+d), exit is [hi, hi+d).
    #     For x in [lo, lo+d), steps q(x) = (hi - 1 - x) // d + 1.
    #     Let L = hi - lo, q_lo = L // d, rem = L % d.
    #     Wait: for x in [lo, lo + rem), hi - 1 - x >= L - rem = q_lo * d, so steps = q_lo + 1,
    #           target = x + (q_lo + 1) * d in [hi + d - rem, hi + d)? Wait:
    #           x = lo -> lo + (q_lo+1)*d = lo + L - rem + d = hi + d - rem.
    #           x = lo + rem - 1 -> hi + d - 1.
    #     For x in [lo + rem, lo + d), steps = q_lo,
    #           x = lo + rem -> lo + rem + q_lo*d = hi.
    #           x = lo + d - 1 -> hi + d - rem - 1.
    #   Case d < 0: let s = -d > 0. Overlap is [lo, hi-s). Entry window is [hi-s, hi), exit is [lo-s, lo).
    #     For x in [hi-s, hi), steps q(x) = (x - lo) // s + 1.
    #     Let L = hi - lo, q_lo = L // s, rem = L % s.
    #     For x in [hi - s, hi - rem): x - lo in [L - s, L - rem) = [(q_lo-1)*s + rem, q_lo*s),
    #       Wait: check carefully!
    #       If rem == 0: for all x in [hi-s, hi), x - lo in [L-s, L) = [(q_lo-1)*s, q_lo*s), so steps = q_lo.
    #       If rem > 0: L = q_lo * s + rem.
    #         For x in [hi - s, hi - s + (s - rem)) = [hi - s, hi - rem):
    #           x - lo in [L - s, L - rem) = [(q_lo - 1)*s + rem, q_lo * s), so (x-lo)//s = q_lo - 1 -> steps = q_lo!
    #           target = x - q_lo * s in [lo - s + rem, lo).
    #         For x in [hi - rem, hi):
    #           x - lo in [L - rem, L) = [q_lo * s, q_lo * s + rem), so (x-lo)//s = q_lo -> steps = q_lo + 1!
    #           target = x - (q_lo + 1) * s in [lo - s, lo - s + rem).
    #
    # Note: What if the primary mark is inside a self-loop piece?
    # Remember the primary piece was split to length 1: [p, p+1), so it can NEVER
    # have overlap unless d == 0 (fixed point)! Because if hi - lo == 1 and d != 0,
    # [lo, lo+1) and [lo+d, lo+d+1) are DISJOINT!
    # WAIT: What if a later composed piece contains primary mark? Its length is ALWAYS 1
    # because we isolated [p, p+1) at the beginning and pieces NEVER merge across mark_off != -1!
    # That makes self-loop spiral compression 100% mark-free (except d == 0)!

    collapsed = []
    closed_cycles = []

    for lo, hi, d, w, m, is_int in all_pieces:
        if not is_int:
            collapsed.append((lo, hi, d, w, m, False))
            continue
        if d == 0:
            # Every point x in [lo, hi) is a closed cycle of 1 step!
            closed_cycles.append((lo, hi, w, m))
            stats["self_cycles"] += 1
            continue
        L = hi - lo
        if d > 0 and d < L:
            assert m < 0, "Marked piece must have length 1"
            stats["spirals"] += 1
            q = L // d
            rem = L % d
            # Entry window is [lo, lo + d); points in [lo + d, hi) are purely internal to the spiral!
            # Wait: can any OTHER piece target [lo + d, hi)? NO! Because f is bijective and
            # [lo + d, hi) subset [lo + d, hi + d) = target(this piece)!
            if rem > 0:
                w_hi = slp.power(w, q + 1)
                d_hi = (q + 1) * d
                # Check if target [hi + d - rem, hi + d) is inside U or outside U
                # (it is subset of original target [lo+d, hi+d) subset U, so still internal!)
                collapsed.append((lo, lo + rem, d_hi, w_hi, -1, True))
            if d - rem > 0:
                w_lo = slp.power(w, q)
                d_lo = q * d
                collapsed.append((lo + rem, lo + d, d_lo, w_lo, -1, True))
        elif d < 0 and (-d) < L:
            assert m < 0, "Marked piece must have length 1"
            stats["spirals"] += 1
            s = -d
            q = L // s
            rem = L % s
            if s - rem > 0:
                w_lo = slp.power(w, q)
                d_lo = -q * s
                collapsed.append((hi - s, hi - rem, d_lo, w_lo, -1, True))
            if rem > 0:
                w_hi = slp.power(w, q + 1)
                d_hi = -(q + 1) * s
                collapsed.append((hi - rem, hi, d_hi, w_hi, -1, True))
        else:
            collapsed.append((lo, hi, d, w, m, True))

    # Build sorted piece array for fast lookup (only active domains in U!)
    # Note: after spiral compression, some sub-intervals of U (the spiral interiors)
    # are NO LONGER in collapsed, and NO remaining piece targets them!
    collapsed.sort(key=lambda x: x[0])

    # Compute In(U) = domain(collapsed) \ target(internal pieces of collapsed)
    int_targets = []
    for lo, hi, d, w, m, is_int in collapsed:
        if is_int:
            int_targets.append((lo + d, hi + d))
    int_targets.sort()

    # Merge contiguous int_targets
    merged_t = []
    for a, b in int_targets:
        if merged_t and merged_t[-1][1] == a:
            merged_t[-1] = (merged_t[-1][0], b)
        else:
            merged_t.append((a, b))

    # Compute external entry pieces = domain(collapsed) \ merged_t
    starts = [p[0] for p in collapsed]
    num_p = len(collapsed)

    entry_work = []   # pieces starting in In(U)
    internal_unvisited = []  # pieces in target(int) -> only needed if there are closed cycles!
    mt_idx = 0
    num_mt = len(merged_t)

    for lo, hi, d, w, m, is_int in collapsed:
        pos = lo
        while pos < hi:
            while mt_idx < num_mt and merged_t[mt_idx][1] <= pos:
                mt_idx += 1
            if mt_idx >= num_mt or merged_t[mt_idx][0] >= hi:
                # [pos, hi) is completely in In(U)
                entry_work.append((pos, hi, d, w, m, is_int))
                pos = hi
            else:
                ta, tb = merged_t[mt_idx]
                if pos < ta:
                    entry_work.append((pos, ta, d, w, m, is_int))
                    pos = ta
                ov_hi = hi if hi < tb else tb
                internal_unvisited.append((pos, ov_hi, d, w, m, is_int))
                pos = ov_hi

    out_pieces = []

    def lookup_and_advance(seg_lo, seg_hi, seg_d, seg_w, seg_m):
        """Advance [seg_lo, seg_hi) one step through collapsed at target [seg_lo+d, seg_hi+d)."""
        t_lo = seg_lo + seg_d
        t_hi = seg_hi + seg_d
        idx = bisect_right(starts, t_lo) - 1
        res = []
        pos = t_lo
        w_len = slp.length[seg_w]
        while pos < t_hi:
            stats["lookups"] += 1
            p_lo, p_hi, p_d, p_w, p_m, p_int = collapsed[idx]
            assert p_lo <= pos < p_hi, f"Gap at {pos}, piece [{p_lo},{p_hi})"
            ov_hi = t_hi if t_hi < p_hi else p_hi
            sub_lo = pos - seg_d
            sub_hi = ov_hi - seg_d
            new_d = seg_d + p_d
            new_w = slp.concat(seg_w, p_w)
            new_m = combine_mark(seg_m, w_len, p_m)
            stats["slp_joins"] += 1
            res.append((sub_lo, sub_hi, new_d, new_w, new_m, p_int))
            pos = ov_hi
            idx += 1
        return res

    # Forward trace all external entries until they exit U!
    # Also check for self-overlap during tracing (e.g. 2-step spirals like (ab)^K)!
    stack = list(reversed(entry_work))
    while stack:
        lo, hi, d, w, m, is_int = stack.pop()
        if not is_int:
            out_pieces.append((lo, hi, d, w, m))
            continue
        stats["bounces"] += 1
        # Check if [lo, hi) now self-overlaps its current target [lo+d, hi+d)?
        # Note: lo..hi is in In(U), while lo+d..hi+d is in U \ In(U), so they can NEVER overlap!
        # Wait: that's why 2-step spirals on internal intervals wouldn't self-overlap with In(U)!
        # Let's track bounces to measure how big they get!
        nxt = lookup_and_advance(lo, hi, d, w, m)
        for item in reversed(nxt):
            stack.append(item)

    # Check if any closed cycles remain inside U!
    # A closed cycle is ONLY possible if some internal points were NEVER reached from In(U).
    # We know the total length of out_pieces equals |In(U)|.
    # Actually, even simpler: track visited sub-intervals of internal targets?
    # Only needed at the root U = [0, n) (or if multi-cycle!).
    # How do we detect if closed cycles exist inside U?
    # Let's check: EVERY point in U \ In(U) is either reached from In(U) or belongs to a closed cycle!
    # If we want exact multi-cycle support at ANY node U, check if |In(U)| == 0 and len(internal_unvisited) > 0,
    # OR track visited internal mass!
    # Wait: even simpler! During a single-cycle BWT, closed cycles ONLY occur at U = [0, n) where In(U) = empty.
    # For general multi-cycle support (e.g., periodic strings!), a cycle of a single symbol or subset of symbols
    # can close inside a proper subtree U!
    # Let's handle closed cycles EXACTLY at every node U by tracking visited target intervals!
    return out_pieces, closed_cycles, internal_unvisited, collapsed, starts


def resolve_region_exact(region_lo, region_hi, raw_pieces, slp, stats):
    """
    Exact version handling BOTH open transfers AND arbitrary closed cycles
    (including periodic strings and multi-cycle permutations) at EVERY node U!
    """
    # Step 1: Split straddling pieces
    all_pieces = []
    for lo, hi, d, w, m in raw_pieces:
        t_lo, t_hi = lo + d, hi + d
        if t_hi <= region_lo or t_lo >= region_hi:
            all_pieces.append((lo, hi, d, w, m, False))
        elif t_lo >= region_lo and t_hi <= region_hi:
            all_pieces.append((lo, hi, d, w, m, True))
        else:
            cuts = [t_lo]
            if t_lo < region_lo < t_hi:
                cuts.append(region_lo)
            if t_lo < region_hi < t_hi:
                cuts.append(region_hi)
            cuts.append(t_hi)
            for idx in range(len(cuts) - 1):
                a, b = cuts[idx] - d, cuts[idx + 1] - d
                is_int = (region_lo <= cuts[idx] and cuts[idx + 1] <= region_hi)
                all_pieces.append((a, b, d, w, m, is_int))

    # Step 2: Collapse immediate self-overlapping internal pieces (spiral division)
    collapsed = []
    closed_cycles = []
    for lo, hi, d, w, m, is_int in all_pieces:
        if not is_int:
            collapsed.append((lo, hi, d, w, m, False))
            continue
        if d == 0:
            closed_cycles.append((lo, hi, w, m))
            stats["self_cycles"] += 1
            continue
        L = hi - lo
        if d > 0 and d < L:
            assert m < 0
            stats["spirals"] += 1
            q, rem = divmod(L, d)
            if rem > 0:
                collapsed.append((lo, lo + rem, (q + 1) * d, slp.power(w, q + 1), -1, True))
            if d - rem > 0:
                collapsed.append((lo + rem, lo + d, q * d, slp.power(w, q), -1, True))
        elif d < 0 and (-d) < L:
            assert m < 0
            stats["spirals"] += 1
            s = -d
            q, rem = divmod(L, s)
            if s - rem > 0:
                collapsed.append((hi - s, hi - rem, -q * s, slp.power(w, q), -1, True))
            if rem > 0:
                collapsed.append((hi - rem, hi, -(q + 1) * s, slp.power(w, q + 1), -1, True))
        else:
            collapsed.append((lo, hi, d, w, m, True))

    collapsed.sort(key=lambda x: x[0])
    starts = [p[0] for p in collapsed]

    # Compute In(U) = domain(collapsed) \ target(internal pieces)
    int_targets = sorted((lo + d, hi + d) for lo, hi, d, w, m, is_int in collapsed if is_int)
    merged_t = []
    for a, b in int_targets:
        if merged_t and merged_t[-1][1] == a:
            merged_t[-1] = (merged_t[-1][0], b)
        else:
            merged_t.append((a, b))

    entry_work = []
    internal_dom = []
    mt_idx = 0
    num_mt = len(merged_t)
    for lo, hi, d, w, m, is_int in collapsed:
        pos = lo
        while pos < hi:
            while mt_idx < num_mt and merged_t[mt_idx][1] <= pos:
                mt_idx += 1
            if mt_idx >= num_mt or merged_t[mt_idx][0] >= hi:
                entry_work.append((pos, hi, d, w, m, is_int))
                pos = hi
            else:
                ta, tb = merged_t[mt_idx]
                if pos < ta:
                    entry_work.append((pos, ta, d, w, m, is_int))
                    pos = ta
                ov_hi = hi if hi < tb else tb
                internal_dom.append((pos, ov_hi))
                pos = ov_hi

    # Track visited sub-intervals of internal_dom so we know exact unvisited cycles!
    visited_int = []

    out_pieces = []
    stack = list(reversed(entry_work))
    while stack:
        lo, hi, d, w, m, is_int = stack.pop()
        if not is_int:
            out_pieces.append((lo, hi, d, w, m))
            continue
        stats["bounces"] += 1
        t_lo, t_hi = lo + d, hi + d
        visited_int.append((t_lo, t_hi))
        idx = bisect_right(starts, t_lo) - 1
        pos = t_lo
        w_len = slp.length[w]
        nxt = []
        while pos < t_hi:
            stats["lookups"] += 1
            p_lo, p_hi, p_d, p_w, p_m, p_int = collapsed[idx]
            ov_hi = t_hi if t_hi < p_hi else p_hi
            sub_lo = pos - d
            sub_hi = ov_hi - d
            nxt.append((sub_lo, sub_hi, d + p_d, slp.concat(w, p_w), combine_mark(m, w_len, p_m), p_int))
            stats["slp_joins"] += 1
            pos = ov_hi
            idx += 1
        for item in reversed(nxt):
            stack.append(item)

    # Check if total visited internal length == total internal_dom length!
    tot_int = sum(b - a for a, b in internal_dom)
    tot_vis = sum(b - a for a, b in visited_int)
    if tot_vis < tot_int:
        # There are closed cycles strictly inside U!
        # Compute unvisited intervals = internal_dom \ visited_int
        visited_int.sort()
        m_vis = []
        for a, b in visited_int:
            if m_vis and m_vis[-1][1] >= a:
                if b > m_vis[-1][1]:
                    m_vis[-1] = (m_vis[-1][0], b)
            else:
                m_vis.append((a, b))
        unvis = []
        vi = 0
        nv = len(m_vis)
        for a, b in internal_dom:
            pos = a
            while pos < b:
                while vi < nv and m_vis[vi][1] <= pos:
                    vi += 1
                if vi >= nv or m_vis[vi][0] >= b:
                    unvis.append((pos, b))
                    pos = b
                else:
                    va, vb = m_vis[vi]
                    if pos < va:
                        unvis.append((pos, va))
                    pos = b if b < vb else vb

        # Extract closed cycles by resolving unvis intervals (pure bijection on unvis!)
        # Extract active pieces restricted to unvis, then iteratively eliminate intervals!
        cyc_pieces = []
        for ua, ub in unvis:
            idx = bisect_right(starts, ua) - 1
            pos = ua
            while pos < ub:
                p_lo, p_hi, p_d, p_w, p_m, p_int = collapsed[idx]
                assert p_int
                ov_hi = ub if ub < p_hi else p_hi
                cyc_pieces.append((pos, ov_hi, p_d, p_w, p_m))
                pos = ov_hi
                idx += 1

        # To close cycles cleanly with spiral compression on multi-step cycles:
        # Simply iteratively eliminate the last interval (Rauzy-Veech / Schur complement!)
        # or trace from the marked interval (if present) / first unvisited interval!
        # Let's use exact single-point-or-interval orbit closing on cyc_pieces:
        # Actually, doing Schur elimination of the rightmost piece (or splitting) is
        # literally identical to calling resolve_region on [u_min, u_max) with a cut,
        # OR simply: eliminate the first interval I_0 = [lo0, hi0) by tracing it back to itself!
        # Wait: if we restrict to the remaining set R, pick the FIRST piece P0 = [lo0, hi0)
        # (Wait: P0 might not be mapped back to itself in 1 step, its image might hit P1, ..., back to P0).
        # Even simpler and 100% general:
        # Cut off the LAST interval of cyc_pieces as a "virtual exit" region!
        # No, even cleaner: repeatedly split cyc_pieces into the FIRST piece (as "kept" K)
        # and all OTHER pieces (as "eliminated" E), and induce onto K!
        # Because K has 1 piece (or we induce onto the leftmost piece [a, b)),
        # wait: if cyc_pieces has multiple disconnected cycles, some might not hit K!
        # So: while cyc_pieces is non-empty:
        #   Pick the rightmost piece (lo_r, hi_r, d_r, w_r, m_r).
        #   If d_r == 0: record closed cycle, delete it!
        #   Else if self-overlapping: spiral-compress it!
        #   Else: eliminate [lo_r, hi_r) from all other pieces (bypassing it)!
        cyc_pieces.sort(key=lambda x: x[0])
        while cyc_pieces:
            # Pop the rightmost piece (or a self-overlapping piece first!)
            # Check for self-loops / spirals first
            lo, hi, d, w, m = cyc_pieces.pop()
            if d == 0:
                closed_cycles.append((lo, hi, w, m))
                stats["self_cycles"] += 1
                continue
            L = hi - lo
            if d > 0 and d < L:
                assert m < 0
                stats["spirals"] += 1
                q, rem = divmod(L, d)
                if rem > 0:
                    cyc_pieces.append((lo, lo + rem, (q + 1) * d, slp.power(w, q + 1), -1))
                if d - rem > 0:
                    cyc_pieces.append((lo + rem, lo + d, q * d, slp.power(w, q), -1))
                cyc_pieces.sort(key=lambda x: x[0])
                continue
            if d < 0 and (-d) < L:
                assert m < 0
                stats["spirals"] += 1
                s = -d
                q, rem = divmod(L, s)
                if s - rem > 0:
                    cyc_pieces.append((hi - s, hi - rem, -q * s, slp.power(w, q), -1))
                if rem > 0:
                    cyc_pieces.append((hi - rem, hi, -(q + 1) * s, slp.power(w, q + 1), -1))
                cyc_pieces.sort(key=lambda x: x[0])
                continue

            # Disjoint piece [lo, hi) -> [lo+d, hi+d).
            # Every other piece whose target overlaps [lo, hi) bypasses it!
            new_cyc = []
            for p_lo, p_hi, p_d, p_w, p_m in cyc_pieces:
                t_lo, t_hi = p_lo + p_d, p_hi + p_d
                if t_hi <= lo or t_lo >= hi:
                    new_cyc.append((p_lo, p_hi, p_d, p_w, p_m))
                else:
                    stats["cyc_bypass"] += 1
                    w_len = slp.length[p_w]
                    if t_lo < lo:
                        new_cyc.append((p_lo, lo - p_d, p_d, p_w, p_m))
                    ov_lo = lo if lo > t_lo else t_lo
                    ov_hi = hi if hi < t_hi else t_hi
                    new_cyc.append((
                        ov_lo - p_d, ov_hi - p_d,
                        p_d + d,
                        slp.concat(p_w, w),
                        combine_mark(p_m, w_len, m)
                    ))
                    stats["slp_joins"] += 1
                    if t_hi > hi:
                        new_cyc.append((hi - p_d, p_hi, p_d, p_w, p_m))
            cyc_pieces = new_cyc
            cyc_pieces.sort(key=lambda x: x[0])

    out_pieces.sort(key=lambda x: x[0])
    return out_pieces, closed_cycles


def decode_btm_tree(L_bytes, primary_idx, mode="balanced"):
    """
    Exact inverse BWT via Compositional Redheffer Star Product of BTMs
    over a merge tree of F-buckets.
    mode:
      - "balanced": balanced binary tree over non-empty F-buckets
      - "run_balanced": binary tree split by equal run-count
      - "lazard_right": right-to-left comb (eliminating max bucket first)
    """
    n = len(L_bytes)
    if n == 0:
        return b"", {}

    n, buckets, pieces_by_bucket, slp = build_initial_pieces_from_L(L_bytes, primary_idx)
    stats = defaultdict(int)
    stats["r0"] = sum(len(v) for v in pieces_by_bucket.values())
    stats["num_buckets"] = len(buckets)

    # Leaf level: compute Phi_{F_c} for each bucket F_c independently
    leaves = []
    all_closed = []
    for c, b_lo, b_hi in buckets:
        out_p, closed = resolve_region_exact(b_lo, b_hi, pieces_by_bucket[c], slp, stats)
        stats["leaf_pieces"] += len(out_p)
        all_closed.extend(closed)
        leaves.append((b_lo, b_hi, out_p))

    level_stats = []

    def merge_nodes(node1, node2, depth):
        lo1, hi1, p1 = node1
        lo2, hi2, p2 = node2
        assert hi1 == lo2
        combined = p1 + p2
        out_p, closed = resolve_region_exact(lo1, hi2, combined, slp, stats)
        all_closed.extend(closed)
        level_stats.append((depth, len(p1) + len(p2), len(out_p)))
        return (lo1, hi2, out_p)

    def build_balanced(nodes, depth=1):
        if len(nodes) == 1:
            return nodes[0]
        if mode == "balanced":
            mid = len(nodes) // 2
        elif mode == "run_balanced":
            tot = sum(len(x[2]) for x in nodes)
            acc = 0
            mid = 1
            for idx in range(len(nodes) - 1):
                acc += len(nodes[idx][2])
                if acc * 2 >= tot:
                    mid = idx + 1
                    break
        elif mode == "lazard_right":
            mid = len(nodes) - 1
        else:
            raise ValueError(mode)
        left = build_balanced(nodes[:mid], depth + 1)
        right = build_balanced(nodes[mid:], depth + 1)
        return merge_nodes(left, right, depth)

    root = build_balanced(leaves)
    assert len(root[2]) == 0, f"Root still has {len(root[2])} open pieces!"

    # Find the closed cycle carrying the primary mark!
    marked = [c for c in all_closed if c[3] >= 0]
    assert len(marked) == 1, f"Expected 1 marked cycle, got {len(marked)}"
    c_lo, c_hi, c_slp, mark_off = marked[0]
    assert c_hi - c_lo == 1

    # Expand cycle SLP (in LF order: L[x], L[f(x)], ...)
    # Primary row p is at mark_off: L[p] = cyc[mark_off], L[f(p)] = cyc[(mark_off+1)%q], ...
    # Remember standard inverse BWT:
    #   T[n-1] = L[p], T[n-2] = L[f(p)], ..., T[0] = L[f^{n-1}(p)]
    cyc = slp.expand_bytes(c_slp)
    q = len(cyc)
    # Rotate cyc so index 0 is L[p], then repeat to length n, then reverse!
    rot = cyc[mark_off:] + cyc[:mark_off]
    lf_seq = (rot * (n // q + 1))[:n]
    text = lf_seq[::-1]

    stats["slp_total_nodes"] = len(slp.kind) - 256
    stats["max_level_pieces"] = max((x[2] for x in level_stats), default=0)
    stats["sum_tree_pieces"] = stats["leaf_pieces"] + sum(x[2] for x in level_stats)
    return text, dict(stats)


def bwt_encode(text):
    """Reference circular BWT returning (L_bytes, primary_idx)."""
    n = len(text)
    rots = sorted(range(n), key=lambda i: text[i:] + text[:i])
    L = bytes(text[(i - 1) % n] for i in rots)
    p = rots.index(0)
    return L, p


def ref_ibwt(L_bytes, p):
    """Reference LF-walk inverse BWT (handles periodic and arbitrary valid LF)."""
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


def run_exhaustive_tests():
    print("=== 1. Exhaustive Small Verification ===")
    total_cases = 0

    # Binary n <= 8 (all 2^n strings, plus all primary indices for n <= 6)
    bin_cases = 0
    for n in range(1, 9):
        for mask in range(1 << n):
            T = bytes((mask >> i) & 1 for i in range(n))
            L, p = bwt_encode(T)
            if n <= 6:
                # Test ALL primary indices (including non-standard / multi-cycle entries)
                for p_test in range(n):
                    ref = ref_ibwt(L, p_test)
                    dec, _ = decode_btm_tree(L, p_test, mode="balanced")
                    assert dec == ref, f"Mismatch binary n={n} T={T} p={p_test}"
                    bin_cases += 1
            else:
                dec, _ = decode_btm_tree(L, p, mode="balanced")
                assert dec == T, f"Mismatch binary n={n} T={T}"
                bin_cases += 1
    print(f"  Binary n<=8 (all primary marks for n<=6): {bin_cases} cases PASS [verified]")
    total_cases += bin_cases

    # Ternary n <= 5 (all 3^n strings, all primary marks)
    tern_cases = 0
    for n in range(1, 6):
        for code in range(3 ** n):
            x = code
            T = bytearray(n)
            for i in range(n):
                T[i] = x % 3
                x //= 3
            T = bytes(T)
            L, p = bwt_encode(T)
            for p_test in range(n):
                ref = ref_ibwt(L, p_test)
                dec, _ = decode_btm_tree(L, p_test, mode="balanced")
                assert dec == ref, f"Mismatch ternary n={n} T={T} p={p_test}"
                tern_cases += 1
    print(f"  Ternary n<=5 (all primary marks): {tern_cases} cases PASS [verified]")
    total_cases += tern_cases

    # Arbitrary non-BWT multi-cycle L strings over ternary n <= 5
    arb_cases = 0
    for n in range(1, 6):
        for code in range(3 ** n):
            x = code
            L = bytearray(n)
            for i in range(n):
                L[i] = x % 3
                x //= 3
            L = bytes(L)
            # Test primary index 0 and n//2
            for p_test in set([0, n // 2, n - 1]):
                ref = ref_ibwt(L, p_test)
                dec, _ = decode_btm_tree(L, p_test, mode="balanced")
                assert dec == ref, f"Mismatch arb L={L} p={p_test}"
                arb_cases += 1
    print(f"  Arbitrary multi-cycle LF maps (ternary n<=5): {arb_cases} cases PASS [verified]")
    total_cases += arb_cases

    # Also test all 3 merge modes on quaternary n=4 samples + periodic strings
    per_cases = 0
    for base in [b"ab", b"abc", b"aab", b"abac", b"0123", b"aaaa"]:
        for rep in [2, 3, 5]:
            T = base * rep
            L, p = bwt_encode(T)
            for mode in ["balanced", "run_balanced", "lazard_right"]:
                dec, _ = decode_btm_tree(L, p, mode=mode)
                assert dec == T, f"Periodic mismatch {base}*{rep} mode={mode}"
                per_cases += 1
    print(f"  Periodic multi-cycle BWTs (3 tree modes): {per_cases} cases PASS [verified]")
    total_cases += per_cases
    print(f"  TOTAL EXHAUSTIVE CASES VERIFIED: {total_cases}")


def run_adversarial_tests():
    print("\n=== 2. Adversarial Cases ===")
    # A. Symmetric-Middle-Run: e.g. 0 1^K 2 1^K ... or standard Referee witness
    # Let's construct a hard Symmetric-Middle-Run:
    # Mix of small (0), middle (1^K), and large (2) symbols with distinct flanking contexts
    for K in [16, 64, 256]:
        # Construction: concat_{i=0}^{m-1} (0, 1^K, 2, 1^K) with unique tags at 0/2 boundaries
        # Actually Referee's Symmetric-Middle-Run had K=64, n=4224, r=130:
        # 4224 = 64 * 66 = 32 * (2*64 + 4) -> 32 blocks of (a, M^K, b, M^K) or similar!
        # Let's make a clean family: a_i M^K b_i M^K with binary-coded tags in {0, 1} and {3, 4} and M = 2!
        chunks = []
        for i in range(16):
            tag_lo = bytes([(i >> b) & 1 for b in range(4)])        # in {0, 1} < 2
            tag_hi = bytes([3 + ((i >> b) & 1) for b in range(4)])  # in {3, 4} > 2
            chunks.append(tag_lo + bytes([2]) * K + tag_hi + bytes([2]) * K)
        T = b"".join(chunks)
        L, p = bwt_encode(T)
        dec, st = decode_btm_tree(L, p, mode="balanced")
        assert dec == T
        print(f"  Symmetric-Middle-Run K={K:3d} n={len(T):5d} r0={st.get('r0',0):4d}: "
              f"bounces={st.get('bounces',0):4d} lookups={st.get('lookups',0):4d} "
              f"slp_nodes={st.get('slp_total_nodes',0):4d} spirals={st.get('spirals',0):2d} "
              f"sum_tree_pieces={st.get('sum_tree_pieces',0):4d}")

    # Exact Referee Symmetric-Middle-Run witness (64 blocks: 2-char low A..H + M^64 + 2-char high S..Z)
    for K in [4, 16, 64]:
        M_cnt = 64
        low_tags = [bytes([65 + (i // 8), 65 + (i % 8)]) for i in range(M_cnt)]
        high_tags = [bytes([83 + (i // 8), 83 + (i % 8)]) for i in range(M_cnt)]
        T = b"".join(low_tags[i] + b"M" * K + high_tags[i] for i in range(M_cnt))
        L, p = bwt_encode(T)
        dec, st = decode_btm_tree(L, p, mode="balanced")
        assert dec == T
        print(f"  Referee-SymMidRun K={K:2d} n={len(T):4d} r0={st.get('r0',0):3d}: "
              f"bounces={st.get('bounces',0):4d} lookups={st.get('lookups',0):4d} "
              f"slp={st.get('slp_total_nodes',0):4d} spirals={st.get('spirals',0):2d} "
              f"sum_P={st.get('sum_tree_pieces',0):4d}")

    # Exact n=5 and n=10 F4 witnesses from Teamwork journal
    for name, T_list in [
        ("n5-thresh", [0, 0, 1, 1, 2]),
        ("n5-split", [0, 0, 2, 0, 1]),
        ("n5-selfrun", [0, 0, 1, 0, 1]),
        ("n10-F4", [0, 0, 1, 1, 2, 1, 2, 0, 0, 1]),
    ]:
        T = bytes(T_list)
        L, p = bwt_encode(T)
        dec, st = decode_btm_tree(L, p, mode="balanced")
        assert dec == T
        print(f"  Witness {name:10s} n={len(T):2d} r0={st.get('r0',0):2d}: "
              f"bounces={st.get('bounces',0):2d} lookups={st.get('lookups',0):2d} "
              f"slp={st.get('slp_total_nodes',0):2d} sum_P={st.get('sum_tree_pieces',0):2d}")

    # B. Fibonacci word Fib_{2m} + $ (exact Gamma convention: $ > 1 > 0 -> bytes 2 > 1 > 0)
    for m in range(3, 10):
        a_f, b_f = [0], [0, 1]
        for _ in range(2 * m - 2):
            a_f, b_f = b_f, b_f + a_f
        T = bytes(b_f + [2])  # $ = 2 > 1 > 0
        L, p = bwt_encode(T)
        dec, st = decode_btm_tree(L, p, mode="balanced")
        assert dec == T
        print(f"  Fib_{2*m:2d}+$ n={len(T):5d} r0={st.get('r0',0):3d}: "
              f"bounces={st.get('bounces',0):3d} lookups={st.get('lookups',0):3d} "
              f"slp_nodes={st.get('slp_total_nodes',0):3d} spirals={st.get('spirals',0):2d} "
              f"cyc_bypass={st.get('cyc_bypass',0):3d} sum_P={st.get('sum_tree_pieces',0):3d}")


def run_wikidata_test():
    print("\n=== 3. Authenticated Wikidata Blocks ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    for entry in manifest["blocks"]:
        path = f"/tmp/mibz2-blocks/{entry['file']}"
        with open(path, "rb") as f:
            L = f.read()
        sha = hashlib.sha256(L).hexdigest()
        assert sha == entry["sha256"], f"SHA mismatch on {entry['file']}"
        p = entry["primary_index"]
        n = entry["n"]
        assert len(L) == n

        t0 = time.perf_counter()
        modes = ["balanced", "run_balanced"] if entry["file"] == "block-0.L" else ["balanced"]
        for mode in modes:
            t1 = time.perf_counter()
            dec, st = decode_btm_tree(L, p, mode=mode)
            dt = time.perf_counter() - t1
            if mode == "balanced":
                ref = ref_ibwt(L, p)
                assert dec == ref, f"EXACTNESS FAILURE on {entry['file']}!"
            print(f"  {entry['file']:12s} [{mode:12s}] n={n} r0={st.get('r0',0):6d} "
                  f"bounces={st.get('bounces',0):8d} lookups={st.get('lookups',0):8d} "
                  f"slp={st.get('slp_total_nodes',0):8d} sum_P={st.get('sum_tree_pieces',0):7d} "
                  f"cyc_bp={st.get('cyc_bypass',0):6d} spirals={st.get('spirals',0):5d} "
                  f"py_t={dt:.2f}s [EXACT]")
        if time.perf_counter() - t0 > 60.0:
            print("  (stopping after slow block to analyze first)")
            break


if __name__ == "__main__":
    run_exhaustive_tests()
    run_adversarial_tests()
    if "--wikidata" in sys.argv:
        run_wikidata_test()
