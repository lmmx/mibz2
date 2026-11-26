"""Build independent phase input fixtures, check the exact two-layer source order and marked output."""
from array import array
from bisect import bisect_right
from collections import Counter, defaultdict
from itertools import groupby,product
from pathlib import Path
import csv
import hashlib
import json
import sys

from lift import Lift,append_run
HERE = Path(__file__).resolve().parent
REPO = next(p for p in HERE.parents if (p/'Cargo.toml').exists())
sys.path.insert(0,str(HERE.parents[1]/'short-core'))
sys.path.insert(0,str(REPO/'docs/journal/2026-11-18-astra-c-output-first'))
from induce import Atlas,Column,heavy_set
from inputs import paths,read_input


def geometry(last):
    psi = array('I', sorted(range(len(last)),key=last.__getitem__))
    lf = array('I',[0])*len(last)
    for x,y in enumerate(psi):
        lf[y]=x
    return psi,lf


def fixture(last,psi,u,h):
    a,b = u
    cores = [None]*(b-a)
    cuts = []
    for src in range(a,b):
        path = [src]
        w = bytearray()
        x = src
        while True:
            x = psi[x]
            w.append(last[x])
            if a <= x < b:
                break
            path.append(x)
        t = min(h,len(w)-1)
        c = bytes(w[t:])
        cuts.append((path[t],src,x,bytes(w[:t]),c))
        cores[x-a]=c
    cuts.sort()
    root = []
    for c in cores:
        append_run(root,c,1)
    minima={c:pos for pos,_,_,_,c in reversed(cuts)}
    return root,{c:c for c in minima},minima,cuts


def source_mark(col,mark,u,leaves):
    # V has priority at coincident ranks; the U token there is empty.
    starts=[v for _,v,_,_ in leaves]
    totals = []
    cursor = 0
    for _,_,w,_ in leaves:
        totals.append(cursor)
        cursor += w
    x = mark
    for skip in range(col.rows):
        i = bisect_right(starts,x)-1
        if i>=0 and x<starts[i]+leaves[i][2]:
            return totals[i]+x-starts[i],skip
        if u[0]<=x<u[1]:
            return cursor+x-u[0],skip
        run=bisect_right(col.starts,x)-1
        x+=col.maps[run]
        if x==mark:
            return None


def plain(last,psi,mark):
    x=mark
    data=bytearray()
    for _ in last:
        x=psi[x]
        data.append(last[x])
    return bytes(data)


def original_fallback(col, mark):
    segments=sorted((x+t,y+t,-t,c) for x,y,t,c in zip(col.starts,col.starts[1:],col.maps,col.symbols))
    starts=[x for x,_,_,_ in segments]
    x=mark
    data=bytearray()
    for _ in range(col.rows):
        lo,hi,delta,c=segments[bisect_right(starts,x)-1]
        assert lo<=x<hi
        data.append(c)
        x+=delta
    return bytes(data)


