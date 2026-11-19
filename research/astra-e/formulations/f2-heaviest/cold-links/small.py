"""Arbitrary columns and all marks, including disjoint and nondividing marked cycles."""
from collections import Counter
from itertools import product
import json
from census import make_index,prune,describe,marked
from truncated import construct
from barriers import descriptors,decode_barriers,verify_partial_intervals

stats=Counter(); witnesses={}
for n in range(1,8):
    for seq in product((33,34,35),repeat=n):
        last=bytes(seq);q,m,lf=make_index(last)
        seeds,a,t=prune(last,q,m,lf)
        src,labs,cut,rem,hot=describe(last,q,m,lf,seeds)
        assert (hot,cut)==construct(last,q,m,lf)
        alt=descriptors(hot,cut)
        psi=sorted(range(n),key=last.__getitem__)
        if alt:
            B,macros,met=alt
            assert verify_partial_intervals(B,hot,cut,macros)[0]==met["regularity_queries"]
            assert len(B)==met["second_rows"]+len(rem)
            assert sum(c is not None for _,c,_ in macros.values())==len(rem)
        stats["columns"]+=1
        for p in range(n):
            x=p;expected=bytearray()
            for _ in range(n):
                x=psi[x];expected.append(last[x])
            got=marked(last,p,q,m,lf,cut,hot)
            if got is None:
                assert 34 not in expected
                if alt: assert decode_barriers(last,p,q,lf,hot,cut,B,macros) is None
                stats["section_free_marks"]+=1;continue
            out,period,visits,work,cold=got
            assert out==expected
            stats["marked_decodes"]+=1
            if alt:
                arc=decode_barriers(last,p,q,lf,hot,cut,B,macros)
                assert arc[0]==expected
                stats["partial_second_decodes"]+=1
                if arc[2]==0: stats["marked_hot_cycles_without_barrier"]+=1
            if n%period:
                stats["nondividing_period_marks"]+=1
                if cold and "nondividing_cold" not in witnesses:
                    witnesses["nondividing_cold"]=dict(last=seq,mark=p,n=n,period=period,out=expected.hex())
print(json.dumps(dict(counts=stats,witnesses=witnesses),indent=2))
