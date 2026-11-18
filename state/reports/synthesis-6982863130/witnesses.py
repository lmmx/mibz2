from itertools import product
from o3 import arclist
from o5 import lf
from boundary import section


def counts(cuts, f, primary=0, reverse_ties=False):
    from bisect import bisect_right
    n=len(f)
    old=[0,*sorted(cuts),n]
    arcs=arclist(old,f)
    i=bisect_right(old,primary)-1
    m=(i,primary-old[i])
    v=len(old)-1
    totals=0
    first=None
    while outcome:=section(arcs,v,m,reverse_ties=reverse_ties):
        chords,bands,m,_,_=outcome
        totals+=len(arcs)
        if first is None:
            first=(len(arcs),v,list(chords),len(bands),sum(arcs[i][4] for i in chords))
        if sum(a[4] for a in bands)==1:
            break
        v,arcs=len(chords),bands
    # Include even an unreduced nonempty level only if it actually runs a forest reduction.
    return totals,first


def find():
    refine=None
    tie=None
    for n in range(3,14):
        for L in product(range(2),repeat=n):
            f=lf(L)
            x=f[0]
            period=1
            while x:
                period+=1
                x=f[x]
            if period!=n:
                continue
            d={i for i in range(1,n) if L[i-1]!=L[i]}
            dk=d|{i for i in range(1,n) if i not in d and f[i] in d}
            a=counts(d,f)
            b=counts(dk,f)
            if refine is None and b[0]<a[0] and b[0]>0:
                refine=("".join(map(str,L)),f,a,b)
            b=counts(d,f,reverse_ties=True)
            if tie is None and a[1] and b[1] and (a[0],a[1][3])!=(b[0],b[1][3]):
                tie=("".join(map(str,L)),f,a,b)
            if refine and tie:
                print("refining decreases total",refine)
                print("tie perturbation",tie)
                return
    assert False,(refine,tie)


if __name__=="__main__":
    find()
