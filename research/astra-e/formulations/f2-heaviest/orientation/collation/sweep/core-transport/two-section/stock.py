"""Untimed call of the existing second-stage code. The case supplies a valid positive-weight mark."""
import ctypes as ct
from pathlib import Path
import sys

sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'native'))
from ordinal_writer import Byte,Object,Owned,Piece,U32


class Stock:
    def __init__(self,library):
        self.lib=ct.CDLL(str(Path(library).resolve()))
        self.lib.consume.argtypes=[ct.POINTER(Object),ct.POINTER(U32)]
        self.lib.consume.restype=Owned
        self.lib.release.argtypes=[Owned]
        self.lib.release.restype=None
        self.lib.token_capacity.argtypes=[ct.POINTER(Object)]
        self.lib.token_capacity.restype=ct.c_size_t

    def __call__(self,model,row,skip,n,capacity=False):
        offsets={}
        data=bytearray()
        slices=[]
        for w in model.weights:
            if w not in offsets:
                offsets[w]=len(data)
                data.extend(w)
            slices.append(Piece(offsets[w],len(w)))
        words=(Piece*len(slices))(*slices)
        backing=(Byte*len(data)).from_buffer(data)
        bases=(U32*len(model.base))(*model.base)
        runs=(Piece*len(model.column))(*(Piece(k,w) for k,w in model.column))
        obj=Object(runs,len(runs),words,bases,len(words),backing,len(backing),row,skip,2*model.m,n)
        used=U32()
        out=self.lib.consume(ct.byref(obj),ct.byref(used))
        try:
            measured=int(self.lib.token_capacity(ct.byref(obj))) if capacity else 0
            return ct.string_at(out.data,out.len),int(used.value),measured
        finally:
            self.lib.release(out)
