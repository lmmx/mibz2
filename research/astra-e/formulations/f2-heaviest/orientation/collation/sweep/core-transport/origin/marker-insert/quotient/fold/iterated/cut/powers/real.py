"""Original RLE BWT → powers/cut grammar → original bytes, RLE1 and block CRC."""
import argparse
import bz2
import csv
import hashlib
import json
import lzma
from pathlib import Path
import resource
import subprocess
import tempfile
import time
from solve import solve,signature
from anchor import Oracle,read_input


def rle1(data):
    out=bytearray()
    i=0
    while i<len(data):
        c=data[i]
        i+=1
        count=1
        while count<4 and i<len(data) and data[i]==c:
            i+=1
            count+=1
        if count==4:
            assert i<len(data)
            count+=data[i]
            i+=1
        out.extend(bytes([c])*count)
    return bytes(out)


def crc32_bzip2(data):
    table=[]
    for x in range(256):
        q=x<<24
        for _ in range(8):
            q=((q<<1) ^ (0x04c11db7 if q>>31 else 0)) & 0xffffffff
        table.append(q)
    value=0xffffffff
    for c in data:
        value=((value<<8)&0xffffffff)^table[(value>>24)^c]
    return value^0xffffffff


def recovered(d,i):
    raw=(d/f'rec{i+1:05d}sample.bz2').read_bytes()
    assert raw[:3]==b'BZh' and raw[4:10]==bytes.fromhex('314159265359')
    return bz2.decompress(raw),int.from_bytes(raw[10:14],'big')


def lib_block(corpus,i):
    with tempfile.TemporaryDirectory() as td:
        d=Path(td)
        (d/'sample.bz2').symlink_to(corpus.resolve())
        subprocess.run(['bzip2recover','sample.bz2'],cwd=d,stdout=subprocess.DEVNULL,
                       stderr=subprocess.DEVNULL,check=True)
        return recovered(d,i)


def source(tape):
    with lzma.open(tape,'rb') as f:
        meta=json.loads(f.readline())
    runs,mark=read_input(tape)
    order=sorted({k for k,n in runs})
    ids={k:i for i,k in enumerate(order)}
    root=[(ids[k],n) for k,n in runs]
    words=[bytes([k]) for k in order]
    return meta,runs,mark,root,words


def original(args):
    meta,runs,mark,root,words=source(args.tape)
    trace=[]
    # Source and libbzip2 checks remain outside the reported pass. The check
    # option additionally validates all dense intermediate partners.
    if args.backward:
        from backward import normalize
    else:
        from normalize import normalize
    start=time.process_time_ns()
    out,totals=solve(root,words,mark,0,meta['n'],trace.append,check=args.differential,normalizer=normalize,gated=args.gated)
    solver_ns=time.process_time_ns()-start
    o=Oracle(b''.join(bytes([k])*n for k,n in runs))
    p=mark
    expected=bytearray()
    for _ in range(meta['n']):
        expected.append(o.first[p])
        p=o.forward[p]
    assert out==expected
    plain=rle1(out)
    reference,header_crc=(recovered(args.recovered,meta['index']) if getattr(args,'recovered',None)
                          else lib_block(args.corpus,meta['index']))
    crc=crc32_bzip2(plain)
    assert plain==reference and crc==header_crc
    with args.trace.open('x',newline='') as f:
        wr=csv.DictWriter(f,trace[0],lineterminator='\n')
        wr.writeheader();wr.writerows(trace)
    with args.corpus.open('rb') as f:
        corpus_sha=hashlib.file_digest(f,'sha256').hexdigest()
    return dict(source_tape=args.tape.name,source_sha256=signature(args.tape.read_bytes()),
                corpus_sha256=corpus_sha,original_block_index=meta['index'],mark=mark,
                source_classes=len(words),source_runs=len(root),source_bytes=meta['n'],
                original_inverse_sha256=signature(out),decoded_bytes=len(plain),decoded_sha256=signature(plain),
                block_crc=f'{crc:08x}',libbzip2_match=True,checked_every_partner=args.differential,
                solver_process_ns=solver_ns,process_peak_kib_including_oracle=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
                costs=totals)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('tape',type=Path)
    p.add_argument('corpus',type=Path)
    p.add_argument('--trace',required=True,type=Path)
    p.add_argument('--differential',action='store_true')
    p.add_argument('--backward',action='store_true')
    p.add_argument('--gated',action='store_true')
    p.add_argument('--recovered',type=Path)
    print(json.dumps(original(p.parse_args()),indent=2))
