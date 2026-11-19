"""Index exactly which adjacent retained source buckets can share an LF record."""
from bisect import bisect_right
from collections import Counter
from itertools import groupby,product
from pathlib import Path
import csv
import json
import sys
from closure import Atlas,Column,Quotient,full,incoming,paths,read_input

def profile(q):
    idx=full(q)
    freq=[q.freq[k] if k!=q.z else q.s for k in q.order]
    r=[];left=[];right=[];owner_min=[idx.end]*len(freq);owner_max=[0]*len(freq)
    at=0
    for c,width in q.root:
        owner_min[c]=min(owner_min[c],at);at+=width;owner_max[c]=at-1
    for c,n in enumerate(freq):
        parts=list(idx.segments(q.bases[c],q.bases[c]+n));r.append(len(parts))
        a,_,x=parts[0];b,w,y=parts[-1]
        left.append((a,x));right.append((b,y+w))
    starts={key:i for i,key in enumerate(left)}
    assert len(starts)==len(freq)
    mates=[starts[k] if k in starts else None for k in right]
    assert all(m is None or m>i for i,m in enumerate(mates))
    return freq,idx,r,mates,owner_min,owner_max

def check(q,cuts,prof):
    freq,idx,counts,mates,_,_=prof
    order=[c for c in range(len(freq)) if c not in cuts]
    spans=sum(counts[c] for c in order)
    bridges=[(a,b) for a,b in zip(order,order[1:]) if mates[a]==b]
    # Independent streamed-column reference with the original LF translation.
    records=0;pending=None;rows=0
    for c in order:
        for w,width,x in idx.segments(q.bases[c],q.bases[c]+freq[c]):
            if pending!=(w,x):records+=1
            pending=(w,x+width);rows+=width
    assert records==spans-len(bridges)
    return dict(rows=rows,raw_runs=len(idx.records),remaining=records,within_bucket_records=spans,
       usable_bridges=len(bridges),long_bridges=sum(b>a+1 for a,b in bridges),
       potential_nonadjacent=sum(b is not None and b>a+1 for a,b in enumerate(mates)))

def small():
    total=Counter()
    for n in range(1,7):
        for l in product(range(3),repeat=n):
            freq=Counter(l);order=sorted(freq);raw=[(c,sum(1 for _ in v)) for c,v in groupby(l)]
            z=order[0];q=Quotient(raw,{c:bytes([c]) for c in order},order,freq,{z},z)
            prof=profile(q)
            for s in product((False,True),repeat=len(order)):
                a=check(q,{i for i,b in enumerate(s) if b},prof)
                total.update(subsets=1,remaining_rows=a['rows'],remaining_records=a['remaining'],long=a['long_bridges'])
    print(json.dumps(total,indent=2,sort_keys=True))

def real(tapes):
    writer=None
    for tape in paths(tapes):
        rs,_=read_input(tape);col=Column(rs);u=col.interval(b'"')
        raw,lead,mins,count,_=incoming(col,u,Atlas(col,u,0));order=sorted(lead,key=mins.get)
        orig={k:w[1:]+b'"' for k,w in lead.items()};z=next(k for k,w in orig.items() if w==b',"')
        for h in (0,4,8,12):
            if h:
                selected={k for k,w in lead.items() if w[min(h,len(w)-1):]==b','};d=z
            else:
                d=max(order,key=lambda k:(count[k],-mins[k]));selected={d}
            q=Quotient(raw,orig,order,count,selected,d);prof=profile(q)
            freq,_,_,_,f,l=prof
            head=[[] for _ in freq]
            for i in range(len(freq)):
                p=bisect_right(q.bases,f[i])-1
                if l[i]<q.bases[p]+freq[p] and i!=q.ids[d]:head[p].append(i)
            cut={q.ids[d]};todo=list(cut)
            while todo:
                p=todo.pop()
                for c in head[p]:todo.append(c);cut.add(c)
            row=dict(tape=int(tape.name.split('.')[0]),h=h,**check(q,cut,prof))
            if writer is None:writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n');writer.writeheader()
            writer.writerow(row);sys.stdout.flush()

if __name__=='__main__':real(sys.argv[1]) if len(sys.argv)>1 else small()
