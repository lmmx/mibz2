#!/usr/bin/env python3
"""
Test width-leverage & piece-cost transition-graph merge trees on all 6 Wikidata blocks:
Can a merge tree over Sigma achieve Q_rand < 1.00 * m_16?
Also inspect exact piece-count per depth vs O(r_0 log |Sigma|) and width-1 floor.
"""
import hashlib
import json
import sys
import time
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import measure_all_trees as mt


def build_leverage_tree(buckets, pieces_by_bucket, bigrams, mode="avg_width"):
    """
    Agglomerative merge tree using EXACT cross-cluster (rows, pieces) statistics from I_0!
    - avg_width:  maximize rows(u,v) / pieces(u,v)  (thick-run pairs FIRST, width-1 GUIDs LAST!)
    - rows_over_p2: maximize rows(u,v) / (pieces(u,v) * (sz[u] + sz[v]))
    - min_pieces: merge pair minimizing pieces(u,v) * (1 / max(1, rows(u,v)/pieces(u,v)))
    - min_cut_piece: recursive balanced min-cut weighted by PIECE count (not row count!)
    """
    syms = [c for c, _, _ in buckets]
    if len(syms) == 1:
        return syms[0]
    sym_to_id = {c: i for i, c in enumerate(syms)}

    # Count initial directed pieces and rows between every pair of symbols (bc -> tsym)
    row_adj = defaultdict(lambda: defaultdict(int))
    pcs_adj = defaultdict(lambda: defaultdict(int))
    tot_pcs = {}
    for c, _, _ in buckets:
        u = sym_to_id[c]
        tot_pcs[u] = len(pieces_by_bucket[c])
        for lo, hi, d, tsym, w, m in pieces_by_bucket[c]:
            if tsym != c:
                v = sym_to_id[tsym]
                ln = hi - lo
                row_adj[u][v] += ln
                row_adj[v][u] += ln
                pcs_adj[u][v] += 1
                pcs_adj[v][u] += 1

    active = {i: syms[i] for i in range(len(syms))}
    sz = dict(tot_pcs)
    nxt_id = len(syms)

    # First, isolate the rarest symbol (min total degree) for the root!
    rarest = min(active.keys(), key=lambda u: sum(row_adj[u].values()))
    rarest_sym = active.pop(rarest)
    for nb in list(row_adj[rarest].keys()):
        row_adj[nb].pop(rarest, None)
        pcs_adj[nb].pop(rarest, None)
    del row_adj[rarest]; del pcs_adj[rarest]

    while len(active) > 1:
        best_pair = None
        best_score = -1.0
        cids = list(active.keys())
        for u in cids:
            for v, rwt in row_adj[u].items():
                if v > u and v in active:
                    pwt = pcs_adj[u][v]
                    if mode == "avg_width":
                        # Average width of crossing pieces: thick transitions first!
                        sc = rwt / (pwt + 0.5)
                    elif mode == "width_times_rows":
                        # (rwt / pwt) * log(1 + rwt) / (sz[u] + sz[v])
                        sc = (rwt * rwt) / ((pwt + 1.0) * (sz[u] + sz[v] + 1.0))
                    elif mode == "min_split":
                        # Absorb highest fraction of cluster's pieces into thick runs
                        sc = rwt / ((pwt + 1.0) * (min(sz[u], sz[v]) + 1.0))
                    elif mode == "thick_then_rare":
                        # Pure row/piece ratio with slight tie-break toward smaller clusters
                        sc = (rwt / (pwt + 1.0)) / (1.0 + 0.0005 * (sz[u] + sz[v]))
                    else:
                        raise ValueError(mode)
                    if sc > best_score:
                        best_score = sc
                        best_pair = (u, v)
        if best_pair is None:
            best_pair = (cids[0], cids[1])
        u, v = best_pair
        tu, tv = active.pop(u), active.pop(v)
        su, sv = sz.pop(u), sz.pop(v)
        cid = nxt_id
        nxt_id += 1
        active[cid] = (tu, tv)
        sz[cid] = su + sv

        new_r = defaultdict(int)
        new_p = defaultdict(int)
        for nb, wt in row_adj[u].items():
            if nb != v and nb in active:
                new_r[nb] += wt
                new_p[nb] += pcs_adj[u][nb]
        for nb, wt in row_adj[v].items():
            if nb != u and nb in active:
                new_r[nb] += wt
                new_p[nb] += pcs_adj[v][nb]
        del row_adj[u]; del row_adj[v]
        del pcs_adj[u]; del pcs_adj[v]
        for nb in new_r:
            row_adj[cid][nb] = new_r[nb]; row_adj[nb][cid] = new_r[nb]
            pcs_adj[cid][nb] = new_p[nb]; pcs_adj[nb][cid] = new_p[nb]
            row_adj[nb].pop(u, None); row_adj[nb].pop(v, None)
            pcs_adj[nb].pop(u, None); pcs_adj[nb].pop(v, None)

    rest_tree = next(iter(active.values()))
    return (rest_tree, rarest_sym)


