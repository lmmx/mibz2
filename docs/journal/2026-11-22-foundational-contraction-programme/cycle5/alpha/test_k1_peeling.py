#!/usr/bin/env python3
"""
Cycle 5 — Run-Overlap Graph G(I_0) & Split-Free kappa=1 2-Core Peeling Algebra!

Constructs G(I_0) in ONE linear two-pointer merge (ZERO sorting, ZERO bucket-splitting):
  - Nodes = r_0 maximal L-runs (with primary row p isolated if desired, or tracked via mark_off)
  - Sorted I-list = L-order (already sorted in I_0!)
  - Sorted J-list = F-order (one O(r_0 + |Sigma|) counting-scatter pass, ZERO sorting!)
  - Overlap edges = single linear two-pointer merge of I-list and J-list (E_raw <= 2 r_0)!

Implements 4 exact split-free rewrite systems on G(I_0):
  - M1 (R1):          Strict 1-to-1 chain peeling: J_u == I_v (Delta q = -1, 0 splits)
  - M2 (R1+R2):       + Full-target unstraddled elimination: I_v = union_{u in In(v)} J_u (Delta q = -1, 0 splits)
  - M3 (R1+R2+R3):    + Right-split-free boundary-aligned peeling: J_u is prefix/suffix of I_v (Delta q <= 0, Delta m < 0)
  - M4 (R1+R2+R3+R3*):+ Bidirectional boundary-aligned peeling (also I_v is prefix/suffix of J_u; + O(1) self-loop spiral!)

Tests:
  1. Church-Rosser Confluence across multiple peeling orders on binary n<=8 & ternary n<=6!
  2. 100% byte-for-byte exactness on exhaustive binary n<=8, ternary n<=6,
     4 adversarial killers (3-phase, AltRing, Low-r Padded-Ring, SymMidRun), Fib_20+$,
     and ALL 6 authenticated Wikidata blocks!
  3. Complete 6-block census: q_0, round-1 peeled, fixpoint 2-Core q_core/q_0,
     m_core_thin / m_16, word-length distribution (==1, <=3, <=8, <=16, >16),
     peeling depth, and PeakM!
"""
import hashlib
import json
import random
import sys
import time
from bisect import bisect_right
from collections import defaultdict, deque

sys.path.insert(0, "/tmp/fcp-a")
import measure_all_trees as mt


class WordStore:
    """Tracks exact word lengths & exact byte strings (with SLP fallback > 16B for exact verify)."""
    __slots__ = ("kind", "a", "b", "length", "inline_bytes", "slp_gt16")

    def __init__(self):
        self.kind = [0] * 256
        self.a = list(range(256))
        self.b = [0] * 256
        self.length = [1] * 256
        self.inline_bytes = [bytes([c]) for c in range(256)]
        self.slp_gt16 = 0

    def concat(self, u, v):
        lu, lv = self.length[u], self.length[v]
        if lu + lv <= 16 and self.kind[u] == 0 and self.kind[v] == 0:
            idx = len(self.kind)
            self.kind.append(0); self.a.append(0); self.b.append(0)
            self.length.append(lu + lv)
            self.inline_bytes.append(self.inline_bytes[u] + self.inline_bytes[v])
            return idx
        idx = len(self.kind)
        self.kind.append(1); self.a.append(u); self.b.append(v)
        self.length.append(lu + lv)
        self.inline_bytes.append(b"")
        self.slp_gt16 += 1
        return idx

    def power(self, u, k):
        if k == 1:
            return u
        lu = self.length[u]
        if lu * k <= 16 and self.kind[u] == 0:
            idx = len(self.kind)
            self.kind.append(0); self.a.append(0); self.b.append(0)
            self.length.append(lu * k)
            self.inline_bytes.append(self.inline_bytes[u] * k)
            return idx
        idx = len(self.kind)
        self.kind.append(2); self.a.append(u); self.b.append(k)
        self.length.append(lu * k)
        self.inline_bytes.append(b"")
        self.slp_gt16 += 1
        return idx

    def expand(self, nid):
        if self.kind[nid] == 0:
            return self.inline_bytes[nid]
        out = bytearray()
        st = [(nid, 1)]
        while st:
            u, mult = st.pop()
            k = self.kind[u]
            if k == 0:
                b = self.inline_bytes[u]
                out.extend(b if mult == 1 else b * mult)
            elif k == 1:
                if mult == 1:
                    st.append((self.b[u], 1)); st.append((self.a[u], 1))
                else:
                    out.extend(self.expand(u) * mult)
            else:
                st.append((self.a[u], self.b[u] * mult))
        return bytes(out)


