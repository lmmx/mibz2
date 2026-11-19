"""Replay paired process CPU and occupancy for the archived four-arm i9 measurement."""
import csv
import hashlib
import json
from pathlib import Path
from statistics import median
import sys
import tarfile

study = Path(__file__).resolve().parent
sys.path.insert(0, str(study.parents[2] / "bench/moonshot"))
from moonshot import paired_ratio

archive = study / "evidence/combined-i9-c381da2.tar.gz"
assert hashlib.sha256(archive.read_bytes()).hexdigest() == "70b3c3a88d330529ff2dc976ce27451a7065b8b8f5fc49e65f20c191a7d32bfa"
with tarfile.open(archive) as z:
    runs = list(map(json.loads,z.extractfile("run-20261119T013135Z/runs.jsonl")))
assert len(runs) == 256
out=csv.writer(sys.stdout,lineterminator="\n")
out.writerow(["workers","arm","median_process_cpu_s","median_wall_s","median_cpu_s_per_wall_s",
              "candidate_over_arm_cpu","cpu_low","cpu_high","candidate_over_arm_occupancy","occ_low","occ_high"])
for w in range(7,11):
    cells={a:sorted((row for row in runs if row["arm"]==a and row["workers"]==w),
                    key=lambda row: row["round"]) for a in ("candidate","lazy","opend","open")}
    assert all([row["round"] for row in v] == list(range(16)) for v in cells.values())
    def cpu(a):
        return [r["cpu_s"] for r in cells[a]]
    def occ(a):
        return [r["cpu_s"]/r["wall_s"] for r in cells[a]]
    for arm in cells:
        c=paired_ratio(cpu("candidate"),cpu(arm))
        o=paired_ratio(occ("candidate"),occ(arm))
        out.writerow([w,arm,median(cpu(arm)),median(r["wall_s"] for r in cells[arm]),median(occ(arm)),
                      c["geomean"],c["lo"],c["hi"],o["geomean"],o["lo"],o["hi"]])
