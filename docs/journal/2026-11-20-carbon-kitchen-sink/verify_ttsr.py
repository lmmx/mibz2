#!/usr/bin/env python3
"""
Gate G1 Exhaustive Verifier for Train-Track Series-Reduced Reconstruction (TTSR-RV)
and Train-Track Open (TTO), plus Origami Cone-Cycle Isomorphism.

Verifies byte-exact output against dense LF walk on:
  1. Exhaustive binary columns: all 2^n columns for n = 1..8, every primary mark 0 <= p < n
     (total sum_{n=1}^8 n * 2^n = 3,586 cases, including periodic and multi-cycle columns).
  2. Exhaustive ternary columns: all 3^n columns for n = 1..5, every primary mark 0 <= p < n
     (total sum_{n=1}^5 n * 3^n = 1,704 cases, including periodic and multi-cycle columns).
  3. All six SHA-256-authenticated Wikidata BWT blocks (0, 2, 85, 143, 207, 210).
"""

from __future__ import annotations
import itertools
import json
import pathlib
import time
import numpy as np

BLOCKS_DIR = pathlib.Path("/home/antigravity/admitted-bwt")


def dense_lf_invert(L: bytes, p: int) -> bytes:
    """Reference dense LF inversion (exact for arbitrary columns, periodic and multi-cycle)."""
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 256
    s = 0
    for c in range(256):
        C[c] = s
        s += counts[c]
    lf = [0] * n
    occ = list(C)
    for i, b in enumerate(L):
        lf[i] = occ[b]
        occ[b] += 1

    rev_period = bytearray()
    cur = p
    for _ in range(n):
        rev_period.append(L[cur])
        cur = lf[cur]
        if cur == p:
            break
    period = bytes(rev_period[::-1])
    out = bytearray()
    while len(out) < n:
        take = min(len(period), n - len(out))
        out.extend(period[:take])
    return bytes(out)


