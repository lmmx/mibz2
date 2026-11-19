"""Induce a token BWT on an arbitrary *single* rank interval by clipping partial terminals."""
from bisect import bisect_right
from collections import Counter
from itertools import groupby,product
from pathlib import Path
import argparse
import csv
import hashlib
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent))
from order import Column,Oracle,heavy_set,truth


def nodes(col, section):
    start,end=section
    # (parent, preceding byte, live interval, low/high terminal widths, sources, selector, children)
    paths=[(-1,None)]
    live=[section]
    low=[0]
    high=[0]
    hit={}
    selectors=[]
    kids=[]
    clipped=[]
    lookups=intersections=0
    for node,(left,right) in enumerate(live):
        child={}
        selector=[]
        kids.append(child)
        selectors.append(selector)
        if left==right:continue
        lookups+=1
        run=bisect_right(col.starts,left)-1
        images={}
        while left<right:
            stop=min(right,col.starts[run+1])
            c=col.symbols[run]
            lo,hi=left+col.maps[run],stop+col.maps[run]
            if c in images:
                assert images[c][1]==lo
                images[c]=(images[c][0],hi)
            else:images[c]=(lo,hi)
            selector.append((c,stop-left))
            intersections+=1
            left=stop
            run+=1
        for c,(a,b) in images.items():
            idx=len(paths)
            child[c]=idx
            paths.append((node,c))
            l,h=max(a,start),min(b,end)
            k=max(0,h-l)
            lo=k if l==a and k else 0
            hi=k-lo
            if k:
                assert l==a or h==b
                hit[idx]=(l,h)
            if 0<k<b-a:
                clipped.append((idx,'left' if l==start else 'right'))
                assert l==start or h==end
            live.append((a+lo,b-hi))
            low.append(lo)
            high.append(hi)
    assert len(clipped)<=2
    assert len({side for _,side in clipped})==len(clipped)
    assert intersections<=lookups+len(col.symbols)-1
    return paths,hit,low,high,selectors,kids,dict(lookups=lookups,run_pieces=intersections,
                                                  partial=len(clipped),nodes=len(paths))


def build(col,section):
    paths,hit,low,high,selectors,kids,cost=nodes(col,section)
    labels={}
    for idx in hit:
        w=bytearray()
        at=idx
        while at:
            at,c=paths[at]
            w.append(c)
        labels[idx]=bytes(w)
    assert len(set(labels.values()))==len(labels)
    def append(out,v,n):
        if n:
            if out and out[-1][0]==v:
                out[-1]=(v,out[-1][1]+n)
            else:out.append((v,n))
    streams={}
    pieces=0
    for idx in reversed(range(len(paths))):
        out=[]
        append(out,idx,low[idx])
        if selectors[idx]:
            pending={}
            for c,j in kids[idx].items():
                it=iter(streams.pop(j))
                label,width=next(it)
                pending[c]=[it,label,width]
            for c,width in selectors[idx]:
                cursor=pending[c]
                while width:
                    assert cursor[2]>0
                    take=min(width,cursor[2])
                    append(out,cursor[1],take)
                    width-=take
                    cursor[2]-=take
                    if cursor[2]==0:
                        cursor[1],cursor[2]=next(cursor[0],(-1,0))
            assert all(cursor[2]==0 for cursor in pending.values())
        append(out,idx,high[idx])
        pieces+=len(out)
        streams[idx]=out
    result=[(labels[i],width) for i,width in streams.pop(0)]
    bases={labels[i]:v for i,v in hit.items()}
    cost.update(intermediate_run_entries=pieces,token_runs=len(result),types=len(labels),
                dictionary_bytes=sum(map(len,labels.values())))
    return result,bases,cost


def check(col,oracle,section):
    result,bases,cost=build(col,section)
    expected,incoming=truth(oracle,section)
    assert result==[(w,sum(1 for _ in g)) for w,g in groupby(incoming)]
    assert {w:(cells[0][0],cells[-1][0]+1) for w,cells in expected.items()}==bases
    assert cost['token_runs']<=len(col.symbols)+2
    return cost


def small():
    total=Counter()
    sharp=None
    for n in range(1,10):
        for raw in product(range(2),repeat=n):
            col=Column([(b,sum(1 for _ in g)) for b,g in groupby(raw)])
            oracle=Oracle(bytes(raw))
            for a in range(n):
                for b in range(a+1,n+1):
                    res=check(col,oracle,(a,b))
                    total['sections']+=1
                    total[f"partial_{res['partial']}"]+=1
                    if res['partial']==2 and sharp is None:
                        sharp=dict(last=bytes(raw).hex(),section=[a,b],**res)
    import json
    print(json.dumps(dict(counters=total,two_clip_witness=sharp),indent=2))


def real(tapes):
    sys.path.insert(0,str(HERE.parents[6]/"docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import paths,read_input
    wr=None
    for t in paths(tapes):
        idx=int(t.name.split(".")[0])
        runs,mark=read_input(t)
        col=Column(runs)
        oracle=Oracle(b"".join(bytes([c])*width for c,width in runs))
        n=col.rows
        p=col.interval(b'"')
        for label,sec in (("quote",p),("frequent",heavy_set(col,p,1)[0][0]),
                          ("fifth",(n//5,n//5+n//11)),("half",(n//2,n//2+n//13))):
            row=dict(ordinal=idx,section=label,lo=sec[0],hi=sec[1],**check(col,oracle,sec))
            if wr is None:
                wr=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator="\n")
                wr.writeheader()
            wr.writerow(row)
            sys.stdout.flush()


if __name__=="__main__":
    p=argparse.ArgumentParser()
    p.add_argument("tapes",nargs="?",type=Path)
    args=p.parse_args()
    real(args.tapes) if args.tapes else small()
