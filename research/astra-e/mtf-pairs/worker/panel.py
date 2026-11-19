"""Balanced complete-process finalist comparison on the official scorer's executable."""
import csv
import hashlib
import itertools
import json
import math
import os
import statistics as st
import subprocess
import sys
import time
from pathlib import Path

labels=("scalar","copied","candidate")
perms=list(itertools.permutations(labels))
def order():
    for r in range(18):
        for w in ([1,4] if r%2==0 else [4,1]):
            for i,a in enumerate(perms[(r+2*(w==4))%6]):
                yield r,w,i,a

def execute(bin_dir,corpus,dest):
    assert hashlib.sha256(corpus.read_bytes()).hexdigest()=="82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
    assert len(os.sched_getaffinity(0))>=4
    def sample(w,label):
        row=json.loads(subprocess.check_output([bin_dir/label,"run","--arm","candidate","--workers",str(w),"--repeat","1","--file",corpus]))
        assert row["workers"]==w and row["out_bytes"]==209700196
        return row
    with (dest/"warm.jsonl").open("x",buffering=1) as f:
        for w in [1,4]:
            for label in labels:print(json.dumps(dict(label=label,**sample(w,label))),file=f)
    with (dest/"timing.jsonl").open("x",buffering=1) as f:
        for r,w,pos,label in order():
            time.sleep(2)
            print(json.dumps(dict(round=r,position=pos,label=label,**sample(w,label))),file=f)

def report(dest):
    rows=[json.loads(l) for l in (dest/"timing.jsonl").read_text().splitlines()]
    assert [(r["round"],r["workers"],r["position"],r["label"]) for r in rows]==list(order())
    idx={(r["round"],r["workers"],r["label"]):r for r in rows}
    result={}
    for w in [1,4]:
        result[w]={}
        for measure in ["wall_s","cpu_s","peak_rss_kb"]:
            entry={}
            for a,b in [("candidate","scalar"),("copied","scalar"),("candidate","copied")]:
                z=[math.log(idx[r,w,a][measure]/idx[r,w,b][measure]) for r in range(18)]
                m=st.mean(z);h=2.109816*st.stdev(z)/math.sqrt(18)
                med=st.median(idx[r,w,a][measure] for r in range(18))/st.median(idx[r,w,b][measure] for r in range(18))
                entry[a+"/"+b]={"ratio":math.exp(m),"paired95":[math.exp(m-h),math.exp(m+h)],"median_ratio":med}
            result[w][measure]=entry
    print(json.dumps(result,indent=2))
if sys.argv[1]=="run":execute(*map(Path,sys.argv[2:]))
else:report(Path(sys.argv[2]))
