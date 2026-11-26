"""A portable bit-deposit model. At runtime it stores no ordered exception clock."""
from collections import Counter
from itertools import product
import json


def deposit(source,slots):
    out=0
    while slots:
        first=slots&-slots
        if source&1:
            out|=first
        source>>=1
        slots^=first
    return out


def rank(mask,x):
    return (mask&((1<<x)-1)).bit_count()


def prepare(last,k,highest):
    n=len(last)
    psi=sorted(range(n),key=last.__getitem__)
    first=[last[v] for v in psi]
    lf=[None]*n
    for i,d in enumerate(psi):
        lf[d]=i
    contexts=[]
    for x in range(n):
        seq=[]
        for _ in range(k):
            seq.append(first[x])
            x=psi[x]
        contexts.append(tuple(seq))
    assert contexts==sorted(contexts)
    starts=[i for i in range(n) if i==0 or contexts[i]!=contexts[i-1]]+[n]
    sid=[s for s,(a,b) in enumerate(zip(starts,starts[1:])) for _ in range(a,b)]
    mask=0;exceptions=[];tables=[];entries=[]
    for s,(a,b) in enumerate(zip(starts,starts[1:])):
        modal=min(set(last[a:b]),key=lambda c:(-last[a:b].count(c),c))
        j=next(i for i in range(a,b) if last[i]==modal)
        base=lf[j]
        entries.append((base,sid[base],bytes((modal,))))
        for x in range(a,b):
            if last[x]==modal:mask|=1<<x
            else:exceptions.append((lf[x],sid[lf[x]],last[x]))
    tables.append((mask,entries))
    while len(tables)<highest.bit_length():
        prev,old=tables[-1]
        new=0;updated=[]
        for s,(a,b) in enumerate(zip(starts,starts[1:])):
            base,next_s,seq=old[s]
            bits=(prev>>a)&((1<<(b-a))-1)
            hits=bits.bit_count()
            incoming=(prev>>base)&((1<<hits)-1)
            new|=deposit(incoming,bits)<<a
            b2,ns2,second=old[next_s]
            newbase=b2+rank(prev,base)-rank(prev,starts[next_s])
            updated.append((newbase,ns2,seq+second))
        tables.append((new,updated))
    # Check each physical source at each level against independent unit stable LF.
    for power,(bits,entries) in enumerate(tables):
        for s,(a,b) in enumerate(zip(starts,starts[1:])):
            base,dest,spelling=entries[s]
            valid=[]
            for x in range(a,b):
                y=x;good=True;letters=bytearray()
                for t in range(1<<power):
                    good &= (tables[0][0]>>y)&1
                    letters.append(last[y]);y=lf[y]
                assert good==((bits>>x)&1)
                if good:
                    assert spelling==letters and dest==sid[y]
                    assert y==base+rank(bits,x)-rank(bits,a)
                    valid.append(y)
            assert valid==list(range(base,base+len(valid)))
    return (starts,sid,tables,exceptions),psi


def advance(x,s,m,model):
    starts,_,tables,exceptions=model
    bits,entries=tables[m]
    if (bits>>x)&1:
        base,next_s,letters=entries[s]
        return base+rank(bits,x)-rank(bits,starts[s]),next_s,letters
    if m:
        return None
    y,t,byte=exceptions[x-rank(bits,x)]
    return y,t,bytes((byte,))


def output(mark,model,n):
    starts,sid,tables,_=model
    x=mark;s=sid[mark];rev=bytearray()
    while len(rev)<n:
        for m in range(min((n-len(rev)).bit_length(),len(tables))-1,-1,-1):
            result=advance(x,s,m,model)
            if result is not None:
                x,s,word=result
                rev.extend(word)
                break
    plain=rev[::-1]
    if x==mark:return plain,False
    d=0;x=mark;s=sid[mark]
    while True:
        x,s,_=advance(x,s,0,model)
        d+=1
        if x==mark:break
    # Here the bzip2 horizon is the population, hence at least its marked lap.
    assert d<=n
    lap=plain[-d:]
    return (lap*((n+d-1)//d))[:n],True


def check():
    counts=Counter()
    for sigma,limit in ((2,8),(3,6)):
        for n in range(1,limit+1):
            for last in product(range(sigma),repeat=n):
                for k in (1,2,3):
                    model,psi=prepare(last,k,8)
                    counts['models']+=1
                    counts['source_level_checks']+=4*n
                    for x in range(n):
                        p=x;expected=bytearray()
                        for _ in range(n):
                            p=psi[p];expected.append(last[p])
                        got,repaired=output(x,model,n)
                        assert got==expected
                        counts['marks']+=1
                        counts['nondivisor_repairs']+=repaired
    return counts

if __name__=='__main__':print(json.dumps(check(),indent=2))
