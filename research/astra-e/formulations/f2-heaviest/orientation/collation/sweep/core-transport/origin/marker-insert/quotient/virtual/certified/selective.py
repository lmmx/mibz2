"""Construct the exact whole-bucket certificates with only demand-built per-word select lists."""
from bisect import bisect_right
from collections import Counter,deque
from itertools import groupby,product
from pathlib import Path
import csv
import json
import sys
from closure import Atlas,Column,Graph,Quotient,full,incoming,paths,read_input

def inspect(q,d):
    idx=full(q);counts=[q.freq[k] if k!=q.z else q.s for k in q.order];k=len(counts);d=q.ids[d]
    first=[idx.end]*k;last=[0]*k
    x=0
    for c,w in q.root:
        first[c]=min(first[c],x);x+=w;last[c]=x-1
    children=[[] for _ in counts]
    for c in range(k):
        if c==d:continue
        p=bisect_right(q.bases,first[c])-1
        if last[c]<q.bases[p]+counts[p]:children[p].append(c)
    known={d:0};work=deque([d])
    while work:
        p=work.popleft()
        for c in children[p]:
            known[c]=known[p]+1;work.append(c)
    initial=len(known)
    # Unknown start c terminates in a decision or an edge to the first full
    # pair encountered. All interior states of these segments are disjoint.
    links={};periodic=set();false=set()
    tasks=[(c,c,q.bases[c],q.bases[c]+counts[c]-1,0) for c in range(k) if c not in known]
    table={};passes=pieces=positions=probes=steps=0;rows_seen=0
    while tasks:
        wait=[];needed=set()
        for c,owner,x,y,depth in tasks:
            while True:
                base=q.bases[owner];end=base+counts[owner]-1
                interior=[v for v in {x,y} if base<v<end]
                if interior and owner not in table:
                    wait.append((c,owner,x,y,depth));needed.add(owner);break
                def advance(v):
                    nonlocal probes
                    if v==base:return first[owner]
                    if v==end:return last[owner]
                    probes+=1
                    src,dest=table[owner];i=bisect_right(src,v)-1
                    return dest[i]+v-src[i]
                if x==y:x=y=advance(x)
                else:x,y=advance(x),advance(y)
                steps+=1;depth+=1
                p=bisect_right(q.bases,x)-1
                if y>=q.bases[p]+counts[p]:false.add(c);break
                if p in known:known[c]=depth+known[p];break
                if x==q.bases[p] and y==x+counts[p]-1:links[c]=(p,depth);break
                if depth==idx.end:periodic.add(c);break
                owner=p
        tasks=wait
        if not needed:break
        passes+=1
        # Build only actually blocked class lists, once. Start-over scans are charged.
        idxs={w:([],[]) for w in needed}
        seen={w:q.bases[w] for w in needed};pos=0
        for c,width in q.root:
            if c in needed:
                src,dst=idxs[c];src.append(seen[c]);dst.append(pos)
                seen[c]+=width;pieces+=1
            pos+=width
        table.update(idxs);positions+=sum(counts[w] for w in needed)
        rows_seen+=len(q.root)
    # Resolve start-to-start dependencies. A directed cycle of these positive
    # length links never visits d, because every link stopped before any positive certificate.
    for c in links:
        if c in known or c in false:continue
        path=[];seen=set();v=c
        while v in links and v not in known and v not in false and v not in seen:
            seen.add(v);path.append(v);v=links[v][0]
        if v in known:
            for w in reversed(path):
                known[w]=known[links[w][0]]+links[w][1]
        else:false.update(path)
    real=Graph(idx,q.bases,counts,d)
    truth={c for c,v in enumerate(real.buckets) if len(v)==1 and real.by_rows[c]==counts[c]}
    assert set(known)==truth
    for c,h in known.items():
        i=real.buckets[c][0];t=0
        while real.parent[i]!=-1:t+=1;i=real.parent[i]
        assert h==t
    return dict(types=k,rows=idx.end,old_runs=len(q.root),initial_types=initial,certified_types=len(known),
        total_steps=steps,interior_queries=probes,selected_types=len(table),selected_run_records=pieces,
        selected_type_mass=positions,additional_full_scans=passes,additional_records_scanned=rows_seen,
        start_links=len(links),budget_cycle_stops=len(periodic))

def small():
    total=Counter()
    for n in range(1,8):
        for l in product(range(3),repeat=n):
            f=Counter(l);order=sorted(f);raw=[(c,sum(1 for _ in v)) for c,v in groupby(l)]
            for d in order:
                a=inspect(Quotient(raw,{c:bytes([c]) for c in order},order,f,{d},d),d)
                total.update(objects=1,rows=n,certified_types=a['certified_types'],
                    start_links=a['start_links'],interior_queries=a['interior_queries'])
    print(json.dumps(total,sort_keys=True,indent=2))
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
            r=dict(tape=int(tape.name.split('.')[0]),h=h,**inspect(Quotient(raw,orig,order,count,selected,d),d))
            if writer is None:writer=csv.DictWriter(sys.stdout,fieldnames=r,lineterminator='\n');writer.writeheader()
            writer.writerow(r);sys.stdout.flush()
if __name__=='__main__':real(sys.argv[1]) if len(sys.argv)>1 else small()
