"""Prove depth-specific lookup optima for independent cut positions inside a shared prefix."""
from collections import Counter
from itertools import product
from pathlib import Path
import csv
import json
import sys

here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parent))
from optimize import Column, Oracle, heavy_set, objects, suffixes


def common(words):
    left,right=min(words),max(words)
    length=0
    while length<min(len(left),len(right)) and left[length]==right[length]:
        length+=1
    return left[:length]


def levels(words,prefix):
    q=suffixes(words)
    prior_exact=True
    for h in range(1,len(prefix)+1):
        p=prefix[:h]
        tails=[w[h:] for w in words]
        new=suffixes([v+p for v in tails])
        assert new<=q
        q=new
        # If all tails are empty the set has one type and uniform motion
        # cannot shorten that type's shared prefix for free in general.
        nontrivial=any(tails)
        bad=sum(len(v)>=2 and v[-1] in p for v in tails)
        count=suffixes(tails)-1
        if nontrivial:
            assert new==1+h+count
        sufficient=nontrivial and bad==0
        prior_exact=prior_exact and sufficient
        yield dict(depth=h,prefix_hex=p.hex(),uniform_Q=new,tail_proper_nonempty=count,
                   interfering_word_types=bad,sufficient_this_h=sufficient,certified_for_0_to_h=prior_exact)


def small():
    tally=Counter()
    first={}
    for n in range(1,8):
        for raw in product(range(2),repeat=n):
            oracle=Oracle(bytes(raw))
            for a in range(n):
                for b in range(a+1,n+1):
                    if oracle.first[a]!=oracle.first[b-1]:
                        continue
                    words,src,ret=objects(oracle,(a,b))
                    p=common(words)[:4]
                    for row in levels(words,p):
                        h=row['depth']
                        if h<2:
                            continue
                        best=row['uniform_Q']
                        for ts in product(range(h+1),repeat=b-a):
                            new=[words[t][ts[i]:]+p[:ts[ret[i]]] for i,t in enumerate(src)]
                            if b'' in new:
                                continue
                            best=min(best,suffixes(new))
                            tally['positive_assignments']+=1
                        tally['sections_and_depths']+=1
                        tally['certified']+=row['certified_for_0_to_h']
                        if row['certified_for_0_to_h']:
                            assert row['uniform_Q']==best
                        if best<row['uniform_Q']:
                            tally['mixed_better']+=1
                            if not first:
                                first=dict(last=bytes(raw).hex(),section=[a,b],h=h,uniform=row['uniform_Q'],optimum=best)
    print(json.dumps(dict(totals=tally,witness=first),indent=2))


def real(tapes):
    sys.path.insert(0,str(here.parents[8]/'docs/journal/2026-11-18-astra-c-output-first'))
    from inputs import read_input,paths
    writer=None
    for tape in paths(tapes):
        runs,_=read_input(tape)
        col=Column(runs);oracle=Oracle(b''.join(bytes([c])*n for c,n in runs))
        quote=col.interval(b'"');deep=heavy_set(col,quote,1)[0][0]
        for name,sec in (('quote',quote),('frequent',deep)):
            words,_,_=objects(oracle,sec)
            p=common(words)
            for row in levels(words,p):
                res=dict(ordinal=int(tape.name.split('.')[0]),section=name,mass=sec[1]-sec[0],
                         word_types=len(words),min_word_length=min(map(len,words)),**row)
                if writer is None:
                    writer=csv.DictWriter(sys.stdout,fieldnames=res,lineterminator='\n'); writer.writeheader()
                writer.writerow(res)
                sys.stdout.flush()


if __name__=='__main__':
    real(Path(sys.argv[1])) if len(sys.argv)>1 else small()
