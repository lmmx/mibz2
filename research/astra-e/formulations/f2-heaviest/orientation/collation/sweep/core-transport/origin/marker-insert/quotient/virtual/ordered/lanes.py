"""Verify the explicit four-run single-cycle family and its monotone-lane lower bound."""
from bisect import bisect_left
import csv
import sys
from cursor_order import Graph,Quotient,full

def lis(seq):
    tails=[]
    for x in seq:
        i=bisect_left(tails,x)
        if i==len(tails):tails.append(x)
        else:tails[i]=x
    return len(tails)

writer=csv.writer(sys.stdout,lineterminator='\n')
writer.writerow(('q','input_rows','input_runs','c_queries','lis','lds','lanes_at_least','optimal_cursors'))
for q in [*range(3,65),96,128]:
    a=q*q;k=q+1;m=a+1
    raw=[(2,1),(0,m),(2,k-1),(1,a)]
    freq={0:m,1:a,2:k}
    token=Quotient(raw,{c:bytes([c]) for c in freq},list(freq),freq,{0},0)
    idx=full(token)
    # Every ordinary LF arc belongs to the same cycle, independently of the trie.
    lf=[x+j for _,width,x in idx.segments(0,idx.end) for j in range(width)]
    seen=set();x=0
    while x not in seen:seen.add(x);x=lf[x]
    assert x==0 and len(seen)==len(lf)==m+a+k
    g=Graph(idx,token.bases,list(freq.values()),0)
    assert g.parent==list(range(-1,len(g.parent)-1))
    ordinals=[v-m for w,v in zip(g.word,g.pos) if w==1]
    assert ordinals==[(-k*(j+1))%a for j in range(a)]
    assert sorted(ordinals)==list(range(a))
    inv=[0]*a
    for j,v in enumerate(ordinals):inv[v]=j
    assert inv==[((q-1)*v-1)%a for v in range(a)]
    rising=lis(ordinals);falling=lis(-x for x in ordinals)
    assert rising<=q+1 and falling<=q
    wraps=sum(j>k for j,k in zip(inv,inv[1:]))
    assert wraps+1==q
    # Use the q disjoint increasing runs of inverse order as forward lanes.
    lane=0; last=[-1]*q
    assign=[]
    for i,j in enumerate(inv):
        if i and j<inv[i-1]:lane+=1
        assign.append(lane)
    for x in ordinals:
        lane=assign[x]
        assert last[lane]<x
        last[lane]=x
    width=max(rising,falling)
    writer.writerow((q,len(lf),len(raw),a,rising,falling,(a+width-1)//width,q))
