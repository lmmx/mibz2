#!/usr/bin/env python3
"""Pre-port test 1 (Deep Dive #1): Deterministic Network Calculus via stock Panco
on character-stage1 traces at k in {1, 6, 8, 10} (plus full sweep k=1..10 for knee).

Extracts all parameters directly from:
  docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/
and exports extracted parameters to JSON so nc_nancy/Program.cs runs on identical inputs.

No production code is touched. Uses stock Panco (BSD-3) with lp_solve.
"""

from __future__ import annotations

import csv
import json
import math
import os
import statistics as st
import sys
from collections import defaultdict
from pathlib import Path

from panco.descriptor.curves import RateLatency, TokenBucket
from panco.descriptor.flow import Flow
from panco.descriptor.network import Network
from panco.descriptor.server import Server
from panco.fifo.fifoLP import FifoLP
from panco.fifo.sfaLP import SfaLP
from panco.fifo.tfaLP import TfaLP


REPO_ROOT = Path(__file__).resolve().parents[5]
DATA_DIR = (
    REPO_ROOT
    / "docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1"
)
OUT_DIR = Path(__file__).resolve().parent

N_BLOCKS = 1724
N_OPEN = 932       # 912 open + 20 open_dense
N_OPEN_STD = 912
N_OPEN_DENSE = 20
N_TINY = 792       # 600 skipped + 192 column
ROWS_OPEN = 900_000

KS_GATE = [1, 6, 8, 10]
KS_SWEEP = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open() as f:
        return list(csv.DictReader(f, delimiter="\t"))


