"""Extract role CPU/wall from the matched, independently measured, sustained i9 sweep."""
import csv
import io
import json
import re
import statistics
import subprocess
import sys
import tarfile
from pathlib import Path

dest = Path(__file__).resolve().parent
commit = "824d095447faa59d1c33f8e685828784aed863ee"
key = "opend-sweep/run-20261118T225038Z.tar.gz"
archive = Path(sys.argv[1]).read_bytes() if len(sys.argv) > 1 else subprocess.check_output(["git","show",f"{commit}:{key}"])
with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
    readings = tar.extractfile("run-20261118T225038Z/runs.jsonl").read().splitlines()
rows = []
for line in readings:
    record = json.loads(line)
    w = record["threads"]
    if record["corpus"] != "wikidata-x4" or w not in (7, 8, 9, 10):
        continue
    assert record["termination"] == "success"
    cpu = {k: float(v) for k,v in (pair.split("=") for pair in record["thread_cpu"].split())}
    workers = [t for k,t in cpu.items() if re.fullmatch(r"mibz2-\d+", k)]
    assert len(workers) == w
    rows.append(dict(round=record["round"],workers=w,arm="lazy" if "-occ-" in record["arm"] else "opend",
                     wall=record["wall"],cpu=record["cpu"],scanner=cpu["mibz2-scan"],
                     worker_sum=round(sum(workers),3),main=cpu.get("main",0),feeder=cpu.get("mibz2-read",0)))
with (dest / "i9-roles.csv").open("w") as out:
    writer = csv.DictWriter(out, fieldnames=list(rows[0]))
    writer.writeheader()
    writer.writerows(rows)
for w in (7,8,9,10):
    for arm in ("lazy","opend"):
        samples = [r for r in rows if (r["arm"],r["workers"]) == (arm,w)]
        assert len(samples) == 12
        med = statistics.median
        print(w,arm,*(round(med(r[c] for r in samples),4) for c in ("wall","scanner","worker_sum","main")),
              *(round(med(r[c]/r["wall"]/divisor for r in samples),4) for c,divisor in (("scanner",1),("worker_sum",w),("main",1))))
