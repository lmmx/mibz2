"""Allow a separate seam for each distinct (word, original translation-run trace)."""
from bisect import bisect_right
from collections import Counter
from itertools import groupby,product
from pathlib import Path
import argparse
import csv
import sys

from optimize import Column, Direct, HERE, heavy_set, optimize


def traces(col, selected):
    inv=Direct(col, selected)
    ordered=sorted((a+col.maps[r],col.starts[r+1]+col.maps[r],r)
                   for r,a in enumerate(col.starts[:-1]))
    starts=[e[0] for e in ordered]
    classes=Counter()
    for a,b,width,word,_ in inv.bands:
        row=a
        trace=[]
        for c in word:
            lo,hi,r=ordered[bisect_right(starts,row)-1]
            assert lo<=row and row+width<=hi and col.symbols[r]==c
            row-=col.maps[r]
            trace.append(r)
        assert row==b
        classes[word,tuple(trace)]+=width
    assert sum(classes.values())==sum(b-a for a,b in selected)
    return classes


def check(col,selected,reference=None,brute=False):
    classes=traces(col,selected)
    words=[w for w,trace in classes]
    inc=[[(x,) for x in trace] for w,trace in classes]
    opt, chosen, price, f,b=optimize(words,inc)
    by_word={}
    for w,seam in zip(words,chosen):
        by_word.setdefault(w,set()).add(seam)
    if reference:
        assert (f,b)==(int(reference["all_forward"]),int(reference["all_backward"]))
        assert opt<=int(reference["minimum_run_events"])
    if brute:
        assert opt==min(price(t) for t in product(*(range(len(w)+1) for w in words)))
    return dict(words=len(by_word),trace_classes=len(classes),section_mass=sum(classes.values()),
                joined_run_optimum=reference["minimum_run_events"] if reference else "",
                independent_trace_optimum=opt,words_receiving_multiple_seams=sum(len(v)>1 for v in by_word.values()),
                assigned_word_seam_pairs=sum(map(len,by_word.values())),weighted_mass_in_split_words=
                sum(m for (w,_),m in classes.items() if len(by_word[w])>1))


def tiny():
    cases=0
    for n in range(1,7):
        for last in product(range(2),repeat=n):
            col=Column([(k,sum(1 for _ in g)) for k,g in groupby(last)])
            for symbol in (b"\x00",b"\x01",b"\x00\x01"):
                s=col.interval(symbol)
                if s[0]<s[1]:
                    check(col,[s],brute=True)
                    cases+=1
    print(f"{cases} arbitrary last columns/sections matched exhaustive per-trace seam assignments")


def main(tapes,blocks):
    sys.path.insert(0,str(HERE.parents[6]/"docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import paths,read_input
    r={(int(x["ordinal"]),x["section"]):x for x in csv.DictReader((HERE/"wikidata.csv").open())}
    writer=None
    for path in paths(tapes):
        idx=int(path.name.split(".")[0])
        if idx not in blocks:continue
        col=Column(read_input(path)[0])
        p=col.interval(b'"')
        for s,selected in (("quote",[p]),("frequent",heavy_set(col,p,1)[0])):
            row=dict(ordinal=idx,section=s,**check(col,selected,r[idx,s]))
            if writer is None:
                writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator="\n")
                writer.writeheader()
            writer.writerow(row)
            sys.stdout.flush()


if __name__=="__main__":
    p=argparse.ArgumentParser()
    p.add_argument("tapes",type=Path,nargs="?")
    p.add_argument("--blocks",type=int,nargs="+",default=[0,24,48,72,96,120,144,168,192,216])
    args=p.parse_args()
    main(args.tapes,args.blocks) if args.tapes else tiny()
