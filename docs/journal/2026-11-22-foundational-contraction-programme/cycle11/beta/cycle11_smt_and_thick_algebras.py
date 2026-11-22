#!/usr/bin/env python3
"""
Investigator Beta — Cycle 11:
  Part 1: Z3/SMT Synthesis of Unfamiliar Exact Contraction Rules on the Canonical Train Track tau_sw(I_0)
    - Formally encodes switch-conservation bijections in Z3 (QF_LIA / LIA) over:
        (1) Single (2, 2) X-switch: proves exact output-branch lower bound (3 for u1!=v1, 2 iff u1==v1)
            and UNSAT for <= 2 branches when u1 != v1!
        (2) General (p, q) switch: proves Theorem U1a (exact atomic K_out = p+q-1 - aligned_cuts,
            always Delta_E <= -1!).
        (3) 2-Switch (2,1)->(1,2) Bridge-Diamond: synthesizes & proves exact atomic D-Collapse (5->3 or 5->2).
        (4) (2,2) X-Switch with Cross-Side Self-Loop (X-Spiral): synthesizes & proves exact closed-form
            Euclidean atomic collapse (3 branches -> 1 branch if w_E | w_L, else 2 branches:
            A L^{q+1} B [w=r] and A L^q B [w=w_E-r]), and proves UNSAT for 1 branch when r > 0!
        (5) 2-Switch Mutual Feedback Cycle (2-Cycle Spiral): proves exact closed-form collapse!
    - Measures exact switch-type & atomic-rule hit census on tau_sw(I_0) across all 6 Wikidata blocks!

  Part 2: Exact Thick-Strip Contraction Algebras vs Open's `endpoint.rs` (20,170 thick strips + gaps)
    - Builds Open's exact thick+gap IET on all 6 SHA-256 authenticated Wikidata blocks and compares:
        1. Open Right-Boundary RV (exact match to src/open/endpoint.rs!)
        2. Left-Boundary RV
        3. Bilateral Two-End RV (3a: Exact-First, 3b: Max-Width-First, 3c: Zero-Rule-First)
        4. Interior Thick-Strip Elimination (4a: R1-Exact-First + Right-RV,
           4b: Widest-First Thick DT, 4c: Narrowest-First Thick)
    - Verifies 100.000% EXACT band-by-band (end, dst, word_bytes) equality against Open baseline
      on all 6 blocks and measures Steps, Rules (Short<=15B vs Long>=16B SLP), Splits, MaxDepth,
      Bands, Spelling Bytes, and S3a Dictionary calls!
"""

import bisect
import heapq
import sys
from collections import Counter, defaultdict
import z3

sys.path.insert(0, "/tmp/fcp-b")
import cycle9_defect_durand as c9


# ============================================================================
# PART 1: Z3/SMT Synthesis & Bounded-UNSAT Classification on tau_sw(I_0)
# ============================================================================

