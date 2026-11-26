"""Report paid work, occupancy and fixed-memory decisions for the direct scalar comparison."""
import csv
import hashlib
import json
from pathlib import Path
import statistics
import sys
from zipfile import ZipFile

root=Path(__file__).resolve().parents[5]
sys.path.insert(0,str(root/"bench/moonshot"))
from moonshot import paired_ratio

archive=Path(sys.argv[1])
assert hashlib.sha256(archive.read_bytes()).hexdigest()==sys.argv[2]
with ZipFile(archive) as z:
    assert z.testzip() is None
    manifest=z.read("manifest.txt").decode()
    assert "badb2aec3624bd65606c429b2e6f79e9d9d0050f research/astra-e/edge-budget/candidate.rs" in manifest
    assert "  research/astra-e/front-fusion/cap-batch/worker/candidate.rs" in manifest
    rows=list(map(json.loads,z.read("runs.jsonl").splitlines()))
out=csv.writer(sys.stdout,lineterminator="\n")
out.writerow(["workers","reference","metric","ratio","lo","hi","candidate_median","reference_median","candidate_at_most_reference"])
for w in [1,2,4]:
    def samples(name):
        group=[r for r in rows if r["arm"]==name and r["workers"]==w]
        assert [r["round"] for r in group]==list(range(20))
        return group
    for arm in ["predecessor","lazy","cells","opend"]:
        for label,fn in [("wall",lambda r:r["wall_s"]),("cpu",lambda r:r["cpu_s"]),
                         ("occupancy",lambda r:r["cpu_s"]/r["wall_s"]),("rss_kib",lambda r:r["peak_rss_kb"])]:
            c=[fn(r)for r in samples("candidate")]
            b=[fn(r)for r in samples(arm)]
            ratio=paired_ratio(c,b)
            out.writerow([w,arm,label,ratio["geomean"],ratio["lo"],ratio["hi"],
                          statistics.median(c),statistics.median(b),sum(x<=y for x,y in zip(c,b,strict=True))])
