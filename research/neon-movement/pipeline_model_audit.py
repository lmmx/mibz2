#!/usr/bin/env python3
"""Audit of wall(k) = c + max(b, a/k) against the four-arm measurements at commit a3f7d9ed
(docs/journal/2026-11-24-algorithm-level/math-targets/data/measurements-four-arm.tsv)
for channel #31/#32 (10-core/20-thread i9-10900K, 1-20 worker grid).

Derives:
  - data/pipeline_model_four_arm_audit.tsv
"""

from __future__ import annotations

import csv
import math
import os
import subprocess
from typing import Any

HERE = os.path.dirname(os.path.abspath(__file__))


def load_four_arm_tsv() -> list[dict[str, str]]:
    raw = subprocess.check_output(
        [
            "git",
            "show",
            "a3f7d9ed:docs/journal/2026-11-24-algorithm-level/math-targets/data/measurements-four-arm.tsv",
        ],
        text=True,
    )
    return list(csv.DictReader(raw.splitlines(), delimiter="\t"))


def main() -> None:
    rows = load_four_arm_tsv()
    wall: dict[tuple[int, str], float] = {}
    ci_half: dict[tuple[int, str], float] = {}
    for r in rows:
        k = int(r["workers"])
        arm = r["arm"]
        wall[(k, arm)] = float(r["wall_median_s"])
        if r["ratio_ci_lo"] and r["ratio_ci_hi"]:
            ci_half[(k, arm)] = (float(r["ratio_ci_hi"]) - float(r["ratio_ci_lo"])) / 2.0

    # Test linear worker model W(k) = c + a/k fitted to Arm C (no scanner) at k=1 and k=6,
    # and check its prediction at k=10 (still <= 10 physical cores, before SMT!)
    wC1, wC6, wC10 = wall[(1, "C")], wall[(6, "C")], wall[(10, "C")]
    a_C = (wC1 - wC6) / (1.0 - 1.0 / 6.0)
    c_C = wC1 - a_C
    pred_C10 = c_C + a_C / 10.0
    err_C10_pct = (wC10 - pred_C10) / pred_C10 * 100.0

    # Same test on Arm D (no scanner + supplied codes)
    wD1, wD6, wD10 = wall[(1, "D")], wall[(6, "D")], wall[(10, "D")]
    a_D = (wD1 - wD6) / (1.0 - 1.0 / 6.0)
    c_D = wD1 - a_D
    pred_D10 = c_D + a_D / 10.0
    err_D10_pct = (wD10 - pred_D10) / pred_D10 * 100.0

    # Estimate per-worker throughput scaling efficiency E(k) = wC1 / (k * wCk)
    # and implied USL contention parameter alpha from (k=1, 6, 10):
    # W_USL(k) = W1 * (1 + alpha*(k-1)) / k  =>  alpha = (k*Wk/W1 - 1) / (k - 1)
    alpha_C6 = (6.0 * wC6 / wC1 - 1.0) / 5.0
    alpha_C10 = (10.0 * wC10 / wC1 - 1.0) / 9.0
    # Under USL (Gunther): W(k) = W1 * (1 + alpha*(k-1) + beta*k*(k-1)) / k
    # Solve exact (alpha, beta) for Arm C through k=1, 6, 10:
    # Let y(k) = k * W(k) / W1 - 1 = alpha*(k-1) + beta*k*(k-1)
    # y(6)/5 = alpha + 6*beta; y(10)/9 = alpha + 10*beta
    beta_C = (alpha_C10 - alpha_C6) / 4.0
    alpha_C_usl = alpha_C6 - 6.0 * beta_C

    # Implied scanner floor b from Arm A at k=10 (noting scanner+10w = 11 threads -> 1 SMT pair!)
    wA1, wA6, wA10 = wall[(1, "A")], wall[(6, "A")], wall[(10, "A")]
    # Naive c + max(b, a/k) crossover k* = a / b using Arm C's (a_C, c_C) and b = wA10 - c_C
    b_naive = wA10 - c_C
    k_star_naive = a_C / b_naive

    # USL crossing: find continuous k where W_USL(k) == wA10 (upper bound since wA10 includes SMT!)
    # and where W_USL(k) == wA10 - 0.015 (if SMT adds ~7% to scanner at k=10)
    def w_usl_C(k: float) -> float:
        return wC1 * (1.0 + alpha_C_usl * (k - 1.0) + beta_C * k * (k - 1.0)) / k

    # Relative slope |d ln W / dk| near k=8..9
    slope_8_9 = abs(math.log(w_usl_C(9.0)) - math.log(w_usl_C(8.0)))
    # From 20-round 95% CI half-width at k=10 (C/A: +/- 0.0245 -> per-round paired SD sigma ~ 0.0245*sqrt(20)/1.96)
    sigma_6 = ci_half[(6, "C")] * math.sqrt(20) / 1.96
    sigma_10 = ci_half[(10, "C")] * math.sqrt(20) / 1.96
    # Rounds needed to pin k* to +/- 1 worker (95% CI half-width <= 0.5 * slope_8_9)
    target_hw = 0.5 * slope_8_9
    rounds_at_sig6 = math.ceil((1.96 * sigma_6 / target_hw) ** 2)
    rounds_at_sig10 = math.ceil((1.96 * sigma_10 / target_hw) ** 2)

    out_rows: list[dict[str, Any]] = [
        {
            "arm": "C_workers_only",
            "w_k1_s": f"{wC1:.4f}",
            "w_k6_s": f"{wC6:.4f}",
            "w_k10_s": f"{wC10:.4f}",
            "lin_a_s": f"{a_C:.4f}",
            "lin_c_s": f"{c_C:.4f}",
            "lin_pred_k10_s": f"{pred_C10:.4f}",
            "lin_err_k10_pct": f"{err_C10_pct:+.2f}",
            "eff_k6": f"{wC1 / (6 * wC6):.4f}",
            "eff_k10": f"{wC1 / (10 * wC10):.4f}",
            "usl_alpha": f"{alpha_C_usl:.5f}",
            "usl_beta": f"{beta_C:.5f}",
            "slope_lnW_k8_9": f"{slope_8_9:.4f}",
            "naive_k_star": f"{k_star_naive:.2f}",
            "sigma_k6": f"{sigma_6:.4f}",
            "sigma_k10": f"{sigma_10:.4f}",
            "rounds_for_pm1_at_sig6": rounds_at_sig6,
            "rounds_for_pm1_at_sig10": rounds_at_sig10,
        },
        {
            "arm": "D_both_supplied",
            "w_k1_s": f"{wD1:.4f}",
            "w_k6_s": f"{wD6:.4f}",
            "w_k10_s": f"{wD10:.4f}",
            "lin_a_s": f"{a_D:.4f}",
            "lin_c_s": f"{c_D:.4f}",
            "lin_pred_k10_s": f"{pred_D10:.4f}",
            "lin_err_k10_pct": f"{err_D10_pct:+.2f}",
            "eff_k6": f"{wD1 / (6 * wD6):.4f}",
            "eff_k10": f"{wD1 / (10 * wD10):.4f}",
            "usl_alpha": "",
            "usl_beta": "",
            "slope_lnW_k8_9": "",
            "naive_k_star": "",
            "sigma_k6": "",
            "sigma_k10": "",
            "rounds_for_pm1_at_sig6": "",
            "rounds_for_pm1_at_sig10": "",
        },
    ]

    out_dir = os.path.join(HERE, "data")
    os.makedirs(out_dir, exist_ok=True)
    out_path = os.path.join(out_dir, "pipeline_model_four_arm_audit.tsv")
    with open(out_path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(out_rows[0].keys()), delimiter="\t")
        w.writeheader()
        w.writerows(out_rows)

    print(f"Arm C linear (1,6)->10: pred={pred_C10:.4f}s vs actual={wC10:.4f}s ({err_C10_pct:+.2f}%)")
    print(f"Arm D linear (1,6)->10: pred={pred_D10:.4f}s vs actual={wD10:.4f}s ({err_D10_pct:+.2f}%)")
    print(f"Arm C USL: alpha={alpha_C_usl:.5f}, beta={beta_C:.5f}, slope(8->9)={slope_8_9*100:.2f}%/worker")
    print(f"Naive c+max(b,a/k) k* = {k_star_naive:.2f}; rounds for +/-1w: {rounds_at_sig6} (at k=6 sig) .. {rounds_at_sig10} (at k=10 sig)")
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
