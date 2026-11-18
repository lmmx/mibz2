"""Summarize the probe's rounds and the paired elapsed-time observations."""

import argparse
import csv
import itertools
import statistics
import sys
from collections import defaultdict
from pathlib import Path


def read(path):
    """Read observations in their recorded order."""
    with Path(path).open(newline="") as stream:
        return list(csv.DictReader(stream))


def collapse(path, output):
    """Combine the successive contraction rounds of each block."""
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(
        "block admitted n period rounds initial_atoms sum_atoms sum_ports "
        "first_minima max_atoms rules reachable_rules".split()
    )
    for block, group in itertools.groupby(read(path), key=lambda row: row["block"]):
        rows = list(group)
        first = rows[0]
        rounds = [row for row in rows if int(row["ports"])]
        writer.writerow(
            [
                block,
                first["admitted"],
                first["n"],
                first["period"],
                len(rounds),
                first["atoms"],
                sum(int(row["atoms"]) for row in rounds),
                sum(int(row["ports"]) for row in rounds),
                first["minima"],
                max(int(row["atoms"]) for row in rows),
                first["rules"],
                first["used"],
            ]
        )


def medians(path, output):
    """Report each timed arm after its warm-up round."""
    times = defaultdict(list)
    for row in read(path):
        if int(row["round"]):
            key = (row["input"], int(row["workers"]), row["arm"])
            times[key].append(float(row["seconds"]))
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(["input", "workers", "arm", "trials", "median_seconds"])
    for (name, workers, arm), values in sorted(times.items()):
        writer.writerow([name, workers, arm, len(values), f"{statistics.median(values):.6f}"])


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("collapse", "medians"))
    parser.add_argument("input", type=Path)
    args = parser.parse_args()
    (collapse if args.action == "collapse" else medians)(args.input, sys.stdout)
