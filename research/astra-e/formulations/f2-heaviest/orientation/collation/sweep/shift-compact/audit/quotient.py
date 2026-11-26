"""Exact descriptor normalization and independent marked rewind for the deleted domain."""
from bisect import bisect_right
from collections import Counter, defaultdict
from itertools import groupby, product
from pathlib import Path
import csv
import json
import sys


class Section:
    def __init__(self, intervals):
        self.spans = []
        for a,b in sorted(intervals):
            if self.spans and a <= self.spans[-1][1]:
                self.spans[-1][1] = max(b,self.spans[-1][1])
            elif b>a:
                self.spans.append([a,b])
        self.starts = [a for a,b in self.spans]
        self.base = []
        self.mass = 0
        for a,b in self.spans:
            self.base.append(self.mass)
            self.mass += b-a

    def ordinal(self, row):
        k=bisect_right(self.starts,row)-1
        if k<0 or self.spans[k][1]<=row:return None
        return self.base[k]+row-self.starts[k]

    def row(self, i):
        k=bisect_right(self.base,i)-1
        return self.starts[k]+i-self.base[k]

    def complement(self,n):
        lo=0;d=[]
        for a,b in self.spans:
            if lo<a:d.append((lo,a))
            lo=b
        if lo<n:d.append((lo,n))
        return Section(d)


def original(runs):
    count=[0]*256
    for c,w in runs:count[c]+=w
    total=0
    for i,w in enumerate(count):count[i]=total;total+=w
    lf=[];psi=[];lo=0
    for c,w in runs:
        y=count[c];lf.append((lo,lo+w,y-lo,c));psi.append((y,y+w,lo-y,c))
        lo+=w;count[c]+=w
    return lf,sorted(psi)


def shifted(level,psi,starts):
    out,first=[],None
    for lo,hi in level.spans:
        i=bisect_right(starts,lo)-1
        while lo<hi:
            _,b,delta,c=psi[i]
            if first is None:first=c
            if c!=first:return None,None
            stop=min(b,hi);out.append((lo+delta,stop+delta));lo=stop;i+=1
    assert out==sorted(out)
    return Section(out),first


def table(lf,levels):
    n=lf[-1][1];v=levels[-1]
    forbidden=Section(x for s in levels[1:] for x in s.spans)
    d=forbidden.complement(n)
    arrivals=sorted((lo,hi,tag,rank) for tag,s in ((0,d),(1,v))
                    for (lo,hi),rank in zip(s.spans,s.base))
    lefts=[a for a,b,t,k in arrivals]
    # Exact effective boundaries between adjacent destination namespaces.
    interfaces={z[0] for y,z in zip(arrivals,arrivals[1:]) if y[1]==z[0] and y[2]!=z[2]}
    raw=extra=source_pieces=0;segments=[]
    original_seams={r[0] for r in lf[1:]}
    original_starts=[r[0] for r in lf]
    for (lo,hi),offset in zip(d.spans,d.base):
        i=bisect_right(original_starts,lo)-1;begin=lo
        while lo<hi:
            _,run_end,delta,c=lf[i];stop=min(hi,run_end);y=lo+delta
            at=bisect_right(lefts,y)-1
            assert at>=0
            source_pieces+=1
            while lo<stop:
                a,b,tag,base=arrivals[at]
                assert a<=y<b
                end=min(stop,lo+b-y)
                s=offset+lo-begin;target=base+y-a;raw+=1
                if segments and segments[-1][3:]==[c,tag] and segments[-1][1]==s and segments[-1][2]+s-segments[-1][0]==target:
                    segments[-1][1]=s+end-lo
                else:
                    segments.append([s,s+end-lo,target,c,tag])
                y+=end-lo;lo=end
                if lo<stop:
                    assert y in interfaces
                    extra+=1;at+=1
            i+=1
    seams=sum(d.ordinal(x) is not None and d.ordinal(x-1) is not None for x in original_seams)
    assert raw==source_pieces+extra==len(d.spans)+seams+extra
    # Same-byte consecutive physical preimages must both survive.
    predicted=0
    # Build source-run images sorted by raw destination rank, then select straddlers.
    images=sorted((a+delta,b+delta,a) for a,b,delta,c in lf)
    image_starts=[a for a,b,x in images]
    for e in interfaces:
        a,b,x=images[bisect_right(image_starts,e)-1]
        if a<e<b and d.ordinal(x+e-a) is not None and d.ordinal(x+e-a-1) is not None:
            predicted+=1
    assert predicted==extra
    assert source_pieces<=len(lf)+len(forbidden.spans)
    assert extra<=len(interfaces)<=2*min(len(forbidden.spans),len(v.spans))
    assert raw-len(segments)<=max(0,len(d.spans)-1)
    stats=dict(r=len(lf),forbidden_components=len(forbidden.spans),complement_components=len(d.spans),
               target_components=len(v.spans),complement_rows=d.mass,retained_raw_L_seams=seams,
               original_run_intersections=source_pieces,physical_interfaces=len(interfaces),
               effective_interior_arrival_cuts=extra,physical_descriptors=raw,gap_coalescences=raw-len(segments),
               normalized_descriptors=len(segments),upper=len(lf)+len(forbidden.spans)+2*min(len(forbidden.spans),len(v.spans)))
    return forbidden,d,segments,stats


