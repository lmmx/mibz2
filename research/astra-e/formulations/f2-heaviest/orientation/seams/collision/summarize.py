"""Aggregate repeat-invariant counters and per-tape median thread CPU."""
import csv
import statistics
import sys
from collections import defaultdict


def summarize(path):
    with open(path) as source:
        by_run = defaultdict(list)
        for row in csv.DictReader(source):
            by_run[row["ordinal"], row["section"], row["mode"]].append(row)
    aggregate = defaultdict(lambda: defaultdict(list))
    for (_, section, mode), rows in by_run.items():
        assert sorted(int(row["lap"]) for row in rows) == list(range(6))
        for key in rows[0]:
            if key in {"ordinal", "section", "mode", "lap"}:
                continue
            items = [row[key] for row in rows]
            if key.endswith("_s"):
                value = statistics.median(map(float, items))
            else:
                assert len(set(items)) == 1
                value = int(items[0])
            aggregate[section, mode][key].append(value)
    writer = csv.writer(sys.stdout, lineterminator="\n")
    keys = list(next(iter(aggregate.values())))
    writer.writerow(["section", "mode", "tapes", *keys])
    for (section, mode), data in aggregate.items():
        sizes = {len(values) for values in data.values()}
        assert sizes == {10}
        aggregates = [max(data[k]) if k.startswith("peak_") else sum(data[k]) for k in keys]
        writer.writerow([section, mode, 10, *aggregates])


if __name__ == "__main__":
    summarize(sys.argv[1])
