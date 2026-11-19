"""Exact incoming-state and marked-word screen for RLE1-expanded interiors."""
import argparse
import bz2
import collections
import csv
import hashlib
import itertools
import re
import struct
import sys
from dataclasses import dataclass
from pathlib import Path

EMPTY = (0, 0)
QUAD = re.compile(rb"(.)\1\1\1", re.DOTALL)

def feed(state, data, out):
    last, same = state
    for b in data:
        if same == 4:
            out.extend(bytes([last]) * b)
            last, same = EMPTY
        else:
            out.append(b)
            same = same + 1 if same and b == last else 1
            last = b
    return (last, same)

def exact_cut(w):
    # After consuming one symbol from an arbitrary incoming state, the last
    # emitted literal byte (when same>0) is w[0]; all five counts are possible.
    mask = 0b11111
    for i in range(1,len(w)):
        if mask.bit_count() == 1 or (not mask & 16 and w[i] != w[i-1]):
            return i
        if w[i] != w[i-1]:
            mask = (1 if mask & 16 else 0) | (2 if mask & 15 else 0)
        else:
            mask = (mask & 15) << 1 | (mask >> 4)
    return len(w)

@dataclass
class Phrase:
    w: bytes
    decoded: bytes
    after: tuple
    lo: int
    hi: int
    middle: bytes
    cut: int
    output_cut: int

def compile_phrase(w):
    assert w
    decoded=bytearray()
    after=feed(EMPTY,w,decoded)
    cuts=[i for i in range(4,len(w)) if w[i] != w[i-1] and len(set(w[i-4:i]))>1]
    lo,hi=(cuts[0],cuts[-1]) if len(cuts)>=2 else (0,0)
    mid=bytearray()
    feed(EMPTY,w[lo:hi],mid)
    cut=exact_cut(w)
    prefix=bytearray()
    feed(EMPTY,w[:cut],prefix)
    return Phrase(w,bytes(decoded),after,lo,hi,bytes(mid),cut,len(prefix))

def pair_action(p,state,out):
    if p.hi==0:
        return feed(state,p.w,out)
    feed(state,p.w[:p.lo],out)
    out.extend(p.middle)
    return feed(EMPTY,p.w[p.hi:],out)

def action(p,state,out,guard):
    # Return ending state, reusable input bytes, copied output bytes, mode.
    if guard and (not state[1] or state[1]!=4 and state[0]!=p.w[0]):
        out.extend(p.decoded)
        return p.after,len(p.w),len(p.decoded),0
    if p.cut==len(p.w):
        return feed(state,p.w,out),0,0,2
    feed(state,p.w[:p.cut],out)
    out.extend(p.decoded[p.output_cut:])
    return p.after,len(p.w)-p.cut,len(p.decoded)-p.output_cut,1

def small():
    # The classes for differential testing are empty, tail/count on either
    # input letter, and tail/count on a third byte. The count-state byte
    # determines output payload even when it never appears in the word.
    states=[EMPTY]+[(c,n) for c in [0,1,239] for n in range(1,5)]
    checked=0
    adversaries=[b'A'*k for k in [4,5,8,9,10,19,128]]
    adversaries += [b'AAAA\xff',b'AAAA\0BBBB', b'\xff\xff\xff\xff\xffAAAA',b'CAAABBBBAAAAB',
                    b'\x00'*10+b'\xff'*5,bytes(range(256)),b'aaabbbcddeeeeefffaaaa']
    words = (bytes(a) for n in range(1,10) for a in itertools.product([0,1], repeat=n))
    for w in itertools.chain(words,adversaries):
        p=compile_phrase(w)
        for state in set(states+[(w[0],i) for i in range(1,5)]):
            ref=bytearray()
            after=feed(state,w,ref)
            alt=bytearray()
            assert pair_action(p,state,alt)==after and alt==ref,(state,w,'pair')
            for guard in [False,True]:
                alt=bytearray()
                assert action(p,state,alt,guard)[0]==after and alt==ref,(state,w,guard)
            checked+=1
    # Repeat a short marked spelling into a partial final word, including a
    # count that begins or finishes across a repetition. Compare scalar token
    # parsing and the unfinished-count error at the actual block boundary.
    periodic=0
    for period in [(b'AAA',b'A\xffB'),(b'\0\0', b'\0\0'),(b'AAAA',b'AAAAA'),(b'ABCDEF',)]:
        for n in range(1,70):
            raw=(b''.join(period)*100)[:n]
            pos=0
            state=EMPTY
            out=bytearray()
            for w in itertools.cycle(period):
                if pos+len(w)>n:
                    state=feed(state,w[:n-pos],out);break
                state,*_=action(compile_phrase(w),state,out,True)
                pos+=len(w)
                if pos==n:break
            i=0; oracle=bytearray();unfinished=False
            while i<n:
                if i+3<n and len(set(raw[i:i+4]))==1:
                    if i+4==n:unfinished=True;break
                    oracle.extend(raw[i:i+1]*(4+raw[i+4]));i+=5
                else:
                    oracle.append(raw[i]);i+=1
            assert (state[1]==4)==unfinished
            assert unfinished or out==oracle
            periodic+=1
    print(f"PASS {checked} independent incoming-state/word cases across all three word actions; {periodic} truncated-period/error checks")

