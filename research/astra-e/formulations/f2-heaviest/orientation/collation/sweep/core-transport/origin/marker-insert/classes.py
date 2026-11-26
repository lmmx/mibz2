"""Factor a weighted token alphabet by several suffix classes and verify the exact RLE cost."""
from collections import Counter,defaultdict
from itertools import groupby,product
from pathlib import Path
import csv
import json
import sys
from insert import Atlas,Column,Oracle,append_run,heavy_set,incoming,paths,read_input


def grouped(raw,words,order,count,assignment):
    # Each selected label i factors as prefix_i followed by shared literal suffix class c.
    # An empty class suffix is allowed in this test, provided each full old word is positive.
    colors=sorted(set(assignment.values()))
    marker={c:i for i,c in enumerate(colors)}
    ids={k:i+len(colors) for i,k in enumerate(order)}
    bins=defaultdict(list)
    remaining=[]
    islands=Counter()
    gaps=Counter()
    last_bound={}
    last_color=None
    last_id=None
    for k,w in raw:
        if k not in assignment:
            append_run(remaining,ids[k],w)
            last_color=None
        else:
            c=assignment[k]
            if last_color!=c:
                islands[c]+=1
                if c in last_bound and last_bound[c]==k:
                    gaps[c]+=1
            last_bound[c]=k
            append_run(bins[c],ids[k],w)
            append_run(remaining,marker[c],w)
            last_color=c
        assert last_id!=k
        last_id=k
    root=[]
    bases=[]
    weights=list(colors)
    total=0
    for c in colors:
        bases.append(total)
        for k,w in bins[c]:
            append_run(root,k,w)
            total+=w
    extra=total
    for k in order:
        bases.append(total)
        total+=count[k]
        suffix=assignment.get(k,b'')
        assert words[k].endswith(suffix)
        weights.append(words[k][:len(words[k])-len(suffix)])
    for k,w in remaining:
        append_run(root,k,w)
    assert sum(w for k,w in root)==total
    assert len(root)==len(raw)+sum(islands.values())-sum(gaps.values())
    assert len(raw)+len(colors)<=len(root)<=2*len(raw)
    if len(colors)==1:
        assert len(root)<=len(raw)+(len(raw)+1)//2
    return root,bases,weights,marker,ids,dict(colors=len(colors),extra_rows=extra,original_R=len(raw),
       factored_R=len(root),islands=sum(islands.values()),equal_bridges=sum(gaps.values()),
       extra_runs=len(root)-len(raw),literal_bytes=sum(map(len,weights)),old_bytes=sum(map(len,words.values())))


def forward(root,bases):
    n=sum(w for k,w in root)
    slots=bases.copy()
    seq=[None]*n
    dest=0
    for k,w in root:
        for _ in range(w):
            seq[slots[k]]=(dest,k)
            slots[k]+=1
            dest+=1
    assert None not in seq
    return seq


def small():
    totals=Counter()
    witnesses={}
    # A three-letter original alphabet, all subsets and two suffix colors.
    for n in range(1,7):
        for last in product(range(3),repeat=n):
            order=sorted(set(last),reverse=True)
            count=Counter(last)
            raw=[(k,sum(1 for _ in g)) for k,g in groupby(last)]
            base=[]
            m=0
            for k in order:
                base.append(m)
                m+=count[k]
            old_ids={k:i for i,k in enumerate(order)}
            old=forward([(old_ids[k],w) for k,w in raw],base)
            for coloring in product(range(3),repeat=len(order)):
                assign={k:bytes([c]) for k,c in zip(order,coloring) if c}
                # Include genuinely empty allocated prefixes on original k=0.
                words={k:bytes([9+k])*k+assign.get(k,b'q') for k in order}
                root,bases,weights,marker,ids,stats=grouped(raw,words,order,count,assign)
                actual=forward(root,bases)
                t=stats['extra_rows']
                select={}
                for c,j in marker.items():
                    rank=bases[j]
                    for y,k in enumerate(last):
                        if assign.get(k)==c:
                            select[y]=rank
                            assert actual[rank]==(t+y,j)
                            rank+=1
                for x,(y,i) in enumerate(old):
                    k=order[i]
                    v,j=actual[t+x]
                    assert j==ids[k]
                    if k in assign:
                        assert v==select[y] and weights[j]+weights[actual[v][1]]==words[k]
                    else:
                        assert v==t+y and weights[j]==words[k]
                totals.update(objects=1,original_arcs=n,new_arcs=n+t)
                if stats['colors']==1 and stats['original_R']>=5 and stats['extra_runs']==(stats['original_R']+1)//2:
                    witnesses.setdefault('one_class_upper',dict(last=last,coloring=coloring,order=order,**stats))
                if stats['colors']>=2 and stats['equal_bridges']>1:
                    witnesses.setdefault('crossing_color_islands',dict(last=last,coloring=coloring,order=order,**stats))
    print(json.dumps(dict(counts=totals,witnesses=witnesses),indent=2))


def real(tapes):
    writer=None
    fixed=b'":"value","property'
    for tape in paths(tapes):
        runs,_=read_input(tape)
        col=Column(runs)
        quote=col.interval(b'"')
        for name,section in [('quote',quote),('frequent',heavy_set(col,quote,1)[0][0])]:
            raw,words,minima,count,_=incoming(col,section,Atlas(col,section,0))
            order=sorted(words,key=minima.get)
            for h in (4,8,12):
                cores={k:w[min(h,len(w)-1):] for k,w in words.items()}
                top=b',' if name=='quote' else fixed[h:]
                for kind,assign in [('one',{k:c for k,c in cores.items() if c==top}),('all',cores)]:
                    *_,metrics=grouped(raw,words,order,count,assign)
                    row=dict(ordinal=int(tape.name.split('.')[0]),section=name,h=h,mode=kind,
                             tokens=section[1]-section[0],old_K=len(words),**metrics)
                    if writer is None:
                        writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
                        writer.writeheader()
                    writer.writerow(row)
                    sys.stdout.flush()


if __name__=='__main__':
    real(sys.argv[1]) if len(sys.argv)>1 else small()
