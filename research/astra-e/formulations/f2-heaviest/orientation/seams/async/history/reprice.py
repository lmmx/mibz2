"""Fresh /proc peaks and paired CPU price of reclaiming wholly expired cohorts."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import statistics
import subprocess
import tempfile
from price import ARMS, F, HERE, read_input, paths, dense_audit

RF="""ordinal section round position arm cpu_s wall_s constructor_cpu_s writer_cpu_s tracked_constructor_bytes
tracked_marked_bytes descriptor_capacity_bytes page_heads_capacity_bytes descriptors lookups examined
reclaim_passes reclaim_scanned reclaim_discarded vmhwm_kib""".split()

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("bin",type=Path);p.add_argument("tapes",type=Path);p.add_argument("out",type=Path)
    a=p.parse_args()
    with (HERE.parent/"native/inputs.csv").open() as f: meta={int(r["ordinal"]):r for r in csv.DictReader(f)}
    with tempfile.TemporaryDirectory() as tmp,(a.out/"fresh-proc.csv").open("w") as fresh,(a.out/"reclaim-paired.csv").open("w") as raw:
        peak=csv.DictWriter(fresh,fieldnames=RF,lineterminator="\n");peak.writeheader()
        pairs=csv.DictWriter(raw,fieldnames=RF,lineterminator="\n");pairs.writeheader()
        for tape in paths(a.tapes):
            o=int(tape.name.split(".")[0]);m=meta[o];runs,mark=read_input(tape)
            _,expected,_=dense_audit(runs,mark);assert hashlib.sha256(expected).hexdigest()==m["marked_sha256"]
            inp=Path(tmp)/"input";gold=Path(tmp)/"gold";gold.write_bytes(expected)
            for name in ("quote","frequent"):
                inp.write_text(f"{len(expected)} {mark} {len(runs)} 1\n"+"".join(f"{x} {y}\n" for x,y in runs)
                               +f"{name} 1 {m[name+'_lo']} {m[name+'_hi']}\n")
                for arm in ARMS:
                    line=subprocess.check_output([a.bin/"match",inp,gold,str(o),"1",arm,"vmhwm"]).decode().splitlines()
                    r,=list(csv.DictReader(line,fieldnames=[*F[:-1],"vmhwm_kib"]));peak.writerow(r)
                base=[a.bin/"reclaim",inp,gold,str(o)]
                for arm in ("reclaim",None):
                    args=[*base,"1",arm] if arm else [*base,"16"]
                    lines=subprocess.check_output(args).decode().splitlines()
                    assert len(lines)==(1 if arm else 32)
                    for r in csv.DictReader(lines,fieldnames=RF):
                        (peak if arm else pairs).writerow(r)
                fresh.flush();raw.flush();print(o,name,flush=True)
    with (a.out/"fresh-proc.csv").open() as f: fresh=list(csv.DictReader(f))
    with (a.out/"reclaim-paired.csv").open() as f: pairs=list(csv.DictReader(f))
    out={}
    for s in ("quote","frequent"):
        g=[r for r in pairs if r["section"]==s]
        d={}
        for field in ("cpu_s","wall_s","constructor_cpu_s","writer_cpu_s"):
            logs=[]
            for i in range(16):
                values=[sum(float(r[field]) for r in g if int(r["round"])==i and r["arm"]==arm)
                        for arm in ("reclaim","packed")]
                logs.append(math.log(values[0]/values[1]))
            m=statistics.mean(logs);h=2.13144954556*statistics.stdev(logs)/4
            d["reclaim_over_packed_"+field]=dict(n=16,ratio=math.exp(m),lo=math.exp(m-h),hi=math.exp(m+h))
        for arm in (*ARMS,"reclaim"):
            g=[r for r in fresh if r["section"]==s and r["arm"]==arm]
            d[arm]={k:max(int(r[k]) for r in g) for k in ("vmhwm_kib","tracked_constructor_bytes","tracked_marked_bytes","descriptor_capacity_bytes")}
            if arm=="reclaim":
                d[arm]["summed"]={k:sum(int(r[k]) for r in g) for k in ("reclaim_passes","reclaim_scanned","reclaim_discarded","descriptors","lookups","examined")}
        out[s]=d
    (a.out/"reclaim.json").write_text(json.dumps(out,indent=2)+"\n")

if __name__=="__main__":
    main()
