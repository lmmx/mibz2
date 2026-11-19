"""Compare first returns to the identical selected rows directly in original-byte and lifted rank orders."""
from bisect import bisect_right
from collections import Counter
from pathlib import Path
import csv
import sys

from check import Atlas,Column,Lift,fixture,geometry,heavy_set,paths,read_input


def compare(col, cut_rows, physical_words, result):
    # H is a physical union, compressed in its order in V. Keep both boundary types!
    lo=[];hi=[];compressed=[]
    for i,row in enumerate(cut_rows):
        if hi and row==hi[-1]:
            hi[-1]+=1
        else:
            lo.append(row);hi.append(row+1);compressed.append(i)
    segments=[]
    dictionary={}
    probes=0
    for comp,left in enumerate(lo):
        dest=left
        while dest < hi[comp]:
            x=dest
            width=hi[comp]-dest
            backward=bytearray()
            while True:
                j=bisect_right(col.starts,x)-1
                width=min(width,col.starts[j+1]-x)
                backward.append(col.symbols[j])
                x+=col.maps[j]
                i=bisect_right(lo,x)-1
                if i>=0 and x<hi[i]:
                    width=min(width,hi[i]-x)
                    break
                if i+1<len(lo):
                    width=min(width,lo[i+1]-x)
            w=bytes(reversed(backward))
            if w not in dictionary:
                dictionary[w]=len(dictionary)
            probes+=len(w)
            segments.append((compressed[i]+x-lo[i],compressed[comp]+dest-left,width,dictionary[w]))
            dest+=width
    assert probes<=col.rows
    segments.sort()
    stored=list(dictionary)
    # Segmentations may differ. Check equal endpoint and complete physical return bytes throughout.
    j=0
    for s,d,w,k in segments:
        pos=s
        while pos<s+w:
            a,b,size,ident=result.segments[j]
            a-=result.lo; b-=result.lo
            assert a<=pos<a+size and b+pos-a==d+pos-s
            assert physical_words[ident]==stored[k]
            pos=min(s+w,a+size)
            if pos==a+size:
                j+=1
    assert j==len(result.segments)
    return len(lo),probes,len(segments),len(stored),sum(map(len,stored))


def main(tapes):
    writer=None
    for tape in paths(tapes):
        ordinal=int(tape.name.split('.')[0])
        if ordinal not in (0,24):
            continue
        runs,_=read_input(tape)
        col=Column(runs)
        last=b''.join(bytes([b])*w for b,w in runs)
        psi,_=geometry(last)
        q=col.interval(b'"')
        for name,u in [('quote',q),('frequent',heavy_set(col,q,1)[0][0])]:
            for h in (4,12):
                atlas=Atlas(col,u,h)
                root,weights,minima,cuts=fixture(last,psi,u,h)
                model=Lift(u,atlas.leaves,root,weights,minima)
                result=model.induce(model.core)
                rows=[z for z,_,_,_,_ in cuts[result.lo:result.hi]]
                strings=[b''.join(model.weights[t] for t in w) for w in result.dictionary]
                components,steps,bands,k,d=compare(col,rows,strings,result)
                record=dict(ordinal=ordinal,section=name,h=h,selected=len(rows),components=components,
                            original_byte_steps=steps,original_bands=bands,original_byte_types=k,original_dictionary_bytes=d,
                            phase_steps=result.probes,phase_bands=len(result.segments),phase_symbol_types=len(strings),
                            phase_dictionary_bytes=sum(map(len,strings)))
                if writer is None:
                    writer=csv.DictWriter(sys.stdout,fieldnames=record,lineterminator='\n')
                    writer.writeheader()
                writer.writerow(record)
                sys.stdout.flush()


if __name__=='__main__':
    main(Path(sys.argv[1]))
