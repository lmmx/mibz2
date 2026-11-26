"""Decimate quotient rank after removing heterogeneous multiplicities and restore the original horizon."""
from bisect import bisect_right
from collections import Counter
from functools import reduce
from itertools import accumulate, groupby, product
from math import gcd
from pathlib import Path
import sys
import json
from trajectory import runs, cascade, spell
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[2]/'interval-states'/'refine'/'proof'/'chains'))
from geometry import Geometry
sys.path.insert(0,str(here.parents[4]))
from clock import index


def quotient(column):
    geo=Geometry(column)
    orbits=geo.fibers()
    starts=sorted(x for orbit,_ in orbits for x in orbit)
    letters=bytes(geo.byte[bisect_right(geo.src,x)-1] for x in starts)
    return geo,starts,letters


def check():
    count=Counter()
    for a,maximum in ((2,7),(3,4)):
        for n in range(1,maximum+1):
            for L in product(range(a),repeat=n):
                rle=[(v,len(list(g))) for v,g in groupby(L)]
                geo,starts,column=quotient(rle)
                F,psi,lf=index(L)
                Fq,psiq,lfq=index(column)
                for i,x in enumerate(starts):
                    for y in range(x,(starts+[n])[i+1]):
                        j=y-x
                        assert lf[y]==starts[lfq[i]]+j and psi[y]==starts[psiq[i]]+j
                        expected=[];rank=y
                        for _ in range(n):
                            expected.append(F[rank]);rank=psi[rank]
                        for q in (2,4):
                            _,grammar,_,t,root=cascade(runs(column),i,q)
                            period=spell(grammar,root)[::-1]
                            assert list((period*((n+t-1)//t))[:n])==expected
                            count['marked_bytes_checked']+=n
                            count['mark_radix_settings']+=1
                            count['heterogeneous_gcd_one_compressions']+=len(starts)<n and reduce(gcd,(w for _,w in rle))==1
                count['tables']+=1
    g=10**12+7
    rle=[(0,g),(1,g+1),(0,g+1)]
    geo,starts,col=quotient(rle)
    assert reduce(gcd,(w for _,w in rle))==1 and col==bytes((0,1,0)) and starts==[0,g,2*g+1]
    small=[]
    for p in (0,g-1,g,2*g,2*g+1,3*g+1):
        phase=bisect_right(starts,p)-1
        _,gr,_,t,root=cascade(runs(col),phase,2)
        word=spell(gr,root)[::-1]
        expected=[];x=p
        for _ in range(16):
            y=geo.psi(x)
            expected.append(geo.byte[bisect_right(geo.src,y)-1])
            x=y
        assert list((word*((16+t-1)//t))[:16])==expected
        small.append(dict(mark=p,quotient=phase,lap=t,period=word.hex()))
    return dict(counts=dict(count),gcd_one_heterogeneous=dict(n=geo.n,base=list(col),starts=starts,marks=small))


if __name__=='__main__':print(json.dumps(check(),indent=2))
