"""Check the sharp maximum circular tape count in all binary columns through 12 rows."""
import csv,sys
from itertools import product
from numeric import components,index
out=csv.writer(sys.stdout,lineterminator="\n");out.writerow(["n","maximum_distinct","shortest_lengths_bound","witness_L"])
pool=[]
for n in range(1,13):
    primitive=0;best=-1
    for L in product((0,1),repeat=n):
        cs=components(*index(L));primitive+=len(cs)==1
        h=len(set(cs))
        if h>best:best=h;witness=L
    pool.extend([n]*primitive)
    total=count=0
    for d in pool:
        if total+d>n:break
        total+=d;count+=1
    assert count==best
    out.writerow([n,best,count,''.join(map(str,witness))])