def build_initial_strips(L_bytes, primary_idx, isolate_p=True):
    """
    Build initial maximal-run strips directly from I_0 in ONE linear scan (ZERO sorting!).
    Returns (n, r0, m16, strips, j_order, ws) where:
      - strips[u] = [lo, hi, delta, word_id, mark_off, alive]
      - j_order = list of strip IDs sorted by target t_lo = lo + delta (via O(r0 + 256) counting scatter!)
    """
    n = len(L_bytes)
    if n == 0:
        return 0, 0, 0, [], [], WordStore()
    runs = []
    counts = [0] * 256
    i = 0
    m16 = 0
    p_thin = False
    while i < n:
        c = L_bytes[i]
        j = i + 1
        while j < n and L_bytes[j] == c:
            j += 1
        ln = j - i
        runs.append((c, i, j))
        counts[c] += ln
        if ln <= 16:
            m16 += ln
            if i <= primary_idx < j:
                p_thin = True
        i = j
    if not p_thin:
        m16 += 1
    r0 = len(runs)

    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += counts[c]
    cur_t = C[:]

    strips = []
    by_c = [[] for _ in range(256)]
    for c, lo, hi in runs:
        d = cur_t[c] - lo
        cur_t[c] += (hi - lo)
        if isolate_p and lo <= primary_idx < hi:
            if lo < primary_idx:
                uid = len(strips); strips.append([lo, primary_idx, d, c, -1, True]); by_c[c].append(uid)
            uid = len(strips); strips.append([primary_idx, primary_idx + 1, d, c, 0, True]); by_c[c].append(uid)
            if primary_idx + 1 < hi:
                uid = len(strips); strips.append([primary_idx + 1, hi, d, c, -1, True]); by_c[c].append(uid)
        else:
            uid = len(strips); strips.append([lo, hi, d, c, -1, True]); by_c[c].append(uid)

    # Global sorted J-order in O(r0 + 256), ZERO sorting!
    j_order = []
    for c in range(256):
        j_order.extend(by_c[c])

    return n, r0, m16, strips, j_order, WordStore()


