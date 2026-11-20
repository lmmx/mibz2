#!/usr/bin/env python3
"""
Carbon Kitchen Sink — Cycle 1 Multi-World Mathematical Probe Suite.

Probes distant mathematical frameworks across:
  - Toy 1: Sturmian circle rotation (d=1 torus, p=2)
  - Toy 2: Damaged circle rotation (d=1 torus, p=1024)
  - Toy 3: Tribonacci Pisot / Rauzy fractal word (d=2 complex torus, p=3)
  - Toy 4: Damaged Rauzy fractal coding (d=2 complex torus, p=1024 Voronoï/grid cells on Rauzy tile)
  - Real:  Six authenticated Wikidata BWT blocks (0, 2, 85, 143, 207, 210)

Worlds tested in this script:
  W1. Koopman-Bohr Spectral Theory & Diophantine Torus Obstruction (TV_row, FFT power, Delta-tau law)
  W2. Origami Cone-Cycle Length-Blindness Theorem & SL(2, Z) Shear / Saddle Spectrum
  W4. Free Left-Regular Band (Exact Pre-Column bzip2 MTF + RLE2 Bijection & nn=1 Pinhole Defects)
  W6. Wheeler Context-Synchronising k-Gram Tower (C_1 .. C_6)
  W7/W8. Open Excursion Anatomy (height=0 thin chains vs thick excursions) & Stallings / AHT Folds
"""

from __future__ import annotations
import json
import math
import pathlib
import time
from collections import Counter
import numpy as np

BLOCKS_DIR = pathlib.Path("/home/antigravity/admitted-bwt")


def bwt_from_int_seq(seq: np.ndarray) -> tuple[np.ndarray, int]:
    """Exact cyclic BWT of integer sequence seq (len n <= 10000) for toy experiments."""
    n = len(seq)
    # Build cyclic rotations and sort lexicographically (breaking ties by start position for stability)
    doubled = np.concatenate([seq, seq])
    rots = np.lib.stride_tricks.sliding_window_view(doubled, n)[:n]
    # Lexsort from last col to first col; include arange(n) as tie-breaker
    keys = [np.arange(n)] + [rots[:, k] for k in range(n - 1, -1, -1)]
    sa = np.lexsort(keys)
    L = seq[(sa - 1) % n]
    primary = int(np.where(sa == 0)[0][0])
    return L, primary


