"""Exact compact predecessor counts and deterministic power-key ordering."""
from bisect import bisect_right
from collections import Counter
from itertools import groupby,product
from math import isqrt
from pathlib import Path
import json
import sys
from unittest.mock import patch
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent))
import decode
import backward
from cut import geometry,overlay
from grammar import Node
from orient import real


class Rank:
    def __init__(self,n,starts):
        self.masks=[0]*((n+63)//64)
        for start in starts:
            self.masks[start>>6] |= 1<<(start&63)
        self.before=[]
        count=0
        for mask in self.masks:
            self.before.append(count)
            count+=mask.bit_count()
        assert count==len(starts)

    def upto(self,x):
        block=x>>6
        bits=(1<<((x&63)+1))-1
        return self.before[block]+(self.masks[block]&bits).bit_count()


def radix(keys,m):
    # Return the stable permutation, including repeated keys. Each pass uses
    # contiguous counters and positional writes with a fixed number of digits.
    b=isqrt(m)+1
    digits=[lambda t:(t[1] if t[2]<0 else m-t[1])%b,
            lambda t:(t[1] if t[2]<0 else m-t[1])//b,
            lambda t:int(t[2]>0),lambda t:t[0]%b,lambda t:t[0]//b]
    order=list(range(len(keys)))
    for digit in digits:
        counts=[0]*b
        for i in order:counts[digit(keys[i])]+=1
        first=[]
        at=0
        for n in counts:
            first.append(at)
            at+=n
        following=[0]*len(order)
        for i in order:
            k=digit(keys[i])
            following[first[k]]=i
            first[k]+=1
        order=following
    return order


def compare(raw,words,mark=None):
    freq,b=geometry(raw,len(words))
    m=b[-1]
    slices,_=overlay(raw,b)
    if all(k==j for k,j,n in slices):
        return dict(fixed=1)
    normal=backward.normalize(raw,words,None,b,slices,True)
    starts=[]
    at=0
    for k,n in raw:
        starts.append(at)
        at+=n
    rank=Rank(m,starts)
    if m<1000:
        assert [rank.upto(i) for i in range(m)]==[bisect_right(starts,i) for i in range(m)]
    stats=Counter()
    donor_positions=None

    def query(positions,x):
        nonlocal donor_positions
        if donor_positions is None:
            assert positions==starts
            donor_positions=positions
        assert positions is donor_positions
        stats['rank_queries']+=1
        return rank.upto(x)

    def order(keys,key):
        items=list(keys)
        p=radix(items,m)
        result=[items[i] for i in p]
        assert result==sorted(items,key=key)
        return result

    with patch.object(backward,'bisect_right',query),patch.object(backward,'sorted',order,create=True):
        fast=backward.normalize(raw,words,None,b,slices,True)
    assert normal[0]==fast[0] and normal[2:]==fast[2:]
    assert [(w.n,w.left,w.repeat) for w in normal[1]]==[(w.n,w.left,w.repeat) for w in fast[1]]
    c=normal[3]
    assert c['point_lookups']<=m and c['intersections']<=c['point_lookups']+len(raw)-1
    return dict(stats,runs=len(raw),types=len(words),rows=m,published=c['macro_runs'],lookups=c['point_lookups'],
                intersections=c['intersections'],rank_blocks=len(rank.masks),leaf_fragments=c['terminal_fragments'])


if __name__=='__main__':
    total=Counter()
    for n in range(1,8):
        for last in product(range(3),repeat=n):
            keys=sorted(set(last))
            if keys!=list(range(len(keys))):continue
            raw=[(k,len(list(g))) for k,g in groupby(last)]
            r=compare(raw,[Node(literal=bytes([k])) for k in keys])
            total['columns']+=1
            total.update(r)
    # Empty bitmap words and exact boundaries around two widely separated starts.
    z=Rank(12346,[0,63,64,65,10239,12345])
    for i in range(12346):
        assert z.upto(i)==bisect_right([0,63,64,65,10239,12345],i)
    # Stable sorting repeated powers on either side of radix boundaries.
    for m in (3,4,15,16,65535,65536):
        keys=[(i,d,s) for i in (0,m-1) for d in (1,m-1,m) for s in (1,-1)]*2
        assert radix(keys,m)==sorted(range(len(keys)),key=lambda i:(keys[i][0],keys[i][2],keys[i][1] if keys[i][2]<0 else -keys[i][1]))
    out=dict(tiny=total,all_positions_across_empty_mask_words=12346)
    _,_,mark,raw,words=real.source(Path(sys.argv[1]))
    out['original_first_power']=compare(raw,[Node(literal=w) for w in words],mark)
    print(json.dumps(out,indent=2))
