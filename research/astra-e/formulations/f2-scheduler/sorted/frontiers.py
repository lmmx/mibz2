"""Sort-free byte-depth frontiers, checked against the native contracted trie on each input."""
from bisect import bisect_right
from collections import Counter,defaultdict
import csv
import itertools
import sys
from pathlib import Path
here=Path(__file__).resolve().parent
form=here.parents[1]
sys.path.insert(0,str(form/"f2-dual"))
from gaps import compile_runs
from check import rle


def build(runs,d):
    n,base,count,top,bottom=compile_runs(runs)
    ends=[b for a,b,c,y in top]
    level=[(base[d],base[d]+count[d])] if count[d] else []
    totals=Counter()
    depths=[]
    while level:
        next_by_byte=defaultdict(list)
        cnt=Counter()
        prev_k=prev_page=prev_lo=None
        for lo,hi in level:
            k=bisect_right(ends,lo)
            cnt["queries"]+=1
            cnt["unique_runs"]+=k!=prev_k
            cnt["unique_pages"]+=lo//64!=prev_page
            if prev_k is not None:
                assert k>=prev_k and lo>=prev_lo
                gap=k-prev_k
                for threshold in (1,2,4,8):
                    if gap<=threshold:
                        cnt[f"savings_{threshold}"]+=1
                        cnt[f"advance_{threshold}"]+=gap
            prev_k,prev_page,prev_lo=k,lo//64,lo
            a,b,c,y=top[k]
            if hi<=b and c!=d:
                first=y+lo-a
                next_by_byte[c].append((first,first+hi-lo))
                cnt["unary"]+=1
                continue
            cnt["nodes"]+=1
            children={}
            while k<len(runs) and top[k][0]<hi:
                a,b,c,y=top[k]
                aa,bb=max(lo,a),min(hi,b)
                cnt["selectors"]+=1
                if c!=d:
                    dest=y+aa-a
                    if c in children:
                        first,end=children[c];assert dest==end
                    else:
                        first=dest
                    children[c]=(first,dest+bb-aa)
                k+=1
            for c,interval in children.items():
                next_by_byte[c].append(interval)
        cnt["buckets"]=len(next_by_byte)
        level=list(itertools.chain.from_iterable(next_by_byte[c] for c in sorted(next_by_byte)))
        assert all(a[1]<=b[0] for a,b in zip(level,level[1:]))
        totals.update(cnt)
        depths.append(cnt)
    return totals,depths

if __name__=="__main__":
    if len(sys.argv)==1:
        cases=nodes=0
        for alpha,m in ((2,8),(3,6)):
            for n in range(1,m+1):
                for col in itertools.product(range(alpha),repeat=n):
                    for d in range(alpha):
                        total,levels=build(rle(col),d)
                        # Independently form exact reachable I(w) intervals against stable dense LF.
                        psi=sorted(range(n),key=col.__getitem__)
                        intervals=set()
                        for x in range(n):
                            y=x;prefix=[]
                            for _ in range(n):
                                y=psi[y];prefix.append(col[y])
                                if col[y]==d:
                                    intervals.add(tuple(prefix));break
                        assert len(intervals)==total["queries"]
                        cases+=1
                        nodes+=len(intervals)
        print(f"{cases} exhaustive columns/delimiters, {nodes} reachable source intervals matched")
    else:
        sys.path.insert(0,str(form.parents[2]/"docs/journal/2026-11-18-astra-c-output-first"))
        from inputs import read_input,paths
        with (form/"f2b/geometry.csv").open() as f:
            profile={(int(r["rows"]),int(r["primary"])):r for r in csv.DictReader(f)}
        fields="queries unique_runs unique_pages unary nodes selectors buckets savings_1 advance_1 savings_2 advance_2 savings_4 advance_4 savings_8 advance_8".split()
        w=csv.writer(sys.stdout,lineterminator="\n")
        w.writerow(["block","stream","depth",*fields])
        for path in paths(sys.argv[1]):
            runs,p=read_input(path)
            n=sum(width for _,width in runs)
            match=profile[n,p]
            total,depths=build(runs,34)
            if match["took"]=="1":
                assert total["nodes"]==int(match["nodes"])
                assert total["queries"]==int(match["lookups"])
                assert total["selectors"]==int(match["selectors"])
            for depth,measured in enumerate(depths):
                w.writerow([path.name.split(".")[0],match["stream"],depth,*[measured[k] for k in fields]])
