from collections import defaultdict
from csv import DictReader
from pathlib import Path
from statistics import median
import json
import sys

root = Path(sys.argv[1])
def rows(name):
    with (root / name).open(newline="") as stream:
        return list(DictReader(stream))

blocks = rows("blocks.csv")
large = [row for row in blocks if row["corpus"] == "wikidata" and int(row["n"]) >= 65536]
assert len(large) == 233
print("wide median", {k: median(int(r[k]) / int(r["n"]) for r in large) for k in ("A", "U")})
assert all(r["B"] == r["induced_B"] and r["U"] == r["induced_U"] for r in blocks)
print("tracer/endpoint count agreement", len(blocks), "rows including frozen")
for r in blocks:
    if r["corpus"] == "wikidata" and int(r["ordinal"]) in (118, 176):
        print("density control", r["ordinal"], int(r["r"]) / int(r["n"]), int(r["U"]), int(r["steps"]))
for r in rows("frozen.csv"):
    if int(r["limit"]) == 16:
        print("frozen", r["mode"], {k: int(r[k]) for k in ("thick", "bands", "traced", "thick_area", "steps", "rules")})

trials = defaultdict(dict)
for r in rows("timings.csv"):
    if r["experiment"] != "paired" or int(r["round"]) == 0:
        continue
    key = (r["input"], int(r["workers"]))
    within_round = trials[key].setdefault(int(r["round"]), {})
    assert r["arm"] not in within_round
    within_round[r["arm"]] = float(r["seconds"])
for key, trials_by_round in trials.items():
    paired = list(trials_by_round.values())
    assert all(set(r) == {"cells", "lazy", "qi16"} for r in paired)
    medians = {arm: median(r[arm] for r in paired) for arm in ("cells", "lazy", "qi16")}
    ratios = {base: median(r["qi16"] / r[base] for r in paired) for base in ("cells", "lazy")}
    divided_medians = {base: medians["qi16"] / medians[base] for base in ratios}
    print(key, len(paired), json.dumps([medians, ratios, divided_medians]))

heap = {r["arm"]: int(r["peak_requested"]) for r in rows("heap.csv")}
print("requested heap saving", 1 - heap["qi16"] / heap["cells"])
for arm in ("qi16", "s16"):
    arms = [r for r in rows("phases-frozen.csv") if r["arm"] == arm and int(r["round"]) > 0]
    print(arm, {k: median(float(r[k]) for r in arms) for k in ("build_seconds", "write_seconds")})