def load_stage1() -> dict:
    obs_rows = [
        r
        for r in read_tsv(DATA_DIR / "observations.tsv")
        if r["section"] == "grid" and r.get("on", "1") in ("1", "True", "true")
    ]
    obs_by_k: dict[int, list[dict[str, str]]] = defaultdict(list)
    for r in obs_rows:
        obs_by_k[int(r["k"])].append(r)

    burst_rows = read_tsv(DATA_DIR / "analysis" / "burst_frontier.tsv")
    # (k, event) -> list of (rate_factor, burst_med, burst_max)
    bursts: dict[tuple[int, str], list[tuple[float, float, float]]] = defaultdict(list)
    for r in burst_rows:
        bursts[(int(r["k"]), r["event"])].append(
            (float(r["rate_factor"]), float(r["burst_median"]), float(r["burst_max"]))
        )
    for key in bursts:
        bursts[key].sort()

    svc_rows = read_tsv(DATA_DIR / "service.tsv")
    svc: dict[tuple[int, str, str], list[tuple[float, float, float]]] = defaultdict(list)
    for r in svc_rows:
        svc[(int(r["k"]), r["role"], r["activity"])].append(
            (float(r["visits"]), float(r["mean_ns"]), float(r["c2"]))
        )

    cont_by_k = {int(r["k"]): r for r in read_tsv(DATA_DIR / "contention.tsv")}
    prog_by_k = {int(r["k"]): r for r in read_tsv(DATA_DIR / "analysis" / "progress.tsv")}

    extracted: dict[int, dict] = {}
    for k in KS_SWEEP:
        rows = obs_by_k[k]
        walls = [float(r["wall"]) for r in rows]
        s_busy = [float(r["scanner_busy"]) for r in rows]
        w_busy = [float(r["worker_busy_mean"]) for r in rows]
        d_act = [float(r["reader_active"]) for r in rows]
        startup = [float(r["startup"]) for r in rows]
        drain = [float(r["drain"]) for r in rows]
        hwm_adm = [float(r["hwm_admitted"]) for r in rows]
        hwm_rdy = [float(r["hwm_ready"]) for r in rows]
        w_c = [
            float(r["worker_A6a"]) + float(r["worker_A6b"]) + float(r["worker_A6c"])
            for r in rows
        ]
        # Total worker busy across all k workers is k * worker_busy_mean
        c_share_of_w = st.median(c / (k * wb) for c, wb in zip(w_c, w_busy))

        # Per-block open vs tiny service times from service.tsv
        def svc_med(role: str, act: str) -> tuple[float, float, float]:
            vals = svc[(k, role, act)]
            return (
                st.median(x[0] for x in vals),
                st.median(x[1] for x in vals) * 1e-9,  # seconds
                st.median(x[2] for x in vals),
            )

        _, s_a6a, _ = svc_med("Worker", "A6a")
        _, s_a6b, _ = svc_med("Worker", "A6b")
        _, s_a6c, _ = svc_med("Worker", "A6c")
        _, s_a7, _ = svc_med("Worker", "A7")
        _, s_a8, _ = svc_med("Worker", "A8")
        v_a5, s_a5, _ = svc_med("Worker", "A5")
        v_a9, s_a9, _ = svc_med("Worker", "A9")
        # Open blocks hold >99.99% of bytes, so nearly all A5 and A9 work is on the 932 Open blocks
        s_open_mean_worker = (s_a6a + s_a6b + s_a6c + s_a7) + (v_a5 * s_a5 + v_a9 * s_a9) / N_OPEN
        # From report.txt lines 361-362: open_dense takes 2.7 ns/row at k=1 (2.43 ms), 3.4 ns/row at k=10 (3.06 ms)
        # versus open at 1.3 ns/row (k=1) and 1.8 ns/row (k=10): dense/open ratio ~2.08 at k=1, ~1.89 at k=10.
        # Scale exact report.txt k=1 open_dense (2.43 ms) by worker contention factor w_busy_med*k / w_busy_med(1)
        extracted[k] = {
            "k": k,
            "n_obs": len(rows),
            "wall_med": st.median(walls),
            "wall_max": max(walls),
            "wall_min": min(walls),
            "S_med": st.median(s_busy),
            "S_max": max(s_busy),
            "W_med": st.median(w_busy),
            "W_max": max(w_busy),
            "D_med": st.median(d_act),
            "D_max": max(d_act),
            "startup_med": st.median(startup),
            "drain_med": st.median(drain),
            "hwm_admitted_max": max(hwm_adm),
            "hwm_ready_max": max(hwm_rdy),
            "C_share_of_worker": c_share_of_w,
            "s_open_mean_worker": s_open_mean_worker,
            "s_tiny_mean_worker": s_a8 + 1e-6,  # A8 (~0.5-1 us) + tiny A5/A9 (~0.5 us)
            "hol_age_ms": float(cont_by_k[k]["hol_age_ms"]),
            "hol_wait_p99_ms": float(prog_by_k[k]["hol_wait_ms_p99"]),
            "bursts_issued": bursts[(k, "t_issued")],
            "bursts_ready": bursts[(k, "t_ready")],
            "bursts_taken": bursts[(k, "t_taken")],
            "bursts_consumed": bursts[(k, "t_consumed")],
        }

    # Add open_dense per-block duration scaled by measured worker inflation (matches report.txt k=1 2.43ms and k=10 3.06ms)
    base_w_total = extracted[1]["W_med"]
    for k in KS_SWEEP:
        inflation = (k * extracted[k]["W_med"]) / base_w_total
        # report.txt: open_dense at k=1 is 2.7 ns/row * 900_000 = 2.43 ms; at k=10 is 3.4 ns/row * 900_000 = 3.06 ms
        # Note inflation(10) = 1.337 -> 2.43 * 1.337 = 3.25 ms (slightly conservative vs 3.06 ms).
        # Use exact linear interpolation on inflation matching 2.43 ms at k=1 and 3.06 ms at k=10, multiplied by max/med round jitter.
        s_dense_med = 0.00243 * (1.0 + (3.06 / 2.43 - 1.0) * (inflation - 1.0) / ((10 * extracted[10]["W_med"]) / base_w_total - 1.0))
        round_jitter = extracted[k]["W_max"] / extracted[k]["W_med"]
        extracted[k]["s_open_dense_med"] = s_dense_med
        extracted[k]["s_open_dense_max"] = s_dense_med * round_jitter
        extracted[k]["s_open_scan_max"] = extracted[k]["S_max"] / N_OPEN
        extracted[k]["s_open_read_max"] = extracted[k]["D_max"] / N_OPEN

    return extracted


def run_panco_tandem(
    arrival_tbs: list[TokenBucket],
    servers_rl: list[tuple[float, float]],
    tag: str,
) -> dict[str, float]:
    """Run Panco FifoLP (PLP), SfaLP, and TfaLP on a 1-flow tandem of rate-latency servers."""
    servers = [Server([RateLatency(r, t)], []) for r, t in servers_rl]
    path = list(range(len(servers)))
    flows = [Flow(arrival_tbs, path)]
    net = Network(servers, flows)

    lp_dir = Path("/tmp/panco_lp")
    lp_dir.mkdir(exist_ok=True)
    res = {}
    for name, cls, kwargs in [
        ("plp", FifoLP, {"polynomial": True, "filename": str(lp_dir / f"{tag}_plp.lp")}),
        ("sfa", SfaLP, {"filename": str(lp_dir / f"{tag}_sfa.lp")}),
        ("tfa", TfaLP, {"filename": str(lp_dir / f"{tag}_tfa.lp")}),
    ]:
        try:
            d = float(cls(net, **kwargs).delay(0))
        except Exception as exc:
            d = float("nan")
            print(f"  [warn] Panco {name} failed on {tag}: {exc}", file=sys.stderr)
        res[name] = d
    return res


