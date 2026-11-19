"""Exact minimum disjoint modular-affine cover and union of long fitting intervals."""
from collections import Counter
from itertools import groupby,permutations
import json
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from encode import real


def partition(pos, bounds):
    n=len(pos)
    spans=singles=0
    implied=0
    sizes=Counter()
    counts=Counter()
    covered=Counter()
    for start,end in zip(bounds,bounds[1:]):
        x=start
        while x<end:
            y=x+1
            if y<end:
                slope=(pos[y]-pos[x])%n
                y+=1
                while y<end and (pos[y]-pos[y-1])%n==slope:
                    y+=1
            singles+=y==x+1
            sizes[y-x]+=1
            implied+=max(0,y-x-2)
            spans+=1
            x=y
        thresholds=(4,8,16)
        last_end=dict.fromkeys(thresholds,start)
        left=start
        for slope,items in groupby((pos[x+1]-pos[x])%n for x in range(start,end-1)):
            steps=sum(1 for _ in items)
            counts[slope]+=steps
            right=left+steps+1
            for t in thresholds:
                if right-left>=t:
                    covered[t]+=right-max(left,last_end[t])
                    last_end[t]=right
            left+=steps
    return dict(spans=spans,singletons=singles,implied=implied,max_span=max(sizes),
                distinct_differences=len(counts),top_differences=counts.most_common(8),
                long_union={t:covered[t] for t in (4,8,16)})


def optimum(p):
    n=len(p)
    best=[n+1]*(n+1)
    best[0]=0
    for end in range(1,n+1):
        for start in range(end):
            diffs={(p[i]-p[i-1])%n for i in range(start+1,end)}
            if len(diffs)<=1:best[end]=min(best[end],best[start]+1)
    return best[-1]


def small():
    cases=0
    for n in range(1,8):
        for p in permutations(range(n)):
            assert partition(p,[0,n])['spans']==optimum(p)
            cases+=1
    return dict(permutation_minima_checked=cases)


def run(tapes):
    for tape in sorted(tapes.glob('*.irec.xz')):
        m,runs,mark,raw,words=real.source(tape)
        last=b''.join(bytes([k])*c for k,c in runs)
        n=len(last)
        forward=sorted(range(n),key=last.__getitem__)
        pos=[-1]*n
        x=mark
        for i in range(n):
            assert pos[x]<0
            pos[x]=i
            x=forward[x]
        assert x==mark
        bounds=[0]
        for k,width in sorted(Counter(last).items()):
            bounds.append(bounds[-1]+width)
        print(json.dumps(dict(index=m['index'],rows=n,global_axis=partition(pos,[0,n]),
                              first_buckets=partition(pos,bounds))),flush=True)


if __name__=='__main__':
    if len(sys.argv)>1:run(Path(sys.argv[1]))
    else:print(json.dumps(small()))
