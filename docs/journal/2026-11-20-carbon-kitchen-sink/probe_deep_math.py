#!/usr/bin/env python3
"""
Carbon Kitchen Sink — Cycle 1 Deep Mathematical Probe (Part 2):
  - W1b: Thin-row vs Thick-row Koopman TV & Text inspection of the 1049-period spike on block-0
  - W3:  Quantum Torus Bivariate Few-Nomial (zw - 1)G_f(z, w) exact support & Berlekamp-Massey over F_257
  - W5:  RSK / Schensted-Greene Young diagram shape (LIS lambda_1, LDS height l(lambda)) of LF and Open's g
  - W8/9: Series-Reduced Measured Train Track Core:
          * Exact sequential pred[k] verification (Q=0 build)
          * Exact thin-branching-core row count m_TTO (vs Open's m)
          * Exact Rauzy-Veech induction step count on the 0.33r-piece Core Exchange (TTSR-RV)
  - W10: Factorisation Monodromy: cycle decomposition of LF_v alone and P alone in g = P o LF_v
"""

from __future__ import annotations
import bisect
import json
import math
import pathlib
import time
import numpy as np

BLOCKS_DIR = pathlib.Path("/home/antigravity/admitted-bwt")


def berlekamp_massey_mod(seq: list[int], mod: int = 257) -> int:
    """Exact linear complexity (minimal LFSR degree) of seq over F_mod (mod prime)."""
    C = [1]
    B = [1]
    L_deg = 0
    m = 1
    b = 1
    for n_idx, x in enumerate(seq):
        d = x % mod
        for i in range(1, L_deg + 1):
            d = (d + C[i] * seq[n_idx - i]) % mod
        if d == 0:
            m += 1
        elif 2 * L_deg <= n_idx:
            T = list(C)
            coef = (d * pow(b, mod - 2, mod)) % mod
            if len(C) < len(B) + m:
                C.extend([0] * (len(B) + m - len(C)))
            for i in range(len(B)):
                C[i + m] = (C[i + m] - coef * B[i]) % mod
            L_deg = n_idx + 1 - L_deg
            B = T
            b = d
            m = 1
        else:
            coef = (d * pow(b, mod - 2, mod)) % mod
            if len(C) < len(B) + m:
                C.extend([0] * (len(B) + m - len(C)))
            for i in range(len(B)):
                C[i + m] = (C[i + m] - coef * B[i]) % mod
            m += 1
    return L_deg


def rsk_lis_and_lds(perm: np.ndarray) -> tuple[int, int]:
    """
    Compute lambda_1 (Longest Increasing Subsequence length) and l(lambda) = lambda'_1
    (Longest Decreasing Subsequence length = min # of increasing subsequences by Dilworth/Greene)
    in O(n log n) via patience sorting tails.
    """
    # LIS (strictly increasing since perm is a permutation)
    tails_inc: list[int] = []
    bsearch = bisect.bisect_left
    for val in perm.tolist():
        idx = bsearch(tails_inc, val)
        if idx == len(tails_inc):
            tails_inc.append(val)
        else:
            tails_inc[idx] = val
    # LDS = LIS of (-perm)
    tails_dec: list[int] = []
    for val in perm.tolist():
        nval = -val
        idx = bsearch(tails_dec, nval)
        if idx == len(tails_dec):
            tails_dec.append(nval)
        else:
            tails_dec[idx] = nval
    return len(tails_inc), len(tails_dec)


def count_cycles(perm: np.ndarray) -> tuple[int, int, int]:
    """Return (num_cycles, num_fixed_points, max_cycle_len)."""
    n = len(perm)
    vis = np.zeros(n, dtype=np.uint8)
    n_cyc = 0
    n_fix = 0
    max_len = 0
    for i in range(n):
        if not vis[i]:
            n_cyc += 1
            c = 0
            cur = i
            while not vis[cur]:
                vis[cur] = 1
                cur = int(perm[cur])
                c += 1
            if c == 1:
                n_fix += 1
            if c > max_len:
                max_len = c
    return n_cyc, n_fix, max_len