def run_panco_two_flow_fifo(
    tb_open: list[TokenBucket],
    tb_tiny: list[TokenBucket],
    servers_rl: list[tuple[float, float]],
    tag: str,
) -> dict[str, float]:
    """Two-flow FIFO tandem in Panco (Flow 0 = Open blocks, Flow 1 = Tiny blocks sharing W_k -> D)."""
    servers = [Server([RateLatency(r, t)], []) for r, t in servers_rl]
    path = list(range(len(servers)))
    flows = [Flow(tb_open, path), Flow(tb_tiny, path)]
    net = Network(servers, flows)

    lp_dir = Path("/tmp/panco_lp")
    lp_dir.mkdir(exist_ok=True)
    out = {}
    for name, cls, kwargs in [
        ("plp", FifoLP, {"polynomial": True, "filename": str(lp_dir / f"{tag}_2f_plp.lp")}),
        ("sfa", SfaLP, {"filename": str(lp_dir / f"{tag}_2f_sfa.lp")}),
        ("tfa", TfaLP, {"filename": str(lp_dir / f"{tag}_2f_tfa.lp")}),
    ]:
        try:
            solver = cls(net, **kwargs)
            d0 = float(solver.delay(0))
            d1 = float(solver.delay(1))
            out[f"{name}_open"] = d0
            out[f"{name}_tiny"] = d1
        except Exception as exc:
            out[f"{name}_open"] = float("nan")
            out[f"{name}_tiny"] = float("nan")
            print(f"  [warn] Panco 2-flow {name} failed on {tag}: {exc}", file=sys.stderr)
    return out


