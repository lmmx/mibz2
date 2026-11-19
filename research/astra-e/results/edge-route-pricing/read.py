"""Splice the changed-block measurements into a frozen proxy baseline and report both levels."""
import argparse
import copy
import json
from pathlib import Path
import sys


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("checkout", type=Path)
    p.add_argument("rows", type=Path)
    args = p.parse_args()
    root = args.checkout.resolve()
    sys.path.insert(0, str(root / "bench/moonshot"))
    import moonshot
    cal = json.loads(moonshot.CALIBRATION.read_text())
    s = root / "research/astra-e/results/combined-v1-proxy/score"
    runs, lats = moonshot.load_runs(s / "proxy-runs.jsonl")
    canonical = json.loads((s / "proxy.json").read_text())
    sample = moonshot.proxy_sample(cal, "48+16")
    old = moonshot.read_proxy(runs, sample, cal, reps=0)
    arms = ("edge", "combined")
    contrast = {a: copy.deepcopy(runs) for a in arms}
    rows = [json.loads(line) for line in args.rows.read_text().splitlines()]
    expected = {(w, a, "base", 165) for a in arms for w in (7, 8, 10)}
    assert len(rows) == len(expected)
    assert {(r["workers"],r["arm"],r["kind"],r["block"]) for r in rows} == expected
    blocks = {}
    for row in rows:
        w, arm, b, kind = str(row["workers"]), row["arm"], row["block"], row["kind"]
        assert row["latencies"] == lats[w]
        contrast[arm][w]["candidate"][kind][b] = row["counters"]
        blk = blocks.setdefault(w, {})
        blk[arm] = moonshot.proxy_metrics(row["counters"])
        if arm == "combined":
            orig = runs[w]["candidate"][kind][b]
            blk["control_differences_from_original_by_counter"] = {
                k: [orig[k], v] for k, v in row["counters"].items() if v != orig[k]}
    proxy = {a: moonshot.read_proxy(r, sample, cal, reps=0) for a,r in contrast.items()}
    cells = {}
    for w, blk in blocks.items():
        p_old = old["predicted_over_lazy"]["candidate"][w]
        assert p_old["ratio"] == canonical["predicted_over_lazy"]["candidate"][w]["ratio"]
        pred = {a: proxy[a]["predicted_over_lazy"]["candidate"][w] for a in arms}
        cells[w] = {"historical_combined_over_lazy": p_old["ratio"],
                    "actual_changed_block": blk,
                    "block165_edge_over_combined_nominal_cycles": blk["edge"]["cycles_per_byte"] / blk["combined"]["cycles_per_byte"],
                    "counterfactual_edge_over_lazy": pred["edge"]["ratio"],
                    "counterfactual_rerun_combined_over_lazy": pred["combined"]["ratio"],
                    "counterfactual_edge_over_rerun_combined": pred["edge"]["ratio"] / pred["combined"]["ratio"],
                    "counterfactual_drivers": {a: pred[a]["drivers"] for a in arms}}
    print(json.dumps({"carried_rows_in_each_counterfactual": 957, "fresh_rows_in_each": 3,
                      "old_rows_reused_from": "23378304c93e85b131afb85f5e86adf6bd71d099",
                      "assumption": "All 63 other base blocks per high-W cell and every slope observation, including code layout and dispatch costs, retain their old combined-program counters. The lazy anchors and other worker cells also retain their archived values. This is a conditional diagnostic.",
                      "cells": cells}, indent=2, allow_nan=False))


if __name__ == "__main__":
    main()