def dense(last,levels,prefix,lf,psi,tallies):
    q=len(prefix);n=len(last);v=levels[-1]
    forbidden,d,segments,geometry=table(lf,levels)
    starts=[a for a,b,z,c,t in segments]; lf_dense=[None]*n
    for a,b,delta,c in lf:
        for x in range(a,b):lf_dense[x]=x+delta
    def step(i):
        a,b,target,c,t=segments[bisect_right(starts,i)-1]
        assert a<=i<b
        return target+i-a,c,t
    for i in range(d.mass):
        x=d.row(i);y=lf_dense[x];z,c,t=step(i)
        assert last[x]==c and ((t==1 and z==v.ordinal(y)) or (t==0 and z==d.ordinal(y)))
    # Derive each exact arc again solely from early ordinals and quotient descriptors.
    for i in range(v.mass):
        for j in range(1,q+1):
            dest=v.ordinal(levels[q-j].row(i))
            if dest is not None:
                word=prefix[q-j:];tallies["early_arcs"]+=1;break
        else:
            z=d.ordinal(levels[0].row(i));out=[]
            assert z is not None
            while True:
                z,c,t=step(z);out.append(c)
                if t:dest=z;break
            word=list(reversed(out))+prefix
        row=v.row(i);expected=[]
        while True:
            expected.append(last[row]);row=lf_dense[row]
            if v.ordinal(row) is not None:break
        assert word==list(reversed(expected)) and dest==v.ordinal(row)
        tallies["arcs"]+=1
    # Each virtual rewind chooses the largest possible j; overlapping-layer ambiguity is harmless.
    for p in range(n):
        x=p;depth=0
        while forbidden.ordinal(x) is not None and v.ordinal(x) is None:
            j=next(j for j in range(q-1,0,-1) if levels[j].ordinal(x) is not None)
            prev=levels[j-1].row(levels[j].ordinal(x))
            assert prev==lf_dense[x]
            x=prev;depth+=1
        assert depth<=q-1
        tallies["max_virtual_rewinds"]=max(tallies["max_virtual_rewinds"],depth)
        tallies["tight_nonzero_virtual_rewinds"]+=depth>0 and depth==q-1
        measured=depth;dest=v.ordinal(x)
        if dest is None:
            z=d.ordinal(x);orig=z
            while True:
                z,c,t=step(z);measured+=1
                if t:dest=z;break
                if z==orig:
                    assert depth==0
                    measured=None;tallies["disjoint_marks"]+=1;break
        y=p;want=0
        while v.ordinal(y) is None:
            y=lf_dense[y];want+=1
            if y==p:want=None;break
        assert measured==want and dest==v.ordinal(y)
        tallies["marks"]+=1
    tallies["cases"]+=1
    tallies["empty_complement"]+=d.mass==0
    return geometry


def small():
    totals=Counter()
    for n in range(1,8):
        for last in product((0,1),repeat=n):
            runs=[(c,len(list(group))) for c,group in groupby(last)]
            lf,psi=original(runs);starts=[v[0] for v in psi]
            for a in range(n):
                for b in range(a+1,n+1):
                    levels=[Section([(a,b)])];prefix=[]
                    for q in range(1,5):
                        image,c=shifted(levels[-1],psi,starts)
                        if image is None:break
                        levels.append(image);prefix.append(c)
                        dense(last,levels,prefix,lf,psi,totals)
    print(json.dumps(totals,sort_keys=True,indent=2))


def real(tapes):
    here=Path(__file__).resolve().parent
    root=here.parents[8]
    sys.path.insert(0,str(root/"docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import paths,read_input
    fixtures={int(r["ordinal"]):r for r in csv.DictReader((here.parents[1]/"shift-constructor/inputs.csv").open())}
    writer=None
    for tape in paths(tapes):
        ordinal=int(tape.name.split(".")[0]);f=fixtures[ordinal]
        runs,_=read_input(tape);lf,psi=original(runs);starts=[v[0] for v in psi]
        levels=[Section([(int(f["lo"]),int(f["hi"]))])]
        for q in range(1,4):
            image,c=shifted(levels[-1],psi,starts)
            if image is None:break
            levels.append(image)
            forbidden,d,seg,stats=table(lf,levels)
            row=dict(ordinal=ordinal,h=q,**stats)
            if writer is None:
                writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator="\n");writer.writeheader()
            writer.writerow(row)
        sys.stdout.flush()


if __name__=="__main__":
    real(Path(sys.argv[1])) if len(sys.argv)>1 else small()
