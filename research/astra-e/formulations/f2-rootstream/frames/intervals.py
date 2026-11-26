"""Projected membership intervals and independent chunk interpreter on real blocks."""
from bisect import bisect_right
from collections import Counter
import csv
from itertools import accumulate,groupby
from pathlib import Path
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"audit"))
from real import ROOT,compact,read_input,spell
from tree import encode,from_root,routes

def check(labels,parent):
    paths=routes(parent)
    tapes=from_root(labels,paths)
    if any(not t for t in tapes):
        return None
    source=encode(labels)
    last=[-2]*len(parent)
    intervals=[0]*len(parent)
    for i,(label,_) in enumerate(source):
        for u in paths[label]:
            if last[u]+1!=i:
                intervals[u]+=1
            last[u]=i
    ends=[list(accumulate(w for _,w in t)) for t in tapes]
    used=[0]*len(parent)
    got=[]
    frames=0
    steps=0
    def consume(u,width):
        nonlocal steps,frames
        frames+=1
        while width:
            steps+=1
            k=bisect_right(ends[u],used[u])
            take=min(ends[u][k]-used[u],width)
            used[u]+=take
            width-=take
            child=tapes[u][k][0]
            pure_child=len(tapes[child])==1 and tapes[child][0][0]==child
            if child==u or pure_child:
                got.append((child,take))
            else:
                consume(child,take)
    consume(0,len(labels))
    assert got==source
    inside=sum(intervals[v] for v,t in enumerate(tapes) if v and (len(t)>1 or t[0][0]!=v))
    assert frames==1+inside
    assert steps==len(source)+inside
    return [len(labels),len(parent),sum(map(len,tapes)),len(source),sum(intervals),frames,steps]

if __name__=="__main__":
    w=csv.writer(sys.stdout,lineterminator="\n")
    w.writerow(["block","rows","nodes","selectors","T","sum_K","frames","overlaps"])
    for path in sorted((ROOT/"research/astra-e/dense/target/wikidata").glob("*.irec.xz")):
        runs,_=read_input(path)
        unique,labels,_=spell(b"".join(bytes([c])*width for c,width in runs),ord('"'))
        parent,terminal=compact(unique)
        result=check([terminal[v] for v in labels],parent)
        w.writerow([int(path.name.split(".")[0]),*result])
