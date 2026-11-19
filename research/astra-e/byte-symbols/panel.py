"""Rotate complete worker decoders using the refreshed scorer's raw timings."""
import argparse
import csv
import json
from pathlib import Path
from statistics import median
import sys

p = argparse.ArgumentParser()
p.add_argument("source",type=Path)
p.add_argument("control",type=Path)
p.add_argument("candidate",type=Path)
p.add_argument("corpus",type=Path)
p.add_argument("out",type=Path)
p.add_argument("--replay",action="store_true")
a=p.parse_args()
sys.path.insert(0,str(a.source/"research/astra-e/ci"))
import paired
import moonshot

arms = ["lazy","edge","candidate"]
workers = [2,4]
if not a.replay:
    runner=paired.Paired(a.control,a.candidate,419400392)
    runner.arms=arms
    runner.binaries["edge"]=(a.control,"candidate","edge")
    runner.a.workers=workers
    runner.a.rounds=16
    runner.a.repeat=2
    runner.a.settle=0
    print("scorer",moonshot.VERSION,moonshot.scorer_commit(),flush=True)
    runner.timing(a.corpus,a.out,arms)
rows=[json.loads(line) for line in (a.out/"runs.jsonl").read_text().splitlines()]
assert [(r["round"],r["workers"],r["arm"]) for r in rows]==[(i,w,arm) for i in range(16) for w in workers for arm in moonshot.order(arms,i+w)]
with (a.out/"comparison.csv").open("w") as f:
    wr=csv.writer(f,lineterminator="\n")
    wr.writerow(["workers","reference","wall","wall_lo","wall_hi","cpu","peak_median","peak_max_candidate_kib","peak_max_reference_kib","faster_pairs"])
    for w in workers:
        def series(arm,column):
            return [r[column] for r in rows if r["arm"]==arm and r["workers"]==w]
        for ref in ("lazy","edge"):
            cw=series("candidate","wall_s");rw=series(ref,"wall_s")
            cp=series("candidate","peak_rss_kb");rp=series(ref,"peak_rss_kb")
            wall=moonshot.paired_ratio(cw,rw)
            cpu=moonshot.paired_ratio(series("candidate","cpu_s"),series(ref,"cpu_s"))
            wr.writerow([w,ref,wall["geomean"],wall["lo"],wall["hi"],cpu["geomean"],median(cp)/median(rp),max(cp),max(rp),sum(c<r for c,r in zip(cw,rw))])
