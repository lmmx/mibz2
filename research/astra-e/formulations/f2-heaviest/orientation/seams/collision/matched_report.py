"""Preserve per-tape paired CPU contrasts, plus unpooled fresh-process memory."""
import csv
import sys
from collections import defaultdict
from pathlib import Path
from statistics import geometric_mean, median


def rows(path):
    with path.open() as stream:
        return list(csv.DictReader(stream))


def main(directory):
    measured = defaultdict(dict)
    for r in rows(directory / "laps.csv"):
        key = (r["section"], int(r["ordinal"]), int(r["lap"]))
        assert r["mode"] not in measured[key]
        measured[key][r["mode"]] = float(r["total_cpu_s"])
    methods = ["fixed_sort", "fixed_radix", "indexed_direct"]
    memory = {}
    for r in rows(directory / "memory.csv"):
        key = (r["section"], int(r["ordinal"]), r["mode"])
        assert key not in memory
        memory[key] = int(r["process_hwm_kib"])
    tapes = sorted({ordinal for _, ordinal, _ in measured})
    assert len(tapes) == 10 and len(measured) == 120 and len(memory) == 60
    headings = ["section", "ordinal"] + [x + "_median_s" for x in methods]
    contrasts = [(0, 2), (1, 2), (1, 0)]
    headings += [methods[i] + "_over_" + methods[j] for i, j in contrasts]
    headings += [x + "_hwm_kib" for x in methods]
    writer = csv.writer(sys.stdout, lineterminator="\n")
    writer.writerow(headings)
    for section in ["quote", "compound"]:
        medians_across = [[], [], []]
        contrasts_across = [[], [], []]
        memory_across = [[], [], []]
        for ordinal in tapes:
            laps = [measured[section, ordinal, i] for i in range(6)]
            assert all(set(lap) == set(methods) for lap in laps)
            medians = [median(lap[method] for lap in laps) for method in methods]
            ratios = [geometric_mean(lap[methods[i]] / lap[methods[j]] for lap in laps)
                      for i, j in contrasts]
            peaks = [memory[section, ordinal, method] for method in methods]
            for agg, points in [(medians_across, medians), (contrasts_across, ratios),
                                (memory_across, peaks)]:
                for target, point in zip(agg, points, strict=True):
                    target.append(point)
            writer.writerow([section, ordinal, *medians, *ratios, *peaks])
        writer.writerow([section, "all", *map(sum, medians_across),
                         *map(geometric_mean, contrasts_across), *map(max, memory_across)])


if __name__ == "__main__":
    main(Path(sys.argv[1]))
