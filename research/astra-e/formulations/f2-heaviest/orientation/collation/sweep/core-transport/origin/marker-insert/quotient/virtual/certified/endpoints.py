"""Recognize exactly the union of complete one-query source buckets by two extremal orbits."""
from bisect import bisect_right
from collections import Counter
from itertools import groupby,product
from pathlib import Path
import csv
import json
import sys
from closure import Atlas,Column,Graph,Quotient,full,incoming,paths,read_input

def observe(q,d,marks=False):
    idx=full(q);counts=[q.freq[k] if k!=q.z else q.s for k in q.order]
    d=q.ids[d]
    g=Graph(idx,q.bases,counts,d)
    cuts=[[] for _ in counts];lands=[[] for _ in counts]
    ends=idx.starts[1:]+[idx.end]
    for (c,lf),x,y in zip(idx.records,idx.starts,ends):
        cuts[c].append(lf);lands[c].append(x)
    def forward(c,x):
        i=bisect_right(cuts[c],x)-1
        return lands[c][i]+x-cuts[c][i]
    # Cache hits only after both endpoints already fall within another known
    # whole source class. A successful tail certificate transfers to its subsets.
    def recognize(cache_on):
        ok={d:0};cost=Counter();maximum=0
        orbit=[]
        for c in sorted(range(len(counts)),key=lambda i:-counts[i]):
            if c==d:continue
            who=c;x=q.bases[c];y=x+counts[c]-1;steps=0
            while True:
                if who==d or (cache_on and who in ok):
                    cost['positive_hits']+=who!=d
                    ok[c]=steps+ok[who];break
                if steps==idx.end:
                    cost['closed_misses']+=1;break
                if steps==0:
                    # These are available during the original scan, no select required.
                    x=lands[c][0]
                    y=lands[c][-1]+q.bases[c]+counts[c]-cuts[c][-1]-1
                else:
                    if x==y:
                        x=y=forward(who,x);cost['selects']+=1
                    else:
                        x,y=forward(who,x),forward(who,y);cost['selects']+=2
                    assert x<=y
                steps+=1
                cost['steps']+=1
                a=bisect_right(q.bases,x)-1
                if y>=q.bases[a]+counts[a]:
                    cost['mismatches']+=1;break
                who=a
            maximum=max(maximum,steps)
        return ok,cost,maximum
    plain,a,p=recognize(False);opt,b,m=recognize(True)
    truth={c for c,bb in enumerate(g.buckets) if len(bb)==1 and g.by_rows[c]==counts[c]}
    assert set(plain)==truth and opt==plain
    for c,depth in opt.items():
        j=g.buckets[c][0];levels=0
        while g.parent[j]!=-1:
            levels+=1;j=g.parent[j]
        assert depth==levels
    if marks:
        lf=[v+j for w,width,v in idx.segments(0,idx.end) for j in range(width)]
        for p0 in range(idx.end):
            past=set();seen_c=set();x=p0
            while x not in past:
                past.add(x)
                c=bisect_right(q.bases,x)-1
                if c in opt:
                    assert c not in seen_c
                    seen_c.add(c)
                x=lf[x]
                if q.bases[d]<=x<q.bases[d]+counts[d]:break
    return dict(roots=len(counts),rows=idx.end,select_records=len(idx.records),true_types=len(opt),
         true_rows=sum(counts[c] for c in opt),plain_steps=a['steps'],plain_selects=a['selects'],plain_max=p,
         cached_steps=b['steps'],cached_selects=b['selects'],cached_max=m,tail_hits=b['positive_hits'],
         closed_misses=b['closed_misses'],mismatches=b['mismatches'],max_depth=max(opt.values()))

def small():
    totals=Counter()
    for n in range(1,8):
        for l in product(range(3),repeat=n):
            f=Counter(l);order=sorted(f)
            raw=[(c,sum(1 for _ in v)) for c,v in groupby(l)]
            for d in order:
                q=Quotient(raw,{c:bytes([c]) for c in order},order,f,{d},d)
                o=observe(q,d,True)
                totals.update(objects=1,marks=n,types=o['roots'],true_types=o['true_types'],closed_misses=o['closed_misses'])
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
            row=dict(tape=int(tape.name.split('.')[0]),h=h,**observe(Quotient(raw,orig,order,count,selected,d),d))
            if writer is None:writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n');writer.writeheader()
            writer.writerow(row);sys.stdout.flush()

if __name__=='__main__':
    real(sys.argv[1]) if len(sys.argv)>1 else small()
