"""Charge the certifying frontier when homogeneous source buckets keep their stock unary contraction."""
from bisect import bisect_right
from collections import Counter
from itertools import groupby,product
from pathlib import Path
import csv
import json
import sys
from closure import Atlas,Column,Graph,Quotient,full,incoming,paths,read_input
from bridges import check,profile

def audit(q,d):
    d=q.ids[d];prof=profile(q);freq,idx,pieces,_,lo,hi=prof
    head=[[] for _ in freq]
    for c in range(len(freq)):
        p=bisect_right(q.bases,lo[c])-1
        if hi[c]<q.bases[p]+freq[p] and c!=d:head[p].append(c)
    base_graph=Graph(idx,q.bases,freq,d)
    unary=set()
    logical=Counter()
    for w,n in enumerate(freq):
        part=list(idx.segments(q.bases[w],q.bases[w]+n))
        if len(part)==1 and part[0][0]!=d:unary.add(w)
    # Count unary steps of the ordinary contracted trie for comparison with native node counts.
    todo=[(d,q.bases[d],q.bases[d]+freq[d])]
    while todo:
        c,a,b=todo.pop()
        seg=list(idx.segments(a,b));children={}
        logical['queries']+=1
        if len(seg)==1 and seg[0][0]!=d:logical['unary']+=1
        for w,n,l in seg:
            if w==d:continue
            first,stop=children.get(w,(l,l));assert stop==l
            children[w]=(first,l+n)
        todo.extend((w,x,y) for w,(x,y) in children.items())
    assert logical['queries']==base_graph.n
    true_n=base_graph.n-logical['unary']
    out=[]
    for name in ('root','depth1','branch_prefix','no_unary','full'):
        bucket={d};todo=[d]
        while todo:
            p=todo.pop()
            for c in head[p]:
                if name=='root':continue
                if name=='depth1' and (p!=d or pieces[c]<2):continue
                if name=='branch_prefix' and pieces[c]<2:continue
                if name=='no_unary' and c in unary:continue
                bucket.add(c);todo.append(c)
        props=check(q,bucket,prof)
        old_n=logical['queries'];extra=len(bucket&unary)
        assert all(len(base_graph.buckets[i])==1 and base_graph.by_rows[i]==freq[i] for i in bucket)
        out.append(dict(policy=name,types=len(bucket),rows=sum(freq[x] for x in bucket),
            rank_records=props['remaining'],rank_pages=(props['rows']+63)//64,
            finds=old_n-len(bucket),canonical_nodes=true_n,extra_nodes=extra,
            physical_nodes=true_n+extra))
    return out

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
            for one in audit(Quotient(raw,orig,order,count,selected,d),d):
                row=dict(tape=int(tape.name.split('.')[0]),h=h,**one)
                if writer is None:writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n');writer.writeheader()
                writer.writerow(row);sys.stdout.flush()
if __name__=='__main__':real(sys.argv[1])