def build_models(ext: dict[int, dict]) -> dict:
    """Build four mathematically distinct deterministic NC curve calibrations for each k."""
    models: dict[str, dict[int, dict]] = {
        "M1_strict_block_worst": {},
        "M2_trace_envelope_block": {},
        "M3a_open_strict_worst": {},
        "M3b_open_mean_hol": {},
    }

    for k in KS_SWEEP:
        d = ext[k]
        # --- Model 1: Strict worst-case per-block (adversarial block order, N = 1724 blocks) ---
        # Any subsequence of blocks can consist of open_dense blocks:
        #   R_S = 1 / s_open_scan_max, T_S = s_open_scan_max
        #   R_W = k / s_open_dense_max, T_W = s_open_dense_max
        #   R_D = 1 / s_open_read_max, T_D = s_open_read_max
        models["M1_strict_block_worst"][k] = {
            "N": N_BLOCKS,
            "W_4k": 4 * k,
            "W_2k": 2 * k,
            "R_S": 1.0 / d["s_open_scan_max"],
            "T_S": d["s_open_scan_max"],
            "R_W": k / d["s_open_dense_max"],
            "T_W": d["s_open_dense_max"],
            "R_D": 1.0 / d["s_open_read_max"],
            "T_D": d["s_open_read_max"],
            "delta_tiny": 0.0,
        }

        # --- Model 2: Empirical trace-calibrated block-unit envelope (N = 1724 blocks) ---
        # Rates from worst-case (max) observed round in observations.tsv;
        # Latencies from burst_frontier.tsv burst_max(f=1.0) / R_stage (the exact strict
        # rate-latency envelope that stays below the empirical departure curve on every subinterval).
        R_S2 = N_BLOCKS / d["S_max"]
        R_W2 = N_BLOCKS / d["W_max"]
        R_D2 = N_BLOCKS / d["D_max"]
        b_issued_1 = dict((f, bm) for f, _, bm in d["bursts_issued"])[1.0]
        b_ready_1 = dict((f, bm) for f, _, bm in d["bursts_ready"])[1.0]
        b_cons_1 = dict((f, bm) for f, _, bm in d["bursts_consumed"])[1.0]
        T_S2 = b_issued_1 / R_S2
        T_W2 = b_ready_1 / R_W2
        # Differential reader burst + p99 HOL wait
        T_D2 = max(0.0, b_cons_1 - b_ready_1) / R_D2 + d["hol_wait_p99_ms"] * 1e-3
        models["M2_trace_envelope_block"][k] = {
            "N": N_BLOCKS,
            "W_4k": 4 * k,
            "W_2k": 2 * k,
            "R_S": R_S2,
            "T_S": T_S2,
            "R_W": R_W2,
            "T_W": T_W2,
            "R_D": R_D2,
            "T_D": T_D2,
            "delta_tiny": 0.0,
            "b_issued_1": b_issued_1,
            "b_ready_1": b_ready_1,
            "b_cons_1": b_cons_1,
        }

        # --- Model 3a: Open-block work-normalized, strict worst-case (N_open = 932 full blocks) ---
        # Separates the 792 tiny blocks (~0.4-0.8 ms total) as an exact additive overhead,
        # and bounds the 932 full 900 KB blocks using worst-case open_dense per-block duration.
        delta_tiny = N_TINY * d["s_tiny_mean_worker"] / k
        models["M3a_open_strict_worst"][k] = {
            "N": N_OPEN,
            "W_4k": 4 * k,
            "W_2k": 2 * k,
            "R_S": 1.0 / d["s_open_scan_max"],
            "T_S": d["s_open_scan_max"],
            "R_W": k / d["s_open_dense_max"],
            "T_W": d["s_open_dense_max"],
            "R_D": 1.0 / d["s_open_read_max"],
            "T_D": d["s_open_read_max"],
            "delta_tiny": delta_tiny,
        }

        # --- Model 3b: Open-block work-normalized, mean rate + single-block/HOL latency (N_open = 932) ---
        # NOTE: Empirical/optimistic rate (mean full-block rate over the worst observed round),
        # with latency T_W = single open_dense block (non-preemptive residual + HOL straggler).
        # Not an adversarial-order certificate (20 open_dense blocks could cluster), tested to see
        # whether NC works even under generous corpus-mean rates.
        R_S3b = N_OPEN / d["S_max"]
        R_W3b = N_OPEN / d["W_max"]
        R_D3b = N_OPEN / d["D_max"]
        models["M3b_open_mean_hol"][k] = {
            "N": N_OPEN,
            "W_4k": 4 * k,
            "W_2k": 2 * k,
            "R_S": R_S3b,
            "T_S": d["s_open_scan_max"],
            "R_W": R_W3b,
            "T_W": d["s_open_dense_max"],
            "R_D": R_D3b,
            "T_D": d["s_open_read_max"] + d["hol_wait_p99_ms"] * 1e-3,
            "delta_tiny": delta_tiny,
        }

    return models


