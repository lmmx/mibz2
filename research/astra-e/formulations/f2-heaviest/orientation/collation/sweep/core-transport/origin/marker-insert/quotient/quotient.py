"""Merge one whole old source bucket with its physical marker rows, directly in RLE."""
from bisect import bisect_right
from collections import Counter
from itertools import groupby,product
from pathlib import Path
import csv
import ctypes as ct
import json
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent))
from classes import forward
from insert import Atlas,Byte,Column,Object,Oracle,Owned,Piece,U32,append_run,incoming,library,paths,read_input


class Quotient:
    def __init__(self,raw,words,order,freq,selected,z):
        assert z in selected and all(words[k].endswith(words[z]) for k in selected)
        self.freq=freq
        self.selected=selected
        self.z=z
        self.raw=raw
        self.words=words
        self.ids={k:i for i,k in enumerate(order)}
        self.order=order
        self.s=sum(freq[k] for k in selected)
        self.extra=self.s-freq[z]
        self.base={}
        self.weights=[]
        x=0
        for k in order:
            self.base[k]=x
            w=words[k]
            self.weights.append(w[:-len(words[z])] if k!=z and k in selected else w)
            x+=freq[k]
        self.m=x
        self.cut=self.base[z]
        self.starts=[self.base[k] for k in order]
        self.bases=[self.base[k]+(self.extra if self.base[k]>self.cut else 0) for k in order]
        streams=[[],[],[]]
        before,middle,after=streams
        filt=[]
        at=0
        self.entry_runs=0
        for k,width in raw:
            if k in selected:
                append_run(filt,k,width)
                self.entry_runs+=k!=z
            label=self.ids[z if k in selected else k]
            until=at+width
            for target,a,b in ((before,0,self.cut),(middle,self.cut,self.cut+freq[z]),
                               (after,self.cut+freq[z],self.m)):
                w=min(b,until)-max(at,a)
                if w>0:
                    append_run(target,label,w)
            at=until
        # Independently count the transition change at each insertion gap.
        # Changed IDs are absent from the relabeled original A.
        self.relabels_lost=len(raw)-sum(map(len,streams))
        for left,right in ((before,middle),(middle,after)):
            self.relabels_lost+=bool(left and right and left[-1][0]==right[0][0])
        self.hazard_runs=self.equal_gaps=self.gaps=self.interior_zero_gaps=0
        pending=False
        consumed=0
        window=remaining=0
        for k,width in filt:
            if k==z:
                pending=False
                consumed+=width
                continue
            self.hazard_runs+=1
            if pending:
                continue
            pending=True
            self.gaps+=1
            self.interior_zero_gaps+=0<consumed<freq[z]
            gap=self.cut+consumed
            if not 0<gap<self.m:
                continue
            if gap==self.cut:
                equal=before[-1][0]==middle[0][0]
            elif consumed==freq[z]:
                equal=middle[-1][0]==after[0][0]
            else:
                while remaining<consumed:
                    remaining+=middle[window][1]
                    window+=1
                equal=remaining!=consumed
            self.equal_gaps+=equal
        self.filtered_runs=len(filt)
        self.window_runs=len(middle)
        j=used=0
        self.root=before
        for k,width in filt:
            if k!=z:
                append_run(self.root,self.ids[k],width)
                continue
            while width:
                old,remaining=middle[j]
                take=min(width,remaining-used)
                append_run(self.root,old,take)
                width-=take
                used+=take
                if used==remaining:
                    j+=1
                    used=0
        assert j==len(middle)
        for k,width in after:
            append_run(self.root,k,width)
        assert sum(w for _,w in self.root)==self.m+self.extra
        assert len(self.root)==len(raw)-self.relabels_lost+self.hazard_runs+self.equal_gaps
        assert self.relabels_lost<=self.hazard_runs+self.interior_zero_gaps
        assert -self.interior_zero_gaps<=len(self.root)-len(raw)<=2*self.entry_runs
        if not self.extra:
            assert self.root==[(self.ids[k],w) for k,w in raw] and self.weights==[words[k] for k in order]

    def mark(self,x,offset):
        i=bisect_right(self.starts,x)-1
        k=self.order[i]
        if not self.extra:
            return x,offset,0
        prefix=len(self.words[k])-len(self.words[self.z])
        if k in self.selected and offset>=prefix:
            slot=self.base[k]
            rank=0
            for j,(t,w) in enumerate(self.raw):
                if t==k:
                    if x<slot+w:
                        return self.cut+rank+x-slot,offset-prefix,j+1
                    slot+=w
                if t in self.selected:
                    rank+=w
            raise AssertionError('unassigned selected mark')
        return x+(self.extra if x>=self.cut+self.freq[self.z] else 0),offset,0

    def abi(self,lib,start,skip,n):
        arena=bytearray()
        offsets=[]
        for w in self.weights:
            offsets.append(Piece(len(arena),len(w)))
            arena.extend(w)
        book=(Piece*len(offsets))(*offsets)
        base=(U32*len(book))(*self.bases)
        data=(Byte*len(arena)).from_buffer(arena)
        runs=(Piece*len(self.root))(*(Piece(k,w) for k,w in self.root))
        obj=Object(runs,len(runs),book,base,len(book),data,len(arena),start,skip,self.m+self.extra,n)
        taken=U32()
        out=lib.consume(ct.byref(obj),ct.byref(taken))
        try:
            return ct.string_at(out.data,out.len),int(taken.value)
        finally:
            lib.release(out)


