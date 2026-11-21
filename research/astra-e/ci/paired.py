"""Run the official paired schedule with reference arms built from the PR's frozen master base."""

import argparse
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "bench" / "moonshot"))
import moonshot

WORKERS = [1, 2, 4]
ROUNDS = 16  # Two complete forward and reverse rotations of four arms.
REPEAT = 4


class Paired(moonshot.Scorer):
    def __init__(self, base, head, expected_bytes):
        options = argparse.Namespace(
            workers=WORKERS, rounds=ROUNDS, settle=2, repeat=REPEAT, target_dir=None
        )
        super().__init__(options)
        self.base = base
        self.head = head
        self.expected_bytes = expected_bytes

    def run_once(self, arm, workers, corpus, perf=None):
        self.bin = self.head if arm == "candidate" else self.base
        row = super().run_once(arm, workers, corpus, perf)
        if (row["arm"], row["workers"], row["out_bytes"]) != (arm, workers, self.expected_bytes):
            raise RuntimeError(f"Unexpected run for {arm} W={workers}: {row}")
        row["build"] = "head" if arm == "candidate" else "base"
        return row


def compare_to_reference(rows, reference):
    # The scorer summarises every comparison against an arm named "lazy".
    # Give it the selected reference from precisely the same round as each candidate run.
    relabelled = [
        dict(r, arm="lazy" if r["arm"] == reference else "candidate")
        for r in rows
        if r["arm"] in (reference, "candidate")
    ]
    return moonshot.summarise_timing(relabelled, ["lazy", "candidate"], WORKERS)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ("base", "head", "corpus", "out"):
        parser.add_argument(arg, type=Path)
    args = parser.parse_args()
    spec = json.loads((ROOT / "bench" / "wikidata.json").read_text())
    scorer = Paired(args.base, args.head, spec["decoded_bytes"] * REPEAT)
    lazy = scorer.timing(args.corpus, args.out, ["lazy", "cells", "opend", "candidate"])
    rows = [json.loads(line) for line in (args.out / "runs.jsonl").read_text().splitlines()]
    tables = {
        "master LazyCells": lazy,
        "master eager Cells": compare_to_reference(rows, "cells"),
        "master opend": compare_to_reference(rows, "opend"),
    }
    (args.out / "ratios.json").write_text(json.dumps(tables, indent=2) + "\n")
    report = [
        "### Astra E paired research benchmark",
        "",
        f"{ROUNDS} paired rounds on Wikidata ×{REPEAT}, with 4W slots and a 2 s settle. "
        "Ratios compare the PR candidate with the base kernel in the same worker configuration. "
        "Eager Cells is the low-worker production kernel. The one-worker row uses the worker pool.",
        "",
        "| Workers | Reference | Wall ratio [95% CI] | Median peak RSS ratio |",
        "| ---: | --- | ---: | ---: |",
    ]
    for w in WORKERS:
        for label, table in tables.items():
            r = table[str(w)]["candidate"]
            report.append(
                f"| {w} | {label} | {r['geomean']:.4f} "
                f"[{r['lo']:.4f}, {r['hi']:.4f}] | {r['peak_rss_ratio']:.4f} |"
            )
    (args.out / "comparison.md").write_text("\n".join(report) + "\n")


if __name__ == "__main__":
    main()