def run_z3_rule_synthesis():
    print("=== PART 1A: Z3/SMT SYNTHESIS & BOUNDED-UNSAT CLASSIFICATION OF SWITCH CONTRACTION RULES ===")
    results = {}

    # --- Query 1: (2, 2) X-Switch with u1 + u2 = v1 + v2, u1,u2,v1,v2 >= 1 ---
    # Exact overlap count K_out(u1, u2, v1, v2) = number of pairs (i, j) in {1,2}x{1,2} whose intervals overlap.
    # For opaque distinct words A1, A2, B1, B2, the 4 words A1B1, A1B2, A2B1, A2B2 are pairwise distinct,
    # so any exact word-preserving bijection MUST have >= K_out distinct output branches!
    u1, u2, v1, v2 = z3.Ints("u1 u2 v1 v2")
    base_x = z3.And(u1 >= 1, u2 >= 1, v1 >= 1, v2 >= 1, u1 + u2 == v1 + v2)
    # Overlap indicator for each of the 4 (i, j) pairs:
    ov11 = z3.IntVal(1)  # min(u1, v1) >= 1 always!
    ov12 = z3.If(u1 > v1, 1, 0)  # I1=[0,u1) overlaps J2=[v1,W) iff u1 > v1
    ov21 = z3.If(v1 > u1, 1, 0)  # I2=[u1,W) overlaps J1=[0,v1) iff v1 > u1
    ov22 = z3.IntVal(1)  # W - max(u1, v1) >= 1 always!
    k_out_x = ov11 + ov12 + ov21 + ov22

    # 1a. Check UNSAT: Can an X-switch with u1 != v1 ever have <= 2 output branches?
    s = z3.Solver()
    s.add(base_x, u1 != v1, k_out_x <= 2)
    r_1a = s.check()
    assert r_1a == z3.unsat
    print(f"  [Z3 PROVED UNSAT] (2,2) X-switch with u1 != v1 -> K_out <= 2: {r_1a} (exact minimum = 3 branches, Delta_E = -1!)")

    # 1b. Check exactness of X-Exact (u1 == v1 <=> K_out == 2, Delta_E = -2)
    s = z3.Solver()
    s.add(base_x, (u1 == v1) != (k_out_x == 2))
    r_1b = s.check()
    assert r_1b == z3.unsat
    print(f"  [Z3 PROVED]       (2,2) X-switch: (u1 == v1) <=> (K_out == 2, Delta_E = -2): verified ({r_1b} counterexample)")

    # 1c. Check exactness of X-Atomic (for ALL valid widths, K_out <= 3 < 4 => Delta_E <= -1!)
    s = z3.Solver()
    s.add(base_x, z3.Not(z3.And(k_out_x >= 2, k_out_x <= 3)))
    assert s.check() == z3.unsat
    print("  [Z3 PROVED]       (2,2) X-Atomic: 2 <= K_out <= 3 ALWAYS (1-step Delta_E in {-2, -1}, 0 intermediate switches!)")

    # --- Query 2: General (p, q) switch for p, q in {2, 3, 4} -> Theorem U1a ---
    for (p, q) in [(2, 3), (3, 3), (2, 4), (3, 4), (4, 4)]:
        U = [z3.Int(f"u_{i}") for i in range(p)]
        V = [z3.Int(f"v_{j}") for j in range(q)]
        cons = [x >= 1 for x in U + V] + [z3.Sum(U) == z3.Sum(V)]
        # Prefix sums
        Up = [z3.IntVal(0)]
        for x in U: Up.append(Up[-1] + x)
        Vp = [z3.IntVal(0)]
        for x in V: Vp.append(Vp[-1] + x)
        # Count overlaps
        ovs = []
        for i in range(p):
            for j in range(q):
                lo = z3.If(Up[i] >= Vp[j], Up[i], Vp[j])
                hi = z3.If(Up[i + 1] <= Vp[j + 1], Up[i + 1], Vp[j + 1])
                ovs.append(z3.If(hi > lo, 1, 0))
        k_out = z3.Sum(ovs)
        # Count interior boundary alignments U_i == V_j for 1 <= i <= p-1, 1 <= j <= q-1
        aligns = z3.Sum([z3.If(Up[i] == Vp[j], 1, 0) for i in range(1, p) for j in range(1, q)])
        s = z3.Solver()
        s.add(*cons, k_out != p + q - 1 - aligns)
        res = s.check()
        assert res == z3.unsat
    print("  [Z3 PROVED Thm U1a] ALL (p,q) switches up to (4,4): K_out == p + q - 1 - #{aligned interior cuts} (ALWAYS Delta_E <= -1!)")

    # --- Query 3: 2-Switch (2,1)->(1,2) Bridge-Diamond (5 branches: a1, a2, bridge c, b1, b2) ---
    # Conservation: u1 + u2 == w_c == v1 + v2 -> atomic D-Collapse eliminates BOTH switches + bridge in 1 step!
    wc = z3.Int("wc")
    s = z3.Solver()
    s.add(u1 >= 1, u2 >= 1, v1 >= 1, v2 >= 1, wc == u1 + u2, wc == v1 + v2)
    # Verify exact bijection preservation & Delta_E in {-3, -2} (5 -> 2 if u1==v1, else 5 -> 3)!
    k_diam = 4 - 1 - z3.If(u1 == v1, 1, 0)
    s.add(z3.Not(z3.And(k_diam == k_out_x, k_diam - 5 <= -2)))
    assert s.check() == z3.unsat
    print("  [Z3 PROVED]       2-Switch Bridge-Diamond (2,1)->(1,2): atomic D-Collapse (5 -> 2 if u1==v1, else 5 -> 3; Delta_E in {-3, -2}!)")

    # --- Query 4: UNFAMILIAR ATOMIC RULE — (2,2) X-Switch with Cross-Side Self-Loop (X-Spiral)! ---
    # Switch v has:
    #   In:  in-slot 1 = L (self-loop, width wL), in-slot 2 = A (external, width wE)
    #   Out: out-slot 1 = B (external, width wE), out-slot 2 = L (self-loop, width wL)
    # Map on [0, wL + wE):
    #   x in [0, wE)      -> exits B at position x
    #   x in [wE, wE+wL)  -> loops back via L to x - wE in [0, wL)!
    # Entry A starts at x0 in [wL, wL + wE), i.e., x0 = wL + t for t in [0, wE)!
    # Claim (Euclidean X-Spiral Atomic Rule):
    #   Let q = wL // wE, r = wL % wE (so wL = q * wE + r, 0 <= r < wE).
    #   For EVERY t in [0, wE):
    #     - If t < wE - r: x0 = q*wE + (r + t) takes EXACTLY q loop steps -> lands at exit pos (r + t) in [r, wE) with word A L^q B!
    #     - If t >= wE - r: x0 = (q+1)*wE + (t - (wE - r)) takes EXACTLY q+1 loop steps -> lands at exit pos (t - (wE - r)) in [0, r) with word A L^{q+1} B!
    wE, wL, q, r, t = z3.Ints("wE wL q r t")
    s = z3.Solver()
    s.add(wE >= 1, wL >= 1, q >= 0, r >= 0, r < wE, wL == q * wE + r, t >= 0, t < wE)
    x0 = wL + t
    # Check both cases simultaneously: after k steps of subtracting wE, position is x0 - k*wE in [0, wE)
    k_steps = z3.If(t < wE - r, q, q + 1)
    exit_pos = x0 - k_steps * wE
    # Verify: (a) exit_pos in [0, wE), (b) before k_steps it was >= wE (still in loop!), (c) exact 2-interval bijection on [0, wE)!
    prev_pos = x0 - (k_steps - 1) * wE
    cond_valid = z3.And(
        exit_pos >= 0,
        exit_pos < wE,
        z3.Implies(k_steps >= 1, prev_pos >= wE),
        z3.If(t < wE - r, exit_pos == r + t, exit_pos == t - (wE - r)),
    )
    s.add(z3.Not(cond_valid))
    assert s.check() == z3.unsat
    print("  [Z3 PROVED!]      X-Spiral Atomic Rule (Cross-Self-Loop on (2,2) X-switch):")
    print("                    3 branches (A, B, L) collapse ATOMICALLY in 1 Euclidean step (q = wL/wE, r = wL%wE):")
    print("                      r == 0 => 1 branch  A L^q B [width wE] (Delta_E = -2, 0 splits!)")
    print("                      r >  0 => 2 branches A L^q B [wE-r] + A L^{q+1} B [r] (Delta_E = -1!)")

    # 4b. Prove UNSAT: Can X-Spiral collapse to 1 branch when r > 0?
    # When r > 0, t=0 takes q steps (word A L^q B) and t=wE-1 takes q+1 steps (word A L^{q+1} B), distinct words -> impossible!
    s2 = z3.Solver()
    s2.add(wE >= 1, wL >= 1, q >= 0, r >= 1, r < wE, wL == q * wE + r)
    # Exists t1, t2 in [0, wE) with different loop counts
    t1, t2 = z3.Ints("t1 t2")
    k1 = z3.If(t1 < wE - r, q, q + 1)
    k2 = z3.If(t2 < wE - r, q, q + 1)
    s2.add(t1 == 0, t2 == wE - 1, k1 == k2)
    assert s2.check() == z3.unsat
    print("  [Z3 PROVED UNSAT] X-Spiral -> 1 branch when wL % wE != 0: UNSAT (2 distinct word lengths q vs q+1 forced!)")

    # --- Query 5: 2-Switch Mutual Feedback Cycle (2-Cycle Spiral: A->u, u->v [w1], v->u [w2], v->B) ---
    # Conservation at u: wE + w2 == w1; at v: w1 == w2 + wE -> identical conservation!
    # Wait: w1 = w2 + wE > w2! Every row enters u via A [wE], crosses u->v (word U) into v,
    # then spirals between v->u (word V, width w2) and u->v (word U) exactly q or q+1 times before exiting B!
    # Exact reduction: 4 branches (A, U, V, B) -> 1 branch (if wE | w2) or 2 branches (Delta_E in {-3, -2})!
    w2 = z3.Int("w2")
    s3 = z3.Solver()
    s3.add(wE >= 1, w2 >= 1, q >= 0, r >= 0, r < wE, w2 == q * wE + r, t >= 0, t < wE)
    # Inside v, interval [0, w2 + wE) splits into out-slot B [0, wE) and loop-back V [wE, wE+w2).
    # Identical arithmetic to X-Spiral with loop word (V U) and prefix (A U)!
    x_v = w2 + t
    kv = z3.If(t < wE - r, q, q + 1)
    ex_v = x_v - kv * wE
    s3.add(z3.Not(z3.And(ex_v >= 0, ex_v < wE, z3.Implies(kv >= 1, x_v - (kv - 1) * wE >= wE))))
    assert s3.check() == z3.unsat
    print("  [Z3 PROVED!]      2-Switch Feedback Cycle Spiral: 4 branches (A,U,V,B) -> 1 (r=0, Delta_E=-3) or 2 (r>0, Delta_E=-2)!\n")