def main() -> None:
    ext = load_stage1()
    models = build_models(ext)

    # Run Panco FIFO PLP / SFA / TFA on the downstream tandem (W_k -> D) fed by empirical
    # arrival curve alpha_S(t) from burst_frontier.tsv (t_issued), AND full 3-stage tandem.
    panco_results: dict[str, dict[int, dict]] = {
        "P1_downstream_empirical_alpha": {},
        "P2_downstream_two_flow_fifo": {},
        "P3_policy_8w_SC_vs_C": {},
    }

    print("=== Panco FIFO PLP / SFA / TFA on character-stage1 (downstream W_k -> D) ===")
    print(
        f"{'k':>2} {'T_wall':>7} {'S_max':>7} | "
        f"{'d_PLP':>8} {'d_SFA':>8} {'d_TFA':>8} | "
        f"{'U_PLP':>7} {'U/wall':>6} | "
        f"{'d_PLP_strict':>12} {'U_strict/w':>10}"
    )

    for k in KS_SWEEP:
        d = ext[k]
        # Empirical arrival curve at worker input from burst_frontier.tsv (t_issued):
        # Mean issue rate in the trace is (N_BLOCKS - 1) / span ~= N_BLOCKS / wall_med.
        # For a strictly valid arrival envelope into W_k -> D when scanner is unconstrained,
        # the rate is capped by R_S = N_BLOCKS / S_med (or mean_rate = N_BLOCKS / wall_med for observed trace).
        mean_rate_trace = N_BLOCKS / d["wall_med"]
        # Use all 5 token-bucket points from burst_frontier.tsv (t_issued, burst_max)
        # Note: in Panco, stability of the LP requires rho < min(R_W, R_D).
        # At k=1..6, mean_rate_trace = N/wall_med < N/W_med only marginally; when W_max > wall_med,
        # rho = 1.0 * mean_rate_trace would exceed R_W2 = N/W_max (unstable in infinite-horizon LP!).
        # That itself is a key NC fact: we test both:
        #  (a) trace median service rates R_W_med = N/W_med, R_D_med = N/D_med with rho = 0.99 * min(R_S_obs, R_W_med)
        #      or exact observed mean_rate_trace (scaled by 0.999 if needed for LP feasibility), and
        #  (b) strict rate-latency with open_dense rate (infeasible when arrival rate > strict worst-case rate!).
        R_W_med = N_BLOCKS / d["W_med"]
        R_D_med = N_BLOCKS / d["D_med"]
        T_W_open = d["s_open_dense_max"]
        T_D_hol = d["s_open_read_max"] + d["hol_wait_p99_ms"] * 1e-3

        # Build concave token-bucket envelope from burst_frontier.tsv (t_issued)
        # Note: only buckets with rho < R_W_med keep the steady-state queue stable;
        # we keep the tightest stable bucket plus higher-rate peak-shaping buckets (valid in Panco!).
        tbs_emp = [
            TokenBucket(sigma=b_max, rho=min(f * mean_rate_trace, 0.995 * R_W_med) if f == 1.0 else f * mean_rate_trace)
            for f, _, b_max in d["bursts_issued"]
        ]
        # In Panco, FifoLP checks sum(rho) < R for each server using the MINIMUM rho of the flow's arrival curve
        # (first or min rho bucket). Let's verify using all 5 buckets.
        res_emp = run_panco_tandem(
            tbs_emp,
            [(R_W_med, T_W_open), (R_D_med, T_D_hol)],
            f"emp_k{k}",
        )
        # Also test with full trace-burst latency T_W2 = b_ready(1.0) / R_W_med
        T_W_burst = models["M2_trace_envelope_block"][k]["T_W"]
        T_D_burst = models["M2_trace_envelope_block"][k]["T_D"]
        res_burst = run_panco_tandem(
            tbs_emp,
            [(R_W_med, T_W_burst), (R_D_med, T_D_burst)],
            f"burst_k{k}",
        )

        # Completion time upper bound from arrival-envelope + Panco delay:
        # Last block enters W_k by T_issue_last <= max(S_max, wall_last_issue) -> in open-loop
        # without window feedback, last block enters by N/rho_min (or S_max if scanner-limited).
        # Specifically: U_Panco = (N_BLOCKS / rho_0) + d_PLP? No: if alpha(t) = min(sigma_i + rho_i t)
        # is an arrival curve, it is an UPPER bound on arrivals, NOT a lower bound on arrival time!
        # The last block finishes scanning by T_scan_last <= S_max (if never blocked) or in general
        # for a 3-stage tandem S -> W_k -> D fed by bursty N_BLOCKS at t=0:
        #   U_PLP(k) = max(S_max, W_max) + d_PLP  (lower-envelope of arrivals + delay bound)
        # Wait: for a finite batch of N blocks arriving at rate <= rho_0 = min_i(rho_i) after scanner S,
        # the last block exits S at most at T_S_last = N/R_S + T_S (when unblocked) or N/rho_eff,
        # and then spends at most d_PLP in W_k -> D!
        T_last_in = max(d["S_max"], d["W_max"])  # actual max-round arrival span to downstream
        U_plp_emp = T_last_in + res_emp["plp"]
        U_plp_burst = T_last_in + res_burst["plp"]

        panco_results["P1_downstream_empirical_alpha"][k] = {
            "wall_med": d["wall_med"],
            "S_max": d["S_max"],
            "W_max": d["W_max"],
            "d_plp_single_blk_lat": res_emp["plp"],
            "d_sfa_single_blk_lat": res_emp["sfa"],
            "d_tfa_single_blk_lat": res_emp["tfa"],
            "U_plp_single_blk_lat": U_plp_emp,
            "ratio_single_blk_lat": U_plp_emp / d["wall_med"],
            "d_plp_burst_lat": res_burst["plp"],
            "d_sfa_burst_lat": res_burst["sfa"],
            "d_tfa_burst_lat": res_burst["tfa"],
            "U_plp_burst_lat": U_plp_burst,
            "ratio_burst_lat": U_plp_burst / d["wall_med"],
        }
        print(
            f"{k:2d} {d['wall_med']:7.4f} {d['S_max']:7.4f} | "
            f"{res_emp['plp']*1e3:6.2f}ms {res_emp['sfa']*1e3:6.2f}ms {res_emp['tfa']*1e3:6.2f}ms | "
            f"{U_plp_emp:7.4f} {U_plp_emp/d['wall_med']:6.3f} | "
            f"{res_burst['plp']*1e3:10.2f}ms {U_plp_burst/d['wall_med']:10.3f}"
        )

        # Two-flow FIFO in Panco (Flow 0 = 932 Open blocks, Flow 1 = 792 Tiny blocks sharing W_k -> D)
        # Rate split proportional to block counts (54.06% Open, 45.94% Tiny), burst split from burst_frontier
        f_open = N_OPEN / N_BLOCKS
        f_tiny = N_TINY / N_BLOCKS
        tb_open = [
            TokenBucket(sigma=max(1.0, b_max * f_open), rho=0.99 * f_open * min(mean_rate_trace, R_W_med) if f == 1.0 else f * f_open * mean_rate_trace)
            for f, _, b_max in d["bursts_issued"]
        ]
        tb_tiny = [
            TokenBucket(sigma=max(1.0, b_max * f_tiny), rho=0.99 * f_tiny * min(mean_rate_trace, R_W_med) if f == 1.0 else f * f_tiny * mean_rate_trace)
            for f, _, b_max in d["bursts_issued"]
        ]
        res_2f = run_panco_two_flow_fifo(
            tb_open,
            tb_tiny,
            [(R_W_med, T_W_open), (R_D_med, T_D_hol)],
            f"2f_k{k}",
        )
        panco_results["P2_downstream_two_flow_fifo"][k] = res_2f

    # Policy separation in Panco at k = 8: baseline vs C-only (-45% worker time) vs S+C (-30% S, -45% worker)
    d8 = ext[8]
    c_share8 = d8["C_share_of_worker"]
    mean_rate8 = N_BLOCKS / d8["wall_med"]
    R_W8 = N_BLOCKS / d8["W_med"]
    R_D8 = N_BLOCKS / d8["D_med"]
    T_W8 = d8["s_open_dense_max"]
    T_D8 = d8["s_open_read_max"] + d8["hol_wait_p99_ms"] * 1e-3

    # Ideal-C removes C_share_of_worker (~45% of worker work); Ideal-S+C removes 85% of S + C
    # Also test a realistic -30% C vs (-30% S and -30% C)
    for pname, s_factor, c_cut in [
        ("baseline", 1.0, 0.0),
        ("C_minus_30pct", 1.0, 0.30),
        ("SC_minus_30pct", 0.70, 0.30),
        ("C_ideal", 1.0, 1.0),
        ("SC_ideal", 0.10, 1.0),
    ]:
        w_factor = 1.0 - c_cut * c_share8
        S_pol = d8["S_max"] * s_factor
        W_pol = d8["W_max"] * w_factor
        R_W_pol = N_BLOCKS / (d8["W_med"] * w_factor)
        T_W_pol = T_W8 * w_factor
        rho_pol = min(N_BLOCKS / (d8["S_med"] * s_factor), 0.995 * R_W_pol)
        tbs_pol = [
            TokenBucket(sigma=b_max, rho=rho_pol if f == 1.0 else f * rho_pol)
            for f, _, b_max in d8["bursts_issued"]
        ]
        r_pol = run_panco_tandem(tbs_pol, [(R_W_pol, T_W_pol), (R_D8, T_D8)], f"pol_{pname}")
        panco_results["P3_policy_8w_SC_vs_C"][pname] = {
            "S_pol": S_pol,
            "W_pol": W_pol,
            "d_plp": r_pol["plp"],
            "U_plp": max(S_pol, W_pol) + r_pol["plp"],
        }

    print("\n=== Panco policy separation at k=8 ===")
    for pname, v in panco_results["P3_policy_8w_SC_vs_C"].items():
        print(f"  {pname:16s}: S={v['S_pol']:.4f} W={v['W_pol']:.4f} d_PLP={v['d_plp']*1e3:6.2f}ms U={v['U_plp']:.4f}s")

    payload = {
        "extracted": ext,
        "models": models,
        "panco": panco_results,
    }
    out_json = OUT_DIR / "nc_stage1_params.json"
    with out_json.open("w") as f:
        json.dump(payload, f, indent=2)
    print(f"\nWrote {out_json}")


if __name__ == "__main__":
    main()
