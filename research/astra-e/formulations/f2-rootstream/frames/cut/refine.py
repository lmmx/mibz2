"""Supported mixed-materialization frontier after maximal subtree frames and leaf absorption."""
import csv
from fractions import Fraction
from itertools import groupby
import lzma
from pathlib import Path
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]/"cut"))
from data import ROOT,compact,read_input,routes,spell
from frontier import Model,load

def local_counts(labels,paths):
    g_self=[0]*len(paths)
    k=[0]*len(paths)
    last=[-2]*len(paths)
    for i,(t,_) in enumerate(groupby(labels)):
        g_self[t]+=1
        for v in paths[t]:
            if last[v]+1!=i:
                k[v]+=1
            last[v]=i
    return g_self,k

class Framed(Model):
    def __init__(self,original,own,intervals):
        super().__init__(original.rows)
        self.lookup=self.g.copy()
        self.g=[own[v]+sum(intervals[c] for c in self.children[v]) if self.children[v] or v==0 else 0
                for v in range(self.n)]
        self.sub=[r if self.children[v] or v==0 else 0 for v,r in enumerate(self.r)]
        for v in range(self.n-1,0,-1):
            self.sub[self.parent[v]]+=self.sub[v]
        assert sum(self.g)==self.r[0]+sum(intervals[v] for v in range(1,self.n) if self.children[v])

    def optimize(self,penalty):
        p,q=penalty.numerator,penalty.denominator
        mem=[0]*self.n
        work=self.g.copy()
        for v in range(self.n-1,-1,-1):
            m=self.sub[v]-(self.r[0] if v==0 else 0)
            w=self.sub[v]+(0 if v==0 else self.lookup[v])
            delta=q*(w-work[v])+p*(m-mem[v])
            if delta<0 or (delta==0 and m<mem[v]):
                mem[v],work[v]=m,w
            if v:
                u=self.parent[v];mem[u]+=mem[v];work[u]+=work[v]
        return mem[0],work[0]

def store_csv(path):
    models=load(HERE.parents[1]/"cut/nodes.csv.xz")
    with lzma.open(path,"wt",newline="") as f:
        w=csv.writer(f,lineterminator="\n");w.writerow(["block","node","self_runs","membership_intervals"])
        for p in sorted((ROOT/"research/astra-e/dense/target/wikidata").glob("*.irec.xz")):
            b=int(p.name.split(".")[0]);model=models[b]
            runs,_=read_input(p)
            unique,labels,_=spell(b"".join(bytes([c])*w for c,w in runs),ord('"'))
            parent,terminal=compact(unique)
            assert parent==model.parent
            own,k=local_counts((terminal[v] for v in labels),routes(parent))
            w.writerows((b,i,g,k[i]) for i,g in enumerate(own))

def frontier(path):
    models=load(HERE.parents[1]/"cut/nodes.csv.xz")
    results={}
    with lzma.open(path,"rt") as f:
        for b,rows in groupby(csv.DictReader(f),key=lambda r:int(r["block"])):
            rows=list(rows)
            results[b]=Framed(models[b],[int(r["self_runs"]) for r in rows],
                              [int(r["membership_intervals"]) for r in rows])
    w=csv.writer(sys.stdout,lineterminator="\n");w.writerow(["block","extra_pieces","work"])
    for b,model in results.items():
        for m,c in model.hull():
            w.writerow([b,m,c])

if __name__=="__main__":
    op,path=sys.argv[1:]
    if op=="collect":
        store_csv(path)
    elif op=="frontier":
        frontier(path)
    else:
        raise ValueError(op)
