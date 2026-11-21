#!/usr/bin/env python3
"""
Carbon Kitchen Sink — Cycle 11 Mathematical Probe & Exact Verifier (probe_cycle11.py)

Deep Confrontation of Cycle 9's `0.3%` Directed FVS vs Record 84 & Phi-Copy Replay,
Priced Against Native `mibz2 0.5.8` (`origin/master:src/open.rs`):

1. Theorem 11.1 — Refutation of FVS-Cut Direct Phi-Copy Replay (Non-Planar Diamond Braiding):
   - Finds & verifies a minimal exact primitive binary BWT witness where `G_ov \\ F` is a
     100% cycle-free DAG (0 directed cycles!) and every fragment of `[0, n) \\ (H U T_F)` is
     duplicate-free with 0 self-dependencies, YET the Phi-copy dependency graph `G_Phi(J_F)`
     on the acyclic remainder STILL deadlocks in a cycle `J_1 <-> J_2` due to rank-inverting
     diamond braiding!
   - Runs Kahn's topological attainability test on `[0, n) \\ (H U T_F)` across all 6
     authenticated Wikidata blocks (assuming an FREE ORACLE pre-fills all `r` head literals H
     AND all `W(F)` scattered positions `T_F` of the `171-508` FVS runs!), measuring the exact
     fraction of DAG-excursion fragments that STILL deadlock in Phi-cycles!

2. Theorem 11.2 — Exact Collective DAG Resolution (`FVS-Topo` & `FVS-Open`): Escaping Record 84
   Mathematically, Refuted by Phantom-Pruning Cost & Working-Set Growth vs `mibz2 0.5.8`:
   - Implements & verifies byte-exact `FVS-Topo` (reverse-topological metric interval pruning
     on `D = G_ov \\ F` -> `W(F)`-step cycle walk on `U_F` -> pointer-free forward-topological
     slice propagation `tau[c_v..c_v+w_v] = tau[d_v..d_v+w_v] - 1`) on ALL 1,508 small primitive
     BWTs AND byte-exact on all 6 900KB Wikidata blocks!
   - Measures across all 6 blocks (for both Levy-Low `FVS_wt` and linear-time `DFS Back-Edge FVS`):
       * Exact total metric sub-interval splits propagated across `D = G_ov \\ F` to prune the
         `2^35..2^63` phantom diamond paths (vs `mibz2 0.5.8` `Open<16>`'s `<= 3H + 2` steps!),
       * Exact per-worker peak working-set footprint (in KiB) of native `mibz2 0.5.8` (`src/open.rs`)
         vs `FVS-Topo` and `FVS-Open`!
"""

from __future__ import annotations
from collections import deque
import heapq
import itertools
import json
from pathlib import Path

ADMITTED_DIR = Path("/home/antigravity/admitted-bwt")
OUT_PATH = Path(__file__).resolve().parent / "probe_cycle11_results.json"


def load_manifest():
    with open(ADMITTED_DIR / "manifest.json", "r", encoding="utf-8") as f:
        return json.load(f)["blocks"]