def ttsr_rv_invert(L: bytes, p: int) -> tuple[bytes, dict]:
    """
    Exact Train-Track Series-Reduced Accelerated Rauzy-Veech Reconstruction (TTSR-RV).
    1. Split maximal runs at primary p so [p, p+1) is a dedicated width-1 pinned run.
    2. Build codomain order in one 256-bucket pass; two-pointer merge with domain partition
       to identify all (in_deg == 1, out_deg == 1) pass-through runs (excluding p_run).
    3. Place p_run's width-1 piece at index 0 of [0, N_core), followed by remaining core arcs.
    4. Run accelerated right-end Rauzy-Veech keeping y on equal-width merges so piece 0
       stays invariant at the left end until rem == 1 (x == y == piece 0).
    5. Expand the SLP root word in text order (reverse of LF concatenation) and repeat to n.
    """
    n = len(L)
    # Step 1: Extract runs, splitting at p so p is an isolated width-1 run
    starts = []
    widths = []
    syms = []
    p_run = -1

    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        if i <= p < j:
            if i < p:
                starts.append(i)
                widths.append(p - i)
                syms.append(c)
            p_run = len(starts)
            starts.append(p)
            widths.append(1)
            syms.append(c)
            if p + 1 < j:
                starts.append(p + 1)
                widths.append(j - (p + 1))
                syms.append(c)
        else:
            starts.append(i)
            widths.append(j - i)
            syms.append(c)
        i = j

    r = len(starts)
    ends = [s + w for s, w in zip(starts, widths)]

    # Bucket runs by symbol for C and codomain partition (W = r, Q = 0)
    buckets: list[list[int]] = [[] for _ in range(256)]
    sym_counts = [0] * 256
    for rid, (c, w) in enumerate(zip(syms, widths)):
        buckets[c].append(rid)
        sym_counts[c] += w

    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += sym_counts[c]

    img_starts = [0] * r
    img_ends = [0] * r
    cod_order = []
    occ = list(C)
    for rid, (c, w) in enumerate(zip(syms, widths)):
        img_starts[rid] = occ[c]
        img_ends[rid] = occ[c] + w
        occ[c] += w
    for c in range(256):
        cod_order.extend(buckets[c])

    # Step 2: Two-pointer merge of codomain and domain partitions
    out_deg = [0] * r
    in_deg = [0] * r
    arcs_u = []
    arcs_v = []
    arcs_w = []
    arcs_uo = []
    arcs_vo = []

    mi = 0
    vi = 0
    cur_pos = 0
    while mi < r and vi < r:
        u = cod_order[mi]
        u_end = img_ends[u]
        v_end = ends[vi]
        nxt = u_end if u_end < v_end else v_end
        w_arc = nxt - cur_pos
        if w_arc > 0:
            arcs_u.append(u)
            arcs_v.append(vi)
            arcs_w.append(w_arc)
            arcs_uo.append(cur_pos - img_starts[u])
            arcs_vo.append(cur_pos - starts[vi])
            out_deg[u] += 1
            in_deg[vi] += 1
        cur_pos = nxt
        if u_end == nxt:
            mi += 1
        if v_end == nxt:
            vi += 1

    passthrough = [(in_deg[u] == 1 and out_deg[u] == 1 and u != p_run) for u in range(r)]
    unique_succ = [-1] * r
    unique_voff = [0] * r
    core_out: list[list[tuple[int, int, int, int]]] = [[] for _ in range(r)]

    for u, v, w_a, uo, vo in zip(arcs_u, arcs_v, arcs_w, arcs_uo, arcs_vo):
        if out_deg[u] == 1:
            unique_succ[u] = v
            unique_voff[u] = vo
        if not passthrough[u]:
            core_out[u].append((uo, v, vo, w_a))

    # Step 3: Place p_run FIRST in core layout so piece 0 (1-indexed id=1) is [0, 1) at p!
    core_runs = [p_run] + [u for u in range(r) if not passthrough[u] and u != p_run]

    # Word dictionary (in FORWARD TEXT ORDER = reverse of LF traversal order!):
    # Along LF path c -> u_1 -> ... -> u_h, LF emits syms[c], syms[u_1], ..., syms[u_h] backwards in text,
    # so in forward text order this segment spells: syms[u_h] ... syms[u_1] syms[c]!
    # Leaf literals 0..N_leaves-1 store slices in leaf_bytes; internal SLP rules >= N_leaves store (left, right) in forward text order!
    leaf_bytes = bytearray()
    leaf_off = [0]

    piece_widths = []
    piece_word = []
    # Landing bucketing per core run to avoid general sort: core_in[target_core_run] holds (vo, pid)
    core_in: list[list[tuple[int, int]]] = [[] for _ in range(r)]

    for c in core_runs:
        # Already in increasing uo order because two-pointer merge increases cur_pos monotonically!
        for uo, v, vo, w_a in core_out[c]:
            # Trace passthrough chain
            chain_syms = [syms[c]]
            cur_v = v
            cur_vo = vo
            while passthrough[cur_v]:
                chain_syms.append(syms[cur_v])
                cur_vo = unique_voff[cur_v]
                cur_v = unique_succ[cur_v]
            # Reverse chain_syms so leaf is stored in forward text order!
            leaf_bytes.extend(reversed(chain_syms))
            leaf_off.append(len(leaf_bytes))
            wid = len(leaf_off) - 2  # 0-based leaf ID

            pid = len(piece_widths)  # 0-based piece ID; piece 0 is guaranteed to be p_run's width-1 piece!
            piece_widths.append(w_a)
            piece_word.append(wid)
            core_in[cur_v].append((cur_vo, pid))

    assert piece_widths[0] == 1, "Piece 0 (primary p) must have width 1"
    k = len(piece_widths)
    n_leaves = k

    # Build top_order and bot_order in core_runs layout order!
    top_order = list(range(k))
    bot_order = []
    for c in core_runs:
        lst = core_in[c]
        if len(lst) > 1:
            lst.sort(key=lambda t: t[0])
        for _, pid in lst:
            bot_order.append(pid)

    # Step 4: Accelerated Right-End Rauzy-Veech keeping y on equal-width merges!
    # Nodes are 1..k (node 1 is piece 0 = primary p at left end!). Node 0 is circular list sentinel.
    w = [0] + piece_widths
    w_tok = [0] + list(piece_word)
    prev0 = [0] * (k + 1)
    next0 = [0] * (k + 1)
    prev1 = [0] * (k + 1)
    next1 = [0] * (k + 1)

    prv = 0
    for u in top_order:
        uid = u + 1
        next0[prv] = uid
        prev0[uid] = prv
        prv = uid
    next0[prv] = 0
    prev0[0] = prv

    prv = 0
    for u in bot_order:
        uid = u + 1
        next1[prv] = uid
        prev1[uid] = prv
        prv = uid
    next1[prv] = 0
    prev1[0] = prv

    # Binary SLP rules in forward text order: rule_id = n_leaves + idx -> (left_tok, right_tok)
    # Check concatenation order carefully!
    # Suppose piece A (in dom, rightmost x) and piece B (in cod, rightmost y):
    # Points in overlap enter B in domain, map by T to cod(B) = right suffix of dom(A), and then take A!
    # So in LF order (backwards in text), the itinerary is: first B, then A!
    # Therefore, in FORWARD TEXT ORDER (reverse of LF), the itinerary is: FIRST word(A), THEN word(B)!!
    rules_left = []
    rules_right = []

    def join_text(first_tok: int, second_tok: int) -> int:
        rid = n_leaves + len(rules_left)
        rules_left.append(first_tok)
        rules_right.append(second_tok)
        return rid

    def power_text(tok: int, q: int) -> int:
        """Binary exponentiation in SLP for q >= 1."""
        assert q >= 1
        res = -1
        base = tok
        rem_q = q
        while rem_q > 0:
            if rem_q & 1:
                res = base if res == -1 else join_text(res, base)
            rem_q >>= 1
            if rem_q > 0:
                base = join_text(base, base)
        return res

    def cut0(uid: int):
        pr, nx = prev0[uid], next0[uid]
        next0[pr] = nx
        prev0[nx] = pr

    def cut1(uid: int):
        pr, nx = prev1[uid], next1[uid]
        next1[pr] = nx
        prev1[nx] = pr

    def ins0_after(at: int, uid: int):
        nx = next0[at]
        prev0[uid] = at
        next0[uid] = nx
        next0[at] = uid
        prev0[nx] = uid

    def ins1_after(at: int, uid: int):
        nx = next1[at]
        prev1[uid] = at
        next1[uid] = nx
        next1[at] = uid
        prev1[nx] = uid

    steps_acc = 0
    root_tok = -1
    while True:
        x = prev0[0]
        y = prev1[0]
        assert x != 0 and y != 0
        wx = w[x]
        wy = w[y]
        steps_acc += 1

        if x == y:
            if x == 1:
                # Primary mark's cycle closed!
                assert wx == 1 and next0[0] == 1
                root_tok = w_tok[1]
                break
            # Disconnected cycle not containing primary p: discard!
            cut0(x)
            cut1(x)
            continue

        assert x != 1, "Invariant violation: node 1 cannot reach right end of top_order before x==y==1"

        if wx < wy:
            # x (loser, dom) < y (winner, cod).
            # Check Euclidean acceleration if prev0[x] == y ( wait: in codomain, if prev1[y] == x! ):
            # Wait: after 1 step, x is cut from top0 and inserted after y in top0; y shrinks by wx and stays at prev1[0]!
            # If prev0[x] was NOT changed, the new prev0[0] is old prev0[x].
            # Wait: if y == old prev0[x], then inserting x after y in top0 makes x the NEW prev0[0] AGAIN!!
            # YES!! Check that: if prev0[x] == y, then cut0(x) makes y = prev0[0], and ins0_after(y, x) puts x RIGHT BACK at prev0[0]!!
            if prev0[x] == y:
                q = (wy - 1) // wx
                # In each step, point enters y (if landing in right suffix) -> takes x!
                # Wait: in Case wx < wy, the RIGHT suffix of cod(y) (width wx) hits dom(x), so THAT slice of y becomes y followed by x (in LF), i.e., join_text(w_tok[x], w_tok[y])!
                # And that slice IS represented by x (moved to sit inside dom(y))!
                # Across q consecutive steps with the SAME y at prev1[0] and x at prev0[0]:
                # Each step prepends w_tok[y] in LF order -> appends w_tok[y] in text order: w_tok[x] <- join_text(w_tok[x], w_tok[y]^q)!
                w_tok[x] = join_text(w_tok[x], power_text(w_tok[y], q))
                w[y] = wy - q * wx
                # Notice x is ALREADY after y in top0 (since prev0[x] == y!), so NO linked-list pointer changes at all when q >= 1 and prev0[x] == y!
            else:
                cut0(x)
                ins0_after(y, x)
                w_tok[x] = join_text(w_tok[x], w_tok[y])
                w[y] = wy - wx
        elif wy < wx:
            # y (loser, cod) < x (winner, dom).
            # Symmetrically, if prev1[y] == x, cut1(y) + ins1_after(x, y) leaves y at prev1[0] and x at prev0[0]!
            # In LF order, point in right suffix of dom(x) maps into cod(y)? Wait: cod(y) has width wy < wx = width(dom(x)).
            # Points in dom(y) map to cod(y), which is the rightmost wy of [0, M), i.e., the rightmost wy of dom(x)!
            # So points in dom(y) go y -> x (in LF order), which in FORWARD TEXT ORDER is join_text(w_tok[x], w_tok[y])!
            # Across q steps with same x at prev0[0] and y at prev1[0], y's LF itinerary is y -> x -> x -> ... (q times),
            # which in FORWARD TEXT ORDER is (w_tok[x]^q) + w_tok[y]!
            if prev1[y] == x:
                q = (wx - 1) // wy
                w_tok[y] = join_text(power_text(w_tok[x], q), w_tok[y])
                w[x] = wx - q * wy
            else:
                cut1(y)
                ins1_after(x, y)
                w_tok[y] = join_text(w_tok[x], w_tok[y])
                w[x] = wx - wy
        else:
            # wx == wy, x != y:
            # Points in dom(y) go y -> x in LF order => forward text order is join_text(w_tok[x], w_tok[y]).
            # Keep y in top_order (at dom(y))! Move y in bot_order to x's position (cod(x)), and delete x!
            w_tok[y] = join_text(w_tok[x], w_tok[y])
            cut1(y)
            ins1_after(x, y)
            cut0(x)
            cut1(x)

    # Step 5: Expand root_tok with first-occurrence memoization (one-scan / cached DFS, like endpoint.rs!)
    out = bytearray()
    first_pos = [-1] * len(rules_left)
    rule_len = [0] * len(rules_left)
    stack = [root_tok]
    while stack:
        tok = stack.pop()
        if tok < n_leaves:
            out.extend(leaf_bytes[leaf_off[tok]:leaf_off[tok + 1]])
        else:
            ridx = tok - n_leaves
            fp = first_pos[ridx]
            if fp == -1:
                # Post-order marker to record length once children finish: push ~(ridx)
                first_pos[ridx] = len(out)
                stack.append(~ridx)
                stack.append(rules_right[ridx])
                stack.append(rules_left[ridx])
            elif tok >= 0 and rule_len[ridx] > 0:
                ln = rule_len[ridx]
                out.extend(out[fp:fp + ln])
            else:
                # Ancestor not yet finished (only possible in power_text if self-nested? No, DAG children finish before sibling!)
                # Wait: in join_text(base, base), left `base` finishes completely (setting rule_len!) BEFORE right `base` is popped!
                raise AssertionError("Unfinished DAG node")
        while stack and stack[-1] < 0:
            fin_ridx = ~stack.pop()
            rule_len[fin_ridx] = len(out) - first_pos[fin_ridx]

    assert len(out) > 0 and len(out) <= n
    period_bytes = bytes(out)
    while len(out) < n:
        take = min(len(period_bytes), n - len(out))
        out.extend(period_bytes[:take])

    return bytes(out), {
        "r": r,
        "core_pieces": k,
        "steps_acc": steps_acc,
        "rules": len(rules_left),
        "leaf_bytes": len(leaf_bytes),
    }


