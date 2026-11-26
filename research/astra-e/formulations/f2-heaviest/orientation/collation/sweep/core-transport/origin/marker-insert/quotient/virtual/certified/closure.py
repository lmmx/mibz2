"""Certify entire one-query source buckets with a reverse deterministic-successor closure."""
from bisect import bisect_right
from collections import Counter,deque
from itertools import groupby,product
from pathlib import Path
import csv
import json
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'ordered'))
from cursor_order import Atlas,Column,Graph,Quotient,full,incoming,paths,read_input

def test(q,d,all_marks):
    idx=full(q)
    counts=[q.freq[k] if k!=q.z else q.s for k in q.order]
    d=q.ids[d]
    g=Graph(idx,q.bases,counts,d)
    lo=[idx.end]*len(counts);hi=[-1]*len(counts)
    for word,start,next_start in zip((w for w,lf in idx.records),idx.starts,idx.starts[1:]+[idx.end]):
        lo[word]=min(lo[word],start);hi[word]=next_start
    children=[[] for _ in counts]
    succ={}
    for c in range(len(counts)):
        p=bisect_right(q.bases,lo[c])-1
        if c!=d and hi[c]<=q.bases[p]+counts[p]:
            succ[c]=p; children[p].append(c)
    depth={d:0}
    queue=deque([d])
    while queue:
        p=queue.popleft()
        for c in children[p]:
            assert c not in depth
            depth[c]=depth[p]+1;queue.append(c)
    for c,h in depth.items():
        bucket=g.buckets[c]
        assert len(bucket)==1 and g.by_rows[c]==counts[c]
        i=bucket[0]
        if c!=d:assert g.parent[i]==g.buckets[succ[c]][0]
    # Every actual LF arc into a certified bucket except the root has exactly
    # the certified parent as its current source bucket. This covers all marks.
    splits=seg_c=seg_o=raw_removed=0
    for c in range(len(counts)):
        v=list(idx.segments(q.bases[c],q.bases[c]+counts[c]))
        if c in depth:seg_c+=len(v)
        else:seg_o+=len(v)
        for w,width,lf in v:
            assert q.bases[w]<=lf and lf+width<=q.bases[w]+counts[w]
            if w in depth and w!=d:assert c in depth and depth[w]==depth[c]+1
    # Number of original runs touching any surviving (non-certified) rank.
    i=0
    for start,end in zip(idx.starts,idx.starts[1:]+[idx.end]):
        while q.bases[i]+counts[i]<=start:i+=1
        c=i
        while c<len(counts) and q.bases[c]<end and c in depth:c+=1
        raw_removed+=c==len(counts) or q.bases[c]>=end
    true={c for c,b in enumerate(g.buckets) if len(b)==1 and g.by_rows[c]==counts[c]}
    if all_marks:
        lf=[v+i for w,width,v in idx.segments(0,idx.end) for i in range(width)]
        for row in range(idx.end):
            j=row;past=set();had_external=False
            while j not in past:
                past.add(j);c=bisect_right(q.bases,j)-1
                if c not in depth:had_external=True
                else:assert not had_external
                j=lf[j]
                if q.bases[d]<=j<q.bases[d]+counts[d]:break
    return dict(types=len(counts),rows=idx.end,root_mass=counts[d],all_runs=len(idx.records),
        one_oracle_types=len(true),one_oracle_rows=sum(counts[c] for c in true),
        certified_types=len(depth),certified_rows=sum(counts[c] for c in depth),
        root_depth=max(depth.values()),certified_pieces=seg_c,remaining_pieces=seg_o,
        bucket_split_pieces=seg_c+seg_o,whole_runs_removed=raw_removed,
        root_pieces=len(list(idx.segments(q.bases[d],q.bases[d]+counts[d]))),
        certified_nonroot_width_one=sum(counts[c]==1 for c in depth if c!=d))

def small():
    totals=Counter()
    for n in range(1,8):
        for last in product(range(3),repeat=n):
            freq=Counter(last);order=sorted(freq)
            raw=[(w,sum(1 for _ in a)) for w,a in groupby(last)]
            for d in order:
                q=Quotient(raw,{k:bytes([k]) for k in order},order,freq,{d},d)
                a=test(q,d,True)
                totals.update(objects=1,marks=n,certified_types=a['certified_types'],certified_rows=a['certified_rows'])
    print(json.dumps(totals,sort_keys=True,indent=2))

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
            q=Quotient(raw,orig,order,count,selected,d)
            row=dict(tape=int(tape.name.split('.')[0]),h=h,**test(q,d,False))
            if writer is None:writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n');writer.writeheader()
            writer.writerow(row);sys.stdout.flush()

if __name__=='__main__':
    real(sys.argv[1]) if len(sys.argv)>1 else small()
