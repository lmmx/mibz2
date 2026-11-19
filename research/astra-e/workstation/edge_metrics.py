"""Summarize CPU, wall and memory for each directly paired arm in the edge sweep."""
import csv
import json
from pathlib import Path
from statistics import median
import sys
import tarfile

from assess import WORKERS, paired_ratio
from edge import ARMS, decision

with tarfile.open(sys.argv[1]) as archive:
    runs = next(item for item in archive if item.name.endswith("/runs.jsonl"))
    rows = [json.loads(line) for line in archive.extractfile(runs)]

decision(rows)
writer = None
for w in WORKERS:
    arms = {a:[r for r in rows if r["workers"] == w and r["arm"] == a] for a in ARMS}
    for ref in ARMS[:-1]:
        a,b = arms["candidate"],arms[ref]
        assert [r["round"] for r in a] == [r["round"] for r in b] == list(range(16))
        def values(key):
            return [r[key] for r in a], [r[key] for r in b]
        wall, cpu, rss = [paired_ratio(*values(k)) for k in ("wall_s", "cpu_s", "peak_rss_kb")]
        occ = paired_ratio(*[[r["cpu_s"]/r["wall_s"] for r in runs] for runs in (a,b)])
        item = dict(workers=w,reference=ref,wall=wall["geomean"],lo=wall["lo"],hi=wall["hi"],
                    throughput=1/wall["geomean"],wall_candidate_s=median(values("wall_s")[0]),
                    wall_ref_s=median(values("wall_s")[1]),cpu=cpu["geomean"],occupancy=occ["geomean"],
                    median_rss=median(values("peak_rss_kb")[0])/median(values("peak_rss_kb")[1]),
                    rss_hi=rss["hi"],candidate_peak_median=median(values("peak_rss_kb")[0]),
                    ref_peak_median=median(values("peak_rss_kb")[1]),candidate_max=max(values("peak_rss_kb")[0]),
                    ref_max=max(values("peak_rss_kb")[1]),
                    faster_pairs=sum(x["wall_s"]<y["wall_s"] for x,y in zip(a,b)),
                    smaller_pairs=sum(x["peak_rss_kb"]<y["peak_rss_kb"] for x,y in zip(a,b)))
        if writer is None:
            writer=csv.DictWriter(sys.stdout, fieldnames=item, lineterminator="\n")
            writer.writeheader()
        writer.writerow(item)
