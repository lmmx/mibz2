"""Paired log-time intervals and median microseconds, one observation per round."""
import csv
import gzip
import math
import statistics as st
import sys
from collections import defaultdict
from pathlib import Path

samples = defaultdict(list)
for path in sorted(Path(sys.argv[1]).glob("*.gz")):
    with gzip.open(path, "rt") as data:
        for line in data:
            row, arm, workers, group, block, ref, cand = line.split()
            samples[workers, group, block, arm].append((float(ref), float(cand)))
writer = csv.writer(sys.stdout)
writer.writerow(["workers_or_emit", "suite", "block", "arm", "master_us", "arm_us",
                 "ratio", "ci95_low", "ci95_high"])
def display_order(key):
    mode, suite, block, arm = key
    index = int(block) if block.isdigit() else {"pass": 100_001, "slowest": 100_002}[block]
    return mode, suite, index, ("AA", "A", "B").index(arm)

for key in sorted(samples, key=display_order):
    pairs = samples[key]
    rounds = len(pairs)
    assert rounds in (12, 24)
    ref, cand = zip(*pairs)
    logs = [math.log(y/x) for x,y in pairs]
    mean = st.mean(logs)
    half = {12: 2.200985, 24: 2.068658}[rounds] * st.stdev(logs) / math.sqrt(rounds)
    writer.writerow([*key, f"{st.median(ref)/1000:.3f}", f"{st.median(cand)/1000:.3f}",
                     *(f"{math.exp(x):.4f}" for x in (mean, mean-half, mean+half))])
