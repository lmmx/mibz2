"""Batch equal-byte excursions using two ordered queues of integer intervals."""
import argparse
import csv
from collections import Counter, deque
from pathlib import Path
import sys

from diagonal import buckets, census, sides
from residual import append_arc, induce, original
from screen import inputs


class Queue:
    def __init__(self, size, counts):
        self.active = size
        self.base = 0
        self.parts = deque([(0,size,0)] if size else [])
        self.counts = counts

    def take(self, size):
        while size:
            first, length, depth = self.parts.popleft()
            self.counts['pops'] += 1
            used = min(size,length)
            if used < length:
                self.parts.appendleft((first+used,length-used,depth))
                self.counts['splits'] += 1
            size -= used
            yield first,used,depth

    def rotate(self, width):
        if not width:
            return
        whole, rest = divmod(width,self.active)
        self.base += whole
        for first,w,d in self.take(rest):
            self.parts.append((first,w,d+1))
            self.counts['requeues'] += 1


def contract(bands, n, mark, observe=None):
    # All keys are the *first* entrance in the emitted interval. Each byte
    # consumes its intervals monotonically. Count every exact-key lookup.
    transfers = {}
    external = []
    counts = Counter(pops=0,splits=0,requeues=0,ranges=0,lookups=0,overlaps=0)
    entrance_mark = None
    support = 0
    for c,begin,end,left,pieces in buckets(bands):
        for x,w,b,y in pieces:
            if b != c:
                if observe:
                    observe('save_source',len(external))
                external.append((x,w,b,y))
        total = sum(w for _,w,b,_ in pieces if b != c)
        for low,high,rev,k,spans in sides(c,begin,end,left,pieces):
            q = Queue(k,counts)
            for lo,hi,inside in spans:
                w = hi-lo
                if inside:
                    if not q.active:
                        if lo <= mark < hi:
                            return (0,c,0,[]), dict(counts)
                        continue
                    if lo <= mark < hi:
                        before = hi-1-mark if rev else mark-lo
                        q.rotate(before)
                        e,_,depth = q.parts[0]
                        entrance_mark = (high-1-e if rev else low+e, 1+q.base+depth)
                        w -= before
                    q.rotate(w)
                else:
                    for e,length,depth in q.take(w):
                        entry = high-e-length if rev else low+e
                        exit_rank = support+(left+q.active-length if rev else left-q.active)
                        assert entry not in transfers
                        if observe:
                            observe('put',entry)
                        transfers[entry] = (entry+length,exit_rank,1+q.base+depth)
                        counts['ranges'] += 1
                        q.active -= length
            assert q.active == 0 and not q.parts
        support += total
    arcs = []
    cursor = [None]*256
    src = 0
    residual_mark = None
    for i,(x,w,b,y) in enumerate(external):
        if observe:
            observe('read_source',i)
        if x <= mark < x+w:
            residual_mark = src+mark-x, 0
        if entrance_mark is not None and y <= entrance_mark[0] < y+w:
            residual_mark = src+entrance_mark[0]-y, entrance_mark[1]
        while w:
            if cursor[b] is None or cursor[b][0] != y:
                if observe:
                    observe('get',y)
                stop,dest,length = transfers.pop(y)
                counts['lookups'] += 1
            else:
                _,stop,dest,length = cursor[b]
            take = min(w,stop-y)
            before = len(arcs)
            append_arc(arcs,src,dest,take,bytes([b])*length)
            if observe and len(arcs)>before:
                observe('save_residual',before)
            counts['overlaps'] += 1
            src += take
            y += take
            w -= take
            cursor[b] = (y,stop,dest+take,length) if y < stop else None
    assert src == support and not transfers and residual_mark is not None
    return (support,*residual_mark,arcs),dict(counts)


def decode(bands,n,p):
    (support,mark,offset,arcs),counts = contract(bands,n,p)
    if not support:
        return bytes([mark])*n, dict(period=1, fixed_mark=1, **counts)
    out,row = induce(arcs,support,mark,n,offset)
    return out, dict(**row, fixed_mark=0, **counts)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('folder',type=Path)
    parser.add_argument('--blocks',nargs='+',type=int)
    args = parser.parse_args()
    writer = None
    for block,n,p,_,bands in inputs(args.folder,args.blocks):
        control,base = induce(original(bands),n,p,n)
        out,row = decode(bands,n,p)
        assert control == out
        result = dict(block=block, original_cuts=base['cuts'], **census(bands), **row)
        if writer is None:
            writer = csv.DictWriter(sys.stdout,fieldnames=result,lineterminator='\n')
            writer.writeheader()
        writer.writerow(result)

if __name__ == '__main__':
    main()
