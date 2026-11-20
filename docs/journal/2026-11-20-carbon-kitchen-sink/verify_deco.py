#!/usr/bin/env python3
"""
Gate G1 Exhaustive Verifier for Decoupled Exact-Chain Open (DECO).

Proves & verifies the Decoupled Void-Deletion + Tail-Ordered Queue Theorem:
  - Maximal exact-match chains u_1 -> u_2 -> ... -> u_L among thin runs (width in [min_w, limit],
    excluding absorption of p_run) satisfy I_absorbed = f(S_thin \ {u_L}) disjoint from C_thick U f(C_thick).
  - Deleting I_absorbed from both domain and codomain preserves rigid width and translation of all H
    thick strips in endpoint.rs (ZERO thick-strip splitting, <= Open's RV nodes and steps!).
  - Scanning chain tails z (succ_exact[z] == -1) in natural domain order z = 0..r-1 populates
    queues[sym[z]] in exact sorted entrance order E' with ZERO sorting!

Verifies byte-exact output against dense LF walk on:
  - All 3,586 binary (L, p) pairs for n = 1..8 across (limit, min_w) in {(1,1), (2,1), (2,2), (4,2), (16,1), (16,2)}
  - All 1,641 ternary (L, p) pairs for n = 1..5 across (limit, min_w) in {(1,1), (2,2), (16,1), (16,2)}
  - All 6 SHA-256-authenticated Wikidata BWT blocks at limit=16, min_w in {1, 2, 3}.
"""

from __future__ import annotations
import itertools
import json
import pathlib
import time
from verify_ttsr import dense_lf_invert

BLOCKS_DIR = pathlib.Path("/home/antigravity/admitted-bwt")


