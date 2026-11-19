"""Exact discontinuity accounting for word sections and arbitrary ordered row sections."""
import itertools
import json
from collections import Counter

def lf_of(column):
    psi = sorted(range(len(column)), key=column.__getitem__)
    lf = [0] * len(psi)
    for i, x in enumerate(psi):
        lf[x] = i
    return lf, psi

def label(lf, column, mask, x):
    word = []
    while True:
        word.append(column[x])
        x = lf[x]
        if mask >> x & 1:
            return tuple(word)

def witness(column, lf, mask, expected_cylinder):
    rows = [x for x in range(len(column)) if mask >> x & 1]
    labels = [label(lf, column, mask, x) for x in rows]
    components = 1 + sum(b != a + 1 for a,b in zip(rows,rows[1:]))
    original_runs = 1 + sum(a != b for a,b in zip(column,column[1:]))
    exits = set()
    seams = 0
    for j,(left,right) in enumerate(zip(rows,rows[1:])):
        if labels[j] == labels[j+1]:
            continue
        if right != left+1:
            seams += 1
            continue
        x=left
        while True:
            if column[x] != column[x+1]:
                key=('run',x)
                break
            x=lf[x]
            m1=(mask>>x)&1
            m2=(mask>>(x+1))&1
            if m1 != m2:
                assert not expected_cylinder
                key=('cut',x)
                break
            assert not m1
        assert key not in exits
        exits.add(key)
    size = 1 + sum(a!=b for a,b in zip(labels,labels[1:]))
    assert size == 1 + seams + len(exits)
    cut_bound = sum(((mask>>i)&1) != ((mask>>(i+1))&1) for i in range(len(column)-1))
    assert size <= original_runs + components-1 + (0 if expected_cylinder else cut_bound)
    return original_runs, size, components

def main():
    totals=Counter()
    exceed=None
    interval_exceed=None
    tight_union=None
    tight=None
    for n in range(1,10):
        for ll in itertools.product(range(2),repeat=n):
            lf,psi=lf_of(ll)
            for mask in range(1,1<<n):
                r,s,k=witness(ll,lf,mask,False)
                totals['arbitrary_sections']+=1
                if s>r and exceed is None:
                    exceed=dict(column=ll,mask=mask,original_runs=r,section_runs=s,intervals=k)
                if s>r and k==1 and interval_exceed is None:
                    interval_exceed=dict(column=ll,mask=mask,original_runs=r,section_runs=s)
            keys=[() for _ in ll]
            futures=list(range(n))
            for width in range(1,5):
                groups={}
                for i,x in enumerate(futures):
                    futures[i]=psi[x]
                    keys[i]+=ll[psi[x]],
                    groups[keys[i]]=groups.get(keys[i],0)|(1<<i)
                for word,mask in groups.items():
                    r,s,k=witness(ll,lf,mask,True)
                    assert k==1 and s<=r
                    totals['nonempty_word_sections']+=1
                    if width>1 and s==r and s>=3 and tight is None:
                        tight=dict(column=ll,pattern=word,original_runs=r,section_runs=s)
                # Adjacent lexical cylinders need to remain individual groups for
                # the fixed-length proof, though their union may touch geometrically.
                if len(groups)>1:
                    for pair in itertools.combinations(groups,2):
                        mask=groups[pair[0]] | groups[pair[1]]
                        # The seam *between* even touching cylinders may separate
                        # unequal futures. Check the union bound directly.
                        rows=[x for x in range(n) if mask>>x&1]
                        words=[label(lf,ll,mask,x) for x in rows]
                        s=1+sum(a!=b for a,b in zip(words,words[1:]))
                        r=1+sum(a!=b for a,b in zip(ll,ll[1:]))
                        assert s<=r+1
                        if s==r+1 and tight_union is None:
                            tight_union=dict(column=ll,patterns=pair,original_runs=r,section_runs=s)
                        totals['two_equal_length_cylinders']+=1
    print(json.dumps(dict(totals=totals,arbitrary_counterexample=exceed,tight_word_section=tight,contiguous_counterexample=interval_exceed,tight_two_cylinder=tight_union),indent=2))

if __name__ == '__main__':
    main()
