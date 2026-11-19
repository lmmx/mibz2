"""Convert only original column runs and mark from the ten archived irec projections."""
import sys
import struct
from pathlib import Path
D=Path(__file__).resolve().parent
ROOT=D.parents[7]
sys.path.insert(0,str(ROOT/'docs/journal/2026-11-18-astra-c-output-first'))
from inputs import paths,read_input

with Path(sys.argv[2]).open('wb') as out:
    for t in paths(sys.argv[1]):
        runs,p=read_input(t);caps=[];n=0
        for b,w in runs:
            n+=w
            while w:
                v=min(127,w);caps.append((b<<8)|v);w-=v
        out.write(struct.pack('<4I',int(t.name.split('.')[0]),n,p,len(caps)))
        out.write(struct.pack(f'<{len(caps)}H',*caps))
