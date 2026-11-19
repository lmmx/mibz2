"""Construct the power column in destination order with interval backward extension."""
from bisect import bisect_right
from collections import Counter
from normalize import append_run,verify_dense


def normalize(raw,words,protected,bounds,slices,check=False):
    positions=[]
    pieces=[]
    occurrences=[0]*len(words)
    pos=0
    for k,n in raw:
        positions.append(pos)
        delta=bounds[k]+occurrences[k]-pos
        pieces.append((pos,pos+n,k,delta))
        occurrences[k]+=n
        pos+=n
    dst=0
    roots=cuts=queries=intersections=advances=steps=leaves=0
    tmp=[]
    keyset=set()
    peak=0
    for k,j,n in slices:
        if k!=j:
            roots+=1
            k0=bisect_right(positions,dst)-1
            _,_,_,delta=pieces[k0]
            side=-1 if j<k else 1
            queue=[(dst+delta,n,1)]
            while queue:
                peak=max(peak,len(queue))
                x,w,d=queue.pop()
                if x<0: # Terminal output record in this destination-order walk.
                    append_run(tmp,(k,d,side),w)
                    keyset.add((k,d,side))
                    leaves+=1
                    continue
                i=bisect_right(positions,x)-1
                queries+=1
                end=x+w
                tail=[]
                while x<end:
                    lo,hi,c,delta=pieces[i]
                    b=min(hi,end)
                    size=b-x
                    if c!=k:
                        item=(-1,size,d)
                    else:
                        assert delta!=0
                        bulk=(hi-b)//delta+1 if delta>0 else (x-lo)//(-delta)+1
                        item=(x+bulk*delta,size,d+bulk)
                        advances+=bulk*size
                        steps+=1
                    tail.append(item)
                    intersections+=1
                    i+=1;x=b
                cuts+=len(tail)-1
                queue.extend(reversed(tail))
        dst+=n
    assert leaves==roots+cuts and cuts<=len(raw)-1
    keys=sorted(keyset,key=lambda t:(t[0],t[2],t[1] if t[2]<0 else -t[1]))
    ids={key:i for i,key in enumerate(keys)}
    power={}
    weights=[]
    for k,d,side in keys:
        if d==1:
            weights.append(words[k])
        else:
            key=words[k],d
            if key not in power:
                power[key]=words[k].power(d)
            weights.append(power[key])
    root=[(ids[t],n) for t,n in tmp]
    sole=[i for i,(k,d,side) in enumerate(keys) if k==protected]
    assert len(sole)==1
    a=sole[0]
    m=sum(n for k,n in root)
    counts=Counter(affine_pieces=len(raw),start_fragments=roots,macro_rows=m,macro_types=len(keys),
                   macro_runs=len(root),power_nodes=len(power),point_lookups=queries,
                   bulk_self_steps=advances,largest_power=max(t[1] for t in keys),
                   nonfixed_old_rows=m+advances,omitted_fixed_rows=bounds[-1]-m-advances,
                   seam_splits=cuts,terminal_fragments=leaves,intersections=intersections,
                   continued_pieces=steps,peak_pending=peak,destination_scatter_rows=0)
    assert counts['omitted_fixed_rows']>=0 and len(root)<=leaves
    if check:
        verify_dense(raw,bounds,words,protected,root,weights,keys,a,counts)
    return root,weights,a,counts
