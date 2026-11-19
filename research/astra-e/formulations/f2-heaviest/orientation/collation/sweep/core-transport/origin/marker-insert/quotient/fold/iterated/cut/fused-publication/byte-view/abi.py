"""Exercise all complete C++ paths against the ordinary marked stable permutation."""
import ctypes, json, sys
from pathlib import Path
from itertools import product
from collections import Counter
sys.dont_write_bytecode=True
sys.path.insert(0,str(Path(__file__).resolve().parent))
from seed import get_caps
lib=ctypes.CDLL(str(Path(sys.argv[1]).resolve()))
C=ctypes.c_uint32
lib.grammar_new.restype=ctypes.c_void_p
lib.grammar_delete.argtypes=[ctypes.c_void_p]
lib.grammar_run.argtypes=[ctypes.c_void_p,ctypes.POINTER(ctypes.c_uint16),C,C,C,C]
lib.grammar_run.restype=ctypes.POINTER(ctypes.c_ubyte)
state=[lib.grammar_new() for _ in range(3)]
counts=Counter()
def check(col,mark,caps):
    a=sorted(range(len(col)),key=lambda p:(col[p],p))
    p=mark;want=bytearray()
    for _ in col:
        want.append(col[a[p]]);p=a[p]
    rec=(ctypes.c_uint16*len(caps))(*(k*256+n for k,n in caps))
    for i,s in enumerate(state):
        y=lib.grammar_run(s,rec,len(caps),len(col),mark,i)
        assert bytes(y[:len(col)])==want
try:
    for n in range(1,7):
        for col in product((3,87,255),repeat=n):
            for cap in get_caps(col):
                for m in range(n):
                    check(col,m,cap);counts['triples']+=1
    for n in (17,128,16384):
        col=[255]+[3]*n;caps=[(255,1)]+[(3,min(127,n-i)) for i in range(0,n,127)]
        for m in [0,1,n//2,n]:
            check(col,m,caps);counts['long_marks']+=1
finally:
    for s in state:lib.grammar_delete(s)
print(json.dumps(counts,indent=2))
