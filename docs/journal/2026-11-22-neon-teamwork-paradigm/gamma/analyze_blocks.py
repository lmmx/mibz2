#!/usr/bin/env python3
"""
Diagnostic measurements on the 6 authenticated Wikidata BWT blocks in /tmp/mibz2-blocks/.
Verifies SHA-256 against manifest.json, builds exact Open retained permutation G (LIMIT=16),
and measures exact Multi-Frontal Bucket-Order Schur Elimination (MF-BSE) statistics.
"""
import hashlib
import json
import os
import time


def verify_and_load_blocks():
    manifest_path = "/tmp/mibz2-blocks/manifest.json"
    with open(manifest_path) as f:
        manifest = json.load(f)
    blocks = []
    for entry in manifest["blocks"]:
        path = os.path.join("/tmp/mibz2-blocks", entry["file"])
        data = open(path, "rb").read()
        sha = hashlib.sha256(data).hexdigest()
        assert sha == entry["sha256"], f"SHA mismatch for {entry['file']}"
        assert len(data) == entry["n"]
        blocks.append((entry, data))
    return blocks


def compute_runs(data):
    n = len(data)
    runs = []  # (start, length, sym)
    i = 0
    while i < n:
        b = data[i]
        j = i + 1
        while j < n and data[j] == b:
            j += 1
        runs.append((i, j - i, b))
        i = j
    return runs


def compute_LF(data):
    n = len(data)
    cnt = [0] * 256
    for b in data:
        cnt[b] += 1
    C = [0] * 256
    s = 0
    for i in range(256):
        C[i] = s
        s += cnt[i]
    cur = C[:]
    LF = [0] * n
    for i, b in enumerate(data):
        LF[i] = cur[b]
        cur[b] += 1
    return LF, C


def build_open_retained_G(data, p, LF, limit=16):
    """Build exact Open retained set S (thin runs width <= limit, plus primary row p)
    and exact first-return permutation G on [0..m) with word lengths w[0..m)."""
    n = len(data)
    runs = compute_runs(data)
    is_retained = bytearray(n)
    w1_count = 0
    thin_runs = 0
    thick_runs = 0
    for start, length, _ in runs:
        if length == 1:
            w1_count += 1
        if length <= limit:
            thin_runs += 1
            for row in range(start, start + length):
                is_retained[row] = 1
        else:
            thick_runs += 1
    is_retained[p] = 1

    # Map retained rows to dense indices 0..m-1 in row order
    row_to_idx = [-1] * n
    retained_rows = []
    for row in range(n):
        if is_retained[row]:
            row_to_idx[row] = len(retained_rows)
            retained_rows.append(row)
    m = len(retained_rows)
    p_idx = row_to_idx[p]

    # Compute exact first return g(u) and step length w(u) by following LF from each retained row
    # (for diagnostic measurement; in production Open computes this in O(H) via endpoint induction)
    G = [0] * m
    W = [0] * m
    for u, row in enumerate(retained_rows):
        cur = LF[row]
        steps = 1
        while not is_retained[cur]:
            cur = LF[cur]
            steps += 1
        G[u] = row_to_idx[cur]
        W[u] = steps

    return m, p_idx, G, W, len(runs), w1_count, thin_runs, thick_runs


