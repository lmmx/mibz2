"""Exhaustive cap-split oracle for arbitrary marked stable byte columns."""
from collections import Counter
from itertools import product
import json

def get_caps(col):
    # Every cut between unequal symbols is mandatory. Within each equal run
    # both split and unsplit are legal whenever the small test cap permits.
    equal=[i for i in range(1,len(col)) if col[i]==col[i-1]]
    distinct={i for i in range(1,len(col)) if col[i]!=col[i-1]}
    for m in range(1<<len(equal)):
        cuts=sorted({0,len(col)}|distinct|{i for j,i in enumerate(equal) if m>>j&1})
        yield [(col[i],j-i) for i,j in zip(cuts,cuts[1:])]

def check(col,caps,p):
    alphabet=sorted(set(col))
    first=sorted(col)
    b=first[p];lo=first.index(b);hi=lo+first.count(b)
    lower=p!=lo;upper=p+1!=hi
    # Independent pointwise source labels on physical sorted ranks.
    src=[];word=[]
    for byte in alphabet:
        for width in ([p-lo,1,hi-p-1] if byte==b else [first.count(byte)]):
            if width:
                src.extend([len(word)]*width);word.append(byte)
    base={k:first.index(k) for k in alphabet};expected=[]
    for v in col:
        expected.append(src[base[v]])
        base[v]+=1
    # Only one old cap holds the unique protected image.
    h=p-lo;pos=0;z=None
    for k,w in caps:
        if k==b:
            if h<w:z=pos+h;break
            h-=w
        pos+=w
    assert z is not None
    center=src[p];starts={k:src[first.index(k)] for k in alphabet}
    got=[];extra=0;pos=0
    for k,n in caps:
        if k!=b:got.extend([starts[k]]*n)
        elif pos+n<=z:got.extend([center-1]*n);assert lower
        elif pos>z:got.extend([center+1]*n);assert upper
        else:
            a=z-pos;t=n-a-1
            got.extend([center-1]*a+[center]+[center+1]*t)
            extra+=bool(a)+bool(t)
        pos+=n
    assert got==expected and got.count(center)==1 and got[z]==center
    assert src[p]==center and word[center]==b and len(word)<=len(alphabet)+2
    return extra

def main():
    counts=Counter()
    for n in range(1,8):
        for c in product(range(3),repeat=n):
            for caps in get_caps(c):
                for mark in range(n):
                    extra=check(c,caps,mark);counts['cases']+=1
                    counts[f'cuts_{extra}']+=1
    print(json.dumps(counts,indent=2,sort_keys=True))
if __name__=='__main__':main()
