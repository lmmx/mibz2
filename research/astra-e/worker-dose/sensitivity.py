"""Preserve all observations; report the effect of omitting each possible single round."""
import csv
import json
from pathlib import Path
from statistics import geometric_mean, median
import sys

raw=[json.loads(x) for x in Path(sys.argv[1]).read_text().splitlines()]
times={(r["workers"],r["round"],r["arm"]):r["wall_s"] for r in raw}
assert len(raw)==len(times)==128
writer=None
for w in (2,4):
    for ref in ("edge","cells","lazy"):
        ratio=[times[w,i,"candidate"]/times[w,i,ref] for i in range(16)]
        omitted=[geometric_mean(ratio[:i]+ratio[i+1:]) for i in range(16)]
        r=dict(workers=w,reference=ref,min_pair=min(ratio),median_pair=median(ratio),max_pair=max(ratio),
               smallest_leave_one_out=min(omitted),largest_leave_one_out=max(omitted),
               shortest_candidate_s=min(times[w,i,"candidate"] for i in range(16)),
               slowest_candidate_s=max(times[w,i,"candidate"] for i in range(16)),
               shortest_reference_s=min(times[w,i,ref] for i in range(16)),
               slowest_reference_s=max(times[w,i,ref] for i in range(16)))
        if writer is None:
            writer=csv.DictWriter(sys.stdout,fieldnames=r,lineterminator="\n");writer.writeheader()
        writer.writerow(r)
