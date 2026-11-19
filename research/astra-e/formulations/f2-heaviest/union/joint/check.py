"""All small parent-return subsets plus an exact work balance for the ten real blocks."""
import csv
from itertools import combinations, groupby, product
from pathlib import Path

from count import Column, Direct, actual

cases = multi = 0
for length in range(1, 9):
    for last in product(range(2), repeat=length):
        col = Column([(b, sum(1 for _ in g)) for b, g in groupby(last)])
        for pattern in (b"\0", b"\0\0", b"\0\1", b"\1\0"):
            section = col.interval(pattern)
            if section[0] == section[1]:
                continue
            parent = Direct(col, [section])
            intervals = []
            for word, group in groupby(parent.bands, lambda band: band[3]):
                bands = list(group)
                intervals.append((bands[0][0], bands[-1][0] + bands[-1][2]))
            for size in range(1, len(intervals) + 1):
                for selected in combinations(intervals, size):
                    row = actual(col, selected)
                    cases += 1
                    multi += row["components"] > 1
assert multi > 0
print(f"{cases} small joint tries matched all tagged source intervals of independent affine returns, {multi} with multiple components")
path = Path(__file__).with_name("wikidata.csv")
rows = list(csv.DictReader(path.open()))
assert len(rows) == 40
for k in (1, 2, 4, 8):
    sample = [r for r in rows if int(r["k"]) == k]
    assert len(sample) == 10
    unique, separate, probes, pieces, words, tagged = (
        sum(int(r[column]) for r in sample)
        for column in ("nodes", "colored_nodes", "predecessor_searches", "pieces", "unique_bytes", "tagged_bytes")
    )
    print(f"k={k} separate={separate} unique={unique} paid_start_probes={probes} "
          f"maximum_saved={1-unique/separate:.5%} saved_starts={1-probes/separate:.5%} "
          f"occupied_runs={pieces-separate+probes} clipped_color_run_pieces={pieces} "
          f"unique_dict_bytes={words} tagged_dict_bytes={tagged}")
