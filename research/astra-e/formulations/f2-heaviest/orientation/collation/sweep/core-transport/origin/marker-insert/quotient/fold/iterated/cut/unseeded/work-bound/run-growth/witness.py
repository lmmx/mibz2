"""Independent stable-BWT oracle and exact convex bound for repeated disjoint alphabets."""
from collections import Counter
import csv
from fractions import Fraction
from itertools import groupby, product
from pathlib import Path
import sys

LAST="11111133333333355500000000222224"


def test(last):
    last=tuple(map(int,last))
    first=sorted(last);p=sorted(range(len(last)),key=last.__getitem__)
    ix=0;word=[]
    while True:
        word.append(first[ix]);ix=p[ix]
        if ix==0:break
    assert len(word)==len(last)
    rotations=sorted(range(len(word)),key=lambda i:word[i:]+word[:i])
    assert [word[(i-1)%len(word)] for i in rotations]==list(last)
    assert all(first[x]!=first[p[x]] for x in range(len(last)))
    k=1+max(last)
    data=[]
    for mask in range(1<<k):
        pred={x for x in range(len(last)) if not (mask>>first[x])&1 and (mask>>first[p[x]])&1}
        dead={p[x] for x in pred}
        after={}
        for x in range(len(last)):
            if x in dead:continue
            y=p[x]
            dst=p[y] if y in dead else y
            tag=(first[x],-1 if (mask>>first[x])&1 else first[y])
            after[dst]=tag
        assert len(after)==len(last)-len(dead)
        ordered=[after[i] for i in sorted(after)]
        runs=sum(1 for _ in groupby(ordered))
        data.append((mask,len(dead),runs))
    return word,data


if __name__=='__main__':
    word,rows=test(LAST)
    n=len(word);r=sum(1 for _ in groupby(LAST))
    assert min(a for _,q,a in rows if 4*q>=n)>r
    lower=[(Fraction(a),i,i) for i,q,a in rows if 4*q>=n]
    for i,x,a in rows:
        for j,y,b in rows:
            if 4*x<n<4*y:
                t=(Fraction(n,4)-x)/(y-x)
                lower.append((a+t*(b-a),i,j))
    v,i,j=min(lower)
    assert all(5*a>=q+23 for _,q,a in rows) and v==Fraction(31,5)
    assert rows[45][1:]==(7,6) and rows[5][1:]==(12,7)
    print("true_cyclic_word="+''.join(map(str,word)),f"n={n} R={r} threshold={Fraction(n,4)}")
    print("all_orientations="+str(len(rows)), "qualified="+str(sum(4*q>=n for _,q,a in rows)),
          "best_qualified_runs="+str(min(a for _,q,a in rows if 4*q>=n)))
    print(f"minimum_mix_runs={v} masks={i},{j}")
    wr=csv.writer(sys.stdout,lineterminator='\n')
    wr.writerow(('mask','ingress','next_runs'))
    wr.writerows(rows)
