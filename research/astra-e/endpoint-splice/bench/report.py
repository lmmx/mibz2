"""Replay structural counts, paired rounds, and isolated RSS for open suffix splices."""
import csv
from collections import Counter,defaultdict
import json
import math
from pathlib import Path
import statistics as st
import sys

def read(p):
    with p.open() as f:return list(csv.DictReader(f))
def dump(p,o):p.write_text(json.dumps(o,indent=2)+"\n")

def structural(folder):
    b=read(folder/"blocks.csv");hist=read(folder/"groups.csv")
    assert len(b)==233 and sum(int(x["dense"]) for x in b)==5
    totals=Counter();by=defaultdict(Counter);orient=defaultdict(Counter)
    for x in hist:
        side=int(x["side"]);k=int(x["length"]);f=int(x["full"]);events=int(x["events"])
        assert int(x["width_min"])*events<=int(x["width_sum"])<=int(x["width_max"])*events
        d=by[x["block"]]
        if side<2:
            tally={"strict_steps":k*events,"long_strict_steps":k*events*(k>=2),"strict_groups":events,"full_groups":events*f}
        else:
            assert k==1 and not f
            tally={("equal_steps" if side==2 else "self_steps"):events}
        d.update(tally);orient[side].update(tally)
    for x in b:
        d=by[x["block"]]
        for k in ("strict_steps","long_strict_steps","strict_groups","full_groups","equal_steps","self_steps"):
            assert int(x[k])==d[k]
        assert d["strict_steps"]+d["equal_steps"]+d["self_steps"]==int(x["steps"])
        old=6*d["strict_steps"]+10*d["equal_steps"]+4*d["self_steps"]
        new=6*(d["strict_groups"]-d["full_groups"])+10*d["equal_steps"]+4*d["self_steps"]
        assert (old,new)==(int(x["old_links"]),int(x["new_links"]))
        d.update(old_links=old,new_links=new,rows=int(x["rows"]),blocks=1)
        totals.update(d)
    saved=1-totals["new_links"]/totals["old_links"]
    long=totals["long_strict_steps"]/totals["strict_steps"]
    result=dict(totals=dict(totals),orientations={s:dict(o) for s,o in orient.items()},
           pct_links_saved=100*saved,pct_strict_steps_in_multi=100*long,eligible_for_price=saved>=.1 and long>=.1)
    dump(folder/"geometry.json",result)
    return result

def ratio(x,y):
    logs=[math.log(a/b) for a,b in zip(x,y,strict=True)]
    assert len(logs)==24
    chunks=[st.mean(logs[i:i+6]) for i in range(0,24,6)]
    m=st.mean(chunks);half=3.1824463053*st.stdev(chunks)/2
    return [round(math.exp(v),6) for v in (m,m-half,m+half)]

def priced(folder):
    rows=read(folder/"times.csv")
    assert len(rows)==233*24*3==len({(x["round"],x["block"],x["arm"]) for x in rows})
    assert set(Counter((x["slot"],x["block"],x["arm"]) for x in rows).values())=={8}
    sums=defaultdict(Counter)
    for x in rows:
        for key in ("all","dense" if x["dense"]=="1" else "common",x["block"]):
            sums[key][int(x["round"]),x["arm"]]+=int(x["inverse_ns"])
    stats={}
    for label,s in sums.items():
        vectors={a:[s[i,a] for i in range(1,25)] for a in ("control","replica","batched")}
        stats[label]={"median_ms":{a:round(st.median(v)/1e6,6) for a,v in vectors.items()},
            "ratios":{f"{a}/{b}":ratio(vectors[a],vectors[b]) for a,b in
                    [("batched","control"),("batched","replica"),("replica","control")]}}
    rss=read(folder/"rss.csv")
    assert len(rss)==90==len({(x["round"],x["block"],x["arm"]) for x in rss})
    memory={}
    for a in ["control","replica","batched"]:
        memory[a]={}
        for k in ["pre_hwm_kib","post_hwm_kib"]:
            v=[int(x[k]) for x in rss if x["arm"]==a]
            memory[a][k]={"max":max(v),"median":st.median(v)}
    dump(folder/"summary.json",dict(stats=stats,rss_kib=memory))

if __name__=="__main__":
    d=Path(sys.argv[2])
    if sys.argv[1]=="geometry":structural(d)
    elif sys.argv[1]=="price":priced(d)
    else:raise ValueError(sys.argv[1])
