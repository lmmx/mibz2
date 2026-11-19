"""Assess the complete i9 rotor with the statistical functions from its frozen scorer."""

import json
import math
from pathlib import Path
import statistics
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "bench/moonshot"))
from moonshot import order, paired_ratio  # noqa: E402

ARMS = ["lazy", "opend", "open", "candidate"]
WORKERS = [7, 8, 9, 10]
ROUNDS = 16
BYTES = 838_800_784


def roster(arms=ARMS):
    return [(rnd, w, arm) for rnd in range(ROUNDS) for w in WORKERS for arm in order(arms, rnd + w)]


def assess(rows, arm_names=ARMS):
    observed = [(r["round"], r["workers"], r["arm"]) for r in rows]
    if observed != roster(arm_names):
        raise ValueError("incomplete, duplicated, reordered or unexpected measurement")
    for row in rows:
        if row["out_bytes"] != BYTES:
            raise ValueError("wrong decoded length")
        if not all(math.isfinite(row[k]) and row[k] > 0 for k in ("wall_s", "peak_rss_kb")):
            raise ValueError("wall time and VmHWM must be finite and positive")
    cells = {}
    for w in WORKERS:
        arms = {a: [r for r in rows if r["arm"] == a and r["workers"] == w] for a in arm_names}
        cell = {}
        for reference in arm_names[:-1]:
            candidate = arms["candidate"]
            ref = arms[reference]
            wall = paired_ratio([r["wall_s"] for r in candidate], [r["wall_s"] for r in ref])
            peaks = [r["peak_rss_kb"] for r in candidate]
            ref_peaks = [r["peak_rss_kb"] for r in ref]
            cell[reference] = {
                "wall": wall,
                "rss": paired_ratio(peaks, ref_peaks),
                "median_peak_ratio": statistics.median(peaks) / statistics.median(ref_peaks),
                "max_peak_kb": max(peaks),
                "ref_max_peak_kb": max(ref_peaks),
            }
        lazy = cell["lazy"]
        cell["target_met"] = (lazy["wall"]["hi"] <= 0.80 and lazy["rss"]["hi"] <= 1.0
                              and lazy["median_peak_ratio"] <= 1.0
                              and lazy["max_peak_kb"] <= lazy["ref_max_peak_kb"])
        cells[w] = cell
    return {"i9_criterion_met": all(c["target_met"] for c in cells.values()), "workers": cells}


if __name__ == "__main__":
    if sys.argv[1:] == ["--plan"]:
        print(json.dumps(roster()))
    else:
        with open(sys.argv[1], encoding="utf-8") as f:
            print(json.dumps(assess([json.loads(line) for line in f]), indent=2, allow_nan=False))
