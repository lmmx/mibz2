"""Same cut mass in either orientation with exact pre-publication counts and singleton transport."""
from collections import Counter
from itertools import groupby, product
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"powers"))
import real
from backward import normalize
from cut import Fold, append_run, choose, forward, geometry, overlay, seed
from grammar import Node, output


def extras(slices):
    seams=Counter()
    for (k,j,_),(t,u,_) in zip(slices,slices[1:]):
        if k==t:
            assert j!=u
            seams[k]+=1
    return seams


def estimate(target,edges,seams,old_runs):
    outside=[k for k in range(len(edges)) if k not in target]
    types=len(target)+sum(len(edges[k]) for k in outside)
    runs=old_runs+sum(seams[k] for k in outside)
    joins=sum(sum(j in target for j in edges[k]) for k in outside)
    return types,runs,joins


def orient(raw,words,anchor,skip,target,choice,check=False):
    freq,bounds=geometry(raw,len(words))
    assert freq[anchor]==1
    slices,edges=overlay(raw,bounds)
    opposite=set(range(len(words)))-target
    seams=extras(slices)
    old_est=estimate(target,edges,seams,len(raw))
    new_est=estimate(opposite,edges,seams,len(raw))
    assert old_est[0]+new_est[0]==len(words)+sum(map(len,edges))
    assert old_est[1]+new_est[1]==2*len(raw)+sum(seams.values())
    flip=choice=="flip" or (choice=="cheaper" and new_est<old_est)
    if flip:
        target=opposite
    table={}
    weights=[]
    selected=set()
    for k,targets in enumerate(edges):
        for j in (tuple(targets) if k not in target else (None,)):
            q=len(weights)
            table[k,j]=q
            weights.append(words[k])
            if j is not None and j in target:
                selected.add(q)
    # A singleton source has exactly one outgoing class, in either orientation.
    old_id=table[anchor,None if anchor in target else next(iter(edges[anchor]))]
    marked=old_id
    increase=0
    refined=[]
    for k,j,n in slices:
        q=table[k,None if k in target else j]
        if j==anchor:
            assert n==1
            if q in selected:
                marked=q
                increase=len(words[k])
        append_run(refined,q,n)
    assert (len(weights),len(refined),len(selected))==(new_est if flip else old_est)
    rf,rb=geometry(refined,len(weights))
    f=Fold(refined,dict(enumerate(weights)),list(range(len(weights))),dict(enumerate(rf)),chosen=selected)
    new_id=f.ids[marked]
    after,ab=geometry(f.root,len(f.weights))
    assert after[new_id]==1 and skip+increase<len(f.weights[new_id])
    if check:
        # Reconstruct full first-return destinations on every surviving row.
        before=forward(refined,rb[:-1])
        dead={nxt for nxt,q in before if q in selected}
        survivors={old:i for i,old in enumerate(i for i in range(len(before)) if i not in dead)}
        adj=forward(f.root,ab[:-1])
        for old,i in survivors.items():
            dst,q=before[old]
            ndst,nq=adj[i]
            assert nq==f.ids[q] and ndst==survivors[before[dst][0] if dst in dead else dst]
        old_mark=rb[old_id]
        mark_lands=next(i for i,(dst,_) in enumerate(before) if dst==old_mark) if old_mark in dead else old_mark
        assert ab[new_id]==survivors[mark_lands] and ((old_mark in dead)==bool(increase))
    stats=dict(flips=int(flip),moved_marks=int(increase>0),skip_added=increase,prior_types=old_est[0],
               prior_runs=old_est[1],refined_types=len(weights),refined_runs=len(refined),joins=len(selected),
               old_mass=bounds[-1],after=f.m,old_runs=len(raw),published_runs=len(f.root))
    return f.root,f.weights,new_id,skip+increase,stats


def solve(raw,literals,mark,skip,horizon,policy,check=False,trace=None):
    words=[Node(literal=s) for s in literals]
    raw,words,a=seed(raw,words,mark)
    total=Counter()
    while True:
        freq,b=geometry(raw,len(words))
        source=b[a]
        assert freq[a]==1
        pos=0
        for k,n in raw:
            if k==a:break
            pos+=n
        if pos==source:break
        slices,edges=overlay(raw,b)
        target=choose(edges,a)
        mass=sum(n for k,targets in enumerate(edges) if k not in target for j,n in targets.items() if j in target)
        if 4*mass<b[-1]:
            raw,words,a,_=normalize(raw,words,a,b,slices,check)
            freq,b=geometry(raw,len(words))
            slices,edges=overlay(raw,b)
            target=choose(edges,a)
            total['normalizations']+=1
        raw,words,a,skip,stats=orient(raw,words,a,skip,target,policy,check)
        assert 4*stats['after']<=3*stats['old_mass']
        if trace is not None:
            trace(dict(depth=total['rounds'],**stats))
        total.update(stats)
        total['rounds']+=1
    out,visits=output(words[a],skip,horizon)
    return out,dict(total,final_skip=skip,lap=len(words[a]),output_visits=visits['visits'])


def small():
    total=Counter()
    for n in range(1,7):
        for last in product(range(3),repeat=n):
            ids=sorted(set(last))
            if ids!=list(range(len(ids))):continue
            raw=[(k,len(list(g))) for k,g in groupby(last)]
            words=[bytes([65+i])*(i+1) for i in ids]
            f,b=geometry(raw,len(ids))
            p=forward(raw,b[:-1])
            for start in range(n):
                text=bytearray()
                i=start
                while True:
                    i,c=p[i]
                    text.extend(words[c])
                    if i==start:break
                horizon=sum(a*len(v) for a,v in zip(f,words))
                for skip in range(len(words[p[start][1]])):
                    expected=(text*((skip+horizon+len(text)-1)//len(text)))[skip:skip+horizon]
                    for policy in ("ordinary","flip","cheaper"):
                        got,stats=solve(raw,words,start,skip,horizon,policy,True)
                        assert got==expected and stats['lap']==len(text)
                        total.update(cases=1,nondividing=int(horizon%len(text)!=0),
                                     moved_marks=stats.get('moved_marks',0),flips=stats.get('flips',0))
    return total
