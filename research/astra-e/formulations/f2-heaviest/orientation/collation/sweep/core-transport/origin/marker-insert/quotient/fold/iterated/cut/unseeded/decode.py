"""Original positive-weight BWT grammar carried by an ordinal and its byte offset."""
from bisect import bisect_right
from collections import Counter
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/'oriented'))
from orient import estimate, extras
from backward import normalize
from cut import Fold, append_run, choose, forward, geometry, overlay
from grammar import Node, output


def point(raw,bases,x):
    """Return the LF predecessor from a single forward run scan."""
    k=bisect_right(bases,x)-1
    counts=[0]*(len(bases)-1)
    start=0
    for c,n in raw:
        if x<start+n:
            return k,bases[c]+counts[c]+x-start
        counts[c]+=n
        start+=n
    raise ValueError("uncovered marked ordinal")


def power_mark(raw,bounds,slices,x,skip,words):
    c=bisect_right(bounds,x)-1
    starts=[]
    offsets=[]
    counts=[0]*len(words)
    pos=0
    for k,n in raw:
        starts.append(pos)
        offsets.append(bounds[k]+counts[k]-pos)
        counts[k]+=n
        pos+=n
    y=x
    hops=queries=0
    while True:
        i=bisect_right(starts,y)-1
        k,n=raw[i]
        queries+=1
        if k!=c:break
        step=offsets[i]
        assert step!=0
        count=(starts[i]+n-1-y)//step+1 if step>0 else (y-starts[i])//(-step)+1
        y+=step*count
        hops+=count
    assert queries<=len(raw) and bounds[c]<=y<bounds[c+1]
    prefix=pos=0
    for k,j,n in slices:
        if pos<=y<pos+n:
            assert k!=j and j==c
            return prefix+(y-pos),skip+hops*len(words[c]),queries,hops
        prefix+=n*(k!=j)
        pos+=n
    raise ValueError("missing power entry")


def solve(raw,literals,mark,skip,horizon,policy='cheaper',check=False,trace=None,observe_cut=None):
    words=[Node(literal=v) for v in literals]
    total=Counter()
    while True:
        freq,bounds=geometry(raw,len(words))
        k,prev=point(raw,bounds,mark)
        assert 0<=skip<len(words[k])
        if prev==mark:
            root=words[k]
            break
        slices,edges=overlay(raw,bounds)
        # Zero is a tie-breaker for the fixed coloring. The mark plays no role.
        choice=choose(edges,0)
        mass=sum(n for k,row in enumerate(edges) if k not in choice for j,n in row.items() if j in choice)
        old_m=bounds[-1]
        if 4*mass<old_m:
            mark,skip,queries,hops=power_mark(raw,bounds,slices,mark,skip,words)
            raw,words,_,_=normalize(raw,words,None,bounds,slices,check)
            total.update(normalizations=1,power_mark_queries=queries,power_mark_hops=hops)
            freq,bounds=geometry(raw,len(words))
            slices,edges=overlay(raw,bounds)
            choice=choose(edges,0)
        complement=set(range(len(words)))-choice
        seams=extras(slices)
        orig=estimate(choice,edges,seams,len(raw))
        alt=estimate(complement,edges,seams,len(raw))
        flip=policy=='flip' or (policy=='cheaper' and alt<orig)
        target=complement if flip else choice
        if observe_cut is not None:
            observe_cut(raw,words,bounds,slices,target)
        selected=set()
        ids={}
        weights=[]
        for k,row in enumerate(edges):
            for j in (tuple(row) if k not in target else (None,)):
                code=len(weights)
                ids[k,j]=code
                weights.append(words[k])
                if j is not None and j in target:
                    selected.add(code)
        refined=[]
        for k,j,n in slices:
            append_run(refined,ids[k,None if k in target else j],n)
        rf,rb=geometry(refined,len(weights))
        f=Fold(refined,dict(enumerate(weights)),list(range(len(weights))),dict(enumerate(rf)),chosen=selected)
        newmark,newskip,moved=f.mark(mark,skip)
        if check:
            before=forward(refined,rb[:-1])
            dead={v for v,j in before if j in selected}
            alive={x:i for i,x in enumerate(x for x in range(len(before)) if x not in dead)}
            after=forward(f.root,f.bases)
            for x,i in alive.items():
                nxt,typ=before[x]
                assert after[i]==(alive[before[nxt][0] if nxt in dead else nxt],f.ids[typ])
            prevrow=next((x for x,(v,c) in enumerate(before) if v==mark),None)
            assert moved==(mark in dead) and newmark==alive[prevrow if moved else mark]
            assert newskip==skip+(len(weights[before[prevrow][1]]) if moved else 0)
        n=len(f.root)
        counts=dict(old_rows=old_m,cut_rows=bounds[-1],after=f.m,old_runs=len(raw),refined_ids=len(weights),
                    refined_runs=len(refined),published_runs=n,joins=len(selected),
                    moved_marks=int(moved),flips=int(flip))
        assert 4*f.m<=3*old_m
        if trace is not None:
            trace(dict(depth=total['rounds'],**counts))
        total.update(counts)
        total['rounds']+=1
        raw,words,mark,skip=f.root,f.weights,newmark,newskip
    out,visits=output(root,skip,horizon)
    return out,dict(total,lap=len(root),terminal_multiplicity=freq[k],final_skip=skip,output_visits=visits['visits'])
