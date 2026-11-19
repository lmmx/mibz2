"""Check the local-rank theorem from the dense survivor source table, across arbitrary cuts."""
from collections import Counter
from itertools import product
import json
def solve():
    total=Counter()
    for n in range(1,8):
        for last in product(range(3),repeat=n):
            k=len(set(last))
            if set(last)!=set(range(k)):continue
            first=sorted(last)
            nxt=sorted(range(n),key=last.__getitem__)
            pred=[nxt.index(i) for i in range(n)]
            bases=[first.index(a) for a in range(k)]
            for mask in range(1<<k):
                inside=lambda a:((mask>>a)&1)!=0
                dead={nxt[p] for p in range(n) if not inside(first[p]) and inside(first[nxt[p]])}
                kept=[p for p in range(n) if p not in dead]
                aliases=[(first[p],-1 if inside(first[p]) else first[nxt[p]]) for p in kept]
                assert aliases==sorted(aliases)
                start={a:aliases.index(a) for a in set(aliases)}
                total['cuts']+=1
                for x in range(n):
                    c=first[x];a=last[x]
                    total['marks']+=1
                    assert (x in dead)==(not inside(a) and inside(c))
                    if x not in dead:continue
                    sub=last[bases[c]:x].count(a)
                    new=start[a,c]+sub
                    assert kept[new]==pred[x] and aliases[new]==(a,c)
                    total['moved']+=1
                    total['positive_within_bucket']+=int(sub>0)
                    total['lower_source']+=int(a<c)
                    total['higher_source']+=int(a>c)
    return total
if __name__=='__main__':print(json.dumps(solve(),sort_keys=True,indent=2))
