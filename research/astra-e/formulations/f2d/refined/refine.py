"""Exact seam-refined LF^k translations and actual F2 discovery probes."""
import bisect
import heapq
from collections import Counter,deque
import csv
from hashlib import sha256
from pathlib import Path
import struct
import sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent))
from chains import Column

class Table:
    def __init__(self,c,depth,prev=None,stop=True):
        self.starts=[]
        self.delta=[]
        self.word=[]
        self.first=[]
        self.terminal=[]
        self.per_run=[]
        self.start_searches=0
        for i,ch in enumerate(c.ids):
            a,b=c.starts[i:i+2]
            delta=c.lf[i]-a
            before=len(self.word)
            if ch==34 and stop:
                self.add(a,0,b"",i,i)
            elif prev is None:
                self.add(a,delta,bytes([ch]),i,-1)
            else:
                j=bisect.bisect_right(prev.starts,a+delta)-1
                self.start_searches+=1
                while a<b:
                    self.add(a,delta+prev.delta[j],bytes([ch])+prev.word[j],i,prev.terminal[j])
                    a=min(b,prev.starts[j+1]-delta)
                    j+=1
            self.per_run.append(len(self.word)-before)
        self.starts.append(c.starts[-1])
        assert len(self.word)<=min(c.starts[-1],1+depth*(len(c.ids)-1))
        assert prev is None or len(self.word)<=len(prev.word)+len(c.ids)-1

    def add(self,a,d,w,f,t):
        self.starts.append(a);self.delta.append(d);self.word.append(w)
        self.first.append(f);self.terminal.append(t)

def topology(c,paths):
    children=[[] for _ in paths]
    next_child=1
    for p,(_,final) in enumerate(paths):
        i,a,b=final
        chosen=set()
        while c.starts[i]<b:
            ch=c.ids[i]
            if ch!=34 and ch not in chosen:
                chosen.add(ch);children[p].append(next_child);next_child+=1
            i+=1
    assert next_child==len(paths) or not paths
    return children

def schedule(children,cost):
    if not cost: return 0,0
    ready=deque([0]);free=list(range(8));working=[];clock=0
    span=cost.copy()
    for u,kids in enumerate(children):
        for v in kids: span[v]+=span[u]
    while ready or working:
        while ready and free:
            j=ready.popleft();lane=heapq.heappop(free)
            heapq.heappush(working,(clock+cost[j],lane,j))
        clock=working[0][0]
        while working and working[0][0]==clock:
            _,lane,j=heapq.heappop(working);heapq.heappush(free,lane)
            ready.extend(children[j])
    return clock,max(span)

def replay(c,paths,t,k):
    metrics=Counter();demand=set();used=set()
    actions=[]
    for unary,final in paths:
        start=metrics["queries"]
        states=unary+[final]
        j=0
        while True:
            i,a,b=states[j]
            d=bisect.bisect_right(t.starts,a)-1
            metrics["queries"]+=1;demand.add(i);used.add(d)
            assert i==t.first[d]
            if b<=t.starts[d+1]:
                w=t.word[d];step=len(w)
                assert bytes(c.ids[z[0]] for z in unary[j:j+step])==w
                assert a+t.delta[d]==states[j+step][1] and b+t.delta[d]==states[j+step][2]
                if t.terminal[d]!=-1:
                    assert j+step==len(unary) and t.terminal[d]==final[0]
                    metrics["quote_terminations"]+=1;break
                assert step==k and j+step<=len(unary)
                j+=step;metrics["jumps"]+=1
            else:
                # Width crosses a refined seam. The descriptor still identifies
                # the initial original run, so ordinary branch or unary work can use it.
                metrics["too_narrow"]+=1
                if j==len(unary):
                    metrics["branch_terminations"]+=1;break
                j+=1;metrics["scalar_advances"]+=1
        actions.append(metrics["queries"]-start)
    children=topology(c,paths)
    metrics["waves"],metrics["span"]=schedule(children,actions)
    metrics["scalar_waves"],metrics["scalar_span"]=schedule(children,[len(x)+1 for x,_ in paths])
    # Learn only exact refined cells that were actually queried. One source
    # rank is already paid to choose the original-run cache bucket on each
    # query. Charge every subsequent original-rank lookup on a first miss.
    for d in used:
        x=t.starts[d];j=t.first[d];low=0;high=c.starts[-1];translation=0;word=b""
        for step in range(k):
            if step:
                j=c.find(x+translation);metrics["lazy_extra_ranks"]+=1
            low=max(low,c.starts[j]-translation)
            high=min(high,c.starts[j+1]-translation)
            if c.ids[j]==34:
                assert t.terminal[d]==j
                break
            word+=bytes([c.ids[j]])
            translation+=c.lf[j]-c.starts[j]
        assert (low,high,word,translation)==(t.starts[d],t.starts[d+1],t.word[d],t.delta[d])
    metrics["lazy_original_ranks"]=metrics["queries"]+metrics["lazy_extra_ranks"]
    metrics["distinct_cells_used"]=len(used)
    metrics["original_run_demand"]=len(demand)
    return metrics,demand

