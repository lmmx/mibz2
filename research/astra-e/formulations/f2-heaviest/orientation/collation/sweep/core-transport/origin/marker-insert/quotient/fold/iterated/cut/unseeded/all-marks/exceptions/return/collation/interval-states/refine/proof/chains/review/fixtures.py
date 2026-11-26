"""Small independent checks of free lane-offset replacement with arbitrary old seams."""
from bisect import bisect_right
from collections import Counter
from itertools import combinations

def check(last, p, starts, g, high, targets):
    n=len(last)
    psi=sorted(range(n),key=last.__getitem__)
    lf=[psi.index(i) for i in range(n)]
    seam={z for z in range(1,n) if last[psi[z-1]]!=last[psi[z]]}
    def score(t):
        q=sorted(p|t)
        for z in t:
            assert z in seam or any(psi[z-1]<v<=psi[z] for v in q)
        return sum(max(Counter((last[x],bisect_right(q,lf[x])) for x in range(l,r)).values())
                   for l,r in zip([0]+q,q+[n]))
    assert all(g+a<=n for a in starts)
    for offsets,want in targets.items():
        cuts={a+j for j in offsets for a in starts}
        assert score(cuts)==want
    for size in range(1,high):
        for offsets in combinations(range(1,high),size):
            old=score({a+j for j in offsets for a in starts})
            canonical=[(1,),(high-1,)] if size==1 else [(1,high-1)]
            assert old<=max(score({a+j for j in ends for a in starts}) for ends in canonical)
    print(last, sorted(p), targets)

check([0]*2+[1]*6+[0]*6,{7},[2,8],6,5,
      {():10,(1,):11,(2,):10,(3,):10,(4,):10,(2,3):10,(1,4):11})
check([1]*3+[0]*3+[1]*4+[0]*7,{3,4},[6,13],4,4,
      {():10,(1,):12,(2,):12,(3,):13,(1,2):13,(1,3):14,(2,3):13,(1,2,3):14})
