#!/usr/bin/env python3
"""
Task 3: Adversarial Ping-Pong Challenge (Alpha's Redheffer Star-Product Merge Tree)
1. Exhaustive verification over ALL (2*sigma - 3)!! binary merge tree topologies (sigma=3,4)
   on smallest natural valid-BWT inputs & scaling families:
   - K_3 Euler tour T = "abcacb" (n=6, |Sigma|=3, r=6)
   - De Bruijn B(3, 2) = "aabacbbcc" (n=9, |Sigma|=3, r=9), B(3, 3..5), B(4, 2..3)
   - Constant-r ping-pong killer: T_k = (ab)^k (bc)^k (ca)^k (|Sigma|=3, n=6k, r=7 CONSTANT!)
   - Constant-r K_4 killer (15 trees): T_k = (ab)^k (bc)^k (cd)^k (da)^k (ac)^k (bd)^k (|Sigma|=4, n=12k, r=O(1))
2. Quantitative analysis on ALL 6 SHA-256-authenticated Wikidata blocks:
   - Byte-transition graph anatomy (total non-self bigrams, GUID hex clique K_16 traffic, SCC/FVS)
   - Exact Alpha Redheffer Star-Product (verify_btm_v2) across multiple merge tree policies
     (balanced, rarest_root, heavy_edge, max_bigram_agglom, min_bounce_greedy)
   - Pinpoint the EXACT irreducible ping-pong core (top merge nodes by bounces & lookups!).
"""
import hashlib
import json
import os
import sys
import time
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import verify_btm_v2 as btm


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


def all_binary_trees(items):
    """Generate all (2m - 3)!! distinct unordered full binary trees over `items`."""
    items = list(items)
    m = len(items)
    if m == 1:
        yield items[0]
        return
    # Fix items[0] in the left subtree to avoid (L, R) vs (R, L) duplicates
    first = items[0]
    rest = items[1:]
    # Choose non-empty proper subset of rest for right subtree (so right has 1..m-1 elements)
    r_len = len(rest)
    for mask in range(1, 1 << r_len):
        left_items = [first] + [rest[i] for i in range(r_len) if not ((mask >> i) & 1)]
        right_items = [rest[i] for i in range(r_len) if ((mask >> i) & 1)]
        for lt in all_binary_trees(left_items):
            for rt in all_binary_trees(right_items):
                yield (lt, rt)


