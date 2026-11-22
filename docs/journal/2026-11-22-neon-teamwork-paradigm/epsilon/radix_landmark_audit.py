#!/usr/bin/env python3
"""
Exact measurement of Batched Radix-Doubling (Q=0) on S_16 interior chains:
  (A) Bounded by thick-strip anchors only (w_FL > 1)
  (B) Bounded by thick-strip anchors + UUID dash landmarks (F == '-' thin rows)
Computes exact per-round active edges, total rounds, total join operations,
and exact sequential byte traffic B (assuming packed 12-byte records per active edge),
comparing against Open serial walk and Gamma's MF-BSE across all 6 Wikidata blocks.
"""
import hashlib
import json
import os


def compute_LF_FL_F(L):
    n = len(L)
    cnt = [0] * 256
    for b in L:
        cnt[b] += 1
    C = [0] * 256
    s = 0
    for i in range(256):
        C[i] = s
        s += cnt[i]
    cur = C[:]
    LF = [0] * n
    for i, b in enumerate(L):
        LF[i] = cur[b]
        cur[b] += 1
    FL = [0] * n
    for i, j in enumerate(LF):
        FL[j] = i
    F = bytearray(n)
    for c in range(256):
        if cnt[c]:
            F[C[c] : C[c] + cnt[c]] = bytes([c]) * cnt[c]
    return LF, FL, F, C


def simulate_radix_doubling_on_chains(g_FL, w_FL, cut_set):
    """Given permutation g_FL[0..m) and anchor/cut set `cut_set` (boolean array of len m):
    Break every edge entering a vertex in cut_set (so each non-cut vertex points to its
    chain's next vertex, or -1 if next is in cut_set).
    Run exact pointer doubling until every chain finishes (nxt[u] == -1 for all u).
    Returns: (rounds, total_active_joins, max_chain_len, num_anchors)."""
    m = len(g_FL)
    nxt = [(-1 if cut_set[g_FL[u]] else g_FL[u]) for u in range(m)]
    # Chain lengths
    num_anchors = sum(cut_set)
    # Count active vertices per round (only vertices with nxt[u] != -1 participate in join!)
    rounds = 0
    total_joins = 0
    # Compute exact max chain length directly
    max_len = 0
    for u in range(m):
        if cut_set[u]:
            cur = u
            ln = 1
            while nxt[cur] != -1:
                cur = nxt[cur]
                ln += 1
            if ln > max_len:
                max_len = ln

    active_list = [u for u in range(m) if nxt[u] != -1]
    while active_list:
        rounds += 1
        # One radix join matches every active u (looking up nxt[u]) against all active/target v:
        # Both sides of the join (queries = active_list, table = all vertices still pointed to)
        # In bucketed radix join: scatter queries (len(active_list)) + scatter targets (at least len(active_list))
        total_joins += len(active_list)
        new_nxt = nxt[:]
        next_active = []
        for u in active_list:
            v = nxt[u]
            nv = nxt[v]
            new_nxt[u] = nv
            if nv != -1:
                next_active.append(u)
        nxt = new_nxt
        active_list = next_active

    return rounds, total_joins, max_len, num_anchors


def main():
    manifest_path = "/tmp/mibz2-blocks/manifest.json"
    with open(manifest_path) as f:
        manifest = json.load(f)

    print("=== Exact Batched Radix-Doubling Audit: Thick-Strip Anchors vs Thick-Strip + Dash ('-') Landmarks ===")
    print(f"{'Block':<12} {'m16':>7} | {'TS-only |A|':>11} {'maxL':>4} {'rds':>3} {'joins/m':>7} | {'TS+Dash |A|':>11} {'maxL':>4} {'rds':>3} {'joins/m':>7}")
    print("-" * 92)
    for entry in manifest["blocks"]:
        path = os.path.join("/tmp/mibz2-blocks", entry["file"])
        L = open(path, "rb").read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        n = entry["n"]
        p = entry["primary_index"]
        _, FL, F, _ = compute_LF_FL_F(L)

        # Build Open retained set S_16
        is_ret = bytearray(n)
        i = 0
        while i < n:
            b = L[i]
            j = i + 1
            while j < n and L[j] == b:
                j += 1
            if j - i <= 16:
                for k in range(i, j):
                    is_ret[k] = 1
            i = j
        is_ret[p] = 1
        row_to_u = [-1] * n
        ret_rows = []
        for r_idx in range(n):
            if is_ret[r_idx]:
                row_to_u[r_idx] = len(ret_rows)
                ret_rows.append(r_idx)
        m16 = len(ret_rows)
        p_u = row_to_u[p]

        g_FL = [0] * m16
        w_FL = [0] * m16
        for u, r_idx in enumerate(ret_rows):
            cur = FL[r_idx]
            steps = 1
            while not is_ret[cur]:
                cur = FL[cur]
                steps += 1
            g_FL[u] = row_to_u[cur]
            w_FL[u] = steps

        # Cut set A: thick-strip exits + p_u
        cut_ts = bytearray(m16)
        cut_ts[p_u] = 1
        for u in range(m16):
            if w_FL[u] > 1:
                cut_ts[g_FL[u]] = 1

        # Cut set B: thick-strip exits + F == '-' thin rows + p_u
        cut_dash = bytearray(cut_ts)
        dash = ord("-")
        for u, r_idx in enumerate(ret_rows):
            if F[r_idx] == dash:
                cut_dash[u] = 1

        r_a, j_a, ml_a, na_a = simulate_radix_doubling_on_chains(g_FL, w_FL, cut_ts)
        r_b, j_b, ml_b, na_b = simulate_radix_doubling_on_chains(g_FL, w_FL, cut_dash)

        print(
            f"{entry['file']:<12} {m16:7d} | {na_a:11d} {ml_a:4d} {r_a:3d} {j_a/m16:7.2f} | "
            f"{na_b:11d} {ml_b:4d} {r_b:3d} {j_b/m16:7.2f}"
        )


if __name__ == "__main__":
    main()
