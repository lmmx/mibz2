"""Transport a core alphabet between two sections. Build the original permutation by intersecting bands."""
from bisect import bisect_right
from collections import Counter
from itertools import groupby, product
from pathlib import Path
import csv
import json
import sys

here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parent/'short-core'))
from induce import Atlas, Column, Oracle, heavy_set, intervals_union, truth
sys.path.insert(0,str(here.parents[1]))
from order import collation


def append_run(out,key,width):
    if out and out[-1][0]==key:
        out[-1]=(key,out[-1][1]+width)
    else:
        out.append((key,width))


def incoming(col,u,atlas):
    v=intervals_union((lo,lo+width) for _,lo,width,_ in atlas.leaves)
    ends=[end for _,end in v]
    live=[[u]]
    parents=[None]
    selectors=[]
    count=Counter()
    bases={}
    seen={}
    searches=pieces=0
    # The only terminal information consumed here is membership in the physical V.
    for node,packets in enumerate(live):
        select=[]
        kids={}
        terminal={}
        for lo,hi in packets:
            run=bisect_right(col.starts,lo)-1
            searches+=1
            while lo<hi:
                stop=min(hi,col.starts[run+1])
                delta=col.maps[run]
                c=col.symbols[run]
                a,b=lo+delta,stop+delta
                j=bisect_right(ends,a)
                while a<b:
                    hit=j<len(v) and v[j][0]<=a
                    end=min(b,v[j][1] if hit else v[j][0] if j<len(v) else b)
                    assert end>a
                    if hit:
                        if c not in terminal:
                            ident=len(seen)
                            terminal[c]=ident
                            seen[ident]=(node,c)
                        ident=terminal[c]
                        bases[ident]=min(bases.get(ident,a),a)
                        count[ident]+=end-a
                        tag=~ident
                    else:
                        if c not in kids:
                            kids[c]=len(live)
                            live.append([])
                            parents.append((node,c))
                        tag=kids[c]
                        other=live[tag]
                        if other and other[-1][1]==a:
                            other[-1]=(other[-1][0],end)
                        else:
                            other.append((a,end))
                    append_run(select,tag,end-a)
                    pieces+=1
                    a=end
                    if hit and end==v[j][1]:
                        j+=1
                lo=stop
                run+=1
        selectors.append(select)
    streams={}
    for node in reversed(range(len(live))):
        out=[]
        cursor={}
        for tag,width in selectors[node]:
            if tag<0:
                append_run(out,~tag,width)
                continue
            if tag not in cursor:
                cursor[tag]=[iter(streams.pop(tag)),None,0]
            lane=cursor[tag]
            while width:
                if lane[2]==0:
                    lane[1],lane[2]=next(lane[0])
                take=min(width,lane[2])
                append_run(out,lane[1],take)
                lane[2]-=take
                width-=take
        streams[node]=out
    dictionary={}
    for ident,pair in seen.items():
        core=bytearray()
        while pair is not None:
            node,c=pair
            core.append(c)
            pair=parents[node]
        dictionary[ident]=bytes(core)
    assert len(set(dictionary.values()))==len(dictionary)
    assert sum(count.values())==u[1]-u[0]
    return streams[0],dictionary,bases,count,dict(live_states=len(live),lookups=searches,lf_pieces=pieces)


def compose(u,geo,stream,dictionary,bases,count):
    order=sorted(dictionary,key=bases.get)
    slot={}
    at=0
    for k in order:
        slot[k]=at
        at+=count[k]
    assert at==u[1]-u[0]
    area=0
    boundaries=[]
    owners=[]
    for origin,lo,width,prefix in geo.leaves:
        boundaries.append(area)
        owners.append((origin,prefix))
        area+=width
    boundaries.append(area)
    # The initial physical band for each core bucket is located by one merge.
    band_of={}
    j=0
    for k in order:
        while boundaries[j+1]<=slot[k]:
            j+=1
        band_of[k]=j
    dest=u[0]
    segments=[]
    joint=[]
    for k,width in stream:
        v=slot[k]
        j=band_of[k]
        while width:
            take=min(width,boundaries[j+1]-v)
            origin,prefix=owners[j]
            s=origin+v-boundaries[j]
            segments.append((s,s+take,dest,prefix,k))
            append_run(joint,(prefix,k),take)
            dest+=take
            width-=take
            v+=take
            if v==boundaries[j+1]:
                j+=1
        band_of[k]=j
        slot[k]=v
    assert len(segments)<=len(geo.leaves)+len(stream)-1
    assert dest==u[1]
    segments.sort()
    assert segments[0][0]==u[0] and segments[-1][1]==u[1]
    assert all(x[1]==y[0] for x,y in zip(segments,segments[1:]))
    return order,segments,joint