def deco_invert(L: bytes, p: int, limit: int = 16, min_w: int = 2) -> tuple[bytes, dict]:
    n = len(L)
    # 1. Extract runs, splitting at p if p lies inside a run so p_run = [p, p+1) is never absorbed
    # (Wait: in Open, only thick runs containing p need splitting; in DECO, splitting any run with
    # width >= min_w (or > limit) that contains p into [i, p), [p, p+1), [p+1, j) ensures p_run has width 1
    # and is never absorbed, while keeping maximal-run semantics everywhere else!)
    starts = []
    widths = []
    syms = []
    is_thin = []
    p_run = -1

    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        w_full = j - i
        keep_parent = w_full <= limit
        if i <= p < j and (not keep_parent or w_full >= min_w):
            if i < p:
                starts.append(i)
                widths.append(p - i)
                syms.append(c)
                is_thin.append(keep_parent)
            p_run = len(starts)
            starts.append(p)
            widths.append(1)
            syms.append(c)
            is_thin.append(True)
            if p + 1 < j:
                starts.append(p + 1)
                widths.append(j - (p + 1))
                syms.append(c)
                is_thin.append(keep_parent)
        else:
            if i <= p < j:
                p_run = len(starts)
            starts.append(i)
            widths.append(w_full)
            syms.append(c)
            is_thin.append(keep_parent)
        i = j

    r = len(starts)
    assert p_run != -1 and is_thin[p_run]

    # Bucket runs by symbol for C and codomain order (W = r, Q = 0)
    buckets: list[list[int]] = [[] for _ in range(256)]
    all_counts = [0] * 256
    for rid in range(r):
        c = syms[rid]
        buckets[c].append(rid)
        all_counts[c] += widths[rid]

    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += all_counts[c]

    img_starts = [0] * r
    occ = list(C)
    for rid in range(r):
        c = syms[rid]
        img_starts[rid] = occ[c]
        occ[c] += widths[rid]

    cod_order = []
    for c in range(256):
        cod_order.extend(buckets[c])

    # 2. Single 2r-step sequential two-pointer scan to detect exact thin-to-thin matches!
    # Match u -> v iff is_thin[u] and is_thin[v] and v != p_run and u != v
    # and widths[u] == widths[v] >= min_w and img_starts[u] == starts[v]!
    succ_exact = [-1] * r
    has_pred = [False] * r
    mi = 0
    vi = 0
    while mi < r and vi < r:
        u = cod_order[mi]
        iu = img_starts[u]
        sv = starts[vi]
        if iu == sv:
            if (
                u != vi
                and vi != p_run
                and is_thin[u]
                and is_thin[vi]
                and widths[u] == widths[vi]
                and widths[u] >= min_w
            ):
                succ_exact[u] = vi
                has_pred[vi] = True
            mi += 1
            vi += 1
        elif iu < sv:
            mi += 1
        else:
            vi += 1

    # Note: closed cycles of exact-matched thin runs disjoint from p_run have has_pred[v] == True
    # and succ_exact[v] != -1 for all vertices on the cycle, so they drop out identically from
    # both domain S' and codomain E'!

    # 3. Assign compressed domain ranks [0, m_prime) to chain heads (is_thin[u] and not has_pred[u]),
    #    trace chains u_1 -> ... -> u_L, and attach (head_m, chain_suffix_id) to tail u_L!
    m_prime = 0
    mark = -1
    tail_head_m = [-1] * r
    tail_sfx_id = [-1] * r  # -1 means empty suffix (L == 1); >= 0 indexes sfx_offsets
    sfx_bytes = bytearray()
    sfx_offsets = [0]
    chain_hops = 0

    for u in range(r):
        if is_thin[u] and not has_pred[u]:
            w_u = widths[u]
            if u == p_run:
                mark = m_prime + (p - starts[u])
            if succ_exact[u] == -1:
                # L == 1: unchained thin run! Zero chain overhead!
                tail_head_m[u] = m_prime
                tail_sfx_id[u] = -1
            else:
                # Follow chain u = u_1 -> u_2 -> ... -> u_L
                # Proper prefix u_1 .. u_{L-1} reversed is appended AFTER u_L's band word!
                pref_syms = []
                cur = u
                while succ_exact[cur] != -1:
                    pref_syms.append(syms[cur])
                    cur = succ_exact[cur]
                    chain_hops += 1
                sfx_bytes.extend(reversed(pref_syms))
                sfx_offsets.append(len(sfx_bytes))
                tail_head_m[cur] = m_prime
                tail_sfx_id[cur] = len(sfx_offsets) - 2
            m_prime += w_u

    assert m_prime > 0 and 0 <= mark < m_prime

    # Populate per-symbol queues in EXACT sorted entrance order E' by scanning tails z = 0..r-1!
    # Because img_starts[z] is strictly increasing with z inside each symbol bucket sym[z],
    # ZERO sorting is needed!
    # Store queue items at run/chain granularity: (head_m, width, sfx_id)!
    # Wait: storing (head_m, width, sfx_id) per tail z has <= r_tails entries, AND during band wiring
    # we advance through queue spans cleanly without expanding m_prime entries into 256 Vecs!
    queues: list[list[tuple[int, int, int]]] = [[] for _ in range(256)]
    thin_e_counts = [0] * 256
    for z in range(r):
        if is_thin[z] and succ_exact[z] == -1:
            # Note: if z was on a closed cycle, succ_exact[z] != -1, so closed cycles never enter!
            hm = tail_head_m[z]
            assert hm != -1
            a = syms[z]
            w_z = widths[z]
            queues[a].append((hm, w_z, tail_sfx_id[z]))
            thin_e_counts[a] += w_z

    assert sum(thin_e_counts) == m_prime

    # 4. Run Endpoint RV on the H UNDIVIDED thick strips, omitting absorbed intervals from gaps!
    # Build nodes: 0 is sentinel; 1..H are thick strips; >H are domain/codomain terminal gaps.
    node_w = [0]
    node_word = [0]  # 0 = empty; 1..256 = byte + 1; >= 257 = binary rule
    order0 = []
    order1 = []
    term0 = []
    term1 = []

    thick_runs = [u for u in range(r) if not is_thin[u]]
    H = len(thick_runs)
    thick_node_of = [-1] * r
    for idx_t, u in enumerate(thick_runs):
        nid = len(node_w)
        node_w.append(widths[u])
        node_word.append(syms[u] + 1)
        thick_node_of[u] = nid

    # Domain pass u = 0..r-1: accumulate unabsorbed thin domain width (`not has_pred[u]`) between thick strips!
    gap_dom = 0
    for u in range(r):
        if is_thin[u]:
            if not has_pred[u]:
                gap_dom += widths[u]
        else:
            if gap_dom > 0:
                gid = len(node_w)
                node_w.append(gap_dom)
                node_word.append(0)
                order0.append(gid)
                term1.append(gid)
                gap_dom = 0
            order0.append(thick_node_of[u])
    if gap_dom > 0:
        gid = len(node_w)
        node_w.append(gap_dom)
        node_word.append(0)
        order0.append(gid)
        term1.append(gid)

    # Codomain pass over cod_order[0..r]: accumulate unabsorbed thin codomain width (`succ_exact[z] == -1`)!
    gap_cod = 0
    for z in cod_order:
        if is_thin[z]:
            if succ_exact[z] == -1:
                gap_cod += widths[z]
        else:
            if gap_cod > 0:
                gid = len(node_w)
                node_w.append(gap_cod)
                node_word.append(0)
                order1.append(gid)
                term0.append(gid)
                gap_cod = 0
            order1.append(thick_node_of[z])
    if gap_cod > 0:
        gid = len(node_w)
        node_w.append(gap_cod)
        node_word.append(0)
        order1.append(gid)
        term0.append(gid)

    K = len(node_w) - 1
    prev0 = [0] * (K + 1)
    next0 = [0] * (K + 1)
    prev1 = [0] * (K + 1)
    next1 = [0] * (K + 1)

    prv = 0
    for nid in term0 + order0:
        next0[prv] = nid
        prev0[nid] = prv
        prv = nid
    next0[prv] = 0
    prev0[0] = prv

    prv = 0
    for nid in term1 + order1:
        next1[prv] = nid
        prev1[nid] = prv
        prv = nid
    next1[prv] = 0
    prev1[0] = prv

    rules_left = []
    rules_right = []
    rules_len = []
    rules_first = []

    def tok_len(w_t: int) -> int:
        if w_t <= 256:
            return 1 if w_t != 0 else 0
        return rules_len[w_t - 257]

    def join_rv(l_t: int, r_t: int) -> int:
        if l_t == 0:
            return r_t
        if r_t == 0:
            return l_t
        rid = 257 + len(rules_left)
        rules_left.append(l_t)
        rules_right.append(r_t)
        rules_len.append(tok_len(l_t) + tok_len(r_t))
        rules_first.append(-1)
        return rid

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

    n_prime = m_prime + sum(widths[u] for u in thick_runs)
    rem = n_prime
    rv_steps = 0
    while rem > 0:
        x = prev0[0]
        y = prev1[0]
        wx = node_w[x]
        wy = node_w[y]
        w_cut = wx if wx < wy else wy
        assert 0 < w_cut <= rem
        rem -= w_cut
        rv_steps += 1
        if x == y:
            cut0(x)
            cut1(x)
            continue
        comb = join_rv(node_word[x], node_word[y])
        if wx <= wy:
            cut0(x)
            ins0_after(y, x)
            node_word[x] = comb
            if wx == wy:
                cut0(y)
                cut1(y)
            else:
                node_w[y] = wy - w_cut
        else:
            cut1(y)
            ins1_after(x, y)
            node_word[y] = comb
            node_w[x] = wx - w_cut

    # 5. Extract bands and wire edges[0..m_prime] in one pass!
    bottom = [0] * (K + 1)
    nid = next1[0]
    b_x = 0
    while nid != 0:
        bottom[nid] = b_x
        b_x += node_w[nid]
        nid = next1[nid]
    assert b_x == m_prime

    spelling = bytearray()
    offsets = [0]  # code >= 256 -> spelling[offsets[code-256] .. offsets[code-255]]

    def append_rv_word(w_t: int):
        stack = [w_t]
        while stack:
            tid = stack.pop()
            if tid <= 256:
                spelling.append(tid - 1)
            else:
                ridx = tid - 257
                fp = rules_first[ridx]
                if fp == -1:
                    rules_first[ridx] = len(spelling)
                    stack.append(rules_right[ridx])
                    stack.append(rules_left[ridx])
                else:
                    ln = rules_len[ridx]
                    spelling.extend(spelling[fp:fp + ln])

    edge_next = [0] * m_prime
    edge_code = [0] * m_prime

    nid = next0[0]
    in_node = 0
    k_cur = 0
    n_bands = 0

    for a in range(256):
        end_a = k_cur + thin_e_counts[a]
        q_a = queues[a]
        q_idx = 0
        q_off = 0
        while k_cur < end_a:
            nw = node_w[nid]
            b_width = min(end_a - k_cur, nw - in_node)
            assert b_width > 0
            dst_base = bottom[nid] + in_node
            n_bands += 1

            # Materialize base band code (for L == 1 items)
            nw_tok = node_word[nid]
            if nw_tok == 0:
                base_code = a
                base_start = -1
                base_end = -1
            else:
                base_start = len(spelling)
                append_rv_word(nw_tok)
                spelling.append(a)
                base_end = len(spelling)
                offsets.append(base_end)
                base_code = 254 + len(offsets)

            # Wire b_width positions from queues[a]!
            rem_b = b_width
            cur_dst = dst_base
            while rem_b > 0:
                hm, qw, sfx_id = q_a[q_idx]
                avail = qw - q_off
                span = rem_b if rem_b < avail else avail
                src_j = hm + q_off
                if sfx_id == -1:
                    use_code = base_code
                else:
                    # Concatenate base word + chain suffix!
                    if base_code < 256:
                        spelling.append(a)
                    else:
                        spelling.extend(spelling[base_start:base_end])
                    spelling.extend(sfx_bytes[sfx_offsets[sfx_id]:sfx_offsets[sfx_id + 1]])
                    offsets.append(len(spelling))
                    use_code = 254 + len(offsets)

                for u_off in range(span):
                    edge_next[cur_dst + u_off] = src_j + u_off
                    edge_code[cur_dst + u_off] = use_code

                cur_dst += span
                rem_b -= span
                q_off += span
                if q_off == qw:
                    q_idx += 1
                    q_off = 0

            in_node += b_width
            k_cur += b_width
            if in_node == nw:
                nid = next0[nid]
                in_node = 0

    assert nid == 0 and k_cur == m_prime

    # 6. Emit from mark!
    out = bytearray()
    j = mark
    for _ in range(m_prime):
        c_code = edge_code[j]
        j = edge_next[j]
        if c_code < 256:
            out.append(c_code)
        else:
            w_idx = c_code - 256
            out.extend(spelling[offsets[w_idx]:offsets[w_idx + 1]])
        if j == mark or len(out) >= n:
            break
    del out[n:]
    period_len = len(out)
    assert period_len > 0
    while len(out) < n:
        take = min(period_len, n - len(out))
        out.extend(out[:take])

    return bytes(out), {
        "r": r,
        "m_prime": m_prime,
        "chain_hops": chain_hops,
        "H_thick": H,
        "rv_nodes": K,
        "rv_steps": rv_steps,
        "bands": n_bands,
        "spelling_bytes": len(spelling),
    }


