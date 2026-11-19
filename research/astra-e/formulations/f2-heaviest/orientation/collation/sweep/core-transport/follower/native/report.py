"""Paired sums of per-thread CPU on ten fixed systematic blocks, no sample exclusions."""
from pathlib import Path
from collections import defaultdict
from statistics import mean, stdev
import csv
import math
import sys
from scipy.stats import t

out=Path(sys.argv[1])
arms=defaultdict(lambda:defaultdict(dict))
with (out/"phase.csv").open() as f:
    for r in csv.DictReader(f):
        arm=r["arm"]
        rep=int(r["rep"])
        idx=int(r["stream"])
        assert idx not in arms[arm][rep]
        v={k:int(x) for k,x in r.items() if k.endswith("_ns")}
        v["merge_only"]=v["merge_ns"]-v["publish_ns"]
        v["second_total"]=v["total_ns"]-v["first_ns"]
        v["index_and_emit"]=v["second_index_shift_ns"]+v["second_emit_ns"]
        arms[arm][rep][idx]=v
assert len(arms)==7
def total(a,i,k):
    assert len(arms[a][i])==10
    return sum(v[k] for v in arms[a][i].values())
def interval(samples):
    logs=[math.log(x) for x in samples]
    m=mean(logs)
    e=t.ppf(.975,len(logs)-1)*stdev(logs)/math.sqrt(len(logs))
    return '%.4f [%.4f, %.4f]'%(math.exp(m),math.exp(m-e),math.exp(m+e))
ref="compact_frequent"
for a in [ref]+[f"{prefix}_h{h}" for prefix in ("original","physical") for h in (0,4,8)]:
    ids=sorted(arms[a])
    assert ids==list(range(35))
    print(a,"mean ms",{k:round(mean(total(a,i,k) for i in ids)/1e6,3)
                       for k in ("setup_mark_ns","discover_ns","merge_only","publish_ns","second_index_shift_ns",
                                 "second_discover_ns","second_merge_ns","second_emit_ns","second_total","total_ns")})
    if a.startswith("physical"):
        for b in [ref,a.replace("physical","original")]:
            print(a,"/",b,{k:interval(total(a,i,k)/total(b,i,k) for i in ids) for k in
                            ("total_ns","first_ns","discover_ns","second_total","second_discover_ns","second_emit_ns")})
        print(a,"/ Compact counterfactual",
              {method:interval((total(a,i,"total_ns")-total(a,i,"setup_mark_ns")
                                +(total(ref,i,"setup_mark_ns") if method=="matched_setup" else 0)
                                -(total(a,i,"publish_ns") if method=="free_setup_and_publication" else 0))
                               /total(ref,i,"total_ns") for i in ids)
               for method in ("matched_setup","free_setup","free_setup_and_publication")})
for name in ("ten","admissions"):
    with (out/f"{name}.csv").open() as f:
        rows=list(csv.DictReader(f))
    for h in ("0","4","8"):
        sel=[r for r in rows if r["h"]==h]
        assert len(sel)==(10 if name=="ten" else 228)
        print(name,h,{k:sum(int(r[k]) for r in sel) for k in
                       ("live_states","core_runs","words","physical_runs","max_mass","overlay_fragments","dict_bytes")},
              "second",sum(r["second"]=="true" for r in sel))
if (out/"rss.csv").is_file() and (out/"rss.csv").stat().st_size:
    with (out/"rss.csv").open() as f:
        rows=list(csv.DictReader(f))
    for a in sorted(arms):
        sel=[r for r in rows if r["arm"]==a]
        assert len(sel)==228
        print(a,"VmHWM KiB",max(int(r["process_hwm_kib"]) for r in sel),
              "retained B",max(int(r["retained_bytes_with_output"]) for r in sel))
