"""Recompute the journal's tables from the checked-in per-block and per-round data."""

from collections import defaultdict
import csv
from pathlib import Path
from statistics import median


EVIDENCE = Path(__file__).resolve().parent / "evidence"


def rows(name):
    """Read a CSV with a header from the evidence directory."""
    with (EVIDENCE / name).open(newline="") as source:
        return list(csv.DictReader(source))


def total(records, field):
    """Sum one integer field across the supplied rows."""
    return sum(int(row[field]) for row in records)


def main():
    """Print structural counts, allocation and timing medians, and held-out coverage."""
    grouped = [row for row in rows("grouped.csv") if int(row["n"]) >= 65_536]
    large = {row["block"] for row in grouped}
    levels = defaultdict(list)
    for row in rows("recursion.csv"):
        if row["block"] in large:
            levels[int(row["level"])].append(row)
    print(f"{len(large)} large blocks")
    print("alphabet,rows,runs")
    print("bytes", total(grouped, "n"), total(grouped, "runs"), sep=",")
    for level, records in sorted(levels.items()):
        print(level + 1, total(records, "m"), total(records, "q"), sep=",")
    print("representative LF steps", sum(total(records, "h") for records in levels.values()))
    print(
        "distinct suffixes, branching suffixes",
        total(grouped, "nodes"),
        total(grouped, "branch_nodes"),
    )

    timings = defaultdict(list)
    with (EVIDENCE / "single-core.csv").open(newline="") as source:
        reader = csv.reader(source)
        next(reader)  # Warm-up emitted by the documented driver.
        for method, seconds, _ in reader:
            timings[method].append(float(seconds))
    memory = rows("allocations.csv")
    print("decoder,seconds,median_extra_live_bytes,admitted_blocks")
    for method, values in timings.items():
        arm = [r for r in memory if r["decoder"] == method]
        peaks = [int(r["additional_peak_requested_bytes"]) for r in arm if r["block"] in large]
        print(method, f"{median(values):.6f}", median(peaks), total(arm, "fused"), sep=",")

    elapsed = defaultdict(list)
    for row in rows("workers.csv"):
        elapsed[int(row["workers"]), row["decoder"]].append(float(row["elapsed_s"]))
    print("workers,decoder,median_seconds")
    for (workers, method), values in sorted(elapsed.items()):
        print(workers, method, f"{median(values):.6f}", sep=",")

    edits = rows("rle1-changes.csv")
    print("RLE1 types, changed types, occurrences, changed occurrences, affected input bytes")
    print(
        *(
            total(edits, field)
            for field in (
                "types",
                "changed_types",
                "occurrences",
                "changed_occurrences",
                "changed_occurrence_input_bytes",
            )
        )
    )
    held = [row for row in rows("holdout.csv") if row["model"] == "windows_0_5"]
    print(f"coverage across {len(held)} held-out blocks")
    for field in ("tokens", "bytes", "dict_bytes", "suffixes"):
        print(field, f"{total(held, 'shared_' + field) / total(held, field):.2%}")

    writers = defaultdict(lambda: defaultdict(list))
    for row in rows("writer.csv"):
        writers[row["block"]][row["method"]].append(int(row["ns"]))
    for method in ("fused", "sparse"):
        ratios = [median(arms[method]) / median(arms["materialized"]) for arms in writers.values()]
        print(
            f"{method} writer / materialized, median across {len(ratios)} blocks",
            f"{median(ratios):.3f}",
        )


if __name__ == "__main__":
    main()
