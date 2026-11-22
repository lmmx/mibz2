#!/usr/bin/env python3
"""
Cycle 1 Deep Triage:
  - Test Candidate 3: ZeroSplit-IET (Parallel Run-Granular Containment & Exact-Cover Contraction)
  - Test Candidate 2: IET-BSE across ALL 6 Wikidata blocks + exact bucket working-set sizes,
    append-only buffer traffic, monotone self-map spiral acceleration (Symmetric-Middle-Run O(r)!),
    and output-backed memoized SLP expansion cost vs OpenCells!
"""

import json
import time
import random
from collections import defaultdict
import sys

sys.path.insert(0, "/tmp/fcp-b")
import explore_triage as et


# ==============================================================================
# CANDIDATE 3: ZeroSplit-IET (Parallel Run-Granular Aligned/Full-Cover Contraction)
# ==============================================================================

def zerosplit_iet(runs, p, build_slp=True, max_rounds=100000):
    """
    Parallel run-granular IET contraction with strict Delta_q <= 0.
    Each active piece u has:
      dom interval [dlo, dhi) of length lam[u]
      im  interval [ilo, ihi) of length lam[u]
      word w[u], wlen[u]
    In each parallel round:
      We find a conflict-free maximal matching of Zero-Split contractions:
        (A) Exact cover: dom(u) == im(v_1) U ... U im(v_k) (k >= 1, none equal to u)
            -> Delete u! Redirect all v_1..v_k straight through u (Delta_q = -1)!
        (B) Dual exact cover: im(v) == dom(u_1) U ... U dom(u_k) (k >= 1, none equal to v)
            -> Delete v! Redirect all u_1..u_k straight through v (Delta_q = -1)!
        (C) Endpoint-aligned containment:
            - im(v) strictly inside dom(u), sharing left or right endpoint of dom(u)
            - dom(u) strictly inside im(v), sharing left or right endpoint of im(v)
            -> Trim the wider interval, compose! (Delta_q = 0)!
      Note: For an exact parallel round, no piece participates in more than one contraction
      in the same round (vertex-disjoint matching of stars!).
      Also: if dom(u) == im(u) (closed cycle), pop it!
    """
    r0 = len(runs)
    n = sum(w for _, w in runs)
    if n == 0:
        return [], {"rounds": 0, "steps": 0}

    max_sym = max(c for c, _ in runs)
    counts = [0] * (max_sym + 1)
    for c, w in runs:
        counts[c] += w
    C = [0] * (max_sym + 2)
    for c in range(max_sym + 1):
        C[c + 1] = C[c] + counts[c]

    f_cur = list(C[:max_sym + 1])
    # pieces: dict id -> [dlo, ilo, lam, word, wlen]
    pieces = {}
    rules = [('lit', c) for c, _ in runs] if build_slp else None

    def make_cat(w1, w2):
        if build_slp:
            nid = len(rules)
            rules.append(('cat', w1, w2))
            return nid
        return 0

    pos = 0
    for u, (c, w) in enumerate(runs):
        dlo = f_cur[c]
        f_cur[c] += w
        pieces[u] = [dlo, pos, w, u, 1]
        pos += w

    # Track marker p as global coordinate p_pos + p_shift
    p_pos = p
    p_shift = 0
    p_cycle = None

    rounds = 0
    total_ops = 0
    exact_covers = 0
    aligned_trims = 0
    max_q = r0

    while pieces:
        rounds += 1
        assert rounds <= max_rounds, "ZeroSplit-IET exceeded max_rounds!"
        if len(pieces) > max_q:
            max_q = len(pieces)

        # Build coordinate indices for dom and im intervals
        # Because intervals are disjoint and tile the SAME surviving set S subset [0, n),
        # dom_by_start[x] = u, dom_by_end[x] = u, im_by_start[x] = v, im_by_end[x] = v
        dom_by_start = {}
        dom_by_end = {}
        im_by_start = {}
        im_by_end = {}
        for u, (dlo, ilo, ln, wd, wl) in pieces.items():
            dom_by_start[dlo] = u
            dom_by_end[dlo + ln] = u
            im_by_start[ilo] = u
            im_by_end[ilo + ln] = u

        used = set()
        ops_this_round = 0

        # Pass 0: Closed cycles dom(u) == im(u)
        to_del = []
        for u, (dlo, ilo, ln, wd, wl) in list(pieces.items()):
            if dlo == ilo:
                if p_cycle is None and dlo <= p_pos < dlo + ln:
                    p_cycle = (wd, p_shift, wl)
                to_del.append(u)
                used.add(u)
                ops_this_round += 1
                total_ops += 1
        for u in to_del:
            del pieces[u]
        if to_del:
            continue

        # Pass 1: Exact covers (Delta_q = -1)! Prioritize exact matches and full covers!
        # (A) dom(u) == im(v_1) U ... U im(v_k)
        actions = []  # ('dom_cover', u, vs) | ('im_cover', v, us) | ('trim_dom_L', u, v) | ...
        for u, (dlo, ilo, ln, wd, wl) in pieces.items():
            if u in used:
                continue
            # Check if dlo is an im_start AND walking im intervals reaches dlo + ln cleanly
            if dlo in im_by_start:
                cur = dlo
                end = dlo + ln
                vs = []
                ok = True
                while cur < end:
                    v = im_by_start.get(cur)
                    if v is None or v == u or v in used:
                        ok = False
                        break
                    v_ln = pieces[v][2]
                    if cur + v_ln > end:
                        ok = False
                        break
                    vs.append(v)
                    cur += v_ln
                if ok and cur == end:
                    used.add(u)
                    for v in vs:
                        used.add(v)
                    actions.append(('dom_cover', u, vs))

        # (B) im(v) == dom(u_1) U ... U dom(u_k)
        for v, (vdlo, vilo, vln, vwd, vwl) in pieces.items():
            if v in used:
                continue
            if vilo in dom_by_start:
                cur = vilo
                end = vilo + vln
                us = []
                ok = True
                while cur < end:
                    u = dom_by_start.get(cur)
                    if u is None or u == v or u in used:
                        ok = False
                        break
                    u_ln = pieces[u][2]
                    if cur + u_ln > end:
                        ok = False
                        break
                    us.append(u)
                    cur += u_ln
                if ok and cur == end:
                    used.add(v)
                    for u in us:
                        used.add(u)
                    actions.append(('im_cover', v, us))

        # Pass 2: Endpoint-aligned containments (Delta_q = 0) on remaining unused pieces!
        # Check matching left endpoints (dlo == vilo) and matching right endpoints (dhi == vihi)
        for u, (dlo, ilo, ln, wd, wl) in pieces.items():
            if u in used:
                continue
            # Left endpoint match
            v = im_by_start.get(dlo)
            if v is not None and v != u and v not in used:
                vln = pieces[v][2]
                if vln < ln:
                    used.add(u); used.add(v)
                    actions.append(('trim_dom_L', u, v))
                    continue
                elif ln < vln:
                    used.add(u); used.add(v)
                    actions.append(('trim_im_L', u, v))
                    continue
            # Right endpoint match
            v = im_by_end.get(dlo + ln)
            if v is not None and v != u and v not in used:
                vln = pieces[v][2]
                if vln < ln:
                    used.add(u); used.add(v)
                    actions.append(('trim_dom_R', u, v))
                    continue
                elif ln < vln:
                    used.add(u); used.add(v)
                    actions.append(('trim_im_R', u, v))
                    continue

        assert len(actions) > 0, f"DEADLOCK in ZeroSplit-IET! q={len(pieces)}"

        # Execute all independent actions in parallel!
        for act in actions:
            kind = act[0]
            if kind == 'dom_cover':
                _, u, vs = act
                udlo, uilo, uln, uwd, uwl = pieces[u]
                # Redirect every v in vs through u!
                for v in vs:
                    vdlo, vilo, vln, vwd, vwl = pieces[v]
                    off_in_u = vilo - udlo
                    # Marker check: if p_pos in [vilo, vilo+vln) inside dom(u), move p back to vdlo!
                    if p_cycle is None and vilo <= p_pos < vilo + vln:
                        p_pos = vdlo + (p_pos - vilo)
                        p_shift += vwl
                    pieces[v] = [vdlo, uilo + off_in_u, vln, make_cat(vwd, uwd), vwl + uwl]
                del pieces[u]
                exact_covers += 1
                total_ops += len(vs)

            elif kind == 'im_cover':
                _, v, us = act
                vdlo, vilo, vln, vwd, vwl = pieces[v]
                # Redirect every u in us: dom(u) moves back to vdlo + (udlo - vilo)!
                for u in us:
                    udlo, uilo, uln, uwd, uwl = pieces[u]
                    off_in_v = udlo - vilo
                    new_dlo = vdlo + off_in_v
                    if p_cycle is None and udlo <= p_pos < udlo + uln:
                        p_pos = new_dlo + (p_pos - udlo)
                        p_shift += vwl
                    pieces[u] = [new_dlo, uilo, uln, make_cat(vwd, uwd), vwl + uwl]
                del pieces[v]
                exact_covers += 1
                total_ops += len(us)

            elif kind == 'trim_dom_L':
                # v (im=[dlo, dlo+vln)) strictly inside prefix of dom(u)=[dlo, dlo+uln)
                _, u, v = act
                udlo, uilo, uln, uwd, uwl = pieces[u]
                vdlo, vilo, vln, vwd, vwl = pieces[v]
                if p_cycle is None and udlo <= p_pos < udlo + vln:
                    p_pos = vdlo + (p_pos - udlo)
                    p_shift += vwl
                pieces[v] = [vdlo, uilo, vln, make_cat(vwd, uwd), vwl + uwl]
                pieces[u] = [udlo + vln, uilo + vln, uln - vln, uwd, uwl]
                aligned_trims += 1
                total_ops += 1

            elif kind == 'trim_dom_R':
                # v (im=[dhi-vln, dhi)) strictly inside suffix of dom(u)=[dlo, dhi)
                _, u, v = act
                udlo, uilo, uln, uwd, uwl = pieces[u]
                vdlo, vilo, vln, vwd, vwl = pieces[v]
                cut = udlo + uln - vln
                if p_cycle is None and cut <= p_pos < udlo + uln:
                    p_pos = vdlo + (p_pos - cut)
                    p_shift += vwl
                pieces[v] = [vdlo, uilo + (uln - vln), vln, make_cat(vwd, uwd), vwl + uwl]
                pieces[u] = [udlo, uilo, uln - vln, uwd, uwl]
                aligned_trims += 1
                total_ops += 1

            elif kind == 'trim_im_L':
                # dom(u)=[dlo, dlo+uln) strictly inside prefix of im(v)=[dlo, dlo+vln)
                _, u, v = act
                udlo, uilo, uln, uwd, uwl = pieces[u]
                vdlo, vilo, vln, vwd, vwl = pieces[v]
                if p_cycle is None and udlo <= p_pos < udlo + uln:
                    p_pos = vdlo + (p_pos - udlo)
                    p_shift += vwl
                pieces[u] = [vdlo, uilo, uln, make_cat(vwd, uwd), vwl + uwl]
                pieces[v] = [vdlo + uln, vilo + uln, vln - uln, vwd, vwl]
                aligned_trims += 1
                total_ops += 1

            elif kind == 'trim_im_R':
                # dom(u)=[dhi-uln, dhi) strictly inside suffix of im(v)=[vilo, dhi)
                _, u, v = act
                udlo, uilo, uln, uwd, uwl = pieces[u]
                vdlo, vilo, vln, vwd, vwl = pieces[v]
                new_dlo = vdlo + (vln - uln)
                if p_cycle is None and udlo <= p_pos < udlo + uln:
                    p_pos = new_dlo + (p_pos - udlo)
                    p_shift += vwl
                pieces[u] = [new_dlo, uilo, uln, make_cat(vwd, uwd), vwl + uwl]
                pieces[v] = [vdlo, vilo, vln - uln, vwd, vwl]
                aligned_trims += 1
                total_ops += 1

    assert p_cycle is not None
    decoded = None
    h = -1
    if build_slp:
        wd, sh, wl = p_cycle
        cyc = et.expand_slp(rules, wd)
        assert len(cyc) == wl
        s = sh % wl
        decoded = cyc[s:] + cyc[:s]
        h = et.slp_height(rules, wd)

    return decoded, {
        "n": n, "r0": r0, "rounds": rounds, "total_ops": total_ops,
        "exact_covers": exact_covers, "aligned_trims": aligned_trims,
        "max_q": max_q, "slp_height": h, "slp_nodes": len(rules) if build_slp else -1
    }


