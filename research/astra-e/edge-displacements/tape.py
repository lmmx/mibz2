"""Lossless compact receipt for all 233 marked edge tables. Use constructer's band order."""
import argparse
import hashlib
import lzma
from pathlib import Path
import struct
import numpy as np

def transfer(mode, source, target):
    digest=hashlib.sha256()
    blocks=0
    zipped=lzma.LZMAFile
    opener_in=(lambda: zipped(source,'rb')) if mode=='restore' else (lambda: source.open('rb'))
    opener_out=(lambda: target.open('wb')) if mode=='restore' else (lambda: zipped(target,'wb',preset=2))
    with opener_in() as f,opener_out() as g:
        while hdr:=f.read(32):
            b,stream,n,m,mark,bits,words,bands=struct.unpack('<8I',hdr)
            g.write(hdr)
            if mode=='pack':
                raw=f.read(4*m)
                spans=f.read(16*bands)
                edges=np.frombuffer(raw,dtype='<u4')
                z=np.frombuffer(spans,dtype='<u4').reshape(-1,4)
                code_byte=np.zeros(256+words,dtype=np.uint8)
                code_byte[z[:,2]]=z[:,3]
                nxt=edges&((1<<bits)-1)
                by_rank=np.empty(m,dtype=np.uint8)
                by_rank[nxt]=code_byte[edges>>bits]
                g.write(by_rank.tobytes())
                g.write(spans)
            else:
                by_rank=np.frombuffer(f.read(m),dtype=np.uint8)
                spans=f.read(16*bands)
                z=np.frombuffer(spans,dtype='<u4').reshape(-1,4)
                queue=np.argsort(by_rank,kind='stable').astype('<u4')
                k=0
                edge=np.empty(m,dtype='<u4')
                for start,size,code,a in z.tolist():
                    edge[start:start+size]=queue[k:k+size] | (code<<bits)
                    k+=size
                assert k==m
                raw=edge.tobytes()
                g.write(raw)
                g.write(spans)
            digest.update(hdr);digest.update(raw);digest.update(spans);blocks+=1
    print(blocks,digest.hexdigest())
if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('mode',choices=['pack','restore'])
    parser.add_argument('source',type=Path)
    parser.add_argument('target',type=Path)
    a=parser.parse_args()
    transfer(a.mode,a.source,a.target)