def read_tape(path):
    data=path.read_bytes()
    words=[]
    count,=struct.unpack_from('<I',data)
    pos=4
    for _ in range(count):
        size,=struct.unpack_from('<I',data,pos);pos+=4
        words.append(data[pos:pos+size]);pos+=size
    return words,[s for s, in struct.iter_unpack('<I',data[pos:])]

def independent(corpus,meta):
    # Rewrap the precise scanner bit interval with its own block and stream CRC.
    bit,end=map(int,(meta['bit'],meta['end']))
    first,last=bit//8,(end+7)//8
    fragment=int.from_bytes(corpus[first:last],'big')>>(8*last-end)
    width=end-bit
    fragment &= (1<<width)-1
    assert fragment>>(width-48)==0x314159265359
    trailer=(0x177245385090<<32)|int(meta['crc'])
    bits=(fragment<<80)|trailer
    padding=(-width-80)%8
    raw=b'BZh'+meta['level'].encode()+(bits<<padding).to_bytes((width+80+padding)//8,'big')
    return bz2.decompress(raw)

def legacy_calls(raw):
    n=len(raw);i=0;windows=0;spans=0;count_tokens=0;tail_literals=0
    while i+35<=n:
        # Window can detect run starts in i..i+31 inclusive.
        windows+=1
        m=QUAD.search(raw,i,i+35)
        if m is None:
            spans+=1;i+=32
        else:
            j=m.start();spans+=int(j>i)+1+int(raw[j+4]!=0);count_tokens+=1;i=j+5
    while i<n:
        if i+3<n and len(set(raw[i:i+4]))==1:
            spans+=1+int(raw[i+4]!=0);count_tokens+=1;i+=5
        else:
            tail_literals+=1;i+=1
    return windows,spans,count_tokens,tail_literals

def measure(meta,words,codes,expected):
    phrases=[compile_phrase(w) for w in words]
    n=int(meta['n'])
    kinds=[bytes([c]) if c<256 else words[c-256] for c in codes]
    raw=b''.join(kinds)
    # This is true of the sampled real blocks, periodic partial words above
    # have a separate executable oracle.
    assert len(raw)==n==int(meta['period_bytes'])
    legacy=legacy_calls(raw)
    counts=collections.Counter()
    out=bytearray();state=EMPTY
    pairout=bytearray();pairstate=EMPTY
    suffixout=bytearray();suffixstate=EMPTY
    stretches=[]
    run=0
    for c in codes:
        if c<256:
            b=bytes([c])
            state=feed(state,b,out);pairstate=feed(pairstate,b,pairout);suffixstate=feed(suffixstate,b,suffixout)
            run+=1;counts['literal_entries']+=1
            continue
        if run:stretches.append(run);run=0
        p=phrases[c-256]
        counts['dict_entries']+=1
        counts['dict_input_bytes']+=len(p.w)
        if p.hi:
            counts['pair_interior_bytes']+=p.hi-p.lo
            counts['pair_occ']+=1
        pairstate=pair_action(p,pairstate,pairout)
        suffixstate,used,_,_=action(p,suffixstate,suffixout,False)
        counts['suffix_bytes']+=used
        state,used,emitted,mode=action(p,state,out,True)
        counts['guard_input_bytes']+=used; counts['guard_output_bytes']+=emitted
        counts[['whole','partial','fallback'][mode]]+=1
        counts['prefix_or_fallback_bytes']+=len(p.w)-used
    if run:stretches.append(run)
    assert state==pairstate==suffixstate and state[1]!=4
    assert out==expected and pairout==expected and suffixout==expected
    # Counters below describe a streaming implementation that aliases identical
    # spelled dictionary bytes. Python above retains all three oracle outputs.
    changed=[p for p in phrases if p.decoded!=p.w]
    row=dict(ordinal=meta['ordinal'],admitted=meta['admitted'],n=n,decoded=len(out),
             words=len(words),unique_input=sum(map(len,words)),changed_words=len(changed),
             owned_decoded=sum(len(p.decoded) for p in changed),legacy_windows=legacy[0],
             legacy_unrle_spans=legacy[1],legacy_count_tokens=legacy[2],legacy_tail_literals=legacy[3],
             literal_stretches=len(stretches),stretches_le3=sum(r<=3 for r in stretches),
             literal_in_short=sum(r for r in stretches if r<=3),longest=max(stretches,default=0))
    for k in ['literal_entries','dict_entries','dict_input_bytes','pair_interior_bytes','pair_occ','suffix_bytes','guard_input_bytes','guard_output_bytes','whole','partial','fallback','prefix_or_fallback_bytes']:
        row[k]=counts[k]
    return row

def real(args):
    data=Path(args.corpus).read_bytes()
    assert hashlib.sha256(data).hexdigest()=="82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
    results=[]
    for meta in csv.DictReader(Path(args.inputs).open()):
        words,codes=read_tape(Path(args.tapes)/(meta['ordinal']+'.words'))
        results.append(measure(meta,words,codes,independent(data,meta)))
    w=csv.DictWriter(sys.stdout,fieldnames=results[0],lineterminator='\n');w.writeheader();w.writerows(results)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['small','real'])
    p.add_argument('--corpus');p.add_argument('--inputs');p.add_argument('--tapes')
    a=p.parse_args()
    if a.mode=='small':small()
    else:real(a)
