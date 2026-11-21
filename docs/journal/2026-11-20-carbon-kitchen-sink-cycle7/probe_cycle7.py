#!/usr/bin/env python3
"""
Carbon Kitchen Sink — Cycle 7 Mathematical Probe & Exact Verifier (probe_cycle7.py)

"Decoding Without an Orbit Walk?": Investigates three collective non-walk mathematical
reformulations of inverse BWT:

1. Theorem 7.1 — The Burnside-BEST De Bruijn Eulerian Identity, Branching Unitig Reduction,
   and Kotzig-Pevzner Condensation Phase Transition:
   - Exact identity:
       sum_{[T'] in W(G_k(T))} (per(T') / n)
         == det(L'_k) * prod_{u in V_k} (d^+(u) - 1)! / prod_{(u,v)} m_k(u, v)!
         == det(L'^{br}_k) * prod_{u in V_k^{br}} [ (d^+(u) - 1)! / prod_c m_k(u, uc)! ]
     Verified exhaustively on ALL binary (2 <= n <= 8) and ternary (2 <= n <= 6) cyclic words
     across k = 1, 2, 3, 4 against brute-force necklace enumeration!
   - Measures on all 6 Wikidata blocks:
     * Exact log2 Z_k(T) and local branching entropy H_loc(G_k) across k = 1..4096,
     * Exact sharp Kotzig phase-transition threshold k*(T) (unique Eulerian condensation Z_k = 1)
       vs naive repeat threshold L_max(T) + 1 (zero branching vertices V_k^{br} = empty)!

2. Theorem 7.2 — Dyadic Interval-Exchange Exponentiation LF^{2^j} via Pointer-Free Stream Merges:
   - Verifies exact O(r(f) + r(g)) stream-merge composition of piecewise unit-slope bijections
     (zero random array gathers), sub-additivity r(g o f) <= r(f) + r(g) - 1, and time-reversal
     symmetry r(LF^N) == r(LF^{n - N}) on small BWTs, Sturmian & Tribonacci Rauzy toys, and
     the 6 Wikidata blocks.
   - Measures the exact dyadic piece-complexity curve r(LF^{2^j}) for j = 0, 1, ..., 19 on both
     structured toys and all 6 Wikidata blocks!

3. Theorem 7.3 — Fourier-Space Cauchy-Like Structure of P_hat_LF & Mantaci-Restivo-Sciortino
   Zero-Defect Criterion:
   - Verifies exact Fourier-space Sylvester/Cauchy identity:
       (1 - zeta^{j - k}) (F P_LF F^{-1})_{j, k}
         == (1/n) sum_{u=0}^{r-1} zeta^{j d_u} (zeta^{-k c_u} - zeta^{-k (LF((d_u - 1) % n) + 1)})
     exhaustively on small BWTs via exact DFT!
   - Counts exact active Cauchy generators r_cauchy on all 6 Wikidata blocks, pricing superfast
     Cauchy/NTT resolvent inversion against O(n).
"""

from __future__ import annotations
from collections import Counter, defaultdict
import itertools
import json
import math
from pathlib import Path
import numpy as np

ADMITTED_DIR = Path("/home/antigravity/admitted-bwt")
OUT_PATH = Path(__file__).resolve().parent / "probe_cycle7_results.json"


def load_manifest():
    with open(ADMITTED_DIR / "manifest.json", "r", encoding="utf-8") as f:
        return json.load(f)["blocks"]


def canonical_necklace(w: tuple[int, ...]) -> tuple[int, ...]:
    n = len(w)
    return min(w[i:] + w[:i] for i in range(n))


