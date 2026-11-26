"""Use the independent original-run context exports; check all physical event returns."""
import argparse
from array import array
from bisect import bisect_right
from collections import Counter
from itertools import accumulate
import csv,hashlib,struct,sys,time
from pathlib import Path
from transport import acquire

def read_model(path):
    raw=path.read_bytes();assert raw[:8]==b"CTXMOD01"
    n,mark,k,K,slices,E,first=struct.unpack_from("<7I",raw,8)
    state=list(struct.iter_unpack("<IIB",raw[36:36+9*K]))
    ranges=list(struct.iter_unpack("<IIIB",raw[36+9*K:]))
    assert len(ranges)==slices and sum(b-a for a,b,_j,_c in ranges)==E
    return n,mark,K,[0]+[v[0] for v in state],[v[2] for v in state],ranges,hashlib.sha256(raw).hexdigest()

def original(path):
    vals=path.read_text().splitlines();mark=int(vals[0]);rle=[tuple(map(int,l.split()))for l in vals[1:]]
    n=sum(w for c,w in rle);freq=[0]*256
    for c,w in rle:freq[c]+=w
    cursor=[0,*accumulate(freq)];L=bytearray();lf=array('I')
    for c,w in rle:
        L.extend(bytes([c])*w);lf.extend(range(cursor[c],cursor[c]+w));cursor[c]+=w
    r=mark;order=array('I')
    while not order or r!=mark:order.append(r);r=lf[r]
    assert len(order)==n
    expect=bytes(L[x] for x in reversed(order))
    assert expect==(path.parent/'cut.bin').read_bytes()
    return mark,rle,L,lf,order,hashlib.sha256(expect).hexdigest()

def run(args):
    write=None
    for source in csv.DictReader(args.sources.open()):
        i=int(source['index'])
        if args.ids and i not in args.ids:continue
        op=args.original/source['tape'].split('.')[0]/'input.txt'
        mark,rle,L,lf,order,raw_hash=original(op)
        for k in args.depths:
            n,p,K,start,modal,ranges,sha=read_model(args.models/str(i)/f"{k}.ctx");assert mark==p and n==len(L)
            before=time.thread_time_ns();got,stats=acquire(rle,start,modal,mark,compact=args.compact);thread=time.thread_time_ns()-before
            expected={mark};clocks=[]
            for lo,hi,j,c in ranges:expected.update(range(lo,hi))
            assert set(got)==expected
            for t,r in enumerate(order):
                if r in expected:clocks.append((t,r))
            for j,(t,r) in enumerate(clocks):
                u,y=clocks[(j+1)%len(clocks)];assert got[r]==(y,(u-t)%n or n)
            payload=(args.wires/f'{i}.pred{k}').read_bytes();sz,initial,z,ne=struct.unpack_from('<4I',payload,8)
            assert (sz,z,initial)==(n,K,bisect_right(start,mark)-1)
            original_events=list(struct.iter_unpack('<IBI',payload[24+5*z:]))
            events=[]
            for t,r in clocks:
                s=bisect_right(start,r)-1
                if L[r]!=modal[s]:events.append((t,L[r],bisect_right(start,lf[r])-1))
            assert len(events)==ne and events==original_events
            row=dict(block=i,k=k,n=n,mark=mark,raw_sha256=raw_hash,context_sha256=sha,computed_partner_checks=len(got),verified_clock_events=ne,
                     thread_ns_python=thread,**stats)
            if write is None:write=csv.DictWriter(sys.stdout,row,lineterminator='\n');write.writeheader()
            write.writerow(row);sys.stdout.flush()
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument("original",type=Path);p.add_argument("models",type=Path);p.add_argument("sources",type=Path);p.add_argument("wires",type=Path)
    p.add_argument("--compact",action="store_true");p.add_argument("--ids",nargs="+",type=int);p.add_argument("--depths",nargs="+",type=int,default=[8]);run(p.parse_args())
