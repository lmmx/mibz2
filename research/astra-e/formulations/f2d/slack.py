"""Test interval certificates learned only while taking otherwise necessary unary steps."""
import argparse
from collections import Counter
import csv
from pathlib import Path
import struct
import sys

from chains import Column

def audit(c, paths, cap):
    # One descriptor per accessed starting run. A later nonfitting query replaces it.
    cache = {}
    count = Counter()
    for unary, stop in paths:
        trace = unary + [stop]
        pos = 0
        count["lookup"] += 1
        while pos < len(unary):
            start, lo, hi = unary[pos]
            count["probe"] += 1
            prior = cache.get(start)
            if prior is not None:
                a, b, delta, spelling = prior
                if a <= lo and hi <= b:
                    count["hit"] += 1
                    h = len(spelling)
                    after = trace[pos+h]
                    assert bytes(c.ids[t[0]] for t in trace[pos:pos+h]) == spelling
                    assert (lo+delta, hi+delta) == after[1:]
                    # No promise about the next run; pay a fresh first-run query.
                    count["lookup"] += 1
                    count["jump_bytes"] += h
                    pos += h
                    continue
            # At a cache miss, discover the next cap unary steps normally, or
            # stop on a branch/delimiter. The hypothetical loop knows a step is
            # unary only after its normal run-boundary test.
            used = unary[pos:pos+cap]
            a, b = c.starts[start], c.starts[start+1]
            spelling = bytearray()
            for i, x, y in used:
                delta = x-lo
                a = max(a, c.starts[i]-delta)
                b = min(b, c.starts[i+1]-delta)
                spelling.append(c.ids[i])
                count["lookup"] += 1
                count["intersections"] += 1
            h = len(used)
            assert a <= lo < hi <= b and h>0
            pos+=h
            if h>=2:
                cache[start]=(a,b,trace[pos][1]-lo,bytes(spelling))
                count["install"] += 1
                count["installed_bytes"] += h
    count["live_descriptors"] = len(cache)
    return count

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder",type=Path)
    args=parser.parse_args()
    out=csv.writer(sys.stdout)
    keys="lookup probe hit jump_bytes intersections install installed_bytes live_descriptors".split()
    out.writerow(["stream","cap","old_lookups",*keys])
    for path in sorted(args.folder.glob("*.recs"),key=lambda p:int(p.stem)):
        c=Column([x[0] for x in struct.iter_unpack("<H",path.read_bytes())])
        paths,_=c.actual()
        old=sum(len(p)+1 for p,_ in paths)
        for cap in (2,4,8,16):
            events=audit(c,paths,cap)
            assert old-events["lookup"]==events["jump_bytes"]-events["hit"]
            out.writerow([path.stem,cap,old,*[events[k] for k in keys]])

if __name__=="__main__":
    main()
