"""Audit all raw pairs, characterize geometry, and summarize the registered screen."""
from collections import Counter, defaultdict
import csv
import json
import math
from pathlib import Path
import statistics as st
import sys

def read(path):
    with path.open() as f:
        return list(csv.DictReader(f))

def ratio(num, den):
    x = [math.log(a/b) for a,b in zip(num,den,strict=True)]
    assert len(x) == 24
    groups = [st.mean(x[j:j+6]) for j in range(0,24,6)]
    mu = st.mean(groups)
    half = 3.1824463053*st.stdev(groups)/2
    return [round(math.exp(v),6) for v in (mu,mu-half,mu+half)]

def summarize(d):
    gate = json.loads((d / "gate.json").read_text())
    raw = read(d / "times.csv")
    stamps = read(d / "stamps.csv")
    x = read(d / "paired.csv")
    assert len(x) == len(raw) == 1440 and len(stamps) == 750
    labels = {"stream":"prepared","streamgaps":"geometry","dense":"complete"}
    seq = iter(stamps[30:])
    for i,(a,b) in enumerate(zip(x,raw,strict=True)):
        assert a["arm"] == labels.get(b["arm"],b["arm"]) and all(a[k]==b[k] for k in b if k!="arm")
        if b["arm"] in labels:
            s = next(seq)
            assert all(s[k]==a[k] for k in s if k not in ("arm","n","primary","records"))
            assert s["n"]==b["rows"] and s["primary"]==b["mark"] and s["records"]==b["cap_records"]
            assert list(labels)[int(s["arm"])] == b["arm"] and s["supply_ns"] == "0"
    assert not list(seq)
    assert max(Counter((q["round"],q["block"],q["arm"]) for q in x).values())==1
    assert set(Counter((q["block"],q["arm"],q["slot"]) for q in x).values())=={4}
    phases = ("thread_ns","caps_ns","geometry_ns","index_ns","emit_ns")
    blocks = [str(q["block"]) for q in gate["block_inputs"]]
    arms = list(labels.values())+["scalar","lazy","opend"]
    stats = {}
    for block in ["all", *blocks]:
        sums = defaultdict(Counter)
        for row in x:
            if block!="all" and row["block"]!=block:
                continue
            for k in phases:
                if row.get(k):
                    sums[row["arm"],k][int(row["round"])]+=int(row[k])
        def get(a,k):
            return [sums[a,k][r] for r in range(1,25)]
        med = {a:{k:round(st.median(get(a,k))/1e6,5) for k in phases if max(get(a,k))} for a in arms}
        ratios = {f"{a}/{control}":ratio(get(a,"thread_ns"),get(control,"thread_ns"))
                  for a in arms[:3] for control in arms[3:]}
        stats[block] = dict(median_ms=med,ratios=ratios)
    g = {}
    for q in x:
        if q["arm"]!="complete": continue
        g[q["block"]]={k:int(q[k]) for k in ("states","exceptions","repair_steps","raw_bytes","geometry_bytes","index_bytes","index_layer_peak","output_bytes")}
    assert len(g)==10
    assert max(len(set(int(q["repair_steps"]) for q in x if q["arm"] in arms[:3] and q["block"]==b)) for b in g)==1
    rss=read(d/"rss.csv");assert len(rss)==180
    assert len({(q["round"],q["block"],q["arm"]) for q in rss})==180
    mem={}
    for a in arms:
        ra=[q for q in rss if q["arm"]==a]
        mem[a]={k:{"median":st.median(int(q[k]) for q in ra),"max":max(int(q[k]) for q in ra)} for k in ("pre_hwm_kib","post_hwm_kib")}
    passing=(stats["all"]["ratios"]["complete/scalar"][2]<1 and mem["complete"]["post_hwm_kib"]["max"]<=mem["scalar"]["post_hwm_kib"]["max"])
    print(json.dumps(dict(mode=gate["mode"],passes_complete_screen=passing,stats=stats,blocks=g,process_kib=mem),indent=2))

if __name__=="__main__":
    summarize(Path(sys.argv[1]))
