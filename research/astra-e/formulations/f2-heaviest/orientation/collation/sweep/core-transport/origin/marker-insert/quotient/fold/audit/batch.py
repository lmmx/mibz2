"""Stream the frozen deterministic DAG over RLE fragments and compare with legal singleton folds."""
import argparse
from collections import Counter, deque
from itertools import groupby, product
import json

from exhaustive import Column


def append(buf,k,w):
    if buf and buf[-1][0]==k:
        buf[-1] = (k,buf[-1][1]+w)
    else:
        buf.append((k,w))


def batch(incoming):
    raw=[(k,len(list(g))) for k,g in groupby(incoming)]
    freq=Counter(incoming)
    labels=sorted(freq)
    slices={j:[] for j in labels}
    targets={j:set() for j in labels}
    j=pos=0
    for k,w in raw:
        while w:
            to=labels[j]
            used=min(w,freq[to]-pos)
            slices[to].append((k,used))
            targets[k].add(to)
            w-=used
            pos+=used
            if pos==freq[to]:
                j+=1
                pos=0
    succ={k:next(iter(t)) for k,t in targets.items() if len(t)==1 and k not in t}
    waiting=Counter(succ.values())
    ready=deque(j for j in labels if waiting[j]==0)
    topo=[]
    streams={}
    cursors={j:(0,0) for j in labels}

    def consume(k,w,output):
        stream=streams[k]
        p,off=cursors[k]
        while w:
            v,n=stream[p]
            size=min(n-off,w)
            append(output,v,size)
            w-=size
            off+=size
            if off==n:
                p+=1
                off=0
        cursors[k]=p,off

    while ready:
        j=ready.popleft()
        topo.append(j)
        a=[]
        for k,w in slices[j]:
            if k in succ:
                consume(k,w,a)
            else:
                append(a,j,w)
        streams[j]=a
        if j in succ:
            to=succ[j]
            waiting[to]-=1
            if not waiting[to]:
                ready.append(to)
    closed=set(labels)-set(topo)
    pending=closed.copy()
    cycles=[]
    while pending:
        start=min(pending)
        cycle=[]
        j=start
        while j in pending:
            cycle.append(j)
            pending.remove(j)
            j=succ[j]
        assert j==start and len({freq[k] for k in cycle})==1
        cycles.append(cycle)
    retain={cycle[0] for cycle in cycles}
    result=[]
    for j in labels:
        if j in closed:
            if j in retain:
                append(result,j,freq[j])
        else:
            for k,w in slices[j]:
                if k not in succ:
                    consume(k,w,result)
    assert all(cursors[j]==(len(stream),0) for j, stream in streams.items())
    removed=sum(freq[j] for j in succ)-sum(freq[j] for j in retain)
    assert sum(w for _,w in result)==len(incoming)-removed
    h=sum(map(len,slices.values()))
    outside_runs=sum(k not in succ for k,w in raw)
    bound=h+outside_runs-len(labels)+len(cycles)
    assert len(result)<=bound<=2*len(raw)-1
    count=Counter(classes=len(labels),runs=len(raw),slices=h,output_runs=len(result),run_bound=bound,
                  ancestry_runs=sum(map(len,streams.values())),cycles=len(cycles),
                  cycle_rows_kept=sum(freq[j] for j in retain),removed=removed)
    return result,succ,topo,cycles,count


def test(args):
    totals=Counter()
    for n in range(1,args.max_rows+1):
        for k in range(1,min(n,args.max_classes)+1):
            for incoming in product(range(k),repeat=n):
                if len(set(incoming))!=k:
                    continue
                raw,succ,topo,cycles,count=batch(incoming)
                o=Column(incoming)
                state=o.initial
                choices=[j for j in reversed(topo) if j in succ]
                for cycle in cycles:
                    choices.extend([cycle[0]]*(len(cycle)-1))
                for j in choices:
                    groups,targets,opts=o.geometry(state)
                    assert j in groups and len(opts[j])==1 and j not in opts[j]
                    state=o.contract(state,groups[j],targets)
                groups,targets,opts=o.geometry(state)
                final,runs=o.run_formula(state,groups,targets)
                assert final==[j for j,w in raw for _ in range(w)]
                assert o.check_marks(state,groups,targets)>0
                totals.update(count,objects=1)
    print(json.dumps(dict(config=vars(args),totals=totals),indent=2))


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--max-rows',type=int,default=7)
    p.add_argument('--max-classes',type=int,default=4)
    test(p.parse_args())
