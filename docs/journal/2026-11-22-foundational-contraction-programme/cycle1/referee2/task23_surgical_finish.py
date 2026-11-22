#!/usr/bin/env python3
"""Surgical finish checks for Task 2 (all 6 orders on Beta iet_bse + Wikidata SCC/FVS)
and Task 3 (exact witness strings + Wikidata irreducible hex-clique & bounce attribution)."""
import hashlib
import itertools
import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import verify_btm_v2 as btm
sys.path.insert(0, "/tmp/fcp-b")
import explore_triage as beta_et


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


def iet_bse_custom_order(runs, p, custom_order):
    """Run Beta's exact iet_bse by patching only the bucket-choice line to follow custom_order."""
    import inspect, types
    src = inspect.getsource(beta_et.iet_bse)
    # Replace the order selection block cleanly
    old_block = """        if order_mode == "min_deg":
            b = min(alive, key=lambda x: (len(dom_pieces[x]) + len(im_pieces[x]), x))
        elif order_mode == "max_width":
            b = max(alive, key=lambda x: (counts[x], -x))
        elif order_mode == "min_width":
            b = min(alive, key=lambda x: (counts[x], x))
        elif order_mode == "max_sym":
            b = max(alive)
        elif order_mode == "min_self":
            def self_cnt(x):
                return sum(1 for pc in dom_pieces[x] if pc[1] == x)
            b = min(alive, key=lambda x: (self_cnt(x), len(dom_pieces[x]) + len(im_pieces[x]), x))
        else:
            b = min(alive)"""
    new_block = """        b = next(x for x in order_mode if x in alive)"""
    assert old_block in src, "Could not patch order_mode block"
    src2 = src.replace(old_block, new_block)
    ns = dict(beta_et.__dict__)
    exec(src2, ns)
    return ns["iet_bse"](runs, p, order_mode=custom_order, build_slp=True)


def tarjan_sccs(nodes, adj):
    index = 0
    stack = []
    on_stack = set()
    idx_map = {}
    low = {}
    sccs = []

    def strongconnect(v):
        nonlocal index
        idx_map[v] = low[v] = index
        index += 1
        stack.append(v)
        on_stack.add(v)
        for w in adj[v]:
            if w not in idx_map:
                strongconnect(w)
                if low[w] < low[v]:
                    low[v] = low[w]
            elif w in on_stack:
                if idx_map[w] < low[v]:
                    low[v] = idx_map[w]
        if low[v] == idx_map[v]:
            comp = []
            while True:
                w = stack.pop()
                on_stack.remove(w)
                comp.append(w)
                if w == v:
                    break
            sccs.append(comp)

    sys.setrecursionlimit(10000)
    for v in nodes:
        if v not in idx_map:
            strongconnect(v)
    return sccs


def greedy_fvs_size(nodes, adj):
    """Upper bound on directed FVS (removing vertices in 2-cycles / high in*out degree until DAG)."""
    rem = set(nodes)
    fvs = []
    while True:
        # Iteratively peel sources (in-deg 0) and sinks (out-deg 0)
        changed = True
        while changed:
            changed = False
            in_d = {v: 0 for v in rem}
            out_d = {v: 0 for v in rem}
            for u in rem:
                for w in adj[u]:
                    if w in rem and w != u:
                        out_d[u] += 1
                        in_d[w] += 1
            to_del = [v for v in rem if in_d[v] == 0 or out_d[v] == 0]
            if to_del:
                for v in to_del:
                    rem.remove(v)
                changed = True
        if not rem:
            break
        # Pick vertex with max in_d * out_d (or in_d + out_d) to put in FVS
        v = max(rem, key=lambda x: (in_d[x] * out_d[x], in_d[x] + out_d[x]))
        fvs.append(v)
        rem.remove(v)
    return fvs


