"""Forced stable cut, including compressed reconstruction and a dense all-partner oracle."""
import argparse

from collections import Counter, defaultdict
import csv
from itertools import groupby, product
import json
from pathlib import Path
import sys
from unittest.mock import patch

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent))
import anchor
from anchor import Fold, Word, append_run, expanded, forward, seed


def geometry(root, classes):
    freq=[0]*classes
    for k,n in root:
        freq[k]+=n
    bounds=[0]
    for n in freq:
        assert n>0
        bounds.append(bounds[-1]+n)
    return freq,bounds


def overlay(root,bounds):
    # Source bucket j owns this physical destination row. Its predecessor
    # label comes directly from the incoming run; slices are in physical order.
    j=pos=0
    slices=[]
    edges=[{} for _ in bounds[:-1]]
    for k,width in root:
        while width:
            n=min(width,bounds[j+1]-pos)
            edges[k][j]=edges[k].get(j,0)+n
            slices.append((k,j,n))
            pos+=n
            width-=n
            if pos==bounds[j+1]:
                j+=1
    return slices,edges


def choose(edges,protected):
    # Protected singleton first, then increasing source ID. Ties stay outside.
    adj=[defaultdict(int) for _ in edges]
    for i,targets in enumerate(edges):
        for j,n in targets.items():
            if i!=j:
                adj[i][j]+=n
                adj[j][i]+=n
    assigned={}
    for i in [protected,*(k for k in range(len(edges)) if k!=protected)]:
        gain=sum(n if not assigned[j] else -n for j,n in adj[i].items() if j in assigned)
        assigned[i]=(i!=protected and gain>0)
    return {k for k,v in assigned.items() if v}


def split(slices, edges, words, target, protected):
    alias={}
    scales=[]
    selected=set()
    for k,neighbors in enumerate(edges):
        if k in target:
            alias[k,None]=len(scales)
            scales.append(words[k])
        else:
            # Dict insertion order is physical destination-bucket order.
            for j in neighbors:
                alias[k,j]=len(scales)
                if j in target:
                    selected.add(len(scales))
                scales.append(words[k])
    assert len(edges[protected])==1 and protected not in target
    a=alias[protected,next(iter(edges[protected]))]
    raw=[]
    for k,j,n in slices:
        append_run(raw,alias[k,None if k in target else j],n)
    return raw,scales,selected,a


def check_step(root,bounds,words,refined,rb,rwords,f,anchor_source):
    before=forward(root,bounds[:-1])
    ref=forward(refined,rb[:-1])
    for (d,k),(q,j) in zip(before,ref,strict=True):
        assert d==q and words[k] is rwords[j]
    after=forward(f.root,f.bases)
    dead={d for d,j in ref if j in f.chosen}
    alive={i:rank for rank,i in enumerate(i for i in range(len(ref)) if i not in dead)}
    assert anchor_source in alive
    for i,v in alive.items():
        target,k=ref[i]
        d,label=after[v]
        weight=f.weights[label]
        if target in dead:
            successor,j=ref[target]
            assert k in f.chosen and weight.left is rwords[k] and weight.right is rwords[j]
        else:
            successor=target
            assert k not in f.chosen and weight is rwords[k]
        assert d==alive[successor]
    return len(ref)+len(alive),alive[anchor_source]


