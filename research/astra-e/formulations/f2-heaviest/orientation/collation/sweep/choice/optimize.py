"""Exact finite model of type-dependent one-byte cuts with a stable-shuffle return interface."""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from itertools import product
from pathlib import Path
import sys
from time import monotonic

import numpy as np
from scipy.optimize import Bounds, LinearConstraint, linprog, milp
from scipy.sparse import coo_matrix, vstack

here = Path(__file__).resolve().parent
sys.path.insert(0, str(here.parent/'context'))
from shifts import Column, Oracle, heavy_set, objects, suffixes


def follower_partition(types, source, returns):
    parent = list(range(len(types)))
    def root(i):
        while parent[i] != i:
            parent[i] = parent[parent[i]]
            i = parent[i]
        return i
    following = [None]*len(types)
    for t, j in zip(source, returns):
        other = source[j]
        f = following[t]
        if f is None:
            following[t] = other
        else:
            parent[root(other)] = root(f)
    classes = {}
    groups = []
    for i in range(len(types)):
        q = root(i)
        groups.append(classes.setdefault(q, len(classes)+1))
    return groups, [groups[j] for j in following]


def normalized(*literals):
    values = set(literals)
    if any(-q in values for q in values):
        return None
    return tuple(sorted(values))


class Choice:
    def __init__(self, types, groups, after):
        self.types = types
        self.g = groups
        self.h = after
        self.n = max(groups)
        self.char = types[0][:1]
        assert all(w[:1] == self.char for w in types)
        # One node per distinct reversed suffix, independent of its owners.
        edge = {}
        owners = [set()]
        full = defaultdict(list)
        self.forbid = set()
        for word, g, h in zip(types, groups, after):
            for t in (0,1):
                core = word[1:] + self.char*t
                lit = h if t else -h
                node = 0
                for x in reversed(core):
                    if node:
                        owners[node].add((lit,))
                    key = (node,x)
                    if key not in edge:
                        edge[key] = len(owners)
                        owners.append(set())
                    node = edge[key]
                if node:
                    case = normalized(-g,lit)
                    if case is not None:
                        owners[node].add(case)
                for s in (0,1):
                    case = normalized(g if s else -g, lit)
                    if case is None:
                        continue
                    new = word[s:] + self.char*t
                    if new:
                        full[new].append(case)
                    else:
                        self.forbid.add(case)
        self.conflict = [Counter(cases) for cases in full.values() if len(cases)>1]
        # The empty suffix is always counted. Complementary singleton literals make
        # a state unconditional. Any singleton also subsumes its longer owner cases.
        self.constant = 1
        self.signatures = Counter()
        for needs in owners[1:]:
            single = {c[0] for c in needs if len(c)==1}
            if any(-x in single for x in single):
                self.constant+=1
                continue
            active = tuple(sorted(c for c in needs if len(c)==1 or not any(x in single for x in c)))
            if active:
                self.signatures[active]+=1

    def words(self,bits):
        return [w[bits[g-1]:]+self.char*bits[h-1] for w,g,h in zip(self.types,self.g,self.h)]

    def exact(self,bits):
        words = self.words(bits)
        if b'' in words or len(set(words)) != len(words):
            return None
        return suffixes(words),sum(map(len,words))

    def evaluate(self,bits):
        def value(case):
            return all(bool(bits[abs(x)-1])==(x>0) for x in case)
        if any(value(c) for c in self.forbid) or any(sum(k for c,k in cells.items() if value(c))>1 for cells in self.conflict):
            return None
        return self.constant + sum(n for sig,n in self.signatures.items() if any(value(c) for c in sig))

    def matrix(self):
        # Only the class bits are integral. The auxiliary conjunctions equal 0/1
        # whenever those bits are integral. For a suffix signature costs force the max.
        cost = [0.0]*self.n
        lower = []
        upper = []
        rows = []
        cols = []
        vals = []
        def col(c=0):
            cost.append(c)
            return len(cost)-1
        def constraint(terms,lo=-np.inf,hi=np.inf):
            i=len(lower)
            lower.append(lo)
            upper.append(hi)
            for j,v in terms:
                rows.append(i); cols.append(j); vals.append(v)
        scope = {c for sig in self.signatures for c in sig} | {c for vals0 in self.conflict for c in vals0} | self.forbid
        ids = {}
        for case in sorted(scope):
            if len(case)==1 and case[0]>0:
                ids[case]=case[0]-1
                continue
            z=col()
            ids[case]=z
            terms=[]
            neg=0
            for lit in case:
                j=abs(lit)-1
                sign=1 if lit>0 else -1
                neg+=lit<0
                terms.append((j,-sign))
                constraint(((z,1),(j,-sign)),hi=int(lit<0))
            constraint([(z,1)]+terms,lo=neg-len(case)+1)
        for sig,weight in self.signatures.items():
            if len(sig)==1:
                cost[ids[sig[0]]]+=weight
            else:
                y=col(weight)
                for c in sig:
                    constraint(((y,1),(ids[c],-1)),lo=0)
        for c in sorted(self.forbid):
            constraint(((ids[c],1),),hi=0)
        for cells in self.conflict:
            constraint([(ids[c],v) for c,v in cells.items()],hi=1)
        mat=coo_matrix((vals,(rows,cols)),shape=(len(lower),len(cost))).tocsr()
        return np.asarray(cost,dtype=np.int64),mat,lower,upper

    def inequalities(self):
        objective,mat,lower,upper=self.matrix()
        at_hi=np.flatnonzero(np.isfinite(upper))
        at_lo=np.flatnonzero(np.isfinite(lower))
        arr=vstack([mat[at_hi],-mat[at_lo]],format="csr").astype(np.int64)
        rhs=np.r_[np.asarray(upper)[at_hi],-np.asarray(lower)[at_lo]].astype(np.int64)
        return objective,arr,rhs

    def exact_lower(self,objective,arr,rhs,weights,unit):
        assert unit>0 and len(weights)==len(rhs) and np.min(weights,initial=0)>=0
        bound=(int(abs(arr).sum())+int(abs(rhs).sum()))*int(weights.max(initial=0))
        assert (self.constant+int(sum(objective)))*unit+bound < 2**62
        reduced=objective*unit+arr.T@weights
        return self.constant*unit+int(np.minimum(0,reduced).sum())-int(rhs@weights)

    def solve(self,limit,certify=False):
        cost,mat,lower,upper=self.matrix()
        integer=np.zeros(len(cost),dtype=np.uint8)
        integer[:self.n]=1
        result=milp(np.array(cost),integrality=integer,bounds=Bounds(0,1),
                    constraints=LinearConstraint(mat,lower,upper),
                    options=dict(time_limit=limit,mip_rel_gap=0))
        report=dict(status=int(result.status),message=result.message,solver_lower=None,
                    nodes=getattr(result,'mip_node_count',None),variables=len(cost),constraints=len(lower))
        lb=getattr(result,'mip_dual_bound',None)
        if lb is not None:
            report['solver_lower']=self.constant + float(lb)
        bits=None
        if result.x is not None:
            bits=[int(v>.5) for v in result.x[:self.n]]
            measured=self.exact(bits)
            assert measured and measured[0]==self.evaluate(bits)
            assert abs(measured[0]-(self.constant+result.fun))<.01
            report.update(Q=measured[0],D=measured[1],shifted_classes=sum(bits))
        if certify and bits is not None:
            # Every inequality uses integer coefficients. A quantized LP multiplier
            # gives an independently checkable exact lower bound, even if dual
            # rounding loses equality. The activation count is an integer.
            objective,arr,rhs=self.inequalities()
            lp=linprog(objective,A_ub=arr,b_ub=rhs,bounds=(0,1),method="highs")
            if lp.success:
                unit=100000
                weights=np.maximum(0,np.rint(-lp.ineqlin.marginals*unit).astype(np.int64))
                numerator=self.exact_lower(objective,arr,rhs,weights,unit)
                report.update(certificate_scale=unit,certificate_lower_numerator=numerator,
                              certified_integer_lower=-(-numerator//unit),
                              multipliers=[[int(i),int(weights[i])] for i in np.flatnonzero(weights)])
            else:
                report['lp_status']=lp.message
        return bits,report


def small():
    checks=sections=solves=0
    for n in range(1,7):
        for raw in product(range(2),repeat=n):
            oracle=Oracle(bytes(raw))
            for a in range(n):
                for b in range(a+1,n+1):
                    if oracle.first[a]!=oracle.first[b-1]:
                        continue
                    types,source,returns=objects(oracle,(a,b))
                    groups,after=follower_partition(types,source,returns)
                    model=Choice(types,groups,after)
                    best=n+1
                    for bits in product(range(2),repeat=model.n):
                        physical=[oracle.forward[a+i] if bits[groups[x]-1] else a+i for i,x in enumerate(source)]
                        got=model.exact(bits)
                        assert (got is None and model.evaluate(bits) is None) or (got and got[0]==model.evaluate(bits))
                        if got:
                            assert len(set(physical))==len(physical)
                            physical_set=set(physical)
                            words=model.words(bits)
                            for i,x in enumerate(physical):
                                w=bytearray()
                                while True:
                                    w.append(oracle.first[x])
                                    x=oracle.forward[x]
                                    if x in physical_set:
                                        break
                                assert w==words[source[i]] and x==physical[returns[i]]
                            best=min(best,got[0])
                        checks+=1
                    if sections%31==0:
                        _,sol=model.solve(5)
                        assert sol['status']==0 and sol['Q']==best and abs(sol['solver_lower']-best)<1.e-6
                        solves+=1
                    sections+=1
    print(json.dumps(dict(sections=sections,assignments=checks,solved_vs_brute=solves)))


def run(tape,kind,out,limit):
    sys.path.insert(0,str(here.parents[7]/'docs/journal/2026-11-18-astra-c-output-first'))
    from inputs import read_input
    runs,mark=read_input(tape)
    col=Column(runs)
    oracle=Oracle(b''.join(bytes([c])*width for c,width in runs))
    section=col.interval(b'"')
    if kind=='frequent':
        section=heavy_set(col,section,1)[0][0]
    start=monotonic()
    types,source,returns=objects(oracle,section)
    groups,after=follower_partition(types,source,returns)
    model=Choice(types,groups,after)
    built=monotonic()
    bits,report=model.solve(limit,True)
    report.update(tape=tape.name,sha256=hashlib.sha256(tape.read_bytes()).hexdigest(),section=section,
                  classes=model.n,types=len(types),Q0=model.exact([0]*model.n),Q1=model.exact([1]*model.n),
                  constant=model.constant,signatures=len(model.signatures),build_s=built-start,solve_s=monotonic()-built)
    if bits:
        words=model.words(bits)
        mapped=[oracle.forward[x] if bits[groups[t]-1] else x for x,t in enumerate(source,section[0])]
        cells=set(mapped)
        assert len(cells)==len(mapped)
        for i,x in enumerate(mapped):
            w=bytearray()
            while True:
                w.append(oracle.first[x])
                x=oracle.forward[x]
                if x in cells:
                    break
            assert x==mapped[returns[i]] and w==words[source[i]]
        order=sorted(cells)
        report.update(shifted_types=sum(bits[g-1] for g in groups),components=1+sum(x+1!=y for x,y in zip(order,order[1:])),
                      class_bits=''.join(map(str,bits)),ordered=all(x<y for x,y in zip(mapped,mapped[1:])))
    out.write_text(json.dumps(report,indent=2)+'\n')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('tape',nargs='?',type=Path)
    p.add_argument('section',nargs='?',choices=('quote','frequent'))
    p.add_argument('output',nargs='?',type=Path)
    p.add_argument('--limit',type=float,default=60)
    args=p.parse_args()
    run(args.tape,args.section,args.output,args.limit) if args.tape else small()
