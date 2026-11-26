"""Exercise recursive frame semantics on all contracted and uncontracted small trees."""
from collections import Counter
from itertools import product
import json
from intervals import check

c=Counter()
for n,limit in ((1,7),(2,7),(3,7),(4,6),(5,5)):
    for tail in product(*(range(i) for i in range(1,n))):
        for m in range(1,limit+1):
            for labels in product(range(n),repeat=m):
                result=check(labels,(-1,)+tail)
                if result is None: continue
                c["tree_tapes"]+=1
                for name,value in zip(("rows","nodes","selectors","root_runs","membership_intervals",
                                      "frames","overlaps"),result):
                    c[name]+=value
print(json.dumps(c,indent=2))