def eval_custom_tree(L_bytes, primary_idx, tree_mode, ping_mode="P1"):
    n, r0, m_16, buckets, pieces_by_bucket, bigrams, slp = mt.acquire_from_L(L_bytes, primary_idx)
    stats = defaultdict(int)
    stats["n"] = n; stats["r0"] = r0; stats["m_16"] = m_16; stats["sigma"] = len(buckets)

    # Count width-1 runs in L
    w1_runs = 0
    for c, _, _ in buckets:
        for lo, hi, *_ in pieces_by_bucket[c]:
            if hi - lo == 1:
                w1_runs += 1
    stats["w1_pieces"] = w1_runs

    leaves = {}
    primary_cycle = None
    for c, b_lo, b_hi in buckets:
        s = frozenset([c])
        out_p, pcyc = mt.resolve_subset_ablation(s, pieces_by_bucket[c], [], slp, stats, ping_mode=ping_mode)
        if pcyc is not None:
            primary_cycle = pcyc
        leaves[c] = (s, out_p)
        stats["leaf_pieces"] += len(out_p)

    tree = build_leverage_tree(buckets, pieces_by_bucket, bigrams, mode=tree_mode)
    max_depth = 0
    sum_p = stats["leaf_pieces"]

    def eval_t(node, depth=0):
        nonlocal primary_cycle, max_depth, sum_p
        if depth > max_depth:
            max_depth = depth
        if isinstance(node, int):
            return leaves[node]
        lr = eval_t(node[0], depth + 1)
        rr = eval_t(node[1], depth + 1)
        s = lr[0] | rr[0]
        out_p, pcyc = mt.resolve_subset_ablation(s, lr[1], rr[1], slp, stats, ping_mode=ping_mode)
        if pcyc is not None:
            primary_cycle = pcyc
        sum_p += len(out_p)
        return (s, out_p)

    root = eval_t(tree, 0)
    assert len(root[1]) == 0 and primary_cycle is not None
    c_slp, mark_off = primary_cycle
    cyc, sm = slp.analyze_and_expand_memo(c_slp)
    q = len(cyc)
    rot = cyc[mark_off:] + cyc[:mark_off]
    text = ((rot * (n // q + 1))[:n])[::-1]
    stats["tree_depth"] = max_depth
    stats["sum_P"] = sum_p
    stats["slp"] = sm
    return text, dict(stats)


def main():
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    # Test 4 leverage objectives on block-0.L first, then run best on ALL 6 blocks!
    with open("/tmp/mibz2-blocks/block-0.L", "rb") as f:
        L0 = f.read()
    e0 = manifest["blocks"][0]
    ref0 = mt.ref_ibwt(L0, e0["primary_index"])

    print("=== Leverage Tree Objectives on block-0.L (P1) ===")
    for tm in ["avg_width", "width_times_rows", "min_split", "thick_then_rare"]:
        dec, st = eval_custom_tree(L0, e0["primary_index"], tm, "P1")
        assert dec == ref0
        q = st["Q_rand"]; r0 = st["r0"]; m16 = st["m_16"]
        sm = st["slp"]
        print(f"  {tm:18s} | d={st['tree_depth']:2d} sumP={st['sum_P']:7d}({st['sum_P']/r0:5.2f}r) "
              f"Q={q:7d} ({q/r0:4.2f}r, {q/m16:5.3f}m16) w1={st['w1_pieces']:5d} "
              f"|G|={sm['reach_nonlit']:6d} h={sm['height']:4d} avgL={sm['avg_copy_len']:5.1f}")

    print("\n=== ALL 6 BLOCKS: Best Leverage Trees (avg_width & width_times_rows) ===")
    for entry in manifest["blocks"]:
        with open(f"/tmp/mibz2-blocks/{entry['file']}", "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        p = entry["primary_index"]
        ref = mt.ref_ibwt(L, p)
        for tm in ["avg_width", "width_times_rows"]:
            dec, st = eval_custom_tree(L, p, tm, "P1")
            assert dec == ref
            q = st["Q_rand"]; r0 = st["r0"]; m16 = st["m_16"]
            sm = st["slp"]
            print(f"  {entry['file']:11s} {tm:16s} | r0={r0:6d} m16={m16:6d} w1={st['w1_pieces']:5d} "
                  f"d={st['tree_depth']:2d} sumP={st['sum_P']:7d}({st['sum_P']/r0:5.2f}r) "
                  f"Q={q:7d}({q/r0:4.2f}r,{q/m16:5.3f}m16) |G|={sm['reach_nonlit']:6d} h={sm['height']:4d} avgL={sm['avg_copy_len']:4.1f}")


if __name__ == "__main__":
    main()