# ==============================================================================
# DEEP DIAGNOSTICS: Output-Backed Memoized SLP Expansion Census!
# ==============================================================================

def measure_slp_expansion(rules, root, n, threshold=4):
    """
    Simulate output-backed memoized SLP expansion:
    - First visit to a nonterminal u with wlen(u) >= threshold: expand its children recursively,
      then record out_pos[u] (its exact start position in the output buffer!).
    - Any subsequent visit to u: emit ONE memcpy(out_pos[u], wlen(u)) without descending!
    - For wlen(u) < threshold: expand directly to literals (or inline short copy).
    Measures:
      - unique_nodes_visited (each nonterminal expanded at most once)
      - total_memcpys (reused nonterminal hits)
      - total_memcpy_bytes
      - total_literal_writes
      - total_dfs_edges (total tree operations = unique_expansions*2 + memcpys)
    """
    # Compute byte lengths of reachable rules
    wlen = {}
    stack = [(root, False)]
    while stack:
        u, done = stack.pop()
        if u in wlen:
            continue
        r = rules[u]
        if r[0] == 'lit':
            wlen[u] = 1
        elif r[0] == 'cat':
            if done:
                wlen[u] = wlen[r[1]] + wlen[r[2]]
            else:
                stack.append((u, True))
                stack.append((r[1], False))
                stack.append((r[2], False))
        elif r[0] == 'pow':
            if done:
                wlen[u] = wlen[r[1]] * r[2]
            else:
                stack.append((u, True))
                stack.append((r[1], False))

    memo_pos = {}
    cur_out = 0
    memcpys = 0
    memcpy_bytes = 0
    lit_writes = 0
    node_expansions = 0

    # Iterative DFS with post-order memoization
    stack = [(root, 0)]  # (u, state: 0=enter, 1=finish(start_pos))
    while stack:
        u, st = stack.pop()
        if st == 1:
            # Record start pos? We pass start_pos in tuple when needed
            continue
        if isinstance(st, tuple):
            # finish marker ('fin', start_pos)
            memo_pos[u] = st[1]
            continue

        wl = wlen[u]
        if wl >= threshold and u in memo_pos:
            memcpys += 1
            memcpy_bytes += wl
            cur_out += wl
            continue

        r = rules[u]
        if r[0] == 'lit':
            lit_writes += 1
            cur_out += 1
        elif r[0] == 'cat':
            node_expansions += 1
            if wl >= threshold:
                stack.append((u, ('fin', cur_out)))
            stack.append((r[2], 0))
            stack.append((r[1], 0))
        elif r[0] == 'pow':
            node_expansions += 1
            base, k = r[1], r[2]
            if wl >= threshold:
                stack.append((u, ('fin', cur_out)))
            # Expand base once, then k-1 memcpys!
            for _ in range(k):
                stack.append((base, 0))

    assert cur_out == wlen[root]
    return {
        "reachable_rules": len(wlen),
        "node_expansions": node_expansions,
        "memcpys": memcpys,
        "memcpy_bytes": memcpy_bytes,
        "avg_copy_len": memcpy_bytes / memcpys if memcpys else 0.0,
        "lit_writes": lit_writes,
        "total_ops": node_expansions + memcpys + lit_writes,
    }


