"""Build a marked singleton with two stream cuts; finish its orbit as a shared grammar."""
from bisect import bisect_right
from collections import Counter
from itertools import groupby,product
from pathlib import Path
import csv
import json
import sys

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from fold import Atlas,Column,Fold,Oracle,append_run,forward,incoming,paths,read_input


class Word:
    __slots__=('n','literal','left','right')
    def __init__(self,literal,left=None,right=None):
        self.literal=literal
        self.left=left
        self.right=right
        self.n=len(literal) if literal is not None else left.n+right.n
    def __len__(self):return self.n
    def __add__(self,right):
        return self if right==b'' else Word(None,self,right)


def seed(raw,weights,x):
    freq=Counter()
    for k,w in raw:freq[k]+=w
    bounds=[0]
    for k in range(len(weights)):bounds.append(bounds[-1]+freq[k])
    old=bisect_right(bounds,x)-1
    ranges=[(lo,hi,lo==x) for lo,hi in zip((bounds[old],x,x+1),(x,x+1,bounds[old+1])) if lo<hi]
    alias=[*weights[:old],*[weights[old]]*len(ranges),*weights[old+1:]]
    a=old+next(i for i,(_,_,is_anchor) in enumerate(ranges) if is_anchor)
    pos=bounds[old]
    part=0
    root=[]
    for k,w in raw:
        if k!=old:
            append_run(root,k+(len(ranges)-1 if k>old else 0),w)
            continue
        while w:
            take=min(w,ranges[part][1]-pos)
            append_run(root,old+part,take)
            pos+=take;w-=take
            if pos==ranges[part][1]:part+=1
    assert len(root)<=len(raw)+2 and len(alias)<=len(weights)+2
    return root,alias,a


def expanded(root,skip,horizon):
    # A cached address refers to a complete spelling already present in the output.
    # Partial nodes are split using byte lengths and are never announced as full.
    out=bytearray()
    cache={}
    metrics=Counter()
    def span(lo,hi):
        stack=[(root,lo,hi,False)]
        while stack:
            w,a,b,close=stack.pop()
            if close:
                cache[w]=len(out)-w.n
                continue
            metrics['visits']+=1
            if w.literal is not None:
                out.extend(w.literal[a:b])
                metrics['literal_bytes']+=b-a
            elif w in cache:
                start=cache[w]
                out.extend(out[start+a:start+b])
                metrics['copied_bytes']+=b-a
            else:
                if a==0 and b==w.n:
                    stack.append((w,0,0,True))
                n=w.left.n
                if b>n:stack.append((w.right,max(a-n,0),b-n,False))
                if a<n:stack.append((w.left,a,min(b,n),False))
            metrics['stack_peak']=max(metrics['stack_peak'],len(stack))
    span(skip,min(root.n,skip+horizon))
    while len(out)<horizon:
        span(0,min(root.n,horizon-len(out)))
    assert len(out)==horizon
    return bytes(out),metrics


def solve(raw,words,mark,skip,horizon):
    weights=[Word(w) for w in words]
    raw,weights,a=seed(raw,weights,mark)
    counts=Counter(rounds=0,created=len(words),scanned_runs=0,scanned_ids=0)
    while True:
        freq=Counter()
        dst=None
        m=0
        for k,n in raw:
            freq[k]+=n
            if k==a:
                assert dst is None and n==1
                dst=m
            m+=n
        assert dst is not None
        source=sum(freq[k] for k in range(a))
        if dst==source:
            break
        order=list(range(len(weights)))
        f=Fold(raw,dict(enumerate(weights)),order,freq,protected=a)
        assert f.chosen and f.m<m and a in f.ids
        counts.update(rounds=1,created=len(f.chosen),scanned_runs=len(raw),scanned_ids=len(weights))
        raw,weights,a=f.root,f.weights,f.ids[a]
    result,exp=expanded(weights[a],skip,horizon)
    counts.update(exp)
    counts.update(lap=len(weights[a]),terminal_rows=m,terminal_ids=len(weights))
    return result,counts


def small():
    c=Counter()
    for n in range(1,8):
        for l in product(range(3),repeat=n):
            ids={k:i for i,k in enumerate(sorted(set(l),reverse=True))}
            raw=[(ids[k],sum(1 for _ in t)) for k,t in groupby(l)]
            f=Counter(ids[k] for k in l)
            words=[bytes([j+31])*(j+1) for j in ids.values()]
            bounds=[sum(f[j] for j in range(k)) for k in range(len(f))]
            edges=forward(raw,bounds)
            horizon=sum(f[k]*len(w) for k,w in enumerate(words))
            for pos in range(n):
                y=pos
                expected=bytearray()
                while len(expected)<horizon+3:
                    y,k=edges[y];expected.extend(words[k])
                lap=0
                y=pos
                while True:
                    y,k=edges[y];lap+=len(words[k])
                    if y==pos:break
                for skip in range(len(words[edges[pos][1]])):
                    out,count=solve(raw,words,pos,skip,horizon)
                    assert out==expected[skip:skip+horizon] and count['lap']==lap
                    c.update(marks=1,disjoint=int(lap<horizon),proper_subperiod=int(horizon%lap!=0),
                             terminal_other_rows=int(count['terminal_rows']>1),rounds=count['rounds'])
    print(json.dumps(c,indent=2))


def real(tapes):
    writer=None
    for tape in paths(tapes):
        runs,mark=read_input(tape)
        col=Column(runs)
        o=Oracle(b''.join(bytes([b])*n for b,n in runs))
        u=col.interval(b'"')
        raw,words,minima,_,_=incoming(col,u,Atlas(col,u,0))
        order=sorted(words,key=minima.get)
        ids={k:i for i,k in enumerate(order)}
        root=[(ids[k],w) for k,w in raw]
        old=[words[k][1:]+b'"' for k in order]
        shifted={o.forward[x]:x-u[0] for x in range(*u)}
        back=[0]*col.rows
        for i,x in enumerate(o.forward):back[x]=i
        x=mark
        skip=0
        while x not in shifted:x=back[x];skip+=1
        got,counts=solve(root,old,shifted[x],skip,col.rows)
        expect=bytearray()
        pos=mark
        for _ in range(col.rows):
            expect.append(o.first[pos]);pos=o.forward[pos]
        assert got==expect
        row=dict(ordinal=tape.name.split('.')[0],original_rows=u[1]-u[0],original_skip=skip,**counts)
        if writer is None:
            writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
            writer.writeheader()
        writer.writerow(row);sys.stdout.flush()


if __name__=='__main__':
    real(sys.argv[1]) if len(sys.argv)>1 else small()