def main():
    print("=== 1. ALL 6 ORDERS ON BETA'S EXACT iet_bse FOR Low-r Padded-Ring (k=4, r=39 CONSTANT) ===")
    db2 = de_bruijn(2, 4)
    for K in [4, 16, 64, 256]:
        w_ab = b"".join((b"a" if x == 0 else b"b") * K for x in db2)
        w_bc = b"".join((b"b" if x == 0 else b"c") * K for x in db2)
        w_ca = b"".join((b"c" if x == 0 else b"a") * K for x in db2)
        T = w_ab + w_bc + w_ca
        L, p = beta_et.bwt_encode(T)
        runs = beta_et.rle_from_L(L)
        r = len(runs)
        min_steps = 10**9
        max_steps = 0
        min_self_rv = 10**9
        for perm in itertools.permutations([ord('a'), ord('b'), ord('c')]):
            got, st = iet_bse_custom_order(runs, p, list(perm))
            assert bytes(got) == T
            min_steps = min(min_steps, st["total_steps"])
            max_steps = max(max_steps, st["total_steps"])
            min_self_rv = min(min_self_rv, st["total_self_rv_steps"])
        print(
            f"  K={K:3d} n={len(T):5d} r={r:2d} | ALL 6 ORDERS on Beta iet_bse: "
            f"min_total_steps={min_steps:5d} ({min_steps/len(T):.3f}n, {min_steps/r:6.2f}r), "
            f"max_total_steps={max_steps:5d}, min_self_rv={min_self_rv:5d}"
        )

    print("\n=== 2. EXPLICIT SMALLEST NATURAL TASK 3 WITNESSES ===")
    for name, T in [
        ("K3 Euler", b"abcacb"),
        ("DeBruijn B(3,2)", bytes(ord('a') + x for x in de_bruijn(3, 2))),
        ("AltRing k=2", b"ab" * 2 + b"bc" * 2 + b"ca" * 2),
        ("AltRing k=4", b"ab" * 4 + b"bc" * 4 + b"ca" * 4),
    ]:
        L, p = btm.bwt_encode(T)
        r = sum(1 for i in range(1, len(L)) if L[i] != L[i - 1]) + 1
        print(f"  {name:16s}: T={T.decode()!r}, L={L.decode()!r}, p={p}, n={len(T)}, |Sigma|={len(set(T))}, r={r}")

    print("\n=== 3. WIKIDATA 6-BLOCK SCC/FVS & IRREDUCIBLE HEX-CLIQUE BOUNCE ATTRIBUTION ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    hex_lo = set(b"0123456789abcdef")
    hex_all = set(b"0123456789abcdefABCDEF")
    hex_dash = hex_all | {ord("-")}

    for entry in manifest["blocks"]:
        path = os.path.join("/tmp/mibz2-blocks", entry["file"])
        L = open(path, "rb").read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        p = entry["primary_index"]
        n = len(L)
        r = entry["r"]
        _, buckets, pbb, bigrams, slp = btm.build_initial_from_L(L, p)
        syms = [c for c, _, _ in buckets]

        # Directed transition graph (ignoring self loops)
        adj = defaultdict(set)
        for (a, b), w in bigrams.items():
            if a != b and w > 0:
                adj[a].add(b)
        sccs = tarjan_sccs(syms, adj)
        max_scc = max(len(c) for c in sccs)
        fvs = greedy_fvs_size(syms, adj)

        # Count initial non-self pieces inside hex_lo, hex_all, hex_dash, and total
        tot_nonself_p = 0
        hexlo_p = 0
        hexlo_w1_p = 0
        hexall_p = 0
        hexall_w1_p = 0
        hexdash_p = 0
        for bc, pcs in pbb.items():
            for lo, hi, d, tsym, w, m in pcs:
                if bc != tsym:
                    tot_nonself_p += 1
                    wlen = hi - lo
                    if bc in hex_lo and tsym in hex_lo:
                        hexlo_p += 1
                        if wlen == 1:
                            hexlo_w1_p += 1
                    if bc in hex_all and tsym in hex_all:
                        hexall_p += 1
                        if wlen == 1:
                            hexall_w1_p += 1
                    if bc in hex_dash and tsym in hex_dash:
                        hexdash_p += 1

        # Re-run heavy_edge and attribute bounces to merges where the smaller side is a subset of hex_dash
        # OR intersects hex_dash!
        stats = defaultdict(int)
        nodes = {}
        for c, blo, bhi in buckets:
            s = frozenset([c])
            out_p, _ = btm.resolve_subset(s, pbb[c], slp, stats)
            nodes[c] = (s, out_p, bhi - blo)
        leaf_b = stats["bounces"]
        active = {i: nodes[c] for i, (c, _, _) in enumerate(buckets)}
        sym_to_cid = {c: i for i, (c, _, _) in enumerate(buckets)}
        w_adj = defaultdict(lambda: defaultdict(int))
        for (a, b), wt in bigrams.items():
            if a != b:
                ca, cb = sym_to_cid[a], sym_to_cid[b]
                w_adj[ca][cb] += wt; w_adj[cb][ca] += wt
        next_cid = len(active)
        hex_small_bounces = 0
        hex_any_bounces = 0
        while len(active) > 1:
            best_pair = None; best_score = -1.0
            cids = list(active.keys())
            for u in cids:
                nu = active[u]
                for v, wt in w_adj[u].items():
                    if v > u and v in active:
                        nv = active[v]
                        sc = wt / (len(nu[1]) + len(nv[1]) + 1.0)
                        if sc > best_score:
                            best_score = sc; best_pair = (u, v)
            if best_pair is None:
                best_pair = (cids[0], cids[1])
            u, v = best_pair
            s1, p1, c1 = active.pop(u)
            s2, p2, c2 = active.pop(v)
            b0 = stats["bounces"]
            out_p, _ = btm.resolve_subset(s1 | s2, p1 + p2, slp, stats)
            db = stats["bounces"] - b0
            small = s1 if len(s1) <= len(s2) else s2
            if small.issubset(hex_dash):
                hex_small_bounces += db
            if (s1 & hex_dash) and (s2 & hex_dash):
                hex_any_bounces += db
            cid = next_cid; next_cid += 1
            active[cid] = (s1 | s2, out_p, c1 + c2)
            new_row = defaultdict(int)
            for nb, wt in w_adj[u].items():
                if nb != v and nb in active: new_row[nb] += wt
            for nb, wt in w_adj[v].items():
                if nb != u and nb in active: new_row[nb] += wt
            del w_adj[u]; del w_adj[v]
            for nb, wt in new_row.items():
                w_adj[cid][nb] = wt; w_adj[nb][cid] = wt
                w_adj[nb].pop(u, None); w_adj[nb].pop(v, None)
        tot_mb = stats["bounces"] - leaf_b

        print(
            f"  {entry['file']:12s} | |Sigma|={len(syms):2d}, max_SCC={max_scc:2d}, greedy_FVS={len(fvs):2d} | "
            f"init_nonself_P={tot_nonself_p:6d} | hex_lo_P={hexlo_p:5d} (w1={hexlo_w1_p:5d}), "
            f"hex_all_P={hexall_p:5d} ({hexall_p/tot_nonself_p:.1%}), hex+dash_P={hexdash_p:5d} ({hexdash_p/tot_nonself_p:.1%}) | "
            f"heavy_edge_mb={tot_mb:6d}: small<=hex_dash={hex_small_bounces:6d} ({hex_small_bounces/tot_mb:.1%}), "
            f"cuts_hex_dash={hex_any_bounces:6d} ({hex_any_bounces/tot_mb:.1%})"
        )


if __name__ == "__main__":
    main()