def run_all():
    print("=== 1. VERIFYING CANDIDATE 3 (ZeroSplit-IET) EXHAUSTIVELY ===")
    cnt = 0
    for n in range(1, 9):
        for mask in range(1 << n):
            T = [(mask >> i) & 1 for i in range(n)]
            L, p = et.bwt_encode(T)
            runs = et.rle_from_L(L)
            got, st = zerosplit_iet(runs, p, build_slp=True)
            assert st["max_q"] <= len(runs)
            assert n % len(got) == 0 and got * (n // len(got)) == T
            cnt += 1
    for n in range(1, 6):
        for idx in range(3 ** n):
            T = []; v = idx
            for _ in range(n): T.append(v % 3); v //= 3
            L, p = et.bwt_encode(T)
            runs = et.rle_from_L(L)
            got, st = zerosplit_iet(runs, p, build_slp=True)
            assert st["max_q"] <= len(runs)
            assert n % len(got) == 0 and got * (n // len(got)) == T
            cnt += 1
    print(f"  PASS: {cnt} exhaustive binary n<=8 + ternary n<=5 BWTs (0 failures, 0 max_q > r0)!")

    print("\n=== 2. CANDIDATE 3 ON COUNTEREXAMPLES ===")
    for name, T in [("F4_n5_thresh", [0,0,1,1,2]), ("F4_n5_split", [0,0,2,0,1]),
                    ("F4_n5_self", [0,0,1,0,1]), ("F4_n10_sat", [0,0,1,1,2,1,2,0,0,1])]:
        L, p = et.bwt_encode(T)
        runs = et.rle_from_L(L)
        got, st = zerosplit_iet(runs, p, build_slp=True)
        assert got == T
        print(f"  {name:14s}: rounds={st['rounds']:3d} ops={st['total_ops']:3d} covers={st['exact_covers']} trims={st['aligned_trims']} h={st['slp_height']}")

    # Symmetric-Middle-Run
    for K in [4, 16, 64]:
        rng = random.Random(123)
        M_cnt = 64
        low_tags = [bytes([65 + (i // 8), 65 + (i % 8)]) for i in range(M_cnt)]
        high_tags = [bytes([83 + (i // 8), 83 + (i % 8)]) for i in range(M_cnt)]
        rng.shuffle(low_tags); rng.shuffle(high_tags)
        T = list(b"".join(low_tags[i] + bytes([77] * K) + high_tags[i] for i in range(M_cnt)))
        L, p = et.bwt_encode(T)
        runs = et.rle_from_L(L)
        got, st = zerosplit_iet(runs, p, build_slp=True)
        assert got == T
        print(f"  SymMid K={K:2d} (n={len(T):4d}, r0={len(runs):3d}): rounds={st['rounds']:4d} ops={st['total_ops']:5d} ({st['total_ops']/len(runs):.2f}r) covers={st['exact_covers']:4d} trims={st['aligned_trims']:4d} h={st['slp_height']}")

    print("\n=== 3. CANDIDATE 3 (ZeroSplit-IET) ON block-0.L ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    b0 = manifest["blocks"][0]
    L0 = list(open("/tmp/mibz2-blocks/" + b0["file"], "rb").read())
    runs0 = et.rle_from_L(L0)
    ref0 = et.decode_lf_reference(L0, b0["primary_index"])
    t0 = time.perf_counter()
    got0, st0 = zerosplit_iet(runs0, b0["primary_index"], build_slp=True)
    dt = time.perf_counter() - t0
    assert got0 == ref0, "ZeroSplit-IET MISMATCH on block-0.L!"
    print(f"  block-0.L: rounds={st0['rounds']} ops={st0['total_ops']} ({st0['total_ops']/len(runs0):.2f}r) "
          f"covers={st0['exact_covers']} trims={st0['aligned_trims']} h={st0['slp_height']} max_q={st0['max_q']} ({dt:.2f}s)")

    print("\n=== 4. CANDIDATE 2 (IET-BSE) ACROSS ALL 6 WIKIDATA BLOCKS ===")
    for binfo in manifest["blocks"]:
        L = list(open("/tmp/mibz2-blocks/" + binfo["file"], "rb").read())
        n, p = binfo["n"], binfo["primary_index"]
        runs = et.rle_from_L(L)
        r0 = len(runs)
        for om in ("min_deg", "max_width"):
            t0 = time.perf_counter()
            got, st = et.iet_bse(runs, p, order_mode=om, build_slp=False)
            dt = time.perf_counter() - t0
            print(f"  {binfo['file']:12s} [{om:9s}]: r0={r0:6d} tot_steps={st['total_steps']:6d} ({st['total_steps']/r0:.2f}r) "
                  f"merges={st['total_merge_ops']:6d} ({st['total_merge_ops']/r0:.2f}r) "
                  f"max_self={st['max_self_pieces']:5d} last_q={st['last_b_q']:5d} ({dt:.2f}s)")


if __name__ == "__main__":
    run_all()