def simulate_rv_steps_on_iet(widths: list[int], top_order: list[int], bot_order: list[int]) -> tuple[int, int]:
    """
    Exact simulation of right-end Rauzy-Veech induction on a general discrete IET
    with k pieces (widths[0..k-1], top_order = domain order of piece IDs, bot_order = codomain order).
    Returns (unaccelerated_steps, accelerated_steps).
    Uses doubly-linked lists in flat Python lists for O(steps) exact execution.
    """
    k = len(widths)
    # Node 0 is sentinel; piece IDs are 1..k
    w = [0] + list(widths)
    prev0 = [0] * (k + 1)
    next0 = [0] * (k + 1)
    prev1 = [0] * (k + 1)
    next1 = [0] * (k + 1)

    p = 0
    for u in top_order:
        uid = u + 1
        next0[p] = uid
        prev0[uid] = p
        p = uid
    next0[p] = 0
    prev0[0] = p

    p = 0
    for u in bot_order:
        uid = u + 1
        next1[p] = uid
        prev1[uid] = p
        p = uid
    next1[p] = 0
    prev1[0] = p

    rem = sum(widths)
    steps_unacc = 0
    steps_acc = 0

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

    # First run unaccelerated exact RV (same loop as src/open/endpoint.rs!)
    # Simultaneously count winner-runs (consecutive steps where the same larger piece wins)!
    last_winner = -1
    last_side = -1
    while rem > 0:
        x = prev0[0]
        y = prev1[0]
        wx = w[x]
        wy = w[y]
        steps_unacc += 1
        if x == y:
            steps_acc += 1
            last_winner = -1
            rem -= wx
            cut0(x)
            cut1(x)
            continue
        if wx < wy:
            if not (last_winner == y and last_side == 1):
                steps_acc += 1
                last_winner = y
                last_side = 1
            cut0(x)
            ins0_after(y, x)
            w[y] = wy - wx
            rem -= wx
        elif wy < wx:
            if not (last_winner == x and last_side == 0):
                steps_acc += 1
                last_winner = x
                last_side = 0
            cut1(y)
            ins1_after(x, y)
            w[x] = wx - wy
            rem -= wy
        else:  # wx == wy, x != y
            steps_acc += 1
            last_winner = -1
            cut0(x)
            ins0_after(y, x)
            cut0(y)
            cut1(y)
            rem -= wx

    return steps_unacc, steps_acc