def verify(q):
    # Independent dense deletion: form the expanded two-step graph, identify the
    # coincident zero-prefix vertices with their markers, then rename in source order.
    raw=[(q.ids[k],w) for k,w in q.raw]
    old=forward(raw,q.starts)
    destinations=[d for d,k in enumerate(k for k,w in q.raw for _ in range(w)) if k in q.selected]
    marker={d:q.cut+i for i,d in enumerate(destinations)}
    embedded=[]
    for x,(y,i) in enumerate(old):
        k=q.order[i]
        if k==q.z:
            embed=marker[y]
        else:
            embed=x+(q.extra if x>q.cut else 0)
        embedded.append(embed)
    assert embedded==sorted(set(embedded))
    moved=forward(q.root,q.bases)
    for y,v in marker.items():
        assert moved[v]==(embedded[y],q.ids[q.z])
    for x,(y,i) in enumerate(old):
        k=q.order[i]
        if k==q.z:
            continue
        dest=marker[y] if k in q.selected else embedded[y]
        assert moved[embedded[x]]==(dest,i)
    return old,moved


def small(lib):
    out=Counter()
    examples={}
    for n in range(1,7):
        for last in product(range(3),repeat=n):
            order=sorted(set(last),reverse=True)
            raw=[(k,sum(1 for _ in v)) for k,v in groupby(last)]
            f=Counter(last)
            for choices in product((0,1),repeat=len(order)):
                active={k for flag,k in zip(choices,order) if flag}
                for z in sorted(active):
                    words={k:(bytes((k,)) if k!=z else b'')+(b'!' if k in active else b'?') for k in order}
                    q=Quotient(raw,words,order,f,active,z)
                    old,new=verify(q)
                    size=sum(f[k]*len(words[k]) for k in order)
                    out.update(objects=1,old_arcs=n,quotient_arcs=len(new),identity=q.extra==0)
                    for x,(_,i) in enumerate(old):
                        for offset in range(len(words[order[i]])):
                            out['checked_byte_marks']+=1
                            source=x
                            expect=bytearray()
                            while len(expect)<size+offset:
                                source,j=old[source]
                                expect.extend(words[order[j]])
                            expected=expect[offset:offset+size]
                            v,skip,_=q.mark(x,offset)
                            if lib:
                                got,_=q.abi(lib,v,skip,size)
                            else:
                                got=bytearray()
                                while len(got)<size:
                                    v,j=new[v]
                                    got.extend(q.weights[j][skip:skip+size-len(got)])
                                    skip=0
                            assert got==expected
                    delta=len(q.root)-len(q.raw)
                    if q.entry_runs and delta in (-q.entry_runs,2*q.entry_runs):
                        name='upper' if delta>0 else 'lower'
                        examples.setdefault(name,dict(last=last,order=order,selected=sorted(active),zero=z,
                                old_R=len(raw),new_R=len(q.root),changed_input_runs=q.entry_runs))
    print(json.dumps(dict(counts=out,witnesses=examples),indent=2))


def real(lib,tapes):
    writer=None
    for tape in paths(tapes):
        runs,mark=read_input(tape)
        col=Column(runs)
        o=Oracle(b''.join(bytes([b])*n for b,n in runs))
        u=col.interval(b'"')
        raw,lead,minima,count,_=incoming(col,u,Atlas(col,u,0))
        order=sorted(lead,key=minima.get)
        words={k:w[1:]+b'"' for k,w in lead.items()}
        z=next(k for k,w in words.items() if w==b',"')
        # U' is the forward image of the old quote bucket, in the same ordinal
        # order. Independently recover the previous U' and the true primary skip.
        shifted={o.forward[x]:x-u[0] for x in range(*u)}
        back=[0]*col.rows
        for i,x in enumerate(o.forward):
            back[x]=i
        x=mark
        skip=0
        while x not in shifted:
            x=back[x]
            skip+=1
            assert skip<=col.rows
        reference=bytearray()
        pos=mark
        for _ in range(col.rows):
            reference.append(o.first[pos]);pos=o.forward[pos]
        for h in (4,8,12):
            selected={k for k,w in lead.items() if w[min(h,len(w)-1):]==b','}
            q=Quotient(raw,words,order,count,selected,z)
            verify(q)
            v,byte,probes=q.mark(shifted[x],skip)
            got,taken=q.abi(lib,v,byte,col.rows)
            assert got==reference
            row=dict(ordinal=int(tape.name.split('.')[0]),h=h,m=q.m,selected=q.s,zero_count=count[z],
                     extra=q.extra,changed_types=len(selected)-1,changed_input_runs=q.entry_runs,
                     old_R=len(raw),quotient_R=len(q.root),lost=q.relabels_lost,hazard_runs=q.hazard_runs,
                     insertion_gaps=q.gaps,equal_gaps=q.equal_gaps,interior_zero_gaps=q.interior_zero_gaps,filtered_R=q.filtered_runs,
                     window_R=q.window_runs,real_skip=byte,mark_probes=probes,second=taken)
            if writer is None:
                writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
                writer.writeheader()
            writer.writerow(row)
            sys.stdout.flush()


if __name__=='__main__':
    lib=library(sys.argv[1])
    real(lib,sys.argv[2]) if len(sys.argv)>2 else small(lib)