def case(last,psi,lf,col,u,h,marks,stock=None,measure_capacity=False):
    atlas=Atlas(col,u,h)
    root,words,minima,cuts=fixture(last,psi,u,h)
    model=Lift(u,atlas.leaves,root,words,minima)
    pos={s:i for i,(_,s,_,_,_) in enumerate(cuts)}
    g=pos
    counts=Counter()
    for i,(z,s,d,p,c) in enumerate(cuts):
        y,k=model.forward(i)
        assert y==model.m+d-u[0] and model.weights[k]==c
        nxt,k=model.forward(model.m+s-u[0])
        assert nxt==pos[s] and model.weights[k]==p
    # The tempting core-only column can be unstable after moving destinations.
    seq=[None]*model.m
    for i,(_,_,d,_,c) in enumerate(cuts):
        seq[g[d]]=(c,i)
    actual=defaultdict(list)
    for c,i in seq:
        actual[c].append(i)
    unstable=sum(any(x>y for x,y in zip(items,items[1:])) for items in actual.values())
    result=model.induce(model.core)
    capped=model.cap_prefixes()
    forced=capped.induce(capped.core) if capped is not model else result
    prefix_types=model.weights[model.core_count:]
    for mark in marks:
        expected=plain(last,psi,mark)
        location=source_mark(col,mark,u,atlas.leaves)
        if location is None:
            assert original_fallback(col,mark)==expected
            counts['first_section_disjoint']+=1
            continue
        x,skip=location
        counts['mark_lifted_row']=x
        counts['mark_original_lf_steps']=skip
        _,outgoing=model.forward(x)
        assert skip<len(model.weights[outgoing])
        assert result.marked(model,x,skip,len(last))==expected
        counts['marked_exact']+=1
        if stock is not None:
            data,used,uncapped_capacity=stock(model,x,skip,len(last),measure_capacity)
            assert data==expected
            counts['stock_second']+=used
            counts['stock_verified']+=1
            if capped is model:
                capped_data,capped_used,capped_capacity=data,used,uncapped_capacity
            else:
                capped_data,capped_used,capped_capacity=stock(capped,x,skip,len(last),measure_capacity)
            assert capped_data==expected
            counts['capped_stock_second']+=capped_used
            counts['capped_stock_verified']+=1
            counts['native_capacity_bytes']=max(counts['native_capacity_bytes'],uncapped_capacity)
            counts['capped_native_capacity_bytes']=max(counts['capped_native_capacity_bytes'],capped_capacity)
    # Positive H-return words have at least one core weight even if H itself is an empty prefix.
    byte_words = [b''.join(model.weights[t] for t in w) for w in result.dictionary]
    assert all(byte_words)
    forced_bytes=[b''.join(capped.weights[t] for t in w) for w in forced.dictionary]
    assert set(forced_bytes)==set(byte_words)
    assert len(forced.segments)>=len(result.segments)
    assert sum((v-1)//model.sizes[model.core] for v in model.sizes[model.core_count:])==len(capped.weights)-len(model.weights)
    return dict(m=model.m,core_types=model.core_count,prefix_ids=len(prefix_types),
                literal_prefixes=len(set(prefix_types)),empty_prefix_ids=prefix_types.count(b''),
                atlas_bands=len(atlas.leaves),prefix_runs=model.prefix_runs,core_runs=len(root),lift_runs=len(model.column),
                core_mass=model.sizes[model.core],stock_mass=model.sizes[model.unforced],
                stock_heaviest='core' if model.unforced<model.core_count else 'prefix',
                stock_delimiter_bytes=len(model.weights[model.unforced]),
                unstable_core_fibers=unstable,first_factor_bytes=sum(map(len,set(model.weights))),
                second_types=len(result.dictionary),second_runs=len(result.segments),
                representative_halfsteps=result.probes,unique_factor_slots=sum(map(len,result.dictionary)),
                second_naive_bytes=sum(map(len,byte_words)),second_unique_bytes=sum(map(len,set(byte_words))),
                first_section_disjoint=counts['first_section_disjoint'],marked_exact=counts['marked_exact'],
                stock_verified=counts['stock_verified'],stock_second=counts['stock_second'],
                capped_extra_ids=len(capped.weights)-len(model.weights),capped_extra_runs=len(capped.column)-len(model.column),
                capped_second_runs=len(forced.segments),capped_second_types=len(forced.dictionary),
                capped_representative_halfsteps=forced.probes,capped_second_dictionary_bytes=sum(map(len,forced_bytes)),
                capped_stock_second=counts['capped_stock_second'],capped_stock_verified=counts['capped_stock_verified'],
                native_capacity_bytes=counts['native_capacity_bytes'],capped_native_capacity_bytes=counts['capped_native_capacity_bytes'],
                mark_lifted_row=counts.get('mark_lifted_row',-1),mark_original_lf_steps=counts.get('mark_original_lf_steps',-1))


def small(stock=None):
    stats=Counter()
    witness=None
    for n in range(1,7):
        for parts in product(range(2),repeat=n):
            last=bytes(parts);psi,lf=geometry(last)
            runs=[(v,len(list(g))) for v,g in groupby(last)]
            col=Column(runs)
            for a in range(n):
                for b in range(a+1,n+1):
                    for h in sorted({0,1,2,n}):
                        row=case(last,psi,lf,col,(a,b),h,range(n),stock)
                        stats['cases']+=1
                        for key in ('marked_exact','first_section_disjoint','stock_verified','stock_second','capped_stock_verified','capped_stock_second'):
                            stats[key]+=row[key]
                        stats['capping_needed_cases']+=row['capped_extra_ids']>0
                        stats['recurrent_prefix_alias_cases']+=row['prefix_ids']>row['literal_prefixes']
                        stats['empty_prefix_cases']+=bool(row['empty_prefix_ids'])
                        stats['unstable_core_cases']+=bool(row['unstable_core_fibers'])
                        if row['unstable_core_fibers'] and witness is None:
                            _,_,_,cuts=fixture(last,psi,(a,b),h)
                            witness=dict(last=list(last),u=[a,b],h=h,psi=list(psi),cut_rows=[
                                dict(z=z,s=s,d=d,p=p.hex(),c=c.hex()) for z,s,d,p,c in cuts])
    print(json.dumps(dict(counts=stats,minimum_unstable_core=witness),indent=2))


def panel(tapes,stock):
    writer=None
    for path in paths(tapes):
        runs,mark=read_input(path)
        last=b''.join(bytes((b,))*w for b,w in runs)
        psi,lf=geometry(last)
        col=Column(runs)
        quote=col.interval(b'"')
        top=heavy_set(col,quote,1)[0][0]
        for name,u in [('frequent',top),('quote',quote)]:
            for h in (4,8,12):
                row=dict(ordinal=int(path.name.split('.')[0]),section=name,h=h,input_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                         mark=mark,marked_sha256=hashlib.sha256(plain(last,psi,mark)).hexdigest(),**case(last,psi,lf,col,u,h,[mark],stock,True))
                if writer is None:
                    writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
                    writer.writeheader()
                writer.writerow(row)
                sys.stdout.flush()


if __name__=='__main__':
    from stock import Stock
    lib = Stock(sys.argv[1]) if len(sys.argv)>1 and sys.argv[1]!='-' else None
    panel(Path(sys.argv[2]),lib) if len(sys.argv)>2 else small(lib)
