#!/usr/bin/env python3
"""
Cycle 13 (Part 2) — Quantitative Deployment-Condition Phase Diagram
Across Block Sizes n in [10^5, 10^9] and Run Ratios rho = n / r_0 in [2, 10^4].

Methodological Tagging:
  - Hardware/scale thresholds are DEPLOYMENT CONDITIONS, NOT algorithmic escapes.
  - Calibrated directly to Cycle 4 & Cycle 12-13 silicon on Intel Core i9-10900K
    (L1d = 32 KiB, L2 = 256 KiB, L3 = 20 MiB shared, STLB = 1536 x 4 KiB = 6 MiB):
      * OpenCells (v0.5.12): 6-block i9 bar = 8.13-10.05 ms (Xeon 1T = 10.35 ms)
      * B4-Grid-Flat16 (CoreRvP16): 55.6 ns/op * 1.45 r_0 + 0.26 ns/B * n (Cycle 4 silicon)
      * S2_dt_heavy (q_core = 0):   1.587 r_0 train-track folds + SLP emit (Cycle 9-11)
      * External-Memory / Tall-Cache MF-BSE: O((r_0 / B) log_{M/B}(r_0)) cache-line I/Os
  - All 4 algorithms use IDENTICAL sequential output bandwidth (apples-to-apples!).
  - Evaluates BOTH:
      (A) Ungated Open (theoretical best case for Open at all rho >= 3.5, removing the 64r >= n sparse gate)
      (B) Shipped OpenCells (v0.5.12, which routes rho > 64 and rho < 3.5 to Segmented 12-chain LF walk).
"""

import math
import json

L1D_B = 32 * 1024
L2_B = 256 * 1024
L3_TOTAL_B = 20 * 1024 * 1024
STLB_REACH_B = 1536 * 4096
CACHE_LINE_B = 64

LAT_L1_NS = 1.2
LAT_L2_NS = 4.8
LAT_L3_NS = 12.0
LAT_DRAM_1T_NS = 65.0
LAT_DRAM_10W_NS = 98.0
TLB_PENALTY_NS = 18.0

DRAM_STREAM_1T_NS_PER_B = 0.045
DRAM_STREAM_10W_NS_PER_B = 0.30


def random_access_ns(ws_bytes: float, workers: int) -> float:
    if ws_bytes <= 0:
        return LAT_L1_NS
    l3_eff = L3_TOTAL_B / workers
    p_l1 = min(1.0, L1D_B / ws_bytes)
    p_l2 = max(0.0, min(1.0, L2_B / ws_bytes) - p_l1)
    p_l3 = max(0.0, min(1.0, l3_eff / ws_bytes) - min(1.0, L2_B / ws_bytes))
    p_dram = max(0.0, 1.0 - max(L2_B, l3_eff) / ws_bytes)
    lat_dram = LAT_DRAM_1T_NS if workers == 1 else LAT_DRAM_10W_NS
    tlb_miss = max(0.0, 1.0 - STLB_REACH_B / ws_bytes) * TLB_PENALTY_NS
    return p_l1 * LAT_L1_NS + p_l2 * LAT_L2_NS + p_l3 * LAT_L3_NS + p_dram * lat_dram + tlb_miss


def seq_emit_ns_per_byte(n: float, workers: int, base_ns: float = 0.26) -> float:
    l3_eff = L3_TOTAL_B / workers
    stream_bw = DRAM_STREAM_1T_NS_PER_B if workers == 1 else DRAM_STREAM_10W_NS_PER_B
    p_dram = max(0.0, 1.0 - l3_eff / max(1.0, n))
    return base_ns + p_dram * stream_bw


def thin_retained_rows(n: float, rho: float, model: str = "heavy_tail") -> float:
    r0 = n / rho
    if model == "heavy_tail":
        return min(0.95 * n, 1.873 * r0)
    p = 1.0 / rho
    phi = 1.0 - ((1.0 - p) ** 16) * (1.0 + 16.0 * p)
    return n * phi