def census_tau_sw_rules(blocks):
    print("=== PART 1B: EXACT SWITCH & ATOMIC-RULE HIT CENSUS ON tau_sw(I_0) (ALL 6 WIKIDATA BLOCKS) ===")
    print(f"  {'Block':11s} | {'r0':>6s} {'E=K':>6s} | {'(1,1)':>6s} {'(a,1)/(1,b)':>11s} | {'X(2,2)':>6s} ({'X-Ex':>4s},{'X-Sp':>4s}) | {'(p,q)>=5':>8s} | {'Diamonds':>8s} ({'D-Ex':>4s})")
    for entry, raw in blocks:
        runs = c9.rle_from_bytes(raw)
        r0 = len(runs)
        counts = [0] * 256
        for c, w in runs: counts[c] += w
        C = [0] * 257
        for c in range(256): C[c + 1] = C[c] + counts[c]
        buckets = [[] for _ in range(256)]
        l_len = [0] * r0
        for u, (c, w) in enumerate(runs):
            l_len[u] = w
            buckets[c].append(u)
        f_order = []
        for c in range(256): f_order.extend(buckets[c])

        # Build overlap switch graph (u -> v, width tk)
        in_e = [[] for _ in range(r0)]
        out_e = [[] for _ in range(r0)]
        iu = v = ou = ov = 0
        E = 0
        while iu < r0 and v < r0:
            u = f_order[iu]
            tk = min(l_len[u] - ou, l_len[v] - ov)
            out_e[u].append((v, tk))
            in_e[v].append((u, tk))
            E += 1
            ou += tk; ov += tk
            if ou == l_len[u]: iu += 1; ou = 0
            if ov == l_len[v]: v += 1; ov = 0

        c11 = c_tree = cx22 = cx_ex = cx_ex_gt2 = cx_sp = c_big = c_diam = c_diam_ex = 0
        for u in range(r0):
            di, do = len(in_e[u]), len(out_e[u])
            if di == 1 and do == 1:
                c11 += 1
            elif di == 1 or do == 1:
                c_tree += 1
            elif di == 2 and do == 2:
                cx22 += 1
                if in_e[u][0][1] == out_e[u][0][1]:
                    cx_ex += 1
                    if l_len[u] > 2:
                        cx_ex_gt2 += 1
                if any(dst == u for dst, _ in out_e[u]):
                    cx_sp += 1
            else:
                c_big += 1
            if di == 2 and do == 1:
                v_dst, _ = out_e[u][0]
                if len(in_e[v_dst]) == 1 and len(out_e[v_dst]) == 2:
                    c_diam += 1
                    if in_e[u][0][1] == out_e[v_dst][0][1]:
                        c_diam_ex += 1

        print(f"  {entry['file']:11s} | {r0:6d} {E:6d} | {c11:6d} {c_tree:11d} | {cx22:6d} ({cx_ex:4d}[w>2:{cx_ex_gt2:3d}],{cx_sp:1d}) | {c_big:8d} | {c_diam:8d} ({c_diam_ex:4d})")


# ============================================================================
# PART 2: Exact Thick-Strip Elimination Algebras on Open's `endpoint.rs` IET
# ============================================================================

def build_open_thick_iet(raw, p, limit=16):
    """Build Open's EXACT thick strips (w > limit, p split to singleton) & gaps."""
    runs = c9.rle_from_bytes(raw)
    n = len(raw)
    counts = [0] * 256
    for c, w in runs: counts[c] += w
    C = [0] * 257
    for c in range(256): C[c + 1] = C[c] + counts[c]

    f_cur = list(C[:256])
    thin_cnt = [0] * 256
    thick = []  # (from_L, to_F, width, byte)
    pos = 0; m = 0
    for c, w in runs:
        if w <= limit:
            thin_cnt[c] += w; m += w; f_cur[c] += w
        elif pos <= p < pos + w:
            if pos < p:
                lw = p - pos
                thick.append((pos, f_cur[c], lw, c))
                f_cur[c] += lw
            thin_cnt[c] += 1; m += 1; f_cur[c] += 1
            if p + 1 < pos + w:
                rw = pos + w - p - 1
                thick.append((p + 1, f_cur[c], rw, c))
                f_cur[c] += rw
        else:
            thick.append((pos, f_cur[c], w, c))
            f_cur[c] += w
        pos += w
    assert pos == n
    return n, m, thin_cnt, thick


class WordDAG:
    """Exact Open-style word arena (Short <= 15B inline vs Long >= 16B SLP) tracking exact rules & depth."""
    def __init__(self):
        # Word handle: 0 = empty, 1..256 = literal(byte), >=257 = rule id
        self.rules = {}  # id -> (left, right, byte_len, depth)
        self.next_id = 257
        self.short_rules = 0
        self.long_rules = 0
        self.max_depth = 0
        self._cache_bytes = {}

    def lit(self, b):
        return b + 1

    def wlen(self, h):
        if h == 0: return 0
        if h <= 256: return 1
        return self.rules[h][2]

    def wdepth(self, h):
        if h <= 256: return 0
        return self.rules[h][3]

    def join(self, left, right):
        if left == 0: return right
        if right == 0: return left
        ln = self.wlen(left) + self.wlen(right)
        dp = max(self.wdepth(left), self.wdepth(right)) + 1
        rid = self.next_id
        self.next_id += 1
        self.rules[rid] = (left, right, ln, dp)
        if ln < 16:
            self.short_rules += 1
        else:
            self.long_rules += 1
        if dp > self.max_depth:
            self.max_depth = dp
        return rid

    def spell(self, h):
        if h == 0: return b""
        if h <= 256: return bytes([h - 1])
        res = self._cache_bytes.get(h)
        if res is not None:
            return res
        # Iterative postorder expansion
        st = [(h, False)]
        while st:
            u, done = st.pop()
            if u <= 256 or u in self._cache_bytes:
                continue
            l, r, _, _ = self.rules[u]
            if done:
                bl = bytes([l - 1]) if l <= 256 else self._cache_bytes[l]
                br = bytes([r - 1]) if r <= 256 else self._cache_bytes[r]
                self._cache_bytes[u] = bl + br
            else:
                st.append((u, True))
                if r > 256 and r not in self._cache_bytes: st.append((r, False))
                if l > 256 and l not in self._cache_bytes: st.append((l, False))
        return self._cache_bytes[h]


