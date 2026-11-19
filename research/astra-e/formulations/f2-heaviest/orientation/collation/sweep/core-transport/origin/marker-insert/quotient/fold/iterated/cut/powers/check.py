"""Every marked input on tiny alphabets, and very large sparse synthetic roots."""
import argparse
from collections import Counter
from itertools import groupby,product
import json
from grammar import Node,output
from solve import solve
from cut import forward,geometry


def small(nmax):
    totals=Counter()
    for n in range(1,nmax+1):
        for k in range(1,min(n,3)+1):
            for column in product(range(k),repeat=n):
                if len(set(column))!=k:
                    continue
                raw=[(v,len(list(g))) for v,g in groupby(column)]
                letters=[bytes([j+65])*(j+1) for j in range(k)]
                freq,bounds=geometry(raw,k)
                edges=forward(raw,bounds[:-1])
                horizon=sum(n*len(w) for n,w in zip(freq,letters))
                for mark in range(n):
                    ref=bytearray()
                    y=mark
                    while True:
                        y,j=edges[y]
                        ref.extend(letters[j])
                        if y==mark:
                            break
                    for skip in range(len(letters[edges[mark][1]])):
                        got,c=solve(raw,letters,mark,skip,horizon)
                        copies=(skip+horizon+len(ref)-1)//len(ref)
                        assert got==(ref*copies)[skip:skip+horizon] and c['lap']==len(ref)
                        totals.update(marks=1,foreign_cycles=int(len(ref)<horizon),nonintegral=int(horizon%len(ref)!=0),
                                      rounds=c['rounds'],normalizations=c['normalized_rounds'],
                                      published_normalized_rows=c['normal_written_rows'],
                                      omitted_fixed=c['omitted_fixed'],powers=c['powers'])
    return totals


def sparse():
    records=[]
    for n in (2,3,17,128,2**32+7):
        for case in ('inside','isolated','fixed-mark'):
            if case=='inside':
                root=[(1,1),(0,n)]; words=[b'a',b'bc']; mark=n
                # A prefix crossing one marked-byte offset but no huge output.
                want=(b'c'+b'a'*min(n,4096)+b'b') if n>=4096 else (b'b'+b'c'+b'a'*n)
                horizon=min(4096,2+n)
                if n>=4096:want=want[:horizon]
                else:want=(want*2)[1:1+horizon]
                lap=n+2
            else:
                root=[(0,n),(2,1),(1,1)];words=[b'a',b'bc',b'def'];mark=n if case=='isolated' else n//2
                horizon=47
                if case=='isolated':want=(b'bcdef'*10)[1:1+horizon];lap=5
                else:want=b'a'*horizon;lap=1
            got,c=solve(root,words,mark,0 if case=='fixed-mark' else 1,horizon,check=n<200)
            assert got==want and c['lap']==lap
            if case=='inside':
                assert c['rounds']==1 and c['lookups']<=3 and c['normal_written_rows']==2 and c['powers']==1
            elif case=='isolated':
                assert c['rounds']==1 and c['omitted_fixed']==n
            else:
                assert c['rounds']==0
            records.append(dict(case=case,n=n,lap=lap,output_bytes=horizon,
                                rounds=c['rounds'],point_lookups=c['lookups'],
                                scattered_rows=c['normal_written_rows'],power_nodes=c['powers'],
                                repeated_bytes=c['power_copy_bytes']))
    return records


def writer():
    a,b=Node(literal=b'ab'),Node(literal=b'Q')
    nodes=[(a+b,b'abQ'),((a+b).power(5)+a,b'abQ'*5+b'ab'),
           ((a.power(7)+b).power(4),(b'ab'*7+b'Q')*4)]
    cases=0
    for root,ref in nodes:
        for skip in range(len(ref)):
            for horizon in (0,1,2,len(ref)-1,len(ref),len(ref)+3,2*len(ref)+5):
                got,c=output(root,skip,horizon)
                assert got==(ref*((skip+horizon+len(ref)-1)//len(ref)))[skip:skip+horizon]
                cases+=1
    return cases


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--max-rows',type=int,default=6)
    args=p.parse_args()
    print(json.dumps(dict(max_rows=args.max_rows,small=small(args.max_rows),sparse=sparse(),writer_marks=writer()),indent=2))