def make_toys(n_toy: int = 4096) -> list[tuple[str, np.ndarray, int]]:
    toys = []
    # Toy 1 & 2: Golden ratio rotation on T^1
    alpha = (math.sqrt(5) - 1.0) / 2.0
    ks = np.arange(n_toy, dtype=np.float64)
    phases = (ks * alpha) % 1.0
    seq_p2 = np.floor(phases * 2.0).astype(np.int32)
    L1, p1 = bwt_from_int_seq(seq_p2)
    toys.append(("Sturmian (d=1, p=2)", L1, p1))

    seq_p1024 = np.floor(phases * 1024.0).astype(np.int32)
    L2, p2 = bwt_from_int_seq(seq_p1024)
    toys.append(("Damaged Rotation (d=1, p=1024)", L2, p2))

    # Toy 3 & 4: Tribonacci Pisot substitution (0 -> 0 1, 1 -> 0 2, 2 -> 0)
    # Characteristic polynomial: x^3 - x^2 - x - 1 = 0.
    # Dominant Pisot root beta_1 ~ 1.8392867552141612; complex contracting conjugate beta_2:
    roots = np.roots([1.0, -1.0, -1.0, -1.0])
    # Pick one complex contracting root |z| < 1
    c_roots = [z for z in roots if abs(z.imag) > 1e-9]
    beta = c_roots[0]  # |beta| ~ 0.73735 < 1

    w = [0]
    while len(w) < n_toy:
        nw = []
        for s in w:
            if s == 0:
                nw.extend([0, 1])
            elif s == 1:
                nw.extend([0, 2])
            else:
                nw.append(0)
        w = nw
    w_arr = np.array(w[:n_toy], dtype=np.int32)
    L3, p3 = bwt_from_int_seq(w_arr)
    toys.append(("Tribonacci Rauzy (d=2, p=3)", L3, p3))

    # Compute exact Rauzy fractal torus coordinate in C for Tribonacci!
    # In the Dumont-Thomas / Rauzy abelianization map, stepping forward along w adds v_{w[k]}
    # projected onto the contracting eigenspace of M^T:
    # M = [[1,1,1],[1,0,0],[0,1,0]] (columns = images of 0,1,2)
    # Left eigenvector of M for eigenvalue beta satisfies u M = beta u:
    # u_1 = beta u_2 => wait: [u0, u1, u2] [[1,1,1],[1,0,0],[0,1,0]] = [u0+u1, u0+u2, u0] = beta [u0, u1, u2]
    # => u2 = u0 / beta, u1 = u0 (beta - 1). Let u0 = 1.
    u_vec = np.array([1.0 + 0j, beta - 1.0, 1.0 / beta], dtype=np.complex128)
    # Notice sum along any substitution word shrinks mod (beta-eigenspace? Wait: beta is contracting,
    # while on T^2 = R^2 / Z^2, the conjugate projection modulo the dual latticeprojects S^k(0) onto the bounded Rauzy fractal!)
    # Wait: the standard Rauzy fractal point z_k in C is obtained from the Ostrowski/Tribonacci representation of k:
    # Greedy Tribonacci digits k = sum d_j T_j => z_k = sum d_j beta^j (bounded in C!).
    trib = [1, 2, 4]
    while trib[-1] <= n_toy:
        trib.append(trib[-1] + trib[-2] + trib[-3])
    trib_rev = trib[::-1]
    zk = np.zeros(n_toy, dtype=np.complex128)
    powers = np.array([beta ** j for j in range(len(trib))], dtype=np.complex128)[::-1]
    for k in range(n_toy):
        rem = k
        val = 0j
        for idx, tval in enumerate(trib_rev):
            if rem >= tval:
                rem -= tval
                val += powers[idx]
        zk[k] = val
    # Partition the bounding box of the 2D Rauzy fractal into 32 x 32 = 1024 fine grid cells!
    rx = zk.real
    ry = zk.imag
    gx = np.clip(np.floor((rx - rx.min()) / (rx.max() - rx.min() + 1e-12) * 32.0).astype(np.int32), 0, 31)
    gy = np.clip(np.floor((ry - ry.min()) / (ry.max() - ry.min() + 1e-12) * 32.0).astype(np.int32), 0, 31)
    seq_rf_1024 = gx * 32 + gy
    L4, p4 = bwt_from_int_seq(seq_rf_1024)
    toys.append(("Damaged Rauzy Fractal (d=2, p=1024)", L4, p4))
    return toys


def build_lf_and_tau(L: np.ndarray, p: int) -> tuple[np.ndarray, np.ndarray, int]:
    """Compute LF array, orbit time tau[x] (steps from p: tau[f^t(p)] = t), and maximal run count r."""
    n = len(L)
    r = int(1 + np.sum(L[1:] != L[:-1]))
    # Stable rank LF via argsort with kind='mergesort'
    order = np.argsort(L, kind="mergesort")
    # order[y] = x means f(x) = y
    lf = np.empty(n, dtype=np.int32)
    lf[order] = np.arange(n, dtype=np.int32)

    tau = np.full(n, -1, dtype=np.int32)
    cur = p
    for t in range(n):
        tau[cur] = t
        cur = int(lf[cur])
        if cur == p:
            break
    return lf, tau, r


