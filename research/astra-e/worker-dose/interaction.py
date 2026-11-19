"""Compare mixed wall time with a same-round constant-capacity conditional model."""
import csv
import json
from pathlib import Path
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[2]/"bench/moonshot"))
from moonshot import paired_ratio

rows=[json.loads(s) for s in Path(sys.argv[1]).read_text().splitlines()]
times={(r["workers"],r["round"],r["arm"]):r["wall_s"] for r in rows}
assert len(times)==len(rows)==128
writer=None
for w in (2,4):
    e=(w+3)//4
    actual=[times[w,i,"candidate"] for i in range(16)]
    edge=[times[w,i,"edge"] for i in range(16)]
    eager=[times[w,i,"cells"] for i in range(16)]
    blend=[w/(e/ce+(w-e)/ed) for ce,ed in zip(eager,edge)]
    def cell(base):return paired_ratio(actual,base)
    fit=cell(blend);model=paired_ratio(blend,edge)
    r=dict(workers=w,eager=e,actual_over_constant_rate=fit["geomean"],lo=fit["lo"],hi=fit["hi"],
           constant_rate_over_edge=model["geomean"],model_lo=model["lo"],model_hi=model["hi"],
           mixed_beats_both_pairs=sum(m<min(c,d) for m,c,d in zip(actual,eager,edge)))
    if writer is None:
        writer=csv.DictWriter(sys.stdout,fieldnames=r,lineterminator="\n");writer.writeheader()
    writer.writerow(r)
