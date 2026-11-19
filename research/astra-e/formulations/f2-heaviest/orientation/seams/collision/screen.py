"""Optimistic first screen: maximum coalescence of unlabelled active ranks; no return-length oracle."""
import argparse
from pathlib import Path
import sys
import numpy as np
ROOT=Path(__file__).resolve().parents[7]
sys.path.insert(0,str(ROOT/"docs/journal/2026-11-18-astra-c-output-first"))
sys.path.insert(0,str(ROOT/"research/astra-e/formulations/f2-heaviest/union"))
from inputs import read_input
from direct import Column,heavy_set


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument("tape",type=Path);a=p.parse_args()
 runs,mark=read_input(a.tape); col=Column(runs); n=col.rows
 l=np.concatenate([np.full(k,c,dtype=np.uint8) for c,k in runs])
 psi=np.argsort(l,kind="stable");lf=np.empty(n,np.int32);lf[psi]=np.arange(n,dtype=np.int32)
 pieces_l=np.repeat(np.arange(len(runs),dtype=np.int32),[k for _,k in runs]);pieces_f=pieces_l[psi]
 q=col.interval(b'"');sections=[("quote",q),("compound",heavy_set(col,q,1)[0][0])]
 print("section,mode,visits,return_edge_steps,occupied_advance_pieces,unlabelled_stored_intervals,intersection_input_intervals,matched_row_count,phases")
 for section,(lo,hi) in sections:
  s=np.arange(lo,hi,dtype=np.int32)
  for mode in ["backward","collision"]:
   f,b=s.copy(),s.copy();count=[0]*5;ph=0
   def seg(x):
    return int(len(x)>0)+np.count_nonzero(np.diff(x)!=1)
   while len(b):
    front=(mode=="collision" and ph%2==0);x=f if front else b
    step,idx=(psi,pieces_f) if front else (lf,pieces_l)
    cost=1+np.count_nonzero((np.diff(x)!=1)|(np.diff(idx[x])!=0))
    count[0]+=len(x);count[1]+=int(cost);count[2]+=seg(x)
    y=np.sort(step[x]);op=b if front else f
    count[3]+=seg(y)+seg(op)
    common=np.intersect1d(y,op,assume_unique=True)
    survivor=np.setdiff1d(y,common,assume_unique=True)
    if mode=="collision":
     rem=np.setdiff1d(op,common,assume_unique=True)
     if front:f,b=survivor,rem
     else:b,f=survivor,rem
    else:b=survivor
    count[4]+=len(common);ph+=1
   assert count[4]==len(s) and count[0]<=n
   print(section,mode,len(s),*count,ph,sep=",")


if __name__=="__main__":main()
