"""Exhaust arbitrary sections, distinguishing cylinder sections, and record tight witnesses."""
from bisect import bisect_right
from itertools import groupby, product
import json
from gaps import compile_runs


def tokens(lf, col, lo, hi):
    words = []
    for start in range(lo,hi):
        x=start
        rev=[]
        while True:
            rev.append(col[x])
            x=lf[x]
            if lo<=x<hi:
                words.append(tuple(reversed(rev)))
                break
    return words


def reverse_boundaries(top,bottom,lo,hi):
    """FM cylinders have no membership-divergence seeds in a legal forward path."""
    ends=[b for _,b,_,_ in bottom]
    cuts=[lo]
    for seed,_,_,_ in top[1:]:
        gap=seed
        while True:
            if lo<gap<hi:
                cuts.append(gap)
                break
            if gap in (lo,hi):
                break
            a,_,_,old=bottom[bisect_right(ends,gap)]
            if gap==a:
                break
            gap+=old-a
    return sorted(cuts)


def study():
    res = {s:{"tested":0,"max_excess":-100,"witness":None} for s in ("arbitrary","word","single-byte")}
    for alphabet,limit in ((2,11),(3,7)):
        for n in range(1,limit+1):
            for col in product(range(alphabet),repeat=n):
                runs=[(c,len(list(g))) for c,g in groupby(col)]
                r=len(runs)
                _,_,_,top,bottom=compile_runs(runs)
                lf=[0]*n
                for a,b,_,e in top:
                    for x in range(a,b): lf[x]=e+x-a
                # Refine until there is no change; equal infinite rotations remain tied.
                word_sections={}
                frontier=[(0,n,())]
                while frontier:
                    nex=[]
                    for a,b,suffix in frontier:
                        for c in set(col[a:b]):
                            pts=[lf[j] for j in range(a,b) if col[j]==c]
                            first,last=min(pts),max(pts)+1
                            assert len(pts)==last-first
                            pat=(c,)+suffix
                            if (first,last) not in word_sections:
                                word_sections[first,last]=pat
                                nex.append((first,last,pat))
                    frontier=nex
                    if not nex: break
                for lo in range(n):
                    for hi in range(lo+1,n+1):
                        w=tokens(lf,col,lo,hi)
                        t=sum(1 for _ in groupby(w))
                        # Split exactly those original LF images strictly crossed by an endpoint.
                        extra=sum(a<g<b for g in (lo,hi) for a,b,_,_ in bottom)
                        assert t<=r+extra
                        pattern=word_sections.get((lo,hi))
                        which=["arbitrary"]
                        if pattern is not None:
                            which.append("word")
                            assert t<=r
                            starts=[lo+i for i,w0 in enumerate(w) if i==0 or w0!=w[i-1]]
                            assert reverse_boundaries(top,bottom,lo,hi)==starts
                            if len(pattern)==1:
                                which.append("single-byte")
                                assert t<=r
                        for s in which:
                            row=res[s]
                            row["tested"]+=1
                            if r>1 and (s!="word" or len(pattern)>1):
                                if t-r>row.get("nontrivial_max",-100):
                                    row["nontrivial_max"]=t-r
                                    row["nontrivial"]={"column":col,"section":[lo,hi],"pattern":pattern,"r":r,"t":t}
                            if t-r>row["max_excess"]:
                                row["max_excess"]=t-r
                                row["witness"]=dict(column=col,section=[lo,hi],r=r,t=t,split=extra,
                                                    pattern=pattern,word_runs=[(list(k),len(list(g))) for k,g in groupby(w)])
    print(json.dumps(res,indent=2))
if __name__=="__main__":study()
