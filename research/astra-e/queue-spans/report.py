"""Summarise complete-pass ratios in the 7-arm queue-stage crossover."""
import csv
from collections import Counter, defaultdict
from pathlib import Path
from statistics import median
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "bench/moonshot"))
from moonshot import paired_ratio

ARMS = ["lazy", "opend", "preserved", "rescan", "rows", "spans", "joined"]
BLOCKS = [1, 19, 61, 108, 150, 194, 240, 282, 329, 371, 407]
ROUNDS = 28

writer = csv.writer(sys.stdout, lineterminator="\n")
writer.writerow(["run", "clock", "arm", "reference", "rounds", "median_pass_ms", "wall_ratio_or_cpu_ratio", "paired_lo", "paired_hi", "sum_block_medians_ratio"])
for path in map(Path, sys.argv[1:]):
    with path.open() as f:
        rows = list(csv.DictReader(f))
    arms = list(ARMS)
    if any(r["arm"] == "low16" for r in rows):
        arms[arms.index("joined")] = "low16"
    expected = {(a, b, q) for a in arms for b in BLOCKS for q in range(ROUNDS)}
    observed = [(r["arm"], int(r["ordinal"]), int(r["round"])) for r in rows]
    assert set(observed) == expected and len(observed) == len(expected)
    assert set(Counter((r["arm"], r["ordinal"], r["position"]) for r in rows).values()) == {ROUNDS // len(ARMS)}
    for clock in ["elapsed_ns", "thread_cpu_ns"]:
        laps = defaultdict(lambda: [0] * ROUNDS)
        blocks = defaultdict(list)
        for r in rows:
            time = int(r[clock])
            laps[r["arm"]][int(r["round"])] += time
            blocks[r["arm"], int(r["ordinal"])].append(time)
        typical = {a: sum(median(blocks[a,b]) for b in BLOCKS) for a in arms}
        for ref in ["preserved", "rows", "rescan"]:
            for arm in arms:
                s = paired_ratio(laps[arm], laps[ref])
                writer.writerow([path.stem, clock, arm, ref, ROUNDS, f"{median(laps[arm])/1e6:.4f}",
                                 *(f"{s[k]:.5f}" for k in ("geomean","lo","hi")),
                                 f"{typical[arm]/typical[ref]:.5f}"])