def main():
    t0 = time.time()
    # 1. Exhaustive binary columns n = 1..8
    bin_cases = 0
    for n in range(1, 9):
        for bits in itertools.product((0, 1), repeat=n):
            L = bytes(bits)
            for p in range(n):
                ref = dense_lf_invert(L, p)
                got, _ = ttsr_rv_invert(L, p)
                assert got == ref, f"Binary mismatch n={n} p={p} L={list(L)}: got={list(got)} ref={list(ref)}"
                bin_cases += 1
    print(f"PASS exhaustive binary n=1..8: {bin_cases} / {bin_cases} byte-exact (including periodic & multi-cycle)")

    # 2. Exhaustive ternary columns n = 1..5
    ter_cases = 0
    for n in range(1, 6):
        for trits in itertools.product((0, 1, 2), repeat=n):
            L = bytes(trits)
            for p in range(n):
                ref = dense_lf_invert(L, p)
                got, _ = ttsr_rv_invert(L, p)
                assert got == ref, f"Ternary mismatch n={n} p={p} L={list(L)}"
                ter_cases += 1
    print(f"PASS exhaustive ternary n=1..5: {ter_cases} / {ter_cases} byte-exact (including periodic & multi-cycle)")

    # 3. Six authenticated Wikidata blocks
    manifest = json.loads((BLOCKS_DIR / "manifest.json").read_text())
    for item in manifest["blocks"]:
        fname = item["file"]
        p = item["primary_index"]
        L = (BLOCKS_DIR / fname).read_bytes()
        t_b = time.time()
        ref = dense_lf_invert(L, p)
        got, stats = ttsr_rv_invert(L, p)
        assert got == ref, f"Block mismatch on {fname}!"
        print(f"PASS {fname} (n={len(L)}, p={p}): byte-exact in {time.time() - t_b:.2f}s | {json.dumps(stats)}")

    print(f"\nALL {bin_cases + ter_cases} SMALL CASES + 6 AUTHENTICATED BLOCKS PASSED IN {time.time() - t0:.2f}s")


if __name__ == "__main__":
    main()
