"""Find the heaviest shortened core's original source intervals from L alone, without a forward atlas."""
from collections import Counter
from heapq import heappop, heappush
from itertools import groupby, product
from pathlib import Path
import csv
import json
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/'audit'))
from intervals import RunIndex, clip, consecutive
from panel import arcs
from inputs import read_input, paths
from induce import Column, heavy_set


def owners(idx,u,word,interval,h,stats):
    # A prefix P is supplied in normal byte order only at an accepted old source.
    level=[(interval,b'')]
    result=[]
    for d in range(h+1):
        nxt=[]
        for (lo,hi),prefix in level:
            stats['probe_nodes']+=1
            a,b=max(lo,u[0]),min(hi,u[1])
            if a<b and (len(word)==1 or d==h):
                result.append((a,b,prefix))
            if d==h:
                continue
            rem=clip((lo,hi),u)
            if rem[0]==rem[1]:
                continue
            kids,pieces=idx.children(rem)
            stats['probe_pieces']+=pieces
            for sym,j in kids.items():
                nxt.append((j,bytes([sym])+prefix))
        level=nxt
        if not level:
            break
    return sorted(result)


def select(idx,u,h):
    queue=[]
    stats=Counter()
    def extend(interval,word):
        kids,pieces=idx.children(interval)
        stats['search_nodes']+=1
        stats['search_pieces']+=pieces
        for c,(l,r) in kids.items():
            heappush(queue,(l-r,bytes([c])+word,l,r))
    extend(u,b'')
    win=0
    ties=[]
    while queue and -queue[0][0]>=win:
        _,word,l,r=heappop(queue)
        own=owners(idx,u,word,(l,r),h,stats)
        stats['tested']+=1
        m=sum(b-a for a,b,p in own)
        assert m<=r-l
        if m>win:
            win=m
            ties=[]
        if m==win and m:
            ties.append((word,own))
        rest=clip((l,r),u)
        if rest[1]-rest[0]>=max(win,1):
            extend(rest,word)
        stats['max_heap']=max(stats['max_heap'],len(queue))
    return win,sorted(ties),stats


def verify(runs,forward,arc_words,u,h):
    idx=RunIndex(runs)
    win,ties,stats=select(idx,u,h)
    tails=Counter(w[min(h,len(w)-1):] for w in arc_words)
    assert win==max(tails.values())
    assert [w for w,_ in ties]==sorted(w for w,n in tails.items() if n==win)
    for c,found in ties:
        source={}
        for a,b,p in found:
            assert a<b
            for x in range(a,b):
                assert x not in source
                source[x]=p
        expected={}
        for x,w in enumerate(arc_words,u[0]):
            d=min(h,len(w)-1)
            if w[d:]==c:
                expected[x]=w[:d]
        assert source==expected
    c,found=ties[0]
    q={i:p for a,b,p in found for i in range(a,b)}
    # Recover only the selected cuts for an independent test of the lift.
    moved={}
    for x,p in q.items():
        y=x
        for _ in p:
            y=forward[y]
        assert y not in moved
        moved[y]=x
    old_spans=consecutive(q)
    prefixes=set(q.values())
    old_dict={}
    new_dict={}
    rotation_pairs=0
    if len(prefixes)==1:
        common=next(iter(prefixes))
        # Old and moved selected orders agree; check all offsets, actual second
        # occurrence partners and every original/rotated second word.
        assert list(sorted(q))==[moved[y] for y in sorted(moved)]
        last=b''.join(bytes([b])*w for b,w in runs)
        for start in q:
            word=bytearray()
            x=start
            while True:
                x=forward[x]
                word.append(last[x])
                if x in q:
                    break
            y=start
            for _ in common:
                y=forward[y]
            z=y
            shifted=bytearray()
            while True:
                z=forward[z]
                shifted.append(last[z])
                if z in moved:
                    break
            assert moved[z]==x and shifted==word[len(common):]+common
            old_dict[bytes(word)]=1
            new_dict[bytes(shifted)]=1
            rotation_pairs+=1
        assert len(old_dict)==len(new_dict)
        assert sum(map(len,old_dict))==sum(map(len,new_dict))
    return dict(top_hex=c.hex(),mass=win,ties=len(ties),source_components=len(old_spans),
                prefix_count=len(prefixes),old_bands=len(found),physical_components=len(consecutive(moved)),
                selected_second_partners=rotation_pairs,second_types=len(old_dict),
                second_D=sum(map(len,old_dict)),**{k:stats[k] for k in
                 ('tested','search_nodes','search_pieces','probe_nodes','probe_pieces','max_heap')})


def small():
    counts=Counter()
    witnesses={}
    for n in range(1,8):
        for xs in product(range(2),repeat=n):
            runs=[(b,sum(1 for _ in it)) for b,it in groupby(xs)]
            first=bytes(xs)
            f=sorted(range(n),key=first.__getitem__)
            for a in range(n):
                for b in range(a+1,n+1):
                    words=arcs(first,f,a,b)
                    for h in (0,1,3,n):
                        row=verify(runs,f,words,(a,b),h)
                        counts.update(objects=1,old_selected_sources=row['mass'],
                                      rotated_second_partners=row['selected_second_partners'])
                        for k,yes in [('multiple_origin_prefix',row['prefix_count']>1),
                                ('fragmented_origin',row['source_components']>1),
                                ('fragmented_physical_single_prefix',row['prefix_count']==1 and row['physical_components']>1),
                                ('tied_winner',row['ties']>1)]:
                            if yes:
                                counts[k]+=1
                                witnesses.setdefault(k,dict(last=first.hex(),u=[a,b],h=h,**row))
    print(json.dumps(dict(counts=counts,witnesses=witnesses),indent=2))


def real(tapes):
    out=None
    for tape in paths(tapes):
        runs,mark=read_input(tape)
        col=Column(runs)
        last=b''.join(bytes([c])*n for c,n in runs)
        f=sorted(range(len(last)),key=last.__getitem__)
        quote=col.interval(b'"')
        for name,u in [('frequent',heavy_set(col,quote,1)[0][0]),('quote',quote)]:
            words=arcs(last,f,*u)
            for h in (4,8,12):
                row=dict(ordinal=int(tape.name.split('.')[0]),section=name,h=h,**verify(runs,f,words,u,h))
                if out is None:
                    out=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
                    out.writeheader()
                out.writerow(row)
                sys.stdout.flush()


if __name__=='__main__':
    real(sys.argv[1]) if len(sys.argv)>1 else small()
