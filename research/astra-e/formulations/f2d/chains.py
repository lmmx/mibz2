"""Count and validate whole-run LF jump certificates for F2b's actual FM intervals."""
import argparse
import bisect
import collections
import csv
import itertools
from pathlib import Path
import struct
import sys

class Column:
    def __init__(self,caps):
        self.starts=[]
        self.ids=[]
        self.lf=[]
        counts=[0]*256
        n=0
        for rec in caps:
            c,width=divmod(rec,256)
            assert 1<=width<=127
            if not self.ids or self.ids[-1]!=c:
                self.ids.append(c)
                self.starts.append(n)
                self.lf.append(counts[c])
            counts[c]+=width
            n+=width
        self.starts.append(n)
        self.m=counts[34]
        total=0
        for i,v in enumerate(counts):
            counts[i],total=total,total+v
        self.quote=counts[34]
        self.lf=[base+counts[c] for c,base in zip(self.ids,self.lf)]

    def find(self,x):
        return bisect.bisect_right(self.starts,x)-1

    def actual(self):
        if not self.m:return [],0
        nodes=[[self.quote,self.quote+self.m]]
        paths=[]
        intersections=0
        for a,b in nodes:
            path=[]
            while True:
                i=self.find(a)
                if self.starts[i+1]<b or self.ids[i]==34:
                    break
                path.append((i,a,b))
                width=b-a
                a=self.lf[i]+a-self.starts[i]
                b=a+width
            # Always include the final paid lookup, even when there was no unary step.
            paths.append((path,(i,a,b)))
            children={}
            while self.starts[i]<b:
                start=max(a,self.starts[i])
                width=min(b,self.starts[i+1])-start
                c=self.ids[i]
                intersections+=1
                if c!=34:
                    mapped=self.lf[i]+start-self.starts[i]
                    if c not in children:
                        children[c]=len(nodes);nodes.append([mapped,mapped])
                    child=nodes[children[c]]
                    assert child[1]==mapped
                    child[1]+=width
                i+=1
        return paths,intersections

def simple(c,paths):
    safe={}
    visits=safe_visits=reuse_savings=partial_hits=0
    last={}
    for path, stop in paths:
        trace=path+[stop]
        for k,(i,a,b) in enumerate(path):
            j=trace[k+1][0]
            image=c.lf[i]+a-c.starts[i]
            assert c.starts[j]<=image<c.starts[j+1]
            cert = c.starts[j]<=c.lf[i] and c.lf[i]+c.starts[i+1]-c.starts[i]<=c.starts[j+1]
            visits+=1
            if cert:
                safe_visits+=1
                if i in safe:reuse_savings+=1
            assert i not in safe or safe[i]==cert
            safe[i]=cert
            if last.get(i)==j:partial_hits+=1
            last[i]=j
    return (visits,len(safe),sum(safe.values()),safe_visits,reuse_savings,partial_hits)

def chunks(c,paths,cap):
    # State of an executable constructor (no future suffix rows are consulted
    # by make). The separate actual trace verifies every displacement and byte.
    full={}
    jump={}
    setup_queries=queries=calls=hits=build_steps=skipped=partial=0
    def make(first):
        nonlocal setup_queries,build_steps
        at=first
        chars=[]
        delta=0
        while len(chars)<cap and at>=0 and c.ids[at]!=34:
            build_steps+=1
            if at not in full:
                j=c.find(c.lf[at])
                setup_queries+=1
                end=c.lf[at]+c.starts[at+1]-c.starts[at]
                full[at]=j if end<=c.starts[j+1] else -1
            chars.append(c.ids[at])
            delta+=c.lf[at]-c.starts[at]
            at=full[at]
        return delta,at,bytes(chars)

    for path,stop in paths:
        seq=path+[stop]
        index=0
        # Find each compact node's initial interval.
        queries+=1
        while index<len(path):
            i,a,b=seq[index]
            calls+=1
            if i not in jump:
                jump[i]=make(i)
            else:
                hits+=1
            delta,after,bytes_=jump[i]
            h=len(bytes_)
            assert h>0
            j,x,y=seq[index+h]
            assert (x,y)==(a+delta,b+delta)
            assert bytes_==bytes(c.ids[q[0]] for q in seq[index:index+h])
            if after==-1:
                queries+=1
                assert c.find(a+delta)==j
                partial+=1
            else:
                assert after==j and b+delta<=c.starts[j+1]
            index+=h
            skipped+=h-1
    return (len(jump),len(full),calls,hits,build_steps,setup_queries,queries,skipped,partial)

def check_bound():
    cases=swaps=0
    for m in range(2,10):
        for bits in itertools.product((0,1),repeat=m):
            # disjoint alphabets on alternating j, with q below all four
            x=[65+2*(j%2)+b for j,b in enumerate(bits)]
            L=x+[34]*m
            order=sorted(range(2*m),key=L.__getitem__)
            lf=[0]*(2*m)
            for i,j in enumerate(order):lf[j]=i
            for j in range(m):
                assert lf[m+j]==j and lf[j]>=m
                assert [L[lf[m+j]],L[m+j]]==[x[j],34]
            cases+=1
            for i,j in itertools.combinations(range(0,m,2),2):
                if x[i]==x[j]:continue
                y=x.copy();y[i],y[j]=y[j],y[i]
                assert collections.Counter(y)==collections.Counter(x) and y!=x
                swaps+=1
    print(f"hard-family columns={cases}, count-preserving token-column swaps={swaps}")

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("dir",nargs="?",type=Path)
    p.add_argument("--small",action="store_true");a=p.parse_args()
    if a.small:return check_bound()
    out=csv.writer(sys.stdout)
    out.writerow(("stream","rows","runs","nodes","unary_visits","original_lookups","branch_intersections",
        "visited_runs","safe_runs","safe_visits","lazy_whole_hits","last_image_hits","width",
        "descriptors","classified_runs","descriptor_reads","descriptor_hits","setup_steps","setup_lookups",
        "remaining_lookups","loops_avoided","ambiguous_jump_exits"))
    for path in sorted(a.dir.glob("*.recs"),key=lambda p:int(p.stem)):
        c=Column([v[0] for v in struct.iter_unpack("<H",path.read_bytes())])
        paths,intersections=c.actual()
        v,unique,safe,uses,hits,last=simple(c,paths)
        assert sum(len(x) for x,y in paths)==v
        for k in (2,4,8,16):
            out.writerow((path.stem,c.starts[-1],len(c.ids),len(paths),v,v+len(paths),intersections,
                unique,safe,uses,hits,last,k,*chunks(c,paths,k)))

if __name__=="__main__":
    main()
