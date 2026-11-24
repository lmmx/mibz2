"""Summarise the four-arm bundle's runs.tsv and report.txt into measurements.tsv.

Usage: python3 extract_four_arm.py data/four-arm-runs.tsv data/four-arm-report.txt > data/measurements-four-arm.tsv
Per (workers, arm), runs.tsv gives n rounds, the median and mean wall time, and the median and
maximum peak RSS. The paired ratio against A and its 95% interval are taken from report.txt,
which computed them from the paired rounds.
"""
import csv, re, statistics, sys
from collections import defaultdict

runs, report = sys.argv[1], sys.argv[2]
cells = defaultdict(lambda: {"wall": [], "peak": []})
with open(runs) as f:
    for r in csv.DictReader(f, delimiter="\t"):
        if r["kind"] != "run":
            continue
        k = (int(r["workers"]), r["arm"])
        cells[k]["wall"].append(float(r["wall_s"]))
        cells[k]["peak"].append(int(r["peak_kb"]))

ratios, w = {}, None
for line in open(report):
    m = re.match(r"## (\d+) workers", line)
    if m:
        w = int(m.group(1))
    m = re.match(r"([BCD])/A: ([\d.]+) \[([\d.]+), ([\d.]+)\] (\w+)", line)
    if m and w:
        ratios[(w, m.group(1))] = (m.group(2), m.group(3), m.group(4), m.group(5))

cols = ["source", "workers", "arm", "arm_meaning", "rounds", "wall_median_s", "wall_mean_s",
        "ratio_vs_A", "ratio_ci_lo", "ratio_ci_hi", "ratio_call", "peak_rss_median_kb", "peak_rss_max_kb"]
meaning = {"A": "as shipped", "B": "output-order codes supplied (emission oracle)",
           "C": "scanner output supplied (Huffman oracle)", "D": "both supplied"}
print("\t".join(cols))
for (wk, arm) in sorted(cells):
    c = cells[(wk, arm)]
    r = ratios.get((wk, arm), ("1.000", "", "", "baseline") if arm == "A" else ("", "", "", ""))
    print("\t".join(map(str, ["four-arm/run-20261124T204154Z", wk, arm, meaning[arm], len(c["wall"]),
        f"{statistics.median(c['wall']):.4f}", f"{statistics.mean(c['wall']):.4f}", *r,
        int(statistics.median(c["peak"])), max(c["peak"])])))
