"""Position-balance original tree, two history layouts and native backward Direct."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import statistics
import subprocess
import sys
import tempfile

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[7]
sys.path.insert(0,str(ROOT/"docs/journal/2026-11-18-astra-c-output-first"))
from inputs import read_input, paths, dense_audit
F="ordinal section round position arm cpu_s wall_s constructor_cpu_s writer_cpu_s tracked_constructor_bytes tracked_marked_bytes descriptor_capacity_bytes page_heads_capacity_bytes descriptors lookups examined rusage_maxrss_kib".split()
ARMS=("tree","history64","packed64","direct")

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("exe",type=Path);p.add_argument("tapes",type=Path);p.add_argument("out",type=Path)
    a=p.parse_args()
    with (HERE.parent/"native/inputs.csv").open() as f:
        manifest={int(r["ordinal"]):r for r in csv.DictReader(f)}
    with tempfile.TemporaryDirectory() as tmp,(a.out/"paired.csv").open("w") as raw,(a.out/"fresh.csv").open("w") as fresh:
        writers=[csv.DictWriter(x,fieldnames=F,lineterminator="\n") for x in (raw,fresh)]
        for w in writers:w.writeheader()
        for tape in paths(a.tapes):
            ordinal=int(tape.name.split(".")[0]);m=manifest[ordinal];runs,mark=read_input(tape)
            _,expected,_=dense_audit(runs,mark)
            assert hashlib.sha256(expected).hexdigest()==m["marked_sha256"]
            assert hashlib.sha256(tape.read_bytes()).hexdigest()==m["tape_sha256"]
            gold=Path(tmp)/"gold";gold.write_bytes(expected);inp=Path(tmp)/"input"
            for name in ("quote","frequent"):
                head=f"{len(expected)} {mark} {len(runs)} 1\n"
                inp.write_text(head+"".join(f"{b} {w}\n" for b,w in runs)+f"{name} 1 {m[name+'_lo']} {m[name+'_hi']}\n")
                cmd=[a.exe.resolve(),inp,gold,str(ordinal)]
                for arm in (*ARMS,None):
                    lines=subprocess.check_output([*cmd,"16"] if arm is None else [*cmd,"1",arm]).decode().splitlines()
                    assert len(lines)==(64 if arm is None else 1)
                    for r in csv.DictReader(lines,fieldnames=F):
                        i,slot=int(r["round"]),int(r["position"])
                        idx=(slot+i+1)%4
                        assert r["arm"]==ARMS[3-idx if i//4%2 else idx]
                        assert arm is None or r["arm"]==arm
                        writers[arm is not None].writerow(r)
                easy=(name,int(m[name+"_hi"])-int(m[name+"_lo"]))
                print(ordinal,*easy,flush=True);raw.flush();fresh.flush()
    with (a.out/"paired.csv").open() as f:rows=list(csv.DictReader(f))
    with (a.out/"fresh.csv").open() as f:fresh=list(csv.DictReader(f))
    summary={}
    for sec in ("quote","frequent"):
        group=[r for r in rows if r["section"]==sec];out={}
        for new,ref in (("history64","tree"),("packed64","tree"),("packed64","history64"),("packed64","direct")):
            comparison={}
            for field in ("cpu_s","wall_s","constructor_cpu_s","writer_cpu_s"):
                logs=[]
                for i in range(16):
                    vals=[sum(float(r[field]) for r in group if int(r["round"])==i and r["arm"]==arm) for arm in (new,ref)]
                    logs.append(math.log(vals[0]/vals[1]))
                center=statistics.mean(logs);half=2.13144954556*statistics.stdev(logs)/4
                comparison[field]=dict(n=16,ratio=math.exp(center),lo=math.exp(center-half),hi=math.exp(center+half))
            out[new+"/"+ref]=comparison
        for arm in ARMS:
            ap=[r for r in fresh if r["arm"]==arm and r["section"]==sec]
            out[arm+"_fresh"]={k:max(int(r[k]) for r in ap) for k in (
                "rusage_maxrss_kib","tracked_constructor_bytes","tracked_marked_bytes","descriptor_capacity_bytes","page_heads_capacity_bytes")}
        summary[sec]=out
    (a.out/"paired.json").write_text(json.dumps(summary,indent=2)+"\n")

if __name__=="__main__":
    main()