def simulate_mfbse(G, W, p_idx, K):
    """Exact Multi-Frontal Bucket-Order Schur Elimination (MF-BSE) on permutation G[0..m)
    with edge weights W[0..m) and marked vertex p_idx, using K contiguous buckets.
    Verifies exact cycle positions pos[0..m) against ground-truth cycle walk."""
    m = len(G)
    S = (m + K - 1) // K

    def bkt_of(v):
        return v // S

    # Keep p_idx in its bucket, but mark it un-eliminable so it survives to the end
    # (equivalently, reindex p_idx to m-1; here we simply never eliminate p_idx)
    # Forward buckets: list of (u, v, weight)
    fwd_buckets = [[] for _ in range(K)]
    initial_intra = 0
    for u in range(m):
        v = G[u]
        bu = bkt_of(u)
        bv = bkt_of(v)
        if bu == bv:
            initial_intra += 1
        # Place edge (u -> v) in bucket of the endpoint eliminated FIRST.
        # Since p_idx is never eliminated until final step, treat p_idx as bucket K.
        eu = K if u == p_idx else bu
        ev = K if v == p_idx else bv
        target = eu if eu < ev else ev
        if target == K:
            # self-loop p_idx -> p_idx
            fwd_buckets[K - 1].append((u, v, W[u]))
        else:
            fwd_buckets[target].append((u, v, W[u]))

    # Track exact statistics
    max_bucket_edges = 0
    total_bucket_edges_read = 0
    total_shortcuts_emitted = 0
    total_intra_contracted = 0
    max_chain_len = 0

    # Elimination log per bucket k: list of chains (parent_u, list of (x, offset_from_u))
    # Plus backward request queue req[b] = list of (u, src_bkt, chain_idx)
    chain_logs = [[] for _ in range(K)]
    req_buckets = [[] for _ in range(K + 1)]

    # Surviving self-loop on p_idx
    p_cycle_len = None

    for k in range(K):
        edges = fwd_buckets[k]
        fwd_buckets[k] = None  # free
        n_edges = len(edges)
        total_bucket_edges_read += n_edges
        if n_edges > max_bucket_edges:
            max_bucket_edges = n_edges

        lo = k * S
        hi = min(m, (k + 1) * S)
        sz = hi - lo

        # Local L1 arrays for vertices in B_k
        in_u = [-1] * sz
        in_w = [0] * sz
        out_v = [-1] * sz
        out_w = [0] * sz

        for u, v, w in edges:
            if u == p_idx and v == p_idx:
                p_cycle_len = w
                continue
            if lo <= u < hi and u != p_idx:
                out_v[u - lo] = v
                out_w[u - lo] = w
            if lo <= v < hi and v != p_idx:
                in_u[v - lo] = u
                in_w[v - lo] = w

        visited = bytearray(sz)

        # 1. Start from every chain head x in B_k whose predecessor u is NOT in B_k (or is p_idx)
        for idx in range(sz):
            x = lo + idx
            if x == p_idx:
                continue
            u = in_u[idx]
            if u == -1:
                continue  # shouldn't happen on valid permutation
            if not (lo <= u < hi and u != p_idx):
                # Chain starts here: u -> x -> ... -> w (where w not in B_k or w == p_idx)
                cur_x = x
                acc_w = in_w[idx]
                chain_nodes = []
                while True:
                    cidx = cur_x - lo
                    visited[cidx] = 1
                    chain_nodes.append((cur_x, acc_w))
                    nxt = out_v[cidx]
                    acc_w += out_w[cidx]
                    if lo <= nxt < hi and nxt != p_idx:
                        total_intra_contracted += 1
                        cur_x = nxt
                    else:
                        w_end = nxt
                        break
                if len(chain_nodes) > max_chain_len:
                    max_chain_len = len(chain_nodes)
                chain_idx = len(chain_logs[k])
                chain_logs[k].append((u, chain_nodes))
                bu = K if u == p_idx else bkt_of(u)
                req_buckets[bu].append((u, k, chain_idx))

                # Emit shortcut edge (u -> w_end, acc_w)
                total_shortcuts_emitted += 1
                if u == p_idx and w_end == p_idx:
                    p_cycle_len = acc_w
                else:
                    eu = K if u == p_idx else bkt_of(u)
                    ew = K if w_end == p_idx else bkt_of(w_end)
                    target = eu if eu < ew else ew
                    fwd_buckets[target].append((u, w_end, acc_w))

        # 2. Any unvisited vertices in B_k form closed cycles entirely inside B_k (disjoint from p_idx!)
        # We simply discard them (they are not on p_idx's cycle).

    # Backward Sweep: compute exact pos[v] for all v on p_idx's cycle
    pos = [-1] * m
    pos[p_idx] = 0
    reply_buckets = [[] for _ in range(K)]

    # Answer requests for p_idx (bucket K)
    for u, src_k, chain_idx in req_buckets[K]:
        assert u == p_idx
        reply_buckets[src_k].append((chain_idx, 0))

    for k in range(K - 1, -1, -1):
        lo = k * S
        hi = min(m, (k + 1) * S)
        # Apply replies to chains in bucket k
        for chain_idx, parent_pos in reply_buckets[k]:
            if parent_pos == -1:
                continue
            _, chain_nodes = chain_logs[k][chain_idx]
            for x, rel_off in chain_nodes:
                pos[x] = parent_pos + rel_off
        reply_buckets[k] = None
        chain_logs[k] = None
        # Answer requests aimed at vertices in bucket k
        for u, src_k, chain_idx in req_buckets[k]:
            reply_buckets[src_k].append((chain_idx, pos[u]))
        req_buckets[k] = None

    # Verify exactness against direct cycle walk from p_idx
    ref_pos = [-1] * m
    cur = p_idx
    off = 0
    steps = 0
    while ref_pos[cur] == -1:
        ref_pos[cur] = off
        off += W[cur]
        cur = G[cur]
        steps += 1
    assert cur == p_idx
    assert off == p_cycle_len
    assert pos == ref_pos, "MF-BSE position mismatch!"

    return {
        "m": m,
        "K": K,
        "S": S,
        "cycle_nodes": steps,
        "cycle_len": p_cycle_len,
        "initial_intra": initial_intra,
        "total_bucket_edges_read": total_bucket_edges_read,
        "total_shortcuts_emitted": total_shortcuts_emitted,
        "total_intra_contracted": total_intra_contracted,
        "max_bucket_edges": max_bucket_edges,
        "max_chain_len": max_chain_len,
    }


