"""Construct the exact chosen physical cut ranks and their destinations, without the atlas or a core column."""
from bisect import bisect_right
from collections import Counter, defaultdict
from itertools import groupby, product
from pathlib import Path
import csv
import json
import sys
from origin import RunIndex, Column, arcs, consecutive, heavy_set, paths, read_input


def push(spans,lo,hi):
    if lo>=hi:
        return
    if spans and spans[-1][1]==lo:
        spans[-1]=(spans[-1][0],hi)
    else:
        assert not spans or spans[-1][1]<lo
        spans.append((lo,hi))


def backward_pieces(index,active,work):
    children=defaultdict(list)
    for interval in active:
        kids,pieces=index.children(interval)
        work['search_intervals']+=1
        work['intersection_pieces']+=pieces
        for c,(l,r) in kids.items():
            push(children[c],l,r)
        work['child_pieces']+=len(kids)
    out=[]
    for c in sorted(children):
        for l,r in children[c]:
            push(out,l,r)
    work['child_unions']+=len(out)
    return out


def forward_piece(index,x):
    c=bisect_right(index.base,x)-1
    q=x-index.base[c]
    j=bisect_right(index.endcounts[c],q)
    delta=q-(0 if j==0 else index.endcounts[c][j-1])
    return index.positions[c][j]+delta,index.endcounts[c][j]-q,c


def move(index,x,d):
    for _ in range(d):
        x=forward_piece(index,x)[0]
    return x


def fixed(index,u,core,h):
    left,right=index.interval(core,u)
    active=[(left,right)] if left<right else []
    # Once a band is classified, its positive-hit distance to Z(C) is the level d.
    bins=[[],[]]
    weight=[0,0]
    work=Counter()
    for d in range(h+1):
        outside=[]
        def classify(lo,hi,good):
            if lo<hi:
                bins[good].append((lo,hi,d))
                weight[good]+=hi-lo
        for lo,hi in active:
            a,b=max(lo,u[0]),min(hi,u[1])
            if a<b:
                classify(a,b,d==h or len(core)==1)
            for l,r in ((lo,min(hi,u[0])),(max(lo,u[1]),hi)):
                if d==h:
                    classify(l,r,False)
                else:
                    push(outside,l,r)
        if not outside:
            break
        active=backward_pieces(index,outside,work)
    assert sum(weight)==right-left
    side=weight[1]<weight[0]
    points=[]
    work['terminal_bands']=sum(map(len,bins))
    for lo,hi,d in bins[side]:
        points.extend(move(index,x,d) for x in range(lo,hi))
        work['recovery_selects']+=(hi-lo)*d
    spans=consecutive(points)
    assert len(points)==weight[side] and sum(y-x for x,y in spans)==len(points)
    if not side:
        saved=[]
        cursor=left
        for lo,hi in spans:
            push(saved,cursor,lo)
            cursor=hi
        push(saved,cursor,right)
        spans=saved
    work['minority_points']=len(points)
    work['enumerated_hits']=int(side)
    # Every selected row emits the same C. The entire forward image remains
    # sorted; a select-run intersection supplies the next affine part.
    dest=spans
    for c in core:
        image=[]
        for l,r in dest:
            while l<r:
                y,width,symbol=forward_piece(index,l)
                assert symbol==c
                take=min(width,r-l)
                push(image,y,y+take)
                work['common_forward_pieces']+=1
                l+=take
        dest=image
    assert sum(y-x for x,y in spans)==weight[1]==sum(y-x for x,y in dest)
    assert all(u[0]<=lo<hi<=u[1] for lo,hi in dest)
    return (left,right),spans,dest,work


