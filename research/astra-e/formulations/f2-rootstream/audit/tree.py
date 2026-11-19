"""Independent selector-tape oracle, reverse projection, and interval descent."""
from bisect import bisect_right
from collections import Counter
from itertools import accumulate,groupby


def encode(seq):
    return [(k,len(list(g))) for k,g in groupby(seq)]


def routes(parents):
    paths=[(0,)]
    for i,p in enumerate(parents[1:],1):
        assert 0<=p<i
        paths.append(paths[p]+(i,))
    return paths


def from_root(labels,paths):
    # Record where each known terminal event goes at every ancestor.
    streams=[[] for _ in paths]
    for v in labels:
        path=paths[v]
        for u,child in zip(path,path[1:]): streams[u].append(child)
        streams[v].append(v)
    return [encode(s) for s in streams]


def materialize(tapes):
    # Independent literal (one row per value) bottom-up stable inverse merge.
    output=[None]*len(tapes)
    for u in reversed(range(len(tapes))):
        out=[];used=Counter()
        for choice,width in tapes[u]:
            if choice==u: out.extend([u]*width)
            else:
                lo=used[choice]
                assert lo+width<=len(output[choice])
                out.extend(output[choice][lo:lo+width]);used[choice]+=width
        for v,count in used.items(): assert count==len(output[v])
        output[u]=out
    return output


def descend(tapes):
    # Inspect cumulative stopping ranks by binary search. This does not share the
    # Rust implementation's per-selector cursors or decrement/update code.
    ends=[list(accumulate(w for _,w in v)) for v in tapes]
    consumed=[0]*len(tapes); result=[]; metrics=Counter()
    while consumed[0]<ends[0][-1]:
        q=0;path=[];remainders=[]
        while True:
            path.append(q)
            j=bisect_right(ends[q],consumed[q])
            remainders.append(ends[q][j]-consumed[q])
            choice=tapes[q][j][0]
            if q==choice:break
            q=choice
        width=min(remainders)
        assert width>0
        metrics["visits"]+=len(path)
        metrics["nonleaf_bottlenecks"]+=int(width<remainders[-1])
        metrics["simultaneous_cuts"]+=int(remainders.count(width)>1)
        for u in path:consumed[u]+=width
        result.append((q,width))
        # At a nonfinal root output boundary the highest exhausted choice must
        # switch to another branch or terminal. This supplies the charge.
        if consumed[0]<ends[0][-1]:
            pos=remainders.index(width)
            u=path[pos];j=bisect_right(ends[u],consumed[u])
            assert 0<j<len(ends[u])
            assert tapes[u][j][0]!=tapes[u][j-1][0]
            metrics["charged_boundaries"]+=1
    assert all(n==x[-1] for n,x in zip(consumed,ends))
    return result,metrics


def audit(labels,parents):
    paths=routes(parents)
    tapes=from_root(labels,paths)
    if any(not tape for tape in tapes): return None
    full=materialize(tapes)
    got,metrics=descend(tapes)
    assert full[0]==list(labels)
    assert got==encode(labels)
    t=len(got); sel=sum(map(len,tapes)); v=len(tapes)
    assert t<=sel-v+1
    assert metrics["charged_boundaries"]==t-1
    assert metrics["visits"]==sum(len(paths[j]) for j,_ in got)
    local=sum(len(encode(out)) for out in full)
    assert metrics["visits"]>=local
    return {"rows":len(labels),"root_runs":t,"nodes":v,"selectors":sel,
            "materialized_runs":local,**metrics}
