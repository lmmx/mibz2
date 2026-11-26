"""Aggregate local structural and paired native receipts."""
import csv
import json
import math
import statistics
import sys
from pathlib import Path

def read(p):
    with p.open() as f:
        return list(csv.DictReader(f))

def main(folder):
    paid=read(folder/"paid.csv")
    geo=read(folder/"partition.csv")
    rss=read(folder/"memory.csv")
    costs=read(folder/"real.csv")
    result={}
    arms=sorted({int(r["h"]) for r in paid})
    assert len(paid)==len(arms)*10*24
    for i in range(24):
        for h in arms:
            assert len([r for r in paid if int(r["h"])==h and int(r["round"])==i])==10
    for h in arms:
        g=[r for r in geo if int(r["h"])==h]
        c=[r for r in costs if int(r["h"])==h and int(r["fused"])==int(h>0)]
        timed={}
        for field in ("setup_cpu_s","build_cpu_s","emit_cpu_s","cpu_s","wall_s"):
            for base in ((0,1) if h>1 else (0,)):
                logs=[]
                for lap in range(24):
                    new,old=(sum(float(r[field]) for r in paid if int(r["h"])==arm and int(r["round"])==lap)
                             for arm in (h,base))
                    logs.append(math.log(new/old))
                center=statistics.mean(logs)
                radius=2.06865761042*statistics.stdev(logs)/math.sqrt(len(logs))
                timed[f"{field}_over_h{base}"]=[math.exp(z) for z in (center,center-radius,center+radius)]
        result[h]=dict(paired=timed,
            structural_sums={k:sum(int(x[k]) for x in g) for k in list(g[0])[2:14]},
            native_sums={k:sum(int(x[k]) for x in c) for k in ("lf_pieces","live_nodes","live_packets","live_rows","translated_pieces","selectors","copied_merge_entries","setup_boundary_tests","fused_root_tests","dictionary_bytes","root_runs","output_words","mark_probes")},
            max_vmhwm_kib=max(int(r["high_water_kib"]) for r in rss if int(r["h"])==h))
    (folder/"summary.json").write_text(json.dumps(result,indent=2)+"\n")

if __name__=="__main__":
    main(Path(sys.argv[1]))
