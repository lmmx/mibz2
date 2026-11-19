"""Strip a bounded prefix of each original return, stopping one row before its next visit."""
from bisect import bisect_right
from collections import Counter, defaultdict
from itertools import groupby, product
from pathlib import Path
import csv
import hashlib
import json
import sys

here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parent/'context'))
from shifts import Column, Oracle, heavy_set, suffixes, truth


def intervals_union(pieces):
    merged=[]
    for a,b in sorted(pieces):
        if merged and a<=merged[-1][1]:
            assert a==merged[-1][1]
            merged[-1]=(merged[-1][0],b)
        else:
            merged.append((a,b))
    return merged


class Atlas:
    def __init__(self,col,section,maximum):
        # Each packet is source-base, current physical base, width, shared prefix.
        a,b=section
        strips=sorted((x+t, y+t, -t, c) for x,y,t,c in
                      zip(col.starts,col.starts[1:],col.maps,col.symbols))
        ends=[y for x,y,t,c in strips]
        active=[(a,a,b-a,b'')]
        terminal=[]
        hidden=[]
        scans=short=0
        self.levels=[]
        for depth in range(maximum):
            forward=[]
            for base,lo,width,prefix in active:
                pos=lo
                idx=bisect_right(ends,pos)
                while pos<lo+width:
                    left,right,delta,c=strips[idx]
                    switch=min(lo+width,right)
                    start=pos+delta
                    end=switch+delta
                    split=sorted({start,end,max(start,min(end,a)),max(start,min(end,b))})
                    for u,v in zip(split,split[1:]):
                        off=u-delta-lo
                        if a<=u<b:
                            terminal.append((base+off,lo+off,v-u,prefix))
                            short+=1
                        else:
                            forward.append((base+off,u,v-u,prefix+bytes([c])))
                            hidden.append((u,v))
                    pos=switch
                    idx+=1
                    scans+=1
            self.levels.append(len(forward))
            active=forward
            if not active:
                break
        terminal.extend(active)
        self.leaves=sorted(terminal,key=lambda p:p[1])
        self.bases=[lo for _,lo,_,_ in self.leaves]
        assert sum(w for _,_,w,_ in self.leaves)==b-a
        assert all(lo+w<=p[1] for (_,lo,w,_),p in zip(self.leaves,self.leaves[1:]))
        self.hidden=intervals_union(hidden)
        self.census=dict(forward_intersections=scans,packet_steps=sum(self.levels),terminal_bands=len(self.leaves),
                         early_bands=short,hidden_components=len(self.hidden),hidden_rows=sum(y-x for x,y in self.hidden),
                         prefix_types=len({p for _,_,w,p in self.leaves}),
                         prefix_dictionary_bytes=sum(map(len,{p for _,_,w,p in self.leaves})))
        # Straight physical predecessor query for a terminal, for the checking adapter.
    def terminal(self,x):
        i=bisect_right(self.bases,x)-1
        if i<0:
            return None
        base,lo,w,p=self.leaves[i]
        return (base+x-lo,p) if x<lo+w else None


def check(col,oracle,sec,h,full):
    a,b=sec
    geo=Atlas(col,sec,h)
    words,incoming=truth(oracle,sec)
    labels=set();cores=set()
    for word,cells in words.items():
        depth=min(h,len(word)-1)
        core=word[depth:]
        prefix=word[:depth]
        cores.add(core)
        labels.add((prefix,core))
        for origin,dest in cells:
            p=origin
            for _ in range(depth):
                p=oracle.forward[p]
            assert geo.terminal(p)==(origin,prefix)
    assert len(labels)==len(words)
    if full:
        lf=[0]*len(oracle.forward)
        for i,v in enumerate(oracle.forward):
            lf[v]=i
        for dest in range(a,b):
            p=dest
            rev=bytearray()
            while True:
                p=lf[p]
                rev.append(oracle.first[p])
                term=geo.terminal(p)
                if term is not None:
                    base,prefix=term
                    break
            actual=prefix+rev[::-1]
            assert actual==incoming[dest-a] and (base,dest) in words[actual]
    return dict(mass=b-a,types=len(words),core_types=len(cores),
                original_D=sum(map(len,words)),core_D=sum(map(len,cores)),original_Q=suffixes(words),
                core_Q=suffixes(cores),**geo.census)


def small():
    stat=Counter();witness=None
    for n in range(1,8):
        for raw in product(range(2),repeat=n):
            oracle=Oracle(bytes(raw))
            col=Column([(c,sum(1 for _ in v)) for c,v in groupby(raw)])
            for a in range(n):
                for b in range(a+1,n+1):
                    for h in (0,1,3,n):
                        out=check(col,oracle,(a,b),h,True)
                        stat['section_depths']+=1
                        stat['individually_verified_arcs']+=b-a
                        stat['early']+=out['early_bands']>0
                        if out['early_bands'] and out['hidden_rows'] and out['types']>1 and witness is None:
                            witness=dict(last=bytes(raw).hex(),section=[a,b],h=h,**out)
    print(json.dumps(dict(counts=stat,witness=witness),indent=2))


def real(tapes):
    sys.path.insert(0,str(here.parents[7]/'docs/journal/2026-11-18-astra-c-output-first'))
    from inputs import read_input,paths
    writer=None
    for tape in paths(tapes):
        runs,_=read_input(tape)
        col=Column(runs);oracle=Oracle(b''.join(bytes([c])*n for c,n in runs))
        quote=col.interval(b'"');deep=heavy_set(col,quote,1)[0][0]
        for name,sec,hs in (('quote',quote,(3,8,16)),('frequent',deep,(3,8,16))):
            for h in hs:
                row=dict(ordinal=int(tape.name.split('.')[0]),section=name,cap=h,**check(col,oracle,sec,h,False))
                if writer is None:
                    writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
                    writer.writeheader()
                writer.writerow(row);sys.stdout.flush()


if __name__=='__main__':
    real(Path(sys.argv[1])) if len(sys.argv)>1 else small()
