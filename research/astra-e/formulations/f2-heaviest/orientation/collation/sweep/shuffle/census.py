"""Count the follower equivalence classes from incoming labels, without building a dense return permutation."""
from collections import Counter, defaultdict
from pathlib import Path
from itertools import groupby
import csv
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]))
from order import Column, Oracle, heavy_set, truth
sys.path.insert(0,str(HERE.parents[7]/"docs/journal/2026-11-18-astra-c-output-first"))
from inputs import read_input, paths


class Classes:
    def __init__(self,n):
        self.parent=list(range(n))

    def root(self,x):
        while self.parent[x]!=x:
            self.parent[x]=self.parent[self.parent[x]]
            x=self.parent[x]
        return x

    def merge(self,x,y):
        x,y=self.root(x),self.root(y)
        if x!=y: self.parent[y]=x


def labels(oracle,section):
    words,incoming=truth(oracle,section)
    order=sorted(words,key=lambda w:words[w][0][0])
    counts=[len(words[w]) for w in order]
    src=[i for i,count in enumerate(counts) for _ in range(count)]
    code={w:i for i,w in enumerate(order)}
    inc=[code[w] for w in incoming]
    return order,counts,src,inc


def follower_classes(order,src,inc):
    dsu=Classes(len(order)); first={}; followers=defaultdict(set)
    for prior,current in zip(inc,src,strict=True):
        followers[prior].add(current)
        if prior in first:dsu.merge(first[prior],current)
        else:first[prior]=current
    roots={};cls=[]
    for i in range(len(order)):
        root=dsu.root(i);cls.append(roots.setdefault(root,len(roots)))
    after=[cls[first[i]] for i in range(len(order))]
    assert all(len({cls[j] for j in followers[i]})==1 for i in range(len(order)))
    return cls,after,followers


def main():
    rows=csv.writer(sys.stdout,lineterminator="\n")
    rows.writerow("ordinal section mass types follower_classes largest_types largest_visits multi_followers length_one_orbits singleton_classes".split())
    for p in paths(sys.argv[1]):
        runs,mark=read_input(p)
        col=Column(runs)
        oracle=Oracle(b"".join(bytes([c])*w for c,w in runs))
        quote=col.interval(b'"')
        deep=heavy_set(col,quote,1)[0][0]
        for name,sec in [("quote",quote),("frequent",deep)]:
            words,counts,src,inc=labels(oracle,sec)
            cls,after,followers=follower_classes(words,src,inc)
            groups=Counter(cls); visits=Counter(cls[x] for x in src)
            rows.writerow([int(p.name.split(".")[0]),name,len(src),len(words),len(groups),
                           max(groups.values()),max(visits.values()),
                           sum(len(v)>1 for v in followers.values()),
                           sum(v for w,v in zip(words,counts) if len(w)==1),
                           sum(v==1 for v in groups.values())])
            sys.stdout.flush()


if __name__=="__main__":
    main()