def analyze_block_deep(fname: str, L: np.ndarray, p: int) -> dict:
    n = len(L)
    order = np.argsort(L, kind="mergesort")
    lf = np.empty(n, dtype=np.int32)
    lf[order] = np.arange(n, dtype=np.int32)

    # Orbit tau (tau[p] = 0, tau[lf[x]] = tau[x] + 1) and text
    tau = np.empty(n, dtype=np.int32)
    text_rev = np.empty(n, dtype=np.uint8)
    cur = p
    for t in range(n):
        tau[cur] = t
        text_rev[t] = L[cur]
        cur = int(lf[cur])
    text = text_rev[::-1]

    # Runs
    change = np.flatnonzero(L[1:] != L[:-1]) + 1
    starts = np.concatenate([[0], change])
    ends = np.concatenate([change, [n]])
    widths = ends - starts
    syms = L[starts]
    r = len(widths)
    img_starts = lf[starts]
    img_ends = img_starts + widths

    # --- W1b: Koopman TV restricted to width-1 rows vs thin rows vs thick rows ---
    dtau = (np.roll(tau, -1) - tau) % n
    row_width = np.repeat(widths, widths)
    w1_mask = row_width == 1
    thin_mask = row_width <= 16
    thick_mask = row_width > 16

    def tv_ratio_on(mask: np.ndarray, m_mult: int = 1) -> float:
        sub = dtau[mask]
        if len(sub) == 0:
            return 0.0
        return float(np.sum(2.0 * np.abs(np.sin(np.pi * ((m_mult * sub.astype(np.int64)) % n) / n))) / ((4.0 / math.pi) * len(sub)))

    # Also test best m in 1..32 on w1_mask
    min_tv_w1 = min(tv_ratio_on(w1_mask, m_test) for m_test in range(1, 33))
    min_tv_thin = min(tv_ratio_on(thin_mask, m_test) for m_test in range(1, 33))
    tv_thick_m1 = tv_ratio_on(thick_mask, 1)

    sample_1049 = ""
    if fname == "block-0.L":
        # Inspect why dtau == 1049 or -1049 occurs 26,062 times!
        # Note tau[x] = n - 1 - SA_pos, so dtau = 1049 means SA differs by 1049 bytes in text!
        idx_1049 = int(np.flatnonzero(dtau == 1049)[0])
        pos_a = n - 1 - int(tau[idx_1049])
        sample_1049 = text[max(0, pos_a - 20):min(n, pos_a + 80)].tobytes().decode("utf-8", errors="replace")

    # --- W3: Quantum Torus Bivariate Few-Nomial (zw - 1) G_f(z, w) ---
    # Positive terms: (ends[j], img_ends[j]); Negative terms: (starts[j], img_starts[j])
    # Note wrap modulo n? Wait: as a polynomial on Z_{\ge 0}^2 (not mod z^n - 1),
    # (zw - 1) sum_{x=0}^{n-1} z^x w^{f(x)} = sum_{j=0}^{r-1} (z^{b_j} w^{d_j} - z^{a_j} w^{c_j}).
    # A cancellation occurs iff (b_j, d_j) == (a_k, c_k) for some j, k!
    pos_pairs = set(zip(ends.tolist(), img_ends.tolist()))
    neg_pairs = set(zip(starts.tolist(), img_starts.tolist()))
    cancelled = len(pos_pairs & neg_pairs)
    fewnomial_terms = 2 * (r - cancelled)

    # Modulo (z^n - 1, w^n - 1) on the discrete torus (Z/nZ)^2:
    pos_mod = set(zip((ends % n).tolist(), (img_ends % n).tolist()))
    neg_mod = set(zip((starts % n).tolist(), (img_starts % n).tolist()))
    cancelled_mod = len(pos_mod & neg_mod)
    torus_fewnomial_terms = 2 * (r - cancelled_mod)

    # Berlekamp-Massey linear complexity of first 2048 run widths & shifts mod 257
    shifts_mod = ((img_starts[:2048] - starts[:2048]) % 257).tolist()
    bm_shifts_2048 = berlekamp_massey_mod(shifts_mod, 257)

    # --- W5: RSK / Schensted-Greene Young Diagram of LF and Open's g ---
    lf_lis, lf_lds = rsk_lis_and_lds(lf)

    # Build Open<16>'s exact first-return map g on S (size m):
    is_thin = np.zeros(n, dtype=bool)
    for s, e, w in zip(starts, ends, widths):
        if w <= 16:
            is_thin[s:e] = True
        elif s <= p < e:
            is_thin[p] = True
    S_rows = np.flatnonzero(is_thin)
    m = len(S_rows)
    rank_in_S = np.full(n, -1, dtype=np.int32)
    rank_in_S[S_rows] = np.arange(m, dtype=np.int32)

    # E_rows = sorted(lf[S_rows]); note LF_v(j) is index of lf[S_rows[j]] in E_rows!
    f_S = lf[S_rows]
    lf_v_order = np.argsort(f_S, kind="mergesort")
    E_rows = f_S[lf_v_order]
    lf_v = np.empty(m, dtype=np.int32)
    lf_v[lf_v_order] = np.arange(m, dtype=np.int32)

    # P(k): follow lf from E_rows[k] until hitting is_thin!
    P_map = np.empty(m, dtype=np.int32)
    for k_idx, start_row in enumerate(E_rows.tolist()):
        x_cur = start_row
        while not is_thin[x_cur]:
            x_cur = int(lf[x_cur])
        P_map[k_idx] = int(rank_in_S[x_cur])

    g_open = P_map[lf_v]
    g_lis, g_lds = rsk_lis_and_lds(g_open)

    # W10: Cycle structure of LF_v alone and P_map alone!
    lfv_cyc, lfv_fix, lfv_max = count_cycles(lf_v)
    P_cyc, P_fix, P_max = count_cycles(P_map)
    g_cyc, _, g_max = count_cycles(g_open)

    # --- W8/9: Series-Reduced Measured Train Track Core + TTSR-RV + TTO ---
    # Build codomain order of runs (bucket by symbol, stable)
    cod_order = np.argsort(syms, kind="mergesort")
    # Two-pointer merge of codomain intervals [img_starts[u], img_ends[u]) (u = cod_order[mi])
    # and domain intervals [starts[v], ends[v]) (v = 0..r-1)
    arcs_u = []
    arcs_v = []
    arcs_w = []
    arcs_u_off = []
    arcs_v_off = []
    out_deg = np.zeros(r, dtype=np.int32)
    in_deg = np.zeros(r, dtype=np.int32)

    mi = 0
    vi = 0
    cur_pos = 0
    while mi < r and vi < r:
        u = int(cod_order[mi])
        u_end = int(img_ends[u])
        v_end = int(ends[vi])
        nxt = min(u_end, v_end)
        w_arc = nxt - cur_pos
        if w_arc > 0:
            arcs_u.append(u)
            arcs_v.append(vi)
            arcs_w.append(w_arc)
            arcs_u_off.append(cur_pos - int(img_starts[u]))
            arcs_v_off.append(cur_pos - int(starts[vi]))
            out_deg[u] += 1
            in_deg[vi] += 1
        cur_pos = nxt
        if u_end == nxt:
            mi += 1
        if v_end == nxt:
            vi += 1

    # Pass-through runs: in_deg == 1 and out_deg == 1 (excluding the run containing primary p so mark stays in core!)
    p_run = int(np.searchsorted(ends, p, side="right"))
    passthrough = (in_deg == 1) & (out_deg == 1)
    passthrough[p_run] = False
    n_passthrough = int(np.sum(passthrough))
    core_runs = np.flatnonzero(~passthrough)
    n_core_runs = len(core_runs)

    # Verify W1 == 0 in core (except possibly p_run if width[p_run] == 1)
    core_widths = widths[core_runs]
    core_w1 = int(np.sum(core_widths == 1))
    core_thin_mask = core_widths <= 16
    m_TTO = int(np.sum(core_widths[core_thin_mask]))
    r_thin_core = int(np.sum(core_thin_mask))
    r_thick_core = int(np.sum(~core_thin_mask))

    # Build the exact Series-Reduced Core IET!
    # Every core run c in core_runs occupies an interval of width widths[c].
    # In domain order, the outgoing arcs from c (in increasing u_off order!) follow unique pass-through chains
    # until hitting a core run c_target at some v_off!
    # Wait: for any pass-through run u, it has unique outgoing arc -> unique_succ[u] with v_off_succ[u]!
    unique_succ = np.full(r, -1, dtype=np.int32)
    unique_succ_voff = np.zeros(r, dtype=np.int32)
    for u, v, w_a, uo, vo in zip(arcs_u, arcs_v, arcs_w, arcs_u_off, arcs_v_off):
        if out_deg[u] == 1:
            unique_succ[u] = v
            unique_succ_voff[u] = vo

    # Group initial arcs originating at core runs, in domain order (core_runs is sorted, and for each c, sort its outgoing arcs by u_off)
    core_out_lists: list[list[tuple[int, int, int]]] = [[] for _ in range(r)]
    for idx_a, (u, v, w_a, uo, vo) in enumerate(zip(arcs_u, arcs_v, arcs_w, arcs_u_off, arcs_v_off)):
        if not passthrough[u]:
            core_out_lists[u].append((uo, v, vo, w_a))

    core_piece_widths: list[int] = []
    core_piece_landing: list[tuple[int, int, int]] = []  # (target_core_run, target_v_off, piece_id)
    max_chain = 0
    total_chain_hops = 0

    for c in core_runs.tolist():
        lst = sorted(core_out_lists[c], key=lambda t: t[0])
        for uo, v, vo, w_a in lst:
            # Trace through passthrough runs!
            # Note: if passthrough[v], then in_deg[v] == 1, so w_a == widths[v] and vo == 0 ALWAYS!
            hops = 0
            cur_v = v
            cur_vo = vo
            while passthrough[cur_v]:
                assert cur_vo == 0 and w_a == int(widths[cur_v])
                cur_vo = int(unique_succ_voff[cur_v])
                cur_v = int(unique_succ[cur_v])
                hops += 1
            if hops > max_chain:
                max_chain = hops
            total_chain_hops += hops
            pid = len(core_piece_widths)
            core_piece_widths.append(w_a)
            core_piece_landing.append((cur_v, cur_vo, pid))

    assert total_chain_hops == n_passthrough, (total_chain_hops, n_passthrough)
    n_core_pieces = len(core_piece_widths)
    top_order = list(range(n_core_pieces))
    # Codomain order of core pieces: sort by (target_core_run, target_v_off)!
    # (In C/Rust, core_runs sorted by domain index -> bucket per core_run or codomain stream order!)
    core_piece_landing.sort(key=lambda t: (t[0], t[1]))
    bot_order = [pid for _, _, pid in core_piece_landing]

    rv_core_unacc, rv_core_acc = simulate_rv_steps_on_iet(core_piece_widths, top_order, bot_order)

    return {
        "file": fname,
        "n": n,
        "r": r,
        "w1b_koopman_split": {
            "min_tv_w1_m1_32": round(min_tv_w1, 4),
            "min_tv_thin_m1_32": round(min_tv_thin, 4),
            "tv_thick_m1": round(tv_thick_m1, 4),
            "sample_1049_context": sample_1049,
        },
        "w3_quantum_torus_and_lfsr": {
            "cancelled_nonperiodic": cancelled,
            "fewnomial_terms": fewnomial_terms,
            "cancelled_torus_mod_n": cancelled_mod,
            "torus_fewnomial_terms": torus_fewnomial_terms,
            "berlekamp_massey_deg_2048_shifts": bm_shifts_2048,
        },
        "w5_rsk_young_shape": {
            "lf_alpha_bound": int(len(np.unique(L))),
            "lf_lis_lambda1": lf_lis,
            "lf_lds_num_rows": lf_lds,
            "open_m": m,
            "open_g_lis_lambda1": g_lis,
            "open_g_lds_num_rows": g_lds,
        },
        "w10_open_factor_cycles": {
            "lf_v_cycles": lfv_cyc,
            "lf_v_fixed": lfv_fix,
            "lf_v_max_cycle": lfv_max,
            "P_cycles": P_cyc,
            "P_fixed_h0_in_place": P_fix,
            "P_max_cycle": P_max,
            "g_cycles": g_cyc,
            "g_max_cycle": g_max,
        },
        "w89_train_track_core": {
            "passthrough_runs": n_passthrough,
            "passthrough_pct_r": round(100.0 * n_passthrough / r, 2),
            "core_runs": n_core_runs,
            "core_w1_runs": core_w1,
            "core_thin_runs_le16": r_thin_core,
            "core_thick_runs_gt16": r_thick_core,
            "m_TTO_thin_core_rows": m_TTO,
            "m_TTO_vs_open_m_ratio": round(m_TTO / m, 4),
            "core_pieces": n_core_pieces,
            "max_passthrough_chain": max_chain,
            "rv_core_unacc_steps": rv_core_unacc,
            "rv_core_acc_steps": rv_core_acc,
            "rv_core_unacc_over_r": round(rv_core_unacc / r, 4),
            "rv_core_acc_over_r": round(rv_core_acc / r, 4),
        },
    }


def main():
    t0 = time.time()
    manifest = json.loads((BLOCKS_DIR / "manifest.json").read_text())
    results = {}
    for item in manifest["blocks"]:
        fname = item["file"]
        p = item["primary_index"]
        L = np.frombuffer((BLOCKS_DIR / fname).read_bytes(), dtype=np.uint8)
        res = analyze_block_deep(fname, L, p)
        results[fname] = res
        print(f"\n=== {fname} ===")
        print(json.dumps(res, indent=2))

    out_path = pathlib.Path(__file__).parent / "probe_deep_math_results.json"
    out_path.write_text(json.dumps(results, indent=2))
    print(f"\nSaved {out_path} in {time.time() - t0:.2f}s")


if __name__ == "__main__":
    main()