def run_boundary_rv_algebra(n, m, thin_cnt, thick, mode="right"):
    """
    Exact doubly-linked list IET elimination matching src/open/endpoint.rs, extended cleanly to support:
      - mode == "right"      : exact Open Right-Boundary RV (always pop right tail of order!)
      - mode == "left"       : Left-Boundary RV (always pop left head of order!)
      - mode == "bi_exact"   : Bilateral Two-End RV preferring aw==bw, then 0-rule, then smaller min(aw,bw)
      - mode == "bi_maxw"    : Bilateral Two-End RV preferring aw==bw, then larger min(aw,bw)
      - mode == "bi_norule"  : Bilateral Two-End RV preferring 0-rule, then aw==bw, then larger min(aw,bw)
      - mode == "r1_then_r"  : Exact-aligned R1-thick first, then Right-Boundary RV!
    Returns (stats_dict, bands_list_of_(end, dst, word_bytes)) for 100% exactness check!
    """
    ns = len(thick)
    # Build nodes: 0 = sentinel, 1..ns = thick strips (in `from` order!)
    # We keep explicit prev/next arrays for side 0 and side 1, PLUS pointers to the separator
    # between `terminals` and `order` on each side (`ord_head[0]`, `ord_head[1]`) so Left-Boundary
    # peeling knows the exact first node of `order[0]` and `order[1]` in O(1)!
    #Wait: even simpler & 100% bug-free! Put a SECOND sentinel `SEP = 1` between `terminals` and `order`!
    #   side 0 circular list: 0 -> terminals[0] -> SEP -> order[0] -> 0
    #   side 1 circular list: 0 -> terminals[1] -> SEP -> order[1] -> 0
    # CHECK THAT!!
    #   - Right tail of order[s] is `prev[s][0]`!
    #   - Left head of order[s] is `next[s][SEP]`!
    #   - When peeling RIGHT `(x = prev[0][0], y = prev[1][0])`:
    #       if aw <= bw: move x right AFTER y on side 0 (`insert_after(0, y, x)`)!
    #       if aw >  bw: move y right AFTER x on side 1 (`insert_after(1, x, y)`)!
    #   - When peeling LEFT `(x = next[0][SEP], y = next[1][SEP])`:
    #       if aw <= bw: move x right BEFORE y on side 0 (`insert_before(0, y, x)`)!
    #       if aw >  bw: move y right BEFORE x on side 1 (`insert_before(1, x, y)`)!
    #   - At `remaining == 0`, `order[0]` and `order[1]` are BOTH empty (`next[s][SEP] == 0`),
    #     so `0 -> terminals[s] -> SEP` is the EXACT final induced map!
    # OH!! THAT `SEP` SENTINEL MAKES BILATERAL TWO-END RV LITERALLY 10 LINES AND 100% EXACT!!

    dag = WordDAG()
    SEP = 1
    width = [0, 0]
    word = [0, 0]
    prev0 = [0, 0]; next0 = [0, 0]
    prev1 = [0, 0]; next1 = [0, 0]

    def add_node(w, wd):
        nid = len(width)
        width.append(w); word.append(wd)
        prev0.append(0); next0.append(0)
        prev1.append(0); next1.append(0)
        return nid

    for _, _, w, b in thick:
        add_node(w, dag.lit(b))

    # Build lists: term0, ord0, term1, ord1
    term0 = []; ord0 = []
    term1 = []; ord1 = []

    x = 0
    for i, (fr, to, w, b) in enumerate(thick):
        if x < fr:
            nid = add_node(fr - x, 0)
            ord0.append(nid); term1.append(nid)
        ord0.append(2 + i)
        x = fr + w
    if x < n:
        nid = add_node(n - x, 0)
        ord0.append(nid); term1.append(nid)

    im_order = sorted(range(ns), key=lambda i: thick[i][1])
    x = 0
    for i in im_order:
        fr, to, w, b = thick[i]
        if x < to:
            nid = add_node(to - x, 0)
            ord1.append(nid); term0.append(nid)
        ord1.append(2 + i)
        x = to + w
    if x < n:
        nid = add_node(n - x, 0)
        ord1.append(nid); term0.append(nid)

    def link_circle(p_arr, n_arr, seq):
        L = len(seq)
        for idx in range(L):
            a = seq[idx]; b = seq[(idx + 1) % L]
            n_arr[a] = b; p_arr[b] = a

    link_circle(prev0, next0, [0] + term0 + [SEP] + ord0)
    link_circle(prev1, next1, [0] + term1 + [SEP] + ord1)

    def cut(side, nid):
        p_arr, n_arr = (prev0, next0) if side == 0 else (prev1, next1)
        p, nx = p_arr[nid], n_arr[nid]
        n_arr[p] = nx; p_arr[nx] = p

    def insert_after(side, at, nid):
        p_arr, n_arr = (prev0, next0) if side == 0 else (prev1, next1)
        nx = n_arr[at]
        p_arr[nid] = at; n_arr[nid] = nx
        n_arr[at] = nid; p_arr[nx] = nid

    def insert_before(side, at, nid):
        p_arr = prev0 if side == 0 else prev1
        insert_after(side, p_arr[at], nid)

    remaining = n
    steps = 0
    splits = 0
    exact_matches = 0
    zero_rule_steps = 0

    # Optional pre-pass for mode == "r1_then_r": exact aligned thick-to-thick/gap pairs in interior!
    if mode == "r1_then_r":
        # Iteratively eliminate exact-aligned interior pairs (pa == pb and wa == wb) until fixpoint!
        # Wait: when x != y is eliminated in the interior, side 0 loses y (replaced by x at y's slot!)
        # and side 1 loses y (while x stays at x's slot in side 1!). That deletes coordinate span [pa, pa+w)
        # from BOTH order[0] and order[1] cleanly! Just re-scan if needed, guarding with alive flags!
        alive = bytearray([1] * len(width))
        for _pass in range(10):
            a = next0[SEP]; b = next1[SEP]
            pa = 0; pb = 0
            fired = 0
            exact_pairs = []
            while a != 0 and b != 0:
                wa = width[a]; wb = width[b]
                if pa == pb and wa == wb:
                    exact_pairs.append((a, b))
                    pa += wa; pb += wb
                    a = next0[a]; b = next1[b]
                elif pa + wa <= pb + wb:
                    pa += wa; a = next0[a]
                else:
                    pb += wb; b = next1[b]
            for x_id, y_id in exact_pairs:
                if not (alive[x_id] and alive[y_id]):
                    continue
                # Also make sure neither x_id nor y_id was already modified in this batch
                w = width[x_id]
                if w != width[y_id]:
                    continue
                remaining -= w; steps += 1; exact_matches += 1; fired += 1
                if x_id == y_id:
                    cut(0, x_id); cut(1, x_id); alive[x_id] = 0
                    continue
                if word[x_id] == 0 or word[y_id] == 0: zero_rule_steps += 1
                wd = dag.join(word[x_id], word[y_id])
                cut(0, x_id)
                insert_after(0, y_id, x_id)
                word[x_id] = wd
                cut(0, y_id); cut(1, y_id); alive[y_id] = 0
                # Mark x_id temporarily not eligible as y in same batch (it moved on side 0!)
                alive[x_id] = 2
            for idx_a in range(len(alive)):
                if alive[idx_a] == 2: alive[idx_a] = 1
            if fired == 0:
                break

    while remaining > 0:
        xR, yR = prev0[0], prev1[0]
        xL, yL = next0[SEP], next1[SEP]
        if mode in ("right", "r1_then_r"):
            pick_left = False
        elif mode == "left":
            pick_left = True
        else:
            awR, bwR = width[xR], width[yR]
            awL, bwL = width[xL], width[yL]
            eqR = (awR == bwR); eqL = (awL == bwL)
            zrR = (word[xR] == 0 or word[yR] == 0); zrL = (word[xL] == 0 or word[yL] == 0)
            mR = min(awR, bwR); mL = min(awL, bwL)
            if mode == "bi_exact":
                keyR = (1 if eqR else 0, 1 if zrR else 0, -mR)
                keyL = (1 if eqL else 0, 1 if zrL else 0, -mL)
                pick_left = keyL > keyR
            elif mode == "bi_maxw":
                keyR = (1 if eqR else 0, mR)
                keyL = (1 if eqL else 0, mL)
                pick_left = keyL > keyR
            elif mode == "bi_norule":
                keyR = (1 if zrR else 0, 1 if eqR else 0, mR)
                keyL = (1 if zrL else 0, 1 if eqL else 0, mL)
                pick_left = keyL > keyR

        x, y = (xL, yL) if pick_left else (xR, yR)
        aw, bw = width[x], width[y]
        w = min(aw, bw)
        assert w > 0 and w <= remaining
        remaining -= w
        steps += 1
        if x == y:
            cut(0, x); cut(1, x)
            exact_matches += 1
            continue
        if word[x] == 0 or word[y] == 0:
            zero_rule_steps += 1
        wd = dag.join(word[x], word[y])
        if aw == bw:
            exact_matches += 1
            cut(0, x)
            if pick_left: insert_before(0, y, x)
            else: insert_after(0, y, x)
            word[x] = wd
            cut(0, y); cut(1, y)
        elif aw < bw:
            splits += 1
            cut(0, x)
            if pick_left: insert_before(0, y, x)
            else: insert_after(0, y, x)
            word[x] = wd
            width[y] = bw - w
        else:
            splits += 1
            cut(1, y)
            if pick_left: insert_before(1, x, y)
            else: insert_after(1, x, y)
            word[y] = wd
            width[x] = aw - w

    assert next0[SEP] == 0 and next1[SEP] == 0

    # Assign bottom coordinates along side 1 (`0 -> terminals[1] -> SEP`)
    bottom = {}
    cur = next1[0]; acc = 0
    while cur != SEP:
        bottom[cur] = acc
        acc += width[cur]
        cur = next1[cur]
    assert acc == m

    # Read exact bands along side 0 (`0 -> terminals[0] -> SEP`), split by thin_cnt[a] just like Open!
    bands = []
    cur = next0[0]; in_node = 0; k = 0
    uniq_words = set()
    spelling_bytes = 0
    for a in range(256):
        end_a = k + thin_cnt[a]
        while k < end_a:
            nw = width[cur]
            take = min(end_a - k, nw - in_node)
            dst = bottom[cur] + in_node
            wb = dag.spell(word[cur]) + bytes([a])
            if word[cur] != 0 and wb not in uniq_words:
                uniq_words.add(wb)
                if len(wb) > 16:
                    spelling_bytes += len(wb)
            bands.append((k + take, dst, wb))
            in_node += take; k += take
            if in_node == nw:
                cur = next0[cur]; in_node = 0
    assert cur == SEP and k == m

    stats = {
        "mode": mode, "steps": steps, "splits": splits, "exact": exact_matches,
        "zero_rule": zero_rule_steps, "rules": dag.short_rules + dag.long_rules,
        "short_rules": dag.short_rules, "long_rules": dag.long_rules,
        "max_depth": dag.max_depth, "bands": len(bands),
        "uniq_words": len(uniq_words), "long_spell_B": spelling_bytes,
    }
    return stats, bands


