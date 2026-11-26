"""Can every first-return query be discovered with one monotone scanner per source class?"""
from bisect import bisect_right
from collections import Counter,deque
from itertools import groupby,product
from pathlib import Path
import csv
import json
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from view import Atlas,Column,Quotient,full,incoming,paths,read_input


class Graph:
    def __init__(self,column,bases,counts,d):
        self.parent=[]
        self.word=[]
        self.pos=[]
        self.buckets=[[] for _ in bases]
        self.by_rows=Counter()
        self.weight=Counter()
        queue=deque([(d,bases[d],bases[d]+counts[d],-1)])
        while queue:
            w,lo,hi,parent=queue.popleft()
            i=len(self.parent)
            self.word.append(w);self.pos.append(lo);self.parent.append(parent)
            self.buckets[w].append(i)
            self.by_rows[w]+=hi-lo
            self.weight[w]+=1
            children={}
            for char,width,lf in column.segments(lo,hi):
                if char==d:continue
                a,b=children.get(char,(lf,lf))
                assert b==lf
                children[char]=(a,b+width)
            queue.extend((c,a,b,i) for c,(a,b) in children.items())
        for w,bucket in enumerate(self.buckets):
            bucket.sort(key=self.pos.__getitem__)
            assert self.by_rows[w]<=counts[w]
        self.n=len(self.parent)

    def ordering(self,reverse,skip=()):
        edges=[[] for _ in self.parent]
        indeg=[0]*self.n
        for i,p in enumerate(self.parent):
            if p>=0:edges[p].append(i);indeg[i]+=1
        for w,bucket in enumerate(self.buckets):
            if w in skip:continue
            order=reversed(bucket) if w in reverse else iter(bucket)
            prev=next(order,None)
            for i in order:
                edges[prev].append(i);indeg[i]+=1;prev=i
        work=[i for i,v in enumerate(indeg) if not v]
        used=0
        while work:
            i=work.pop();used+=1
            for v in edges[i]:
                indeg[v]-=1
                if not indeg[v]:work.append(v)
        return self.n-used

    def forced(self):
        # Existing parent precedence alone forces both ends of any comparable
        # same-bucket pair. Compute each descendant's range of ancestor starts
        # of the same class along its unique trie path.
        children=[[] for _ in self.parent]
        for i,p in enumerate(self.parent):
            if p>=0:children[p].append(i)
        extrema={}
        req=[0]*len(self.buckets)
        # Along one ancestor chain a fixed ascending scan must reset at every
        # descent. A bidirectional scan must turn whenever consecutive signs change.
        ascending=[0]*len(self.buckets);descending=ascending.copy();turns=ascending.copy()
        stack=[(0,False,None)]
        while stack:
            i,exit_,history=stack.pop()
            w=self.word[i];x=self.pos[i]
            if exit_:
                if history is None:del extrema[w]
                else:extrema[w]=history
                continue
            before=extrema.get(w)
            if before is None:extrema[w]=(x,x,x,0,0,0,0)
            else:
                a,b,last,sign,up,down,changed=before
                if a<x:req[w]|=1
                if b>x:req[w]|=2
                current=1 if last<x else -1
                up+=int(current>0); down+=int(current<0)
                changed+=int(sign!=0 and sign!=current)
                ascending[w]=max(ascending[w],down)
                descending[w]=max(descending[w],up)
                turns[w]=max(turns[w],changed)
                extrema[w]=(min(a,x),max(b,x),x,current,up,down,changed)
            stack.append((i,True,before))
            for child in children[i]:stack.append((child,False,None))
        return req,ascending,descending,turns


def small():
    total=Counter();witness={}
    for n in range(1,10):
        for last in product(range(3),repeat=n):
            f=Counter(last);order=sorted(f)
            if len(order)<2: continue
            raw=[(k,sum(1 for _ in v)) for k,v in groupby(last)]
            for d in order:
                q=Quotient(raw,{k:bytes([k]) for k in order},order,f,{d},d)
                g=Graph(full(q),q.bases,[f[k] for k in order],q.ids[d]);total['graphs']+=1
                # Skip classes receiving no competing order constraints.
                active=[i for i,x in enumerate(g.buckets) if len(x)>1]
                req,up,down,turn=g.forced()
                ok=any(g.ordering({k for k,flag in zip(active,flags) if flag})==0 for flags in product((0,1),repeat=len(active)))
                if not ok:
                    key='local' if 3 in req else 'coupled'
                    total[key]+=1
                    if key not in witness:witness[key]=dict(last=last,delimiter=d,order=order,
                        nodes=list(zip(g.parent,g.word,g.pos)),buckets=g.buckets,forced=req)
                else:assert 3 not in req
    print(json.dumps(dict(counts=total,witnesses=witness),indent=2))


def real(tapes):
    writer=None
    for tape in paths(tapes):
        rs,_=read_input(tape);col=Column(rs);u=col.interval(b'"')
        raw,lead,mins,count,_=incoming(col,u,Atlas(col,u,0))
        order=sorted(lead,key=mins.get)
        orig={k:w[1:]+b'"' for k,w in lead.items()}
        z=next(k for k,w in orig.items() if w==b',"')
        for h in (0,4,8,12):
            if h:
                selected={k for k,w in lead.items() if w[min(h,len(w)-1):]==b','};d=z
            else:
                d=max(order,key=lambda k:(count[k],-mins[k]));selected={d}
            q=Quotient(raw,orig,order,count,selected,d)
            f=[count[k] if k!=d else q.s for k in order]
            g=Graph(full(q),q.bases,f,q.ids[d])
            forced,up,down,turn=g.forced()
            clash={i for i,v in enumerate(forced) if v==3}
            rev={i for i,v in enumerate(forced) if v==2}
            one={i for i,v in enumerate(g.buckets) if len(v)==1 and g.by_rows[i]==f[i]}
            # The forced-clash count is a lower bound for every orientation.
            # Remaining blocked nodes under one orientation do not prove infeasibility.
            report=dict(tape=int(tape.name.split('.')[0]),h=h,queries=g.n,types=len(order),rows=sum(f),
              reachable_rows=sum(g.by_rows.values()),full_once_types=len(one),full_once_rows=sum(f[i] for i in one),
              both_required_types=len(clash),both_required_rows=sum(f[i] for i in clash),
              both_required_queries=sum(len(g.buckets[i]) for i in clash),
              unresolved_after_clash_exemption=g.ordering(rev,clash),
              required_fixed_direction_resets=sum(min(a,b) for a,b in zip(up,down)),
              max_symbol_resets=max(min(a,b) for a,b in zip(up,down)),
              required_turns=sum(turn),max_symbol_turns=max(turn),
              stalled_ascending=g.ordering(set()),stalled_descending=g.ordering(set(range(len(order)))))
            if writer is None:
                writer=csv.DictWriter(sys.stdout,fieldnames=report,lineterminator='\n');writer.writeheader()
            writer.writerow(report);sys.stdout.flush()
if __name__=='__main__':
    real(sys.argv[1]) if len(sys.argv)>1 else small()
