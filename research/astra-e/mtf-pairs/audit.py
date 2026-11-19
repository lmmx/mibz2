"""Exact corpus geometry, retention and shared-scratch native contrasts."""
import csv
import json
import math
import statistics as st
from pathlib import Path
here=Path(__file__).resolve().parent
def read(path):
    with (here/path).open() as f:return list(csv.DictReader(f))
r=[{k:int(v) for k,v in x.items()} for x in read("routes.csv")]
assert len(r)==431
geometry={}
for threshold in [16,32]:
    pairs=[x for x in read("geometry/pairs.csv") if int(x["threshold"])==threshold]
    counts=sorted((int(x["contiguous"])+int(x["separated"]) for x in pairs),reverse=True)
    geometry[threshold]={"pairs":sum(counts),"observed_entries":sum(c>0 for c in counts),
        "share_top_entries":{n:sum(counts[:n])/sum(counts) for n in [64,128,256,512]},
        "run_separated_pairs":sum(int(x["separated"]) for x in pairs)}
memory={"record_routes":sum(x["record_route"] for x in r),"private_attempts":sum(x["attempt"] for x in r),
        "accepted":sum(x["accept"] for x in r),"retried_symbols":sum(x["syms"] for x in r if x["attempt"]>x["accept"]),
        "baseline_max_vector_bytes":max(x["baseline_vec_bytes"] for x in r),
        "worker_max_vector_bytes":max(x["private_vec_bytes"]+x["fallback_vec_bytes"] for x in r),
        "extra_fixed_counter_bytes":2048}
stage={}
rows=read("placements.csv")
assert len(rows)==108
for depth in [0,3,9]:
    idx={(int(x["round"]),x["arm"]):int(x["ns"]) for x in rows if int(x["depth"])==depth}
    stage[depth]={}
    for num,den in [("stock","library"),("wide","stock"),("wide","library")]:
        z=[math.log(idx[r,num]/idx[r,den]) for r in range(1,13)];m=st.mean(z);h=2.200985*st.stdev(z)/math.sqrt(len(z))
        stage[depth][num+"/"+den]={"ratio":math.exp(m),"paired95":[math.exp(m-h),math.exp(m+h)]}
print(json.dumps({"geometry":geometry,"front_memory":memory,"placements":stage},indent=2))
