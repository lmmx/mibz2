"""Exact complete-page escape cost for a dictionary of (word label, displacement bin)."""
import csv
from pathlib import Path
import struct
import sys
import numpy as np

def blocks(path):
    with path.open('rb') as f:
        while hdr:=f.read(32):
            b,stream,n,m,mark,bits,words,bands=struct.unpack('<8I',hdr)
            raw=np.fromfile(f,dtype='<u4',count=m)
            spans=np.fromfile(f,dtype='<u4',count=4*bands).reshape(-1,4)
            assert len(raw)==m and len(spans)==bands
            yield b,stream,n,m,bits,words,raw,spans

def best_window(x,group,width):
    # The intervals of consecutive group keys have a gap larger than any tested width.
    packed=np.sort((group.astype(np.int64)<<23)+x+(1<<20))
    starts=np.r_[0,np.flatnonzero((packed[1:]>>23)!=(packed[:-1]>>23))+1]
    reaches=np.searchsorted(packed,packed+width,side='left')-np.arange(len(packed))
    return int(np.maximum.reduceat(reaches,starts).sum())

def estimate(path,out):
    out.mkdir(parents=True,exist_ok=True)
    with (out/'magnitude.csv').open('w') as rawfile,(out/'palette.csv').open('w') as palette:
        entropy_file=(out/'entropy.csv').open('w')
        ent=csv.writer(entropy_file,lineterminator='\n')
        ent.writerow(['block','m','H_label','H_delta','H_delta_given_terminal','H_delta_given_label','H_joint'])
        lower_file=(out/'palette-packed.csv').open('w')
        lower=csv.writer(lower_file,lineterminator='\n')
        lower.writerow(['block','m','bin_bits','variant','descriptor_width','groups','escaped','total'])
        def entropy(x):
            _,counts=np.unique(x,return_counts=True)
            return float(-np.dot(counts,np.log2(counts/len(x)))/len(x))
        basic=csv.writer(rawfile,lineterminator='\n')
        basic.writerow(['block','stream','m','bits','width','plain_centered','equal_adjacent_delta','increasing_adjacent_next','optimal_one_base','optimal_per_terminal','optimal_per_label'])
        w=csv.writer(palette,lineterminator='\n')
        w.writerow(['block','stream','m','bits','bin_bits','slots','groups_present','groups_used','escape','inline','descriptors','escape_pages','escape_storage','total','staged_full_table_bytes','normal_dependency','escape_dependency'])
        for b,stream,n,m,bits,words,edges,spans in blocks(path):
            src=np.arange(m,dtype=np.int64)
            nxt=(edges&((1<<bits)-1)).astype(np.int64)
            delta=nxt-src
            code=(edges>>bits).astype(np.int64)
            byte_of=np.zeros(words+256,dtype=np.int64)
            byte_of[spans[:,2]]=spans[:,3]
            a=byte_of[code]
            hd=entropy(delta); hc=entropy(code); joint=entropy((code<<23)+delta)
            ent.writerow([b,m,hc,hd,entropy((a<<23)+delta)-entropy(a),joint-hc,joint])
            replicated=int((delta[1:]==delta[:-1]).sum())
            increasing=int((nxt[1:]>nxt[:-1]).sum())
            for bts in [4,8,12,16]:
                width=1<<bts
                common=int(((delta>=-width//2)&(delta<width//2)).sum())
                basic.writerow([b,stream,m,bits,width,common,replicated,increasing,best_window(delta,code*0,width),
                                best_window(delta,a,width),best_window(delta,code,width)])
            for k in range(0,13):
                # 16-bit slot's high bit selects a local u32 escape; the other 15 bits name
                # a descriptor and the residual. Bins include the original word identity.
                capacity=1<<(15-k)
                keys=(code<<23)+((delta+(1<<20))>>k)
                _,counts=np.unique(keys,return_counts=True)
                frequencies=np.sort(counts)[::-1]
                cbits=int(code.max()).bit_length()
                # Shift by 2^bits to encode both signed extremes, then remove k low bits.
                # Two's-complement wrap into our original biased bins is a constant add.
                dwidth=(cbits+max(1,bits+1-k)+7)//8
                for variant in ['index15','bitmap16']:
                    cap=capacity if variant=='index15' else (1<<(16-k))-1
                    picked=frequencies[:cap]
                    picked=picked[4*picked>dwidth]
                    e=m-int(picked.sum())
                    directory=4*((m+32767)//32768) if variant=='index15' else 12*((m+63)//64)
                    lower.writerow([b,m,k,variant,dwidth,len(picked),e,2*m+dwidth*len(picked)+4*e+directory])
                ranked=frequencies[:capacity]
                # 8B descriptor vs 4B eliminated escapes: adding a bin used twice ties in size.
                used=ranked[ranked>=3]
                hit=int(used.sum())
                escape=m-hit
                # Each fixed 32768-row group has a u32 base; a slot stores its escape's
                # 15-bit local ordinal. A contiguous raw fallback array costs 4 * escapes.
                pages=(m+32767)//32768
                descriptors=8*len(used)
                total=2*m+4*pages+4*escape+descriptors
                # This field accounts for a postpass keeping original and final simultaneously.
                w.writerow([b,stream,m,bits,k,capacity,len(counts),len(used),escape,2*m,descriptors,4*pages,4*escape,total,4*m+total,hit,escape])
        entropy_file.close()
        lower_file.close()
if __name__=='__main__':
    estimate(Path(sys.argv[1]),Path(sys.argv[2]))
