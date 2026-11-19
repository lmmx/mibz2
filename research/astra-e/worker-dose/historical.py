"""Reprice directly paired eager/Lazy forced workers from the three original i9 receipts."""
import csv
import hashlib
import io
import json
import math
import statistics
import subprocess
import sys
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT/"bench/moonshot"))
from moonshot import paired_ratio

SNAPSHOTS = (
    ("78226d1cef87b3a51bb8d633cf222fcf6ef6deae","20261116T014443Z"),
    ("c36d70fd7a0f89c44f3a66b493b7c68c12fdc9d1","20261116T021858Z"),
    ("4d3daf69376a8028e1d7716aff2e84d15cb3e3b5","20261116T093528Z"),
)
writer = None
for rev,stamp in SNAPSHOTS:
    filename = f"lazy-crossover/run-{stamp}.tar.gz"
    def gitfile(name):
        return subprocess.check_output(["git","show",f"{rev}:{name}"])
    raw = gitfile(filename)
    digest = hashlib.sha256(raw).hexdigest()
    assert gitfile(filename+".sha256").decode().split()[0] == digest
    with tarfile.open(fileobj=io.BytesIO(raw)) as archive:
        def extract(name):
            return archive.extractfile(next(m for m in archive if m.name.endswith("/"+name))).read()
        source = extract("commit.txt").decode().strip()
        rows = list(map(json.loads,extract("runs.jsonl").splitlines()))
        dose = list(map(json.loads,extract("dose.jsonl").splitlines()))
    for w in (1,4,6,7,8,9,10):
        observations = dose if w==1 else rows
        arms = {}
        for a in ("eager","lazy"):
            hits = [r for r in observations if r["threads"]==w and r["arm"]=="mibz2-workers-"+a and r["corpus"]=="wikidata"]
            assert len(hits)==10 and all(r["termination"]=="success" and r["out"]==209700196 for r in hits)
            arms[a] = sorted(hits,key=lambda r:r["round"])
        a,b=arms["eager"],arms["lazy"]
        assert [r["round"] for r in a]==[r["round"] for r in b]==list(range(10))
        assert {r["hash"] for r in a+b}=={"a8830caee5c511a6"}
        def ratio(field):
            return paired_ratio([r[field] for r in a],[r[field] for r in b])
        wall=ratio("wall"); cpu=ratio("cpu")
        def times(samples):
            measured=[dict((key,float(value)) for key,value in (x.split("=") for x in r["thread_cpu"].split())) for r in samples]
            # The sampling ledger prints zero or more members per pool to .01 seconds.
            workers=[sum(m.get(f"mibz2-{i}",0.0) for i in range(w)) for m in measured]
            scanner=[m.get("mibz2-scan",0.0) for m in measured]
            return statistics.mean(workers),statistics.mean(scanner)
        ew,es=times(a);lw,ls=times(b)
        row=dict(evidence=rev,archive=filename,sha256=digest,build=source,workers=w,cpus=a[0]["cpus"],
                 pairs=len(a),eager_over_lazy_wall=wall["geomean"],lo=wall["lo"],hi=wall["hi"],
                 eager_over_lazy_cpu=cpu["geomean"],mean_eager_worker_cpu=ew,mean_lazy_worker_cpu=lw,
                 mean_eager_scanner_cpu=es,mean_lazy_scanner_cpu=ls,median_peak_ratio=statistics.median(r["rss_kb"] for r in a)/statistics.median(r["rss_kb"] for r in b))
        if writer is None:
            writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator="\n");writer.writeheader()
        writer.writerow(row)