def minimal_period(w: tuple[int, ...]) -> int:
    n = len(w)
    for p in range(1, n + 1):
        if n % p == 0 and w == w[:p] * (n // p):
            return p
    return n


def kplus1_spectrum(w: tuple[int, ...], k: int) -> tuple[tuple[tuple[int, ...], int], ...]:
    n = len(w)
    ww = (w * ((n + k) // n + 2))[: n + k]
    cnt = Counter(ww[i : i + k + 1] for i in range(n))
    return tuple(sorted(cnt.items()))


def burnside_best_exact_small(w: tuple[int, ...], k: int) -> tuple[float, float]:
    """
    Returns (full_best_value, condensed_branching_best_value) for cyclic word `w` at order `k`.
    """
    n = len(w)
    ww = (w * ((n + k) // n + 2))[: n + k]
    k1_counts = Counter(ww[i : i + k + 1] for i in range(n))
    v_set = sorted({kp1[:-1] for kp1 in k1_counts})
    v_idx = {u: i for i, u in enumerate(v_set)}
    nv = len(v_set)

    # 1. Full De Bruijn multigraph G_k
    out_deg = [0] * nv
    L_mat = np.zeros((nv, nv), dtype=np.float64)
    denom_fact = 1
    for kp1, m in k1_counts.items():
        u = v_idx[kp1[:-1]]
        v = v_idx[kp1[1:]]
        out_deg[u] += m
        L_mat[u, u] += m
        L_mat[u, v] -= m
        denom_fact *= math.factorial(m)

    det_full = 1.0 if nv <= 1 else float(np.linalg.det(L_mat[:-1, :-1]))
    num_fact = 1
    for d in out_deg:
        num_fact *= math.factorial(d - 1)
    val_full = (det_full * num_fact) / denom_fact

    # 2. Condensed Right-Branching Multigraph G_k^{br}
    out_by_u: dict[tuple[int, ...], list[tuple[tuple[int, ...], int]]] = defaultdict(list)
    for kp1, m in k1_counts.items():
        out_by_u[kp1[:-1]].append((kp1[1:], m))

    br_verts = sorted(u for u, lst in out_by_u.items() if len(lst) >= 2)
    if not br_verts:
        # Pure cycle of length |V_k|, multiplicity n / |V_k|
        val_cond = len(v_set) / n
        return val_full, val_cond

    br_idx = {u: i for i, u in enumerate(br_verts)}
    nb = len(br_verts)
    L_br = np.zeros((nb, nb), dtype=np.float64)
    log_ratio = 0.0
    for u in br_verts:
        iu = br_idx[u]
        du = sum(m for _, m in out_by_u[u])
        log_ratio += math.lgamma(du)  # ln((du - 1)!)
        for v_next, m in out_by_u[u]:
            log_ratio -= math.lgamma(m + 1)  # ln(m!)
            # Follow deterministic non-branching chain until hitting br_idx!
            cur = v_next
            while cur not in br_idx:
                cur = out_by_u[cur][0][0]
            iv = br_idx[cur]
            L_br[iu, iu] += m
            L_br[iu, iv] -= m

    det_br = 1.0 if nb <= 1 else float(np.linalg.det(L_br[:-1, :-1]))
    val_cond = det_br * math.exp(log_ratio)
    return val_full, val_cond


def verify_theorem_7_1_small() -> int:
    """Exhaustively verify Burnside-BEST & Branching Unitig Reduction on binary & ternary words."""
    checked = 0
    for alph, max_n in (((0, 1), 8), ((0, 1, 2), 6)):
        for n in range(2, max_n + 1):
            necklaces = sorted({canonical_necklace(w) for w in itertools.product(alph, repeat=n)})
            for k in (1, 2, 3, 4):
                by_spec: dict[tuple, list[tuple[int, ...]]] = defaultdict(list)
                for nk in necklaces:
                    by_spec[kplus1_spectrum(nk, k)].append(nk)
                for spec, group in by_spec.items():
                    exact_lhs = sum(minimal_period(nk) / n for nk in group)
                    rep = group[0]
                    v_full, v_cond = burnside_best_exact_small(rep, k)
                    assert abs(v_full - exact_lhs) < 1e-7, f"BEST mismatch: {v_full} vs {exact_lhs}"
                    assert abs(v_cond - exact_lhs) < 1e-7, f"Condensed mismatch: {v_cond} vs {exact_lhs}"
                    checked += 1
    return checked


# ---------------------------------------------------------------------------
# Theorem 7.2: Pointer-Free Stream-Merge Composition of Interval Exchanges
# ---------------------------------------------------------------------------

def iet_runs_from_dense(f: list[int] | np.ndarray) -> list[tuple[int, int, int]]:
    """Extract maximal unit-slope intervals (dom_start, width, cod_start) sorted by dom_start."""
    n = len(f)
    runs = []
    i = 0
    while i < n:
        c0 = int(f[i])
        j = i + 1
        while j < n and int(f[j]) == c0 + (j - i):
            j += 1
        runs.append((i, j - i, c0))
        i = j
    return runs


def compose_iet_stream_merge(
    f_runs: list[tuple[int, int, int]], g_runs: list[tuple[int, int, int]]
) -> list[tuple[int, int, int]]:
    """
    Compose h = g o f (h(x) = g(f(x))) using PURE STREAM MERGING + sorting of run records,
    with ZERO O(n) arrays or random pointer chases!
      1. Sort `f_runs` by `cod_start` (already partitions [0, n) monotonically).
      2. Linear two-pointer merge of `f_by_cod` against `g_runs` (already sorted by `dom_start`!).
      3. Sort split pieces back by `f_dom_start` and coalesce adjacent co-translated pieces!
    """
    f_cod = sorted(f_runs, key=lambda t: t[2])
    pieces = []
    fi = 0
    gi = 0
    f_off = 0
    g_off = 0
    nf = len(f_cod)
    ng = len(g_runs)
    while fi < nf and gi < ng:
        fd, fw, fc = f_cod[fi]
        gd, gw, gc = g_runs[gi]
        cur_pos = fc + f_off
        assert cur_pos == gd + g_off, f"Partition mismatch: {cur_pos} vs {gd + g_off}"
        rem_f = fw - f_off
        rem_g = gw - g_off
        step = rem_f if rem_f < rem_g else rem_g
        pieces.append((fd + f_off, step, gc + g_off))
        f_off += step
        g_off += step
        if f_off == fw:
            fi += 1
            f_off = 0
        if g_off == gw:
            gi += 1
            g_off = 0

    pieces.sort(key=lambda t: t[0])
    # Coalesce adjacent co-translated intervals!
    merged = []
    for d0, w0, c0 in pieces:
        if merged and merged[-1][0] + merged[-1][1] == d0 and merged[-1][2] + merged[-1][1] == c0:
            pd, pw, pc = merged[-1]
            merged[-1] = (pd, pw + w0, pc)
        else:
            merged.append((d0, w0, c0))
    return merged


def verify_theorem_7_2_and_7_3_small() -> dict:
    """Verify IET stream-merge composition, symmetry r(LF^N)==r(LF^{n-N}), and Fourier Cauchy rank!"""
    iet_checks = 0
    fourier_checks = 0

    for n in range(2, 8):
        zeta = np.exp(2j * np.pi / n)
        F_dft = np.array([[zeta ** (j * x) for x in range(n)] for j in range(n)], dtype=np.complex128)
        F_inv = np.conj(F_dft) / n

        for w in itertools.product((0, 1), repeat=n):
            if minimal_period(w) != n:
                continue
            rots = sorted((w[i:] + w[:i], i) for i in range(n))
            L = [rot[-1] for rot, _ in rots]
            cnt0 = L.count(0)
            occ = [0, cnt0]
            lf = []
            for b in L:
                lf.append(occ[b]); occ[b] += 1

            # 1. Verify stream-merge squaring LF, LF^2, LF^4 and r(LF^s) == r(LF^{n-s})
            r1 = iet_runs_from_dense(lf)
            r2 = compose_iet_stream_merge(r1, r1)
            lf2 = [lf[lf[x]] for x in range(n)]
            assert r2 == iet_runs_from_dense(lf2)
            assert len(r2) <= 2 * len(r1) - 1
            r4 = compose_iet_stream_merge(r2, r2)
            lf4 = [lf2[lf2[x]] for x in range(n)]
            assert r4 == iet_runs_from_dense(lf4)

            cur = list(range(n))
            powers = [cur]
            for _ in range(n):
                cur = [lf[x] for x in cur]
                powers.append(cur)
            assert powers[n] == list(range(n))
            for s in range(1, n):
                assert len(iet_runs_from_dense(powers[s])) == len(iet_runs_from_dense(powers[n - s]))
            iet_checks += 1

            # 2. Verify exact Fourier-space Cauchy-like identity (Theorem 7.3)!
            P_lf = np.zeros((n, n), dtype=np.complex128)
            for x in range(n):
                P_lf[x, lf[x]] = 1.0
            P_hat = F_dft @ P_lf @ F_inv

            run_d = [0] + [x for x in range(1, n) if L[x] != L[x - 1]]
            lhs = np.zeros((n, n), dtype=np.complex128)
            rhs = np.zeros((n, n), dtype=np.complex128)
            for j in range(n):
                for k in range(n):
                    lhs[j, k] = (1.0 - zeta ** (j - k)) * P_hat[j, k]
                    s_val = 0.0j
                    for d_u in run_d:
                        c_u = lf[d_u]
                        prev_c = lf[(d_u - 1) % n] + 1
                        s_val += (zeta ** (j * d_u)) * (
                            zeta ** (-k * c_u) - zeta ** (-k * prev_c)
                        )
                    rhs[j, k] = s_val / n
            assert np.max(np.abs(lhs - rhs)) < 1e-10, "Fourier Cauchy identity failed!"
            fourier_checks += 1

    # Also build a 2D Tribonacci Rauzy morphic word (0->01, 1->02, 2->0) of length 1,951 (or 1069/3589)
    # Wait: lengths of Tribonacci words: 1, 2, 4, 7, 13, 24, 44, 81, 149, 274, 504, 927, 1705, 3136!
    tw = [0]
    sub = {0: [0, 1], 1: [0, 2], 2: [0]}
    for _ in range(12):  # length 1705
        nxt = []
        for c in tw:
            nxt.extend(sub[c])
        tw = nxt
    n_t = len(tw)
    rots_t = sorted((tuple(tw[i:] + tw[:i]), i) for i in range(n_t))
    L_t = [rot[-1] for rot, _ in rots_t]
    c_t = [0, 0, 0]
    for b in L_t: c_t[b] += 1
    occ_t = [0, c_t[0], c_t[0] + c_t[1]]
    lf_t = []
    for b in L_t:
        lf_t.append(occ_t[b]); occ_t[b] += 1
    f_t = np.array(lf_t, dtype=np.int32)
    tribonacci_dyadic = {}
    for j in range(12):
        rj = int(np.count_nonzero(f_t[1:] != f_t[:-1] + 1) + 1)
        tribonacci_dyadic[f"2^{j}"] = rj
        f_t = f_t[f_t]

    return {
        "iet_stream_merge_and_symmetry_verified": iet_checks,
        "fourier_cauchy_dft_verified": fourier_checks,
        "tribonacci_n1705_dyadic_r": tribonacci_dyadic,
    }


# ---------------------------------------------------------------------------
# Block-level De Bruijn Condensation, Dyadic Squaring & Fourier Cauchy Census
# ---------------------------------------------------------------------------

def build_cyclic_sa_and_lcp(L: bytes, p_mark: int) -> tuple[bytes, list[int], list[int], np.ndarray]:
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    occ = [0] * 256
    acc = 0
    for c in range(256):
        occ[c] = acc; acc += counts[c]
    lf = np.empty(n, dtype=np.int32)
    for x, b in enumerate(L):
        lf[x] = occ[b]; occ[b] += 1

    T_arr = bytearray(n)
    sa = [0] * n
    rank = [0] * n
    cur = p_mark
    for s in range(n):
        pos = (n - 1 - s) % n
        T_arr[pos] = L[cur]
        start_rot = (pos + 1) % n
        sa[cur] = start_rot
        rank[start_rot] = cur
        cur = int(lf[cur])
    assert cur == p_mark
    T = bytes(T_arr)

    # Kasai linear-time cyclic LCP
    T2 = T + T
    lcp = [0] * n
    h = 0
    for i in range(n):
        rk = rank[i]
        if rk == 0:
            h = 0
            continue
        j = sa[rk - 1]
        if h > 0:
            h -= 1
        while h < n and T2[i + h] == T2[j + h]:
            h += 1
        lcp[rk] = h

    return T, sa, lcp, lf


def debruijn_stats_at_k(n: int, sa: list[int], lcp: list[int], k: int, max_det_dim: int = 1600) -> dict:
    """
    Compute exact |V_k|, |V_k^{br}|, local branching entropy H_loc(G_k) (in bits),
    and (when |V_k^{br}| <= max_det_dim) exact Burnside-BEST log2 Z_k(T) (in bits)!
    """
    n_vk = 0
    n_br = 0
    h_loc_ln = 0.0
    br_intervals: list[tuple[int, int, float]] = []  # (a, b, log_factorial_ratio_ln)

    a = 0
    while a < n:
        b = a + 1
        has_eq_k = False
        while b < n and lcp[b] >= k:
            if lcp[b] == k:
                has_eq_k = True
            b += 1
        n_vk += 1
        du = b - a
        if has_eq_k:
            n_br += 1
            # Enumerate (k+1)-mer sub-intervals (split where lcp[y] == k)
            sub_a = a
            sum_ln_m_fact = 0.0
            for y in range(a + 1, b):
                if lcp[y] == k:
                    m = y - sub_a
                    sum_ln_m_fact += math.lgamma(m + 1)
                    sub_a = y
            m_last = b - sub_a
            sum_ln_m_fact += math.lgamma(m_last + 1)

            h_loc_ln += math.lgamma(du + 1) - sum_ln_m_fact
            br_intervals.append((a, b, math.lgamma(du) - sum_ln_m_fact))
        a = b

    h_loc_bits = h_loc_ln / math.log(2.0)
    if n_br == 0:
        return {
            "V_k": n_vk,
            "V_k_br": 0,
            "H_loc_bits": 0.0,
            "log2_Z_k_bits": 0.0,
            "unique_condensation": True,
        }

    if n_br > max_det_dim:
        return {
            "V_k": n_vk,
            "V_k_br": n_br,
            "H_loc_bits": round(h_loc_bits, 3),
            "log2_Z_k_bits": None,
            "unique_condensation": False,
        }

    # Build condensed branching Laplacian L_br in O(n) by marking text positions of branching k-mers!
    pos_to_br = [-1] * n
    log_ratio_total_ln = 0.0
    for bid, (ia, ib, lr_ln) in enumerate(br_intervals):
        log_ratio_total_ln += lr_ln
        for x in range(ia, ib):
            pos_to_br[sa[x]] = bid

    # Walk cyclic text positions 0..n-1 and connect consecutive branching visits!
    hit_seq = [pos_to_br[i] for i in range(n) if pos_to_br[i] >= 0]
    assert len(hit_seq) >= 2

    # First peel degree-1 / single-target vertices? At n_br <= 1600, dense float64 slogdet is < 60ms!
    if n_br == 1:
        log_det_ln = 0.0
    else:
        L_br = np.zeros((n_br, n_br), dtype=np.float64)
        m_hits = len(hit_seq)
        for idx in range(m_hits):
            u = hit_seq[idx]
            v = hit_seq[(idx + 1) % m_hits]
            L_br[u, u] += 1.0
            L_br[u, v] -= 1.0
        sign, log_det_ln = np.linalg.slogdet(L_br[:-1, :-1])
        assert sign > 0, f"Non-positive Laplacian cofactor sign={sign} at k={k}, n_br={n_br}"

    log2_z = float((log_det_ln + log_ratio_total_ln) / math.log(2.0))
    # Check exact uniqueness: since T is primitive, Z_k is an integer >= 1, so Z_k == 1 iff log2_z < 0.5!
    is_unique = bool(log2_z < 0.5)
    if is_unique and abs(log2_z) < 1e-6:
        log2_z = 0.0
    return {
        "V_k": int(n_vk),
        "V_k_br": int(n_br),
        "H_loc_bits": round(float(h_loc_bits), 3),
        "log2_Z_k_bits": round(log2_z, 4),
        "unique_condensation": is_unique,
    }


def probe_block_cycle7(L: bytes, p_mark: int) -> dict:
    n = len(L)
    T, sa, lcp, lf = build_cyclic_sa_and_lcp(L, p_mark)
    max_lcp = max(lcp)  # L_max(T): naive zero-branching threshold is max_lcp + 1!

    # 1. De Bruijn condensation curve & binary search for exact Kotzig k*(T)!
    k_grid = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192]
    db_curve = {}
    for k in k_grid:
        db_curve[str(k)] = debruijn_stats_at_k(n, sa, lcp, k)

    # Binary search exact k*(T) in [1, max_lcp + 1] where unique_condensation becomes True!
    lo = 1
    hi = max_lcp + 1
    while lo < hi:
        mid = (lo + hi) // 2
        st = debruijn_stats_at_k(n, sa, lcp, mid, max_det_dim=2500)
        if st["log2_Z_k_bits"] is None:
            lo = mid + 1
        elif st["unique_condensation"]:
            hi = mid
        else:
            lo = mid + 1
    k_star = lo
    st_kstar = debruijn_stats_at_k(n, sa, lcp, k_star, max_det_dim=3000)
    st_kstar_minus_1 = debruijn_stats_at_k(n, sa, lcp, max(1, k_star - 1), max_det_dim=3000)
    assert st_kstar["unique_condensation"] is True
    if k_star > 1:
        assert not st_kstar_minus_1["unique_condensation"], (
            f"k*-1 was also unique?! k*={k_star}, st={st_kstar_minus_1}"
        )

    # 2. Dyadic binary exponentiation LF^{2^j} for j = 0..19
    # Verify exact stream-merge algorithm at j = 0 -> 1 -> 2 on the full 900KB block!
    r0_list = iet_runs_from_dense(lf)
    r1_merge = compose_iet_stream_merge(r0_list, r0_list)
    lf2_check = lf[lf]
    assert len(r1_merge) == int(np.count_nonzero(lf2_check[1:] != lf2_check[:-1] + 1) + 1)

    f_cur = lf.copy()
    dyadic_curve = {}
    sum_rj = 0
    for j in range(20):
        rj = int(np.count_nonzero(f_cur[1:] != f_cur[:-1] + 1) + 1)
        dyadic_curve[f"2^{j}"] = {"r_pieces": rj, "frac_n": round(rj / n, 4)}
        sum_rj += rj
        f_cur = f_cur[f_cur]

    # 3. Fourier Cauchy active generator count r_cauchy on the 900KB block
    run_d = [0] + [x for x in range(1, n) if L[x] != L[x - 1]]
    r_total = len(run_d)
    r_cauchy_active = sum(
        1 for d_u in run_d if int(lf[d_u]) != (int(lf[(d_u - 1) % n]) + 1) % n
    )

    return {
        "n": n,
        "r": r_total,
        "debruijn_condensation": {
            "k_star_unique_eulerian": k_star,
            "L_max_plus_1_zero_branching": max_lcp + 1,
            "V_br_at_k_star": st_kstar["V_k_br"],
            "log2_Z_at_k_star_minus_1": st_kstar_minus_1["log2_Z_k_bits"],
            "curve": db_curve,
        },
        "dyadic_lf_squaring": {
            "sum_rj_j0_to_19": sum_rj,
            "sum_rj_over_n": round(sum_rj / n, 2),
            "curve": dyadic_curve,
        },
        "fourier_cauchy_structure": {
            "r": r_total,
            "active_cauchy_generators": r_cauchy_active,
            "perfect_cluster_cancellations": r_total - r_cauchy_active,
        },
    }


def main():
    c71 = verify_theorem_7_1_small()
    print(f"[PASS] Theorem 7.1 (Burnside-BEST & Branching Unitig Reduction) verified on {c71:,} small spectrum classes!")
    c723 = verify_theorem_7_2_and_7_3_small()
    print(
        f"[PASS] Theorems 7.2 & 7.3 (Stream-Merge IET Squaring + Symmetry + Exact Fourier Cauchy DFT) "
        f"verified on {c723['iet_stream_merge_and_symmetry_verified']} primitive BWTs!"
    )
    print(f"       Tribonacci (n=1705) dyadic r(LF^{{2^j}}): {c723['tribonacci_n1705_dyadic_r']}")

    blocks = load_manifest()
    results = {
        "small_verifications": {
            "theorem_7_1_spectrum_classes": c71,
            **c723,
        },
        "blocks": {},
    }

    for b in blocks:
        name = b["file"]
        L = (ADMITTED_DIR / name).read_bytes()
        res = probe_block_cycle7(L, b["primary_index"])
        results["blocks"][name] = res
        db = res["debruijn_condensation"]
        dy = res["dyadic_lf_squaring"]
        fc = res["fourier_cauchy_structure"]
        print(
            f"[{name}] "
            f"DeBruijn: log2(Z_1)={db['curve']['1']['log2_Z_k_bits']:,.1f}b, "
            f"k*={db['k_star_unique_eulerian']} (V_br@k*={db['V_br_at_k_star']}, Z_{{k*-1}}={db['log2_Z_at_k_star_minus_1']}b) "
            f"vs L_max+1={db['L_max_plus_1_zero_branching']} | "
            f"Dyadic r(LF^{{2^j}}): j0={dy['curve']['2^0']['r_pieces']:,}, "
            f"j1={dy['curve']['2^1']['r_pieces']:,}, j2={dy['curve']['2^2']['r_pieces']:,}, "
            f"j4={dy['curve']['2^4']['r_pieces']:,}, j8={dy['curve']['2^8']['r_pieces']:,}, "
            f"sum_j0..19={dy['sum_rj_over_n']}n | "
            f"Cauchy gen={fc['active_cauchy_generators']:,}/{fc['r']:,}"
        )

    with open(OUT_PATH, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)
    print(f"\nSaved Cycle 7 results to {OUT_PATH}")


if __name__ == "__main__":
    main()
