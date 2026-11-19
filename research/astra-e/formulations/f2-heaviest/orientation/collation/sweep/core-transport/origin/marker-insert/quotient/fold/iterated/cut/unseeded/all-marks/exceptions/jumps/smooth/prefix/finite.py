"""Exact sliced destination-prefix identity on arbitrary last columns and all marked horizons."""
from collections import Counter, defaultdict
from itertools import islice, product
import json

def check(last, counts, witness):
    n=len(last)
    psi=sorted(range(n),key=last.__getitem__)
    lf=[0]*n
    for x,y in enumerate(psi):lf[y]=x
    first=[last[y] for y in psi]
    def walk(p,step):
        while True:
            yield p
            p=step[p]
    def path(p,step,limit):return tuple(islice(walk(p,step),limit))
    def word(p,length):return tuple(first[x] for x in path(p,psi,length))
    seen=set()
    cycle_sizes=[]
    for p in range(n):
        if p in seen:continue
        x=p;length=0
        while x not in seen:seen.add(x);length+=1;x=lf[x]
        assert x==p
        cycle_sizes.append(length)
    counts['columns_multiple_cycles']+=len(cycle_sizes)>1
    counts['columns_heterogeneous_laps']+=len(set(cycle_sizes))>1
    counts['nondividing_marks']+=sum(d for d in cycle_sizes if n%d)
    counts['arbitrary_columns']+=1
    for k in range(1,7):
        prefixes=[word(x,k) for x in range(n)]
        hist=defaultdict(Counter)
        for x in range(n):hist[prefixes[x]][last[x]]+=1
        modes={s:min(h,key=lambda b:(-h[b],b)) for s,h in hist.items()}
        anchors={lf[x] for x in range(n) if last[x]!=modes[prefixes[x]]}
        shifted={word(x,k+1) for x in anchors}
        assert anchors=={x for x in range(n) if word(x,k+1) in shifted}
        counts['depths_empty_shifted_set']+=not anchors
        for u in sorted(anchors):
            v=psi[u];d=1
            while v not in anchors:v=psi[v];d+=1
            assert d<=n
            if d<=k+1:
                counts['short_shifted_returns']+=1
                w=word(u,d);assert w==word(u,k+1)[:d]
                wrong=word(v,k+1)[:d]
                if w!=wrong:
                    counts['wrong_next_patterns']+=1
                    if 'copy_next_boundary' not in witness:
                        witness['copy_next_boundary']=dict(n=n,last=last,k=k,source=u,next=v,
                              length=d,source_pattern=word(u,k+1),next_pattern=word(v,k+1),actual=w)
        # Strong witness: both full words are in the SAME column with the same destination k-prefix.
        longer=defaultdict(set)
        for x in range(n):longer[prefixes[x]].add(word(x,k+1))
        for v,full in longer.items():
            if len(full)>1 and 'insufficient_destination' not in witness:
                xs=[x for x in range(n) if prefixes[x]==v]
                witness['insufficient_destination']=dict(n=n,last=last,k=k,d=k+1,destination_rows=xs,
                                                          common_prefix=v,required=sorted(full))
        for p in range(n):
            back=path(p,lf,k+1)
            for d in range(k+1):
                full=tuple(last[x] for x in reversed(back[:d]))
                dest=prefixes[back[d]]
                assert full==dest[:d]
                for r in range(d+1):
                    assert tuple(last[x] for x in reversed(back[:r]))==dest[d-r:d]
                    counts['backward_clips']+=1
            # Compute a forward marked horizon through a clipped backward traversal from its true endpoint.
            spell=word(p,2*n+2)
            endpoints=path(p,psi,2*n+3)
            for horizon,at in enumerate(endpoints):
                left=horizon;out=[None]*horizon
                while left:
                    seg=path(at,lf,k+1);take=min(k,left)
                    out[left-take:left]=prefixes[seg[k]][k-take:k]
                    at=seg[take];left-=take
                assert at==p and tuple(out)==spell[:horizon]
                counts['marked_horizons']+=1
    counts['rows']+=n

def main():
    counts=Counter();witness={}
    # Increasing n, then alphabet and lexicographic order permits checking minimal witnesses.
    for n in range(1,9):
        for size in (2,3):
            if size==3 and n>6:continue
            for last in product(range(size),repeat=n):check(last,counts,witness)
    assert counts['columns_heterogeneous_laps'] and counts['nondividing_marks']
    assert set(witness)=={'copy_next_boundary','insufficient_destination'}
    print(json.dumps(dict(counts=dict(sorted(counts.items())),minimal_witnesses=witness),indent=2))

if __name__=='__main__':main()
