"""Summarise phase thread CPU; pair all arms at the same round and same blocks."""
import csv
from collections import defaultdict
from math import exp, log, sqrt
from pathlib import Path
from statistics import mean, median, stdev
import sys

folder = Path(sys.argv[1])
with (folder / "phase-cpu.csv").open() as source:
    rows = list(csv.DictReader(source))
phases = ("prep_mark_ns", "discover_ns", "merge_ns", "residual_output_ns", "total_ns")
blocks = sorted({r["stream"] for r in rows}, key=int)
arms = list(dict.fromkeys(r["arm"] for r in rows))
pairs = [("learn", "f2c"), ("learn", "lazy")] if "learn" in arms else []
if "owned" in arms:
    pairs.extend([("owned", "f2c"), ("owned", "lazy")])
if "combined" in arms:
    pairs.extend([("combined", "batched"), ("combined", "lazy"), ("batched", "lazy")])
    if "owned" in arms: pairs.append(("combined", "owned"))
if "outowned" in arms:
    pairs.extend([("outowned","combined"), ("outowned","batched"), ("outowned","lazy")])
if "splice" in arms:
    pairs.extend([("splice", "f2c"), ("splice", "lazy")])
if "eight" in arms:
    pairs.extend([("eight", "f2c"), ("eight", "lazy")])
if "direct-learn" in arms:
    pairs.extend([("direct-learn", "direct"), ("direct-learn", "learn"), ("direct-learn", "lazy")])
for selected in (["1"], blocks):
    print("streams " + ",".join(selected))
    observations = [r for r in rows if r["stream"] in selected]
    paired = defaultdict(lambda: defaultdict(int))
    for arm in arms:
        arm_rows = [r for r in observations if r["arm"] == arm]
        sums = [sum(median(int(r[k]) for r in arm_rows if r["stream"] == block)
                    for block in selected) for k in phases]
        print(arm, " ".join(f"{k.removesuffix('_ns')}={v/1e6:.4f}ms" for k, v in zip(phases, sums)))
        for r in arm_rows:
            paired[arm][int(r["round"])] += int(r["total_ns"])
    for arm, ref in pairs:
        samples = [log(paired[arm][i] / paired[ref][i]) for i in sorted(paired[arm])]
        assert len(samples) in (20, 21, 24, 25) and set(paired[arm]) == set(paired[ref])
        critical = {20: 2.093024, 21: 2.085963, 24: 2.068658, 25: 2.063899}[len(samples)]
        delta = critical * stdev(samples) / sqrt(len(samples))
        center = mean(samples)
        print(f"{arm}/{ref} {exp(center):.4f} [{exp(center-delta):.4f},{exp(center+delta):.4f}]")