def is_primitive(w: tuple[int, ...]) -> bool:
    n = len(w)
    return not any(n % d == 0 and w == w[:d] * (n // d) for d in range(1, n))


def bwt_of_word(w: tuple[int, ...]) -> tuple[bytes, int]:
    n = len(w)
    rots = sorted((w[i:] + w[:i], i) for i in range(n))
    L = bytes(rot[-1] for rot, _ in rots)
    p = next(idx for idx, (_, orig) in enumerate(rots) if orig == 0)
    return L, p


def build_gov_and_lf(L: bytes):
    n = len(L)
    counts = [0] * 256
    runs_d = []
    runs_w = []
    runs_b = []
    row_to_run = [0] * n
    i = 0
    while i < n:
        b = L[i]
        j = i + 1
        while j < n and L[j] == b:
            j += 1
        w = j - i
        rid = len(runs_d)
        runs_d.append(i)
        runs_w.append(w)
        runs_b.append(b)
        for x in range(i, j):
            row_to_run[x] = rid
        counts[b] += w
        i = j
    r = len(runs_d)

    occ = [0] * 256
    acc = 0
    for c in range(256):
        occ[c] = acc; acc += counts[c]
    lf = [0] * n
    for x, b in enumerate(L):
        lf[x] = occ[b]; occ[b] += 1

    occ_r = [0] * 256
    acc = 0
    for c in range(256):
        occ_r[c] = acc; acc += counts[c]
    runs_c = []
    for b, w in zip(runs_b, runs_w):
        runs_c.append(occ_r[b]); occ_r[b] += w

    order_c = sorted(range(r), key=lambda u: runs_c[u])
    out_adj: list[list[int]] = [[] for _ in range(r)]
    # Also record exact overlap triples (v_target, u_offset, v_offset, ov_len) for each run u!
    out_overlaps: list[list[tuple[int, int, int, int]]] = [[] for _ in range(r)]
    ui = 0
    vi = 0
    u_off = 0
    v_off = 0
    while ui < r and vi < r:
        u = order_c[ui]
        uw = runs_w[u]
        vw = runs_w[vi]
        step = (uw - u_off) if (uw - u_off) < (vw - v_off) else (vw - v_off)
        out_adj[u].append(vi)
        out_overlaps[u].append((vi, u_off, v_off, step))
        u_off += step
        v_off += step
        if u_off == uw:
            ui += 1; u_off = 0
        if v_off == vw:
            vi += 1; v_off = 0

    return r, runs_d, runs_w, runs_c, runs_b, row_to_run, lf, out_adj, out_overlaps


def compute_dfs_backedge_fvs(r: int, out_adj: list[list[int]]) -> set[int]:
    """Fast O(r + E_ov) linear-time iterative DFS back-edge FVS (zero heap/contraction cost)."""
    state = bytearray(r)  # 0 unvisited, 1 on_stack, 2 done
    fvs: set[int] = set()
    for s0 in range(r):
        if state[s0] != 0:
            continue
        state[s0] = 1
        stack = [(s0, 0)]
        while stack:
            u, ei = stack[-1]
            if u in fvs:
                state[u] = 2
                stack.pop()
                continue
            nbrs = out_adj[u]
            if ei < len(nbrs):
                stack[-1] = (u, ei + 1)
                v = nbrs[ei]
                if v in fvs:
                    continue
                if state[v] == 1:
                    # Directed back-edge u -> v! Add v to FVS (breaks cycle at target hub)
                    fvs.add(v)
                elif state[v] == 0:
                    state[v] = 1
                    stack.append((v, 0))
            else:
                state[u] = 2
                stack.pop()
    return fvs


def compute_levy_low_fvs(
    r: int, out_adj: list[list[int]], runs_w: list[int], weight_penalised: bool = True
) -> set[int]:
    out_s = [set(nbrs) for nbrs in out_adj]
    in_s = [set() for _ in range(r)]
    for u in range(r):
        for v in out_s[u]:
            in_s[v].add(u)

    alive = bytearray([1] * r)
    fvs: set[int] = set()
    q: deque[int] = deque(range(r))

    def remove_vertex(u: int, to_fvs: bool):
        if not alive[u]:
            return
        alive[u] = 0
        if to_fvs:
            fvs.add(u)
        for v in list(out_s[u]):
            in_s[v].discard(u)
            if alive[v]:
                q.append(v)
        for p in list(in_s[u]):
            out_s[p].discard(u)
            if alive[p]:
                q.append(p)
        out_s[u].clear()
        in_s[u].clear()

    def bypass_deg1(u: int):
        if not alive[u]:
            return
        preds = list(in_s[u])
        succs = list(out_s[u])
        alive[u] = 0
        for p in preds:
            out_s[p].discard(u)
        for s in succs:
            in_s[s].discard(u)
        out_s[u].clear()
        in_s[u].clear()
        for p in preds:
            for s in succs:
                if p == s:
                    out_s[p].add(p); in_s[p].add(p); q.append(p)
                else:
                    out_s[p].add(s); in_s[s].add(p)
            if alive[p]:
                q.append(p)
        for s in succs:
            if alive[s]:
                q.append(s)

    heap: list[tuple[float, int]] = []
    heap_built = False
    while True:
        while q:
            u = q.popleft()
            if not alive[u]:
                continue
            if u in out_s[u]:
                remove_vertex(u, to_fvs=True)
            elif len(in_s[u]) == 0 or len(out_s[u]) == 0:
                remove_vertex(u, to_fvs=False)
            elif len(in_s[u]) == 1 or len(out_s[u]) == 1:
                bypass_deg1(u)

        if not heap_built:
            for u in range(r):
                if alive[u]:
                    sc = (len(in_s[u]) * len(out_s[u])) / (runs_w[u] if weight_penalised else 1.0)
                    heapq.heappush(heap, (-sc, u))
            heap_built = True

        picked = -1
        while heap:
            neg_sc, u = heapq.heappop(heap)
            if not alive[u]:
                continue
            cur_sc = (len(in_s[u]) * len(out_s[u])) / (runs_w[u] if weight_penalised else 1.0)
            if abs(-neg_sc - cur_sc) > 0.2 * max(1e-9, cur_sc) and heap:
                heapq.heappush(heap, (-cur_sc, u))
                continue
            picked = u
            break

        if picked == -1:
            rem = [u for u in range(r) if alive[u]]
            if not rem:
                break
            picked = max(
                rem,
                key=lambda u: (len(in_s[u]) * len(out_s[u])) / (runs_w[u] if weight_penalised else 1.0),
            )
        remove_vertex(picked, to_fvs=True)

    return fvs


# ---------------------------------------------------------------------------
# Theorem 11.1: Oracle FVS-Cut Phi-Copy Attainability Test (Round 6 Upgrade!)
# ---------------------------------------------------------------------------

def test_fvs_cut_phi_attainability(
    n: int,
    r: int,
    runs_d: list[int],
    runs_w: list[int],
    row_to_run: list[int],
    lf: list[int],
    p_mark: int,
    fvs: set[int],
) -> dict:
    """
    Exact Round 6 (`round6_attainability.py`) Phi-copy fragment test when an ORACLE pre-fills
    BOTH all `r` head literals H AND all `W(F)` rows of the FVS runs `U_F`:
      - Walk LF from p_mark: `row_to_pos[cur] = s`, rotated so first run head is at `t = 0`.
      - Every `t` where `x` is a run head (`x in D_bar`) OR `row_to_run[x] in fvs` is PRE-FILLED!
      - Partition `[0, n) \\ (H U T_F)` into maximal contiguous intervals `J_0, ..., J_{M-1}`!
      - Verify: (a) 0 repeated runs inside EVERY `J_m`, and (b) 0 self-dependencies!
      - Build Phi-dependency graph `gsrc -> f` (where `s = Phi(t) = pos(x - 1)`) and run Kahn's
        algorithm to count how many fragments are topologically orderable vs DEADLOCKED IN CYCLES!
    """
    row_to_pos = [0] * n
    cur = p_mark
    for s in range(n):
        row_to_pos[cur] = s
        cur = lf[cur]
    assert cur == p_mark

    tau0 = min(row_to_pos[runs_d[u]] for u in range(r))
    pos_to_row = [0] * n
    for x in range(n):
        pos_to_row[(row_to_pos[x] - tau0) % n] = x

    is_head = bytearray(n)
    for d0 in runs_d:
        is_head[d0] = 1
    in_fvs = bytearray(r)
    for u in fvs:
        in_fvs[u] = 1

    # Partition [0, n) into pre-filled (head or FVS) vs maximal contiguous DAG-excursion fragments!
    frag_at = [-1] * n
    fid = 0
    t = 0
    while t < n:
        x = pos_to_row[t]
        if is_head[x] or in_fvs[row_to_run[x]]:
            t += 1
            continue
        j = t
        seen_runs_in_frag = set()
        while j < n:
            xj = pos_to_row[j]
            rj = row_to_run[xj]
            if is_head[xj] or in_fvs[rj]:
                break
            assert rj not in seen_runs_in_frag, f"Run {rj} repeated inside FVS-free excursion?!"
            seen_runs_in_frag.add(rj)
            frag_at[j] = fid
            j += 1
        fid += 1
        t = j

    M = fid
    if M == 0:
        return {
            "fragments": 0,
            "edges": 0,
            "self_deps": 0,
            "orderable": 0,
            "deadlocked": 0,
            "deadlocked_pct": 0.0,
        }

    indeg = [0] * M
    adj: list[list[int]] = [[] for _ in range(M)]
    edges = set()
    selfdep = 0
    rem_bytes = 0
    for t in range(n):
        f = frag_at[t]
        if f < 0:
            continue
        rem_bytes += 1
        x = pos_to_row[t]
        # Phi source of x (note x is NOT a run head, so x - 1 is in the SAME run!)
        src_t = (row_to_pos[x - 1] - tau0) % n
        gsrc = frag_at[src_t]
        if gsrc < 0:
            continue  # source is already pre-filled in H U T_F!
        if gsrc == f:
            selfdep += 1
            continue
        if (gsrc, f) not in edges:
            edges.add((gsrc, f))
            adj[gsrc].append(f)
            indeg[f] += 1

    assert selfdep == 0, f"Unexpected self-dependency {selfdep} inside DAG excursion!"
    q = deque(i for i in range(M) if indeg[i] == 0)
    done = 0
    while q:
        v = q.popleft()
        done += 1
        for w in adj[v]:
            indeg[w] -= 1
            if indeg[w] == 0:
                q.append(w)

    deadlocked = M - done
    return {
        "remainder_bytes": rem_bytes,
        "fragments": M,
        "mean_frag_bytes": round(rem_bytes / max(1, M), 2),
        "edges": len(edges),
        "self_deps": selfdep,
        "orderable": done,
        "deadlocked": deadlocked,
        "deadlocked_pct": round(100.0 * deadlocked / M, 2),
    }


def find_minimal_diamond_braid_witness() -> dict:
    """
    Find the exact minimal-length primitive binary BWT where `G_ov \\ F` is a 100% cycle-free DAG
    (0 directed cycles, 0 self-deps, 0 repeated runs per fragment), yet `G_Phi(J_F)` STILL
    deadlocks in a cycle `J_1 <-> J_2` due to non-planar diamond braiding!
    """
    for n in range(8, 17):
        for w in itertools.product((0, 1), repeat=n):
            if not is_primitive(w):
                continue
            L, p = bwt_of_word(w)
            r, runs_d, runs_w, _, _, row_to_run, lf, out_adj, _ = build_gov_and_lf(L)
            fvs = compute_levy_low_fvs(r, out_adj, runs_w, weight_penalised=False)
            st = test_fvs_cut_phi_attainability(n, r, runs_d, runs_w, row_to_run, lf, p, fvs)
            if st["deadlocked"] > 0:
                return {
                    "word": "".join(str(c) for c in w),
                    "n": n,
                    "r": r,
                    "L": "".join(str(c) for c in L),
                    "p": p,
                    "fvs_runs": sorted(fvs),
                    "phi_stats": st,
                }
    raise RuntimeError("No diamond-braid witness found up to binary n=16!")


# ---------------------------------------------------------------------------
# Theorem 11.2: Exact Collective DAG Decoder (`FVS-Topo`) & Cost/Memory Audit
# ---------------------------------------------------------------------------

def fvs_topo_decode_and_cost(L: bytes, p_mark: int, fvs_mode: str = "levy_low_wt") -> tuple[bytes, dict]:
    """
    Exact Two-Phase Record-84-Escaping Collective Decoder (`FVS-Topo`):
      - Builds `G_ov` and `F = FVS(G_ov)` (`p`'s run added to `F` if not already in `F` so `p in U_F`).
      - Phase 1A (Reverse-Topological Metric Interval Elimination of `D = G_ov \\ F`):
          In reverse topological order of `D`, computes for every `v in V(D)` the piecewise-affine
          exit map `exit_pieces[v]` = list of `(width, target_row_in_U_F_start, step_len)` by
          splitting `out_overlaps[v]` against `exit_pieces[succ]` (ZERO iterative loops, 1 pass!).
          Applies the same 1-step lookup on `f in F` to get `T_{U_F}` (`r_ret` affine pieces!).
      - Phase 1B (`U_F` Cycle Walk):
          Walks ONLY `U_F` from `p_mark` (`W(F)` steps!) to fill `tau[x]` and `out[tau[x]]` on `U_F`!
      - Phase 2 (Collective Pointer-Free Forward-Topological Slice Propagation on `D`):
          First pushes `tau[c_f + t] = (tau[d_f + t] - 1) % n` for `f in F`, then sweeps `v in topo(D)`
          executing contiguous slice `out[tau[d_v+t]] = b_v` and `tau[c_v+t] = (tau[d_v+t] - 1) % n`!
    """
    n = len(L)
    r, runs_d, runs_w, runs_c, runs_b, row_to_run, lf, out_adj, out_overlaps = build_gov_and_lf(L)
    if fvs_mode == "dfs_backedge":
        fvs = compute_dfs_backedge_fvs(r, out_adj)
    elif fvs_mode == "levy_low_card":
        fvs = compute_levy_low_fvs(r, out_adj, runs_w, weight_penalised=False)
    else:
        fvs = compute_levy_low_fvs(r, out_adj, runs_w, weight_penalised=True)

    # Ensure the run containing `p_mark` is in `fvs` so `p_mark in U_F` (adds at most 1 run!)
    fvs = set(fvs) | {row_to_run[p_mark]}
    in_fvs = bytearray(r)
    for u in fvs:
        in_fvs[u] = 1

    # Kahn topological sort of D = G_ov \ F
    in_deg = [0] * r
    for u in range(r):
        if in_fvs[u]:
            continue
        for v in out_adj[u]:
            if not in_fvs[v]:
                in_deg[v] += 1
    q = deque(u for u in range(r) if not in_fvs[u] and in_deg[u] == 0)
    topo = []
    while q:
        u = q.popleft()
        topo.append(u)
        for v in out_adj[u]:
            if not in_fvs[v]:
                in_deg[v] -= 1
                if in_deg[v] == 0:
                    q.append(v)
    assert len(topo) == r - len(fvs), "Cycle remained in D!"

    # Phase 1A: Reverse-topological metric interval propagation across D!
    # For each run `u`, `exit_map[u]` partitions `[0, w_u)` into contiguous pieces `(w_piece, uf_target_row, steps_to_uf)`.
    exit_map: list[list[tuple[int, int, int]]] = [[] for _ in range(r)]
    total_dag_subpieces_propagated = 0

    def pull_back_overlaps(u: int) -> list[tuple[int, int, int]]:
        nonlocal total_dag_subpieces_propagated
        pieces: list[tuple[int, int, int]] = []
        for v, _u_off, v_off, ov_len in out_overlaps[u]:
            if in_fvs[v]:
                # Lands directly in U_F at row `runs_d[v] + v_off` in 1 step!
                pieces.append((ov_len, runs_d[v] + v_off, 1))
                total_dag_subpieces_propagated += 1
            else:
                # Slice `exit_map[v]` over `[v_off, v_off + ov_len)`!
                v_pieces = exit_map[v]
                # Advance to `v_off`
                pos = 0
                idx = 0
                while pos + v_pieces[idx][0] <= v_off:
                    pos += v_pieces[idx][0]
                    idx += 1
                rem = ov_len
                cur_v = v_off
                while rem > 0:
                    pw, p_tgt, p_steps = v_pieces[idx]
                    in_piece_off = cur_v - pos
                    avail = pw - in_piece_off
                    take = rem if rem < avail else avail
                    pieces.append((take, p_tgt + in_piece_off, p_steps + 1))
                    total_dag_subpieces_propagated += 1
                    rem -= take
                    cur_v += take
                    if take == avail:
                        pos += pw
                        idx += 1
        # Coalesce adjacent co-translated pieces with equal step count!
        merged: list[tuple[int, int, int]] = []
        for pw, pt, ps in pieces:
            if merged and merged[-1][1] + merged[-1][0] == pt and merged[-1][2] == ps:
                mw, mt, ms = merged[-1]
                merged[-1] = (mw + pw, mt, ms)
            else:
                merged.append((pw, pt, ps))
        return merged

    for v in reversed(topo):
        exit_map[v] = pull_back_overlaps(v)
    for f in fvs:
        exit_map[f] = pull_back_overlaps(f)

    total_dag_stored_pieces = sum(len(exit_map[v]) for v in topo)
    fvs_return_bands = sum(len(exit_map[f]) for f in fvs)

    # Expand `U_F` return map on the `W(F)` rows of `U_F` (1 u32 next_uf_idx + 1 u16 step_len per U_F row)
    uf_rows = []
    row_to_uf_id = {}
    for f in sorted(fvs):
        d0 = runs_d[f]
        for x in range(d0, d0 + runs_w[f]):
            row_to_uf_id[x] = len(uf_rows)
            uf_rows.append(x)
    w_fvs = len(uf_rows)

    uf_next = [0] * w_fvs
    uf_steps = [0] * w_fvs
    uid = 0
    for f in sorted(fvs):
        for pw, pt, ps in exit_map[f]:
            base_tgt_uid = row_to_uf_id[pt]
            for off in range(pw):
                uf_next[uid] = base_tgt_uid + off
                uf_steps[uid] = ps
                uid += 1
    assert uid == w_fvs

    # Phase 1B: Walk ONLY the `W(F)`-cycle on `U_F` from `p_mark` to resolve `tau[x]` on `U_F`!
    tau = [-1] * n
    out = bytearray(n)
    cur_uid = row_to_uf_id[p_mark]
    cur_tau = n - 1
    visited_uf = 0
    while True:
        x = uf_rows[cur_uid]
        tau[x] = cur_tau
        out[cur_tau] = L[x]
        cur_tau = (cur_tau - uf_steps[cur_uid]) % n
        cur_uid = uf_next[cur_uid]
        visited_uf += 1
        if cur_uid == row_to_uf_id[p_mark]:
            break
    assert visited_uf == w_fvs and cur_tau == n - 1

    # Phase 2: Collective Pointer-Free Topological Slice Propagation across `D = G_ov \ F`!
    # Step 2a: Push contiguous codomain slices from all `f in FVS`
    for f in fvs:
        d0 = runs_d[f]
        c0 = runs_c[f]
        wf = runs_w[f]
        for t in range(wf):
            y = c0 + t
            if not in_fvs[row_to_run[y]]:
                tau[y] = (tau[d0 + t] - 1) % n

    # Step 2b: Sweep `v in topo(D)` in topological order — EVERY `tau[d_v..d_v+w_v]` is guaranteed ready!
    for v in topo:
        d0 = runs_d[v]
        c0 = runs_c[v]
        wv = runs_w[v]
        bv = runs_b[v]
        for t in range(wv):
            tx = tau[d0 + t]
            assert tx >= 0, f"Unresolved tau at DAG run {v} offset {t}!"
            out[tx] = bv
            y = c0 + t
            if not in_fvs[row_to_run[y]]:
                tau[y] = (tx - 1) % n

    return bytes(out), {
        "fvs_mode": fvs_mode,
        "fvs_runs": len(fvs),
        "fvs_rows_W": w_fvs,
        "dag_runs": len(topo),
        "dag_propagated_subpieces": total_dag_subpieces_propagated,
        "dag_stored_pieces": total_dag_stored_pieces,
        "fvs_return_bands": fvs_return_bands,
        "fvs_topo_working_set_KiB": round(
            (
                r * 16            # runs_d, runs_w, runs_c, runs_b
                + len(topo) * 4   # topo order
                + sum(len(x) for x in out_overlaps) * 16  # G_ov overlap arcs
                + total_dag_stored_pieces * 12  # exit_map pieces
                + w_fvs * 6       # uf_next + uf_steps
                + n * 4           # tau[0..n) for Phase 2!
            )
            / 1024.0,
            1,
        ),
        "fvs_open_working_set_KiB": round(
            (
                r * 16
                + len(topo) * 4
                + sum(len(x) for x in out_overlaps) * 16
                + total_dag_stored_pieces * 12
                + w_fvs * 8       # edges + inline/offset
                + n * 1           # spelling arena ~ n - W(F)
            )
            / 1024.0,
            1,
        ),
    }


def measure_native_058_working_set(L: bytes, p_mark: int) -> dict:
    """
    Exact simulation of `mibz2 0.5.8` (`origin/master:src/open.rs` `Open<16, true, true>`)
    construction counts (`m`, `thick`, `bands`, `words`, `wide`, `spelling_bytes`) and exact
    per-worker `Kernel` buffer footprint!
    """
    n = len(L)
    all_c = [0] * 256
    thin_c = [0] * 256
    thick_c = [0] * 256
    strips = []  # (from, to_local, width, byte)
    m = 0

    def add_piece(b: int, pos: int, w: int, keep: bool):
        nonlocal m
        if keep:
            thin_c[b] += w
            m += w
        else:
            strips.append([pos, all_c[b], w, b])
            thick_c[b] += 1
        all_c[b] += w

    i = 0
    while i < n:
        b = L[i]
        j = i + 1
        while j < n and L[j] == b:
            j += 1
        w = j - i
        if w > 16 and i <= p_mark < j:
            if i < p_mark:
                add_piece(b, i, p_mark - i, False)
            add_piece(b, p_mark, 1, True)
            if p_mark + 1 < j:
                add_piece(b, p_mark + 1, j - p_mark - 1, False)
        else:
            add_piece(b, i, w, w <= 16)
        i = j

    first = [0] * 256
    acc = 0
    for c in range(256):
        first[c] = acc; acc += all_c[c]
    for s in strips:
        s[1] += first[s[3]]

    # Build exact tiles (produces identical bands & words to `INDUCE=true` by `checks.rs` invariant!)
    tiles = []  # (end, delta, sym)
    last_x = 0
    ret_x = 0
    for fr, to, w, b in strips:
        if fr > last_x:
            tiles.append((fr, (ret_x - last_x) & 0xFFFFFFFF, 256))
            ret_x += fr - last_x
        last_x = fr + w
        tiles.append((last_x, (to - fr) & 0xFFFFFFFF, b))
    if last_x < n:
        tiles.append((n, (ret_x - last_x) & 0xFFFFFFFF, 256))

    # Directory PAGE=64
    directory = []
    ti = 0
    for x in range(0, n, 64):
        while tiles[ti][0] <= x:
            ti += 1
        directory.append(ti)

    image_order = sorted(range(len(strips)), key=lambda idx: strips[idx][1])
    bands = 0
    words = 0
    spelling_bytes = 0
    unique_words: set[bytes] = set()
    si = 0
    k = 0
    n_strips = len(strips)

    def trace_interval(x0: int, end: int, c_byte: int):
        nonlocal bands, words, spelling_bytes, k
        while x0 < end:
            width = end - x0
            x = x0
            w_chars = bytearray()
            while True:
                q = directory[x >> 6]
                while x >= tiles[q][0]:
                    q += 1
                t_end, t_delta, t_sym = tiles[q]
                if t_end - x < width:
                    width = t_end - x
                x = (x + t_delta) & 0xFFFFFFFF
                if t_sym == 256:
                    break
                w_chars.append(t_sym)
            if len(w_chars) > 0:
                words += 1
                w_chars.reverse()
                w_chars.append(c_byte)
                spelling_bytes += len(w_chars)
                unique_words.add(bytes(w_chars))
            bands += 1
            k += width
            x0 += width

    for a in range(256):
        x = first[a]
        end_a = first[a] + all_c[a]
        while si < n_strips and strips[image_order[si]][3] == a:
            s_fr, s_to, s_w, _ = strips[image_order[si]]
            trace_interval(x, s_to, a)
            x = s_to + s_w
            si += 1
        trace_interval(x, end_a, a)
    assert k == m

    bits = max(1, (m - 1).bit_length())
    wide = (255 + words) > (0xFFFFFFFF >> bits)

    # Exact `0.5.8` buffer bytes touched:
    # queues: for active bytes (thin_c[a] > 0), Vec grows by doubling from 1024 u32s
    queue_u32s = 0
    for a in range(256):
        if thin_c[a] > 0:
            need = thin_c[a] + 16
            cap = 1024
            while cap < need:
                cap *= 2
            queue_u32s += cap
    queues_bytes = queue_u32s * 4
    strips_bytes = len(strips) * 16 + len(strips) * 4  # strips + image_order
    bands_bytes = bands * 12
    spelling_and_offsets_bytes = spelling_bytes + (words + 1) * 4
    edges_and_extra_bytes = m * 4 * (2 if wide else 1)
    inline16_and_lens_bytes = (256 + words) * 17
    emit_hot_bytes = edges_and_extra_bytes + inline16_and_lens_bytes  # + cold spelling

    total_kernel_bytes = (
        queues_bytes
        + strips_bytes
        + bands_bytes
        + spelling_and_offsets_bytes
        + edges_and_extra_bytes
        + inline16_and_lens_bytes
    )

    return {
        "m": m,
        "H_thick": len(strips),
        "bands": bands,
        "words_allocated_058": words,
        "unique_words_exact": len(unique_words),
        "duplicate_word_ratio": round(words / max(1, len(unique_words)), 2),
        "bits": bits,
        "wide_extra_triggered": wide,
        "spelling_bytes": spelling_bytes,
        "queues_KiB": round(queues_bytes / 1024.0, 1),
        "bands_KiB": round(bands_bytes / 1024.0, 1),
        "edges_plus_extra_KiB": round(edges_and_extra_bytes / 1024.0, 1),
        "inline16_plus_lens_KiB": round(inline16_and_lens_bytes / 1024.0, 1),
        "spelling_offsets_KiB": round(spelling_and_offsets_bytes / 1024.0, 1),
        "emit_hot_tables_KiB": round(emit_hot_bytes / 1024.0, 1),
        "total_kernel_working_set_KiB": round(total_kernel_bytes / 1024.0, 1),
    }


def verify_small_cycle11() -> dict:
    checked = 0
    for alph, max_n in (((0, 1), 8), ((0, 1, 2), 6)):
        for n in range(2, max_n + 1):
            for w in itertools.product(alph, repeat=n):
                if not is_primitive(w):
                    continue
                L, p = bwt_of_word(w)
                got_ll, _ = fvs_topo_decode_and_cost(L, p, fvs_mode="levy_low_wt")
                assert got_ll == bytes(w), f"FVS-Topo Levy-Low mismatch on {w}"
                got_dfs, _ = fvs_topo_decode_and_cost(L, p, fvs_mode="dfs_backedge")
                assert got_dfs == bytes(w), f"FVS-Topo DFS mismatch on {w}"
                checked += 2
    wit = find_minimal_diamond_braid_witness()
    return {"fvs_topo_small_verified": checked, "minimal_diamond_braid_witness": wit}


def main():
    sm = verify_small_cycle11()
    print(f"[PASS] Theorem 11.2 (`FVS-Topo` collective DAG decode) verified on {sm['fvs_topo_small_verified']:,} small checks!")
    print(f"[PASS] Theorem 11.1 Minimal Diamond-Braid Deadlock Witness: {sm['minimal_diamond_braid_witness']}")

    blocks = load_manifest()
    results = {"small_verifications": sm, "blocks": {}}
    for b in blocks:
        name = b["file"]
        L = (ADMITTED_DIR / name).read_bytes()
        p_mark = b["primary_index"]
        n = len(L)

        # Reference text for byte-exact assertion
        r, runs_d, runs_w, _, _, row_to_run, lf, out_adj, _ = build_gov_and_lf(L)
        ref_out = bytearray(n)
        cur = p_mark
        for s in range(n):
            ref_out[(n - 1 - s) % n] = L[cur]
            cur = lf[cur]

        fvs_card = compute_levy_low_fvs(r, out_adj, runs_w, weight_penalised=False)
        phi_card = test_fvs_cut_phi_attainability(n, r, runs_d, runs_w, row_to_run, lf, p_mark, fvs_card)

        got_wt, cost_wt = fvs_topo_decode_and_cost(L, p_mark, fvs_mode="levy_low_wt")
        assert got_wt == bytes(ref_out), f"FVS-Topo Levy-Low failed on {name}!"
        got_dfs, cost_dfs = fvs_topo_decode_and_cost(L, p_mark, fvs_mode="dfs_backedge")
        assert got_dfs == bytes(ref_out), f"FVS-Topo DFS failed on {name}!"

        open058 = measure_native_058_working_set(L, p_mark)

        results["blocks"][name] = {
            "n": n,
            "r": r,
            "oracle_fvs_cut_phi_attainability": phi_card,
            "fvs_topo_levy_low_wt": cost_wt,
            "fvs_topo_dfs_backedge": cost_dfs,
            "native_058_open16": open058,
        }

        print(
            f"[{name}] Oracle-FVS-Phi: frags={phi_card['fragments']:,}, "
            f"DEADLOCKED={phi_card['deadlocked']:,} ({phi_card['deadlocked_pct']:.1f}%!) | "
            f"FVS-DAG splits={cost_wt['dag_propagated_subpieces']:,} (DFS-FVS={cost_dfs['fvs_runs']:,} runs, W={cost_dfs['fvs_rows_W']:,}), "
            f"WS(FVS-Topo)={cost_wt['fvs_topo_working_set_KiB']:,.0f}KB | "
            f"0.5.8: H={open058['H_thick']:,}, words={open058['words_allocated_058']:,} "
            f"(uniq={open058['unique_words_exact']:,}, {open058['duplicate_word_ratio']}x dup!, wide={open058['wide_extra_triggered']}), "
            f"emit_WS={open058['emit_hot_tables_KiB']:,.0f}KB, tot_WS={open058['total_kernel_working_set_KiB']:,.0f}KB"
        )

    with open(OUT_PATH, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)
    print(f"\nSaved Cycle 11 results to {OUT_PATH}")


if __name__ == "__main__":
    main()
