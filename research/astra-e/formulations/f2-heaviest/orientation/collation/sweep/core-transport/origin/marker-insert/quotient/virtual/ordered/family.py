"""An actual five-run BWT with a unique heaviest pivot and arbitrarily many cursor turns."""
from collections import Counter
from itertools import groupby
import csv
import sys
from order import Graph,Quotient,full

writer=csv.writer(sys.stdout,lineterminator='\n')
writer.writerow(('t','rows','runs','heaviest','c_lookups','c_required_ascending_resets','c_required_descending_resets','c_required_turns'))
for t in list(range(1,65))+[96,128,256,1024]:
    m=2*t+2
    rle=[(1,1),(0,m-1),(2,t),(0,1),(1,2*t)]
    # The cyclic text d^m (ccx)^t c has exactly the above last column.
    if t<=128:
        word=bytes([0])*m+bytes([1,1,2])*t+bytes([1])
        rotations=sorted(range(len(word)),key=lambda i:word[i:]+word[:i])
        last=[word[i-1] for i in rotations]
        assert [(k,sum(1 for _ in v)) for k,v in groupby(last)]==rle
    freq={0:m,1:2*t+1,2:t}
    q=Quotient(rle,{i:bytes([i]) for i in freq},list(freq),freq,{0},0)
    g=Graph(full(q),q.bases,list(freq.values()),0)
    req,up,down,turn=g.forced()
    assert max(freq,key=freq.get)==0
    assert len(g.buckets[1])==2*t+1 and req[1]==3
    assert up[1]==down[1]==t and turn[1]==2*t-1
    writer.writerow((t,sum(freq.values()),len(rle),0,len(g.buckets[1]),up[1],down[1],turn[1]))
