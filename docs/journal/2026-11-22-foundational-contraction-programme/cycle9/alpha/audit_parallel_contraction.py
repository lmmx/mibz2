#!/usr/bin/env python3
"""
Cycle 9 — Task 2:
Exact Parallel List & Tree Contraction Census (Shun-Gu-Blelloch-Fineman-Gibbons vs.
Cong-Bader Acquisition Warning) on ALL 6 Authenticated Wikidata Blocks!

Measures exact critical-path depth D, tree/chain distributions, and parallel work inflation
(W_par / W_seq) for:
  1. AP4_max gradient forest (cap = inf AND cap = 15)
  2. AP2_asc monotone forest (cap = inf AND cap = 15)
  3. M1 (R1) chain forest (cap = inf AND cap = 8)
  4. Exact Parallel Pointer-Jumping (PJ): rounds = ceil(log2(max_depth)), exact total work W_PJ!
  5. Exact SGBFG Rake-Compress (deterministic alternating rake leaves + compress deg-2 paths):
     exact rounds D_RC and exact total work W_RC!
  6. Evaluates Theorem P1 (Inter-Block vs. Intra-Block Parallelism Law) against i9 OpenCells!
"""
import hashlib
import json
import math
import sys
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import bwt_core as am
import bwt_core as tm


def analyze_forest_parallel(par, active_mask):
    """
    Given an acyclic forest `par[u] = v` (where `par[u] == -1` for roots/inactive nodes)
    over nodes with `active_mask[u] == True`:
      1. Computes exact depth[u] of every active node (distance to root `par == -1`),
         #trees (roots with >= 1 child), max_depth, p95_depth, p99_depth, mean_depth.
      2. Simulates EXACT Parallel Pointer-Jumping (each round: for all active u with par[u] >= 0
         and par[par[u]] >= 0: par[u] = par[par[u]], cost +1 pointer-jump op!).
         Measures exact PJ rounds D_PJ and exact PJ total ops W_PJ (vs serial W_seq = #active)!
      3. Simulates EXACT SGBFG Rake-Compress (each round:
           - Rake: delete all leaves (in_deg == 0 among active nodes), cost +1 per raked leaf;
           - Compress: contract independent set of deg-2 path nodes (in_deg == 1, out_deg == 1),
             picking even-parity / 2-colored independent set -> shrinks paths by ~1/2 each round!).
         Measures exact RC rounds D_RC and exact RC total ops W_RC!
    """
    q = len(par)
    # Break any cycles cleanly first
    p = list(par)
    color = [0] * q
    for u in range(q):
        if color[u] != 0 or p[u] < 0: continue
        path = []; cur = u
        while cur >= 0 and color[cur] == 0:
            color[cur] = 1; path.append(cur); cur = p[cur]
        if cur >= 0 and color[cur] == 1: p[cur] = -1
        for x in path: color[x] = 2

    # Compute exact depths
    depth = [0] * q
    vis = [False] * q
    roots_with_kids = set()
    active_nodes = []
    for u in range(q):
        if p[u] >= 0:
            active_nodes.append(u)
    w_seq = len(active_nodes)

    for u in active_nodes:
        if vis[u]: continue
        st = []; cur = u
        while cur >= 0 and p[cur] >= 0 and not vis[cur]:
            st.append(cur); cur = p[cur]
        base_d = depth[cur] if (cur >= 0 and vis[cur]) else 0
        if cur >= 0 and p[cur] < 0:
            roots_with_kids.add(cur)
        for nd in reversed(st):
            base_d += 1
            depth[nd] = base_d
            vis[nd] = True

    depths = sorted(depth[u] for u in active_nodes)
    max_d = depths[-1] if depths else 0
    p95_d = depths[int(0.95 * (len(depths) - 1))] if depths else 0
    p99_d = depths[int(0.99 * (len(depths) - 1))] if depths else 0
    mean_d = sum(depths) / max(1, len(depths))

    # Exact Parallel Pointer-Jumping simulation!
    pj_par = list(p)
    pj_rounds = 0
    pj_work = 0
    # Nodes still needing jumps: those whose parent is NOT yet a root (pj_par[pj_par[u]] >= 0)
    live_pj = [u for u in active_nodes if pj_par[pj_par[u]] >= 0]
    pj_work += w_seq  # initial 1-hop read of par[u]
    while live_pj:
        pj_rounds += 1
        nxt_live = []
        new_p = {}
        for u in live_pj:
            v = pj_par[u]
            pv = pj_par[v]
            pj_work += 1  # 1 synchronous word-concat + pointer-jump op
            new_p[u] = pv
        for u, pv in new_p.items():
            pj_par[u] = pv
            if pj_par[pv] >= 0:
                nxt_live.append(u)
        live_pj = nxt_live

    # Exact SGBFG Rake-Compress simulation on active forest!
    # Maintain active set + in_deg (number of active children)
    rc_par = list(p)
    alive = {u for u in active_nodes}
    in_deg = defaultdict(int)
    for u in alive:
        v = rc_par[u]
        if v in alive:
            in_deg[v] += 1
    rc_rounds = 0
    rc_work = 0
    while alive:
        rc_rounds += 1
        # Step A: Rake ALL leaves (in_deg == 0) simultaneously!
        leaves = [u for u in alive if in_deg[u] == 0]
        # Step B: Compress maximal independent set of internal deg-1-child nodes (in_deg == 1)!
        # (Use deterministic parity/ID rule: u compresses with par[u] if in_deg[u]==1, par[u] in alive,
        #  in_deg[par[u]]==1, and u < par[u] or alternating!)
        raked_set = set(leaves)
        comp_pairs = []
        used = set(raked_set)
        for u in alive:
            if u in used or in_deg[u] != 1:
                continue
            v = rc_par[u]
            if v in alive and v not in used and in_deg[v] == 1:
                comp_pairs.append((u, v))
                used.add(u); used.add(v)
        # Apply rakes
        for u in leaves:
            rc_work += 1
            v = rc_par[u]
            alive.remove(u)
            if v in alive:
                in_deg[v] -= 1
        # Apply compressions (contract u into v: u's child now points to v, u removed -> Wait: u's parent v stays!)
        # Actually to update child->u->v to child->v, track child ptr for in_deg==1 nodes!
        # Wait: simpler & exact — if we remove v (the parent) and let u point to rc_par[v],
        # no child pointer needs updating! Check: u is the UNIQUE child of v (since in_deg[v] == 1)!
        for u, v in comp_pairs:
            if u in alive and v in alive and in_deg[v] == 1 and rc_par[u] == v:
                rc_work += 1
                rc_par[u] = rc_par[v]
                alive.remove(v)
                in_deg[v] = 0
                # Note: rc_par[v]'s in_deg is unchanged (v replaced by u!)

    return {
        "w_seq": w_seq,
        "trees": len(roots_with_kids),
        "max_d": max_d,
        "p95_d": p95_d,
        "p99_d": p99_d,
        "mean_d": mean_d,
        "pj_rounds": pj_rounds,
        "pj_work": pj_work,
        "pj_ratio": pj_work / max(1, w_seq),
        "rc_rounds": rc_rounds,
        "rc_work": rc_work,
        "rc_ratio": rc_work / max(1, w_seq),
    }