def verify_strips_exact(strips, ws, n, ref_bytes, closed_primary=None):
    """Verify exact decoding from remaining live strips (or closed primary cycle)."""
    if closed_primary is not None:
        w, m0 = closed_primary
        cyc = ws.expand(w)
        q = len(cyc)
        rot = cyc[m0:] + cyc[:m0]
        text = ((rot * (n // q + 1))[:n])[::-1]
        assert text == ref_bytes, f"Closed-cycle mismatch!"
        return
    live = sorted((s for s in strips if s[5]), key=lambda x: x[0])
    starts = [s[0] for s in live]
    p_idx = next(i for i, s in enumerate(live) if s[4] >= 0)
    x0 = live[p_idx][0]
    m0 = live[p_idx][4]
    x = x0
    chunks = []
    while True:
        idx = bisect_right(starts, x) - 1
        lo, hi, d, w, m, _ = live[idx]
        assert lo <= x < hi, f"Missing live row {x}!"
        chunks.append(ws.expand(w))
        x = x + d
        if x == x0:
            break
    cyc = b"".join(chunks)
    q = len(cyc)
    rot = cyc[m0:] + cyc[:m0]
    text = ((rot * (n // q + 1))[:n])[::-1]
    assert text == ref_bytes, f"Strip-walk mismatch! len(cyc)={q}, n={n}"


def peel_k1_algebra(L_bytes, primary_idx, mode="M1", order_policy="fifo", rng_seed=0, isolate_p=True):
    """
    Exact split-free kappa=1 2-Core Peeling on the Run-Overlap Graph G(I_0)!

    Maintains dynamic exact overlap incidence via sorted domain-interval / target-interval maps
    (clean O(log q) lookup in Python prototype; in C/Rust for M1/M2 this is O(1) linked adjacency!).
    Modes:
      - M1: R1 only (exact 1-to-1 chain contraction: J_u == I_v, u != v, Delta q = -1)
      - M2: R1 + R2 (full-target unstraddled elimination: I_v = disjoint union of J_u for u in In(v), v not in In(v), Delta q = -1)
      - M3: R1 + R2 + R3 (+ right-split-free boundary-aligned peeling: J_u is prefix/suffix of I_v, u != v, Delta q <= 0, Delta m < 0)
      - M4: R1 + R2 + R3 + R3* (+ bidirectional boundary-aligned peeling: I_v is prefix/suffix of J_u, + O(1) self-loop spiral!)
    order_policy: "fifo", "lifo", "narrow_first", "wide_first", "random"
    """
    n, r0, m16, strips, j_order, ws = build_initial_strips(L_bytes, primary_idx, isolate_p=isolate_p)
    q0 = len(strips)
    if q0 == 0:
        return [], ws, None, {}

    # Build initial G(I_0) overlap adjacency in ONE linear two-pointer merge (ZERO sorting!)
    # Maintain for each live strip u:
    #   lo, hi, delta, w, m, alive
    # And exact sorted maps:
    #   dom_by_lo: dict lo -> u  (and sorted lo list or direct adjacency!)
    # Actually: let's maintain EXPLICIT adjacency lists `out_adj[u]` (list of target strips v overlapping J_u)
    # and `in_adj[v]` (list of source strips u whose J_u overlaps I_v), KEPT EXACTLY SYNCHRONIZED!
    # Wait: for M1 and M2, updating `out_adj` and `in_adj` is PURE O(1) GRAPH POINTER SURGERY —
    # NO binary searches AND NO coordinate lookups EVER!!
    # Check that!!
    #   - Under R1 (J_u == I_v, delete v):
    #       u's new target IS J_v! So u's new `out_adj[u]` IS LITERALLY `out_adj[v]`!
    #       And for every w in `out_adj[v]`, replace `v` by `u` in `in_adj[w]`!
    #       ZERO BINARY SEARCHES! ZERO SORTING! PURE O(1) GRAPH SURGERY!
    #   - Under R2 (I_v = union_{u in In(v)} J_u, delete v):
    #       Every u in In(v) has J_u shifted by +delta_v into J_v, so u's new `out_adj[u]` is simply
    #       a sub-slice of `out_adj[v]` (found in ONE linear scan across the two already-ordered lists
    #       In(v) and Out(v)!!)!
    #       ZERO BINARY SEARCHES! ZERO SORTING!
    #   - Under R3 (J_u is prefix/suffix of I_v):
    #       u's new target is a prefix/suffix of J_v -> linear scan from left/right end of `out_adj[v]`!
    #       ZERO BINARY SEARCHES!
    #   - Under R3* (I_v is prefix/suffix of J_u):
    #       v is deleted, u splits into u1 (target J_v -> inherits `out_adj[v]`) and u2 (target J_u \ I_v ->
    #       inherits `out_adj[u] \ {v}`), and `in_adj[u]` splits at `lo_u + |I_v|` (linear scan from left/right
    #       end of `in_adj[u]`)!
    #       ZERO BINARY SEARCHES!

    # Let's build exact ordered `out_adj[u]` (ordered left-to-right along J_u) and `in_adj[v]` (ordered left-to-right along I_v)
    # via ONE linear two-pointer merge of I-list (0..q0-1) and J-list (j_order)!
    out_adj = [[] for _ in range(q0)]
    in_adj = [[] for _ in range(q0)]
    e0 = 0
    ii = 0
    jj = 0
    while ii < q0 and jj < q0:
        v = ii
        u = j_order[jj]
        v_lo, v_hi = strips[v][0], strips[v][1]
        u_tlo, u_thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
        if u_thi <= v_lo:
            jj += 1
        elif v_hi <= u_tlo:
            ii += 1
        else:
            out_adj[u].append(v)
            in_adj[v].append(u)
            e0 += 1
            if u_thi < v_hi:
                jj += 1
            elif v_hi < u_thi:
                ii += 1
            else:
                ii += 1
                jj += 1

    # Helper to check if a strip v or u is eligible for peeling
    def is_r1_edge(u):
        """u has out_deg == 1 -> v, and v has in_deg == 1 <- u, and u != v."""
        if not strips[u][5] or len(out_adj[u]) != 1:
            return False
        v = out_adj[u][0]
        return v != u and strips[v][5] and len(in_adj[v]) == 1 and in_adj[v][0] == u

    def is_r2_node(v):
        """v has all incoming u with out_deg(u) == 1 (no straddling at s_v or e_v) and v not in in_adj[v]."""
        if not strips[v][5] or not in_adj[v]:
            return False
        for u in in_adj[v]:
            if u == v or (not strips[u][5]) or len(out_adj[u]) != 1:
                return False
        return True

    def is_r3_edge(u):
        """u has out_deg == 1 -> v (u != v), and J_u is prefix or suffix of I_v (includes R1!)."""
        if not strips[u][5] or len(out_adj[u]) != 1:
            return False
        v = out_adj[u][0]
        if v == u or not strips[v][5]:
            return False
        t_lo = strips[u][0] + strips[u][2]
        t_hi = strips[u][1] + strips[u][2]
        return (t_lo == strips[v][0]) or (t_hi == strips[v][1])

    def is_r3star_node(v):
        """v has in_deg == 1 <- u (u != v), and I_v is strict prefix or suffix of J_u."""
        if not strips[v][5] or len(in_adj[v]) != 1:
            return False
        u = in_adj[v][0]
        if u == v or not strips[u][5]:
            return False
        t_lo = strips[u][0] + strips[u][2]
        t_hi = strips[u][1] + strips[u][2]
        v_lo, v_hi = strips[v][0], strips[v][1]
        return (v_lo == t_lo and v_hi < t_hi) or (v_hi == t_hi and v_lo > t_lo)

    def is_spiral_node(v):
        if not strips[v][5]:
            return False
        lo, hi, d, w, m, _ = strips[v]
        if d == 0:
            return True
        L = hi - lo
        return (d > 0 and d < L) or (d < 0 and (-d) < L)

    # Track exact peeling stats
    stats = defaultdict(int)
    stats["r0"] = r0
    stats["q0"] = q0
    stats["e0"] = e0
    stats["m16"] = m16
    closed_primary = None

    # Worklist with wave-round tracking for depth measurement!
    rng = random.Random(rng_seed)
    in_q = set()
    work = deque()

    def push_cand(node):
        if node not in in_q and strips[node][5]:
            in_q.add(node)
            work.append(node)

    # Seed initial worklist
    init_cands = list(range(q0))
    if order_policy == "random":
        rng.shuffle(init_cands)
    elif order_policy == "narrow_first":
        init_cands.sort(key=lambda u: strips[u][1] - strips[u][0])
    elif order_policy == "wide_first":
        init_cands.sort(key=lambda u: -(strips[u][1] - strips[u][0]))
    for u in init_cands:
        push_cand(u)

    # Track Round 1 peeled count (nodes eligible in the INITIAL graph G(I_0)!)
    r1_init_elig = 0
    for u in range(q0):
        if mode == "M1" and is_r1_edge(u): r1_init_elig += 1
        elif mode == "M2" and is_r2_node(u): r1_init_elig += 1
        elif mode == "M3" and (is_r2_node(u) or is_r3_edge(u)): r1_init_elig += 1
        elif mode == "M4" and (is_r2_node(u) or is_r3_edge(u) or is_r3star_node(u) or is_spiral_node(u)): r1_init_elig += 1
    stats["round1_eligible"] = r1_init_elig

    # Helper: replace element old_id with new_ids (list) in an ordered adjacency list!
    def replace_in_list(lst, old_id, new_ids):
        idx = lst.index(old_id)
        lst[idx:idx + 1] = new_ids

    # Execute peeling until fixpoint!
    wave_marker = -1
    work.append(wave_marker)
    waves = 0
    ops_in_wave = 0

    while work:
        if order_policy == "fifo":
            cand = work.popleft()
        elif order_policy == "lifo":
            cand = work.pop()
            if cand == wave_marker:
                continue
        elif order_policy == "random":
            # Pop random non-marker
            if len(work) == 1 and work[0] == wave_marker:
                break
            idx = rng.randrange(len(work))
            cand = work[idx]
            work[idx] = work[-1]
            work.pop()
            if cand == wave_marker:
                continue
        elif order_policy in ("narrow_first", "wide_first"):
            # Pick narrowest/widest (only used on small confluence tests!)
            non_m = [x for x in work if x != wave_marker]
            if not non_m:
                break
            if order_policy == "narrow_first":
                cand = min(non_m, key=lambda x: (strips[x][1] - strips[x][0], x))
            else:
                cand = max(non_m, key=lambda x: (strips[x][1] - strips[x][0], -x))
            work.remove(cand)
        else:
            cand = work.popleft()

        if cand == wave_marker:
            if ops_in_wave > 0:
                waves += 1
                ops_in_wave = 0
                if work:
                    work.append(wave_marker)
            continue

        in_q.discard(cand)
        if not strips[cand][5]:
            continue

        # Try applicable rules for `mode`!
        # --- Rule: Self-loop / Spiral (in M4, or d == 0 cycle) ---
        if mode == "M4" and is_spiral_node(cand):
            lo, hi, d, w, m, _ = strips[cand]
            if d == 0:
                if m >= 0:
                    closed_primary = (w, m)
                strips[cand][5] = False
                out_adj[cand] = []
                in_adj[cand] = []
                stats["spirals"] += 1
                ops_in_wave += 1
                continue
            L = hi - lo
            # Spiral division on cand! Note: overlap [lo+d, hi) (for d>0) is ONLY targeted by cand itself!
            # Wait: check that! No OTHER strip can target [lo+d, hi) because it's inside J_cand!
            # And no OTHER strip's domain can be inside [lo+d, hi) because it's inside I_cand!
            # So cand's `in_adj[cand]` (other than cand) ONLY targets the entry window!
            # And cand's `out_adj[cand]` (other than cand) ONLY overlaps the exit window!
            assert m < 0, "Marked piece has length 1 so cannot have 0 < |d| < L"
            stats["spirals"] += 1
            ops_in_wave += 1
            other_in = [x for x in in_adj[cand] if x != cand]
            other_out = [x for x in out_adj[cand] if x != cand]
            new_ids = []
            if d > 0:
                q_sp, rem = divmod(L, d)
                # Entry window [lo, lo+d) -> exit window [hi, hi+d)
                # Piece 1: [lo, lo+rem) -> [hi+d-rem, hi+d) with w^{q+1}
                # Piece 2: [lo+rem, lo+d) -> [hi, hi+d-rem) with w^q
                if rem > 0:
                    nid = len(strips)
                    strips.append([lo, lo + rem, (q_sp + 1) * d, ws.power(w, q_sp + 1), -1, True])
                    out_adj.append([]); in_adj.append([])
                    new_ids.append(nid)
                if d - rem > 0:
                    nid = len(strips)
                    strips.append([lo + rem, lo + d, q_sp * d, ws.power(w, q_sp), -1, True])
                    out_adj.append([]); in_adj.append([])
                    new_ids.append(nid)
            else:
                s_sp = -d
                q_sp, rem = divmod(L, s_sp)
                # Entry window [hi-s, hi) -> exit window [lo-s, lo)
                if s_sp - rem > 0:
                    nid = len(strips)
                    strips.append([hi - s_sp, hi - rem, -q_sp * s_sp, ws.power(w, q_sp), -1, True])
                    out_adj.append([]); in_adj.append([])
                    new_ids.append(nid)
                if rem > 0:
                    nid = len(strips)
                    strips.append([hi - rem, hi, -(q_sp + 1) * s_sp, ws.power(w, q_sp + 1), -1, True])
                    out_adj.append([]); in_adj.append([])
                    new_ids.append(nid)
            # Rewire other_in -> new_ids (ordered by lo) and new_ids -> other_out (ordered by target!)
            strips[cand][5] = False
            for u in other_in:
                u_tlo, u_thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
                hits = [nid for nid in new_ids if strips[nid][0] < u_thi and strips[nid][1] > u_tlo]
                replace_in_list(out_adj[u], cand, hits)
                for nid in hits:
                    in_adj[nid].append(u)
                push_cand(u)
            # Order new_ids by target t_lo for rewiring into other_out!
            new_by_t = sorted(new_ids, key=lambda nid: strips[nid][0] + strips[nid][2])
            for v in other_out:
                v_lo, v_hi = strips[v][0], strips[v][1]
                hits = [nid for nid in new_by_t if (strips[nid][0] + strips[nid][2]) < v_hi and (strips[nid][1] + strips[nid][2]) > v_lo]
                replace_in_list(in_adj[v], cand, hits)
                for nid in hits:
                    out_adj[nid].append(v)
                push_cand(v)
            for nid in new_ids:
                push_cand(nid)
            out_adj[cand] = []; in_adj[cand] = []
            continue

        # --- Rule R1: exact 1-to-1 chain contraction (u -> v, J_u == I_v) ---
        if is_r1_edge(cand):
            u = cand
            v = out_adj[u][0]
            # Merge u o v, delete v!
            w_u_len = ws.length[strips[u][3]]
            strips[u][2] += strips[v][2]
            strips[u][3] = ws.concat(strips[u][3], strips[v][3])
            strips[u][4] = mt.combine_mark(strips[u][4], w_u_len, strips[v][4])
            strips[v][5] = False
            # O(1) graph pointer surgery: u inherits out_adj[v]!
            v_outs = out_adj[v]
            out_adj[u] = list(v_outs)
            for nxt in v_outs:
                replace_in_list(in_adj[nxt], v, [u])
                push_cand(nxt)
            out_adj[v] = []; in_adj[v] = []
            stats["r1_ops"] += 1
            stats["total_ops"] += 1
            ops_in_wave += 1
            push_cand(u)
            for prev in in_adj[u]:
                push_cand(prev)
            continue

        # --- Rule R2: Full-target unstraddled elimination of v (I_v = union J_u, all u have out_deg == 1) ---
        if mode in ("M2", "M3", "M4") and is_r2_node(cand):
            v = cand
            v_d = strips[v][2]
            v_w = strips[v][3]
            v_m = strips[v][4]
            v_ins = list(in_adj[v])
            v_outs = list(out_adj[v])
            strips[v][5] = False

            # Update every u in v_ins in place!
            # Their new targets [u_tlo + v_d, u_thi + v_d) partition J_v in EXACTLY the order of v_ins!
            # Linear two-pointer merge between v_ins (ordered by new target) and v_outs (ordered by lo)!
            new_in_lists = {nxt: [] for nxt in v_outs}
            for u in v_ins:
                w_u_len = ws.length[strips[u][3]]
                strips[u][2] += v_d
                strips[u][3] = ws.concat(strips[u][3], v_w)
                strips[u][4] = mt.combine_mark(strips[u][4], w_u_len, v_m)
                out_adj[u] = []
                stats["r2_joins"] += 1
                stats["total_ops"] += 1
            stats["r2_nodes"] += 1
            ops_in_wave += 1

            # Two-pointer merge of v_ins (new targets) against v_outs (domains)
            i_u = 0; i_w = 0
            nu = len(v_ins); nw = len(v_outs)
            while i_u < nu and i_w < nw:
                u = v_ins[i_u]; nxt = v_outs[i_w]
                u_tlo, u_thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
                w_lo, w_hi = strips[nxt][0], strips[nxt][1]
                if u_thi <= w_lo:
                    i_u += 1
                elif w_hi <= u_tlo:
                    i_w += 1
                else:
                    out_adj[u].append(nxt)
                    new_in_lists[nxt].append(u)
                    if u_thi < w_hi: i_u += 1
                    elif w_hi < u_thi: i_w += 1
                    else: i_u += 1; i_w += 1

            for nxt in v_outs:
                replace_in_list(in_adj[nxt], v, new_in_lists[nxt])
                push_cand(nxt)
            for u in v_ins:
                push_cand(u)
                for prev in in_adj[u]:
                    push_cand(prev)
            out_adj[v] = []; in_adj[v] = []
            continue

        # --- Rule R3: Right-split-free boundary-aligned peeling (J_u is strict prefix or suffix of I_v) ---
        if mode in ("M3", "M4") and is_r3_edge(cand):
            u = cand
            v = out_adj[u][0]
            # Note: if J_u == I_v it was already handled by R1 above!
            u_tlo, u_thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
            v_lo, v_hi, v_d, v_w, v_m, _ = strips[v]
            assert v_m < 0, "Marked piece has length 1 so cannot have strict sub-interval!"
            is_prefix = (u_tlo == v_lo)
            ln = u_thi - u_tlo

            # Update u in place: J_u -> [u_tlo + v_d, u_thi + v_d)
            w_u_len = ws.length[strips[u][3]]
            strips[u][2] += v_d
            strips[u][3] = ws.concat(strips[u][3], v_w)
            strips[u][4] = mt.combine_mark(strips[u][4], w_u_len, -1)

            # Shrink v cleanly from left or right (ZERO split of v!)
            if is_prefix:
                strips[v][0] += ln
                # Remove u from left end of in_adj[v]
                assert in_adj[v][0] == u
                in_adj[v].pop(0)
            else:
                strips[v][1] -= ln
                assert in_adj[v][-1] == u
                in_adj[v].pop()

            # Rewire out_adj[u] and out_adj[v] by scanning from left or right end of out_adj[v]!
            new_u_tlo = strips[u][0] + strips[u][2]
            new_u_thi = strips[u][1] + strips[u][2]
            new_v_tlo = strips[v][0] + strips[v][2]
            new_v_thi = strips[v][1] + strips[v][2]
            out_adj[u] = []

            if is_prefix:
                # Scan out_adj[v] from LEFT
                k = 0
                while k < len(out_adj[v]):
                    nxt = out_adj[v][k]
                    w_lo, w_hi = strips[nxt][0], strips[nxt][1]
                    if w_lo >= new_u_thi:
                        break
                    out_adj[u].append(nxt)
                    if w_hi <= new_u_thi:
                        # nxt is completely transferred from v to u!
                        replace_in_list(in_adj[nxt], v, [u])
                        out_adj[v].pop(k)
                        push_cand(nxt)
                    else:
                        # nxt straddles new_u_thi: BOTH u and v now point to nxt!
                        replace_in_list(in_adj[nxt], v, [u, v])
                        push_cand(nxt)
                        break
            else:
                # Scan out_adj[v] from RIGHT
                u_hits_rev = []
                k = len(out_adj[v]) - 1
                while k >= 0:
                    nxt = out_adj[v][k]
                    w_lo, w_hi = strips[nxt][0], strips[nxt][1]
                    if w_hi <= new_u_tlo:
                        break
                    u_hits_rev.append(nxt)
                    if w_lo >= new_u_tlo:
                        replace_in_list(in_adj[nxt], v, [u])
                        out_adj[v].pop(k)
                        push_cand(nxt)
                        k -= 1
                    else:
                        replace_in_list(in_adj[nxt], v, [v, u])
                        push_cand(nxt)
                        break
                out_adj[u] = list(reversed(u_hits_rev))

            stats["r3_ops"] += 1
            stats["total_ops"] += 1
            ops_in_wave += 1
            push_cand(u)
            push_cand(v)
            for prev in in_adj[u]: push_cand(prev)
            if in_adj[v]:
                push_cand(in_adj[v][0])
                push_cand(in_adj[v][-1])
            continue

        # --- Rule R3*: Left-split-free boundary-aligned peeling (I_v is strict prefix/suffix of J_u, in_deg(v)==1) ---
        if mode == "M4" and is_r3star_node(cand):
            v = cand
            u = in_adj[v][0]
            u_lo, u_hi, u_d, u_w, u_m, _ = strips[u]
            v_lo, v_hi, v_d, v_w, v_m, _ = strips[v]
            assert u_m < 0, "Marked piece has length 1 so cannot have strict sub-interval!"
            u_tlo, u_thi = u_lo + u_d, u_hi + u_d
            is_prefix = (v_lo == u_tlo)
            ln = v_hi - v_lo

            # Delete v, split u into u_abs (new ID, absorbing v) and u (shrunk)!
            # Net Delta q = 0, Delta m = -ln!
            strips[v][5] = False
            u_abs = len(strips)
            w_u_len = ws.length[u_w]
            new_w = ws.concat(u_w, v_w)
            new_m = mt.combine_mark(-1, w_u_len, v_m)
            if is_prefix:
                # u_abs = [u_lo, u_lo+ln) -> J_v; u shrinks to [u_lo+ln, u_hi) -> [u_tlo+ln, u_thi)
                strips.append([u_lo, u_lo + ln, u_d + v_d, new_w, new_m, True])
                strips[u][0] += ln
                assert out_adj[u][0] == v
                out_adj[u].pop(0)
            else:
                # u_abs = [u_hi-ln, u_hi) -> J_v; u shrinks to [u_lo, u_hi-ln) -> [u_tlo, u_thi-ln)
                strips.append([u_hi - ln, u_hi, u_d + v_d, new_w, new_m, True])
                strips[u][1] -= ln
                assert out_adj[u][-1] == v
                out_adj[u].pop()

            out_adj.append(list(out_adj[v]))
            in_adj.append([])

            # Rewire out_adj[v] -> u_abs!
            for nxt in out_adj[u_abs]:
                replace_in_list(in_adj[nxt], v, [u_abs])
                push_cand(nxt)
            out_adj[v] = []; in_adj[v] = []

            # Rewire in_adj[u] between u_abs and shrunk u (scan from left or right end of in_adj[u]!)
            abs_lo, abs_hi = strips[u_abs][0], strips[u_abs][1]
            if is_prefix:
                k = 0
                while k < len(in_adj[u]):
                    prev = in_adj[u][k]
                    p_tlo, p_thi = strips[prev][0] + strips[prev][2], strips[prev][1] + strips[prev][2]
                    if p_tlo >= abs_hi:
                        break
                    in_adj[u_abs].append(prev)
                    if p_thi <= abs_hi:
                        replace_in_list(out_adj[prev], u, [u_abs])
                        in_adj[u].pop(k)
                        push_cand(prev)
                    else:
                        replace_in_list(out_adj[prev], u, [u_abs, u])
                        push_cand(prev)
                        break
            else:
                hits_rev = []
                k = len(in_adj[u]) - 1
                while k >= 0:
                    prev = in_adj[u][k]
                    p_tlo, p_thi = strips[prev][0] + strips[prev][2], strips[prev][1] + strips[prev][2]
                    if p_thi <= abs_lo:
                        break
                    hits_rev.append(prev)
                    if p_tlo >= abs_lo:
                        replace_in_list(out_adj[prev], u, [u_abs])
                        in_adj[u].pop(k)
                        push_cand(prev)
                        k -= 1
                    else:
                        replace_in_list(out_adj[prev], u, [u, u_abs])
                        push_cand(prev)
                        break
                in_adj[u_abs] = list(reversed(hits_rev))

            stats["r3star_ops"] += 1
            stats["total_ops"] += 1
            ops_in_wave += 1
            push_cand(u_abs)
            push_cand(u)
            if out_adj[u]:
                push_cand(out_adj[u][0])
                push_cand(out_adj[u][-1])
            continue

    # Collect final 2-Core statistics!
    live = [s for s in strips if s[5]]
    q_core = len(live)
    rows_core = sum(s[1] - s[0] for s in live)
    m_core_thin = 0
    has_p_thin = False
    p_l1 = p_l3 = p_l8 = p_l16 = p_gt16 = 0
    r_l1 = r_l3 = r_l8 = r_l16 = r_gt16 = 0
    w1_core = 0
    for lo, hi, d, w, m, _ in live:
        width = hi - lo
        wl = ws.length[w]
        if width == 1: w1_core += 1
        if width <= 16:
            m_core_thin += width
            if m >= 0: has_p_thin = True
        if wl == 1: p_l1 += 1; r_l1 += width
        if wl <= 3: p_l3 += 1; r_l3 += width
        if wl <= 8: p_l8 += 1; r_l8 += width
        if wl <= 16: p_l16 += 1; r_l16 += width
        else: p_gt16 += 1; r_gt16 += width
    if q_core > 0 and not has_p_thin:
        m_core_thin += 1

    # Inspect WHY remaining thin strips in the core stopped peeling (degree histogram!)
    deg_hist = defaultdict(int)
    for idx, s in enumerate(strips):
        if s[5] and (s[1] - s[0]) <= 16:
            od = min(len(out_adj[idx]), 3)
            id_ = min(len(in_adj[idx]), 3)
            deg_hist[(id_, od)] += 1

    stats["q_core"] = q_core
    stats["rows_core"] = rows_core
    stats["m_core_thin"] = m_core_thin
    stats["w1_core"] = w1_core
    stats["waves"] = waves
    stats["slp_gt16"] = ws.slp_gt16
    stats["pct_l1"] = 100.0 * p_l1 / max(1, q_core)
    stats["pct_l3"] = 100.0 * p_l3 / max(1, q_core)
    stats["pct_l8"] = 100.0 * p_l8 / max(1, q_core)
    stats["pct_l16"] = 100.0 * p_l16 / max(1, q_core)
    stats["pct_gt16"] = 100.0 * p_gt16 / max(1, q_core)
    stats["deg_hist_thin"] = dict(deg_hist)
    return strips, ws, closed_primary, dict(stats)


def test_confluence_and_exhaustive():
    print("=== 1. Exhaustive Exactness & Church-Rosser Confluence Test (binary n<=8, ternary n<=6) ===")
    tot_verified = 0
    for mode in ("M1", "M2", "M3", "M4"):
        non_conf = []
        non_conf_q = []
        checked = 0
        for base, max_n in [(2, 8), (3, 6)]:
            for n in range(2, max_n + 1):
                for code in range(base ** n):
                    T = bytes((code // (base ** i)) % base for i in range(n))
                    if len(set(T)) < 2: continue
                    L, p = mt.bwt_encode(T)
                    # Test 4 distinct orders: fifo, lifo, narrow_first, wide_first
                    # Compare final core canonical form (sorted (lo, hi, delta, word_bytes)) AND q_core!
                    cores = []
                    qcs = []
                    for pol in ("fifo", "lifo", "narrow_first", "wide_first"):
                        strips, ws, cp, st = peel_k1_algebra(L, p, mode=mode, order_policy=pol)
                        verify_strips_exact(strips, ws, n, T, cp)
                        tot_verified += 1
                        live = tuple(sorted((s[0], s[1], s[2], ws.expand(s[3])) for s in strips if s[5]))
                        cores.append(live)
                        qcs.append(st["q_core"])
                    checked += 1
                    if len(set(cores)) > 1:
                        # Ignore pure single-cycle rotation difference when q_core == 1!
                        if not (all(q == 1 for q in qcs) and len(set(qcs)) == 1):
                            if len(non_conf) < 3:
                                non_conf.append((list(T), qcs, [len(c) for c in cores]))
                        if len(set(qcs)) > 1 and len(non_conf_q) < 3:
                            non_conf_q.append((list(T), qcs))
        print(f"  Mode {mode}: verified {checked*4} runs exact | "
              f"non-confluent (ignoring q=1 rot): {len(non_conf)} (diff q_core: {len(non_conf_q)})")
        if non_conf:
            print(f"    Minimal non-confluence witness: T={non_conf[0][0]}, q_cores(fifo/lifo/nar/wid)={non_conf[0][1]}")
        if non_conf_q:
            print(f"    Different q_core witness:       T={non_conf_q[0][0]}, q_cores={non_conf_q[0][1]}")
    print(f"  TOTAL EXHAUSTIVE CHECKS PASSED: {tot_verified} [verified]")


def test_adversarial_killers():
    print("\n=== 2. Adversarial Killers (3-phase, AltRing, Padded-Ring, SymMidRun, Fib_20+$) ===")
    cases = []
    # 1. AltRing (k=64)
    cases.append(("AltRing(k=64)", b"ab" * 64 + b"bc" * 64 + b"ca" * 64))
    # 2. 3-phase (abc)^k + delimiter (or 3-phase non-trivial: (ab)^k (bc)^k (ca)^k + (ac)^k)
    cases.append(("3-phase(k=64)", b"abc" * 64 + b"abd" * 64 + b"acd" * 64))
    # 3. Low-r Padded-Ring: e.g. M^64 A M^64 B M^64 C (constant r, large n)
    cases.append(("PaddedRing(K=64)", b"M" * 64 + b"A" + b"M" * 64 + b"B" + b"M" * 64 + b"C"))
    # 4. Minimal SymMidRun T_K = M^K A M^K Z (K=128)
    cases.append(("SymMidRun(K=128)", b"M" * 128 + b"A" + b"M" * 128 + b"Z"))
    # 5. Fib_20 + $
    a_f, b_f = [0], [0, 1]
    for _ in range(18): a_f, b_f = b_f, b_f + a_f
    cases.append(("Fib_20+$", bytes(b_f + [2])))

    for name, T in cases:
        L, p = mt.bwt_encode(T)
        parts = []
        for mode in ("M1", "M2", "M3", "M4"):
            strips, ws, cp, st = peel_k1_algebra(L, p, mode=mode, order_policy="fifo")
            verify_strips_exact(strips, ws, len(T), T, cp)
            parts.append(f"{mode}:q={st['q_core']}/mth={st['m_core_thin']}/ops={st.get('total_ops',0)}/sp={st.get('spirals',0)}")
        print(f"  {name:18s} (n={len(T):5d}, r0={st['r0']:3d}, m16={st['m16']:3d}) | " + " | ".join(parts))


def test_wikidata_6blocks():
    print("\n=== 3. EXACT 6-BLOCK CENSUS OF THE kappa=1 2-CORE (M1, M2, M3, M4) ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    agg = {m: defaultdict(float) for m in ("M1", "M2", "M3", "M4")}
    for entry in manifest["blocks"]:
        with open(f"/tmp/mibz2-blocks/{entry['file']}", "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        p = entry["primary_index"]
        ref = mt.ref_ibwt(L, p)
        print(f"\n--- {entry['file']} (n={entry['n']}, r0={entry['r']}) ---")
        for mode in ("M1", "M2", "M3", "M4"):
            t0 = time.perf_counter()
            strips, ws, cp, st = peel_k1_algebra(L, p, mode=mode, order_policy="fifo")
            dt = time.perf_counter() - t0
            verify_strips_exact(strips, ws, entry["n"], ref, cp)
            q0 = st["q0"]; m16 = st["m16"]; r0 = st["r0"]
            ops = st.get("total_ops", 0)
            print(f"  {mode} | q0={q0:6d} r1_el={st['round1_eligible']:6d} -> q_core={st['q_core']:6d}({st['q_core']/q0:5.3f}q0) "
                  f"m_th={st['m_core_thin']:6d}({st['m_core_thin']/m16:5.3f}m16) rows={st['rows_core']:6d} "
                  f"| ops={ops:6d}({ops/r0:4.2f}r) wv={st['waves']:3d} "
                  f"| <=1:{st['pct_l1']:4.1f}% <=3:{st['pct_l3']:4.1f}% <=8:{st['pct_l8']:4.1f}% <=16:{st['pct_l16']:4.1f}% >16:{st['pct_gt16']:4.1f}% slp={st['slp_gt16']:5d} t={dt:.2f}s [EXACT]", flush=True)
            if entry["file"] == "block-0.L":
                print(f"       thin (in,out) deg hist: {sorted(st['deg_hist_thin'].items())}")
            a = agg[mode]
            a["r0"] += r0; a["q0"] += q0; a["m16"] += m16
            a["q_core"] += st["q_core"]; a["m_core_thin"] += st["m_core_thin"]
            a["rows_core"] += st["rows_core"]; a["total_ops"] += ops
            a["slp_gt16"] += st["slp_gt16"]; a["r1_el"] += st["round1_eligible"]

    print("\n=== 4. 6-BLOCK SUM SUMMARY (r0_sum=509,269, m16_sum=895,391) ===")
    for mode in ("M1", "M2", "M3", "M4"):
        a = agg[mode]
        r0 = a["r0"]; q0 = a["q0"]; m16 = a["m16"]
        print(f"  {mode} | q_core={int(a['q_core']):7d} ({a['q_core']/q0:5.3f}q0) | "
              f"m_core_thin={int(a['m_core_thin']):7d} ({a['m_core_thin']/m16:5.3f}m16) | "
              f"rows_core={int(a['rows_core']):7d} | ops={int(a['total_ops']):7d} ({a['total_ops']/r0:4.2f}r) | "
              f"ops+m_th={int(a['total_ops']+a['m_core_thin']):7d} ({(a['total_ops']+a['m_core_thin'])/m16:5.3f}m16) | "
              f"slp>16={int(a['slp_gt16']):6d}")


if __name__ == "__main__":
    test_confluence_and_exhaustive()
    test_adversarial_killers()
    test_wikidata_6blocks()
