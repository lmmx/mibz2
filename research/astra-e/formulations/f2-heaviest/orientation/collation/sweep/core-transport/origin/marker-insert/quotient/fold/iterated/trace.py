"""Iterate exact folds to saturation, count the subsequent native discovery schedule."""
from collections import Counter, deque
from itertools import groupby, product
from pathlib import Path
from types import SimpleNamespace
import csv
import json
import sys

D=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(D))
sys.path.insert(0,str(D.parent/'virtual'))
from fold import Atlas,Column,Fold,Oracle,Quotient,differential,incoming,library,paths,read_input
from view import full


def discovery(obj):
    index=full(obj)
    freq=[b-a for a,b in zip(obj.bases,obj.bases[1:]+[index.end])]
    z=max(range(len(freq)),key=lambda k:(freq[k],-k))
    todo=deque([(obj.bases[z],obj.bases[z]+freq[z],len(obj.weights[z]),1)])
    counts=Counter(queries=0,nodes=0,selectors=0,second_ids=0,spelling_reads=0,second_dict=0)
    while todo:
        a,b,size,tokens=todo.popleft()
        counts['nodes']+=1
        while True:
            parts=list(index.segments(a,b))
            counts['queries']+=1
            if len(parts)!=1 or parts[0][0]==z:
                break
            k,w,a=parts[0]
            b=a+w
            size+=len(obj.weights[k]);tokens+=1
        children={}
        terminal=False
        for k,w,to in parts:
            if k==z:
                terminal=True
            else:
                left,right=children.get(k,(to,to))
                assert right==to
                children[k]=(left,to+w)
        counts['selectors']+=len(parts)
        if terminal:
            counts.update(second_ids=1,spelling_reads=tokens,second_dict=size)
        for k,(left,right) in children.items():
            todo.append((left,right,size+len(obj.weights[k]),tokens+1))
    return dict(counts,second_rows=freq[z])


def census(lib,tapes):
    writer=None
    for tape in paths(tapes):
        runs,mark=read_input(tape)
        col=Column(runs)
        o=Oracle(b''.join(bytes([b])*n for b,n in runs))
        u=col.interval(b'"')
        raw,words,minima,old_freq,_=incoming(col,u,Atlas(col,u,0))
        order=sorted(words,key=minima.get)
        ids={k:i for i,k in enumerate(order)}
        weights=[words[k][1:]+b'"' for k in order]
        root=[(ids[k],w) for k,w in raw]
        positions={o.forward[x]:x-u[0] for x in range(*u)}
        back=[0]*col.rows
        for i,x in enumerate(o.forward):back[x]=i
        x=mark
        skip=0
        while x not in positions:
            x=back[x];skip+=1
        x=positions[x]
        ref=bytearray()
        pos=mark
        for _ in range(col.rows):
            ref.append(o.first[pos]);pos=o.forward[pos]
        obj=SimpleNamespace(root=root,weights=weights)
        depth=0
        paid_runs=paid_ids=0
        moved=False
        while True:
            freq=Counter()
            for k,w in obj.root:freq[k]+=w
            order=list(range(len(obj.weights)))
            m=sum(freq.values())
            bases=[]
            total=0
            for k in order:
                bases.append(total);total+=freq[k]
            obj.bases=bases
            obj.m=m
            obj.extra=0
            assert sum(len(obj.weights[k])*freq[k] for k in order)==col.rows
            got,taken=Quotient.abi(obj,lib,x,skip,col.rows)
            assert got==ref
            row=dict(ordinal=tape.stem,depth=depth,rows=m,runs=len(obj.root),ids=len(order),
                     dictionary=sum(map(len,obj.weights)),cum_old_runs=paid_runs,cum_old_ids=paid_ids,
                     mark_moved=int(moved),native_second=taken,**discovery(obj))
            if writer is None:
                writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
                writer.writeheader()
            writer.writerow(row);sys.stdout.flush()
            f=Fold(obj.root,dict(enumerate(obj.weights)),order,freq)
            if not f.chosen:break
            assert f.m<m
            differential(f)
            x,skip,moved=f.mark(x,skip)
            paid_runs+=len(obj.root)
            paid_ids+=len(order)
            depth+=1
            obj=f


if __name__=='__main__':
    census(library(sys.argv[1]),sys.argv[2])
