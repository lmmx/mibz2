"""Exact supported work/storage frontier for disjoint complete-subtree cuts."""
from collections import defaultdict
from fractions import Fraction
import csv
from itertools import groupby
import lzma
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent

class Model:
    def __init__(self, rows):
        self.rows = rows
        self.n = len(rows)
        self.parent, self.depth, self.sel, self.r, self.g = (
            [row[k] for row in rows] for k in ("parent","depth","S","R","G"))
        self.sub = self.r.copy()
        self.children = [[] for _ in rows]
        for v in range(self.n-1,0,-1):
            p = self.parent[v]
            self.sub[p] += self.sub[v]
            self.children[p].append(v)

    def count(self, stops):
        """Return cursor visits, cut reads, materialization writes, extra pieces, cut count."""
        query = reads = built = 0
        stack = [0]
        cut_count = 0
        while stack:
            v = stack.pop()
            if v in stops:
                cut_count += 1
                built += self.sub[v]
                if v != 0:
                    reads += self.g[v]
            else:
                query += self.g[v]
                stack.extend(self.children[v])
        mem = built - (self.r[0] if 0 in stops else 0)
        return query,reads,built,mem,cut_count

    def optimize(self, penalty):
        # Minimize qW+pM with ties broken by smaller M. R(root) is stored by all
        # plans. Selecting the root writes it directly to the shared output.
        p, q = penalty.numerator, penalty.denominator
        mem = [0]*self.n
        work = self.g.copy()
        for v in range(self.n-1,-1,-1):
            m = self.sub[v] - (self.r[0] if v == 0 else 0)
            w = self.sub[v] + (0 if v == 0 else self.g[v])
            delta = q*(w-work[v]) + p*(m-mem[v])
            if delta < 0 or (delta == 0 and m < mem[v]):
                mem[v],work[v] = m,w
            if v:
                u = self.parent[v]
                mem[u] += mem[v]
                work[u] += work[v]
        return mem[0],work[0]

    def hull(self):
        # Probe the slope of two adjacent known points. A lower optimum
        # partitions the segment, otherwise the segment is supported.
        first = (0,sum(self.g))
        last = self.optimize(Fraction(0))
        found = {first,last}
        segments = [(first,last)]
        while segments:
            a,b = segments.pop()
            if a == b:
                continue
            penalty = Fraction(a[1]-b[1], b[0]-a[0])
            c = self.optimize(penalty)
            if c != a and c != b:
                assert a[0] < c[0] < b[0]
                found.add(c)
                segments.extend(((a,c),(c,b)))
        return sorted(found)

def load(path):
    with lzma.open(path, "rt") as f:
        records = ({k:int(v) for k,v in r.items()} for r in csv.DictReader(f))
        return {block:Model(list(g)) for block,g in groupby(records,key=lambda r:r["block"])}

if __name__ == "__main__":
    models = load(sys.argv[1])
    out = Path(sys.argv[2])
    with (out/"frontier.csv").open("w") as h,(out/"depth.csv").open("w") as d:
        hull = csv.writer(h,lineterminator="\n")
        dep = csv.writer(d,lineterminator="\n")
        hull.writerow(["block","extra_pieces","work"])
        dep.writerow(["block","cut_depth","route_visits","cut_reads","materialized_writes","extra_pieces","subtrees","work"])
        for block,model in models.items():
            frontier = model.hull()
            hull.writerows((block,*pair) for pair in frontier)
            for depth in [*range(max(model.depth)+1),"never"]:
                stops = {v for v,h in enumerate(model.depth) if h == depth}
                parts = model.count(stops)
                dep.writerow([block,depth,*parts,sum(parts[:3])])
