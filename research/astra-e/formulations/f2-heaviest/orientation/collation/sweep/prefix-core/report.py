"""Use 24 log-paired lap totals over the same ten blocks for each depth."""
import csv
import json
import math
import statistics
import sys
from pathlib import Path

def main(folder):
    def read(name):
        with (folder/name).open() as f:
            return list(csv.DictReader(f))
    rows=read("paid.csv")
    structural=read("real.csv")
    assert len(rows)==1440 and len({(r["ordinal"],r["round"],r["position"]) for r in rows})==1440
    out={}
    for h in (0,3,4,6,8,10,11,12):
        arm=[r for r in structural if int(r["h"])==h]
        assert len(arm)==10
        item=dict(sums={k:sum(int(r[k]) for r in arm) for k in list(arm[0])[4:-5]})
        if h<=10:
            for key in ("setup_cpu_s","build_cpu_s","emit_cpu_s","cpu_s","wall_s"):
                log=[]
                for lap in range(24):
                    sums=[]
                    for depth in (h,0):
                        vals=[float(r[key]) for r in rows if int(r["round"])==lap and int(r["h"])==depth]
                        assert len(vals)==10
                        sums.append(sum(vals))
                    log.append(math.log(sums[0]/sums[1]))
                center=statistics.mean(log);half=2.06865761*statistics.stdev(log)/math.sqrt(24)
                item[key]=[math.exp(x) for x in (center,center-half,center+half)]
        out[h]=item
    (folder/"summary.json").write_text(json.dumps(out,indent=2)+"\n")
if __name__=="__main__":
    main(Path(sys.argv[1]))
