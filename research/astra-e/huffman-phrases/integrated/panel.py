"""Run independent frozen worker binaries through the scorer's complete normal stream path."""
import argparse
import csv
import gzip
import hashlib
import json
import math
import os
from pathlib import Path
import statistics
import subprocess
import sys
from tempfile import TemporaryDirectory

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/"bench/moonshot"))
from moonshot import paired_ratio

LABELS=["stock","phrase"]
def roster(rounds):
    for rnd in range(rounds):
        workers=[1,4] if rnd//2%2==0 else [4,1]
        for w in workers:
            labels=LABELS if (rnd%2)==(w==1) else LABELS[::-1]
            for pos,a in enumerate(labels):
                yield rnd,w,pos,a

def run(bases,corpus,out,rounds,traced):
    out.mkdir(parents=True,exist_ok=True)
    bins={a:bases/f"{a}-target/release/moonshot" for a in LABELS}
    def sha(f):return hashlib.sha256(f.read_bytes()).hexdigest()
    assert sha(corpus)=="82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
    ident={a:dict(path=str(b),sha256=sha(b),info=json.loads(subprocess.check_output([b,"info"])))for a,b in bins.items()}
    assert len({m["sha256"] for m in ident.values()})==2
    assert all(m["info"]["candidate"].endswith("front-fusion/cap-batch/worker/candidate.rs") for m in ident.values())
    (out/"binaries.json").write_text(json.dumps(ident,indent=2)+"\n")
    (out/"roster.json").write_text(json.dumps(list(roster(rounds)))+"\n")
    with (out/"runs.jsonl").open("x",buffering=1) as log,TemporaryDirectory() as tmp:
        trace=Path(tmp)/"trace.csv"
        for rnd,w,pos,a in roster(rounds):
            env=dict(os.environ)
            if traced:env["ASTRA_SCAN_MARKS"]=str(trace)
            row=json.loads(subprocess.check_output([bins[a],"run","--arm","candidate","--workers",str(w),"--file",corpus,"--repeat","2"],env=env))
            assert row["arm"]=="candidate" and row["workers"]==w and row["out_bytes"]==419400392
            if traced:
                target=out/f"marks-{rnd}-{w}-{a}.csv.gz"
                with gzip.open(target,"wb",compresslevel=3) as dst:dst.write(trace.read_bytes())
                trace.unlink()
            row.update(round=rnd,position=pos,source=a)
            log.write(json.dumps(row)+"\n")
            if pos==1 and w==([1,4] if rnd//2%2==0 else [4,1])[-1]:
                print(f"completed {rnd+1}/{rounds}",flush=True)

def summary(path):
    rows=[json.loads(s)for s in path.read_text().splitlines()]
    n=len(rows)//4
    assert n>=8 and n%4==0 and len(rows)==4*n
    assert [(r["round"],r["workers"],r["position"],r["source"]) for r in rows]==list(roster(n))
    lookup={(r["round"],r["workers"],r["source"]):r for r in rows}
    result={}
    for w in (1,4):
        cell={}
        for metric in ("wall_s","cpu_s","peak_rss_kb"):
            x=[lookup[i,w,"phrase"][metric] for i in range(n)]
            y=[lookup[i,w,"stock"][metric] for i in range(n)]
            logs=[math.log(a/b)for a,b in zip(x,y)]
            batches=[math.exp(statistics.mean(logs[i:i+4]))for i in range(0,n,4)]
            cell[metric]=dict(paired=paired_ratio(x,y),four_round=paired_ratio(batches,[1.0]*len(batches)),
                              median_phrase=statistics.median(x),median_stock=statistics.median(y),
                              maximum_phrase=max(x),maximum_stock=max(y))
        result[w]=cell
    print(json.dumps(result,indent=2))

if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="operation",required=True)
    cmd=sub.add_parser("run")
    for arg in ["build","corpus","out"]:cmd.add_argument(arg,type=lambda f:Path(f).resolve())
    cmd.add_argument("--rounds",type=int,required=True);cmd.add_argument("--traced",action="store_true")
    sub.add_parser("summary").add_argument("runs",type=Path)
    a=p.parse_args()
    if a.operation=="run":run(a.build,a.corpus,a.out,a.rounds,a.traced)
    else:summary(a.runs)