def time_segmented_ms(n: float, workers: int) -> dict:
    ws_t = (4 if n <= (1 << 24) else 8) * n
    l_rand = random_access_ns(ws_t, workers)
    mlp = 5.5 if workers == 1 else 3.5
    t_build = n * (1.2 + 0.15 * l_rand)
    t_walk = n * (l_rand / mlp + seq_emit_ns_per_byte(n, workers, 0.20))
    return {"ms": (t_build + t_walk) / 1e6, "ws_mib": ws_t / (1024 * 1024), "route": "Segmented"}


def time_open_cells_ms(
    n: float, rho: float, workers: int, thin_model: str = "heavy_tail", ungated: bool = True
) -> dict:
    if rho < 3.5 or (not ungated and rho > 64.0):
        return time_segmented_ms(n, workers)

    r0 = n / rho
    m16 = thin_retained_rows(n, rho, thin_model)
    edge_bytes = 4 if n <= (1 << 20) else 8

    if rho <= 16.0:
        thick_frac = 0.0396 * (rho / 10.6)
    else:
        thick_frac = 0.06 + 0.94 * (1.0 - 16.0 / rho)
    ns = min(r0, max(1.0, r0 * thick_frac))

    ws_edges = edge_bytes * m16
    l_edges = random_access_ns(ws_edges, workers)
    ws_rv = 48.0 * ns
    l_rv = random_access_ns(ws_rv, workers)

    t_s1 = r0 * 3.5 + m16 * 0.35 + n * 0.015
    t_s2 = (3.93 * ns) * (22.2 * max(1.0, l_rv / LAT_L2_NS))
    spell_bytes = min(0.5 * n, 0.042 * n * math.sqrt(max(1.0, rho / 10.6)))
    t_s3a = (1.26 * ns) * (31.0 * max(1.0, l_rv / LAT_L2_NS)) + spell_bytes * seq_emit_ns_per_byte(
        spell_bytes, workers, 0.22
    )
    t_s4 = m16 * (0.18 * l_edges + 0.15)
    t_emit_chase = m16 * (0.45 * l_edges)
    t_emit_copy = n * seq_emit_ns_per_byte(n, workers, 0.26)
    t_emit = max(t_emit_chase, t_emit_copy) + 0.35 * min(t_emit_chase, t_emit_copy)

    return {
        "ms": (t_s1 + t_s2 + t_s3a + t_s4 + t_emit) / 1e6,
        "ws_mib": (ws_edges + ws_rv + spell_bytes) / (1024 * 1024),
        "route": "Open",
    }


def time_b4_grid_flat16_ms(n: float, rho: float, workers: int) -> dict:
    r0 = n / rho
    ws_b4 = 48.0 * r0
    l_b4 = random_access_ns(ws_b4, workers)
    op_ns = 55.6 * (0.55 + 0.45 * (l_b4 / LAT_L3_NS))
    t_contract = (1.45 * r0) * op_ns
    t_emit = n * seq_emit_ns_per_byte(n, workers, 0.26)
    return {"ms": (t_contract + t_emit) / 1e6, "ws_mib": ws_b4 / (1024 * 1024)}


def time_s2_dt_heavy_ms(n: float, rho: float, workers: int) -> dict:
    r0 = n / rho
    ws_dt = 32.0 * r0
    l_dt = random_access_ns(ws_dt, workers)
    op_ns = 44.0 * (0.50 + 0.50 * (l_dt / LAT_L3_NS))
    t_fold = (1.587 * r0) * op_ns
    t_emit = n * seq_emit_ns_per_byte(n, workers, 0.27)
    return {"ms": (t_fold + t_emit) / 1e6, "ws_mib": ws_dt / (1024 * 1024)}