def main():
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    blocks = []
    for entry in manifest["blocks"]:
        with open(f"/tmp/mibz2-blocks/{entry['file']}", "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        n, r0, m16, strips, j_order, C = am.load_runs_and_open_baseline(L, entry["primary_index"])
        od, id_, ot, ins, nr1 = tm.build_full_incidence_256cursor(strips, j_order)
        blocks.append((entry, n, r0, m16, strips, ot, nr1))

    print("=== PARALLEL LIST & TREE CONTRACTION CENSUS ACROSS ALL 6 WIKIDATA BLOCKS ===")
    for name, fn_mode in [
        ("AP4_max (ALL out1 w<=16, uncapped)", "ap4"),
        ("AP2_asc (out1 & d>0, w<=16, uncapped)", "ap2"),
        ("M1 (Static R1 chains, w<=16, uncapped)", "m1"),
    ]:
        print(f"\n--- {name} ---")
        print("block       |  w_seq | trees | maxD  p95  p99 meanD | PJ_rnd  PJ_W (xSeq) | RC_rnd  RC_W (xSeq)")
        print("------------+--------+-------+----------------------+---------------------+--------------------")
        sw = spj = src = 0
        max_pj_r = max_rc_r = max_all_d = 0
        for entry, n, r0, m16, strips, ot, nr1 in blocks:
            q0 = len(strips)
            if fn_mode == "ap4":
                par = [ot[u] if (ot[u] >= 0 and ot[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0) else -1 for u in range(q0)]
            elif fn_mode == "ap2":
                par = [ot[u] if (ot[u] >= 0 and ot[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0 and strips[u][2] > 0) else -1 for u in range(q0)]
            else:
                par = [nr1[u] if (nr1[u] >= 0 and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0) else -1 for u in range(q0)]
            res = analyze_forest_parallel(par, [p >= 0 for p in par])
            sw += res["w_seq"]; spj += res["pj_work"]; src += res["rc_work"]
            max_pj_r = max(max_pj_r, res["pj_rounds"])
            max_rc_r = max(max_rc_r, res["rc_rounds"])
            max_all_d = max(max_all_d, res["max_d"])
            print(f"{entry['file']:11s} | {res['w_seq']:6d} | {res['trees']:5d} | {res['max_d']:4d} {res['p95_d']:4d} {res['p99_d']:4d} {res['mean_d']:5.1f} | "
                  f"  {res['pj_rounds']:3d}  {res['pj_work']:7d} ({res['pj_ratio']:4.2f}x) | "
                  f"  {res['rc_rounds']:3d}  {res['rc_work']:6d} ({res['rc_ratio']:4.2f}x)")
        print("------------+--------+-------+----------------------+---------------------+--------------------")
        print(f"SUM/MAX     | {sw:6d} |       | {max_all_d:4d}                 |   {max_pj_r:3d}  {spj:7d} ({spj/sw:4.2f}x) | "
              f"  {max_rc_r:3d}  {src:6d} ({src/sw:4.2f}x)")


if __name__ == "__main__":
    main()
