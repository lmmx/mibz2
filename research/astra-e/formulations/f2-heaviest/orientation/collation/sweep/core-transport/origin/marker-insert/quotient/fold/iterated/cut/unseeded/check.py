import argparse
from collections import Counter
from itertools import groupby,product
import json
from decode import solve
from cut import forward,geometry


def tiny():
    summary=Counter()
    for n in range(1,7):
        for last in product(range(3),repeat=n):
            keys=sorted(set(last))
            if keys!=list(range(len(keys))):continue
            raw=[(k,len(list(g))) for k,g in groupby(last)]
            words=[bytes([65+i])*(i+1) for i in keys]
            freq,b=geometry(raw,len(words))
            p=forward(raw,b[:-1])
            for start in range(n):
                text=bytearray()
                i=start
                while True:
                    i,c=p[i]
                    text.extend(words[c])
                    if i==start:break
                horizon=sum(a*len(v) for a,v in zip(freq,words))
                for skip in range(len(words[p[start][1]])):
                    want=(text*((skip+horizon+len(text)-1)//len(text)))[skip:skip+horizon]
                    for policy in ('ordinary','flip','cheaper'):
                        out,c=solve(raw,words,start,skip,horizon,policy,True)
                        assert out==want and c['lap']==len(text)
                        summary.update(cases=1,nondividing=int(horizon%len(text)!=0),
                                       normalizations=c.get('normalizations',0),jumps=c.get('power_mark_hops',0),
                                       terminal_shared=int(c['terminal_multiplicity']>1))
    return summary


def huge():
    examples=[]
    for n in (17,2**32+7):
        raw=[(1,1),(0,n)]
        # Source classes a=[0,n) and b={n}. Forward maps i→i+1 mod n+1.
        for mark in (0,1,n//2,n-1,n):
            start=2*mark
            lap=2*n+1
            expected=bytes(ord('b') if (start+t)%lap==2*n else ord('A') for t in range(51))
            out,c=solve(raw,[b'AA',b'b'],mark,0,51,check=n==17)
            assert c['lap']==lap and out==expected and c.get('power_mark_queries',0)<=2
            examples.append(dict(n=n,mark=mark,queries=c.get('power_mark_queries',0),hops=c.get('power_mark_hops',0)))
        for x in (1,n//2):
            out,c=solve([(0,n),(2,1),(1,1)],[b'AA',b'b',b'c'],x,1,53,check=n==17)
            assert out==b'A'*53 and c['terminal_multiplicity']==n and not c.get('rounds')
    return examples


if __name__=='__main__':
    print(json.dumps(dict(tiny=tiny(),sparse=huge()),indent=2))
