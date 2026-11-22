#!/usr/bin/env python3
"""Phase 1 benchmark entrypoint for 1-coiled-dragon-roundel.jpg delegating to the reusable vtracer library."""
from __future__ import annotations

import argparse
from pathlib import Path

from run_specimen import run_specimen

TRACED = Path(__file__).resolve().parent
SPEC_PATH = TRACED / "specs" / "romanesque-roundel.json"


def main() -> None:
    parser = argparse.ArgumentParser(description="Extract and trace romanesque-roundel using the generalizable vtracer pipeline.")
    parser.add_argument("--sweep", action="store_true", help="Run automated Pareto parameter sweep")
    parser.add_argument("--skip-sheets", action="store_true", help="Skip QA sheet rendering")
    args = parser.parse_args()

    res = run_specimen(SPEC_PATH, sweep=args.sweep, skip_sheets=args.skip_sheets)
    m = res["metrics"]
    print(
        f"Romanesque Roundel -> True IoU: {m['silhouette_iou_true_pct']:.2f}% | "
        f"Legacy IoU: {m['silhouette_iou_legacy_pct']:.2f}% | "
        f"Ink IoU: {m['ink_iou_pct']:.2f}% | "
        f"BF1(1u): {m['boundary_f1_1u_pct']:.2f}% | "
        f"Color RMSE: {m['masked_color_rmse']:.2f} | "
        f"Size: {m['raw_kb']:.2f} KB ({m['gzip_kb']:.2f} KB gz)"
    )


if __name__ == "__main__":
    main()
