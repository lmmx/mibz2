"""Paired systematic-block totals and complete phase accounting."""
from collections import defaultdict
from pathlib import Path
from statistics import mean, median, stdev
import csv
import math
import sys
from scipy.stats import t

d=Path(sys.argv[1])
with (d/"phase.csv").open() as f:
    rows=list(csv.DictReader(f))
stats=defaultdict(lambda:defaultdict(dict))
for r in rows:
    a=r["arm"]
    rep=int(r["rep"])
    stream=int(r["stream"])
    assert stream not in stats[a][rep]
    s={k:int(r[k]) for k in ("first_ns","total_ns","setup_mark_ns","discover_ns","merge_ns","compose_ns")}
    s["merge_only_ns"]=s["merge_ns"]-s["compose_ns"]
    s["writer_ns"]=s["total_ns"]-s["first_ns"]
    stats[a][rep][stream]=s
def value(a,rep,k):
    assert len(stats[a][rep])==10
    return sum(s[k] for s in stats[a][rep].values())
def interval(ratios):
    logs=[math.log(x) for x in ratios]
    m=mean(logs)
    e=t.ppf(.975,len(logs)-1)*stdev(logs)/math.sqrt(len(logs))
    return '%.4f [%.4f, %.4f]'%(math.exp(m),math.exp(m-e),math.exp(m+e))
ref="compact_frequent"
for arm,rounds in stats.items():
    assert len(rounds)==32
    print(arm,"mean 10-block CPU ms",
          {k:round(mean(value(arm,i,k) for i in rounds)/1e6,3)
           for k in ("setup_mark_ns","discover_ns","merge_only_ns","compose_ns","writer_ns","total_ns")})
    if arm==ref:continue
    for base in ([ref] if arm=="core_h0" else [ref,"core_h0"]):
        print(arm,"/",base,{k:interval(value(arm,i,k)/value(base,i,k) for i in rounds)
                            for k in ("first_ns","total_ns","discover_ns","writer_ns")})
    for scenario in ("matched-setup","free-setup","free-setup-and-compose"):
        def adjust(i):
            x=value(arm,i,"total_ns")-value(arm,i,"setup_mark_ns")
            if scenario=="matched-setup":x+=value(ref,i,"setup_mark_ns")
            if scenario=="free-setup-and-compose":x-=value(arm,i,"compose_ns")
            return x/value(ref,i,"total_ns")
        print("counterfactual",arm,scenario,interval(adjust(i) for i in rounds))

for name in ("ten","admissions"):
    with (d/f"{name}.csv").open() as f:
        table=list(csv.DictReader(f))
    for h in ("0","4","8"):
        selected=[r for r in table if r["h"]==h]
        assert all(r["full_runs"]==r["transport_segments"] for r in selected)
        print(name,h,len(selected),{k:sum(int(r[k]) for r in selected) for k in
              ("live_states","lf_pieces","forward_intersections","origin_bands","overlay_slices","full_runs","core_runs")},
              "second",sum(r["second"]=="true" for r in selected),"max retained B",max(int(r["reserved_bytes"]) for r in selected),
              "max extra atlas scratch/old B",max(int(r["setup_extra_capacity_upper"]) for r in selected))
if (d/"rss.csv").is_file() and (d/"rss.csv").stat().st_size:
    with (d/"rss.csv").open() as f:
        rows=list(csv.DictReader(f))
    for arm in stats:
        selected=[r for r in rows if r["arm"]==arm]
        assert len(selected)==228
        print(arm,"process high water KiB",max(int(r["process_hwm_kib"]) for r in selected),
              "max retained B",max(int(r["retained_bytes_with_output"]) for r in selected))
