"""Check all one-step target subsets excluding the primary, including zero-mass cuts."""
from collections import Counter
from itertools import groupby, product
import json
from cut import Fold, Word, check_step, forward, geometry, overlay, seed, split


def main():
    counts=Counter()
    for n in range(1,7):
        for classes in range(1,min(n,3)+1):
            for col in product(range(classes),repeat=n):
                if len(set(col))!=classes:
                    continue
                raw=[(k,len(list(g))) for k,g in groupby(col)]
                words=[Word(bytes([k+65])*(k+1)) for k in range(classes)]
                for mark in range(n):
                    original,weights,a=seed(raw,words,mark)
                    _,bounds=geometry(original,len(weights))
                    source=bounds[a]
                    assert source==mark
                    slices,edges=overlay(original,bounds)
                    others=[k for k in range(len(weights)) if k!=a]
                    for mask in product((False,True),repeat=len(others)):
                        target={k for k,take in zip(others,mask) if take}
                        incoming,aliases,selected,ra=split(slices,edges,weights,target,a)
                        freq,rb=geometry(incoming,len(aliases))
                        f=Fold(incoming,dict(enumerate(aliases)),list(range(len(aliases))),
                               dict(enumerate(freq)),protected=ra,chosen=selected)
                        checked,anchor=check_step(original,bounds,weights,incoming,rb,aliases,f,mark,target)
                        old_dest=forward(original,bounds[:-1])
                        entering=sum(k not in target and old_dest[d][1] in target for d,k in old_dest)
                        leaving=sum(k in target and old_dest[d][1] not in target for d,k in old_dest)
                        assert entering==leaving==n-f.m and f.bases[f.ids[ra]]==anchor
                        assert len(incoming)<=len(original)+len(weights)-1
                        assert len(f.root)<=len(incoming)+sum(k in selected for k,w in incoming)
                        counts.update(cuts=1,zero=int(entering==0),partner_checks=checked,
                                      marked_byte_offsets=weights[a].n)
    print(json.dumps(counts,indent=2))


if __name__=='__main__':
    main()
