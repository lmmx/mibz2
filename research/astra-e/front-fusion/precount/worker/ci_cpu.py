"""Replay full-worker CPU and average concurrent thread occupancy against frozen 80d8409."""
import csv
import hashlib
import json
from pathlib import Path
import sys
from zipfile import ZipFile

here = Path(__file__).resolve().parent
root = here.parents[4]
sys.path.insert(0, str(root / "bench/moonshot"))
from moonshot import paired_ratio

archive = root / "research/astra-e/ci/evidence/front-fusion-44445335323.zip"
assert hashlib.sha256(archive.read_bytes()).hexdigest() == "799ea11a50501ad419e102449f9fd9872af5974846baaf6ee656da13aa6f1460"
with ZipFile(archive) as z:
    rows = list(map(json.loads, z.read("runs.jsonl").splitlines()))
out = csv.writer(sys.stdout, lineterminator="\n")
out.writerow(["workers", "metric", "candidate_over_predecessor", "lo", "hi"])
for w in [1, 2, 4]:
    groups = {}
    for arm in ("candidate", "predecessor"):
        values = [r for r in rows if r["workers"] == w and r["arm"] == arm]
        assert [r["round"] for r in values] == list(range(20))
        groups[arm] = values
    for metric, value in [("cpu", lambda r: r["cpu_s"]), ("occupancy", lambda r: r["cpu_s"] / r["wall_s"])]:
        ratio = paired_ratio([value(r) for r in groups["candidate"]], [value(r) for r in groups["predecessor"]])
        out.writerow([w, metric, ratio["geomean"], ratio["lo"], ratio["hi"]])