def solve(raw,literals,mark,skip,horizon,trace=None,finish=None):
    weights=[Word(w) for w in literals]
    initial_runs=len(raw)
    raw,weights,a=seed(raw,weights,mark)
    counts=Counter(rounds=0,old_runs=0,old_ids=0,joins=0,splits=0,contacts=0,
                   slice_cuts=0,refined_runs=0,refined_ids=0,target_stream_runs=0,
                   selected_runs=0,checked_partners=0,peak_refined_runs=0,peak_refined_ids=0,
                   peak_self_ppm=0,seed_extra_runs=len(raw)-initial_runs)
    while True:
        freq,bounds=geometry(raw,len(weights))
        assert freq[a]==1
        source=bounds[a]
        pos=0
        for k,n in raw:
            if k==a:
                assert n==1
                break
            pos+=n
        if pos==source:
            break
        slices,edges=overlay(raw,bounds)
        target=choose(edges,a)
        m=bounds[-1]
        self_mass=sum(neighbors.get(k,0) for k,neighbors in enumerate(edges))
        entering=sum(n for k,neighbors in enumerate(edges) if k not in target
                     for j,n in neighbors.items() if j in target)
        leaving=sum(n for k,neighbors in enumerate(edges) if k in target
                    for j,n in neighbors.items() if j not in target)
        assert entering==leaving and 4*entering>=m-self_mass and entering>0
        refined,rwords,selected,ra=split(slices,edges,weights,target,a)
        rf,rb=geometry(refined,len(rwords))
        assert rb[ra]==source and rf[ra]==1
        f=Fold(refined,dict(enumerate(rwords)),list(range(len(rwords))),dict(enumerate(rf)),
               protected=ra,chosen=selected)
        h=sum(k in selected for k,n in refined)
        assert f.chosen==selected and m-f.m==entering
        assert len(slices)<=len(raw)+len(weights)-1
        assert len(refined)<=len(slices) and len(f.root)<=len(refined)+h
        checked,newsource=check_step(raw,bounds,weights,refined,rb,rwords,f,source)
        assert f.bases[f.ids[ra]]==newsource and f.m<m
        row=dict(depth=counts['rounds'],old_rows=m,residual_rows=f.m,old_runs=len(raw),
                 new_runs=len(f.root),old_ids=len(weights),refined_runs=len(refined),
                 new_ids=len(f.weights),refined_ids=len(rwords),splits=len(rwords)-len(weights),
                 slice_cuts=len(slices)-len(raw),contacts=sum(map(len,edges)),
                 target_classes=len(target),self_mass=self_mass,nonself=m-self_mass,
                 removed=entering,joins=len(selected),target_stream_runs=f.slices,selected_runs=h,
                 checked_partners=checked)
        if trace:
            trace(row)
        for k in ('old_runs','old_ids','splits','slice_cuts','joins','contacts',
                  'refined_runs','refined_ids','target_stream_runs','selected_runs','checked_partners'):
            counts[k]+=row[k]
        for name,val in [('peak_refined_runs',len(refined)),('peak_refined_ids',len(rwords)),
                         ('peak_self_ppm',self_mass*1_000_000//m)]:
            counts[name]=max(counts[name],val)
        counts['rounds']+=1
        raw,weights,a=f.root,f.weights,f.ids[ra]
    result,stats=expanded(weights[a],skip,horizon)
    counts.update(stats)
    counts.update(lap=len(weights[a]),terminal_rows=bounds[-1],terminal_ids=len(weights),
                  created=len(literals)+counts['joins'])
    if finish:
        finish(counts)
    return result,counts


def only_small(args):
    checks=Counter()
    witness={}
    for n in range(1,args.max_rows+1):
        for labels in range(1,min(n,3)+1):
            for col in product(range(labels),repeat=n):
                if len(set(col))!=labels:
                    continue
                raw=[(k,len(list(g))) for k,g in groupby(col)]
                literal=[bytes([65+j])*(j+1) for j in range(labels)]
                freq,bounds=geometry(raw,labels)
                succ=forward(raw,bounds[:-1])
                horizon=sum(w*len(s) for w,s in zip(freq,literal))
                for mark in range(n):
                    buf=bytearray()
                    x=mark
                    while True:
                        x,k=succ[x]
                        buf.extend(literal[k])
                        if x==mark:
                            break
                    for skip in range(len(literal[succ[mark][1]])):
                        history=[]
                        out,c=solve(raw,literal,mark,skip,horizon,history.append)
                        length=len(buf)
                        expect=(buf*((skip+horizon+length-1)//length))[skip:skip+horizon]
                        assert c['lap']==length and out==expect
                        checks.update(marks=1,other_cycles=int(length<horizon),
                                      incomplete_final_lap=int(horizon%length!=0),
                                      rounds=c['rounds'],checked_partners=c['checked_partners'])
                        for r in history:
                            if r['self_mass']*2>r['old_rows'] and 'high_self' not in witness:
                                witness['high_self']=dict(incoming=col,mark=mark,row=r)
                        if c['terminal_rows']>1:
                            checks['other_rows_at_stop']+=1
        print(n,file=sys.stderr,flush=True)
    print(json.dumps(dict(max_rows=args.max_rows,totals=checks,witness=witness),indent=2))


def real(args):
    tapes=list(anchor.paths(args.corpus))
    with args.trace.open('x',newline='') as out:
        writer=None
        def run(raw,literals,mark,skip,horizon):
            nonlocal writer
            tape=tapes.pop(0)
            def trace(row):
                nonlocal writer
                full=dict(ordinal=tape.name.split('.')[0],**row)
                if writer is None:
                    writer=csv.DictWriter(out,full,lineterminator='\n')
                    writer.writeheader()
                writer.writerow(full)
            return solve(raw,literals,mark,skip,horizon,trace)
        # Reuse the pinned source's independent original-LF reference, primary
        # transfer and original-horizon comparison. Only its solver is replaced.
        with patch.object(anchor,'solve',run):
            anchor.real(args.corpus)
        assert not tapes


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest='mode',required=True)
    small=sub.add_parser('small')
    small.add_argument('--max-rows',type=int,default=6)
    real_parser=sub.add_parser('real')
    real_parser.add_argument('corpus',type=Path)
    real_parser.add_argument('--trace',required=True,type=Path)
    args=p.parse_args()
    only_small(args) if args.mode=='small' else real(args)
