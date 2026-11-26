"""Recompute log-paired intervals and absolute phase medians for each complete-corpus lap."""
import csv
import json
import math
import statistics as st
from pathlib import Path

def summarize(path):
    with path.open() as f:
        rows=list(csv.DictReader(f))
    labels=sorted(set(r["arm"] for r in rows))
    index={(int(r["round"]),r["arm"]):int(r["front_ns"])/1e6 for r in rows}
    assert set(index)=={(r,a) for r in range(1,37) for a in labels}
    assert all(r["blocks"]=="383" and r["records"]=="16326954" for r in rows)
    result={"median_ms":{a:st.median(index[r,a] for r in range(1,37)) for a in labels},"ratios":{}}
    for name in labels:
        if name=="stock":continue
        q=[math.log(index[r,name]/index[r,"stock"]) for r in range(1,37)]
        m=st.mean(q);h=2.03010793*st.stdev(q)/6
        result["ratios"][name+"/stock"]={"ratio":math.exp(m),"paired95":[math.exp(m-h),math.exp(m+h)]}
    return result
here=Path(__file__).resolve().parent
print(json.dumps({f:summarize(here/(f+".csv")) for f in ["front","wide"]},indent=2))
