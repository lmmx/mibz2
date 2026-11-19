"""Compare projected root streams and pair native stage observations."""
import csv
from pathlib import Path
import sys

here = Path(__file__).resolve().parent
sys.path.insert(0, str(here.parents[3] / "bench/moonshot"))
from moonshot import paired_ratio


def admitted(path):
    with path.open() as handle:
        return {int(r["stream"]): {k: int(v) for k, v in r.items()}
                for r in csv.DictReader(handle) if r["took"] == "1"}


current = admitted(here / "geometry.csv")
reference = admitted(here.parent / "f2b" / "geometry.csv")
assert current.keys() == reference.keys() and len(current) == 228
for stream, row in current.items():
    other = reference[stream]
    for field in ("words", "selectors", "nodes", "root_runs", "token_rows"):
        assert row[field] == other[field]
    assert row["merged"] == row["root_runs"]
    assert row["nodes"] <= 2 * row["words"] - 1
for field in ("nodes", "words", "selectors", "root_runs", "merged", "merge_steps"):
    print(field, "root", sum(r[field] for r in current.values()),
          "materialized", sum(r[field] for r in reference.values()))

with (here / "phase.csv").open() as handle:
    observations = list(csv.DictReader(handle))
index = {(int(r["round"]), r["stream"], r["arm"]): r for r in observations}
rounds = sorted({key[0] for key in index})
blocks = sorted({key[1] for key in index})
assert len(observations) == len(index) == 9 * 30 * 3
for field, control in (("merge_ns", "refill"), ("total_ns", "refill"), ("total_ns", "lazy")):
    totals = {arm: [sum(int(index[r, b, arm][field]) for b in blocks) for r in rounds]
              for arm in ("stream", control)}
    print(field, "stream /", control, paired_ratio(totals["stream"], totals[control]))