def verify(index,last,f,u,words,core,h,marks):
    full,spans,outgoing,work=fixed(index,u,core,h)
    rows=[x for l,r in spans for x in range(l,r)]
    ds=[x for l,r in outgoing for x in range(l,r)]
    actual={}
    for x,w in enumerate(words,u[0]):
        p=min(h,len(w)-1)
        if w[p:]!=core:
            continue
        y=x
        for _ in range(p):
            y=f[y]
        z=y
        for _ in core:
            z=f[z]
        actual[y]=z
    assert rows==sorted(actual) and [actual[y] for y in rows]==ds
    # Reconstruct complete return maps on BOTH sections in the original rank space.
    def direct(ranks):
        order={x:i for i,x in enumerate(ranks)}
        partners=[]
        wordlist=[]
        for x in ranks:
            w=bytearray()
            while True:
                x=f[x]
                w.append(last[x])
                if x in order:
                    break
            partners.append(order[x])
            wordlist.append(bytes(w))
        return partners,wordlist
    a,w=direct(rows)
    b,v=direct(ds)
    assert a==b and [old[len(core):]+core for old in w]==v
    assert all(old.startswith(core) for old in w)
    def marked(ranks,successors,words,mark):
        table={x:i for i,x in enumerate(ranks)}
        x=mark
        data=bytearray()
        while x not in table and len(data)<len(last):
            x=f[x]
            data.append(last[x])
        if x in table:
            i=table[x]
            while len(data)<len(last):
                data.extend(words[i][:len(last)-len(data)])
                i=successors[i]
        return data
    for mark in marks:
        expected=bytearray()
        x=mark
        for _ in range(len(last)):
            x=f[x]
            expected.append(last[x])
        assert marked(rows,a,w,mark)==marked(ds,b,v,mark)==expected
    return dict(full_mass=full[1]-full[0],chosen=len(rows),holes=full[1]-full[0]-len(rows),
                physical_components=len(spans),landing_components=len(outgoing),second_types=len(set(w)),
                second_D=sum(map(len,set(w))),marks=len(marks),endpoint_queries=2*len(core),
                **{k:work[k] for k in ('search_intervals','intersection_pieces','child_pieces','child_unions',
                   'terminal_bands','minority_points','enumerated_hits','recovery_selects','common_forward_pieces')})


def small():
    counts=Counter()
    for n in range(1,7):
        for seq in product(range(2),repeat=n):
            last=bytes(seq)
            f=sorted(range(n),key=last.__getitem__)
            runs=[(c,sum(1 for _ in g)) for c,g in groupby(seq)]
            idx=RunIndex(runs)
            for a in range(n):
                for b in range(a+1,n+1):
                    u=(a,b)
                    words=arcs(last,f,*u)
                    # Include every positive hit spelling anywhere, independent of the cut.
                    universe={w[i:] for w in words for i in range(len(w))}|{b'x'}
                    for h in (0,1,3,n):
                        for core in sorted(universe):
                            row=verify(idx,last,f,u,words,core,h,range(n))
                            counts.update(objects=1,marks=n,second_partners=row['chosen'],
                                positive_holes=row['holes']>0,minority_hits=row['enumerated_hits'],
                                absent=row['full_mass']==0,empty_selection=row['chosen']==0,
                                nonconvex_landing=row['landing_components']>1)
    print(json.dumps(counts,indent=2))


def real(tapes):
    writer=None
    fixed_word=b'":"' + b'value","property'
    assert len(fixed_word)==19
    for tape in paths(tapes):
        runs,mark=read_input(tape)
        last=b''.join(bytes([c])*w for c,w in runs)
        f=sorted(range(len(last)),key=last.__getitem__)
        col=Column(runs)
        idx=RunIndex(runs)
        quote=col.interval(b'"')
        for name,u in [('quote',quote),('frequent',heavy_set(col,quote,1)[0][0])]:
            words=arcs(last,f,*u)
            for h in (4,8,12):
                core=b',' if name=='quote' else fixed_word[h:]
                row=dict(ordinal=int(tape.name.split('.')[0]),section=name,h=h,core_hex=core.hex(),
                         **verify(idx,last,f,u,words,core,h,[mark]))
                if writer is None:
                    writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
                    writer.writeheader()
                writer.writerow(row)
                sys.stdout.flush()


if __name__=='__main__':
    real(sys.argv[1]) if len(sys.argv)>1 else small()