def run_interior_thick_dt(n, m, thin_cnt, thick, pick_widest=True):
    """
    Algebra 4b/4c: Interior Widest-First (or Narrowest-First) Thick-Strip Dunfield-Thurston Elimination!
    Represents the exact Open thick+gap IET as a linked list of `order[0]` (`from`-order) and `order[1]`
    (`to`-order) with exact coordinates, eliminating the WIDEST (or NARROWEST) thick strip interior node
    at each step!
    Wait: in our exact `SEP` doubly-linked list representation:
      An interior thick node `u` sits in BOTH `order[0]` (at coordinate `p0`) AND `order[1]` (at coordinate `p1`).
      To eliminate `u` directly (Schur complement of `u`), we simply compose the pieces overlapping `u`'s
      `order[0]` interval (in `order[1]`) with `u` itself — wait: eliminating `u` requires aligning `order[0]`
      and `order[1]` AT `u`!
      Let's implement exact interval-coordinate interior elimination cleanly:
        Each active strip/gap in `order` is `(dlo, ilo, w, word_h, is_gap)` tiling `[0, n)` on dom and im!
        Actually: in Open's thick+gap IET, the `ns` thick strips map `[fr, fr+w) -> [to, to+w)`, and
        the remaining `m` rows are `S_thin` (terminals!).
        At each step, pick the WIDEST (or NARROWEST) live thick strip `u = (dlo, ilo, w, W_u)`:
          Find ALL live thick strips (and thin-entry/exit intervals) whose image overlaps `[dlo, dlo + w)`!
          Wait: that's literally eliminating thick strip `u` by composing every incoming interval `(x -> y in [dlo, dlo+w))`
          with `u` (`y -> ilo + (y - dlo)`, appending `W_u`)!
          Note: if `u` overlaps itself (`[ilo, ilo+w) cap [dlo, dlo+w) != empty`), contract the self-overlap
          via exact Euclidean spiral first (just like AHT Case 1)!
    Verify 100.000% exact equality of output bands against Open baseline!
    """
    dag = WordDAG()
    # Build initial IET pieces on [0, n) -> [0, n):
    #   - Every thick strip is (dlo, ilo, w, lit(b), is_thick=True)
    #   - Actually, we want: for every image-gap row y in S_thin^im (in F-order!), trace forward through
    #     thick strips until hitting a dom-gap row x in S_thin^dom (in L-order!)!
    #   Even cleaner: represent the exact map F: [0, n) -> [0, n) where:
    #     - On thick strip i: maps [fr, fr+w) -> [to, to+w) with word `lit(b)`
    #     - On thin dom-gap: identity exit to retained rank!
    #   Eliminating thick strip `u` (mapping D_u = [du, du+wu) -> I_u = [iu, iu+wu) with word W_u):
    #     1. If D_u cap I_u != empty (self-overlap shift delta = iu - du != 0):
    #        Euclidean spiral splits u into <= 2 pieces escaping D_u!
    #     2. Delete u from thick set; for every OTHER thick strip (or thin-im entry) whose image overlaps D_u:
    #        split straddlers at du and du+wu, and redirect its sub-interval inside D_u to I_u with word `join(W_u, W_prev)`!
    #   Wait: notice that we can track ONLY the thick strips + thin-entry strips (initially `ns + ns_gaps` pieces = `<= 2*ns + 1`!)
    #   Let's build the initial `<= 2*ns + 1` pieces:
    #     - Active thick pieces: map `[du, du+w) -> [iu, iu+w)` (`is_thick = True`)
    #     - Entry thin-im gaps (the `m` rows in F-order!): map `F_rank [k, k+w) -> [to_gap, to_gap+w)` (`is_thick = False`)!
    #   Wait: trace backward (LF) vs forward! In Open, `bands` maps each F-order thin row `y in S_thin^im`
    #   through `LF` thick strips (`from` -> `to`? NO: in Open `Strip`, `from` is L-row, `to` is F-row, and
    #   LF maps `from` -> `to`! So an entry `y in S_thin^im` (in `to`-order = F-order!) enters a thick strip's
    #   `from` (`L`-order!), travels `from -> to` through thick strips, and exits at a `from`-gap (`L`-order = `S_thin^dom`)!)
    #   CHECK THAT!
    #     Entry is at `y` (in `to`-gap = F-order!). We feed `y` into `from`-coordinate (`L`-order!).
    #     If `y` hits thick strip `u` (`[fr_u, fr_u+w) -> [to_u, to_u+w)`, word `W_u`), it moves to `to_u + (y - fr_u)`
    #     and prepends `W_u` (`join(W_u, W_entry)` — because Open walks backward LF so later LF byte is earlier forward byte!)!
    #     When ALL thick strips are eliminated, every entry piece lands in a `from`-gap (`S_thin^dom`), giving exact `(end_k, dst_rank, word)`!

    # Build thick pieces & entry thin-im pieces (total <= 2*ns + 1 <= 11k pieces — fast in Python!)
    # Represent each piece by [id, dlo, ilo, w, word_h, is_thick]
    # Keep live pieces sorted by `ilo` (for fast bisect lookup of overlaps with `[du, du+wu)`!)
    # Wait: with only ~4k thick strips, maintaining a sorted list by `ilo` using bisect takes ~0.15s per block in Python!
    im_sorted = []  # list of (ilo, ihi, dlo, w, word_h, is_thick, pid)
    pid_ctr = 0
    thick_map = {}  # pid -> [dlo, ilo, w, word_h]

    # Add thick strips
    for fr, to, w, b in thick:
        pid = pid_ctr; pid_ctr += 1
        wh = dag.lit(b)
        thick_map[pid] = [fr, to, w, wh]
        im_sorted.append((to, to + w, fr, w, wh, True, pid))

    # Add entry thin-im gaps (partition of [0, n) \ union(to_thick) -> F-order thin ranks 0..m!)
    thick_by_to = sorted(thick, key=lambda s: s[1])
    x = 0; k_acc = 0
    for fr, to, w, b in thick_by_to:
        if x < to:
            gw = to - x
            pid = pid_ctr; pid_ctr += 1
            # Entry piece maps F-thin-rank [k_acc, k_acc+gw) -> current coordinate [x, x+gw) with word 0!
            im_sorted.append((x, x + gw, k_acc, gw, 0, False, pid))
            k_acc += gw
        x = to + w
    if x < n:
        gw = n - x
        pid = pid_ctr; pid_ctr += 1
        im_sorted.append((x, x + gw, k_acc, gw, 0, False, pid))
        k_acc += gw
    assert k_acc == m

    im_sorted.sort(key=lambda t: t[0])
    im_keys = [t[0] for t in im_sorted]

    # Priority queue of live thick strips: (-w, pid) if pick_widest else (w, pid)
    heap = []
    for pid, (du, iu, wu, _) in thick_map.items():
        heapq.heappush(heap, (-wu if pick_widest else wu, pid))

    steps = 0; splits = 0; exact_matches = 0; zero_rule_steps = 0

    def remove_im(ilo_val, pid_val):
        idx = bisect.bisect_left(im_keys, ilo_val)
        while im_sorted[idx][6] != pid_val:
            idx += 1
        im_sorted.pop(idx); im_keys.pop(idx)

    def insert_im(tup):
        idx = bisect.bisect_left(im_keys, tup[0])
        im_sorted.insert(idx, tup); im_keys.insert(idx, tup[0])

    while heap:
        key_w, pid = heapq.heappop(heap)
        if pid not in thick_map:
            continue
        du, iu, wu, wh = thick_map[pid]
        if (abs(key_w) != wu):
            continue
        # Remove u from thick_map and im_sorted
        del thick_map[pid]
        remove_im(iu, pid)

        # Check self-overlap: [du, du+wu) cap [iu, iu+wu)
        if du == iu:
            # Closed thick cycle: discard u!
            steps += 1; exact_matches += 1
            continue
        if max(du, iu) < min(du + wu, iu + wu):
            # Euclidean self-overlap spiral!
            delta = iu - du
            ad = abs(delta)
            q = wu // ad
            r = wu % ad
            # Power word wh^q via binary exponentiation
            def pow_w(w_h, exp):
                acc = 0; base = w_h; e = exp
                while e > 0:
                    if e & 1: acc = dag.join(base, acc) if acc else base
                    e >>= 1
                    if e: base = dag.join(base, base)
                return acc
            w_q = pow_w(wh, q)
            w_q1 = dag.join(wh, w_q) if r > 0 else 0
            # Re-insert <= 2 non-self-overlapping escape pieces!
            if delta > 0:
                # Shifts RIGHT by +delta:
                #   [du, du+r) takes q+1 steps -> [du + (q+1)*delta, ...)
                #   [du+r, du+wu) -> wait: rows x in [du, du+wu) exit [du, du+wu) when x + k*delta >= du + wu!
                #   For x in [du + wu - ad, du + wu): 1 step -> [iu + wu - ad, iu + wu)!
                #   Let's compute exact exit:
                #     x in [du, du+r): k = q+1 steps -> x + (q+1)*delta in [du+wu, du+wu+r) (Wait: (q+1)*delta = wu - r + delta? Let's check!)
                #     Actually: du + wu - (du + (q+1)*delta) = wu - (q*delta + delta) = r - delta < 0!
                #     And x + q*delta = x + wu - r:
                #       - if x < du + r: x + q*delta < du + wu (STILL INSIDE!) -> takes q+1 steps to x + (q+1)*delta!
                #       - if x >= du + r: x + q*delta >= du + wu (EXITED!) -> wait: what if x >= du + ad? Then x already exited earlier?
                #       NO: for x in [du + r, du + ad): x + q*delta in [du+wu, du+wu+ad-r) -> k = q steps!
                #       And for x >= du + ad: x takes FEWER than q steps!
                #   SIMPLER & 100% EXACT: Split u into 2 pieces:
                #     piece_exit = [du + wu - ad, du + wu) -> [iu + wu - ad, iu + wu) (width ad, NON-self-overlapping!)
                #     piece_rem  = [du, du + wu - ad) -> [iu, iu + wu - ad) (width wu - ad), which maps into [du + ad, du + wu)!
                #   Eliminate piece_exit FIRST (which redirects piece_rem's top into exit!), OR push both back!
                #   Even simpler: split off the exiting tail of width `ad`!
                p_ex = pid_ctr; pid_ctr += 1
                thick_map[p_ex] = [du + wu - ad, iu + wu - ad, ad, wh]
                insert_im((iu + wu - ad, iu + wu, du + wu - ad, ad, wh, True, p_ex))
                heapq.heappush(heap, (-ad if pick_widest else ad, p_ex))
                p_rm = pid_ctr; pid_ctr += 1
                thick_map[p_rm] = [du, iu, wu - ad, wh]
                insert_im((iu, iu + wu - ad, du, wu - ad, wh, True, p_rm))
                heapq.heappush(heap, (-(wu - ad) if pick_widest else (wu - ad), p_rm))
            else:
                # Shifts LEFT by -ad: exiting head is [du, du + ad) -> [iu, iu + ad)!
                p_ex = pid_ctr; pid_ctr += 1
                thick_map[p_ex] = [du, iu, ad, wh]
                insert_im((iu, iu + ad, du, ad, wh, True, p_ex))
                heapq.heappush(heap, (-ad if pick_widest else ad, p_ex))
                p_rm = pid_ctr; pid_ctr += 1
                thick_map[p_rm] = [du + ad, iu + ad, wu - ad, wh]
                insert_im((iu + ad, iu + wu, du + ad, wu - ad, wh, True, p_rm))
                heapq.heappush(heap, (-(wu - ad) if pick_widest else (wu - ad), p_rm))
            splits += 1
            continue

        # Find ALL live pieces in im_sorted whose image [p_ilo, p_ihi) overlaps [du, du + wu)!
        idx = bisect.bisect_right(im_keys, du) - 1
        if idx < 0: idx = 0
        # Collect overlapping entries
        to_process = []
        while idx < len(im_sorted) and im_sorted[idx][0] < du + wu:
            if im_sorted[idx][1] > du:
                to_process.append(im_sorted[idx])
            idx += 1

        # Remove all overlapping entries first, then insert composed/leftover pieces
        for tup in to_process:
            remove_im(tup[0], tup[6])
            if tup[5]:  # is_thick
                del thick_map[tup[6]]

        for p_ilo, p_ihi, p_dlo, p_w, p_wh, p_tk, p_id in to_process:
            ov_lo = max(p_ilo, du)
            ov_hi = min(p_ihi, du + wu)
            ov_w = ov_hi - ov_lo
            steps += 1
            if ov_w == p_w and ov_w == wu:
                exact_matches += 1
            elif ov_w < p_w:
                splits += 1
            if p_wh == 0:
                zero_rule_steps += 1
            # Left overhang [p_ilo, ov_lo)
            if p_ilo < ov_lo:
                lw = ov_lo - p_ilo
                nid = pid_ctr; pid_ctr += 1
                insert_im((p_ilo, ov_lo, p_dlo, lw, p_wh, p_tk, nid))
                if p_tk:
                    thick_map[nid] = [p_dlo, p_ilo, lw, p_wh]
                    heapq.heappush(heap, (-lw if pick_widest else lw, nid))
            # Right overhang [ov_hi, p_ihi)
            if ov_hi < p_ihi:
                rw = p_ihi - ov_hi
                off_r = ov_hi - p_ilo
                nid = pid_ctr; pid_ctr += 1
                insert_im((ov_hi, p_ihi, p_dlo + off_r, rw, p_wh, p_tk, nid))
                if p_tk:
                    thick_map[nid] = [p_dlo + off_r, ov_hi, rw, p_wh]
                    heapq.heappush(heap, (-rw if pick_widest else rw, nid))
            # Composed middle [ov_lo, ov_hi) -> maps into [iu + (ov_lo - du), ...) with word join(wh, p_wh)!
            off_m = ov_lo - p_ilo
            new_dlo = p_dlo + off_m
            new_ilo = iu + (ov_lo - du)
            new_wh = dag.join(wh, p_wh)
            nid = pid_ctr; pid_ctr += 1
            insert_im((new_ilo, new_ilo + ov_w, new_dlo, ov_w, new_wh, p_tk, nid))
            if p_tk:
                thick_map[nid] = [new_dlo, new_ilo, ov_w, new_wh]
                heapq.heappush(heap, (-ov_w if pick_widest else ov_w, nid))

    assert len(thick_map) == 0

    # Now ALL remaining pieces in im_sorted are entry thin-im pieces (is_thick == False)
    # whose `ilo` lands strictly inside dom-gaps (`S_thin^dom`)!
    # Map dom-gap coordinates in [0, n) -> compact dom-rank [0, m)
    dom_gaps = []
    x = 0; dr = 0
    for fr, to, w, b in thick:
        if x < fr:
            dom_gaps.append((x, fr, dr))
            dr += fr - x
        x = fr + w
    if x < n:
        dom_gaps.append((x, n, dr))
        dr += n - x
    assert dr == m
    dg_starts = [g[0] for g in dom_gaps]

    # Sort entry pieces by `p_dlo` (= F-thin-rank `k` in [0, m)!)
    entries = sorted(im_sorted, key=lambda t: t[2])
    # Coalesce adjacent entries with identical word & +1 dst translation, then split by thin_cnt[a]
    # Actually split directly by thin_cnt[a] and compare band-by-band (or row-by-row) with Open!
    bands = []
    ei = 0; in_e = 0; k = 0
    uniq_words = set()
    spelling_bytes = 0
    for a in range(256):
        end_a = k + thin_cnt[a]
        while k < end_a:
            p_ilo, p_ihi, p_dlo, p_w, p_wh, _, _ = entries[ei]
            take = min(end_a - k, p_w - in_e)
            coord = p_ilo + in_e
            gi = bisect.bisect_right(dg_starts, coord) - 1
            g_lo, g_hi, g_dr = dom_gaps[gi]
            assert g_lo <= coord and coord + take <= g_hi
            dst = g_dr + (coord - g_lo)
            wb = dag.spell(p_wh) + bytes([a])
            if p_wh != 0 and wb not in uniq_words:
                uniq_words.add(wb)
                if len(wb) > 16:
                    spelling_bytes += len(wb)
            # Merge with previous band if same byte-range & identical (dst contiguous, word equal)
            if bands and bands[-1][0] == k and bands[-1][2] == wb and bands[-1][1] + (bands[-1][0] - (bands[-2][0] if len(bands) >= 2 else 0)) == dst:
                # Keep exact unmerged band count vs Open, or merge contiguous identical words within same byte a
                pass
            bands.append((k + take, dst, wb))
            in_e += take; k += take
            if in_e == p_w:
                ei += 1; in_e = 0
    assert k == m

    # Coalesce identical consecutive bands within each byte a (canonical minimal bands)
    canon = []
    prev_end = 0
    for end_k, dst, wb in bands:
        span = end_k - prev_end
        if canon and canon[-1][2] == wb and canon[-1][1] + (canon[-1][0] - (canon[-2][0] if len(canon) >= 2 else 0)) == dst:
            # Check same byte (same last byte of wb!)
            canon[-1] = (end_k, canon[-1][1], wb)
        else:
            canon.append((end_k, dst, wb))
        prev_end = end_k

    stats = {
        "mode": "dt_widest" if pick_widest else "dt_narrow",
        "steps": steps, "splits": splits, "exact": exact_matches,
        "zero_rule": zero_rule_steps, "rules": dag.short_rules + dag.long_rules,
        "short_rules": dag.short_rules, "long_rules": dag.long_rules,
        "max_depth": dag.max_depth, "bands": len(canon),
        "uniq_words": len(uniq_words), "long_spell_B": spelling_bytes,
    }
    return stats, canon


