from array import array
from bisect import bisect_right
from collections import Counter
from hashlib import sha256
from itertools import accumulate
from pathlib import Path
import sys

from boundary import section


def data(path):
    column=Path(path).read_bytes()
    assert sha256(column).hexdigest()=="db8764224ac4980112ddefbe85b8593f5236643fed71bf8f0b739567d4f2ac67"
    counts=Counter(column)
    buckets=list(accumulate([0]+[counts[i] for i in range(256)]))
    lf=array("i")
    for byte in column:
        lf.append(buckets[byte])
        buckets[byte]+=1
    return column,lf


def arclist(boundaries,lf):
    arcs=[]
    for u,(x,end) in enumerate(zip(boundaries,boundaries[1:])):
        y=lf[x]
        target=bisect_right(boundaries,y)-1
        for off_start in [x]:
            while x<end:
                width=min(end-x,boundaries[target+1]-y)
                arcs.append((u,target,x-off_start,y-boundaries[target],width))
                x,y=x+width,y+width
                target+=1
    return arcs


if __name__=="__main__":
    column,lf=data(sys.argv[1])
    n=len(column)
    starts={i for i in range(1,n) if column[i-1]!=column[i]}
    prev=array("i",[-1])*n
    for x in range(1,n):
        if x not in starts:
            g=lf[x]
            assert g==lf[x-1]+1 and prev[g]==-1
            prev[g]=x
    cuts,frontier=set(starts),starts
    marked=264045
    wanted={1,2,3,4,5,7,10,16,32}
    print("depth,sites,arcs,excess,first_retained,levels,total_arcs",flush=True)
    for k in range(1,max(wanted)+1):
        next_=set(prev[x] for x in frontier if prev[x]>=0)
        assert not cuts.intersection(next_)
        if k in wanted:
            bounds=[0,*sorted(cuts),n]
            deg=len(bounds)-1
            arcs=arclist(bounds,lf)
            E=len(arcs)
            assert E==deg+len(next_)
            u=bisect_right(bounds,marked)-1
            mark=(u,marked-bounds[u])
            total,levels,first=0,0,None
            while result:=section(arcs,deg,mark):
                chords,new,mark,_,_=result
                if first is None:
                    first=sum(arcs[i][4] for i in chords)
                total+=len(arcs)
                levels+=1
                if sum(a[4] for a in new)==1:
                    break
                arcs,deg=new,len(chords)
            print(k,len(bounds)-1,E,len(next_),first,levels,total,sep=",",flush=True)
        cuts.update(next_)
        frontier=next_
