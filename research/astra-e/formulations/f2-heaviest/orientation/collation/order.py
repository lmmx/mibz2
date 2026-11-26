"""Collate return words on an arbitrary contiguous section using one fixed section-tail itinerary."""
from collections import defaultdict
from functools import cmp_to_key
from itertools import groupby, product
from pathlib import Path
import argparse
import csv
import hashlib
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]/"union"))
from direct import Column,heavy_set
from audit import Oracle


def truth(oracle, section):
    a,b=section
    words={}
    incoming=[None]*(b-a)
    for s in range(a,b):
        row=s
        word=bytearray()
        while True:
            word.append(oracle.first[row])
            row=oracle.forward[row]
            if a<=row<b:
                break
        key=bytes(word)
        assert incoming[row-a] is None
        incoming[row-a]=key
        words.setdefault(key,[]).append((s,row))
    for entries in words.values():
        assert all(u[0]+1==v[0] and u[1]<v[1] for u,v in zip(entries,entries[1:]))
    return words,incoming


def collation(oracle, words, q):
    longest=max(map(len,words)) if len(words)>1 else 0
    row=q
    tail=bytearray()
    offset=[q]
    for _ in range(longest):
        tail.append(oracle.first[row])
        row=oracle.forward[row]
        offset.append(row)
    compares=prefix=same=0

    def compare(x,y):
        nonlocal compares,prefix,same
        compares+=1
        if x==y:return 0
        if x.startswith(y):return -shorter(y,x)
        if y.startswith(x):return shorter(x,y)
        return -1 if x<y else 1

    def shorter(x,y):
        nonlocal prefix,same
        prefix+=1
        # X T vs X v T. If T starts with v use the shifted rank for the final comparison.
        v=y[len(x):]
        initial=tail[:len(v)]
        if initial != v:
            return -1 if initial < v else 1
        same+=1
        assert offset[len(v)]!=q
        return -1 if offset[len(v)]<q else 1

    return sorted(words,key=cmp_to_key(compare)),dict(tail_steps=longest,compare=compares,prefix=prefix,shift=same)


def check(oracle,section,qs,marks,expected_digest=None):
    a,b=section
    words,incoming=truth(oracle,section)
    truth_order=sorted(words,key=lambda w:words[w][0][0])
    counts={w:len(t) for w,t in words.items()}
    ambiguous=0
    totals=defaultdict(int)
    last=None
    for q in qs:
        order,cost=collation(oracle,words,q)
        assert order==truth_order
        if last is None:
            last=order
        if len(oracle.first)<=15:
            # The first unequal position is within max word length plus one tail period.
            tail=bytearray()
            row=q
            length=max(map(len,words))+len(oracle.first)
            for _ in range(length):
                tail.append(oracle.first[row])
                row=oracle.forward[row]
            keys=[(w+tail)[:length] for w in words]
            assert len(set(map(bytes,keys)))==len(words)
            assert sorted(words,key=lambda w:(w+tail)[:length])==order
        for k,v in cost.items():totals[k]+=v
    assert last is not None
    by_source=[]
    next_slot={}
    for w in last:
        next_slot[w]=a+len(by_source)
        by_source.extend([w]*counts[w])
    succ=[0]*(b-a)
    for dest,w in enumerate(incoming,a):
        pos=next_slot[w]
        next_slot[w]+=1
        succ[pos-a]=dest
    assert all(succ[u-a]==v for cells in words.values() for u,v in cells)
    marked=disjoint=0
    for p in marks:
        row=p
        need=len(oracle.first)
        out=bytearray()
        while len(out)<need and not a<=row<b:
            out.append(oracle.first[row])
            row=oracle.forward[row]
        if len(out)==need and not a<=row<b:disjoint+=1
        while len(out)<need:
            w=by_source[row-a]
            out.extend(w[:need-len(out)])
            row=succ[row-a]
        if expected_digest:
            assert hashlib.sha256(out).hexdigest()==expected_digest
        else:
            want=bytearray()
            row=p
            for _ in range(need):
                want.append(oracle.first[row])
                row=oracle.forward[row]
            assert out==want
        marked+=1
    return dict(types=len(words),section_rows=b-a,bare_lex_wrong=int(last!=sorted(words)),fixed_tails=len(qs),
                marked=marked,disjoint=disjoint,**totals)


def small():
    totals=defaultdict(int)
    witness=None
    for k,maximum in ((2,9),(3,6)):
        for n in range(1,maximum+1):
            for letters in product(range(k),repeat=n):
                if k==3 and 2 not in letters:continue
                oracle=Oracle(bytes(letters))
                for a in range(n):
                    for b in range(a+1,n+1):
                        res=check(oracle,(a,b),range(a,b),range(n))
                        totals["sections"]+=1
                        for key in res:totals[key]+=res[key]
                        if res["bare_lex_wrong"] and witness is None:
                            words,_=truth(oracle,(a,b))
                            correct=sorted(words,key=lambda w:words[w][0][0])
                            witness=dict(last=bytes(letters).hex(),section=[a,b],
                                         actual=[w.hex() for w in correct],bare=[w.hex() for w in sorted(words)])
    import json
    print(json.dumps(dict(counters=totals,bare_lex_witness=witness),indent=2))


def real(tapes):
    sys.path.insert(0,str(HERE.parents[5]/"docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import paths,read_input
    with (HERE.parents[5]/"docs/journal/2026-11-18-astra-c-output-first/section.csv").open() as f:
        digests={int(r["block"]):r["marked_sha256"] for r in csv.DictReader(f)}
    wr=None
    for path in paths(tapes):
        idx=int(path.name.split(".")[0])
        runs,primary=read_input(path)
        col=Column(runs)
        oracle=Oracle(b"".join(bytes([c])*width for c,width in runs))
        n=col.rows
        p=col.interval(b'"')
        cases=[("quote",p),("frequent",heavy_set(col,p,1)[0][0]),
               ("fifth",(n//5,n//5+n//11)),("half",(n//2,n//2+n//13))]
        for name,(a,b) in cases:
            res=check(oracle,(a,b),sorted({a,(a+b-1)//2,b-1}),[primary],digests[idx])
            row=dict(ordinal=idx,section=name,left=a,right=b,marked_sha256=digests[idx],**res)
            if wr is None:
                wr=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator="\n")
                wr.writeheader()
            wr.writerow(row)
            sys.stdout.flush()


if __name__=="__main__":
    p=argparse.ArgumentParser()
    p.add_argument("tapes",nargs="?",type=Path)
    args=p.parse_args()
    real(args.tapes) if args.tapes else small()
