"""Audit raw offsets, marked bytes, RLE1 and stored CRC using independent oracles."""
import argparse
from array import array
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
from encode import real,serialize_parts


def verify_positions(path, runs, mark):
    last=b''.join(bytes([symbol])*count for symbol,count in runs)
    # The true inverse permutation is stable sorting of the original last column.
    forward=sorted(range(len(last)),key=last.__getitem__)
    values=array('I')
    with path.open('rb') as stream:
        values.fromfile(stream,len(last))
        assert not stream.read(1)
    if sys.byteorder!='little':
        values.byteswap()
    row=mark
    for offset in range(len(last)):
        assert values[row]==offset, (row,offset,values[row])
        row=forward[row]
    assert row==mark


def main():
    p=argparse.ArgumentParser()
    p.add_argument('tapes',type=Path)
    p.add_argument('corpus',type=Path)
    p.add_argument('executable',type=Path)
    a=p.parse_args()
    with TemporaryDirectory() as td:
        base=Path(td)
        (base/'sample.bz2').symlink_to(a.corpus.resolve())
        subprocess.run(['bzip2recover','sample.bz2'],cwd=base,check=True,capture_output=True)
        for tape in sorted(a.tapes.glob('*.irec.xz')):
            m,runs,mark,raw,words=real.source(tape)
            wire=base/(tape.stem+'.pos')
            plain=base/'out'
            pos=base/'coords'
            hist=serialize_parts(raw,words,mark,wire)
            assert hist is not None
            result=subprocess.check_output([a.executable,wire,plain,pos],text=True)
            counters={k:int(v) for k,v in (x.split('=') for x in result.split())}
            verify_positions(pos,runs,mark)
            coded=plain.read_bytes()
            data=real.rle1(coded)
            ref,crc=real.recovered(base,m['index'])
            assert data==ref and real.crc32_bzip2(data)==crc
            print(json.dumps(dict(index=m['index'],source=tape.name,source_sha=hashlib.sha256(tape.read_bytes()).hexdigest(),
                                  block_crc=hex(crc),pre_rle_sha=hashlib.sha256(coded).hexdigest(),
                                  original_positions_checked=m['n'],post_rle_bytes=len(data),
                                  wire_sha=hashlib.sha256(wire.read_bytes()).hexdigest(),wire_bytes=wire.stat().st_size,
                                  avoided_joins=sum(x['joins_avoided'] for x in hist),
                                  refined_numeric_types=sum(x['numeric_types'] for x in hist),**counters)),flush=True)


if __name__=='__main__':
    main()
