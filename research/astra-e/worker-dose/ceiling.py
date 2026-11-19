"""Conditional service rate required of the eager subset to close the measured target gap."""
import csv
import json
from pathlib import Path
import sys
import zipfile

HERE=Path(__file__).resolve().parent
data=json.loads((HERE.parent/"workstation/evidence/edge-i9-683c22e.json").read_text())["decision"]["workers"]
# The independent five-arm runner's actual eager/edge comparison on the same corpus x4.
with zipfile.ZipFile(HERE.parent/"ci/evidence/edge-budget-44431030145.zip") as z:
    path=next(x for x in z.namelist() if x.endswith("runs.jsonl"))
    rows=list(map(json.loads,z.read(path).splitlines()))
import statistics
low={}
for w in (1,2,4):
    a=[r for r in rows if r["workers"]==w and r["arm"]=="candidate"]
    b=[r for r in rows if r["workers"]==w and r["arm"]=="cells"]
    assert len(a)==len(b)==20
    low[w]=statistics.geometric_mean(x["wall_s"]/y["wall_s"] for x,y in zip(a,b))
writer=None
for worker,entry in data.items():
    w=int(worker);e=(w+3)//4
    edge=entry["lazy"]["wall"]["geomean"]
    rate=lambda target:1+w/e*(edge/target-1)
    scaled=lambda factor:edge/(1+e/w*(factor-1))
    row=dict(workers=w,eager=e,edge_wall_vs_lazy=edge,
             eager_rate_vs_edge_to_reach_080=rate(.8),eager_rate_for_5pct_wall_gain=rate(edge*.95),
             ci_w1_edge_over_eager_wall=low[1],model_with_lowW1_eager_rate=scaled(low[1]),
             ci_w4_edge_over_eager_wall=low[4],model_with_lowW4_eager_rate=scaled(low[4]))
    if writer is None:
        writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator="\n");writer.writeheader()
    writer.writerow(row)
