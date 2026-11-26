"""All-rank transport under an ordinary delimiter cut, in original source order."""
from array import array
from bisect import bisect_right
import csv
import hashlib
from itertools import groupby,product
import json
from pathlib import Path
import struct
import sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent))
from decode import solve
from orient import real
from cut import forward,geometry,overlay


def atlas(raw,words,b,slices,target):
    lost=[0]*len(words)
    for k,j,width in slices:
        if k not in target and j in target:lost[j]+=width
    deleted_before=[]
    total=0
    for v in lost:
        deleted_before.append(total)
        total+=v
    counts=[0]*len(words)
    descriptors=[]
    old=removed=0
    for k,j,w in slices:
        if k not in target and j in target:
            # All source rows in k survive. deleted_before[k] is constant there.
            shift=b[k]+counts[k]-deleted_before[k]-old
            add=len(words[k])
            removed+=w
        else:
            shift=-removed
            add=0
        if not descriptors or descriptors[-1][1:]!=(shift,add):
            descriptors.append((old,shift,add))
        old+=w
        counts[k]+=w
    assert removed==total and len(descriptors)<=len(slices)<=len(raw)+len(words)-1
    return descriptors


def small():
    cases=points=0
    for n in range(1,7):
        for last in product(range(3),repeat=n):
            keys=sorted(set(last))
            if keys!=list(range(len(keys))):continue
            raw=[(k,len(list(g))) for k,g in groupby(last)]
            words=[bytes([i])*(i+1) for i in keys]
            _,b=geometry(raw,len(keys))
            slices,_=overlay(raw,b)
            p=forward(raw,b[:-1])
            for mask in range(1<<len(keys)):
                t={k for k in keys if mask>>k&1}
                records=atlas(raw,words,b,slices,t)
                starts=[r[0] for r in records]
                dead={d for d,k in p if k not in t and p[d][1] in t}
                new={x:i for i,x in enumerate(x for x in range(n) if x not in dead)}
                pred={d:x for x,(d,k) in enumerate(p)}
                for x in range(n):
                    _,shift,add=records[bisect_right(starts,x)-1]
                    y=pred[x] if x in dead else x
                    assert x+shift==new[y] and add==(len(words[p[y][1]]) if y!=x else 0)
                    points+=1
                cases+=1
    return dict(cuts=cases,ordinals=points)


def frozen(tape,archive,table):
    m,_,mark,raw,words=real.source(tape)
    rows=m['n']
    with archive.open('xb') as f,table.open('x',newline='') as fh:
        writer=csv.writer(fh,lineterminator='\n')
        writer.writerow(('depth','rows','old_runs','types','overlay_fragments','atlas_pieces','delta_bytes','weight_bytes'))
        f.write(b'ATLS0001')
        f.write(struct.pack('<IIII',rows,mark,len(raw),len(words)))
        for k,n in raw:
            f.write(struct.pack('<II',k,n))
        depth=0
        def collect(r,w,b,s,t):
            nonlocal depth
            rec=atlas(r,w,b,s,t)
            f.write(struct.pack('<I',len(rec)))
            for start,shift,add in rec:
                f.write(struct.pack('<IiI',start,shift,add))
            writer.writerow((depth,b[-1],len(r),len(w),len(s),len(rec),4*len(rec),4*len(rec)))
            depth+=1
        out,c=solve(raw,words,mark,0,rows,observe_cut=collect)
        assert not c.get('normalizations') and c['lap']==rows
        f.write(struct.pack('<I',0))
    ref=next(r for r in map(json.loads,(HERE.parent/'real.jsonl').read_text().splitlines()) if r['index']==m['index'])
    assert hashlib.sha256(out).hexdigest()==ref['pre_rle_sha'] and c['final_skip']==ref['final_skip']
    return dict(mark=mark,rows=rows,depth=depth,original_runs=len(raw),pre_rle_sha=ref['pre_rle_sha'],
                final_skip=ref['final_skip'],wire_bytes=archive.stat().st_size,
                wire_sha256=hashlib.sha256(archive.read_bytes()).hexdigest())


if __name__=='__main__':
    if len(sys.argv)==1:print(json.dumps(small(),indent=2))
    else:print(json.dumps(frozen(*map(Path,sys.argv[1:])),indent=2))