def check(col,oracle,u,h,marks):
    geo=Atlas(col,u,h)
    raw,words,bases,count,cost=incoming(col,u,geo)
    order,seg,joint=compose(u,geo,raw,words,bases,count)
    expected,dest_words=truth(oracle,u)
    # Independent full spelling + each incoming run, not just a core count.
    assert len(seg)==len(joint)
    assert [(p+words[k],w) for (p,k),w in joint]==[(w,sum(1 for _ in g)) for w,g in groupby(dest_words)]
    truth_buckets=sorted(expected,key=lambda w:expected[w][0][0])
    starts=[v[0] for v in seg]
    for w in truth_buckets:
        for origin,dest in expected[w]:
            s,end,d,p,k=seg[bisect_right(starts,origin)-1]
            assert d+origin-s==dest and p+words[k]==w
    # Independently use a single continuation from U instead of physical minima.
    sorted_words,_=collation(oracle,[words[k] for k in words],u[0])
    assert sorted_words==[words[k] for k in order]
    n=col.rows
    for primary in marks:
        ref=bytearray()
        x=primary
        for _ in range(n):
            ref.append(oracle.first[x]);x=oracle.forward[x]
        x=primary
        out=bytearray()
        while not u[0]<=x<u[1] and len(out)<n:
            out.append(oracle.first[x]);x=oracle.forward[x]
        while len(out)<n:
            s,end,d,p,k=seg[bisect_right(starts,x)-1]
            out.extend(p[:n-len(out)])
            out.extend(words[k][:n-len(out)])
            x=d+x-s
        assert out==ref
    return dict(mass=u[1]-u[0],core_types=len(words),full_types=len(expected),
                core_runs=len(raw),full_runs=len(joint),origin_bands=len(geo.leaves),transport_segments=len(seg),
                independent_marked=len(marks),**cost)


def small():
    total=Counter();sharp=None
    for n in range(1,8):
        for x in product(range(2),repeat=n):
            col=Column([(c,sum(1 for _ in g)) for c,g in groupby(x)])
            oracle=Oracle(bytes(x))
            for l in range(n):
                for r in range(l+1,n+1):
                    for h in (0,1,3,n):
                        row=check(col,oracle,(l,r),h,range(n))
                        total['objects']+=1
                        total['marks']+=n
                        total['coalescing_cores']+=row['core_types']<row['full_types']
                        if row['core_types']<row['full_types'] and row['origin_bands']>1 and row['core_runs']<row['full_runs'] and sharp is None:
                            sharp=dict(last=bytes(x).hex(),u=[l,r],depth=h,**row)
    print(json.dumps(dict(counts=total,coalesced_core_witness=sharp),indent=2))


def real(tapes):
    sys.path.insert(0,str(here.parents[7]/'docs/journal/2026-11-18-astra-c-output-first'))
    from inputs import read_input,paths
    writer=None
    for tape in paths(tapes):
        runs,mark=read_input(tape)
        col=Column(runs);o=Oracle(b''.join(bytes([c])*n for c,n in runs))
        sec=heavy_set(col,col.interval(b'"'),1)[0][0]
        for h in (0,4,8,12):
            row=dict(ordinal=int(tape.name.split('.')[0]),h=h,**check(col,o,sec,h,[mark]))
            if writer is None:
                writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n');writer.writeheader()
            writer.writerow(row);sys.stdout.flush()


if __name__=='__main__':
    real(Path(sys.argv[1])) if len(sys.argv)>1 else small()
