"""Recompute the statistical and phase receipts for the matched single-block panels."""
import csv
import json
import math
from pathlib import Path
import statistics as st

here=Path(__file__).resolve().parent
d={}
for folder in ("four","five","stream-panel"):
    p=here/folder
    raw=list(csv.DictReader((p/"times.csv").open()))
    arms={}
    for r in raw:arms.setdefault(r["arm"],[]).append(r)
    counts={len(rows) for rows in arms.values()}
    assert len(counts)==1;n=counts.pop();critical={24:2.068657610419,30:2.045229642133}[n]
    table={}
    for arm,rows in arms.items():
        r=list(csv.DictReader((p/f"{arm}-rss.csv").open()));assert len(r)==1
        table[arm]={"vmhwm_kib":int(r[0]["peak_kib"]),"median_ms":{k:round(st.median(int(x[k+"_ns"]) for x in rows)/1e6,6) for k in ("thread","choice","publish","normalize","emit")}}
    rivals={}
    for a,ts in arms.items():
        for b,refs in arms.items():
            if a==b:continue
            z=[math.log(int(u["thread_ns"])/int(v["thread_ns"])) for u,v in zip(ts,refs)]
            assert all(u["round"]==v["round"] for u,v in zip(ts,refs))
            m=st.mean(z);r=critical*st.stdev(z)/math.sqrt(n)
            rivals[f"{a}/{b}"]=[round(math.exp(x),6) for x in (m,m-r,m+r)]
    d[folder]={"rounds":n,"profiles":table,"paired_time_ratios":rivals}
print(json.dumps(d,indent=2))
