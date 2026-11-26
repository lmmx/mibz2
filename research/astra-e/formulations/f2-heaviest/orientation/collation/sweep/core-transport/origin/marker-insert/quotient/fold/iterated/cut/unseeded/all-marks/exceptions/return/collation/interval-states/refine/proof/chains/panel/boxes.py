"""Use integer arithmetic to bound independently feasible choices for explicit state budgets."""
import csv
import math
from collections import defaultdict
from pathlib import Path
import sys
src=Path(sys.argv[1])
path=defaultdict(list)
for row in csv.DictReader((src/'path-hist.csv').open()):
    p,g,l,k,c=map(int,row.values());path[p].append((g,l,k,c))
out=csv.writer(sys.stdout,lineterminator='\n')
out.writerow(['block','grounded_only_floor_log2','feasible_box_floor_log2','relaxed_space_ceil_log2',
              'optional_one_cut_paths','budget128_floor_log2','budget512_floor_log2','budget1024_floor_log2',
              'budget8192_floor_log2','budget32768_floor_log2'])
for row in csv.DictReader((src/'blocks.csv').open()):
    b=int(row['block']);q=path[b];sig=int(row['alphabet']);m=int(row['independent_optional_singletons'])
    a=math.prod(l**n for g,l,k,n in q if g)
    c=math.prod(k**n for g,l,k,n in q)
    z=math.prod((l+(1-g))**n for g,l,k,n in q)
    # Any subset of independent optional terminals over the F seams gives sigma+subset_size states.
    bits=[math.comb(m,min(cap-sig,m//2)).bit_length()-1 for cap in (128,512,1024,8192,32768)]
    ans=[a.bit_length()-1,c.bit_length()-1,(z-1).bit_length()]
    assert ans[1]>=int(row['feasible_integer_lower_exponent'])
    out.writerow([b,*ans,m,*bits])
