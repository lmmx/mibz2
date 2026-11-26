"""Price the full common-overhead envelope on matched wall observations."""
from collections import defaultdict
import csv
import json
from pathlib import Path
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[2]/"bench/moonshot"))
from moonshot import paired_ratio


def harmonic(a,b,p):
    return 1/(p/a+(1-p)/b)


def envelope(a,b,s,p):
    if a==b or s==min(a,b):
        return min(a,b)
    return s+harmonic(a-s,b-s,p)


def check():
    cases=0
    for a in (1,2,3,8):
        for b in (1,2,3,8):
            for p in (.1,.25,.5,.9):
                samples=[envelope(a,b,min(a,b)*i/64,p) for i in range(65)]
                assert all(x+1.e-12>=y for x,y in zip(samples,samples[1:]))
                assert abs(samples[0]-harmonic(a,b,p))<1.e-12
                assert abs(samples[-1]-min(a,b))<1.e-12
                # Common, perfectly overlapped scanner with unobserved raw worker times.
                for bound in (1,2,4,9):
                    aa,bb=max(a,bound),max(b,bound)
                    mixed=max(bound,harmonic(a,b,p))
                    assert min(aa,bb)-1.e-12<=mixed<=harmonic(aa,bb,p)+1.e-12
                    cases+=1
    print(f"{cases} model grids satisfy the nonincreasing additive envelope and the pipelined bound")


def study(rows,mixed,edge,eager,eager_count):
    cells=defaultdict(dict)
    for row in rows:
        if row["arm"] in (mixed,edge,eager):
            cells[row["workers"],row["round"]][row["arm"]]=row["wall_s"]
    result=[]
    for w in sorted({w for w,_ in cells}):
        active=[v for (workers,_),v in sorted(cells.items()) if workers==w]
        p=eager_count(w)/w
        actual=[r[mixed] for r in active]
        best=[min(r[eager],r[edge]) for r in active] if p<1 else [r[eager] for r in active]
        top=[harmonic(r[eager],r[edge],p) for r in active]
        lo=paired_ratio(actual,best);hi=paired_ratio(actual,top)
        result.append(dict(workers=w,eager_fraction=p,rounds=len(active),
                           mixed_over_faster=lo["geomean"],faster_lo=lo["lo"],faster_hi=lo["hi"],
                           mixed_over_harmonic=hi["geomean"],harmonic_lo=hi["lo"],harmonic_hi=hi["hi"],
                           below=sum(x<a for x,a in zip(actual,best)),above=sum(x>b for x,b in zip(actual,top)),
                           inside=sum(a<=x<=b for x,a,b in zip(actual,best,top))))
    return result


if __name__=="__main__":
    if len(sys.argv)==1:
        check()
    else:
        rows=list(map(json.loads,Path(sys.argv[1]).read_text().splitlines()))
        result=study(rows,"candidate",sys.argv[2],"cells",lambda w:(w+3)//4)
        writer=csv.DictWriter(sys.stdout,fieldnames=result[0],lineterminator="\n");writer.writeheader();writer.writerows(result)
