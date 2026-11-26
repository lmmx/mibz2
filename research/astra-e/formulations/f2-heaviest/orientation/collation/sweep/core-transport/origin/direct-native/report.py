"""Ten-block paired thread-CPU totals, 36 complete rotations, with no sample exclusions."""
from pathlib import Path
from collections import defaultdict
from statistics import mean,stdev
import math
import csv
import sys

out=Path(sys.argv[1])
with (out/"phase.csv").open() as f: panel=list(csv.DictReader(f))
arms=defaultdict(lambda:defaultdict(dict))
for r in panel:
    table=arms[r["arm"]][int(r["rep"])]
    idx=int(r["stream"])
    assert idx not in table
    table[idx]={k:int(v) for k,v in r.items() if k.endswith("_ns")}
def total(a,i,k):
    rows=arms[a][i]
    assert len(rows)==10
    return sum(r[k] for r in rows.values())
def interval(ratios):
    logs=[math.log(x) for x in ratios]
    assert len(logs)==36
    m=mean(logs)
    # Student t quantile .975 for 35 degrees of freedom.
    e=2.030107928250343*stdev(logs)/6
    return '%.4f [%.4f, %.4f]'%(math.exp(m),math.exp(m-e),math.exp(m+e))
order=["compact_quote","adaptive_frequent","literal_frequent","comma_quote","property_landed","twice_heaviest"]
for a in order:
    ids=sorted(arms[a])
    assert ids==list(range(36))
    keys=next(iter(arms[a][0].values())).keys()
    print(a,"mean ms",{k:round(mean(total(a,i,k) for i in ids)/1e6,3) for k in keys})
    for b in ["compact_quote","adaptive_frequent","literal_frequent","twice_heaviest"]:
        if a==b:continue
        print(a,"/",b,interval(total(a,i,"total_ns")/total(b,i,"total_ns") for i in ids))
for a,b in [("property_landed","twice_heaviest"),("comma_quote","adaptive_frequent"),
            ("property_landed","compact_quote"),("comma_quote","compact_quote")]:
    ids=range(36)
    print(a,"/",b,"both selectors free",interval((total(a,i,"total_ns")-total(a,i,"select_ns"))/
                                                (total(b,i,"total_ns")-total(b,i,"select_ns")) for i in ids))
for a in ("comma_quote","property_landed"):
    keys=("second_index_ns","second_discover_ns","second_merge_ns","second_emit_ns")
    print(a,"inferred full stage with second phases free / complete quote",
          interval((total(a,i,"total_ns")-sum(total(a,i,k) for k in keys))/total("compact_quote",i,"total_ns")
                   for i in range(36)))
for name in ("ten","admissions"):
    path=out/f"{name}.csv"
    if not path.is_file():continue
    with path.open() as f:rows=list(csv.DictReader(f))
    for a in order:
        sel=[r for r in rows if r["arm"]==a]
        assert len(sel)==(10 if name=="ten" else 228)
        print(name,a,"blocks",len(sel),"total mass",sum(int(r["hi"])-int(r["lo"]) for r in sel),
              {k:sum(int(r[k]) for r in sel) for k in ("words","dict_bytes","token_runs","edge_bytes","second_dict_bytes")},
              "second",sum(r["second"]=="true" for r in sel))
    by={(r["stream"],r["arm"]):r for r in rows}
    for a,b in [("literal_frequent","adaptive_frequent"),("property_landed","twice_heaviest")]:
        print(name,a,b,"equal masses",sum(int(r["hi"])-int(r["lo"])==int(by[i,b]["hi"])-int(by[i,b]["lo"])
                                         for (i,arm),r in by.items() if arm==a),
             "equal intervals",sum((r["lo"],r["hi"])==(by[i,b]["lo"],by[i,b]["hi"]) for (i,arm),r in by.items() if arm==a))
if (out/"rss.csv").is_file() and (out/"rss.csv").stat().st_size:
    with (out/"rss.csv").open() as f:rows=list(csv.DictReader(f))
    for a in order:
        sel=[r for r in rows if r["arm"]==a];assert len(sel)==228
        print(a,"VmHWM KiB",max(int(r["process_hwm_kib"]) for r in sel),"retained B",max(int(r["reserved_bytes"]) for r in sel))
