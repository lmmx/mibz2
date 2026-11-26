"""Arbitrary large run counts and delimiter-word lengths can attain the cylinder bound."""
from itertools import groupby
from gaps import compile_runs
from interval_bound import reverse_boundaries, tokens

witnesses=0
for k in range(1,45):
    for h in range(1,15):
        col=[1+(i%2) for i in range(k)]+[0]*(h*k+1)
        runs=[(c,len(list(g))) for c,g in groupby(col)]
        _,_,_,top,bottom=compile_runs(runs)
        lf=[0]*len(col)
        for a,b,_,y in top:
            for x in range(a,b):
                lf[x]=y+x-a
        lo,hi=0,len(col)
        for _ in range(h):
            images=[lf[x] for x in range(lo,hi) if col[x]==0]
            lo,hi=min(images),max(images)+1
        assert (lo,hi)==(0,k+1)
        labels=tokens(lf,col,lo,hi)
        assert len(list(groupby(labels)))==len(runs)==k+1
        assert reverse_boundaries(top,bottom,lo,hi)==list(range(hi))
        witnesses+=1
print(f"{witnesses} exact cylinders attain t=r, k=1..44, marker length h=1..14")
