"""Little endian tagged fixture for a first historical-cut index; excludes raw expected bytes."""
from bisect import bisect_right
from pathlib import Path
import struct,sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent))
from constructor import initial
from check import old
from hints import Hints
from implicit import Implicit

def pack(vals):return struct.pack("<"+"I"*len(vals),*vals)
def export(p,out):
    lines=p.read_text().splitlines();mark=int(lines[0])
    src=[tuple(map(int,x.split()))for x in lines[1:]]
    b,ix=initial(src);col={x[3]:old["colour"](x[3],0) for x in b}
    c=Implicit(b,ix,mark,col,{});h=Hints(c)
    basei=[0]*len(b)
    for e,x in zip(c.image,c.istart):basei[e]=x
    header=[0x41335049,sum(n for _,n in src),c.n,c.mark,bisect_right(c.source,c.mark)-1,len(b),len(c.image),len(c.lo)]
    with out.open("wb") as f:
        f.write(pack(header))
        for e,(x,y,n,word) in enumerate(b):
            f.write(pack([c.source[e],basei[e],x,y,c.f[e]&0xffffffff,c.end[e],h.lo[e],h.hi[e],word|c.color[e]<<8]))
        for a in (c.image,c.istart,c.lo,c.hi,c.closed,c.zero,c.mass):f.write(pack(a))
if __name__=="__main__":
    input_dir,out=map(Path,sys.argv[1:]);out.mkdir(parents=True,exist_ok=True)
    ids=[1,41,85,134,176,225,269,313,359,397]
    entries=sorted(input_dir.glob("0*/input.txt"));assert len(entries)==len(ids)
    for p,i in zip(entries,ids):export(p,out/f"{i}.idx")