def lazy_closure(c,tables,demand):
    # Optimistic whole-original-run lazy construction. Earlier-level tables
    # are required for each original run intersected by a demanded image.
    work=0;images=0
    for table in reversed(tables[1:]):
        prior=set()
        for i in demand:
            work+=table.per_run[i]
            if c.ids[i]==34: continue
            images+=1
            lo=c.lf[i];hi=lo+c.starts[i+1]-c.starts[i]-1
            prior.update(range(c.find(lo),c.find(hi)+1))
        demand=prior
    return work,images

def double(c,left,right,whole_demands):
    # Compose stopped two-step descriptors with themselves. An early quote
    # propagates without touching the second table.
    total=0;live=0;need2=set(whole_demands)
    for i,base in enumerate(left.starts[:-1]):
        x=base;hi=left.starts[i+1];shift=left.delta[i]
        requested=left.first[i] in whole_demands
        if left.terminal[i]!=-1:
            assert (right.starts[total],right.starts[total+1],right.word[total],right.delta[total])==(
                x,hi,left.word[i],shift)
            total+=1;continue
        live+=1
        j=bisect.bisect_right(left.starts,x+shift)-1
        while x<hi:
            end=min(hi,left.starts[j+1]-shift)
            assert (right.starts[total],right.starts[total+1],right.word[total],right.delta[total],right.terminal[total])==(
                x,end,left.word[i]+left.word[j],shift+left.delta[j],left.terminal[j])
            if requested:
                need2.add(left.first[j])
            total+=1;x=end;j+=1
    assert total==len(right.word)
    lazy=sum(right.per_run[i] for i in whole_demands)+sum(left.per_run[i] for i in need2)
    return total+len(left.word),live+sum(1 for v in c.ids if v!=34),lazy

def solve(c,paths):
    raw=sum(len(p)+1 for p,z in paths)
    full=[];prev=None
    for k in range(1,5):
        prev=Table(c,k,prev,False)
        full.append(len(prev.word))
    del prev
    tables=[];out=[]
    for k in range(1,5):
        t=Table(c,k,tables[-1] if tables else None)
        tables.append(t)
        if k not in (2,4): continue
        m,used=replay(c,paths,t,k)
        lazy,imgs=lazy_closure(c,tables,used)
        best_emit,best_seek,best_lazy=(double(c,tables[1],tables[3],used) if k==4 else (
            len(t.word),t.start_searches,lazy))
        out.append(dict(k=k,rows=c.starts[-1],runs=len(c.ids),nodes=len(paths),base_queries=raw,
                        full_cells=full[k-1],stop_cells=len(t.word),
                        full_setup_emissions=sum(full[1:k]),stop_setup_emissions=sum(len(x.word) for x in tables[1:]),
                        setup_first_seeks=sum(x.start_searches for x in tables[1:]),**{
                            f:m[f] for f in ("queries","jumps","scalar_advances","too_narrow","quote_terminations",
                                              "branch_terminations","distinct_cells_used","original_run_demand","lazy_extra_ranks","lazy_original_ranks","waves","span","scalar_waves","scalar_span")},
                        saved_queries=raw-m["queries"],lazy_setup_emissions=lazy,lazy_image_starts=imgs,doubling_setup_emissions=best_emit,
                        doubling_start_seeks=best_seek,lazy_doubling_emissions=best_lazy))
    return out

if __name__=="__main__":
    w=None
    for p in sorted(Path(sys.argv[1]).glob("*.recs"),key=lambda p:int(p.stem)):
        blob=p.read_bytes();c=Column([x[0] for x in struct.iter_unpack("<H",blob)])
        paths,parts=c.actual()
        for r in solve(c,paths):
            r=dict(stream=int(p.stem),capped_sha256=sha256(blob).hexdigest(),branch_pieces=parts,**r)
            if w is None:
                w=csv.DictWriter(sys.stdout,fieldnames=list(r),lineterminator="\n");w.writeheader()
            w.writerow(r)
        sys.stdout.flush()
