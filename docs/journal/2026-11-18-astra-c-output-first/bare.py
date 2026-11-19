"""Histogram-only inversion and exact full-period cost without executing the long scans."""
import argparse
import csv
import itertools
import sys

from inputs import dense_audit, paths, read_input
from small import runs_of


def caps(runs):
    for c, width in runs:
        while width > 127:
            yield c,127
            width -= 127
        yield c,width


def byte_at(ends, x):
    lo, hi, tests = 0,256,0
    while lo < hi:
        mid = (lo+hi)//2
        tests += 1
        if x < ends[mid]:
            hi = mid
        else:
            lo = mid+1
    return lo,tests


def totals(records):
    hist = [0]*256
    for c,w in records:
        hist[c] += w
    return list(itertools.accumulate(hist))


def literal_cursor(records, x):
    ends = totals(records)
    out = bytearray()
    reads = lines = tests = 0
    for i in range(ends[-1]):
        c,t = byte_at(ends,x)
        tests += t
        out.append(c)
        if i+1 == ends[-1]:
            break
        need = x-(ends[c-1] if c else 0)
        top = 0
        for j,(v,w) in enumerate(records):
            if c==v:
                if need < w:
                    x = top+need
                    break
                need -= w
            top += w
        else:
            raise AssertionError("missing occurrence")
        reads += j+1
        lines += j//32+1
    return bytes(out),(reads,lines,tests)


def full_cost(records, p):
    ends = totals(records)
    scans = spans = tests = top = mark = 0
    for i,(c,w) in enumerate(records):
        scans += (i+1)*w
        spans += (1+i//32)*w
        tests += w*byte_at(ends,ends[c]-1)[1]
        if top<=p<top+w:
            mark = i
        top += w
    return scans-mark-1,spans-mark//32-1,tests


def check_small():
    count = full = 0
    for n in range(1,9):
        for word in itertools.product(range(2),repeat=n):
            runs=runs_of(bytes(word))
            rec=list(caps(runs))
            for p in range(n):
                _,want,orb=dense_audit(runs,p)
                out,cost=literal_cursor(rec,p)
                assert out == want
                count += 1
                if orb.index(p,1)==n:
                    assert cost == full_cost(rec,p)
                    full += 1
    return count,full


if __name__ == "__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input")
    args=parser.parse_args()
    checked,full=check_small()
    writer=csv.writer(sys.stdout,lineterminator="\n")
    writer.writerow("block n capped histogram_reads select_queries cap_reads cap_bytes "
                    "select_line_spans byte_bucket_comparisons output_writes "
                    "small_marks small_full_periods".split())
    for path in paths(args.input):
        runs,p=read_input(path)
        rec=list(caps(runs))
        column,_,orb=dense_audit(runs,p)
        n=len(column)
        assert orb.index(p,1)==n
        reads,lines,tests=full_cost(rec,p)
        writer.writerow([int(path.name.split('.')[0]),n,len(rec),len(rec),n-1,
                         reads,2*reads,lines,tests,n,checked,full])
