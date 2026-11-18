"""Verify marked bytes against libbz2 on single-block envelopes of the hashed input."""
import argparse
import bz2
import csv
from hashlib import sha256
from pathlib import Path
import sys

from packets import contract as packet_contract
from queue_ranges import decode
from residual import induce, quotient, original
from screen import inputs


def envelope(data, bit, end, crc):
    span = data[bit//8:(end+7)//8]
    width = end-bit
    payload = (int.from_bytes(span,'big')>>(-end%8)) & ((1<<width)-1)
    payload = ((payload<<48 | 0x177245385090)<<32) | crc
    padding = -(width+80)%8
    return b'BZh9'+(payload<<padding).to_bytes((width+80+padding)//8,'big')


def unrle(raw):
    out = bytearray()
    previous,seen = None,0
    for c in raw:
        if seen==4:
            out.extend(bytes([previous])*c)
            previous,seen = None,0
        else:
            out.append(c)
            seen = seen+1 if c==previous else 1
            previous = c
    assert seen!=4
    return bytes(out)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('corpus',type=Path)
    parser.add_argument('folder',type=Path)
    args=parser.parse_args()
    data=args.corpus.read_bytes()
    assert sha256(data).hexdigest()=='82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915'
    with (args.folder/'inputs.csv').open() as f:
        manifest={int(r['block']):r for r in csv.DictReader(f)}
    writer=None
    for b,n,p,m,bands in inputs(args.folder,None):
        r=manifest[b]
        block=envelope(data,int(r['bit']),int(r['end']),int(r['crc']))
        target=bz2.decompress(block)
        want,baseline=induce(original(bands),n,p,n)
        assert unrle(want)==target
        for size in (2,4,8,16,32,64):
            s,mark,arcs=quotient(bands,n,p,size)
            got,_=induce(arcs,s,mark,n)
            assert got==want
            if size==2:
                gs,gm,ga,_=packet_contract(bands,n,p,size)
                assert (s,mark,arcs)==(gs,gm,ga)
        got,_=decode(bands,n,p)
        assert got==want
        sb,decoded=sha256(block).hexdigest(),sha256(target).hexdigest()
        if b==0:
            assert sb=='e3bf5372b15885da138764a25bb2b7cd784ee3cbee1e664937647921ae3122b0'
            assert decoded=='d2584c09f1d643d6abff8a25513b63af4e05969c15c30256aed3a05c74281210'
        row=dict(**r,period=baseline['period'],decoded_bytes=len(target),
                 records_sha256=sha256((args.folder/f'{b:04}.recs').read_bytes()).hexdigest(),
                 block_sha256=sb,pre_rle1_sha256=sha256(want).hexdigest(),decoded_sha256=decoded)
        if writer is None:
            writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
            writer.writeheader()
        writer.writerow(row)


if __name__=='__main__':
    main()
