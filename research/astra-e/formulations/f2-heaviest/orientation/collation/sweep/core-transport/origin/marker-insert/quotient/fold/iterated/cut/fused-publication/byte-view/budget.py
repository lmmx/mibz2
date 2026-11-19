"""Price the new first root against the exact residual completion budget."""
from pathlib import Path
import csv,statistics,math,json
from collections import defaultdict,Counter
here=Path(__file__).resolve().parent
rows=list(csv.DictReader((here/'capped/capped.csv').open()))
def ci(n,d):
    x=[math.log(a/b) for a,b in zip(n,d)];g=[statistics.mean(x[i:i+6]) for i in range(0,len(x),6)]
    m=statistics.mean(g);hw=3.1824463053*statistics.stdev(g)/2
    return [round(math.exp(v),6) for v in (m,m-hw,m+hw)]
out={}
for tag in ['all',*sorted({q['block'] for q in rows},key=int)]:
    t=defaultdict(Counter)
    for x in rows:
        if tag!='all' and x['block']!=tag:continue
        for k in ['thread_ns','first_paid_ns','cut_only_ns']:
            if x[k]:t[int(x['round']),x['arm']][k]+=int(x[k])
    seq=lambda a,k:[t[r,a][k] for r in range(1,25)]
    s=seq('scalar','thread_ns');f=seq('byte_residual','first_paid_ns');y=seq('byte_residual','thread_ns')
    j=seq('joint','first_paid_ns');g=seq('byte_grammar','first_paid_ns')
    # Convert all individual round times into the same work units using the
    # aggregate measured scalar CPU for that round. Residual removal is signed
    # even on a rare individual round where the paid prefix already loses.
    left=[(a-b)/a for a,b in zip(s,f)]
    reduction=[1-(a-b)/(c-b) for a,b,c in zip(s,f,y)]
    remaining=[c-b for b,c in zip(f,y)]
    out[tag]={'new_first/explicit_first':ci(g,j),'direct_remaining/scalar':ci(remaining,s),
              'rounds_with_positive_remaining_budget':sum(v>0 for v in left),
              'median_remaining_scalar_share':round(statistics.median(left),5),
              'median_fraction_of_direct_residual_cost_to_remove':round(statistics.median(reduction),5)}
print(json.dumps(out,indent=2))