def probe_w1_koopman(name: str, L: np.ndarray, p: int, lf: np.ndarray, tau: np.ndarray, r: int) -> dict:
    """World 1: Koopman-Bohr spectral coherence & Delta-tau gap distribution."""
    n = len(L)
    dtau = (np.roll(tau, -1) - tau) % n
    distinct_dtau = len(np.unique(dtau))
    vals, cnts = np.unique(dtau, return_counts=True)
    top_idx = np.argsort(cnts)[::-1][:5]
    top_dtau = [(int(vals[i]), int(cnts[i])) for i in top_idx]

    # Fundamental Koopman eigenfunction psi_1[x] = exp(2 pi i tau[x] / n)
    psi1 = np.exp(2j * np.pi * tau.astype(np.float64) / n)
    spec = np.fft.fft(psi1)
    # Normalize so Parseval sum_{xi} |spec[xi]|^2 / n^2 = 1; random permutation has E[|spec|^2 / n] = 1, max ~ log n
    power = (np.abs(spec[1:]) ** 2) / n
    max_fft_power = float(np.max(power))
    arg_fft = int(np.argmax(power) + 1)

    # Minimum normalized Koopman TV_row across candidate multipliers m:
    # Random uniform phase difference gives E[2|sin(pi U)|] = 4/pi ~ 1.2732395.
    norm_const = (4.0 / math.pi) * n
    # Candidate m's: small m (1..64), plus modular inverses / resonances of top dtau values, plus random sample of 128 m's
    cand_m = set(range(1, min(n, 65)))
    for v, _ in top_dtau:
        if v > 0:
            # m such that m * v / n is near 1 or small integer
            for k in range(1, 8):
                m_approx = int(round(k * n / v)) % n
                if m_approx > 0:
                    cand_m.add(m_approx)
            g = math.gcd(v, n)
            if g > 1 and (n // g) < n:
                cand_m.add(n // g)
    # Also for toys, test Fibonacci / Tribonacci denominators or golden ratio multiples!
    rng = np.random.default_rng(42)
    if n > 100:
        for m_rand in rng.integers(1, n, size=128):
            cand_m.add(int(m_rand))
    # If n <= 8192 (toys!), sweep ALL m in 1..n-1 exactly!
    if n <= 8192:
        ms = np.arange(1, n, dtype=np.int64)
        # Vectorized in chunks of 256
        min_tv_ratio = 999.0
        best_m = 1
        for start in range(0, len(ms), 256):
            batch = ms[start:start + 256][:, None]
            tvs = np.sum(2.0 * np.abs(np.sin(np.pi * ((batch * dtau[None, :]) % n) / n)), axis=1) / norm_const
            idx = int(np.argmin(tvs))
            if tvs[idx] < min_tv_ratio:
                min_tv_ratio = float(tvs[idx])
                best_m = int(ms[start + idx])
    else:
        min_tv_ratio = 999.0
        best_m = 1
        for m in sorted(cand_m):
            tv = float(np.sum(2.0 * np.abs(np.sin(np.pi * ((m * dtau.astype(np.int64)) % n) / n))) / norm_const)
            if tv < min_tv_ratio:
                min_tv_ratio = tv
                best_m = m

    return {
        "name": name,
        "n": n,
        "r": r,
        "mean_width": round(n / r, 2),
        "distinct_dtau": distinct_dtau,
        "distinct_dtau_over_r": round(distinct_dtau / r, 4),
        "top_dtau": top_dtau,
        "max_fft_power_norm": round(max_fft_power, 2),
        "max_fft_freq": arg_fft,
        "min_koopman_tv_ratio": round(min_tv_ratio, 4),
        "best_koopman_m": best_m,
    }


def cycle_lengths_perm(perm: np.ndarray) -> list[int]:
    n = len(perm)
    vis = np.zeros(n, dtype=np.uint8)
    lens = []
    for i in range(n):
        if not vis[i]:
            c = 0
            cur = i
            while not vis[cur]:
                vis[cur] = 1
                cur = int(perm[cur])
                c += 1
            if c > 1:
                lens.append(c)
    lens.sort(reverse=True)
    return lens


def probe_w2_origami(L: np.ndarray, lf: np.ndarray) -> dict:
    """World 2: Verify Cone-Cycle Length-Blindness Theorem & SL(2, Z) shear cycle counts."""
    n = len(L)
    inv_lf = np.empty(n, dtype=np.int32)
    inv_lf[lf] = np.arange(n, dtype=np.int32)
    # Commutator k(x) = r(u(r^-1(u^-1(x)))) where r = lf, u(x) = (x + 1) % n
    # Apply right-to-left: x1 = (x - 1) % n; x2 = inv_lf[x1]; x3 = (x2 + 1) % n; k[x] = lf[x3]
    x = np.arange(n, dtype=np.int32)
    k_full = lf[(inv_lf[(x - 1) % n] + 1) % n]
    cone_full = cycle_lengths_perm(k_full)

    # Reduced width-1 run-symbol column w_runs of length r!
    run_mask = np.empty(n, dtype=bool)
    run_mask[0] = True
    run_mask[1:] = L[1:] != L[:-1]
    w_runs = L[run_mask]
    r_len = len(w_runs)
    order_w = np.argsort(w_runs, kind="mergesort")
    lf_w = np.empty(r_len, dtype=np.int32)
    lf_w[order_w] = np.arange(r_len, dtype=np.int32)
    inv_lf_w = np.empty(r_len, dtype=np.int32)
    inv_lf_w[lf_w] = np.arange(r_len, dtype=np.int32)
    xw = np.arange(r_len, dtype=np.int32)
    k_red = lf_w[(inv_lf_w[(xw - 1) % r_len] + 1) % r_len]
    cone_red = cycle_lengths_perm(k_red)

    # SL(2, Z) shears: sigma_s(x) = (lf[x] - s * x? Wait: Nielsen move r * u^-s is x -> lf[(x - s) % n]!)
    shears = {}
    for s in (-2, -1, 1, 2):
        perm_s = lf[(x - s) % n]
        vis = np.zeros(n, dtype=np.uint8)
        n_cyc = 0
        n_fix = int(np.sum(perm_s == x))
        for i in range(n):
            if not vis[i]:
                n_cyc += 1
                cur = i
                while not vis[cur]:
                    vis[cur] = 1
                    cur = int(perm_s[cur])
        shears[f"r*u^{-s}"] = {"cycles": n_cyc, "fixed": n_fix}

    return {
        "cone_cycles_full": cone_full,
        "cone_cycles_reduced_w_runs": cone_red,
        "exact_cone_isomorphism": cone_full == cone_red,
        "sl2z_shears": shears,
    }


def probe_w4_mtf_lrb(L: np.ndarray) -> dict:
    """World 4: Reconstruct exact bzip2 MTF+RLE2 stream & Left-Regular Band recency / pinhole defects."""
    n = len(L)
    # Active alphabet in ascending byte order (exact libbzip2 seq_to_unseq)
    present = sorted(set(L.tolist()))
    alpha = len(present)
    yy = list(present)

    # Extract maximal runs
    change = np.flatnonzero(L[1:] != L[:-1]) + 1
    starts = np.concatenate([[0], change])
    ends = np.concatenate([change, [n]])
    widths = ends - starts
    syms = L[starts].tolist()
    r = len(syms)

    mtf_indices = []
    rle2_syms = 0
    for j, (c, w) in enumerate(zip(syms, widths.tolist())):
        idx = yy.index(c)
        if idx == 0:
            # Only possible at j == 0!
            assert j == 0, f"Impossible idx==0 at j={j}"
            rem = w
        else:
            mtf_indices.append(idx)
            yy.pop(idx)
            yy.insert(0, c)
            rem = w - 1
        # Bijective base-2 count of RUNA/RUNB symbols for rem
        while rem > 0:
            rem = (rem - 1) // 2
            rle2_syms += 1

    nn_arr = np.array(mtf_indices, dtype=np.int32)
    # Pinhole defects: run j has nn == 1 (so syms[j] == syms[j-2]) AND middle run j-1 has width == 1!
    # Wait: align indices carefully (if j=0 had idx==0, mtf_indices has len r-1, else len r).
    offset = r - len(mtf_indices)
    pinhole_w1 = 0
    pinhole_thin_coalesce = 0
    for k_idx, nn in enumerate(mtf_indices):
        j = k_idx + offset
        if nn == 1 and j >= 2:
            assert syms[j] == syms[j - 2]
            if widths[j - 1] == 1:
                pinhole_w1 += 1
                if widths[j] <= 16 and widths[j - 2] <= 16 and (widths[j] + widths[j - 2]) > 16:
                    pinhole_thin_coalesce += 1

    return {
        "alpha": alpha,
        "r": r,
        "total_bzip2_syms": len(mtf_indices) + rle2_syms + 1,  # +1 for EOB
        "mtf_nn1_share": round(float(np.mean(nn_arr == 1)), 4),
        "mtf_nn_le2_share": round(float(np.mean(nn_arr <= 2)), 4),
        "mtf_nn_le4_share": round(float(np.mean(nn_arr <= 4)), 4),
        "mtf_nn_le16_share": round(float(np.mean(nn_arr <= 16)), 4),
        "mtf_nn_le32_share": round(float(np.mean(nn_arr <= 32)), 4),
        "pinhole_aba_w1_count": pinhole_w1,
        "pinhole_thin_to_thick_count": pinhole_thin_coalesce,
    }


def probe_w6_wheeler_kgrams(L: np.ndarray, lf: np.ndarray) -> dict:
    """World 6: Wheeler k-gram context partition sizes N_k and unique-context row share."""
    n = len(L)
    # Compute SA from lf? Or compute k-gram intervals iteratively by monotone bucket refinement!
    # Wait: C_1 is partition of [0, n) by F_0 (sorted L).
    # If partition P_k divides [0, n) into intervals [s_i, e_i), then P_{k+1} is obtained by:
    # scanning domain runs (or rows) x = 0..n-1 with class id_k[x], grouping by L[x] in stable order!
    # Even simpler in numpy for exact census: maintain class_id[0..n-1] of M_x[0..k)!
    F0 = np.sort(L)
    _, class_id = np.unique(F0, return_inverse=True)
    # Wait: M_y[0..k+1) = (F0[y], M_{\Psi(y)}[0..k))!
    # Since y = lf[x], M_{lf[x]}[0..k+1) = (L[x], M_x[0..k)), and as x goes 0..n-1,
    # M_x[0..k) is ALREADY monotonically non-decreasing!!
    # So within each bucket of L[x], class_id[x] is monotonically non-decreasing!
    res = {}
    for k in range(1, 9):
        # Count intervals and singleton intervals in current sorted class_id
        # Because M_x is sorted lexicographically, equal k-grams ALWAYS form a contiguous interval of rows!
        cuts = np.flatnonzero(class_id[1:] != class_id[:-1]) + 1
        starts = np.concatenate([[0], cuts])
        ends = np.concatenate([cuts, [n]])
        widths = ends - starts
        n_intervals = len(widths)
        singleton_rows = int(np.sum(widths == 1))
        thin_rows_k = int(np.sum(widths[widths <= 16]))
        res[f"k={k}"] = {
            "N_k": n_intervals,
            "unique_rows_pct": round(100.0 * singleton_rows / n, 2),
            "thin16_rows_pct": round(100.0 * thin_rows_k / n, 2),
        }
        if k < 8:
            # Advance to k+1: at row y = lf[x], pair is (L[x], class_id[x]).
            # Since y = lf[x] is sorted by (L[x], x) and class_id[x] is monotone in x,
            # new_pair[y] = (L[x], class_id[x]) is monotonically non-decreasing in y!
            y_c0 = np.empty(n, dtype=np.int32)
            y_c1 = np.empty(n, dtype=np.int32)
            y_c0[lf] = L
            y_c1[lf] = class_id
            diff = (y_c0[1:] != y_c0[:-1]) | (y_c1[1:] != y_c1[:-1])
            new_id = np.zeros(n, dtype=np.int32)
            new_id[1:] = np.cumsum(diff)
            class_id = new_id
    return res


def probe_w7_w8_open_and_folds(L: np.ndarray, p: int, lf: np.ndarray) -> dict:
    """Worlds 7 & 8: Open<16> excursion anatomy + Run-Network exact matches & Stallings folds."""
    n = len(L)
    change = np.flatnonzero(L[1:] != L[:-1]) + 1
    starts = np.concatenate([[0], change])
    ends = np.concatenate([change, [n]])
    widths = ends - starts
    r = len(widths)

    # Exact whole-run matches f(I_a) == I_b and out_deg == 1 count via searchsorted
    img_starts = lf[starts]
    img_ends = img_starts + widths  # because inside each run, lf(x) = lf(start) + (x - start)
    # Run j has out_deg == 1 iff starts[k] <= img_starts[j] and img_ends[j] <= ends[k] for k = searchsorted(ends, img_starts[j], side='right')
    target_run = np.searchsorted(ends, img_starts, side="right")
    out_deg_1 = img_ends <= ends[target_run]
    exact_match = (img_starts == starts[target_run]) & (img_ends == ends[target_run])

    # Open<16> retained mask S (matching src/open.rs: thin iff width <= 16, plus singleton p if p in thick run)
    is_thin_row = np.zeros(n, dtype=bool)
    for s, e, w in zip(starts, ends, widths):
        if w <= 16:
            is_thin_row[s:e] = True
        elif s <= p < e:
            is_thin_row[p] = True
    m = int(np.sum(is_thin_row))
    w1_rows = int(np.sum(widths == 1))
    w2_16_runs = int(np.sum((widths >= 2) & (widths <= 16)))
    w2_16_rows = int(np.sum(widths[(widths >= 2) & (widths <= 16)]))

    # For each retained row x in S, check whether lf[x] is ALREADY in S (height == 0: direct thin->thin!)
    thin_rows = np.flatnonzero(is_thin_row)
    targets = lf[thin_rows]
    height_zero_mask = is_thin_row[targets]
    h0_count = int(np.sum(height_zero_mask))
    h_pos_count = m - h0_count  # number of thick excursions (= switch count h in proof.md §4!)

    # Measure distribution of maximal thin->thin chain lengths (excursions inside S between thick visits!)
    # Follow orbit from p and measure consecutive visits in S before hitting S^c!
    thin_streaks = []
    cur = p
    streak = 0
    for _ in range(n):
        if is_thin_row[cur]:
            streak += 1
        else:
            if streak > 0:
                thin_streaks.append(streak)
                streak = 0
        cur = int(lf[cur])
    if streak > 0:
        thin_streaks.append(streak)
    streaks_arr = np.array(thin_streaks, dtype=np.int32) if thin_streaks else np.array([0], dtype=np.int32)

    return {
        "r": r,
        "m_retained": m,
        "m_pct_n": round(100.0 * m / n, 2),
        "w1_runs_and_rows": w1_rows,
        "w2_16_runs": w2_16_runs,
        "w2_16_rows": w2_16_rows,
        "out_deg_1_runs": int(np.sum(out_deg_1)),
        "out_deg_1_pct_r": round(100.0 * float(np.mean(out_deg_1)), 2),
        "exact_whole_run_matches": int(np.sum(exact_match)),
        "exact_match_pct_r": round(100.0 * float(np.mean(exact_match)), 2),
        "open_height0_thin_to_thin": h0_count,
        "open_height0_pct_m": round(100.0 * h0_count / m, 2),
        "open_thick_excursions_h": h_pos_count,
        "open_thick_excursions_pct_n": round(100.0 * h_pos_count / n, 3),
        "thin_streak_mean": round(float(np.mean(streaks_arr)), 2),
        "thin_streak_p50": int(np.median(streaks_arr)),
        "thin_streak_p90": int(np.percentile(streaks_arr, 90)),
        "thin_streak_max": int(np.max(streaks_arr)),
    }


def main():
    t0 = time.time()
    out = {"toys": [], "blocks": {}}

    print("=== 1. Toy & Damaged-Toy Koopman/Rauzy Experiments ===")
    for name, L_toy, p_toy in make_toys(4096):
        lf, tau, r = build_lf_and_tau(L_toy, p_toy)
        res = probe_w1_koopman(name, L_toy, p_toy, lf, tau, r)
        out["toys"].append(res)
        print(json.dumps(res))

    manifest = json.loads((BLOCKS_DIR / "manifest.json").read_text())
    print("\n=== 2. Authenticated Wikidata Blocks (6 blocks) ===")
    for item in manifest["blocks"]:
        fname = item["file"]
        p = item["primary_index"]
        raw = (BLOCKS_DIR / fname).read_bytes()
        L = np.frombuffer(raw, dtype=np.uint8)
        lf, tau, r = build_lf_and_tau(L, p)
        assert r == item["r"], (fname, r, item["r"])

        w1 = probe_w1_koopman(fname, L, p, lf, tau, r)
        w2 = probe_w2_origami(L, lf)
        w4 = probe_w4_mtf_lrb(L)
        w6 = probe_w6_wheeler_kgrams(L, lf) if fname in ("block-0.L", "block-210.L") else {}
        w78 = probe_w7_w8_open_and_folds(L, p, lf)

        out["blocks"][fname] = {
            "w1_koopman": w1,
            "w2_origami": w2,
            "w4_mtf_lrb": w4,
            "w6_wheeler": w6,
            "w78_open_folds": w78,
        }
        print(f"\n--- {fname} ---")
        print("W1 Koopman:", json.dumps(w1))
        print("W2 Origami:", json.dumps(w2))
        print("W4 MTF/LRB:", json.dumps(w4))
        if w6:
            print("W6 Wheeler:", json.dumps(w6))
        print("W7/8 Open & Folds:", json.dumps(w78))

    out_path = pathlib.Path(__file__).parent / "probe_worlds_results.json"
    out_path.write_text(json.dumps(out, indent=2))
    print(f"\nSaved {out_path} in {time.time() - t0:.2f}s")


if __name__ == "__main__":
    main()
