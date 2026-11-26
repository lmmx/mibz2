"""Reduce the balanced outer-scope panel, including destruction of native scratch."""
import csv
import json
import math
import statistics
import sys
from pathlib import Path

def main(folder):
    with (folder/"hull-paid.csv").open() as f:
        rows=list(csv.DictReader(f))
    assert len(rows)==1280 and len({(x["ordinal"],x["round"],x["pos"]) for x in rows})==1280
    out={}
    for subject,ref in (((3,0),(0,0)),((0,1),(0,0)),((3,1),(3,0)),((3,1),(0,1)),((3,1),(0,0))):
        pair={}
        for field in ("cpu_s","wall_s","setup_cpu_s","build_cpu_s","emit_cpu_s"):
            log=[]
            for i in range(32):
                values=[]
                for h,fused in (subject,ref):
                    run=[float(r[field]) for r in rows if int(r["round"])==i and int(r["h"])==h and int(r["hull"])==fused]
                    assert len(run)==10
                    values.append(sum(run))
                log.append(math.log(values[0]/values[1]))
            mean=statistics.mean(log);half=2.039513446*statistics.stdev(log)/math.sqrt(len(log))
            pair[field]=[math.exp(z) for z in (mean,mean-half,mean+half)]
        out[f"h{subject[0]}_{'hull' if subject[1] else 'packet'}/h{ref[0]}_{'hull' if ref[1] else 'packet'}"]=pair
    (folder/"hull-summary.json").write_text(json.dumps(out,indent=2)+"\n")

if __name__=="__main__":
    main(Path(sys.argv[1]))
