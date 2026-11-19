"""Dense independent raw LF, stopped-power maximality and interval replay."""
from bisect import bisect_right
from collections import Counter
from itertools import product
import json
from refine import Column,Table,double,replay

tot=Counter();sharp={}
for n in range(1,8):
    for vals in product((33,34,35),repeat=n):
        c=Column([(x<<8)|1 for x in vals])
        lf=[0]*n
        for i,j in enumerate(sorted(range(n),key=vals.__getitem__)): lf[j]=i
        paths,_=c.actual()
        for stop in (False,True):
            tables=[]
            for k in (1,2,3,4):
                t=Table(c,k,tables[-1] if tables else None,stop)
                tables.append(t)
                prev=None
                for x in range(n):
                    p=x;w=[];term=-1
                    for _ in range(k):
                        if stop and vals[p]==34:
                            term=c.find(p);break
                        w.append(vals[p]);p=lf[p]
                    have=(bytes(w),p-x,term)
                    d=bisect_right(t.starts,x)-1
                    assert have==(t.word[d],t.delta[d],t.terminal[d])
                    assert (prev!=have)==(t.starts[d]==x)
                    prev=have
                    tot["dense_rows"]+=1
                if stop and k in (2,4):
                    m,demand=replay(c,paths,t,k)
                    if k==4: double(c,tables[1],t,demand)
                if len(t.word)==1+k*(len(c.ids)-1) and k>=2 and len(c.ids)>1:
                    sharp.setdefault(str((stop,k)),list(vals))
        tot["columns"]+=1
print(json.dumps(dict(counts=tot,sharp_bound_witnesses=sharp),indent=2))