def main():
    blocks = verify_and_load_blocks()
    print(f"Verified SHA-256 for all {len(blocks)} blocks.\n")
    print(f"{'Block':<12} {'n':>7} {'r':>7} {'w1/r':>6} {'m(L16)':>7} {'m/n':>6} {'H(>16)':>7}")
    print("-" * 62)

    open_data = []
    for entry, data in blocks:
        LF, C = compute_LF(data)
        m, p_idx, G, W, r, w1, thin_r, thick_r = build_open_retained_G(
            data, entry["primary_index"], LF, limit=16
        )
        assert r == entry["r"]
        print(
            f"{entry['file']:<12} {entry['n']:>7} {r:>7} {w1/r:>6.3f} {m:>7} {m/entry['n']:>6.3f} {thick_r:>7}"
        )
        open_data.append((entry, data, LF, m, p_idx, G, W))

    print("\n=== MF-BSE Exact Verification & Traffic Census on Open Retained G (K=64) ===")
    print(
        f"{'Block':<12} {'m':>7} {'S':>5} {'InitIntra':>9} {'TotIntra':>8} "
        f"{'Shortcuts':>9} {'Sc/m':>6} {'MaxBktE':>7} {'2S':>5} {'MaxCh':>5}"
    )
    print("-" * 86)
    for entry, data, LF, m, p_idx, G, W in open_data:
        st = simulate_mfbse(G, W, p_idx, K=64)
        print(
            f"{entry['file']:<12} {st['m']:>7} {st['S']:>5} {st['initial_intra']:>9} "
            f"{st['total_intra_contracted']:>8} {st['total_shortcuts_emitted']:>9} "
            f"{st['total_shortcuts_emitted']/st['m']:>6.4f} {st['max_bucket_edges']:>7} "
            f"{2*st['S']:>5} {st['max_chain_len']:>5}"
        )

    print("\n=== MF-BSE Scaling Across K on block-0.L (Open Retained G) ===")
    entry0, _, _, m0, p0, G0, W0 = open_data[0]
    for K in [16, 32, 64, 128, 256]:
        st = simulate_mfbse(G0, W0, p0, K=K)
        print(
            f"K={K:>3} S={st['S']:>5} L1_state~{st['S']*16//1024:>3}KiB "
            f"Sc/m={st['total_shortcuts_emitted']/st['m']:.4f} "
            f"TotRead/m={st['total_bucket_edges_read']/st['m']:.4f} "
            f"MaxBktE/2S={st['max_bucket_edges']/(2*st['S']):.3f} "
            f"MaxCh={st['max_chain_len']}"
        )


if __name__ == "__main__":
    main()
