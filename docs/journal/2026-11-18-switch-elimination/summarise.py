"""Regenerate all aggregate counts and phase ratios quoted in the journal."""
import csv
import statistics as st
import sys
from collections import Counter, defaultdict
from pathlib import Path


def groups(path):
    blocks = defaultdict(list)
    with path.open() as f:
        for r in csv.DictReader(f):
            row = {k: int(v) for k, v in r.items()}
            blocks[row["ordinal"]].append(row)
    return blocks


def stats(label, rows, measure):
    values = [measure(v) for v in rows]
    print(label, min(values), st.median(values), max(values), sep=",")


def report(d):
    forest = groups(d / "forest-wikidata.csv")
    large = [v for v in forest.values() if v[0]["n"] >= 65536]
    print("wikidata_blocks,large", len(forest), len(large), sep=",")
    print("forest depth", sorted(Counter(map(len, large)).items()))
    print("forest final mass", sorted(Counter(v[-1]["retained"] for v in large).items()))
    print("forest budget stops", sum(v[-1]["budget"] for v in forest.values()))
    print("measure,min,median,max")
    for name, metric in [
        ("first retained / n", lambda v: v[0]["retained"] / v[0]["n"]),
        ("first peak pairs", lambda v: v[0]["peak_pairs"]),
        ("first slab pairs / reports", lambda v: v[0]["pair_tests"] / v[0]["bands"]),
        ("zero gain / retained", lambda v: v[0]["zero_gain_mass"] / v[0]["retained"]),
        ("all pair tests / n", lambda v: sum(r["pair_tests"] for r in v) / v[0]["n"]),
        ("all recipes / n", lambda v: sum(r["recipes"] for r in v) / v[0]["n"]),
        ("marked child requests / n", lambda v: v[0]["children_read"] / v[0]["n"]),
    ]:
        stats(name, large, metric)
    for name, size in [("wikidata", "rows"), ("chains-wikidata", "n")]:
        b = groups(d / (name + ".csv"))
        select = [v for v in b.values() if v[0][size] >= 65536]
        print(name, "pass counts", sorted(Counter(map(len, select)).items()))
        if name == "wikidata":
            stats("first remaining sites", select, lambda v: v[0]["left_vertices"])
            stats("first remaining rows / n", select, lambda v: v[0]["left_rows"] / v[0][size])
            stats("local splice rules / n", select, lambda v: sum(r["joins"] for r in v) / v[0][size])
        else:
            stats("two-list band visits / n", select, lambda v: sum(r["bands"] for r in v) / v[0][size])
    for name, timing in [("frozen-time", "total_seconds"), ("stream-time", "seconds")]:
        by_lap = defaultdict(dict)
        with (d / (name + ".csv")).open() as f:
            for r in csv.DictReader(f):
                if int(r["lap"]):
                    by_lap[r["lap"]][r["arm"]] = float(r[timing])
        for arm in ["cells", "switches", "chains", "forest"]:
            seconds = st.median(v[arm] for v in by_lap.values())
            ratio = st.median(v[arm] / v["cells"] for v in by_lap.values())
            print(name, arm, len(by_lap), seconds, ratio, sep=",")


if __name__ == "__main__":
    report(Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name("evidence"))
