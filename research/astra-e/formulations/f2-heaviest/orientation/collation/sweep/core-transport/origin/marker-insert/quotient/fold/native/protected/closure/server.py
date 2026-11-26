"""Persistent, length-framed reference solver for the native full-cost experiment."""
from array import array
from collections import Counter
from struct import pack,unpack
from time import thread_time_ns
import sys
from solve import solve

assert sys.byteorder=='little' and array('I').itemsize==4
src=sys.stdin.buffer;out=sys.stdout.buffer
while header:=src.read(4):
    t=thread_time_ns()
    size=unpack('<I',header)[0];data=src.read(size);assert len(data)==size
    words=array('I');words.frombytes(data);n,h=words[:2]
    ix=2;g={};cost={}
    for _ in range(n):
        k,m,target,c=words[ix:ix+4];ix+=4
        if target!=4294967295:g[k]=target;cost[k]=c
    hs=Counter()
    for _ in range(h):
        width=words[ix];ix+=1;hs[tuple(words[ix:ix+width])]+=1;ix+=width
    assert ix==len(words)
    (q,neg),chosen,_=solve(g,cost,hs)
    payload=array('I',sorted(chosen)).tobytes()
    elapsed=thread_time_ns()-t
    out.write(pack('<IQII',len(chosen),elapsed,q,-neg));out.write(payload);out.flush()
