"""Drive the compiled shared-dictionary cap ABI on arbitrary short cycles and long cap seams."""
import ctypes as c
from itertools import groupby,product
import json
from pathlib import Path
import sys
lib=c.CDLL(str(Path(sys.argv[1]).resolve()))
lib.grammar_new.restype=c.c_void_p
lib.grammar_delete.argtypes=[c.c_void_p]
lib.grammar_run.argtypes=[c.c_void_p,c.POINTER(c.c_uint16),c.c_uint32,c.c_uint32,c.c_uint32,c.c_uint32]
lib.grammar_run.restype=c.c_void_p
s=lib.grammar_new()
marked=repairs=invocations=0
def check(last,marks):
    global marked,repairs,invocations
    psi=sorted(range(len(last)),key=last.__getitem__)
    encoded=[]
    for a,g in groupby(last):
        n=sum(1 for _ in g)
        while n:
            count=min(n,127);encoded.append((a<<8)|count);n-=count
    caps=(c.c_uint16*len(encoded))(*encoded)
    for mark in marks:
        x=mark;out=bytearray()
        for i in range(len(last)):
            x=psi[x];out.append(last[x])
        repairs+=x!=mark
        for arm in range(3):
            ptr=lib.grammar_run(s,caps,len(caps),len(last),mark,arm)
            assert c.string_at(ptr,len(last))==out
            invocations+=1
        marked+=1
try:
    for n in range(1,6):
        for data in product((0,255),repeat=n):check(bytes(data),range(n))
    for m in (126,127,128,254,255,256,900):
        for data in (bytes(m+2),bytes([255])*m+b'ab',b'aa'+bytes([125])*m,
                     b'b'+b'a'*(m+1),b'ba'+b'b'+b'a'*(m+1),b'a'*m+b'b'*m):
            check(data,sorted(set([0,1,m-1,len(data)-1])))
finally:
    lib.grammar_delete(s)
print(json.dumps(dict(marked_cases=marked,nondivisor=repairs,native_invocations=invocations),indent=2))