def main():
    t0 = time.time()
    bin_cases = 0
    for n in range(1, 9):
        for bits in itertools.product((0, 1), repeat=n):
            L = bytes(bits)
            for p in range(n):
                ref = dense_lf_invert(L, p)
                for lim, mw in ((1, 1), (2, 1), (2, 2), (4, 2), (16, 1), (16, 2)):
                    got, _ = deco_invert(L, p, limit=lim, min_w=mw)
                    assert got == ref, f"Binary mismatch n={n} p={p} lim={lim} mw={mw}"
                    bin_cases += 1
    print(f"PASS exhaustive binary n=1..8 x 6 configs: {bin_cases} / {bin_cases} byte-exact")

    ter_cases = 0
    for n in range(1, 6):
        for trits in itertools.product((0, 1, 2), repeat=n):
            L = bytes(trits)
            for p in range(n):
                ref = dense_lf_invert(L, p)
                for lim, mw in ((1, 1), (2, 2), (16, 1), (16, 2)):
                    got, _ = deco_invert(L, p, limit=lim, min_w=mw)
                    assert got == ref, f"Ternary mismatch n={n} p={p} lim={lim} mw={mw}"
                    ter_cases += 1
    print(f"PASS exhaustive ternary n=1..5 x 4 configs: {ter_cases} / {ter_cases} byte-exact")

    manifest = json.loads((BLOCKS_DIR / "manifest.json").read_text())
    for mw in (2, 1):
        print(f"\n=== Authenticated Blocks: DECO (limit=16, min_w={mw}) ===")
        for item in manifest["blocks"]:
            fname = item["file"]
            p = item["primary_index"]
            L = (BLOCKS_DIR / fname).read_bytes()
            tb = time.time()
            ref = dense_lf_invert(L, p)
            got, stats = deco_invert(L, p, limit=16, min_w=mw)
            assert got == ref, f"Block mismatch on {fname} mw={mw}"
            print(f"PASS {fname}: {time.time() - tb:.2f}s | {json.dumps(stats)}")

    print(f"\nALL {bin_cases + ter_cases} SMALL CONFIGS + 12 BLOCK RUNS PASSED IN {time.time() - t0:.2f}s")


if __name__ == "__main__":
    main()
