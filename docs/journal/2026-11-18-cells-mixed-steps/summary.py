#!/usr/bin/env python3
"""Reduce the per-block census. Percentages use all logical Cells steps as denominator."""
import csv
import statistics
import sys
from pathlib import Path

path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name("blocks.csv")
with path.open() as f:
    rows = [{k: (v if k == "corpus" else int(v)) for k, v in row.items()} for row in csv.DictReader(f)]
wiki = [r for r in rows if r["corpus"] == "wikidata.bz2"]
prose = [r for r in rows if r["corpus"] == "prose.bz2"]
writer = csv.writer(sys.stdout, lineterminator="\n")
writer.writerow(["group", "blocks", "steps", "fresh", "eligible", "shared", "arena_bytes",
                 "fresh_TB", "min_pct", "median_pct", "max_pct", "balance", "min_balance"])
for name, group in (("frozen", wiki[:1]), ("every24", [r for r in wiki if r["block"] % 24 == 0]),
                    ("all", wiki), ("admitted", [r for r in wiki if r["admitted"]]), ("prose", prose)):
    if not group:
        continue
    hits = [r["short_TE"] + r["short_TB"] for r in group]
    ratios = [100 * h / r["steps"] for r, h in zip(group, hits)]
    balances = [7 * h - r["fresh"] - 3 * r["eligible"] for r, h in zip(group, hits)]
    def total(field):
        return sum(r[field] for r in group)
    writer.writerow([name, len(group), total("steps"), total("fresh"), total("eligible"), sum(hits),
                     total("bytes_saved"), total("fresh_TB"),
                     *(f"{value:.4f}" for value in (min(ratios), statistics.median(ratios), max(ratios))),
                     sum(balances), min(balances)])
