"""Send the physical RLE to the unchanged second Compact writer, pricing every mark transfer hop."""
from bisect import bisect_right
import csv
import ctypes as ct
import hashlib
from pathlib import Path
import sys

from physical import Atlas, Column, HERE, Oracle, heavy_set, incoming, physical
sys.path.insert(0, str(HERE.parent/'native'))
from ordinal_writer import Byte, Object, Owned, Piece, U32
sys.path.insert(0, str(HERE.parents[8]/'docs/journal/2026-11-18-astra-c-output-first'))
from inputs import paths, read_input


def marked(col, atlas, mark):
    # Physical predecessor search and LF, both on original runs/bands.
    starts = []
    offsets = []
    total = 0
    for _, lo, width, _ in atlas.leaves:
        starts.append(lo)
        offsets.append((lo+width, total))
        total += width
    x = mark
    for skip in range(col.rows):
        band = bisect_right(starts,x)-1
        if band>=0 and x<offsets[band][0]:
            return offsets[band][1]+x-starts[band], skip
        run = bisect_right(col.starts,x)-1
        x += col.maps[run]
        if x==mark:
            return None
    raise AssertionError('LF did not close')


def native(lib,col,sec,mark,h):
    atlas = Atlas(col,sec,h)
    stream, cores, minima, counts, _ = incoming(col,sec,atlas)
    column, labels, _, _ = physical(sec,atlas,stream,cores,minima,counts)
    points = sorted(labels)
    names = {start:i for i,start in enumerate(points)}
    bases = (U32*len(points))(*points)
    a = []
    arena = bytearray()
    for start in points:
        core, prefix = labels[start]
        width = len(cores[core])+len(prefix)
        a.append(Piece(len(arena),width))
        arena.extend(cores[core])
        arena.extend(prefix)
    words = (Piece*len(points))(*a)
    backing = (Byte*len(arena)).from_buffer(arena)
    runs = (Piece*len(column))(*(Piece(names[k],w) for k,w in column))
    location = marked(col,atlas,mark)
    if location is None:
        return None
    v,skip = location
    obj = Object(runs,len(runs),words,bases,len(words),backing,len(backing),v,skip,sec[1]-sec[0],col.rows)
    second = U32()
    out = lib.consume(ct.byref(obj),ct.byref(second))
    try:
        data = ct.string_at(out.data,out.len)
    finally:
        lib.release(out)
    return data,int(second.value),v,skip


def main(library,tapes):
    lib = ct.CDLL(str(Path(library).resolve()))
    lib.consume.argtypes=[ct.POINTER(Object),ct.POINTER(U32)]
    lib.consume.restype=Owned
    lib.release.argtypes=[Owned]
    lib.release.restype=None
    writer = csv.writer(sys.stdout,lineterminator='\n')
    writer.writerow(('ordinal','section','h','second','physical_mark','byte_skip','n','sha256'))
    for tape in paths(tapes):
        runs,mark=read_input(tape)
        col=Column(runs)
        o=Oracle(b''.join(bytes([c])*w for c,w in runs))
        expected=bytearray()
        x=mark
        for _ in range(col.rows):
            expected.append(o.first[x])
            x=o.forward[x]
        digest=hashlib.sha256(expected).hexdigest()
        q=col.interval(b'"')
        for name,u in [('frequent',heavy_set(col,q,1)[0][0]),('quote',q)]:
            for h in (0,4,8,12):
                data,taken,v,skip=native(lib,col,u,mark,h)
                assert data==expected
                writer.writerow((int(tape.name.split('.')[0]),name,h,taken,v,skip,col.rows,digest))
                sys.stdout.flush()


if __name__=='__main__':
    main(*sys.argv[1:])
