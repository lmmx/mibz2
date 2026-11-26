"""Publish maximal equal-source-type words with a clipped affine successor index."""
from bisect import bisect_right
from collections import Counter
from itertools import groupby
from pathlib import Path
import sys

sys.path.insert(0,str(Path(__file__).resolve().parent.parent))
from cut import append_run, forward, geometry


def normalize(raw,words,protected,bounds,slices,check=False):
    off=[0]*len(words)
    buckets=[[] for _ in words]
    dst=0
    for k,n in raw:
        lo=bounds[k]+off[k]
        buckets[k].append((lo,lo+n,dst-lo))
        off[k]+=n
        dst+=n
    pieces=[p for per_type in buckets for p in per_type]
    starts=[p[0] for p in pieces]
    survivors=[]
    pos=total=0
    for incoming,owner,n in slices:
        if incoming!=owner:
            survivors.append((pos,pos+n,total,owner))
            total+=n
        pos+=n
    assert total>0
    # The sparse physical-row → retained-ordinal predecessor index.
    from_rows=[s[0] for s in survivors]
    def compressed(x):
        j=bisect_right(from_rows,x)-1
        assert j>=0
        lo,hi,prefix,_=survivors[j]
        assert x<hi
        return prefix+x-lo

    counts=Counter(affine_pieces=len(pieces),start_fragments=len(survivors),
                   macro_rows=total,macro_types=0,macro_runs=0,power_nodes=0,
                   point_lookups=0,bulk_self_steps=0,largest_power=0)
    power_by_base={}
    aliases=[]
    weights=[]
    target_labels=[None]*total
    seen=set()
    mark_alias=None
    for x0,x1,_,k in survivors:
        for x in range(x0,x1):
            v=x
            length=0
            while True:
                lo,hi,delta=pieces[bisect_right(starts,v)-1]
                assert lo<=v<hi
                counts['point_lookups']+=1
                y=v+delta
                if not bounds[k]<=y<bounds[k+1]:
                    length+=1
                    break
                # Source interval on which this affine piece stays in bucket k.
                low=max(lo,bounds[k]-delta)
                high=min(hi,bounds[k+1]-delta)
                assert low<=v<high and delta!=0
                steps=(high-1-v)//delta+1 if delta>0 else (v-low)//(-delta)+1
                v+=steps*delta
                length+=steps
                counts['bulk_self_steps']+=steps
            side=-1 if y<bounds[k] else 1
            key=(k,length,side)
            if not aliases or aliases[-1]!=key:
                assert key not in seen
                seen.add(key)
                aliases.append(key)
                if length==1:
                    word=words[k]
                else:
                    base_key=(words[k],length)
                    if base_key not in power_by_base:
                        power_by_base[base_key]=words[k].power(length)
                    word=power_by_base[base_key]
                weights.append(word)
            a=len(weights)-1
            if k==protected:
                assert length==1 and mark_alias is None
                mark_alias=a
            dest=compressed(y)
            assert target_labels[dest] is None
            target_labels[dest]=a
            counts['largest_power']=max(counts['largest_power'],length)
    assert mark_alias is not None and all(k is not None for k in target_labels)
    projected=[]
    for k,g in groupby(target_labels):
        append_run(projected,k,sum(1 for _ in g))
    counts.update(macro_types=len(weights),macro_runs=len(projected),power_nodes=len(power_by_base))
    # Each complete non-fixed orbit was partitioned into disjoint maximal runs.
    counts['nonfixed_old_rows']=counts['bulk_self_steps']+total
    counts['omitted_fixed_rows']=bounds[-1]-counts['nonfixed_old_rows']
    assert 0<=counts['omitted_fixed_rows'] and counts['point_lookups']<=counts['nonfixed_old_rows']
    if check:
        verify_dense(raw,bounds,words,protected,projected,weights,aliases,mark_alias,counts)
    return projected,weights,mark_alias,counts


def verify_dense(raw,bounds,words,protected,root,new_words,keys,a,counts):
    """Separate dense source→destination construction, row ownership and weights."""
    old=forward(raw,bounds[:-1])
    starts=set()
    fixed=set()
    for i,(j,k) in enumerate(old):
        if old[j][1]!=k:
            starts.add(j)
        elif i==j:
            fixed.add(i)
    retained=sorted(starts)
    mapped={x:j for j,x in enumerate(retained)}
    _,bases=geometry(root,len(new_words))
    actual=forward(root,bases[:-1])
    assert len(fixed)==counts['omitted_fixed_rows'] and len(actual)==len(retained)
    consumed=set()
    for x,rank in mapped.items():
        k=old[x][1]
        y=x
        d=0
        while True:
            assert y not in consumed
            consumed.add(y)
            y=old[y][0]
            d+=1
            if old[y][1]!=k:
                break
        dest,label=actual[rank]
        assert (k,d,-1 if y<bounds[k] else 1)==keys[label]
        assert dest==mapped[y] and label!=actual[dest][1]
        w=new_words[label]
        assert w.n==d*words[k].n
        assert (d==1 and w is words[k]) or (d>1 and w.left is words[k] and w.repeat==d and w.right is None)
        if k==protected:
            assert label==a and d==1
    assert consumed.isdisjoint(fixed) and len(consumed)+len(fixed)==len(old)