def eval_tree_exact(L_bytes, primary_idx, tree):
    """Run Alpha's exact Redheffer Star-Product along a specified binary tree topology."""
    n, buckets, pieces_by_bucket, bigrams, slp = btm.build_initial_from_L(L_bytes, primary_idx)
    stats = defaultdict(int)
    stats["r0"] = sum(len(v) for v in pieces_by_bucket.values())

    leaf_nodes = {}
    primary_cycle = None
    for c, b_lo, b_hi in buckets:
        s = frozenset([c])
        out_p, pcyc = btm.resolve_subset(s, pieces_by_bucket[c], slp, stats)
        if pcyc is not None:
            primary_cycle = pcyc
        leaf_nodes[c] = (s, out_p)

    leaf_bounces = stats["bounces"]
    leaf_lookups = stats["lookups"]

    def rec(node):
        nonlocal primary_cycle
        if not isinstance(node, tuple):
            return leaf_nodes[node]
        s1, p1 = rec(node[0])
        s2, p2 = rec(node[1])
        s = s1 | s2
        out_p, pcyc = btm.resolve_subset(s, p1 + p2, slp, stats)
        if pcyc is not None:
            primary_cycle = pcyc
        return (s, out_p)

    root_s, root_p = rec(tree)
    assert len(root_p) == 0 and primary_cycle is not None
    c_slp, mark_off = primary_cycle
    cyc = slp.expand_bytes(c_slp)
    q = len(cyc)
    rot = cyc[mark_off:] + cyc[:mark_off]
    text = ((rot * (n // q + 1))[:n])[::-1]

    merge_bounces = stats["bounces"] - leaf_bounces
    merge_lookups = stats["lookups"] - leaf_lookups
    root_steps = stats["root_cycle_steps"]
    return text, {
        "merge_bounces": merge_bounces,
        "root_steps": root_steps,
        "total_pp": merge_bounces + root_steps,
        "merge_lookups": merge_lookups,
        "total_work": merge_lookups + root_steps,
        "slp_joins": stats["slp_joins"],
        "spirals": stats["spirals"],
    }


def check_all_trees(label, T: bytes):
    L, p = btm.bwt_encode(T)
    n = len(T)
    r = sum(1 for i in range(1, n) if L[i] != L[i - 1]) + 1
    syms = sorted(set(T))
    trees = list(all_binary_trees(syms))
    min_pp = 10**9
    max_pp = 0
    min_mb = 10**9
    min_work = 10**9
    min_tree = None
    for tr in trees:
        got, st = eval_tree_exact(L, p, tr)
        assert got == T, f"EXACTNESS FAIL on {label}!"
        if st["total_pp"] < min_pp:
            min_pp = st["total_pp"]
            min_tree = tr
        if st["total_pp"] > max_pp:
            max_pp = st["total_pp"]
        min_mb = min(min_mb, st["merge_bounces"])
        min_work = min(min_work, st["total_work"])
    print(
        f"[{label:26s}] n={n:4d} r={r:4d} (r/n={r/n:.4f}) |Sigma|={len(syms)} ({len(trees):2d} trees) | "
        f"MIN over ALL trees: merge_bounces>={min_mb:4d}, total_pp>={min_pp:4d} ({min_pp/n:.3f}n, {min_pp/r:6.2f}r), "
        f"total_work>={min_work:4d} ({min_work/n:.3f}n) | MAX pp={max_pp:4d}"
    )


def analyze_wikidata_block(entry):
    path = os.path.join("/tmp/mibz2-blocks", entry["file"])
    L = open(path, "rb").read()
    assert hashlib.sha256(L).hexdigest() == entry["sha256"], f"SHA mismatch {entry['file']}"
    p = entry["primary_index"]
    n = len(L)
    r = entry["r"]

    _, buckets, pieces_by_bucket, bigrams, _ = btm.build_initial_from_L(L, p)
    total_nonself = sum(w for (a, b), w in bigrams.items() if a != b)
    total_self = sum(w for (a, b), w in bigrams.items() if a == b)

    # Analyze subgraphs: hex lower {0..9,a..f}, hex upper {0..9,A..F}, digits {0..9}, ascii lower, json punct
    hex_lo = set(b"0123456789abcdef")
    hex_hi = set(b"0123456789ABCDEF")
    hex_all = hex_lo | hex_hi
    digits = set(b"0123456789")
    json_p = set(b'"{}[],:-')

    def subgraph_stats(S):
        internal_nonself = sum(w for (a, b), w in bigrams.items() if a in S and b in S and a != b)
        internal_self = sum(w for (a, b), w in bigrams.items() if a in S and b in S and a == b)
        edges = sum(1 for (a, b), w in bigrams.items() if a in S and b in S and a != b and w > 0)
        present = len([c for c, _, _ in buckets if c in S])
        return present, edges, internal_nonself, internal_self

    print(f"\n==================== {entry['file']} (n={n}, r={r}, |Sigma|={len(buckets)}) ====================")
    print(f"  Total bigrams: non-self={total_nonself} ({total_nonself/n:.3f}n), self={total_self} ({total_self/n:.3f}n)")
    for sname, S in [
        ("digits 0-9", digits),
        ("hex_lo 0-9a-f", hex_lo),
        ("hex_hi 0-9A-F", hex_hi),
        ("hex_all", hex_all),
        ("json punct", json_p),
    ]:
        pr, ed, ins, iself = subgraph_stats(S)
        max_ed = pr * (pr - 1) if pr > 1 else 1
        print(f"  Subgraph {sname:14s}: |V|={pr:2d}, dir_edges={ed:3d}/{max_ed:3d} ({ed/max_ed:.2f}), non-self={ins:7d} ({ins/n:.3f}n), self={iself:6d}")

    # Run Alpha's decode_btm_v2 with detailed per-merge logging for 4 tree modes:
    # balanced, rarest_root, heavy_edge, AND pure max-weight agglomerative (ignoring piece count)
    ref = None
    for mode in ("balanced", "rarest_root", "heavy_edge", "max_wt_agglom"):
        t0 = time.time()
        n0, bkts, pbb, bgs, slp = btm.build_initial_from_L(L, p)
        stats = defaultdict(int)
        nodes = {}
        primary_cycle = None
        for c, blo, bhi in bkts:
            s = frozenset([c])
            out_p, pcyc = btm.resolve_subset(s, pbb[c], slp, stats)
            if pcyc is not None:
                primary_cycle = pcyc
            nodes[c] = (s, out_p, bhi - blo)
        leaf_b = stats["bounces"]
        leaf_l = stats["lookups"]

        merge_records = []
        def do_m(n1, n2):
            nonlocal primary_cycle
            s1, p1, c1 = n1
            s2, p2, c2 = n2
            s = s1 | s2
            b0, l0 = stats["bounces"], stats["lookups"]
            out_p, pcyc = btm.resolve_subset(s, p1 + p2, slp, stats)
            if pcyc is not None:
                primary_cycle = pcyc
            db = stats["bounces"] - b0
            dl = stats["lookups"] - l0
            # Count exact cross-bigrams between s1 and s2
            cross_w = sum(w for (a, b), w in bgs.items() if (a in s1 and b in s2) or (a in s2 and b in s1))
            merge_records.append((db, dl, len(s1), len(s2), len(p1) + len(p2), len(out_p), cross_w, s1, s2))
            return (s, out_p, c1 + c2)

        if mode == "balanced":
            ordered = [nodes[c] for c, _, _ in bkts]
            def rec(arr):
                if len(arr) == 1: return arr[0]
                m = len(arr) // 2
                return do_m(rec(arr[:m]), rec(arr[m:]))
            root = rec(ordered)
        elif mode == "rarest_root":
            ordered = sorted([nodes[c] for c, _, _ in bkts], key=lambda x: -x[2])
            rarest = ordered.pop()
            def rec(arr):
                if len(arr) == 1: return arr[0]
                m = len(arr) // 2
                return do_m(rec(arr[:m]), rec(arr[m:]))
            root = do_m(rec(ordered), rarest)
        elif mode in ("heavy_edge", "max_wt_agglom"):
            active = {i: nodes[c] for i, (c, _, _) in enumerate(bkts)}
            sym_to_cid = {c: i for i, (c, _, _) in enumerate(bkts)}
            w_adj = defaultdict(lambda: defaultdict(int))
            for (a, b), wt in bgs.items():
                if a != b:
                    ca, cb = sym_to_cid[a], sym_to_cid[b]
                    w_adj[ca][cb] += wt
                    w_adj[cb][ca] += wt
            next_cid = len(active)
            while len(active) > 1:
                best_pair = None
                best_score = -1.0
                cids = list(active.keys())
                for u in cids:
                    nu = active[u]
                    for v, wt in w_adj[u].items():
                        if v > u and v in active:
                            nv = active[v]
                            if mode == "heavy_edge":
                                score = wt / (len(nu[1]) + len(nv[1]) + 1.0)
                            else:
                                # Normalized cut / conductance: max wt / min(cnt_u, cnt_v)
                                score = wt / (min(nu[2], nv[2]) + 1.0)
                            if score > best_score:
                                best_score = score
                                best_pair = (u, v)
                if best_pair is None:
                    best_pair = (cids[0], cids[1])
                u, v = best_pair
                merged = do_m(active.pop(u), active.pop(v))
                cid = next_cid
                next_cid += 1
                active[cid] = merged
                new_row = defaultdict(int)
                for nb, wt in w_adj[u].items():
                    if nb != v and nb in active:
                        new_row[nb] += wt
                for nb, wt in w_adj[v].items():
                    if nb != u and nb in active:
                        new_row[nb] += wt
                del w_adj[u]; del w_adj[v]
                for nb, wt in new_row.items():
                    w_adj[cid][nb] = wt; w_adj[nb][cid] = wt
                    w_adj[nb].pop(u, None); w_adj[nb].pop(v, None)
            root = next(iter(active.values()))

        dt = time.time() - t0
        mb = stats["bounces"] - leaf_b
        ml = stats["lookups"] - leaf_l
        rc = stats["root_cycle_steps"]
        merge_records.sort(key=lambda x: -x[0])
        top3_str = []
        for db, dl, ls1, ls2, pin, pout, cw, s1, s2 in merge_records[:3]:
            def fmt_s(ss):
                chs = "".join(chr(c) if 32 <= c < 127 else f"\\x{c:02x}" for c in sorted(ss))
                return chs if len(chs) <= 14 else f"{chs[:12]}..({len(ss)})"
            top3_str.append(f"[{fmt_s(s1)}|{fmt_s(s2)}: b={db}, l={dl}, cw={cw}]")
        print(
            f"  tree={mode:13s} ({dt:4.2f}s) | merge_bounces={mb:7d} ({mb/r:5.2f}r, {mb/n:.3f}n) | "
            f"lookups={ml:7d} ({ml/r:5.2f}r, {ml/n:.3f}n) | root_cyc={rc:5d} | "
            f"tot_pp={mb+rc:7d} ({(mb+rc)/r:5.2f}r)"
        )
        print(f"    top3 merges: {' ; '.join(top3_str)}")


def main():
    print("=== PART 1: EXHAUSTIVE ALL-TREES ADVERSARIAL PING-PONG WITNESSES ===")
    # 1. Smallest natural K_3 Euler tour (n=6, |Sigma|=3)
    check_all_trees("K3 Euler 'abcacb'", b"abcacb")
    # 2. Ternary De Bruijn B(3, k)
    for k in [2, 3, 4, 5]:
        T = bytes(ord('a') + x for x in de_bruijn(3, k))
        check_all_trees(f"DeBruijn B(3,{k})", T)
    # 3. Quaternary De Bruijn B(4, 2..3) (15 trees!)
    for k in [2, 3]:
        T = bytes(ord('a') + x for x in de_bruijn(4, k))
        check_all_trees(f"DeBruijn B(4,{k})", T)
    # 4. Constant-r ping-pong killer: T_k = (ab)^k (bc)^k (ca)^k (|Sigma|=3, r=7 CONSTANT!)
    for k in [2, 4, 16, 64, 256]:
        T = b"ab" * k + b"bc" * k + b"ca" * k
        check_all_trees(f"AltRing (ab)^k(bc)^k(ca)^k k={k}", T)
    # 5. Constant-r K_4 killer (|Sigma|=4, 15 trees!)
    for k in [2, 8, 32]:
        T = b"ab" * k + b"bc" * k + b"cd" * k + b"da" * k + b"ac" * k + b"bd" * k
        check_all_trees(f"K4-AltRing k={k}", T)

    print("\n=== PART 2: QUANTITATIVE PING-PONG AUDIT ON ALL 6 AUTHENTICATED WIKIDATA BLOCKS ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    for entry in manifest["blocks"]:
        analyze_wikidata_block(entry)


if __name__ == "__main__":
    main()
