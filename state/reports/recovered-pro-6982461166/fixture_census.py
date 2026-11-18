from array import array
from collections import Counter
from itertools import accumulate
from pathlib import Path


column=Path("frozen.L").read_bytes()
n=len(column)
counts=Counter(column)
next_=list(accumulate([0]+[counts[b] for b in range(256)]))
forward=array("I")
for byte in column:
    forward.append(next_[byte])
    next_[byte]+=1
phase=array("i",[-1])*n
row=264045
for j in range(n-1,-1,-1):
    assert phase[row]==-1
    phase[row]=j
    row=forward[row]
assert row==264045
delta=array("i",[n])*n
for x in range(1,n):
    if column[x]==column[x-1]:
        delta[phase[x]]=phase[x-1]-phase[x]
spans=sum((i==0 or d!=delta[i-1]) and d!=n for i,d in enumerate(delta))
copied=sum(d!=n for d in delta)
assert copied==848747
print(spans,copied,n-copied)
