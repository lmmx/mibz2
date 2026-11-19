#!/usr/bin/env python3
"""Price the two position schedules and reduce the per-block evidence."""
import argparse
import csv
import json
import statistics
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent


def read_table(path):
    with path.open() as f:
        return [{k: (v if k in ("corpus", "example") else int(v)) for k, v in row.items()}
                for row in csv.DictReader(f)]


def ledger(row):
    r, m, j, k = (row[key] for key in ("rules", "marked_rules", "long_edges", "copies"))
    assert row["full"] and r > 0 and j - k == r - m
    base_mem = 6 * r + 3 * j
    joint_mem = 4 * r + 5 * k
    base_extra = 2 * r + 4 * (r - m) + 2 * j
    joint_extra = 7 * r + (j + m - 1) + m + 5 * k + 12
    return dict(base_mem=base_mem, joint_mem=joint_mem,
                base_price=base_mem + base_extra, joint_price=joint_mem + joint_extra,
                net=base_mem + base_extra - joint_mem - joint_extra,
                base_bytes=40 * r + 12 * j, joint_bytes=24 * r + 28 * k)


def profile(paths):
    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(("arm", "function", "Ir", "Dr", "Dw", "D1mr", "D1mw"))
    for path in paths:
        totals = {}
        fn = ""
        for line in Path(path).read_text().splitlines():
            if line.startswith("events:"):
                events = line.split()[1:]
            elif line.startswith("fn="):
                fn = line[3:].split("::")[-1]
            elif line[:1].isdigit() and fn in ("slots", "slots_lazy", "induce_cells", "induce_lazy", "write_cells"):
                totals.setdefault(fn, Counter()).update(dict(zip(events, map(int, line.split()[1:]))))
        arm = "lazy" if "induce_lazy" in totals else "cells"
        for fn, counts in sorted(totals.items()):
            writer.writerow((arm, fn, *(counts[k] for k in ("Ir", "Dr", "Dw", "D1mr", "D1mw"))))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profiles", nargs="+", help="frozen baseline Cachegrind files")
    parser.add_argument("--per-block", action="store_true")
    args = parser.parse_args()
    if args.profiles:
        profile(args.profiles)
        return
    rows, schedule = read_table(HERE / "blocks.csv"), read_table(HERE / "schedules.csv")
    assert len(rows) == len(schedule)
    for row, out in zip(rows, schedule):
        for key in ("corpus", "block", "copies", "copy_bytes", "literal_edges", "literal_bytes"):
            assert row[key] == out[key]
        row.update(out)
        row.update(ledger(row))
    wiki = [r for r in rows if r["corpus"] == "wikidata.bz2"]
    historical = HERE.parent / "monodromy/2026-11-10-cells-gap/evidence"
    phase = []
    for suffix in (1, 2, 3):
        run = [json.loads(s) for s in (historical / f"i9-census-{suffix}.jsonl").read_text().splitlines()]
        assert len(run) == len(wiki)
        for r, h in zip(wiki, run):
            assert (r["n"], r["r"], r["steps"], r["records"]) == (h["n"], h["c_letters"], h["c_cells"], h["records"])
        kept = [h for r, h in zip(wiki, run) if r["eager"]]
        phase.append(sum(h["n"] * h["c_write"] for h in kept) / sum(h["n"] * h["c_whole"] for h in kept))
    if args.per_block:
        keys = ["corpus", "block", "base_mem", "joint_mem", "base_price", "joint_price", "net", "base_bytes", "joint_bytes"]
        w = csv.DictWriter(sys.stdout, fieldnames=keys, extrasaction="ignore", lineterminator="\n")
        w.writeheader()
        w.writerows(rows)
        return
    print("Historical eager writer shares:", *(round(v, 6) for v in phase))
    for name, group in (("frozen", wiki[:1]), ("every24", wiki[::24]), ("wiki", wiki),
                        ("eager", [r for r in wiki if r["eager"]]), ("lazy", [r for r in wiki if r["lazy"]]),
                        ("prose", [r for r in rows if r["corpus"] == "prose.bz2"])):
        sums = Counter()
        for r in group:
            sums.update({k: v for k, v in r.items() if isinstance(v, int)})
        keys = ("steps rules long_edges copies from_later base_mem joint_mem base_price joint_price net "
                "base_bytes joint_bytes short digram live_adjacent older_same_front other_literal_pair "
                "same_tree_new_handles study575 T B old_literal_T TT BB lookahead_TT live_fresh_TT").split()
        print(name, len(group), {k: sums[k] for k in keys})
        print("minimum net", min(r["net"] for r in group), "weighted reduction",
              round(100 * sums["net"] / sums["base_price"], 4),
              "minimum spare units/rule", round(min(r["net"] / r["rules"] for r in group), 4),
              "median metadata reduction", round(statistics.median(1 - r["joint_mem"] / r["base_mem"] for r in group), 4))


if __name__ == "__main__":
    main()
