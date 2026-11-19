"""Untimed ABI test of unmodified second Compact. Only the independent oracle expands small columns."""
import ctypes as C
from collections import Counter
from itertools import groupby, product
from pathlib import Path
import sys

U32, Size, Byte = C.c_uint32, C.c_size_t, C.c_uint8
class Piece(C.Structure):
    _fields_ = [("id",U32),("width",U32)]
class Object(C.Structure):
    _fields_ = [("root",C.POINTER(Piece)),("r",Size),("words",C.POINTER(Piece)),
                ("bases",C.POINTER(U32)),("k",Size),("dict",C.POINTER(Byte)),("bytes",Size),
                ("mark",U32),("skip",U32),("tokens",U32),("n",U32)]
class Owned(C.Structure):
    _fields_ = [("data",C.POINTER(Byte)),("len",Size),("capacity",Size)]


def check(lib,last,alphabet,counts):
    size=len(last)
    # The source order reverses numeric IDs. The first two IDs share the exact
    # same physical dictionary slice. The third is lexically earlier and shorter.
    book=[b'q\x00',b'q\x00',b'!'][:alphabet]
    if alphabet==1:
        book=[b'q\x00']
    backing=(Byte*3)(ord('q'),0,ord('!'))
    words=(Piece*alphabet)(*(Piece(2 if i==2 else 0,len(s)) for i,s in enumerate(book)))
    source=sorted(range(size),key=lambda i:-last[i])
    tokens=[last[j] for j in source]
    bases=(U32*alphabet)(*(tokens.index(i) for i in range(alphabet)))
    packed=[Piece(i,sum(1 for _ in it)) for i,it in groupby(last)]
    runs=(Piece*len(packed))(*packed)
    n=sum(len(book[i]) for i in last)
    heavy=max(range(alphabet),key=lambda i:(last.count(i),-bases[i]))
    for start in range(size):
        orbit=[]
        x=start
        while not orbit or x!=start:
            orbit.append(tokens[x])
            x=source[x]
        period=b''.join(book[i] for i in orbit)
        for skip in range(len(book[tokens[start]])):
            original=Object(runs,len(runs),words,bases,alphabet,backing,len(backing),start,skip,size,n)
            second=U32()
            out=lib.consume(C.byref(original),C.byref(second))
            try:
                result=C.string_at(out.data,out.len)
            finally:
                lib.release(out)
            ref=(period*((n+skip+len(period)-1)//len(period)))[skip:skip+n]
            assert result==ref
            anticipated=last.count(heavy)!=size and heavy in orbit
            assert bool(second.value)==anticipated
            counts["marked"]+=1
            counts["second" if anticipated else "fallback"]+=1
            counts["second_with_nonsingleton_spelling_aliases"]+=int(anticipated and alphabet>=2)
            counts["second_with_alias_and_distinct_physical_spelling"]+=int(anticipated and alphabet==3)
            counts["multicycle_partial_byte_lap"]+=int(len(orbit)<size and n%len(period)!=0)
            counts["in_word_skip"]+=int(skip>0)


if __name__=="__main__":
    lib=C.CDLL(str(Path(sys.argv[1]).resolve()))
    lib.consume.argtypes=[C.POINTER(Object),C.POINTER(U32)]
    lib.consume.restype=Owned
    lib.release.argtypes=[Owned]
    lib.release.restype=None
    stats=Counter()
    for k in (1,2,3):
        for m in range(k,7):
            for last in product(range(k),repeat=m):
                if len(set(last))!=k:
                    continue
                check(lib,last,k,stats)
                stats["columns"]+=1
    for k in sorted(stats):
        print(k,stats[k])
