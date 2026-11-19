"""Ordered liveness and immutable ownership using two masks per 64 original rows."""
from collections import Counter,deque
from itertools import groupby,product
from pathlib import Path
from random import Random
import csv
import json
import sys

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from anchor import Atlas,Column,Oracle,Word,expanded,forward,incoming,paths,read_input,seed

def low(x):return (x&-x).bit_length()-1

class Order:
    def __init__(self,bases,n):
        size=(n+63)//64
        self.fixed=[0]*size
        for b in bases:self.fixed[b//64]|=1<<(b%64)
        self.prefix=[]
        t=0
        for mask in self.fixed:
            self.prefix.append(t);t+=mask.bit_count()
        self.live=[(1<<64)-1]*size
        self.live[-1]=(1<<((n-1)%64+1))-1
        self.left=[i-1 for i in range(size)]
        self.right=[i+1 for i in range(size)]
        self.right[-1]=-1
    def owner(self,x):
        i,b=divmod(x,64)
        return self.prefix[i]+(self.fixed[i]&((2<<b)-1)).bit_count()-1
    def next(self,x):
        i,b=divmod(x,64)
        bits=self.live[i]&~((2<<b)-1)
        if bits:return i*64+low(bits)
        j=self.right[i]
        return j*64+low(self.live[j]) if j>=0 else -1
    def prev(self,x):
        i,b=divmod(x,64)
        bits=self.live[i]&((1<<b)-1)
        if bits:return i*64+bits.bit_length()-1
        j=self.left[i]
        return j*64+self.live[j].bit_length()-1 if j>=0 else -1
    def delete(self,x):
        i,b=divmod(x,64)
        assert self.live[i]&(1<<b)
        self.live[i]^=1<<b
        if not self.live[i]:
            a,c=self.left[i],self.right[i]
            if a>=0:self.right[a]=c
            if c>=0:self.left[c]=a

def solve(original,literals,mark,skip,horizon):
    raw,words,anchor=seed(original,[Word(w) for w in literals],mark)
    size=[0]*len(words)
    for k,width in raw:size[k]+=width
    first=[];last=[];n=0
    for mass in size:
        first.append(n);n+=mass;last.append(n-1)
    ord=Order(first,n)
    cursor=first.copy()
    edge=[0]*n
    y=0
    for k,width in raw:
        a=cursor[k]
        edge[a:a+width]=range(y,y+width)
        cursor[k]+=width
        y+=width
    assert size[anchor]==1 and first[anchor]==mark
    queued=set();ready=deque()
    counts=Counter(row_edits=0,joins=0,pops=0,checks=0)
    def choice(k):
        counts['checks']+=1
        if size[k]:
            t=ord.owner(edge[first[k]])
            if t!=k and t!=anchor and t==ord.owner(edge[last[k]]):
                return t
        return None
    def enroll(k):
        if k not in queued and choice(k) is not None:
            queued.add(k);ready.append(k)
    for k in range(len(words)):enroll(k)
    while edge[mark]!=mark:
        assert ready
        k=ready.popleft();queued.remove(k)
        counts['pops']+=1
        t=choice(k)
        if t is None:continue
        old_mass=size[k]
        words[k]=words[k]+words[t]
        x=first[k]
        for i in range(old_mass):
            dead=edge[x]
            assert ord.owner(dead)==t
            edge[x]=edge[dead]
            size[t]-=1
            if size[t]:
                if dead==first[t]:first[t]=ord.next(dead)
                if dead==last[t]:last[t]=ord.prev(dead)
            ord.delete(dead)
            if i+1<old_mass:
                x=ord.next(x)
                assert x>=0 and ord.owner(x)==k
        counts.update(row_edits=old_mass,joins=1)
        enroll(k);enroll(t)
    assert counts['row_edits']==n-sum(size)
    assert counts['pops']<=len(words)+2*counts['joins']
    out,copy=expanded(words[anchor],skip,horizon)
    counts.update(copy)
    return out,dict(counts,lap=len(words[anchor]),remaining=sum(size),
                    mutable_row_and_masks_bytes=4*n+28*((n+63)//64))

def adversarial():
    total=Counter();rng=Random(402619)
    cases=[last for n in range(1,7) for last in product(range(3),repeat=n)]
    for n in [63,64,65,127,128,129,191,192,193,511]:
        cases.extend(tuple(rng.randrange(k) for _ in range(n)) for k in [2,3,7,30,65,257])
        cases.extend([(1,)+(0,)*(n-1),tuple(range(n)),tuple(reversed(range(n)))])
    for last in cases:
        symbols=sorted(set(last),reverse=True)
        ids={k:i for i,k in enumerate(symbols)}
        raw=[(ids[k],sum(1 for _ in t)) for k,t in groupby(last)]
        f=Counter(ids[k] for k in last)
        words=[bytes([w%251])*(w%4+1) for w in range(len(f))]
        bases=[sum(f[j] for j in range(k)) for k in range(len(f))]
        edges=forward(raw,bases)
        horizon=sum(f[k]*len(w) for k,w in enumerate(words))
        marks=range(len(last)) if len(last)<=6 else sorted({0,1,62,63,len(last)//2,len(last)-1} & set(range(len(last))))
        for x in marks:
            lap=bytearray();y=x
            while True:
                y,k=edges[y];lap.extend(words[k])
                if y==x:break
            for skip in range(len(words[edges[x][1]])):
                ans,count=solve(raw,words,x,skip,horizon)
                truth=(lap*((skip+horizon+len(lap)-1)//len(lap)))[skip:skip+horizon]
                assert ans==truth and count['lap']==len(lap)
                total.update(marks=1,edits=count['row_edits'],joins=count['joins'],disjoint=int(count['remaining']>1))
    print(json.dumps(total,indent=2))

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
        x=mark;skip=0
        while x not in shifted:x=back[x];skip+=1
        got,count=solve(root,old,shifted[x],skip,col.rows)
        pos=mark
        for b in got:
            assert b==o.first[pos]
            pos=o.forward[pos]
        data=dict(ordinal=tape.name.split('.')[0],original_rows=u[1]-u[0],original_runs=len(root),**count)
        if writer is None:
            writer=csv.DictWriter(sys.stdout,fieldnames=data,lineterminator='\n');writer.writeheader()
        writer.writerow(data);sys.stdout.flush()

if __name__=='__main__':
    real(sys.argv[1]) if len(sys.argv)>1 else adversarial()
