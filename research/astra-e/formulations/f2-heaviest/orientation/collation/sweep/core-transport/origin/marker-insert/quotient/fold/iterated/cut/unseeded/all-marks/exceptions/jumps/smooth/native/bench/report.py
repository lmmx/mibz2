"""Reconcile the original donor rows, three prepared-arm stamps, and fresh HWM."""
from collections import Counter
import math
from pathlib import Path
import statistics as st
import sys
from run import OLD, NEW, read, dump

def ratio(a,b):
    logs=[math.log(x/y) for x,y in zip(a,b,strict=True)]
    assert len(logs)==24
    chunks=[st.mean(logs[i:i+6]) for i in range(0,24,6)]
    mu=st.mean(chunks);margin=3.1824463053*st.stdev(chunks)/2
    return [round(math.exp(v),6) for v in (mu,mu-margin,mu+margin)]

def report(d):
    data=read(d/"paired.csv");raw=read(d/"times.csv");calls=read(d/"stamps.csv")
    assert len(data)==len(raw)==1440 and len(calls)==750
    assert len({(x["round"],x["block"],x["arm"]) for x in data})==1440
    assert set(Counter((x["block"],x["slot"],x["arm"]) for x in data).values())=={4}
    it=iter(calls[30:])
    for x,y in zip(data,raw,strict=True):
        a=OLD.index(y["arm"])
        assert x["arm"]==NEW[a] and all(y[k]==x[k] for k in y if k!="arm")
        if a<3:
            s=next(it)
            assert int(s["arm"])==a and all(s[k]==x[k] for k in ("depth","repair_steps","supply_ns","emit_ns","index_bytes","output_bytes"))
            assert s["supply_ns"]=="0" and [s["n"],s["records"],s["primary"]]==[x["rows"],x["cap_records"],x["mark"]]
    assert not list(it)
    scopes=["all",*sorted({x["block"] for x in data},key=int)];stats={};sizes={}
    for block in scopes:
        sums=Counter()
        for x in data:
            if block=="all" or x["block"]==block:sums[int(x["round"]),x["arm"]]+=int(x["thread_ns"])
        seq={a:[sums[r,a] for r in range(1,25)] for a in NEW}
        stats[block]={"median_ms":{a:round(st.median(v)/1e6,5) for a,v in seq.items()},
                  "paired_ratios":{f"{a}/{c}":ratio(seq[a],seq[c]) for a in NEW[:3] for c in NEW[3:]}}
        if block!="all":
            sizes[block]={}
            for a in NEW[:3]:
                rows=[x for x in data if x["block"]==block and x["arm"]==a]
                static={}
                for k in ("index_bytes","repair_steps"):
                    v={int(x[k]) for x in rows}
                    assert len(v)==1
                    static[k]=v.pop()
                sizes[block][a]=dict(**static,max_output_capacity=max(int(x["output_bytes"]) for x in rows))
    rss=read(d/"rss.csv")
    assert len(rss)==len({(x["round"],x["block"],x["arm"]) for x in rss})==180
    mem={}
    for a in NEW:
        mem[a]={}
        for k in ("pre_hwm_kib","post_hwm_kib"):
            v=[int(x[k]) for x in rss if x["arm"]==a]
            mem[a][k]={"median":st.median(v),"max":max(v)}
    ratios=stats["all"]["paired_ratios"]
    dump(d/"summary.json",dict(measured=stats,geometry=sizes,rss_kib=mem,
                    competitive_depths=[a for a in NEW[:3] if ratios[f"{a}/scalar"][2]<1],
                    unresolved_depths=[a for a in NEW[:3] if ratios[f"{a}/scalar"][1]<=1<=ratios[f"{a}/scalar"][2]]))

if __name__=="__main__":
    report(Path(sys.argv[1]))