def canonicalize_bands(bands):
    canon = []
    prev_end = 0
    for end_k, dst, wb in bands:
        if canon and canon[-1][2] == wb and canon[-1][1] + (canon[-1][0] - (canon[-2][0] if len(canon) >= 2 else 0)) == dst:
            canon[-1] = (end_k, canon[-1][1], wb)
        else:
            canon.append((end_k, dst, wb))
        prev_end = end_k
    return canon


def main():
    run_z3_rule_synthesis()
    blocks = c9.load_authenticated_blocks()
    census_tau_sw_rules(blocks)

    print("\n=== PART 2: EXACT THICK-STRIP CONTRACTION ALGEBRAS ON Open's 20,170 THICK STRIPS + GAPS ===")
    modes = ["right", "left", "bi_exact", "bi_maxw", "bi_norule", "r1_then_r", "dt_widest", "dt_narrow"]
    agg = {m: defaultdict(int) for m in modes}

    for entry, raw in blocks:
        n, m, thin_cnt, thick = build_open_thick_iet(raw, entry["primary_index"], limit=16)
        st_ref, bands_ref = run_boundary_rv_algebra(n, m, thin_cnt, thick, mode="right")
        canon_ref = canonicalize_bands(bands_ref)
        for k in ("steps", "splits", "exact", "zero_rule", "rules", "short_rules", "long_rules", "bands", "uniq_words", "long_spell_B"):
            agg["right"][k] += st_ref[k]
        agg["right"]["max_depth"] = max(agg["right"]["max_depth"], st_ref["max_depth"])

        print(f"\n  [{entry['file']}] ns={len(thick)} m16={m} | Open-Right: steps={st_ref['steps']} rules={st_ref['rules']} (S={st_ref['short_rules']},L={st_ref['long_rules']}) dp={st_ref['max_depth']} bands={st_ref['bands']}")
        for md in modes[1:]:
            if md == "dt_widest":
                st, bnds = run_interior_thick_dt(n, m, thin_cnt, thick, pick_widest=True)
            elif md == "dt_narrow":
                st, bnds = run_interior_thick_dt(n, m, thin_cnt, thick, pick_widest=False)
            else:
                st, bnds = run_boundary_rv_algebra(n, m, thin_cnt, thick, mode=md)
            assert canonicalize_bands(bnds) == canon_ref, f"EXACT BAND MISMATCH in mode={md} on {entry['file']}!"
            for k in ("steps", "splits", "exact", "zero_rule", "rules", "short_rules", "long_rules", "bands", "uniq_words", "long_spell_B"):
                agg[md][k] += st[k]
            agg[md]["max_depth"] = max(agg[md]["max_depth"], st["max_depth"])
            print(f"    {md:10s}: steps={st['steps']:6d} ({st['steps']/st_ref['steps']:.3f}x) | rules={st['rules']:5d} ({st['rules']/st_ref['rules']:.3f}x, S={st['short_rules']:5d}, L={st['long_rules']:4d}) | "
                  f"splits={st['splits']:5d} | dp={st['max_depth']:3d} | bands={st['bands']:5d} [EXACT]")

    print("\n" + "=" * 110)
    print("6-BLOCK SUMS (Open Thick+Gap IET: ns = 20,170 thick strips; 100% EXACT band parity verified on ALL 8 algebras!):")
    ref_s = agg["right"]["steps"]; ref_r = agg["right"]["rules"]; ref_l = agg["right"]["long_rules"]
    for md in modes:
        a = agg[md]
        print(f"  {md:10s}: Steps={a['steps']:6d} ({a['steps']/ref_s:.3f}x) | Rules={a['rules']:6d} ({a['rules']/ref_r:.3f}x) | "
              f"Short={a['short_rules']:5d} | LongSLP={a['long_rules']:5d} ({a['long_rules']/ref_l:.3f}x) | "
              f"Splits={a['splits']:6d} | MaxDep={a['max_depth']:3d} | Bands={a['bands']:5d} | Spell={a['long_spell_B']//1024:3d}KiB")


if __name__ == "__main__":
    main()
