"""Archive only the original marked column runs, independently of every return map."""
from hashlib import sha256
import lzma
from pathlib import Path
import struct
import sys

MAGIC=b'JHSRCv1\0'
HEAD=struct.Struct('<4I')
RUN=struct.Struct('<BI')

def pack(tapes,out,paths,read_input):
    with lzma.open(out,'wb',preset=6) as f:
        f.write(MAGIC)
        for tape in paths(tapes):
            runs,mark=read_input(tape)
            f.write(HEAD.pack(int(tape.name.split('.')[0]),sum(w for _,w in runs),mark,len(runs)))
            f.write(sha256(tape.read_bytes()).digest())
            for c,w in runs:f.write(RUN.pack(c,w))

def read(p):
    with lzma.open(p,'rb') as f:
        assert f.read(8)==MAGIC
        while header:=f.read(16):
            i,n,m,count=HEAD.unpack(header)
            digest=f.read(32).hex()
            runs=list(RUN.iter_unpack(f.read(RUN.size*count)))
            assert len(runs)==count and sum(q for _,q in runs)==n and 0<=m<n
            yield i,runs,m,digest

if __name__=='__main__':
    from generate import paths,read_input
    pack(Path(sys.argv[1]),Path(sys.argv[2]),paths,read_input)