def time_mf_bse_ms(n: float, rho: float, workers: int) -> dict:
    r0 = n / rho
    m_eff = L3_TOTAL_B / workers
    ws_runs = 32.0 * r0
    if ws_runs <= m_eff:
        passes = 1.0
    else:
        fanout = max(16.0, min(256.0, (m_eff / CACHE_LINE_B) ** 0.5))
        passes = 1.0 + math.ceil(math.log(ws_runs / m_eff) / math.log(fanout))
    stream_bw = DRAM_STREAM_1T_NS_PER_B if workers == 1 else DRAM_STREAM_10W_NS_PER_B
    t_contract = r0 * (75.0 + passes * (32.0 * (0.12 + stream_bw) + 18.0))
    t_emit = n * seq_emit_ns_per_byte(n, workers, 0.28)
    return {"ms": (t_contract + t_emit) / 1e6, "ws_mib": min(ws_runs, m_eff) / (1024 * 1024)}


def find_crossover_n(rho: float, workers: int, competitor_fn, thin_model: str = "heavy_tail", ungated: bool = True):
    lo, hi = 1e5, 1e10
    if competitor_fn(lo, rho, workers)["ms"] < time_open_cells_ms(lo, rho, workers, thin_model, ungated)["ms"]:
        return lo
    if competitor_fn(hi, rho, workers)["ms"] >= time_open_cells_ms(hi, rho, workers, thin_model, ungated)["ms"]:
        return None
    for _ in range(60):
        mid = math.sqrt(lo * hi)
        if competitor_fn(mid, rho, workers)["ms"] < time_open_cells_ms(mid, rho, workers, thin_model, ungated)["ms"]:
            hi = mid
        else:
            lo = mid
    return hi


def main():
    ns = [10**5, 9 * 10**5, 10**7, 10**8, 10**9]
    rhos = [2, 10, 100, 1000, 10000]
    results = {}
    for workers in [1, 10]:
        grid = []
        for n in ns:
            for rho in rhos:
                os_d = time_open_cells_ms(n, rho, workers, "heavy_tail", ungated=False)
                ou_d = time_open_cells_ms(n, rho, workers, "heavy_tail", ungated=True)
                b4_d = time_b4_grid_flat16_ms(n, rho, workers)
                dt_d = time_s2_dt_heavy_ms(n, rho, workers)
                mf_d = time_mf_bse_ms(n, rho, workers)
                bc_name, bc_ms = min(
                    [("B4-Flat16", b4_d["ms"]), ("S2_dt_hvy", dt_d["ms"]), ("MF-BSE", mf_d["ms"])],
                    key=lambda x: x[1],
                )
                grid.append({
                    "n": n, "rho": rho, "r0": int(n / rho),
                    "open_ship_ms": round(os_d["ms"], 4), "open_ung_ms": round(ou_d["ms"], 4),
                    "b4_ms": round(b4_d["ms"], 4), "dt_ms": round(dt_d["ms"], 4), "mfbse_ms": round(mf_d["ms"], 4),
                    "best_comp": bc_name,
                    "speedup_vs_ung": round(ou_d["ms"] / bc_ms, 3),
                    "speedup_vs_ship": round(os_d["ms"] / bc_ms, 3),
                })
        crossovers = {}
        for rho in [2, 5, 10, 15, 20, 50, 100, 1000, 10000]:
            crossovers[str(rho)] = {
                "b4": find_crossover_n(rho, workers, time_b4_grid_flat16_ms, "heavy_tail", True),
                "dt": find_crossover_n(rho, workers, time_s2_dt_heavy_ms, "heavy_tail", True),
                "mfbse": find_crossover_n(rho, workers, time_mf_bse_ms, "heavy_tail", True),
                "b4_geom": find_crossover_n(rho, workers, time_b4_grid_flat16_ms, "geometric", True),
                "dt_vs_ship": find_crossover_n(rho, workers, time_s2_dt_heavy_ms, "heavy_tail", False),
            }
        results[f"{workers}w"] = {"grid": grid, "crossovers": crossovers}
    with open("/tmp/fcp-b/cycle13_phase_diagram.json", "w") as f:
        json.dump(results, f, indent=2)
    print("OK: wrote /tmp/fcp-b/cycle13_phase_diagram.json")


if __name__ == "__main__":
    main()
